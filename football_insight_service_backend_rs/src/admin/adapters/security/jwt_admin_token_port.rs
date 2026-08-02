use chrono::{DateTime, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::admin::{domain::admin_auth::AdminTokenClaims, ports::admin_token_port::AdminTokenPort};

#[derive(Clone)]
pub struct JwtAdminTokenPort {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JwtAdminClaims {
    kind: String,
    sub: String,
    sid: String,
    username: String,
    role: String,
    exp: i64,
}

impl JwtAdminTokenPort {
    pub fn new(secret: String) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
        }
    }
}

impl AdminTokenPort for JwtAdminTokenPort {
    fn issue_token(
        &self,
        admin_id: Uuid,
        session_id: Uuid,
        username: &str,
        role: &str,
        expires_at: DateTime<Utc>,
    ) -> anyhow::Result<String> {
        let claims = JwtAdminClaims {
            kind: "admin_access".to_string(),
            sub: admin_id.to_string(),
            sid: session_id.to_string(),
            username: username.to_string(),
            role: role.to_string(),
            exp: expires_at.timestamp(),
        };
        Ok(encode(&Header::default(), &claims, &self.encoding_key)?)
    }

    fn verify_token(&self, token: &str) -> anyhow::Result<AdminTokenClaims> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        validation.leeway = 0;
        let claims = decode::<JwtAdminClaims>(token, &self.decoding_key, &validation)?.claims;
        if claims.kind != "admin_access" {
            anyhow::bail!("invalid admin token kind");
        }
        Ok(AdminTokenClaims {
            admin_id: Uuid::parse_str(&claims.sub)?,
            session_id: Uuid::parse_str(&claims.sid)?,
            username: claims.username,
            role: claims.role,
            expires_at: DateTime::from_timestamp(claims.exp, 0)
                .ok_or_else(|| anyhow::anyhow!("invalid admin token expiration"))?,
        })
    }
}
