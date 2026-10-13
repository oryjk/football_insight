use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AdminCredentialsError {
    #[error("{0}")]
    InvalidInput(&'static str),
    #[error("current password is incorrect")]
    WrongPassword,
    #[error("username already exists")]
    Conflict,
    #[error("admin login required")]
    Unauthorized,
    #[error("credentials update failed")]
    Internal,
}

// Secrets intentionally do not implement Debug or Serialize.
pub struct AdminCredentialsInput {
    pub username: String,
    pub current_password: String,
    pub new_password: Option<String>,
}
pub struct AdminCredentialsSnapshot {
    pub username: String,
    pub password_hash: String,
}
pub struct AdminCredentialsChange {
    pub admin_id: Uuid,
    pub session_id: Uuid,
    pub expected_username: String,
    pub expected_password_hash: String,
    pub username: String,
    pub password_hash: Option<String>,
}
impl AdminCredentialsInput {
    pub fn validate(mut self) -> Result<Self, AdminCredentialsError> {
        self.username = self.username.trim().to_ascii_lowercase();
        if !(3..=64).contains(&self.username.len())
            || !self
                .username
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
        {
            return Err(AdminCredentialsError::InvalidInput(
                "username must be 3 to 64 ASCII letters, numbers, dots, dashes or underscores",
            ));
        }
        if self.current_password.is_empty() {
            return Err(AdminCredentialsError::InvalidInput(
                "current_password is required",
            ));
        }
        if self
            .new_password
            .as_ref()
            .is_some_and(|password| !(8..=128).contains(&password.chars().count()))
        {
            return Err(AdminCredentialsError::InvalidInput(
                "new_password must be 8 to 128 characters",
            ));
        }
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn input(username: &str, new_password: Option<&str>) -> AdminCredentialsInput {
        AdminCredentialsInput {
            username: username.into(),
            current_password: "test-current-password".into(),
            new_password: new_password.map(Into::into),
        }
    }
    #[test]
    fn normalizes_username_and_preserves_password_whitespace() {
        let input = input(" Mixed.Name_1- ", Some("  exact password  "))
            .validate()
            .unwrap();
        assert_eq!(input.username, "mixed.name_1-");
        assert_eq!(input.new_password.as_deref(), Some("  exact password  "));
    }
    #[test]
    fn validates_username_bounds_and_allowed_ascii() {
        for username in ["ab", "a b", "管理员", "bad/name", ""] {
            assert!(matches!(
                input(username, None).validate(),
                Err(AdminCredentialsError::InvalidInput(_))
            ));
        }
        assert!(input(&"a".repeat(64), None).validate().is_ok());
        assert!(input(&"a".repeat(65), None).validate().is_err());
    }
    #[test]
    fn requires_current_password_and_bounds_optional_new_password() {
        let mut missing = input("owner", None);
        missing.current_password.clear();
        assert!(missing.validate().is_err());
        for password in ["", "short"] {
            assert!(input("owner", Some(password)).validate().is_err());
        }
        for length in [8, 128] {
            assert!(input("owner", Some(&"x".repeat(length))).validate().is_ok());
        }
        assert!(input("owner", Some(&"x".repeat(129))).validate().is_err());
        assert!(input("owner", None).validate().is_ok());
    }
}
