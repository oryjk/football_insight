use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum StudentError {
    #[error("{0}")]
    InvalidInput(String),
    #[error("该服务暂未开放购买")]
    Unavailable,
    #[error("请先使用微信登录后购买")]
    WechatRequired,
    #[error("服务请求失败，请稍后重试")]
    Internal,
    #[error("支付未完成，请稍后重试")]
    PaymentFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceCode {
    CertificateRenewal,
    ToolUpgrade,
}
impl ServiceCode {
    pub const ALL: [Self; 2] = [Self::CertificateRenewal, Self::ToolUpgrade];
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CertificateRenewal => "certificate_renewal",
            Self::ToolUpgrade => "tool_upgrade",
        }
    }
    pub fn title(self) -> &'static str {
        match self {
            Self::CertificateRenewal => "证书续期",
            Self::ToolUpgrade => "工具升级",
        }
    }
    pub fn price(self) -> i32 {
        match self {
            Self::CertificateRenewal => 9900,
            Self::ToolUpgrade => 69900,
        }
    }
    pub fn unit(self) -> &'static str {
        match self {
            Self::CertificateRenewal => "年",
            Self::ToolUpgrade => "套",
        }
    }
    pub fn product_type(self) -> String {
        format!("student_service:{}", self.as_str())
    }
    pub fn product_key(self) -> String {
        format!("{}@{}", self.product_type(), self.price())
    }
    pub fn from_product_type(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|code| code.product_type() == value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StudentService {
    pub code: ServiceCode,
    pub title: String,
    pub description: String,
    pub price_cents: i32,
    pub unit: String,
    pub enabled: bool,
    pub product_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StudentConfig {
    pub contact_qr_url: String,
    pub services: Vec<StudentService>,
}
impl Default for StudentConfig {
    fn default() -> Self {
        Self {
            contact_qr_url: String::new(),
            services: ServiceCode::ALL
                .into_iter()
                .map(|code| StudentService {
                    code,
                    title: code.title().into(),
                    description: String::new(),
                    price_cents: code.price(),
                    unit: code.unit().into(),
                    enabled: false,
                    product_id: String::new(),
                })
                .collect(),
        }
    }
}
impl StudentConfig {
    pub fn validate(mut self) -> Result<Self, StudentError> {
        let invalid = |message: &str| StudentError::InvalidInput(message.into());
        self.contact_qr_url = self.contact_qr_url.trim().into();
        if !self.contact_qr_url.is_empty() {
            let url = url::Url::parse(&self.contact_qr_url)
                .map_err(|_| invalid("二维码地址须为 HTTPS 图片地址"))?;
            if self.contact_qr_url.chars().count() > 2048
                || url.scheme() != "https"
                || url.host_str().is_none()
                || !url.username().is_empty()
                || url.password().is_some()
                || self
                    .contact_qr_url
                    .chars()
                    .any(|c| c.is_control() || c.is_whitespace())
            {
                return Err(invalid("二维码地址须为不含账号密码的 HTTPS 图片地址"));
            }
        }
        if self.services.len() != 2
            || ServiceCode::ALL
                .into_iter()
                .any(|code| self.services.iter().filter(|s| s.code == code).count() != 1)
        {
            return Err(invalid("必须包含证书续期和工具升级两项服务"));
        }
        for service in &mut self.services {
            if service.title != service.code.title()
                || service.price_cents != service.code.price()
                || service.unit != service.code.unit()
            {
                return Err(invalid("服务名称、金额和计费单位不可更改"));
            }
            service.description = service.description.trim().into();
            service.product_id = service.product_id.trim().into();
            if service.description.chars().count() > 4000
                || service
                    .description
                    .chars()
                    .any(|c| c.is_control() && c != '\n' && c != '\r' && c != '\t')
            {
                return Err(invalid("服务说明最多 4000 字，不能包含无效控制字符"));
            }
            if service.product_id.chars().count() > 200
                || service
                    .product_id
                    .chars()
                    .any(|c| c.is_whitespace() || c.is_control())
            {
                return Err(invalid("支付商品 ID 最多 200 字且不能包含空白"));
            }
            if service.enabled
                && (service.description.is_empty()
                    || service.product_id.is_empty()
                    || self.contact_qr_url.is_empty())
            {
                return Err(invalid(
                    "开放购买前请填写服务说明、微信支付商品 ID 和联系二维码",
                ));
            }
        }
        self.services.sort_by_key(|s| match s.code {
            ServiceCode::CertificateRenewal => 0,
            ServiceCode::ToolUpgrade => 1,
        });
        Ok(self)
    }
}

#[derive(Debug, Clone)]
pub struct StudentOrder {
    pub order_no: String,
    pub code: ServiceCode,
    pub amount_cents: i32,
    pub status: String,
    pub paid_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

pub struct StudentOrderPage {
    pub items: Vec<StudentOrder>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}
