use crate::{
    config::VirtualPaymentConfig,
    payment::{
        domain::virtual_payment::{
            Delivery, VirtualOrder, VirtualPayParams, VirtualPaymentError as Error, VirtualProduct,
            build_sign_data,
        },
        ports::virtual_payment_port::VirtualPaymentGateway,
    },
};
use hmac::{Hmac, Mac};
use serde_json::{Value, json};
use sha2::Sha256;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

pub struct HttpVirtualPaymentGateway {
    config: VirtualPaymentConfig,
    app_id: String,
    app_secret: String,
    client: reqwest::Client,
    sessions: Mutex<HashMap<String, (String, Instant)>>,
    access_token: Mutex<Option<(String, Instant)>>,
}

pub fn payment_signature(key: &str, message: &str) -> String {
    let mut mac =
        Hmac::<Sha256>::new_from_slice(key.as_bytes()).expect("HMAC accepts arbitrary key sizes");
    mac.update(message.as_bytes());
    mac.finalize()
        .into_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

impl HttpVirtualPaymentGateway {
    pub fn new(config: VirtualPaymentConfig, app_id: String, app_secret: String) -> Self {
        Self {
            config,
            app_id,
            app_secret,
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(12))
                .build()
                .expect("HTTP client"),
            sessions: Mutex::new(HashMap::new()),
            access_token: Mutex::new(None),
        }
    }
    async fn token(&self) -> Result<String, Error> {
        let mut cache = self.access_token.lock().await;
        if let Some((token, expires)) = cache.as_ref()
            && *expires > Instant::now()
        {
            return Ok(token.clone());
        }
        let response: Value = self
            .client
            .get("https://api.weixin.qq.com/cgi-bin/token")
            .query(&[
                ("grant_type", "client_credential"),
                ("appid", &self.app_id),
                ("secret", &self.app_secret),
            ])
            .send()
            .await
            .map_err(|_| Error::Provider)?
            .error_for_status()
            .map_err(|_| Error::Provider)?
            .json()
            .await
            .map_err(|_| Error::Provider)?;
        let token = response["access_token"]
            .as_str()
            .filter(|s| !s.is_empty())
            .ok_or(Error::Provider)?
            .to_owned();
        let seconds = response["expires_in"]
            .as_u64()
            .unwrap_or(7200)
            .saturating_sub(300)
            .max(1);
        *cache = Some((token.clone(), Instant::now() + Duration::from_secs(seconds)));
        Ok(token)
    }
    async fn api(&self, path: &str, body: Value) -> Result<Value, Error> {
        let body = body.to_string();
        let signature = payment_signature(&self.config.app_key, &format!("{path}&{body}"));
        for attempt in 0..2 {
            let token = self.token().await?;
            let response: Value = self
                .client
                .post(format!("https://api.weixin.qq.com{path}"))
                .query(&[
                    ("access_token", token.as_str()),
                    ("pay_sig", signature.as_str()),
                ])
                .header("Content-Type", "application/json")
                .body(body.clone())
                .send()
                .await
                .map_err(|_| Error::Provider)?
                .error_for_status()
                .map_err(|_| Error::Provider)?
                .json()
                .await
                .map_err(|_| Error::Provider)?;
            let error = response["errcode"].as_i64().ok_or(Error::Provider)?;
            if error == 0 {
                return Ok(response);
            }
            if attempt == 0 && matches!(error, 40001 | 40014 | 42001) {
                *self.access_token.lock().await = None;
                continue;
            }
            return Err(Error::Provider);
        }
        Err(Error::Provider)
    }
}

pub fn query_delivery(response: &Value, local: &VirtualOrder) -> Result<Option<Delivery>, Error> {
    let order = response.get("order").ok_or(Error::Provider)?;
    let status = order["status"].as_i64().ok_or(Error::Provider)?;
    if !matches!(status, 2..=4) {
        return Ok(None);
    }
    // env_type uses 1 for production (unlike the request's env=0).
    if order["order_id"].as_str() != Some(local.order_no.as_str())
        || order["env_type"].as_i64() != Some(1)
        || !matches!(order["order_type"].as_i64(), Some(0 | 7))
        || order["order_fee"].as_i64() != Some(i64::from(local.amount_cents))
        || order["paid_fee"].as_i64() != Some(i64::from(local.amount_cents))
        || order["left_fee"].as_i64() != Some(i64::from(local.amount_cents))
        || order["refund_fee"].as_i64().unwrap_or(0) != 0
    {
        return Err(Error::InvalidOrder);
    }
    let wx_order_id = order["wx_order_id"]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or(Error::InvalidOrder)?;
    if let Some(meta) = order["biz_meta"].as_str().filter(|s| !s.is_empty()) {
        let meta: Value = serde_json::from_str(meta).map_err(|_| Error::InvalidOrder)?;
        if let Some(product_id) = meta.get("productId").and_then(Value::as_str)
            && product_id != local.product_id
        {
            return Err(Error::InvalidOrder);
        }
    }
    Ok(Some(Delivery {
        order_no: local.order_no.clone(),
        open_id: local.open_id.clone(),
        product_id: local.product_id.clone(),
        amount_cents: local.amount_cents,
        original_price_cents: local.amount_cents,
        quantity: 1,
        env: 0,
        wx_order_id: wx_order_id.into(),
    }))
}

