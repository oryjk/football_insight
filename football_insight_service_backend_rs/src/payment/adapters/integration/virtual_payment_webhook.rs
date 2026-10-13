use crate::payment::{
    domain::virtual_payment::{Delivery, Refund, VirtualEvent, VirtualPaymentError},
    ports::virtual_payment_port::VirtualWebhookPort,
};
use aes::Aes256;
use base64::{Engine, engine::general_purpose::STANDARD};
use cbc::cipher::{BlockDecryptMut, KeyIvInit, block_padding::NoPadding};
use serde::Deserialize;
use sha1::{Digest, Sha1};

pub struct VirtualWebhookCodec {
    token: String,
    key: Vec<u8>,
    app_id: String,
}
impl VirtualWebhookCodec {
    pub fn new(token: String, aes_key: &str, app_id: String) -> anyhow::Result<Self> {
        let key = STANDARD.decode(format!("{}=", aes_key.trim_end_matches('=')))?;
        anyhow::ensure!(
            key.len() == 32 && !token.is_empty() && !app_id.is_empty(),
            "invalid virtual payment webhook configuration"
        );
        Ok(Self { token, key, app_id })
    }
    fn verify(
        &self,
        signature: &str,
        timestamp: &str,
        nonce: &str,
        encrypted: Option<&str>,
    ) -> anyhow::Result<()> {
        let mut parts = vec![self.token.as_str(), timestamp, nonce];
        if let Some(value) = encrypted {
            parts.push(value);
        }
        parts.sort_unstable();
        let hash = format!("{:x}", Sha1::digest(parts.join("").as_bytes()));
        let equal = signature.len() == hash.len()
            && signature
                .bytes()
                .zip(hash.bytes())
                .fold(0u8, |acc, (a, b)| acc | (a ^ b))
                == 0;
        anyhow::ensure!(equal, "invalid virtual payment webhook signature");
        Ok(())
    }
    fn decrypt(&self, encrypted: &str) -> anyhow::Result<String> {
        let mut bytes = STANDARD.decode(encrypted)?;
        let decrypted = cbc::Decryptor::<Aes256>::new_from_slices(&self.key, &self.key[..16])
            .map_err(|_| VirtualPaymentError::InvalidOrder)?
            .decrypt_padded_mut::<NoPadding>(&mut bytes)
            .map_err(|_| VirtualPaymentError::InvalidOrder)?;
        let padding = usize::from(*decrypted.last().ok_or(VirtualPaymentError::InvalidOrder)?);
        anyhow::ensure!(
            (1..=32).contains(&padding)
                && padding <= decrypted.len()
                && decrypted[decrypted.len() - padding..]
                    .iter()
                    .all(|b| usize::from(*b) == padding),
            "invalid virtual payment webhook padding"
        );
        let plain = &decrypted[..decrypted.len() - padding];
        anyhow::ensure!(plain.len() >= 20, "invalid virtual payment webhook payload");
        let size = u32::from_be_bytes(plain[16..20].try_into()?) as usize;
        anyhow::ensure!(
            size <= plain.len() - 20 && &plain[20 + size..] == self.app_id.as_bytes(),
            "invalid virtual payment webhook app id"
        );
        Ok(String::from_utf8(plain[20..20 + size].to_vec())?)
    }
}

