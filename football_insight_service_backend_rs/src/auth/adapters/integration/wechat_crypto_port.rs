use aes::Aes256;
use anyhow::{Context, anyhow};
use base64::{
    Engine,
    engine::{DecodePaddingMode, GeneralPurpose, GeneralPurposeConfig, general_purpose::STANDARD},
};
use cbc::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, block_padding::Pkcs7};
use sha1::{Digest, Sha1};

use crate::auth::{
    domain::wechat::{WechatIncomingMessage, WechatTextReply},
    ports::wechat_crypto_port::WechatCryptoPort,
};

type Aes256CbcDec = cbc::Decryptor<Aes256>;
type Aes256CbcEnc = cbc::Encryptor<Aes256>;

#[derive(Clone)]
pub struct OfficialWechatCryptoPort {
    token: String,
    aes_key: Vec<u8>,
    iv: [u8; 16],
}

impl OfficialWechatCryptoPort {
    pub fn new(token: String, encoding_aes_key: String) -> anyhow::Result<Self> {
        let forgiving_engine = GeneralPurpose::new(
            &base64::alphabet::STANDARD,
            GeneralPurposeConfig::new()
                .with_decode_padding_mode(DecodePaddingMode::Indifferent)
                .with_decode_allow_trailing_bits(true),
        );

        let decoded = STANDARD
            .decode(format!("{encoding_aes_key}="))
            .or_else(|_| forgiving_engine.decode(format!("{encoding_aes_key}=")))
            .or_else(|_| forgiving_engine.decode(&encoding_aes_key))
            .context("invalid WECHAT_ENCODING_AES_KEY")?;

        if decoded.len() != 32 {
            return Err(anyhow!("WECHAT_ENCODING_AES_KEY must decode to 32 bytes"));
        }

        let mut iv = [0_u8; 16];
        iv.copy_from_slice(&decoded[..16]);

        Ok(Self {
            token,
            aes_key: decoded,
            iv,
        })
    }

    pub(crate) fn verify_signature(
        &self,
        signature: &str,
        timestamp: &str,
        nonce: &str,
        value: &str,
    ) -> anyhow::Result<()> {
        let mut parts = [self.token.as_str(), timestamp, nonce, value];
        parts.sort_unstable();

        let mut hasher = Sha1::new();
        hasher.update(parts.join("").as_bytes());
        let computed = format!("{:x}", hasher.finalize());

        if computed != signature {
            return Err(anyhow!("wechat signature mismatch"));
        }

        Ok(())
    }

    pub(crate) fn verify_url_signature(
        &self,
        signature: &str,
        timestamp: &str,
        nonce: &str,
    ) -> anyhow::Result<()> {
        let mut parts = [self.token.as_str(), timestamp, nonce];
        parts.sort_unstable();

        let mut hasher = Sha1::new();
        hasher.update(parts.join("").as_bytes());
        let computed = format!("{:x}", hasher.finalize());

        if computed != signature {
            return Err(anyhow!("wechat signature mismatch"));
        }

        Ok(())
    }

    pub(crate) fn verify_url_with_echostr_signature(
        &self,
        signature: &str,
        timestamp: &str,
        nonce: &str,
        echostr: &str,
    ) -> anyhow::Result<()> {
        self.verify_signature(signature, timestamp, nonce, echostr)
    }

    pub(crate) fn decrypt_cipher_text(
        &self,
        cipher_text: &str,
    ) -> anyhow::Result<(String, String)> {
        let mut errors = Vec::new();

        for candidate in normalized_cipher_text_candidates(cipher_text) {
            match self.decrypt_cipher_text_candidate(&candidate) {
                Ok(result) => return Ok(result),
                Err(error) => errors.push(error.to_string()),
            }
        }

        Err(anyhow!(
            "failed to decrypt wechat payload: {}",
            errors.join(" | ")
        ))
    }

