use async_trait::async_trait;
use rand::Rng;

use crate::payment::{domain::order::WxPayParams, ports::wechat_pay_port::WechatPayPort};

pub struct HttpWechatPayPort {
    app_id: String,
    mch_id: String,
    api_key: String,
    notify_url: String,
    client: reqwest::Client,
}

impl HttpWechatPayPort {
    pub fn new(app_id: String, mch_id: String, api_key: String, public_base_url: String) -> Self {
        let notify_url = format!(
            "{}/api/v1/payment/wx-notify",
            public_base_url.trim_end_matches('/')
        );
        Self {
            app_id,
            mch_id,
            api_key,
            notify_url,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl WechatPayPort for HttpWechatPayPort {
    async fn unified_order(
        &self,
        order_no: &str,
        description: &str,
        amount_cents: i32,
        openid: &str,
    ) -> anyhow::Result<WxPayParams> {
        let nonce_str = generate_nonce_str();
        let time_stamp = chrono::Utc::now().timestamp().to_string();

        let amount_cents_str = amount_cents.to_string();
        let mut params = std::collections::BTreeMap::new();
        params.insert("appid", self.app_id.as_str());
        params.insert("mch_id", self.mch_id.as_str());
        params.insert("body", description);
        params.insert("out_trade_no", order_no);
        params.insert("total_fee", amount_cents_str.as_str());
        params.insert("spbill_create_ip", "127.0.0.1");
        params.insert("notify_url", self.notify_url.as_str());
        params.insert("openid", openid);
        params.insert("trade_type", "JSAPI");
        params.insert("nonce_str", nonce_str.as_str());

        let sign = md5_sign(&params, &self.api_key);

        let mut xml = String::from("<xml>");
        for (k, v) in &params {
            xml.push_str(&format!("<{}><![CDATA[{}]]></{}>", k, v, k));
        }
        xml.push_str(&format!("<sign><![CDATA[{}]]></sign>", sign));
        xml.push_str("</xml>");

        let response = self
            .client
            .post("https://api.mch.weixin.qq.com/pay/unifiedorder")
            .header("Content-Type", "application/xml")
            .body(xml)
            .send()
            .await?;

        let xml_text = response.text().await?;
        let result = parse_xml(&xml_text);

        if result.get("return_code").map(|s| s.as_str()) != Some("SUCCESS") {
            let msg = result
                .get("return_msg")
                .cloned()
                .unwrap_or_else(|| "未知错误".to_string());
            anyhow::bail!("微信支付下单失败: {}", msg);
        }

        if result.get("result_code").map(|s| s.as_str()) != Some("SUCCESS") {
            let msg = result
                .get("err_code_des")
                .cloned()
                .unwrap_or_else(|| "未知错误".to_string());
            anyhow::bail!("微信支付下单失败: {}", msg);
        }

        let prepay_id = result.get("prepay_id").cloned().unwrap_or_default();

        let package = format!("prepay_id={}", prepay_id);

        let mut pay_params = std::collections::BTreeMap::new();
        pay_params.insert("appId", self.app_id.as_str());
        pay_params.insert("nonceStr", nonce_str.as_str());
        pay_params.insert("package", package.as_str());
        pay_params.insert("signType", "MD5");
        pay_params.insert("timeStamp", time_stamp.as_str());

        let pay_sign = md5_sign(&pay_params, &self.api_key);

        Ok(WxPayParams {
            time_stamp,
            nonce_str,
            package,
            sign_type: "MD5".to_string(),
            pay_sign,
        })
    }
}

fn generate_nonce_str() -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();
    (0..32)
        .map(|_| CHARS[rng.random_range(0..CHARS.len())] as char)
        .collect()
}

fn md5_sign(params: &std::collections::BTreeMap<&str, &str>, key: &str) -> String {
    let sign_str = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&")
        + "&key="
        + key;
    format!("{:x}", md5::compute(sign_str)).to_uppercase()
}

fn parse_xml(xml: &str) -> std::collections::HashMap<String, String> {
    let mut result = std::collections::HashMap::new();
    let mut rest = xml;

    while let Some(open_idx) = rest.find('<') {
        rest = &rest[open_idx + 1..];

        let Some(tag_end_idx) = rest.find('>') else {
            break;
        };

        let tag = rest[..tag_end_idx].trim();
        rest = &rest[tag_end_idx + 1..];

        if tag.is_empty() || tag.starts_with('/') || tag.starts_with('!') || tag.starts_with('?') {
            continue;
        }

        let close_tag = format!("</{tag}>");
        let Some(close_idx) = rest.find(&close_tag) else {
            continue;
        };

        let raw_value = &rest[..close_idx];
        if tag == "xml" {
            rest = raw_value;
            continue;
        }

        let value = raw_value
            .trim()
            .strip_prefix("<![CDATA[")
            .and_then(|item| item.strip_suffix("]]>"))
            .unwrap_or(raw_value.trim());

        result.insert(tag.to_string(), value.to_string());
        rest = &rest[close_idx + close_tag.len()..];
    }

    result
}
