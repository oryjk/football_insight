use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};

use crate::auth::ports::password_port::PasswordPort;

#[derive(Clone, Default)]
pub struct Argon2PasswordPort;

impl PasswordPort for Argon2PasswordPort {
    fn hash_password(&self, password: &str) -> anyhow::Result<String> {
        let salt = SaltString::encode_b64(uuid::Uuid::new_v4().as_bytes())
            .map_err(|error| anyhow::anyhow!("failed to build password salt: {error}"))?;

        let hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|error| anyhow::anyhow!("failed to hash password: {error}"))?
            .to_string();

        Ok(hash)
    }

    fn verify_password(&self, password: &str, password_hash: &str) -> anyhow::Result<bool> {
        let parsed_hash = PasswordHash::new(password_hash)
            .map_err(|error| anyhow::anyhow!("failed to parse password hash: {error}"))?;
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}