    fn decrypt_cipher_text_candidate(&self, cipher_text: &str) -> anyhow::Result<(String, String)> {
        let encrypted = decode_wechat_cipher_text(cipher_text)?;

        let mut buf = encrypted;
        let decrypted = Aes256CbcDec::new_from_slices(&self.aes_key, &self.iv)
            .map_err(|_| anyhow!("invalid aes key or iv"))?
            .decrypt_padded_mut::<Pkcs7>(&mut buf)
            .map_err(|_| anyhow!("invalid aes block padding"))?;

        if decrypted.len() < 20 {
            return Err(anyhow!("wechat payload is too short"));
        }

        let xml_len =
            u32::from_be_bytes([decrypted[16], decrypted[17], decrypted[18], decrypted[19]])
                as usize;
        if decrypted.len() < 20 + xml_len {
            return Err(anyhow!("wechat payload xml length is invalid"));
        }

        let xml = String::from_utf8(decrypted[20..20 + xml_len].to_vec())
            .context("wechat xml is not utf-8")?;
        let app_id = String::from_utf8(decrypted[20 + xml_len..].to_vec())
            .context("wechat app id is not utf-8")?;
        Ok((xml, app_id))
    }

    pub(crate) fn encrypt_cipher_text(&self, xml: &str, app_id: &str) -> anyhow::Result<String> {
        let random_prefix = uuid::Uuid::new_v4().into_bytes();
        let xml_bytes = xml.as_bytes();
        let xml_len = (xml_bytes.len() as u32).to_be_bytes();

        let mut plain = Vec::with_capacity(16 + 4 + xml_bytes.len() + app_id.len());
        plain.extend_from_slice(&random_prefix);
        plain.extend_from_slice(&xml_len);
        plain.extend_from_slice(xml_bytes);
        plain.extend_from_slice(app_id.as_bytes());

        let block_size = 16;
        let padded_len = ((plain.len() / block_size) + 1) * block_size;
        let mut buf = vec![0_u8; padded_len];
        buf[..plain.len()].copy_from_slice(&plain);

        let encrypted = Aes256CbcEnc::new_from_slices(&self.aes_key, &self.iv)
            .map_err(|_| anyhow!("invalid aes key or iv"))?
            .encrypt_padded_mut::<Pkcs7>(&mut buf, plain.len())
            .map_err(|_| anyhow!("failed to encrypt wechat payload"))?;

        Ok(STANDARD.encode(encrypted))
    }
}

impl WechatCryptoPort for OfficialWechatCryptoPort {
    fn verify_url(
        &self,
        signature: &str,
        timestamp: &str,
        nonce: &str,
        echostr: &str,
    ) -> anyhow::Result<String> {
        if self
            .verify_url_with_echostr_signature(signature, timestamp, nonce, echostr)
            .is_ok()
        {
            let (plain, _) = self.decrypt_cipher_text(echostr)?;
            return Ok(plain);
        }

        self.verify_url_signature(signature, timestamp, nonce)?;
        match self.decrypt_cipher_text(echostr) {
            Ok((plain, _)) => Ok(plain),
            Err(_) => Ok(echostr.to_string()),
        }
    }

    fn decrypt_message(
        &self,
        signature: &str,
        timestamp: &str,
        nonce: &str,
        request_body: &str,
    ) -> anyhow::Result<WechatIncomingMessage> {
        let encrypt = extract_xml_field(request_body, "Encrypt")
            .ok_or_else(|| anyhow!("wechat Encrypt field is missing"))?;
        self.verify_signature(signature, timestamp, nonce, &encrypt)?;
        let (plain_xml, app_id) = self.decrypt_cipher_text(&encrypt)?;

        let to_user_name = extract_xml_field(&plain_xml, "ToUserName")
            .ok_or_else(|| anyhow!("wechat ToUserName is missing"))?;
        let from_user_name = extract_xml_field(&plain_xml, "FromUserName")
            .ok_or_else(|| anyhow!("wechat FromUserName is missing"))?;
        let create_time = extract_xml_field(&plain_xml, "CreateTime")
            .and_then(|value| value.parse::<i64>().ok())
            .ok_or_else(|| anyhow!("wechat CreateTime is invalid"))?;
        let msg_type = extract_xml_field(&plain_xml, "MsgType")
            .ok_or_else(|| anyhow!("wechat MsgType is missing"))?;
        let event = extract_xml_field(&plain_xml, "Event");
        let content = extract_xml_field(&plain_xml, "Content");

        Ok(WechatIncomingMessage {
            to_user_name,
            from_user_name,
            create_time,
            msg_type,
            event,
            content,
            app_id,
        })
    }

