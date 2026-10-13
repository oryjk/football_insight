#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AdminSetupError {
    #[error("password must be 8 to 128 characters")]
    InvalidPassword,
    #[error("administrator is already initialized")]
    AlreadyInitialized,
    #[error("administrator initialization failed")]
    Internal,
}
// Password is deliberately neither Debug nor Serialize.
pub struct AdminSetupInput {
    pub password: String,
}
impl AdminSetupInput {
    pub fn validate(self) -> Result<Self, AdminSetupError> {
        if !(8..=128).contains(&self.password.chars().count()) {
            return Err(AdminSetupError::InvalidPassword);
        }
        Ok(self)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn setup_password_is_bounded_and_keeps_whitespace() {
        for password in ["", "short"] {
            assert!(
                AdminSetupInput {
                    password: password.into()
                }
                .validate()
                .is_err()
            );
        }
        for size in [8, 128] {
            assert!(
                AdminSetupInput {
                    password: "字".repeat(size)
                }
                .validate()
                .is_ok()
            );
        }
        assert!(
            AdminSetupInput {
                password: "x".repeat(129)
            }
            .validate()
            .is_err()
        );
        let value = AdminSetupInput {
            password: "  exact password  ".into(),
        }
        .validate()
        .unwrap();
        assert_eq!(value.password, "  exact password  ");
    }
}
