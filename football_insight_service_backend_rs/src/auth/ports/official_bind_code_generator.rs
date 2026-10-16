#[derive(thiserror::Error, Debug)]
#[error("secure bind code generation unavailable")]
pub struct SecureRandomUnavailable;

pub trait OfficialBindCodeGenerator: Send + Sync {
    fn generate(&self) -> Result<String, SecureRandomUnavailable>;
}
