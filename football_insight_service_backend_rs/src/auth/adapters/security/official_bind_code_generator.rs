use crate::auth::{
    domain::official_bind::encode_bind_code,
    ports::official_bind_code_generator::{OfficialBindCodeGenerator, SecureRandomUnavailable},
};
use rand::TryRngCore;

pub struct SecureOfficialBindCodeGenerator;

impl OfficialBindCodeGenerator for SecureOfficialBindCodeGenerator {
    fn generate(&self) -> Result<String, SecureRandomUnavailable> {
        let mut bytes = [0u8; 6];
        rand::rngs::OsRng
            .try_fill_bytes(&mut bytes)
            .map_err(|_| SecureRandomUnavailable)?;
        Ok(encode_bind_code(bytes))
    }
}
