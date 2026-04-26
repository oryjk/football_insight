pub trait PasswordPort: Send + Sync {
    fn hash_password(&self, password: &str) -> anyhow::Result<String>;
    fn verify_password(&self, password: &str, password_hash: &str) -> anyhow::Result<bool>;
}
