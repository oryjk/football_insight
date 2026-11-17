use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State, rejection::JsonRejection, rejection::QueryRejection},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::admin::{
    application::{
        admin_auth_service::{AdminAuthService, AdminAuthorizationError},
        admin_telegraph_comments_service::AdminTelegraphCommentsService,
    },
    domain::{
        admin_auth::{AdminPermission, AdminPrincipal},
        admin_telegraph_comments::AdminTelegraphCommentsError,
    },
};

type HttpError = (StatusCode, String);

#[derive(Clone)]
pub struct AdminTelegraphCommentsWebState {
    pub service: Arc<AdminTelegraphCommentsService>,
    pub auth_service: Arc<AdminAuthService>,
}

#[derive(Deserialize)]
pub struct ListCommentsQuery {
    pub status: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Deserialize)]
pub struct SetCommentsConfigBody {
    pub comments_enabled: bool,
}

#[derive(Deserialize)]
pub struct SetCommentStatusBody {
    pub status: String,
}

pub async fn list_comments(
    State(state): State<AdminTelegraphCommentsWebState>,
    headers: HeaderMap,
    query: Result<Query<ListCommentsQuery>, QueryRejection>,
) -> Result<Json<Value>, HttpError> {
    authorize(&state, &headers).await?;
    let Query(query) =
        query.map_err(|_| (StatusCode::BAD_REQUEST, "invalid query parameters".to_string()))?;
    let view = state
        .service
        .list(query.status.as_deref(), query.limit)
        .await
        .map_err(map_error)?;
    Ok(Json(json!({
        "comments_enabled": view.comments_enabled,
        "items": view.items.into_iter().map(|item| json!({
            "id": item.id,
            "team_id": item.team_id,
            "topic_id": item.topic_id,
            "entry_id": item.entry_id,
            "entry_title": item.entry_title,
            "user_id": item.user_id,
            "author_display_name": item.author_display_name,
            "author_avatar_url": item.author_avatar_url,
            "content": item.content,
            "status": item.status,
            "created_at": item.created_at,
        })).collect::<Vec<_>>(),
    })))
}

pub async fn set_comments_config(
    State(state): State<AdminTelegraphCommentsWebState>,
    headers: HeaderMap,
    body: Result<Json<SetCommentsConfigBody>, JsonRejection>,
) -> Result<Json<Value>, HttpError> {
    let principal = authorize(&state, &headers).await?;
    let Json(body) =
        body.map_err(|_| (StatusCode::BAD_REQUEST, "invalid request body".to_string()))?;
    let enabled = state
        .service
        .set_comments_enabled(body.comments_enabled, principal.admin_id)
        .await
        .map_err(map_error)?;
    Ok(Json(json!({ "comments_enabled": enabled })))
}

