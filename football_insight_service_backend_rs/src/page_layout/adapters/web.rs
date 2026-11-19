use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Path, State, rejection::JsonRejection},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    admin::{
        application::admin_auth_service::{AdminAuthService, AdminAuthorizationError},
        domain::admin_auth::AdminPermission,
    },
    page_layout::{
        application::PageLayoutService,
        domain::{PageLayoutBlock, PageLayoutError, SavePageLayoutInput},
    },
};

type HttpError = (StatusCode, String);

const UPLOAD_BODY_LIMIT_BYTES: usize = 8 * 1024 * 1024;

#[derive(Clone)]
pub struct PublicPageLayoutWebState {
    pub service: Arc<PageLayoutService>,
}

#[derive(Clone)]
pub struct AdminPageLayoutWebState {
    pub service: Arc<PageLayoutService>,
    pub auth_service: Arc<AdminAuthService>,
}

pub fn public_routes(state: PublicPageLayoutWebState) -> Router {
    Router::new()
        .route("/api/v1/page-layouts", get(list_public_layouts))
        .route("/api/v1/page-layouts/{page_key}", get(get_public_layout))
        .with_state(state)
        .layer(axum::middleware::map_response(no_store))
}

pub fn admin_routes(state: AdminPageLayoutWebState) -> Router {
    let upload_route = Router::new()
        .route(
            "/api/v1/admin/page-layouts/upload-image",
            post(upload_layout_image),
        )
        .layer(DefaultBodyLimit::max(UPLOAD_BODY_LIMIT_BYTES));

    Router::new()
        .route("/api/v1/admin/page-layouts", get(list_admin_layouts))
        .route(
            "/api/v1/admin/page-layouts/{page_key}",
            axum::routing::put(save_admin_layout).delete(reset_admin_layout),
        )
        .merge(upload_route)
        .with_state(state)
        .layer(axum::middleware::map_response(no_store))
}

async fn no_store(mut response: axum::response::Response) -> axum::response::Response {
    response.headers_mut().insert(
        axum::http::header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("no-store"),
    );
    response
}

async fn authorize_admin(
    state: &AdminPageLayoutWebState,
    headers: &HeaderMap,
) -> Result<Uuid, HttpError> {
    let token = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or((StatusCode::UNAUTHORIZED, "admin login required".into()))?;
    state
        .auth_service
        .authorize_checked(token, AdminPermission::ManageProducts)
        .await
        .map(|principal| principal.admin_id)
        .map_err(|error| {
            let status = match error {
                AdminAuthorizationError::Unauthorized => StatusCode::UNAUTHORIZED,
                AdminAuthorizationError::Forbidden => StatusCode::FORBIDDEN,
                AdminAuthorizationError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, error.to_string())
        })
}

