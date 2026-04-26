#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use jsonwebtoken::{EncodingKey, Header, encode};
    use serde::Serialize;
    use uuid::Uuid;

    use crate::auth::{
        adapters::security::jwt_token_port::JwtTokenPort,
        ports::token_port::{TokenPort, WechatBindTokenPayload},
    };

    #[derive(Serialize)]
    struct LegacyJwtClaims<'a> {
        kind: &'a str,
        sub: String,
        phone_number: &'a str,
        open_id: Option<&'a str>,
        union_id: Option<&'a str>,
        display_name: Option<&'a str>,
        avatar_url: Option<&'a str>,
        exp: i64,
    }

    #[test]
    fn jwt_token_roundtrip_preserves_user_identity() {
        let port = JwtTokenPort::new("test-secret".to_string());
        let user_id = Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();
        let expires_at = Utc::now() + Duration::hours(12);

        let token = port
            .issue_token(user_id, "13800138000", expires_at)
            .unwrap();
        let claims = port.verify_token(&token).unwrap();

        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.account_identifier, "13800138000");
    }

    #[test]
    fn jwt_token_roundtrip_accepts_legacy_phone_number_claim() {
        let port = JwtTokenPort::new("test-secret".to_string());
        let user_id = Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();
        let expires_at = Utc::now() + Duration::hours(12);

        let token = encode(
            &Header::default(),
            &LegacyJwtClaims {
                kind: "access",
                sub: user_id.to_string(),
                phone_number: "13800138000",
                open_id: None,
                union_id: None,
                display_name: None,
                avatar_url: None,
                exp: expires_at.timestamp(),
            },
            &EncodingKey::from_secret("test-secret".as_bytes()),
        )
        .unwrap();

        let claims = port.verify_token(&token).unwrap();

        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.account_identifier, "13800138000");
    }

    #[test]
    fn wechat_bind_token_roundtrip_preserves_profile() {
        let port = JwtTokenPort::new("test-secret".to_string());
        let expires_at = Utc::now() + Duration::minutes(10);

        let token = port
            .issue_wechat_bind_token(
                WechatBindTokenPayload {
                    open_id: "openid-123".to_string(),
                    union_id: Some("union-123".to_string()),
                    display_name: Some("何止编程".to_string()),
                    avatar_url: Some("https://img.example.com/avatar.png".to_string()),
                },
                expires_at,
            )
            .unwrap();
        let claims = port.verify_wechat_bind_token(&token).unwrap();

        assert_eq!(claims.open_id, "openid-123");
        assert_eq!(claims.union_id.as_deref(), Some("union-123"));
        assert_eq!(claims.display_name.as_deref(), Some("何止编程"));
        assert_eq!(
            claims.avatar_url.as_deref(),
            Some("https://img.example.com/avatar.png")
        );
    }
}