    fn parse_plain_message(
        &self,
        signature: &str,
        timestamp: &str,
        nonce: &str,
        request_body: &str,
    ) -> anyhow::Result<WechatIncomingMessage> {
        self.verify_url_signature(signature, timestamp, nonce)?;

        let to_user_name = extract_xml_field(request_body, "ToUserName")
            .ok_or_else(|| anyhow!("wechat ToUserName is missing"))?;
        let from_user_name = extract_xml_field(request_body, "FromUserName")
            .ok_or_else(|| anyhow!("wechat FromUserName is missing"))?;
        let create_time = extract_xml_field(request_body, "CreateTime")
            .and_then(|value| value.parse::<i64>().ok())
            .ok_or_else(|| anyhow!("wechat CreateTime is invalid"))?;
        let msg_type = extract_xml_field(request_body, "MsgType")
            .ok_or_else(|| anyhow!("wechat MsgType is missing"))?;
        let event = extract_xml_field(request_body, "Event");
        let content = extract_xml_field(request_body, "Content");

        Ok(WechatIncomingMessage {
            to_user_name,
            from_user_name,
            create_time,
            msg_type,
            event,
            content,
            app_id: String::new(),
        })
    }

    fn encrypt_reply(
        &self,
        reply: &WechatTextReply,
        timestamp: &str,
        nonce: &str,
        app_id: &str,
    ) -> anyhow::Result<String> {
        let plain_xml = format!(
            "<xml><ToUserName><![CDATA[{to}]]></ToUserName><FromUserName><![CDATA[{from}]]></FromUserName><CreateTime>{timestamp}</CreateTime><MsgType><![CDATA[text]]></MsgType><Content><![CDATA[{content}]]></Content></xml>",
            to = reply.to_user_name,
            from = reply.from_user_name,
            timestamp = timestamp,
            content = reply.content,
        );
        let encrypt = self.encrypt_cipher_text(&plain_xml, app_id)?;

        let mut parts = [self.token.as_str(), timestamp, nonce, encrypt.as_str()];
        parts.sort_unstable();
        let mut hasher = Sha1::new();
        hasher.update(parts.join("").as_bytes());
        let msg_signature = format!("{:x}", hasher.finalize());

        Ok(format!(
            "<xml><Encrypt><![CDATA[{encrypt}]]></Encrypt><MsgSignature><![CDATA[{msg_signature}]]></MsgSignature><TimeStamp>{timestamp}</TimeStamp><Nonce><![CDATA[{nonce}]]></Nonce></xml>"
        ))
    }

    fn build_plain_reply(
        &self,
        reply: &WechatTextReply,
        timestamp: &str,
    ) -> anyhow::Result<String> {
        Ok(format!(
            "<xml><ToUserName><![CDATA[{to}]]></ToUserName><FromUserName><![CDATA[{from}]]></FromUserName><CreateTime>{timestamp}</CreateTime><MsgType><![CDATA[text]]></MsgType><Content><![CDATA[{content}]]></Content></xml>",
            to = reply.to_user_name,
            from = reply.from_user_name,
            content = reply.content,
            timestamp = timestamp,
        ))
    }
}

fn extract_xml_field(xml: &str, field: &str) -> Option<String> {
    let cdata_open = format!("<{field}><![CDATA[");
    let cdata_close = format!("]]></{field}>");
    if let Some(start) = xml.find(&cdata_open) {
        let value_start = start + cdata_open.len();
        let rest = &xml[value_start..];
        let end = rest.find(&cdata_close)?;
        return Some(rest[..end].to_string());
    }

    let plain_open = format!("<{field}>");
    let plain_close = format!("</{field}>");
    let start = xml.find(&plain_open)?;
    let value_start = start + plain_open.len();
    let rest = &xml[value_start..];
    let end = rest.find(&plain_close)?;
    Some(rest[..end].trim().to_string())
}

fn normalized_cipher_text_candidates(cipher_text: &str) -> Vec<String> {
    let trimmed = cipher_text.trim();
    let whitespace_stripped: String = trimmed
        .chars()
        .filter(|char| !char.is_whitespace())
        .collect();

    let plus_restored: String = trimmed
        .chars()
        .filter_map(|char| match char {
            '\r' | '\n' | '\t' => None,
            ' ' => Some('+'),
            other => Some(other),
        })
        .collect();

    if whitespace_stripped == plus_restored {
        vec![whitespace_stripped]
    } else {
        vec![whitespace_stripped, plus_restored]
    }
}

fn decode_wechat_cipher_text(cipher_text: &str) -> anyhow::Result<Vec<u8>> {
    let forgiving_engine = GeneralPurpose::new(
        &base64::alphabet::STANDARD,
        GeneralPurposeConfig::new()
            .with_decode_padding_mode(DecodePaddingMode::Indifferent)
            .with_decode_allow_trailing_bits(true),
    );

    STANDARD
        .decode(cipher_text)
        .or_else(|_| forgiving_engine.decode(cipher_text))
        .context("invalid base64 encrypt payload")
}