#[async_trait::async_trait]
impl VirtualPaymentGateway for HttpVirtualPaymentGateway {
    fn available(&self) -> bool {
        self.config.available() && !self.app_id.is_empty() && !self.app_secret.is_empty()
    }
    async fn prepare_session(&self, code: &str, expected_open_id: &str) -> Result<(), Error> {
        if !self.available() {
            return Err(Error::Unavailable);
        }
        if code.is_empty() || code.len() > 512 {
            return Err(Error::SessionExpired);
        }
        let response: Value = self
            .client
            .get("https://api.weixin.qq.com/sns/jscode2session")
            .query(&[
                ("appid", self.app_id.as_str()),
                ("secret", self.app_secret.as_str()),
                ("js_code", code),
                ("grant_type", "authorization_code"),
            ])
            .send()
            .await
            .map_err(|_| Error::Provider)?
            .error_for_status()
            .map_err(|_| Error::Provider)?
            .json()
            .await
            .map_err(|_| Error::Provider)?;
        if response["openid"].as_str() != Some(expected_open_id) {
            return Err(Error::IdentityMismatch);
        }
        let key = response["session_key"]
            .as_str()
            .filter(|s| !s.is_empty())
            .ok_or(Error::SessionExpired)?;
        let mut sessions = self.sessions.lock().await;
        sessions.retain(|_, (_, expires)| *expires > Instant::now());
        sessions.insert(
            expected_open_id.to_owned(),
            (key.to_owned(), Instant::now() + Duration::from_secs(300)),
        );
        Ok(())
    }
    async fn payment_params(
        &self,
        order_no: &str,
        open_id: &str,
        product: &VirtualProduct,
    ) -> Result<VirtualPayParams, Error> {
        if !self.available() {
            return Err(Error::Unavailable);
        }
        let data = build_sign_data(&self.config.offer_id, order_no, product)?;
        let sessions = self.sessions.lock().await;
        let (key, expires) = sessions.get(open_id).ok_or(Error::SessionExpired)?;
        if *expires <= Instant::now() {
            return Err(Error::SessionExpired);
        }
        Ok(VirtualPayParams {
            mode: "short_series_goods".into(),
            pay_sig: payment_signature(
                &self.config.app_key,
                &format!("requestVirtualPayment&{data}"),
            ),
            signature: payment_signature(key, &data),
            sign_data: data,
        })
    }
    async fn query_order(&self, order: &VirtualOrder) -> Result<Option<Delivery>, Error> {
        let value = self
            .api(
                "/xpay/query_order",
                json!({"openid": order.open_id, "env": 0, "order_id":order.order_no}),
            )
            .await?;
        query_delivery(&value, order)
    }
    async fn confirm_delivery(&self, order: &VirtualOrder) -> Result<(), Error> {
        self.api(
            "/xpay/notify_provide_goods",
            json!({"order_id":order.order_no,"env":0}),
        )
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn signature_matches_independent_hmac_vector() {
        assert_eq!(
            payment_signature("key", "The quick brown fox jumps over the lazy dog"),
            "f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8"
        );
    }
    #[test]
    fn query_requires_production_paid_price_and_matching_order() {
        let order = VirtualOrder {
            order_no: "ORDER123".into(),
            open_id: "buyer".into(),
            product_id: "v6".into(),
            amount_cents: 3900,
            wx_order_id: None,
        };
        let response = json!({"order":{"status":2,"order_id":"ORDER123","env_type":1,"order_type":7,"order_fee":3900,"paid_fee":3900,"refund_fee":0,"left_fee":3900,"wx_order_id":"WX123"}});
        assert!(query_delivery(&response, &order).unwrap().is_some());
        for (field, value) in [
            ("env_type", json!(2)),
            ("paid_fee", json!(1)),
            ("order_id", json!("other")),
            ("refund_fee", json!(100)),
            ("left_fee", json!(0)),
        ] {
            let mut changed = response.clone();
            changed["order"][field] = value;
            assert!(query_delivery(&changed, &order).is_err());
        }
    }
}