pub async fn set_comment_status(
    State(state): State<AdminTelegraphCommentsWebState>,
    headers: HeaderMap,
    Path(comment_id): Path<Uuid>,
    body: Result<Json<SetCommentStatusBody>, JsonRejection>,
) -> Result<StatusCode, HttpError> {
    let principal = authorize(&state, &headers).await?;
    let Json(body) =
        body.map_err(|_| (StatusCode::BAD_REQUEST, "invalid request body".to_string()))?;
    state
        .service
        .set_comment_status(comment_id, &body.status, principal.admin_id)
        .await
        .map_err(map_error)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn authorize(
    state: &AdminTelegraphCommentsWebState,
    headers: &HeaderMap,
) -> Result<AdminPrincipal, HttpError> {
    let token = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or((StatusCode::UNAUTHORIZED, "admin login required".into()))?;
    state
        .auth_service
        .authorize_checked(token, AdminPermission::ManageUsers)
        .await
        .map_err(|error| {
            let status = match error {
                AdminAuthorizationError::Unauthorized => StatusCode::UNAUTHORIZED,
                AdminAuthorizationError::Forbidden => StatusCode::FORBIDDEN,
                AdminAuthorizationError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, error.to_string())
        })
}

fn map_error(error: AdminTelegraphCommentsError) -> HttpError {
    let status = match error {
        AdminTelegraphCommentsError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        AdminTelegraphCommentsError::NotFound => StatusCode::NOT_FOUND,
        AdminTelegraphCommentsError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (status, error.to_string())
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use chrono::{Duration, Utc};
    use serde_json::{Value, json};
    use tower::ServiceExt;
    use uuid::Uuid;

    use crate::admin::{
        adapters::{
            security::{
                jwt_admin_token_port::JwtAdminTokenPort,
                role_based_admin_authorization::RoleBasedAdminAuthorization,
            },
            web::telegraph_comments_routes::admin_telegraph_comments_routes,
        },
        application::{
            admin_auth_service::AdminAuthService,
            admin_telegraph_comments_service::AdminTelegraphCommentsService,
        },
        domain::{
            admin_auth::{AdminAccount, AdminSession},
            admin_telegraph_comments::{
                AdminTelegraphComment, AdminTelegraphCommentStatus,
                AdminTelegraphCommentStatusFilter, AdminTelegraphCommentsError,
                AdminTelegraphCommentsView,
            },
        },
        ports::{
            admin_auth_repository::AdminAuthRepository,
            admin_telegraph_comments_repository::AdminTelegraphCommentsRepository,
            admin_token_port::AdminTokenPort,
        },
    };
    use crate::auth::ports::password_port::PasswordPort;

    struct FakeCommentsRepo {
        enabled: Mutex<bool>,
        items: Mutex<Vec<AdminTelegraphComment>>,
    }

    #[async_trait]
    impl AdminTelegraphCommentsRepository for FakeCommentsRepo {
        async fn list(
            &self,
            filter: AdminTelegraphCommentStatusFilter,
            limit: usize,
        ) -> Result<AdminTelegraphCommentsView, AdminTelegraphCommentsError> {
            let items = self
                .items
                .lock()
                .unwrap()
                .iter()
                .filter(|item| match filter {
                    AdminTelegraphCommentStatusFilter::All => item.status != "deleted",
                    AdminTelegraphCommentStatusFilter::Visible => item.status == "visible",
                    AdminTelegraphCommentStatusFilter::Hidden => item.status == "hidden",
                })
                .take(limit)
                .cloned()
                .collect();
            Ok(AdminTelegraphCommentsView {
                comments_enabled: *self.enabled.lock().unwrap(),
                items,
            })
        }

        async fn set_comments_enabled(
            &self,
            enabled: bool,
            _admin_id: Uuid,
        ) -> Result<bool, AdminTelegraphCommentsError> {
            *self.enabled.lock().unwrap() = enabled;
            Ok(enabled)
        }

        async fn set_comment_status(
            &self,
            comment_id: Uuid,
            status: AdminTelegraphCommentStatus,
            _admin_id: Uuid,
        ) -> Result<(), AdminTelegraphCommentsError> {
            let mut items = self.items.lock().unwrap();
            let Some(item) = items.iter_mut().find(|c| c.id == comment_id) else {
                return Err(AdminTelegraphCommentsError::NotFound);
            };
            item.status = status.as_str().to_string();
            Ok(())
        }
    }

    struct FakeAdminRepo {
        account: AdminAccount,
        session_id: Uuid,
    }

    #[async_trait]
    impl AdminAuthRepository for FakeAdminRepo {
        async fn find_by_username(&self, _: &str) -> anyhow::Result<Option<AdminAccount>> {
            Ok(None)
        }
        async fn create_session(&self, _: AdminSession) -> anyhow::Result<()> {
            Ok(())
        }
        async fn find_active_account_for_session(
            &self,
            id: Uuid,
            session: Uuid,
            _: chrono::DateTime<Utc>,
        ) -> anyhow::Result<Option<AdminAccount>> {
            Ok((id == self.account.id && session == self.session_id)
                .then(|| self.account.clone()))
        }
        async fn revoke_session(
            &self,
            _: Uuid,
            _: Uuid,
            _: chrono::DateTime<Utc>,
        ) -> anyhow::Result<bool> {
            Ok(false)
        }
    }

    struct FakePassword;
    impl PasswordPort for FakePassword {
        fn hash_password(&self, _: &str) -> anyhow::Result<String> {
            Ok("h".into())
        }
        fn verify_password(&self, _: &str, _: &str) -> anyhow::Result<bool> {
            Ok(true)
        }
    }

    #[tokio::test]
    async fn admin_telegraph_comments_routes_require_auth_and_update_config_and_status() {
        let admin_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let comment_id = Uuid::new_v4();
        let account = AdminAccount {
            id: admin_id,
            username: "owner".into(),
            password_hash: "h".into(),
            display_name: "Owner".into(),
            role: "owner".into(),
            status: "active".into(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let token_port = Arc::new(JwtAdminTokenPort::new("admin-secret-at-least-32-chars-long".into()));
        let token = token_port
            .issue_token(
                admin_id,
                session_id,
                "owner",
                "owner",
                Utc::now() + Duration::hours(1),
            )
            .unwrap();
        let auth_service = Arc::new(AdminAuthService::new(
            Arc::new(FakeAdminRepo {
                account,
                session_id,
            }),
            Arc::new(FakePassword),
            token_port,
            Arc::new(RoleBasedAdminAuthorization),
            Duration::hours(1),
            Arc::new(Utc::now),
        ));
        let repo = Arc::new(FakeCommentsRepo {
            enabled: Mutex::new(true),
            items: Mutex::new(vec![AdminTelegraphComment {
                id: comment_id,
                team_id: 77680,
                topic_id: "topic-1".into(),
                entry_id: "entry-1".into(),
                entry_title: "国安热身赛获胜".into(),
                user_id: Uuid::new_v4(),
                author_display_name: "蓉城铁粉".into(),
                author_avatar_url: None,
                content: "好球！".into(),
                status: "visible".into(),
                created_at: Utc::now(),
            }]),
        });
        let service = Arc::new(AdminTelegraphCommentsService::new(repo));
        let app = admin_telegraph_comments_routes(service, auth_service);

        let unauth = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/admin/telegraph-comments")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(unauth.status(), StatusCode::UNAUTHORIZED);

        let list_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/admin/telegraph-comments")
                    .header("Authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(list_resp.status(), StatusCode::OK);
        let bytes = to_bytes(list_resp.into_body(), 100_000).await.unwrap();
        let body: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["comments_enabled"], true);
        assert_eq!(body["items"].as_array().unwrap().len(), 1);

        let cfg_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/api/v1/admin/telegraph-comments/config")
                    .header("Authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(json!({ "comments_enabled": false }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(cfg_resp.status(), StatusCode::OK);

        let status_resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/api/v1/admin/telegraph-comments/{comment_id}/status"))
                    .header("Authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(json!({ "status": "hidden" }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(status_resp.status(), StatusCode::NO_CONTENT);

        let hidden_resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/admin/telegraph-comments?status=hidden")
                    .header("Authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let hidden_bytes = to_bytes(hidden_resp.into_body(), 100_000).await.unwrap();
        let hidden_body: Value = serde_json::from_slice(&hidden_bytes).unwrap();
        assert_eq!(hidden_body["comments_enabled"], false);
        assert_eq!(hidden_body["items"].as_array().unwrap().len(), 1);
    }
}