#[cfg(test)]
mod tests {
    use sha1::{Digest, Sha1};

    use super::OfficialWechatCryptoPort;
    use crate::auth::{
        domain::wechat::WechatTextReply, ports::wechat_crypto_port::WechatCryptoPort,
    };

    #[test]
    fn verify_url_returns_plaintext_echo() {
        let port = OfficialWechatCryptoPort::new(
            "token123".to_string(),
            "l8TIs4SKawzdwJE+K0nmaiqAJ7qCe5OYLql2Jtufu+Y".to_string(),
        )
        .unwrap();

        let plain = "hello-wechat";
        let echostr = port.encrypt_cipher_text(plain, "wx_app_id").unwrap();
        let mut parts = ["token123", "1712312312", "nonce123"];
        parts.sort_unstable();
        let mut hasher = Sha1::new();
        hasher.update(parts.join("").as_bytes());
        let signature = format!("{:x}", hasher.finalize());

        let result = port
            .verify_url(&signature, "1712312312", "nonce123", &echostr)
            .unwrap();

        assert_eq!(result, plain);
    }

    #[test]
    fn verify_url_accepts_encrypted_msg_signature_with_echostr() {
        let port = OfficialWechatCryptoPort::new(
            "token123".to_string(),
            "l8TIs4SKawzdwJE+K0nmaiqAJ7qCe5OYLql2Jtufu+Y".to_string(),
        )
        .unwrap();

        let plain = "hello-wechat";
        let echostr = port.encrypt_cipher_text(plain, "wx_app_id").unwrap();
        let mut parts = ["token123", "1712312312", "nonce123", echostr.as_str()];
        parts.sort_unstable();
        let mut hasher = Sha1::new();
        hasher.update(parts.join("").as_bytes());
        let signature = format!("{:x}", hasher.finalize());

        let result = port
            .verify_url(&signature, "1712312312", "nonce123", &echostr)
            .unwrap();

        assert_eq!(result, plain);
    }

    #[test]
    fn verify_url_accepts_plain_echostr() {
        let port = OfficialWechatCryptoPort::new(
            "token123".to_string(),
            "l8TIs4SKawzdwJE+K0nmaiqAJ7qCe5OYLql2Jtufu+Y".to_string(),
        )
        .unwrap();

        let echostr = "5641419719536795051";
        let mut parts = ["token123", "1712312312", "nonce123"];
        parts.sort_unstable();
        let mut hasher = Sha1::new();
        hasher.update(parts.join("").as_bytes());
        let signature = format!("{:x}", hasher.finalize());

        let result = port
            .verify_url(&signature, "1712312312", "nonce123", echostr)
            .unwrap();

        assert_eq!(result, echostr);
    }

    #[test]
    fn encrypt_reply_generates_outer_xml_envelope() {
        let port = OfficialWechatCryptoPort::new(
            "token123".to_string(),
            "l8TIs4SKawzdwJE+K0nmaiqAJ7qCe5OYLql2Jtufu+Y".to_string(),
        )
        .unwrap();

        let encrypted = port
            .encrypt_reply(
                &WechatTextReply {
                    to_user_name: "openid".to_string(),
                    from_user_name: "gh_xxx".to_string(),
                    content: "邀请码：FI-TEST".to_string(),
                },
                "1712312312",
                "nonce123",
                "wx_app_id",
            )
            .unwrap();

        assert!(encrypted.contains("<Encrypt><![CDATA["));
        assert!(encrypted.contains("<MsgSignature><![CDATA["));
    }

    #[test]
    fn parse_plain_message_accepts_plaintext_xml() {
        let port = OfficialWechatCryptoPort::new(
            "token123".to_string(),
            "l8TIs4SKawzdwJE+K0nmaiqAJ7qCe5OYLql2Jtufu+Y".to_string(),
        )
        .unwrap();

        let request_body = "<xml><ToUserName><![CDATA[gh_xxx]]></ToUserName><FromUserName><![CDATA[openid]]></FromUserName><CreateTime>1712312312</CreateTime><MsgType><![CDATA[text]]></MsgType><Content><![CDATA[下一场]]></Content></xml>";
        let mut parts = ["token123", "1712312312", "nonce123"];
        parts.sort_unstable();
        let mut hasher = Sha1::new();
        hasher.update(parts.join("").as_bytes());
        let signature = format!("{:x}", hasher.finalize());

        let incoming = port
            .parse_plain_message(&signature, "1712312312", "nonce123", request_body)
            .unwrap();

        assert_eq!(incoming.content.as_deref(), Some("下一场"));
        assert_eq!(incoming.app_id, "");
    }

