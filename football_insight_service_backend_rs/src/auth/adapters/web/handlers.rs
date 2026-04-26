use std::sync::Arc;

use axum::{
    extract::Json,
    http::StatusCode,
    http::{HeaderMap, header::AUTHORIZATION},
    response::{IntoResponse, Redirect},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use reqwest::Url;
use serde::Deserialize;

use crate::auth::{
    adapters::web::dto::{AuthResponseDto, CurrentUserDto, MiniWechatLoginResponseDto},
    application::{
        bind_wechat_account::{BindWechatAccountInput, BindWechatAccountUseCase},
        bind_wechat_mini_program_account::{
            BindWechatMiniProgramAccountInput, BindWechatMiniProgramAccountUseCase,
        },
        get_current_user::GetCurrentUserUseCase,
        login_with_mini_wechat::{CompleteMiniWechatLoginInput, CompleteMiniWechatLoginUseCase},
        login_with_password::{LoginInput, LoginWithPasswordUseCase},
        login_with_wechat::{
            CompleteWechatLoginInput, CompleteWechatLoginUseCase, WechatLoginResult,
        },
        logout::LogoutUseCase,
        register_with_invite::{RegisterInput, RegisterWithInviteUseCase},
        reset_password_with_invite::{ResetPasswordInput, ResetPasswordWithInviteUseCase},
    },
};

#[derive(Clone)]
pub struct AuthWebConfig {
    pub wechat_app_id: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub invite_code: String,
    pub referral_code: Option<String>,
    #[serde(alias = "phone_number")]
    pub account_identifier: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    #[serde(alias = "phone_number")]
    pub account_identifier: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub invite_code: String,
    #[serde(alias = "phone_number")]
    pub account_identifier: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct WechatBindRequest {
    pub bind_token: String,
    pub invite_code: Option<String>,
    pub referral_code: Option<String>,
    pub phone_number: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct MiniWechatLoginRequest {
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct MiniWechatBindRequest {
    pub bind_token: String,
    pub invite_code: String,
    pub referral_code: Option<String>,
    pub display_name: String,
    pub avatar_data_url: String,
}

#[derive(Debug, Deserialize)]
pub struct WechatCallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
}

pub async fn register_handler(
    Json(request): Json<RegisterRequest>,
    use_case: Arc<RegisterWithInviteUseCase>,
) -> Result<(StatusCode, Json<AuthResponseDto>), (StatusCode, String)> {
    let result = use_case
        .execute(RegisterInput {
            invite_code: request.invite_code,
            referral_code: request.referral_code,
            account_identifier: request.account_identifier,
            password: request.password,
        })
        .await
        .map_err(map_auth_error)?;

    Ok((StatusCode::CREATED, Json(result.into())))
}

pub async fn login_handler(
    Json(request): Json<LoginRequest>,
    use_case: Arc<LoginWithPasswordUseCase>,
) -> Result<Json<AuthResponseDto>, (StatusCode, String)> {
    let result = use_case
        .execute(LoginInput {
            account_identifier: request.account_identifier,
            password: request.password,
        })
        .await
        .map_err(map_auth_error)?;

    Ok(Json(result.into()))
}

pub async fn reset_password_handler(
    Json(request): Json<ResetPasswordRequest>,
    use_case: Arc<ResetPasswordWithInviteUseCase>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    use_case
        .execute(ResetPasswordInput {
            invite_code: request.invite_code,
            account_identifier: request.account_identifier,
            password: request.password,
        })
        .await
        .map_err(map_auth_error)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_current_user_handler(
    headers: HeaderMap,
    use_case: Arc<GetCurrentUserUseCase>,
) -> Result<Json<CurrentUserDto>, (StatusCode, String)> {
    let token = extract_bearer_token(&headers)
        .ok_or((StatusCode::UNAUTHORIZED, "not logged in".to_string()))?;

    let user = use_case
        .execute(token)
        .await
        .map_err(|error| (StatusCode::UNAUTHORIZED, error.to_string()))?
        .ok_or((StatusCode::UNAUTHORIZED, "not logged in".to_string()))?;

    Ok(Json(user.into()))
}

pub async fn logout_handler(
    use_case: Arc<LogoutUseCase>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    use_case
        .execute()
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn wechat_authorize_handler(
    headers: HeaderMap,
    config: Arc<AuthWebConfig>,
) -> Result<Redirect, (StatusCode, String)> {
    let callback_url = build_callback_url(&headers)?;
    let return_to = build_frontend_user_url(&headers)?;
    let state = URL_SAFE_NO_PAD.encode(return_to.as_bytes());

    let authorize_url = Url::parse_with_params(
        "https://open.weixin.qq.com/connect/oauth2/authorize",
        &[
            ("appid", config.wechat_app_id.as_str()),
            ("redirect_uri", callback_url.as_str()),
            ("response_type", "code"),
            ("scope", "snsapi_base"),
            ("state", state.as_str()),
        ],
    )
    .map_err(|error| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to build wechat authorize url: {error}"),
        )
    })?;

    Ok(Redirect::temporary(&format!(
        "{authorize_url}#wechat_redirect"
    )))
}

pub async fn wechat_callback_handler(
    headers: HeaderMap,
    axum::extract::Query(query): axum::extract::Query<WechatCallbackQuery>,
    use_case: Arc<CompleteWechatLoginUseCase>,
) -> Result<Redirect, (StatusCode, String)> {
    let code = query.code.ok_or((
        StatusCode::BAD_REQUEST,
        "wechat code is required".to_string(),
    ))?;

    let result = use_case
        .execute(CompleteWechatLoginInput { code })
        .await
        .map_err(map_auth_error)?;

    let return_to = decode_state_to_return_to(query.state.as_deref(), &headers)?;
    let redirect_url = match result {
        WechatLoginResult::Authenticated(bundle) => format!(
            "{return_to}#wechat=success&access_token={}&expires_at={}",
            url_encode_component(&bundle.access_token),
            url_encode_component(&bundle.expires_at.to_rfc3339()),
        ),
        WechatLoginResult::BindingRequired {
            bind_token,
            expires_at,
            profile,
        } => format!(
            "{return_to}#wechat=bind&bind_token={}&expires_at={}&display_name={}&avatar_url={}",
            url_encode_component(&bind_token),
            url_encode_component(&expires_at.to_rfc3339()),
            url_encode_component(profile.display_name.as_deref().unwrap_or("")),
            url_encode_component(profile.avatar_url.as_deref().unwrap_or("")),
        ),
    };

    Ok(Redirect::temporary(&redirect_url))
}

pub async fn wechat_bind_handler(
    Json(request): Json<WechatBindRequest>,
    use_case: Arc<BindWechatAccountUseCase>,
) -> Result<Json<AuthResponseDto>, (StatusCode, String)> {
    let result = use_case
        .execute(BindWechatAccountInput {
            bind_token: request.bind_token,
            invite_code: request.invite_code,
            referral_code: request.referral_code,
            phone_number: request.phone_number,
            password: request.password,
        })
        .await
        .map_err(map_auth_error)?;

    Ok(Json(result.into()))
}

pub async fn mini_wechat_login_handler(
    Json(request): Json<MiniWechatLoginRequest>,
    use_case: Arc<CompleteMiniWechatLoginUseCase>,
) -> Result<Json<MiniWechatLoginResponseDto>, (StatusCode, String)> {
    let result = use_case
        .execute(CompleteMiniWechatLoginInput { code: request.code })
        .await
        .map_err(map_auth_error)?;

    Ok(Json(result.into()))
}

pub async fn mini_wechat_bind_handler(
    Json(request): Json<MiniWechatBindRequest>,
    use_case: Arc<BindWechatMiniProgramAccountUseCase>,
) -> Result<Json<AuthResponseDto>, (StatusCode, String)> {
    let result = use_case
        .execute(BindWechatMiniProgramAccountInput {
            bind_token: request.bind_token,
            invite_code: request.invite_code,
            referral_code: request.referral_code,
            display_name: request.display_name,
            avatar_data_url: request.avatar_data_url,
        })
        .await
        .map_err(map_auth_error)?;

    Ok(Json(result.into()))
}

fn map_auth_error(error: anyhow::Error) -> (StatusCode, String) {
    let message = error.to_string();

    if let Some(localized) = localize_auth_error_message(&message) {
        return (StatusCode::BAD_REQUEST, localized);
    }

    (StatusCode::INTERNAL_SERVER_ERROR, message)
}

fn localize_auth_error_message(message: &str) -> Option<String> {
    if message.contains("invalid phone number or password")
        || message.contains("invalid account identifier or password")
    {
        return Some("账号或密码错误".to_string());
    }

    if message.contains("phone number must be a valid 11-digit mainland China mobile number") {
        return Some("请输入正确的手机号".to_string());
    }

    if message.contains("password must be at least 6 characters") {
        return Some("密码至少 6 位".to_string());
    }

    if message.contains("invite code is required") {
        return Some("请输入邀请码".to_string());
    }

    if message.contains("display name is required") {
        return Some("请输入昵称".to_string());
    }

    if message.contains("avatar data url is required") {
        return Some("请先选择头像".to_string());
    }

    if message.contains("avatar data url is invalid") {
        return Some("头像数据无效，请重新选择".to_string());
    }

    if message.contains("invalid account identifier or invite code") {
        return Some("账号或邀请码错误".to_string());
    }

    if message.contains("invite code")
        && (message.contains("invalid") || message.contains("expired") || message.contains("used"))
    {
        return Some("邀请码无效或已失效".to_string());
    }

    if message.contains("already registered")
        || message.contains("phone number") && message.contains("registered")
    {
        return Some("该用户名或手机号已注册".to_string());
    }

    if message.contains("bind token is required") {
        return Some("微信绑定信息已失效，请重新发起微信登录".to_string());
    }

    if message.contains("invalid wechat state") {
        return Some("微信登录状态已失效，请重新发起登录".to_string());
    }

    if message.contains("invalid")
        || message.contains("invite code")
        || message.contains("account identifier")
        || message.contains("phone number")
        || message.contains("password")
        || message.contains("registered")
    {
        return Some(message.to_string());
    }

    None
}

fn extract_bearer_token(headers: &HeaderMap) -> Option<&str> {
    let header_value = headers.get(AUTHORIZATION)?.to_str().ok()?;
    header_value.strip_prefix("Bearer ")
}

fn build_callback_url(headers: &HeaderMap) -> Result<String, (StatusCode, String)> {
    let origin = detect_public_origin(headers)?;
    Ok(format!("{origin}/api/v1/auth/wechat/callback"))
}

fn build_frontend_user_url(headers: &HeaderMap) -> Result<String, (StatusCode, String)> {
    let origin = detect_public_origin(headers)?;
    let host = headers
        .get("host")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();

    let path = if host.contains("127.0.0.1") || host.contains("localhost") {
        "/user"
    } else {
        "/football/user"
    };

    Ok(format!("{origin}{path}"))
}

fn detect_public_origin(headers: &HeaderMap) -> Result<String, (StatusCode, String)> {
    let host = headers
        .get("host")
        .and_then(|value| value.to_str().ok())
        .ok_or((
            StatusCode::BAD_REQUEST,
            "host header is required".to_string(),
        ))?;
    let scheme = headers
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("http");

    Ok(format!("{scheme}://{host}"))
}

fn decode_state_to_return_to(
    state: Option<&str>,
    headers: &HeaderMap,
) -> Result<String, (StatusCode, String)> {
    let Some(state) = state else {
        return build_frontend_user_url(headers);
    };

    let decoded = URL_SAFE_NO_PAD
        .decode(state)
        .map_err(|_| (StatusCode::BAD_REQUEST, "invalid wechat state".to_string()))?;
    let return_to = String::from_utf8(decoded)
        .map_err(|_| (StatusCode::BAD_REQUEST, "invalid wechat state".to_string()))?;

    Ok(return_to)
}

fn url_encode_component(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderValue, header::LOCATION};

    #[tokio::test]
    async fn wechat_authorize_redirect_uses_snsapi_base_scope() {
        let mut headers = HeaderMap::new();
        headers.insert("host", HeaderValue::from_static("match.oryjk.cn"));
        headers.insert("x-forwarded-proto", HeaderValue::from_static("https"));

        let redirect = wechat_authorize_handler(
            headers,
            Arc::new(AuthWebConfig {
                wechat_app_id: "wx-test-appid".to_string(),
            }),
        )
        .await
        .expect("wechat authorize redirect should succeed");

        let response = redirect.into_response();
        let location = response
            .headers()
            .get(LOCATION)
            .and_then(|value| value.to_str().ok())
            .expect("location header should exist");

        assert!(location.contains("scope=snsapi_base"));
        assert!(location.contains(
            "redirect_uri=https%3A%2F%2Fmatch.oryjk.cn%2Fapi%2Fv1%2Fauth%2Fwechat%2Fcallback"
        ));
        assert!(location.ends_with("#wechat_redirect"));
    }

    #[test]
    fn localize_auth_error_message_translates_invalid_credentials() {
        assert_eq!(
            localize_auth_error_message("invalid phone number or password"),
            Some("账号或密码错误".to_string())
        );
    }

    #[test]
    fn localize_auth_error_message_translates_invalid_account_identifier_credentials() {
        assert_eq!(
            localize_auth_error_message("invalid account identifier or password"),
            Some("账号或密码错误".to_string())
        );
    }

    #[test]
    fn localize_auth_error_message_translates_invalid_account_identifier_or_invite_code() {
        assert_eq!(
            localize_auth_error_message("invalid account identifier or invite code"),
            Some("账号或邀请码错误".to_string())
        );
    }
}
