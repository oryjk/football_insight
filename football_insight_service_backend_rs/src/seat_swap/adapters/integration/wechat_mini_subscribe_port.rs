use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

use crate::{
    auth::ports::user_membership_port::UserMembershipPort,
    seat_swap::ports::mini_program_subscribe_port::{
        SeatSwapConfirmedNotification, SeatSwapMiniProgramSubscribePort,
    },
};

#[derive(Clone)]
pub struct OfficialWechatMiniSubscribePort {
    app_id: String,
    app_secret: String,
    template_id: String,
    page: String,
    user_membership_port: Arc<dyn UserMembershipPort>,
    client: Client,
}

impl OfficialWechatMiniSubscribePort {
    pub fn new(
        app_id: String,
        app_secret: String,
        template_id: String,
        page: String,
        user_membership_port: Arc<dyn UserMembershipPort>,
    ) -> Self {
        Self {
            app_id,
            app_secret,
            template_id,
            page,
            user_membership_port,
            client: Client::new(),
        }
    }

    async fn fetch_access_token(&self) -> anyhow::Result<String> {
        #[derive(Deserialize)]
        struct AccessTokenResponse {
            access_token: Option<String>,
            errcode: Option<i64>,
            errmsg: Option<String>,
        }

        let response = self
            .client
            .get("https://api.weixin.qq.com/cgi-bin/token")
            .query(&[
                ("grant_type", "client_credential"),
                ("appid", self.app_id.as_str()),
                ("secret", self.app_secret.as_str()),
            ])
            .send()
            .await?
            .error_for_status()?
            .json::<AccessTokenResponse>()
            .await?;

        if let Some(errcode) = response.errcode {
            anyhow::bail!(
                "wechat mini subscribe token failed: {} {}",
                errcode,
                response.errmsg.unwrap_or_default()
            );
        }

        response
            .access_token
            .filter(|value| !value.trim().is_empty())
            .context("wechat mini subscribe access_token missing")
    }
}

#[async_trait]
impl SeatSwapMiniProgramSubscribePort for OfficialWechatMiniSubscribePort {
    async fn send_confirmed_notification(
        &self,
        payload: SeatSwapConfirmedNotification,
    ) -> anyhow::Result<()> {
        let Some(notice_enabled) = self
            .user_membership_port
            .is_seat_swap_notice_enabled(payload.recipient_user_id)
            .await?
        else {
            return Ok(());
        };
        if !notice_enabled || self.template_id.trim().is_empty() {
            return Ok(());
        }

        let access_token = self.fetch_access_token().await?;
        let seat_label = format!(
            "{} {}排 {}号",
            payload.current_region_name, payload.current_row, payload.current_seat_no
        );
        let desired_summary = if payload.desired_region_summary.trim().is_empty() {
            "未填写".to_string()
        } else {
            payload.desired_region_summary.clone()
        };

        let body = json!({
            "touser": payload.recipient_open_id,
            "template_id": self.template_id,
            "page": self.page,
            "data": {
                "thing1": { "value": "有人确认你的换座请求" },
                "name2": { "value": payload.confirmer_display_name.chars().take(10).collect::<String>() },
                "thing3": { "value": seat_label.chars().take(20).collect::<String>() },
                "thing4": { "value": desired_summary.chars().take(20).collect::<String>() }
            }
        });

        #[derive(Deserialize)]
        struct SendResponse {
            errcode: i64,
            errmsg: String,
        }

        let response = self
            .client
            .post("https://api.weixin.qq.com/cgi-bin/message/subscribe/send")
            .query(&[("access_token", access_token.as_str())])
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json::<SendResponse>()
            .await?;

        if response.errcode != 0 {
            anyhow::bail!(
                "wechat mini subscribe send failed: {} {}",
                response.errcode,
                response.errmsg
            );
        }

        Ok(())
    }
}
