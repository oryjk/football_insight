//! Pure payment protocol serialization shared by the three purchase surfaces.
use crate::payment::domain::order::WxPayParams;
use serde::Serialize;
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum PaymentParams {
    Virtual(crate::payment::domain::virtual_payment::VirtualPayParams),
    Ordinary {
        #[serde(rename = "timeStamp")]
        time_stamp: String,
        #[serde(rename = "nonceStr")]
        nonce_str: String,
        package: String,
        #[serde(rename = "signType")]
        sign_type: String,
        #[serde(rename = "paySign")]
        pay_sign: String,
    },
}

impl From<WxPayParams> for PaymentParams {
    fn from(value: WxPayParams) -> Self {
        if let Some(params) = value.virtual_payment {
            return Self::Virtual(params);
        }
        Self::Ordinary {
            time_stamp: value.time_stamp,
            nonce_str: value.nonce_str,
            package: value.package,
            sign_type: value.sign_type,
            pay_sign: value.pay_sign,
        }
    }
}