#[derive(Deserialize)]
struct Envelope {
    #[serde(rename = "Encrypt")]
    encrypted: String,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct Goods {
    #[serde(rename = "ProductId")]
    product_id: String,
    #[serde(rename = "Quantity")]
    quantity: i32,
    #[serde(rename = "OrigPrice")]
    original: i32,
    #[serde(rename = "ActualPrice")]
    actual: i32,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct Message {
    #[serde(rename = "Event")]
    event: String,
    #[serde(rename = "OpenId")]
    open_id: String,
    #[serde(rename = "OutTradeNo")]
    order_no: String,
    #[serde(rename = "Env")]
    env: Option<i32>,
    #[serde(rename = "GoodsInfo")]
    goods: Goods,
    #[serde(rename = "MchOrderId")]
    refund_order_no: String,
    #[serde(rename = "WxOrderId")]
    wx_order_id: String,
    #[serde(rename = "WxRefundId")]
    refund_id: String,
    #[serde(rename = "RefundFee")]
    refund_fee: i32,
    #[serde(rename = "RetCode")]
    refund_code: Option<i32>,
}

fn parse<T: serde::de::DeserializeOwned>(body: &str) -> anyhow::Result<T> {
    if body.trim_start().starts_with('{') {
        Ok(serde_json::from_str(body)?)
    } else {
        Ok(quick_xml::de::from_str(body)?)
    }
}

impl VirtualWebhookPort for VirtualWebhookCodec {
    fn challenge(
        &self,
        signature: &str,
        timestamp: &str,
        nonce: &str,
        echo: &str,
    ) -> anyhow::Result<String> {
        // Safe-mode encrypted URL verification; plain verification is also permitted by WeChat.
        if self.verify(signature, timestamp, nonce, Some(echo)).is_ok() {
            return self.decrypt(echo);
        }
        self.verify(signature, timestamp, nonce, None)?;
        Ok(echo.into())
    }
    fn decode(
        &self,
        signature: &str,
        timestamp: &str,
        nonce: &str,
        body: &str,
    ) -> anyhow::Result<VirtualEvent> {
        let envelope: Envelope = parse(body)?;
        self.verify(signature, timestamp, nonce, Some(&envelope.encrypted))?;
        let plain = self.decrypt(&envelope.encrypted)?;
        let event: Message = parse(&plain)?;
        Ok(match event.event.as_str() {
            "xpay_goods_deliver_notify" => VirtualEvent::Delivery(Delivery {
                order_no: event.order_no,
                open_id: event.open_id,
                product_id: event.goods.product_id,
                amount_cents: event.goods.actual,
                original_price_cents: event.goods.original,
                quantity: event.goods.quantity,
                env: event.env.ok_or(VirtualPaymentError::InvalidOrder)?,
                wx_order_id: String::new(),
            }),
            "xpay_refund_notify" if event.refund_code == Some(0) => VirtualEvent::Refund(Refund {
                order_no: event.refund_order_no,
                open_id: event.open_id,
                wx_order_id: event.wx_order_id,
                refund_id: event.refund_id,
                amount_cents: event.refund_fee,
            }),
            "xpay_subscribe_ios_refund_query_notify" => VirtualEvent::RefundQuery,
            _ => VirtualEvent::Ignored,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cbc::cipher::BlockEncryptMut;

    fn encrypted_message(key: &[u8; 32], app_id: &str, body: &str) -> (String, String) {
        let mut plain = vec![0u8; 16];
        plain.extend_from_slice(&(body.len() as u32).to_be_bytes());
        plain.extend_from_slice(body.as_bytes());
        plain.extend_from_slice(app_id.as_bytes());
        let pad = 32 - plain.len() % 32;
        plain.extend(std::iter::repeat_n(pad as u8, pad));
        let encrypted = cbc::Encryptor::<Aes256>::new_from_slices(key, &key[..16])
            .unwrap()
            .encrypt_padded_mut::<NoPadding>(&mut plain, body.len() + 20 + app_id.len() + pad)
            .unwrap();
        let encrypted = STANDARD.encode(encrypted);
        let mut parts = ["token", "1", "nonce", encrypted.as_str()];
        parts.sort_unstable();
        let signature = format!("{:x}", Sha1::digest(parts.join("").as_bytes()));
        (
            serde_json::json!({"Encrypt":encrypted}).to_string(),
            signature,
        )
    }

    #[test]
    fn safe_mode_payload_checks_app_id_and_preserves_nested_goods() {
        let key = [7u8; 32];
        let codec =
            VirtualWebhookCodec::new("token".into(), &STANDARD.encode(key), "app".into()).unwrap();
        let body = r#"<xml><Event>xpay_goods_deliver_notify</Event><OpenId>buyer</OpenId><OutTradeNo>ORDER123</OutTradeNo><Env>0</Env><GoodsInfo><ProductId>v6</ProductId><Quantity>1</Quantity><OrigPrice>3900</OrigPrice><ActualPrice>3900</ActualPrice></GoodsInfo></xml>"#;
        let (envelope, signature) = encrypted_message(&key, "app", body);
        let VirtualEvent::Delivery(delivery) =
            codec.decode(&signature, "1", "nonce", &envelope).unwrap()
        else {
            panic!("expected delivery")
        };
        assert_eq!(delivery.product_id, "v6");
        assert_eq!(delivery.amount_cents, 3900);
        assert_eq!(delivery.open_id, "buyer");
        let (envelope, signature) = encrypted_message(&key, "wrong-app", body);
        assert!(codec.decode(&signature, "1", "nonce", &envelope).is_err());
    }
    #[test]
    fn rejects_plain_and_invalid_signature_messages() {
        let codec =
            VirtualWebhookCodec::new("token".into(), &STANDARD.encode([1u8; 32]), "app".into())
                .unwrap();
        assert!(
            codec
                .decode("bad", "1", "n", r#"{"Event":"xpay_goods_deliver_notify"}"#)
                .is_err()
        );
        assert!(
            codec
                .decode("bad", "1", "n", r#"{"Encrypt":"AAAA"}"#)
                .is_err()
        );
    }
}
