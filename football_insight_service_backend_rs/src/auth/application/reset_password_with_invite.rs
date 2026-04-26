use std::sync::Arc;

use crate::auth::{
    domain::user::{normalize_password, validate_account_identifier},
    ports::{auth_repository::AuthRepository, password_port::PasswordPort},
};

#[derive(Debug, Clone)]
pub struct ResetPasswordInput {
    pub invite_code: String,
    pub account_identifier: String,
    pub password: String,
}

pub struct ResetPasswordWithInviteUseCase {
    repository: Arc<dyn AuthRepository>,
    password_port: Arc<dyn PasswordPort>,
}

impl ResetPasswordWithInviteUseCase {
    pub fn new(repository: Arc<dyn AuthRepository>, password_port: Arc<dyn PasswordPort>) -> Self {
        Self {
            repository,
            password_port,
        }
    }

    pub async fn execute(&self, input: ResetPasswordInput) -> anyhow::Result<()> {
        let invite_code = input.invite_code.trim().to_string();
        if invite_code.is_empty() {
            anyhow::bail!("invite code is required");
        }

        let account_identifier = validate_account_identifier(&input.account_identifier)?;
        let password = normalize_password(&input.password);
        if password.len() < 6 {
            anyhow::bail!("password must be at least 6 characters");
        }

        let password_hash = self.password_port.hash_password(&password)?;
        self.repository
            .reset_password_with_invite(&invite_code, &account_identifier, &password_hash)
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::{ResetPasswordInput, ResetPasswordWithInviteUseCase};
    use crate::auth::{
        domain::{
            user::{AuthUser, StoredAuthUser},
            wechat::{IssuedInviteCode, WechatOauthProfile},
        },
        ports::{auth_repository::AuthRepository, password_port::PasswordPort},
    };
    use async_trait::async_trait;
    use uuid::Uuid;

    #[derive(Default)]
    struct FakeRepository {
        events: Mutex<Vec<String>>,
    }

    #[async_trait]
    impl AuthRepository for FakeRepository {
        async fn create_user_with_invite(
            &self,
            _invite_code: &str,
            _account_identifier: &str,
            _password_hash: &str,
        ) -> anyhow::Result<AuthUser> {
            unreachable!()
        }

        async fn find_user_by_account_identifier(
            &self,
            _account_identifier: &str,
        ) -> anyhow::Result<Option<StoredAuthUser>> {
            unreachable!()
        }

        async fn find_user_by_wechat_open_id(
            &self,
            _open_id: &str,
        ) -> anyhow::Result<Option<AuthUser>> {
            unreachable!()
        }

        async fn bind_wechat_to_user(
            &self,
            _user_id: Uuid,
            _profile: &WechatOauthProfile,
        ) -> anyhow::Result<AuthUser> {
            unreachable!()
        }

        async fn create_user_with_invite_and_wechat(
            &self,
            _invite_code: &str,
            _phone_number: &str,
            _password_hash: &str,
            _profile: &WechatOauthProfile,
        ) -> anyhow::Result<AuthUser> {
            unreachable!()
        }

        async fn create_user_with_invite_and_mini_program_wechat(
            &self,
            _invite_code: &str,
            _profile: &WechatOauthProfile,
        ) -> anyhow::Result<AuthUser> {
            unreachable!()
        }

        async fn get_user_by_id(&self, _user_id: Uuid) -> anyhow::Result<Option<AuthUser>> {
            unreachable!()
        }

        async fn issue_invite_code_for_wechat_follower(
            &self,
            _open_id: &str,
        ) -> anyhow::Result<IssuedInviteCode> {
            unreachable!()
        }

        async fn mark_wechat_follower_unsubscribed(&self, _open_id: &str) -> anyhow::Result<()> {
            unreachable!()
        }

        async fn reset_password_with_invite(
            &self,
            invite_code: &str,
            account_identifier: &str,
            password_hash: &str,
        ) -> anyhow::Result<()> {
            self.events.lock().unwrap().push(format!(
                "reset:{invite_code}:{account_identifier}:{password_hash}"
            ));
            Ok(())
        }
    }

    struct FakePasswordPort;

    impl PasswordPort for FakePasswordPort {
        fn hash_password(&self, password: &str) -> anyhow::Result<String> {
            Ok(format!("hashed::{password}"))
        }

        fn verify_password(&self, _password: &str, _password_hash: &str) -> anyhow::Result<bool> {
            unreachable!()
        }
    }

    #[tokio::test]
    async fn execute_resets_password_with_invite() {
        let repository = Arc::new(FakeRepository::default());
        let use_case =
            ResetPasswordWithInviteUseCase::new(repository.clone(), Arc::new(FakePasswordPort));

        use_case
            .execute(ResetPasswordInput {
                invite_code: "FI-INVITE".to_string(),
                account_identifier: "18602812970".to_string(),
                password: "secret123".to_string(),
            })
            .await
            .unwrap();

        let events = repository.events.lock().unwrap().clone();
        assert_eq!(
            events,
            vec!["reset:FI-INVITE:18602812970:hashed::secret123"]
        );
    }

    #[tokio::test]
    async fn execute_trims_password_before_resetting() {
        let repository = Arc::new(FakeRepository::default());
        let use_case =
            ResetPasswordWithInviteUseCase::new(repository.clone(), Arc::new(FakePasswordPort));

        use_case
            .execute(ResetPasswordInput {
                invite_code: "FI-INVITE".to_string(),
                account_identifier: "18602812970".to_string(),
                password: "  secret123  ".to_string(),
            })
            .await
            .unwrap();

        let events = repository.events.lock().unwrap().clone();
        assert_eq!(
            events,
            vec!["reset:FI-INVITE:18602812970:hashed::secret123"]
        );
    }
}
