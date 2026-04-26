use chrono::{DateTime, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::ports::token_port::{
    AuthTokenClaims, TokenPort, WechatBindTokenClaims, WechatBindTokenPayload,
};

#[derive(Clone)]
pub struct JwtTokenPort {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JwtClaims {
    kind: String,
    sub: String,
    #[serde(alias = "phone_number")]
    account_identifier: String,
    open_id: Option<String>,
    union_id: Option<String>,
    display_name: Option<String>,
    avatar_url: Option<String>,
    exp: i64,
}

impl JwtTokenPort {
    pub fn new(secret: String) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
        }
    }
}

impl TokenPort for JwtTokenPort {
    fn issue_token(
        &self,
        user_id: Uuid,
        account_identifier: &str,
        expires_at: DateTime<Utc>,
    ) -> anyhow::Result<String> {
        let claims = JwtClaims {
            kind: "access".to_string(),
            sub: user_id.to_string(),
            account_identifier: account_identifier.to_string(),
            open_id: None,
            union_id: None,
            display_name: None,
            avatar_url: None,
            exp: expires_at.timestamp(),
        };

        let token = encode(&Header::default(), &claims, &self.encoding_key)?;
        Ok(token)
    }

    fn verify_token(&self, token: &str) -> anyhow::Result<AuthTokenClaims> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;

        let claims = decode::<JwtClaims>(token, &self.decoding_key, &validation)?.claims;
        if claims.kind != "access" {
            anyhow::bail!("invalid token kind");
        }
        Ok(AuthTokenClaims {
            sub: Uuid::parse_str(&claims.sub)?,
            account_identifier: claims.account_identifier,
            exp: DateTime::from_timestamp(claims.exp, 0)
                .ok_or_else(|| anyhow::anyhow!("invalid exp"))?,
        })
    }

    fn issue_wechat_bind_token(
        &self,
        payload: WechatBindTokenPayload,
        expires_at: DateTime<Utc>,
    ) -> anyhow::Result<String> {
        let claims = JwtClaims {
            kind: "wechat_bind".to_string(),
            sub: "00000000-0000-0000-0000-000000000000".to_string(),
            account_identifier: String::new(),
            open_id: Some(payload.open_id),
            union_id: payload.union_id,
            display_name: payload.display_name,
            avatar_url: payload.avatar_url,
            exp: expires_at.timestamp(),
        };

        let token = encode(&Header::default(), &claims, &self.encoding_key)?;
        Ok(token)
    }

    fn verify_wechat_bind_token(&self, token: &str) -> anyhow::Result<WechatBindTokenClaims> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;

        let claims = decode::<JwtClaims>(token, &self.decoding_key, &validation)?.claims;
        if claims.kind != "wechat_bind" {
            anyhow::bail!("invalid token kind");
        }

        Ok(WechatBindTokenClaims {
            open_id: claims
                .open_id
                .ok_or_else(|| anyhow::anyhow!("missing open_id"))?,
            union_id: claims.union_id,
            display_name: claims.display_name,
            avatar_url: claims.avatar_url,
            exp: DateTime::from_timestamp(claims.exp, 0)
                .ok_or_else(|| anyhow::anyhow!("invalid exp"))?,
        })
    }
}
