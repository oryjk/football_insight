use std::sync::Arc;

use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::auth::{
    domain::user::AuthTokenBundle,
    ports::{h5_test_login_port::H5TestLoginUserPort, token_port::TokenPort},
};

/// H5 测试登录：白名单内的用户可一键换取真实 JWT，
/// 用于在 H5 上验证需要登录的场景。白名单来自 env `H5_TEST_LOGIN_USER_IDS`，
/// 为空时接口不开放。
pub struct LoginAsH5TestUserUseCase {
    user_port: Arc<dyn H5TestLoginUserPort>,
    token_port: Arc<dyn TokenPort>,
    allowed_user_ids: Vec<Uuid>,
    session_ttl: Duration,
}

impl LoginAsH5TestUserUseCase {
    pub fn new(
        user_port: Arc<dyn H5TestLoginUserPort>,
        token_port: Arc<dyn TokenPort>,
        allowed_user_ids: Vec<Uuid>,
        session_ttl: Duration,
    ) -> Self {
        Self {
            user_port,
            token_port,
            allowed_user_ids,
            session_ttl,
        }
    }

    pub fn allowed_user_ids(&self) -> &[Uuid] {
        &self.allowed_user_ids
    }

    pub async fn list_users(&self) -> anyhow::Result<Vec<crate::auth::domain::user::AuthUser>> {
        let mut users = Vec::with_capacity(self.allowed_user_ids.len());
        for user_id in &self.allowed_user_ids {
            if let Some(user) = self.user_port.find_active_user_by_id(*user_id).await? {
                users.push(user);
            }
        }
        Ok(users)
    }

    pub async fn execute(&self, user_id: Uuid) -> anyhow::Result<AuthTokenBundle> {
        if !self.allowed_user_ids.contains(&user_id) {
            anyhow::bail!("h5 test login is not available for this user");
        }

        let user = self
            .user_port
            .find_active_user_by_id(user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("user not found or inactive"))?;

        let expires_at = Utc::now() + self.session_ttl;
        let access_token = self.token_port.issue_token(
            user.id,
            &user.account_identifier,
            expires_at,
        )?;

        Ok(AuthTokenBundle {
            user,
            access_token,
            expires_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use chrono::{TimeZone, Utc};
    use uuid::Uuid;

    use super::LoginAsH5TestUserUseCase;
    use crate::auth::{
        adapters::security::jwt_token_port::JwtTokenPort,
        domain::user::AuthUser,
        ports::{h5_test_login_port::H5TestLoginUserPort, token_port::TokenPort},
    };

    struct FakeUserPort {
        users: Vec<AuthUser>,
    }

    #[async_trait]
    impl H5TestLoginUserPort for FakeUserPort {
        async fn find_active_user_by_id(&self, user_id: Uuid) -> anyhow::Result<Option<AuthUser>> {
            Ok(self.users.iter().find(|user| user.id == user_id).cloned())
        }
    }

    fn fake_user(id: Uuid, identifier: &str) -> AuthUser {
        AuthUser {
            id,
            account_identifier: identifier.to_string(),
            display_name: Some(identifier.to_string()),
            invite_code: None,
            avatar_url: None,
            has_wechat_binding: false,
            membership_tier: "V1".to_string(),
            membership_expires_at: None,
            membership_benefits_enabled: true,
            ticket_watch_poll_interval_seconds: 60,
            created_at: Utc.with_ymd_and_hms(2026, 8, 26, 12, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2026, 8, 26, 12, 0, 0).unwrap(),
        }
    }

    fn use_case(allowed: Vec<Uuid>, existing: Vec<AuthUser>) -> LoginAsH5TestUserUseCase {
        LoginAsH5TestUserUseCase::new(
            Arc::new(FakeUserPort { users: existing }),
            Arc::new(JwtTokenPort::new("test-secret-for-h5-test-login".to_string())),
            allowed,
            chrono::Duration::days(30),
        )
    }

    #[tokio::test]
    async fn issues_token_for_whitelisted_user() {
        let admin = Uuid::new_v4();
        let token_port = Arc::new(JwtTokenPort::new("test-secret-for-h5-test-login".to_string()));
        let use_case = LoginAsH5TestUserUseCase::new(
            Arc::new(FakeUserPort { users: vec![fake_user(admin, "admin")] }),
            token_port.clone(),
            vec![admin],
            chrono::Duration::days(30),
        );

        let bundle = use_case.execute(admin).await.expect("login bundle");

        assert_eq!(bundle.user.id, admin);
        assert!(!bundle.access_token.is_empty());
        let claims = token_port.verify_token(&bundle.access_token).expect("verified");
        assert_eq!(claims.sub, admin);
    }

    #[tokio::test]
    async fn rejects_user_outside_whitelist() {
        let admin = Uuid::new_v4();
        let other = Uuid::new_v4();
        let use_case = use_case(vec![admin], vec![fake_user(other, "other")]);

        let error = use_case.execute(other).await.expect_err("should reject");

        assert!(error.to_string().contains("not available"));
    }

    #[tokio::test]
    async fn rejects_missing_user() {
        let admin = Uuid::new_v4();
        let use_case = use_case(vec![admin], vec![]);

        let error = use_case.execute(admin).await.expect_err("should reject");

        assert!(error.to_string().contains("not found"));
    }

    #[tokio::test]
    async fn list_users_skips_unknown_ids() {
        let admin = Uuid::new_v4();
        let ghost = Uuid::new_v4();
        let use_case = use_case(vec![admin, ghost], vec![fake_user(admin, "admin")]);

        let users = use_case.list_users().await.expect("list");

        assert_eq!(users.len(), 1);
        assert_eq!(users[0].id, admin);
    }
}
