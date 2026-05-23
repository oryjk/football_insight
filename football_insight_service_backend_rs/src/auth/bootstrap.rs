use std::sync::Arc;

use axum::Router;
use chrono::Duration;
use sqlx::PgPool;

use crate::{
    auth::{
        adapters::{
            integration::{
                current_standard_match_port::HttpCurrentStandardMatchPort,
                wechat_crypto_port::OfficialWechatCryptoPort,
                wechat_oauth_port::OfficialWechatOauthPort,
            },
            persistence::postgres_auth_repository::PostgresAuthRepository,
            security::{argon2_password_port::Argon2PasswordPort, jwt_token_port::JwtTokenPort},
            web::{handlers::AuthWebConfig, routes::auth_routes},
        },
        application::{
            bind_wechat_account::BindWechatAccountUseCase,
            bind_wechat_mini_program_account::BindWechatMiniProgramAccountUseCase,
            get_current_user::GetCurrentUserUseCase,
            handle_wechat_webhook::HandleWechatWebhookUseCase,
            login_with_mini_wechat::CompleteMiniWechatLoginUseCase,
            login_with_password::LoginWithPasswordUseCase,
            login_with_wechat::CompleteWechatLoginUseCase, logout::LogoutUseCase,
            register_with_invite::RegisterWithInviteUseCase,
            reset_password_with_invite::ResetPasswordWithInviteUseCase,
        },
    },
    config::AppConfig,
    system_config::ports::system_config_port::SystemConfigPort,
};

pub struct AuthBootstrap {
    pub routes: Router,
    pub auth_repository: Arc<PostgresAuthRepository>,
    pub token_port: Arc<JwtTokenPort>,
    pub get_current_user_use_case: Arc<GetCurrentUserUseCase>,
}

pub fn build_auth(
    pool: PgPool,
    config: &AppConfig,
    system_config_port: Arc<dyn SystemConfigPort>,
) -> AuthBootstrap {
    let auth_repository = Arc::new(PostgresAuthRepository::new(pool));
    let password_port = Arc::new(Argon2PasswordPort);
    let token_port = Arc::new(JwtTokenPort::new(config.jwt_secret.clone()));
    let wechat_oauth_port = Arc::new(OfficialWechatOauthPort::new(
        config.wechat_app_id.clone(),
        config.wechat_app_secret.clone(),
    ));
    let mini_wechat_oauth_port = Arc::new(OfficialWechatOauthPort::new(
        config.wechat_mini_app_id.clone(),
        config.wechat_mini_app_secret.clone(),
    ));
    let wechat_crypto_port = Arc::new(
        OfficialWechatCryptoPort::new(
            config.wechat_webhook_token.clone(),
            config.wechat_encoding_aes_key.clone(),
        )
        .expect("invalid wechat webhook config"),
    );
    let current_standard_match_port = Arc::new(HttpCurrentStandardMatchPort::new(
        config.ticket_monitor_base_url.clone(),
    ));
    let register_with_invite_use_case = Arc::new(RegisterWithInviteUseCase::new(
        auth_repository.clone(),
        password_port.clone(),
        token_port.clone(),
        Duration::days(30),
    ));
    let login_with_password_use_case = Arc::new(LoginWithPasswordUseCase::new(
        auth_repository.clone(),
        password_port,
        token_port.clone(),
        Duration::days(30),
    ));
    let reset_password_with_invite_use_case = Arc::new(ResetPasswordWithInviteUseCase::new(
        auth_repository.clone(),
        Arc::new(Argon2PasswordPort),
    ));
    let complete_wechat_login_use_case = Arc::new(CompleteWechatLoginUseCase::new(
        auth_repository.clone(),
        wechat_oauth_port,
        token_port.clone(),
        Duration::days(30),
        Duration::minutes(10),
    ));
    let complete_mini_wechat_login_use_case = Arc::new(CompleteMiniWechatLoginUseCase::new(
        auth_repository.clone(),
        mini_wechat_oauth_port,
        token_port.clone(),
        Duration::days(30),
        Duration::minutes(10),
    ));
    let bind_wechat_account_use_case = Arc::new(BindWechatAccountUseCase::new(
        auth_repository.clone(),
        Arc::new(Argon2PasswordPort),
        token_port.clone(),
        Duration::days(30),
    ));
    let bind_mini_wechat_account_use_case = Arc::new(BindWechatMiniProgramAccountUseCase::new(
        auth_repository.clone(),
        token_port.clone(),
        Duration::days(30),
    ));
    let get_current_user_use_case = Arc::new(GetCurrentUserUseCase::new(
        auth_repository.clone(),
        token_port.clone(),
    ));
    let logout_use_case = Arc::new(LogoutUseCase::new());
    let wechat_webhook_use_case = Arc::new(HandleWechatWebhookUseCase::new(
        auth_repository.clone(),
        wechat_crypto_port,
        current_standard_match_port,
        system_config_port,
    ));
    let auth_web_config = Arc::new(AuthWebConfig {
        wechat_app_id: config.wechat_app_id.clone(),
    });

    AuthBootstrap {
        routes: auth_routes(
            register_with_invite_use_case,
            login_with_password_use_case,
            reset_password_with_invite_use_case,
            complete_wechat_login_use_case,
            bind_wechat_account_use_case,
            complete_mini_wechat_login_use_case,
            bind_mini_wechat_account_use_case,
            get_current_user_use_case.clone(),
            logout_use_case,
            wechat_webhook_use_case,
            auth_web_config,
        ),
        auth_repository,
        token_port,
        get_current_user_use_case,
    }
}
