use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};

use crate::auth::application::{
    bind_wechat_account::BindWechatAccountUseCase,
    bind_wechat_mini_program_account::BindWechatMiniProgramAccountUseCase,
    get_current_user::GetCurrentUserUseCase, handle_wechat_webhook::HandleWechatWebhookUseCase,
    login_with_mini_wechat::CompleteMiniWechatLoginUseCase,
    login_with_password::LoginWithPasswordUseCase, login_with_wechat::CompleteWechatLoginUseCase,
    logout::LogoutUseCase, register_with_invite::RegisterWithInviteUseCase,
    reset_password_with_invite::ResetPasswordWithInviteUseCase,
};

use super::{
    handlers::{
        AuthWebConfig, get_current_user_handler, login_handler, logout_handler,
        mini_wechat_bind_handler, mini_wechat_login_handler, register_handler,
        reset_password_handler, wechat_authorize_handler, wechat_bind_handler,
        wechat_callback_handler,
    },
    wechat_handlers::{wechat_message_handler, wechat_verify_handler},
};

pub fn auth_routes(
    register_with_invite_use_case: Arc<RegisterWithInviteUseCase>,
    login_with_password_use_case: Arc<LoginWithPasswordUseCase>,
    reset_password_with_invite_use_case: Arc<ResetPasswordWithInviteUseCase>,
    complete_wechat_login_use_case: Arc<CompleteWechatLoginUseCase>,
    bind_wechat_account_use_case: Arc<BindWechatAccountUseCase>,
    complete_mini_wechat_login_use_case: Arc<CompleteMiniWechatLoginUseCase>,
    bind_mini_wechat_account_use_case: Arc<BindWechatMiniProgramAccountUseCase>,
    get_current_user_use_case: Arc<GetCurrentUserUseCase>,
    logout_use_case: Arc<LogoutUseCase>,
    wechat_webhook_use_case: Arc<HandleWechatWebhookUseCase>,
    web_config: Arc<AuthWebConfig>,
) -> Router {
    let authorize_config = web_config.clone();

    Router::new()
        .route(
            "/api/v1/auth/register",
            post(move |payload| register_handler(payload, register_with_invite_use_case.clone())),
        )
        .route(
            "/api/v1/auth/login",
            post(move |payload| login_handler(payload, login_with_password_use_case.clone())),
        )
        .route(
            "/api/v1/auth/reset-password",
            post(move |payload| {
                reset_password_handler(payload, reset_password_with_invite_use_case.clone())
            }),
        )
        .route(
            "/api/v1/auth/wechat/authorize",
            get(move |headers| wechat_authorize_handler(headers, authorize_config.clone())),
        )
        .route(
            "/api/v1/auth/wechat/callback",
            get(move |headers, query| {
                wechat_callback_handler(headers, query, complete_wechat_login_use_case.clone())
            }),
        )
        .route(
            "/api/v1/auth/wechat/bind",
            post(move |payload| wechat_bind_handler(payload, bind_wechat_account_use_case.clone())),
        )
        .route(
            "/api/v1/auth/mini-wechat/login",
            post(move |payload| {
                mini_wechat_login_handler(payload, complete_mini_wechat_login_use_case.clone())
            }),
        )
        .route(
            "/api/v1/auth/mini-wechat/bind",
            post(move |payload| {
                mini_wechat_bind_handler(payload, bind_mini_wechat_account_use_case.clone())
            }),
        )
        .route(
            "/api/v1/auth/me",
            get(move |headers| {
                get_current_user_handler(headers, get_current_user_use_case.clone())
            }),
        )
        .route(
            "/api/v1/auth/logout",
            post(move || logout_handler(logout_use_case.clone())),
        )
        .route(
            "/football/wechat/webhook",
            get(wechat_verify_handler).post(wechat_message_handler),
        )
        .with_state(wechat_webhook_use_case)
}
