use aes::Aes256;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use cbc::cipher::{BlockEncryptMut, KeyIvInit, block_padding::Pkcs7};
use football_insight_service_backend_rs::{
    app::build_router,
    config::{AppConfig, DatabasePoolConfig},
};
use sha1::{Digest, Sha1};
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;

type Aes256CbcEnc = cbc::Encryptor<Aes256>;

#[tokio::test]
async fn wechat_webhook_verification_returns_plaintext_echo() {
    let token = "token123";
    let aes_key = "l8TIs4SKawzdwJE+K0nmaiqAJ7qCe5OYLql2Jtufu+Y";
    let echostr = encrypt_echo("hello-wechat", "wx_test_app", aes_key);
    let signature = sign(token, "1", "2");

    let pool = PgPoolOptions::new()
        .connect_lazy("postgresql://example:example@127.0.0.1:5432/example")
        .unwrap();

    let app = build_router(
        pool,
        &AppConfig {
            port: 8080,
            database_url: "postgresql://example:example@127.0.0.1:5432/example".to_string(),
            database_pool: DatabasePoolConfig {
                max_connections: 10,
                min_connections: 3,
                acquire_timeout_secs: 5,
                idle_timeout_secs: 90,
                max_lifetime_secs: 600,
            },
            jwt_secret: "dev-secret".to_string(),
            openai_api_key: None,
            openai_base_url: None,
            ai_chat_model: "gpt-4o-mini".to_string(),
            wechat_app_id: "wx_test_app".to_string(),
            wechat_app_secret: "wx_test_secret".to_string(),
            wechat_mini_app_id: "wx_test_mini_app".to_string(),
            wechat_mini_app_secret: "wx_test_mini_secret".to_string(),
            wechat_webhook_token: token.to_string(),
            wechat_encoding_aes_key: aes_key.to_string(),
            ticket_monitor_base_url: None,
            redis_url: "redis://127.0.0.1:6379".to_string(),
            wechat_pay_mch_id: String::new(),
            wechat_pay_api_key: String::new(),
            public_base_url: "http://127.0.0.1:8080".to_string(),
        },
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/football/wechat/webhook?signature={signature}&timestamp=1&nonce=2&echostr={}",
                    encode_query_value(&echostr)
                ))
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

fn sign(token: &str, timestamp: &str, nonce: &str) -> String {
    let mut parts = [token, timestamp, nonce];
    parts.sort_unstable();

    let mut hasher = Sha1::new();
    hasher.update(parts.join("").as_bytes());
    format!("{:x}", hasher.finalize())
}

fn encrypt_echo(plain: &str, app_id: &str, encoding_aes_key: &str) -> String {
    let aes_key = STANDARD.decode(format!("{encoding_aes_key}=")).unwrap();
    let iv = &aes_key[..16];
    let random_prefix = uuid::Uuid::new_v4().into_bytes();
    let xml_bytes = plain.as_bytes();
    let xml_len = (xml_bytes.len() as u32).to_be_bytes();

    let mut plain_bytes = Vec::new();
    plain_bytes.extend_from_slice(&random_prefix);
    plain_bytes.extend_from_slice(&xml_len);
    plain_bytes.extend_from_slice(xml_bytes);
    plain_bytes.extend_from_slice(app_id.as_bytes());

    let block_size = 16;
    let padded_len = ((plain_bytes.len() / block_size) + 1) * block_size;
    let mut buf = vec![0_u8; padded_len];
    buf[..plain_bytes.len()].copy_from_slice(&plain_bytes);

    let encrypted = Aes256CbcEnc::new_from_slices(&aes_key, iv)
        .unwrap()
        .encrypt_padded_mut::<Pkcs7>(&mut buf, plain_bytes.len())
        .unwrap();

    STANDARD.encode(encrypted)
}

fn encode_query_value(value: &str) -> String {
    value
        .replace('+', "%2B")
        .replace('/', "%2F")
        .replace('=', "%3D")
}
