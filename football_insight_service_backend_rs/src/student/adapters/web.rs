use crate::{
    admin::{
        application::admin_auth_service::{AdminAuthService, AdminAuthorizationError},
        domain::admin_auth::AdminPermission,
    },
    auth::ports::token_port::TokenPort,
    payment::adapters::payment_params::PaymentParams,
    student::{
        application::{StudentConfigService, StudentService},
        domain::{ServiceCode, StudentConfig, StudentError, StudentService as ServiceConfig},
    },
};
use axum::{
    Json, Router,
    extract::{Query, State},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    routing::get,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;
use uuid::Uuid;
type HttpError = (StatusCode, String);
#[derive(Clone)]
pub struct StudentWebState {
    pub service: Arc<StudentService>,
    pub tokens: Arc<dyn TokenPort>,
}
#[derive(Clone)]
pub struct AdminStudentWebState {
    pub service: Arc<StudentConfigService>,
    pub auth: Arc<AdminAuthService>,
}

pub fn routes(state: StudentWebState) -> Router {
    Router::new()
        .route("/api/v1/student/config", get(config))
        .route("/api/v1/student/orders", get(orders).post(create_order))
        .with_state(state)
        .layer(axum::middleware::map_response(no_store))
}
pub fn admin_routes(state: AdminStudentWebState) -> Router {
    Router::new()
        .route(
            "/api/v1/admin/student/config",
            get(admin_config).put(save_config),
        )
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
fn token(headers: &HeaderMap) -> Result<&str, HttpError> {
    headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .filter(|v| !v.trim().is_empty())
        .ok_or((StatusCode::UNAUTHORIZED, "请先登录".into()))
}
fn user(state: &StudentWebState, headers: &HeaderMap) -> Result<Uuid, HttpError> {
    state
        .tokens
        .verify_token(token(headers)?)
        .map(|claims| claims.sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "请先登录".into()))
}
async fn admin(state: &AdminStudentWebState, headers: &HeaderMap) -> Result<Uuid, HttpError> {
    state
        .auth
        .authorize_checked(token(headers)?, AdminPermission::ManageProducts)
        .await
        .map(|p| p.admin_id)
        .map_err(|e| {
            let status = match e {
                AdminAuthorizationError::Unauthorized => StatusCode::UNAUTHORIZED,
                AdminAuthorizationError::Forbidden => StatusCode::FORBIDDEN,
                AdminAuthorizationError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, e.to_string())
        })
}
fn config_json(config: StudentConfig, private: bool) -> Value {
    json!({"contact_qr_url":config.contact_qr_url,"services":config.services.into_iter().map(|s| {
        let mut value = json!({"code":s.code,"title":s.title,"description":s.description,"price_cents":s.price_cents,"unit":s.unit,"enabled":s.enabled});
        if private { value["product_id"] = json!(s.product_id); } value
    }).collect::<Vec<_>>()})
}
async fn config(
    State(state): State<StudentWebState>,
    headers: HeaderMap,
) -> Result<Json<Value>, HttpError> {
    user(&state, &headers)?;
    Ok(Json(config_json(
        state.service.config().await.map_err(map_error)?,
        false,
    )))
}
async fn admin_config(
    State(state): State<AdminStudentWebState>,
    headers: HeaderMap,
) -> Result<Json<Value>, HttpError> {
    admin(&state, &headers).await?;
    Ok(Json(config_json(
        state.service.config().await.map_err(map_error)?,
        true,
    )))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigInput {
    contact_qr_url: String,
    services: Vec<ServiceInput>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ServiceInput {
    code: ServiceCode,
    title: String,
    description: String,
    price_cents: i32,
    unit: String,
    enabled: bool,
    product_id: String,
}
impl From<ConfigInput> for StudentConfig {
    fn from(input: ConfigInput) -> Self {
        Self {
            contact_qr_url: input.contact_qr_url,
            services: input
                .services
                .into_iter()
                .map(|s| ServiceConfig {
                    code: s.code,
                    title: s.title,
                    description: s.description,
                    price_cents: s.price_cents,
                    unit: s.unit,
                    enabled: s.enabled,
                    product_id: s.product_id,
                })
                .collect(),
        }
    }
}
async fn save_config(
    State(state): State<AdminStudentWebState>,
    headers: HeaderMap,
    body: Result<Json<ConfigInput>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<Value>, HttpError> {
    let admin_id = admin(&state, &headers).await?;
    let Json(input) = body.map_err(|_| (StatusCode::BAD_REQUEST, "配置格式无效".into()))?;
    let config = state
        .service
        .save(input.into(), admin_id)
        .await
        .map_err(map_error)?;
    Ok(Json(config_json(config, true)))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateOrderInput {
    code: ServiceCode,
}
async fn create_order(
    State(state): State<StudentWebState>,
    headers: HeaderMap,
    Json(input): Json<CreateOrderInput>,
) -> Result<Json<Value>, HttpError> {
    let user_id = user(&state, &headers)?;
    let checkout = state
        .service
        .create_order(user_id, input.code)
        .await
        .map_err(map_error)?;
    Ok(Json(
        json!({"order_no":checkout.order_no,"params":PaymentParams::from(checkout.params)}),
    ))
}
#[derive(Deserialize)]
struct PageInput {
    #[serde(default = "first_page")]
    page: i64,
    #[serde(default = "page_size")]
    page_size: i64,
}
fn first_page() -> i64 {
    1
}
fn page_size() -> i64 {
    20
}
async fn orders(
    State(state): State<StudentWebState>,
    headers: HeaderMap,
    Query(query): Query<PageInput>,
) -> Result<Json<Value>, HttpError> {
    let user_id = user(&state, &headers)?;
    let page = state
        .service
        .orders(user_id, query.page, query.page_size)
        .await
        .map_err(map_error)?;
    Ok(Json(
        json!({"items":page.items.into_iter().map(|o| json!({"order_no":o.order_no,"code":o.code,"title":o.code.title(),"amount_cents":o.amount_cents,"status":o.status,"paid_at":o.paid_at,"created_at":o.created_at})).collect::<Vec<_>>(),"total":page.total,"page":page.page,"page_size":page.page_size}),
    ))
}
fn map_error(error: StudentError) -> HttpError {
    let code = match error {
        StudentError::InvalidInput(_) | StudentError::WechatRequired => StatusCode::BAD_REQUEST,
        StudentError::Unavailable => StatusCode::CONFLICT,
        StudentError::PaymentFailed => StatusCode::BAD_GATEWAY,
        StudentError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (code, error.to_string())
}
