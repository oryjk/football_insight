use std::collections::HashMap;

use async_trait::async_trait;

use crate::{
    mcp::{domain::PromoTarget, ports::PromoCodePort},
    share_code::{
        adapters::wechat_code::WechatMiniCode,
        domain::{CodeImage, ShareCodeError},
        ports::TicketWatchCodePort,
    },
};

/// One cached WeChat mini program code per landing target.
pub struct WechatPromoCodes {
    codes: HashMap<PromoTarget, WechatMiniCode>,
}

impl WechatPromoCodes {
    pub fn new(app_id: &str, secret: &str) -> Self {
        let codes = PromoTarget::ALL
            .into_iter()
            .map(|target| {
                let code = WechatMiniCode::for_target(
                    app_id.to_string(),
                    secret.to_string(),
                    target.scene(),
                    target.page().to_string(),
                );
                (target, code)
            })
            .collect();
        Self { codes }
    }
}

#[async_trait]
impl PromoCodePort for WechatPromoCodes {
    async fn image(&self, target: PromoTarget) -> Result<CodeImage, ShareCodeError> {
        match self.codes.get(&target) {
            Some(code) => code.image().await,
            None => Err(ShareCodeError::Unavailable),
        }
    }
}