    #[test]
    fn build_plain_reply_generates_plain_xml_envelope() {
        let port = OfficialWechatCryptoPort::new(
            "token123".to_string(),
            "l8TIs4SKawzdwJE+K0nmaiqAJ7qCe5OYLql2Jtufu+Y".to_string(),
        )
        .unwrap();

        let plain = port
            .build_plain_reply(
                &WechatTextReply {
                    to_user_name: "openid".to_string(),
                    from_user_name: "gh_xxx".to_string(),
                    content: "当前下一场比赛的 match_id 是：75".to_string(),
                },
                "1712312312",
            )
            .unwrap();

        assert!(plain.contains("<ToUserName><![CDATA[openid]]></ToUserName>"));
        assert!(plain.contains("<Content><![CDATA[当前下一场比赛的 match_id 是：75]]></Content>"));
        assert!(!plain.contains("<Encrypt><![CDATA["));
    }

    #[test]
    fn decrypt_message_accepts_whitespace_wrapped_encrypt_payload() {
        let port = OfficialWechatCryptoPort::new(
            "token123".to_string(),
            "l8TIs4SKawzdwJE+K0nmaiqAJ7qCe5OYLql2Jtufu+Y".to_string(),
        )
        .unwrap();

        let plain_xml = "<xml><ToUserName><![CDATA[gh_xxx]]></ToUserName><FromUserName><![CDATA[openid]]></FromUserName><CreateTime>1712312312</CreateTime><MsgType><![CDATA[text]]></MsgType><Content><![CDATA[下一场 id]]></Content></xml>";
        let encrypt = port.encrypt_cipher_text(plain_xml, "wx_app_id").unwrap();

        let wrapped_encrypt = format!("\n  {encrypt}\n");
        let mut parts = [
            "token123",
            "1712312312",
            "nonce123",
            wrapped_encrypt.as_str(),
        ];
        parts.sort_unstable();
        let mut hasher = Sha1::new();
        hasher.update(parts.join("").as_bytes());
        let signature = format!("{:x}", hasher.finalize());

        let request_body = format!("<xml><Encrypt><![CDATA[{wrapped_encrypt}]]></Encrypt></xml>");

        let incoming = port
            .decrypt_message(&signature, "1712312312", "nonce123", &request_body)
            .unwrap();

        assert_eq!(incoming.content.as_deref(), Some("下一场 id"));
    }

    #[test]
    fn decrypt_message_accepts_plus_signs_replaced_with_spaces() {
        let port = OfficialWechatCryptoPort::new(
            "token123".to_string(),
            "l8TIs4SKawzdwJE+K0nmaiqAJ7qCe5OYLql2Jtufu+Y".to_string(),
        )
        .unwrap();

        let plain_xml = "<xml><ToUserName><![CDATA[gh_xxx]]></ToUserName><FromUserName><![CDATA[openid]]></FromUserName><CreateTime>1712312312</CreateTime><MsgType><![CDATA[text]]></MsgType><Content><![CDATA[下一场id]]></Content></xml>";
        let encrypt = (0..128)
            .map(|_| port.encrypt_cipher_text(plain_xml, "wx_app_id").unwrap())
            .find(|value| value.contains('+'))
            .expect("expected generated ciphertext with at least one '+'");

        let plus_injecting_encrypt = encrypt.replace('+', " ");

        let mut parts = [
            "token123",
            "1712312312",
            "nonce123",
            plus_injecting_encrypt.as_str(),
        ];
        parts.sort_unstable();
        let mut hasher = Sha1::new();
        hasher.update(parts.join("").as_bytes());
        let signature = format!("{:x}", hasher.finalize());

        let request_body =
            format!("<xml><Encrypt><![CDATA[{plus_injecting_encrypt}]]></Encrypt></xml>");

        let incoming = port
            .decrypt_message(&signature, "1712312312", "nonce123", &request_body)
            .unwrap();

        assert_eq!(incoming.content.as_deref(), Some("下一场id"));
    }
}