fn map_error(error: PageLayoutError) -> HttpError {
    let status = match error {
        PageLayoutError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        PageLayoutError::NotFound => StatusCode::NOT_FOUND,
        PageLayoutError::ImageStorageUnavailable => StatusCode::SERVICE_UNAVAILABLE,
        PageLayoutError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (status, error.to_string())
}

async fn list_public_layouts(
    State(state): State<PublicPageLayoutWebState>,
) -> Result<Json<Value>, HttpError> {
    let items = state.service.list_public_layouts().await.map_err(map_error)?;
    Ok(Json(json!({ "items": items })))
}

async fn get_public_layout(
    State(state): State<PublicPageLayoutWebState>,
    Path(page_key): Path<String>,
) -> Result<Json<Value>, HttpError> {
    let item = state
        .service
        .get_public_layout(&page_key)
        .await
        .map_err(map_error)?;
    Ok(Json(json!(item)))
}

async fn list_admin_layouts(
    State(state): State<AdminPageLayoutWebState>,
    headers: HeaderMap,
) -> Result<Json<Value>, HttpError> {
    authorize_admin(&state, &headers).await?;
    let items = state.service.list_admin_layouts().await.map_err(map_error)?;
    Ok(Json(json!({ "items": items })))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SavePageLayoutBody {
    #[serde(default)]
    background_image_url: Option<String>,
    blocks: Vec<PageLayoutBlock>,
}

async fn save_admin_layout(
    State(state): State<AdminPageLayoutWebState>,
    headers: HeaderMap,
    Path(page_key): Path<String>,
    body: Result<Json<SavePageLayoutBody>, JsonRejection>,
) -> Result<Json<Value>, HttpError> {
    let admin_id = authorize_admin(&state, &headers).await?;
    let Json(body) =
        body.map_err(|_| (StatusCode::BAD_REQUEST, "invalid request body".to_string()))?;
    let saved = state
        .service
        .save_layout(
            &page_key,
            SavePageLayoutInput {
                background_image_url: body.background_image_url,
                blocks: body.blocks,
            },
            admin_id,
        )
        .await
        .map_err(map_error)?;
    Ok(Json(json!(saved)))
}

async fn reset_admin_layout(
    State(state): State<AdminPageLayoutWebState>,
    headers: HeaderMap,
    Path(page_key): Path<String>,
) -> Result<Json<Value>, HttpError> {
    let admin_id = authorize_admin(&state, &headers).await?;
    let reset = state
        .service
        .reset_layout(&page_key, admin_id)
        .await
        .map_err(map_error)?;
    Ok(Json(json!(reset)))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UploadLayoutImageBody {
    image_data_url: String,
}

async fn upload_layout_image(
    State(state): State<AdminPageLayoutWebState>,
    headers: HeaderMap,
    body: Result<Json<UploadLayoutImageBody>, JsonRejection>,
) -> Result<Json<Value>, HttpError> {
    authorize_admin(&state, &headers).await?;
    let Json(body) =
        body.map_err(|_| (StatusCode::BAD_REQUEST, "invalid request body".to_string()))?;
    let url = state
        .service
        .upload_image(&body.image_data_url)
        .await
        .map_err(map_error)?;
    Ok(Json(json!({ "url": url })))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use async_trait::async_trait;
    use axum::{body::Body, http::Request};
    use base64::{Engine, engine::general_purpose::STANDARD};
    use chrono::{Duration, Utc};
    use tower::ServiceExt;

    use super::*;
    use crate::{
        admin::{
            adapters::security::{
                jwt_admin_token_port::JwtAdminTokenPort,
                role_based_admin_authorization::RoleBasedAdminAuthorization,
            },
            domain::admin_auth::{
                AdminAccount, AdminBootstrapOwner, AdminSession,
            },
            ports::{
                admin_auth_repository::AdminAuthRepository,
                admin_token_port::AdminTokenPort,
            },
        },
        auth::adapters::security::argon2_password_port::Argon2PasswordPort,
        page_layout::{
            domain::{ParsedLayoutImage, StoredPageLayout, ValidatedPageLayoutWrite},
            ports::{LayoutImageStoragePort, PageLayoutRepository},
        },
    };

    #[derive(Default)]
    struct InMemoryPageLayoutRepo {
        items: Mutex<HashMap<String, StoredPageLayout>>,
    }

    #[async_trait]
    impl PageLayoutRepository for InMemoryPageLayoutRepo {
        async fn list_all(&self) -> Result<Vec<StoredPageLayout>, PageLayoutError> {
            Ok(self.items.lock().unwrap().values().cloned().collect())
        }

        async fn get_by_page_key(
            &self,
            page_key: &str,
        ) -> Result<Option<StoredPageLayout>, PageLayoutError> {
            Ok(self.items.lock().unwrap().get(page_key).cloned())
        }

        async fn upsert(
            &self,
            write: ValidatedPageLayoutWrite,
            _admin_id: Uuid,
        ) -> Result<StoredPageLayout, PageLayoutError> {
            let mut guard = self.items.lock().unwrap();
            let next_version = guard
                .get(&write.page_key)
                .map(|v| v.version + 1)
                .unwrap_or(1);
            let stored = StoredPageLayout {
                page_key: write.page_key.clone(),
                background_image_url: write.background_image_url,
                blocks: write.blocks,
                version: next_version,
                updated_at: Utc::now(),
            };
            guard.insert(write.page_key, stored.clone());
            Ok(stored)
        }

        async fn reset_to_default(
            &self,
            page_key: &str,
            _admin_id: Uuid,
        ) -> Result<(), PageLayoutError> {
            self.items.lock().unwrap().remove(page_key);
            Ok(())
        }
    }

    struct FakeImageStorage {
        available: bool,
    }

    #[async_trait]
    impl LayoutImageStoragePort for FakeImageStorage {
        async fn upload_image(&self, image: ParsedLayoutImage) -> Result<String, PageLayoutError> {
            if !self.available {
                return Err(PageLayoutError::ImageStorageUnavailable);
            }
            Ok(format!("https://oryjk.cn:82/minio/football-insight/page-layouts/test.{}", image.ext))
        }
    }

    struct FakeAdminAuthRepo {
        account: AdminAccount,
        session_id: Uuid,
    }

    #[async_trait]
    impl AdminAuthRepository for FakeAdminAuthRepo {
        async fn find_by_username(&self, _: &str) -> anyhow::Result<Option<AdminAccount>> {
            Ok(Some(self.account.clone()))
        }
        async fn create_session(&self, _: AdminSession) -> anyhow::Result<()> {
            Ok(())
        }
        async fn find_active_account_for_session(
            &self,
            admin_id: Uuid,
            session_id: Uuid,
            _: chrono::DateTime<Utc>,
        ) -> anyhow::Result<Option<AdminAccount>> {
            if admin_id == self.account.id && session_id == self.session_id {
                Ok(Some(self.account.clone()))
            } else {
                Ok(None)
            }
        }
        async fn revoke_session(
            &self,
            _: Uuid,
            _: Uuid,
            _: chrono::DateTime<Utc>,
        ) -> anyhow::Result<bool> {
            Ok(true)
        }
        async fn ensure_owner(&self, _: AdminBootstrapOwner) -> anyhow::Result<AdminAccount> {
            Ok(self.account.clone())
        }
    }

    fn build_test_app(storage_available: bool) -> (Router, String) {
        let admin_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let account = AdminAccount {
            id: admin_id,
            username: "superadmin".into(),
            password_hash: "hash".into(),
            display_name: "Super Admin".into(),
            role: "owner".into(),
            status: "active".into(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let token_port = Arc::new(JwtAdminTokenPort::new("0123456789abcdef0123456789abcdef".into()));
        let token = token_port
            .issue_token(
                admin_id,
                session_id,
                "superadmin",
                "owner",
                Utc::now() + Duration::hours(1),
            )
            .unwrap();
        let auth_service = Arc::new(AdminAuthService::new(
            Arc::new(FakeAdminAuthRepo {
                account,
                session_id,
            }),
            Arc::new(Argon2PasswordPort),
            token_port,
            Arc::new(RoleBasedAdminAuthorization),
            Duration::hours(12),
            Arc::new(Utc::now),
        ));
        let service = Arc::new(PageLayoutService::new(
            Arc::new(InMemoryPageLayoutRepo::default()),
            Arc::new(FakeImageStorage {
                available: storage_available,
            }),
        ));
        let app = public_routes(PublicPageLayoutWebState {
            service: service.clone(),
        })
        .merge(admin_routes(AdminPageLayoutWebState {
            service,
            auth_service,
        }));
        (app, token)
    }

    #[tokio::test]
    async fn admin_can_save_reset_and_public_reads_updated_layout() {
        let (app, token) = build_test_app(true);

        // Unauthorized admin request fails with 401.
        let unauth = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/admin/page-layouts")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(unauth.status(), StatusCode::UNAUTHORIZED);

        // Save custom order & backgrounds for home.
        let save_body = json!({
            "background_image_url": "https://oryjk.cn:82/minio/football-insight/bg.webp",
            "blocks": [
                { "key": "my_team_support", "visible": true, "background_image_url": "https://oryjk.cn:82/minio/football-insight/card.png" },
                { "key": "daily_topics", "visible": false, "background_image_url": null }
            ]
        });
        let save_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/api/v1/admin/page-layouts/home")
                    .header(AUTHORIZATION, format!("Bearer {token}"))
                    .header("Content-Type", "application/json")
                    .body(Body::from(save_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(save_resp.status(), StatusCode::OK);

        // Public read reflects saved layout.
        let public_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/page-layouts/home")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(public_resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(public_resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let payload: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(
            payload["background_image_url"],
            "https://oryjk.cn:82/minio/football-insight/bg.webp"
        );
        assert_eq!(payload["blocks"][0]["key"], "my_team_support");
        assert_eq!(payload["blocks"][1]["key"], "daily_topics");
        assert_eq!(payload["blocks"][1]["visible"], false);

        // Reset restores defaults.
        let reset_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/api/v1/admin/page-layouts/home")
                    .header(AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(reset_resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(reset_resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let reset_payload: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(reset_payload["is_customized"], false);
        assert_eq!(reset_payload["blocks"][0]["key"], "daily_topics");
    }

    #[tokio::test]
    async fn upload_image_validates_and_returns_url() {
        let (app, token) = build_test_app(true);
        let png_bytes = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR";
        let body = json!({
            "image_data_url": format!("data:image/png;base64,{}", STANDARD.encode(png_bytes))
        });
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/admin/page-layouts/upload-image")
                    .header(AUTHORIZATION, format!("Bearer {token}"))
                    .header("Content-Type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let payload: Value = serde_json::from_slice(&bytes).unwrap();
        assert!(payload["url"].as_str().unwrap().ends_with(".png"));
    }
}
