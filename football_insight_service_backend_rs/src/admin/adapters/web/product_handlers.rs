use super::product_dto::{
    AdminProductDto, AdminProductListQuery, AdminProductPageDto, AdminProductRequestDto,
};
use crate::admin::{
    application::{
        admin_auth_service::{AdminAuthService, AdminAuthorizationError},
        admin_product_service::AdminProductService,
    },
    domain::{
        admin_auth::{AdminPermission, AdminPrincipal},
        admin_product::AdminProductError,
    },
};
use axum::{
    Json,
    extract::{
        Path, Query, State,
        rejection::{JsonRejection, QueryRejection},
    },
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
};
use std::sync::Arc;
type HttpError = (StatusCode, String);
#[derive(Clone)]
pub struct AdminProductWebState {
    pub service: Arc<AdminProductService>,
    pub auth_service: Arc<AdminAuthService>,
}
pub async fn list_products(
    State(state): State<AdminProductWebState>,
    headers: HeaderMap,
    query: Result<Query<AdminProductListQuery>, QueryRejection>,
) -> Result<Json<AdminProductPageDto>, HttpError> {
    authorize(&state, &headers).await?;
    let Query(query) =
        query.map_err(|_| (StatusCode::BAD_REQUEST, "invalid product query".into()))?;
    let query = query.try_into().map_err(map_product_error)?;
    Ok(Json(
        state
            .service
            .list(query)
            .await
            .map_err(map_product_error)?
            .into(),
    ))
}
pub async fn get_product(
    State(state): State<AdminProductWebState>,
    headers: HeaderMap,
    Path(key): Path<String>,
) -> Result<Json<AdminProductDto>, HttpError> {
    authorize(&state, &headers).await?;
    Ok(Json(
        state
            .service
            .get(&key)
            .await
            .map_err(map_product_error)?
            .into(),
    ))
}
pub async fn create_product(
    State(state): State<AdminProductWebState>,
    headers: HeaderMap,
    request: Result<Json<AdminProductRequestDto>, JsonRejection>,
) -> Result<Json<AdminProductDto>, HttpError> {
    let principal = authorize(&state, &headers).await?;
    let Json(request) =
        request.map_err(|_| (StatusCode::BAD_REQUEST, "invalid product request".into()))?;
    Ok(Json(
        state
            .service
            .create(request.into(), principal.admin_id)
            .await
            .map_err(map_product_error)?
            .into(),
    ))
}
pub async fn update_product(
    State(state): State<AdminProductWebState>,
    headers: HeaderMap,
    Path(key): Path<String>,
    request: Result<Json<AdminProductRequestDto>, JsonRejection>,
) -> Result<Json<AdminProductDto>, HttpError> {
    let principal = authorize(&state, &headers).await?;
    let Json(request) =
        request.map_err(|_| (StatusCode::BAD_REQUEST, "invalid product request".into()))?;
    Ok(Json(
        state
            .service
            .update(&key, request.into(), principal.admin_id)
            .await
            .map_err(map_product_error)?
            .into(),
    ))
}
pub async fn delete_product(
    State(state): State<AdminProductWebState>,
    headers: HeaderMap,
    Path(key): Path<String>,
) -> Result<StatusCode, HttpError> {
    let principal = authorize(&state, &headers).await?;
    state
        .service
        .delete(&key, principal.admin_id)
        .await
        .map_err(map_product_error)?;
    Ok(StatusCode::NO_CONTENT)
}
async fn authorize(
    state: &AdminProductWebState,
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
        .authorize_checked(token, AdminPermission::ManageProducts)
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
fn map_product_error(error: AdminProductError) -> HttpError {
    let status = match error {
        AdminProductError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        AdminProductError::NotFound => StatusCode::NOT_FOUND,
        AdminProductError::Conflict => StatusCode::CONFLICT,
        AdminProductError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (status, error.to_string())
}

pub async fn product_purchases(
    State(state): State<AdminProductWebState>,
    headers: HeaderMap,
    Path(key): Path<String>,
    query: Result<Query<super::product_dto::AdminPurchaseQuery>, QueryRejection>,
) -> Result<Json<super::product_dto::ProductPurchasePageDto>, HttpError> {
    authorize(&state, &headers).await?;
    let Query(query) =
        query.map_err(|_| (StatusCode::BAD_REQUEST, "invalid purchases query".into()))?;
    Ok(Json(
        state
            .service
            .purchases(&key, query.page.unwrap_or(1), query.page_size.unwrap_or(20))
            .await
            .map_err(map_product_error)?
            .into(),
    ))
}
pub async fn product_no_store(mut response: axum::response::Response) -> axum::response::Response {
    response.headers_mut().insert(
        axum::http::header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("no-store"),
    );
    response
}
