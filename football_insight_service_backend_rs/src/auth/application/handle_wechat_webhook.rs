use std::sync::Arc;

use crate::auth::{
    domain::wechat::{WechatMessageMode, WechatTextReply},
    ports::{
        auth_repository::AuthRepository, current_standard_match_port::CurrentStandardMatchPort,
        official_account_match_id_access_port::OfficialAccountMatchIdAccessPort,
        wechat_crypto_port::WechatCryptoPort,
    },
};
use crate::system_config::ports::system_config_port::SystemConfigPort;

const PRODUCT_INTRO_ARTICLE_URL: &str = "https://mp.weixin.qq.com/s?__biz=MzA3Nzc1NTk1Mg==&mid=2247483892&idx=1&sn=c41ff0a1987269db51417a11d0c7b885";
const WECHAT_INVITE_REPLY_TEMPLATE_KEY: &str = "wechat_invite_reply_template";
const WECHAT_NEXT_MATCH_REPLY_ENABLED_KEY: &str = "wechat_next_match_reply_enabled";

#[derive(Debug, Clone)]
pub struct WechatVerificationInput {
    pub signature: String,
    pub timestamp: String,
    pub nonce: String,
    pub echostr: String,
}

#[derive(Debug, Clone)]
pub struct WechatMessageInput {
    pub signature: String,
    pub timestamp: String,
    pub nonce: String,
    pub body: String,
}

pub struct HandleWechatWebhookUseCase {
    repository: Arc<dyn AuthRepository>,
    crypto_port: Arc<dyn WechatCryptoPort>,
    current_standard_match_port: Arc<dyn CurrentStandardMatchPort>,
    system_config_port: Arc<dyn SystemConfigPort>,
    official_account_access_port: Arc<dyn OfficialAccountMatchIdAccessPort>,
    official_bind_use_case:
        Option<Arc<crate::auth::application::official_bind::CompleteOfficialBindUseCase>>,
}

impl HandleWechatWebhookUseCase {
    pub fn new(
        repository: Arc<dyn AuthRepository>,
        crypto_port: Arc<dyn WechatCryptoPort>,
        current_standard_match_port: Arc<dyn CurrentStandardMatchPort>,
        system_config_port: Arc<dyn SystemConfigPort>,
        official_account_access_port: Arc<dyn OfficialAccountMatchIdAccessPort>,
    ) -> Self {
        Self {
            repository,
            crypto_port,
            current_standard_match_port,
            system_config_port,
            official_account_access_port,
            official_bind_use_case: None,
        }
    }

    pub fn verify_endpoint(&self, input: WechatVerificationInput) -> anyhow::Result<String> {
        self.crypto_port.verify_url(
            &input.signature,
            &input.timestamp,
            &input.nonce,
            &input.echostr,
        )
    }

    pub fn with_official_bind(
        mut self,
        use_case: Arc<crate::auth::application::official_bind::CompleteOfficialBindUseCase>,
    ) -> Self {
        self.official_bind_use_case = Some(use_case);
        self
    }

    pub async fn handle_message(
        &self,
        input: WechatMessageInput,
    ) -> anyhow::Result<Option<String>> {
        tracing::info!("received wechat webhook message");

        let message_mode = detect_message_mode(&input.body);
        let incoming = match message_mode {
            WechatMessageMode::Encrypted => self.crypto_port.decrypt_message(
                &input.signature,
                &input.timestamp,
                &input.nonce,
                &input.body,
            )?,
            WechatMessageMode::Plain => self.crypto_port.parse_plain_message(
                &input.signature,
                &input.timestamp,
                &input.nonce,
                &input.body,
            )?,
        };

        let is_binding_message = incoming.msg_type == "text"
            && incoming
                .content
                .as_deref()
                .is_some_and(|text| normalize_text_command(text).starts_with("绑定"));
        if is_binding_message {
            tracing::info!("verified official-account bind message received");
        } else {
            tracing::info!(
                mode = ?message_mode,
                msg_type = %incoming.msg_type,
                event = incoming.event.as_deref().unwrap_or(""),
                content = incoming.content.as_deref().unwrap_or(""),
                from_user = %incoming.from_user_name,
                to_user = %incoming.to_user_name,
                app_id = %incoming.app_id,
                "wechat message decrypted"
            );
        }

        if incoming.msg_type == "text" {
            let normalized_content = incoming
                .content
                .as_deref()
                .map(str::trim)
                .unwrap_or_default();
            let normalized_command = normalize_text_command(normalized_content);

            if let Some(code) = normalized_command.strip_prefix("绑定") {
                let reply_content = self
                    .official_bind_reply(&incoming.from_user_name, code)
                    .await;
                return Ok(Some(self.build_text_reply(
                    message_mode,
                    &incoming.from_user_name,
                    &incoming.to_user_name,
                    &input.timestamp,
                    &input.nonce,
                    &incoming.app_id,
                    reply_content,
                )?));
            }

            if normalized_command == "邀请码" {
                tracing::info!(
                    open_id = %incoming.from_user_name,
                    "wechat invite code text command received"
                );
                let invite = self
                    .repository
                    .issue_invite_code_for_wechat_follower(&incoming.from_user_name)
                    .await?;

                tracing::info!(
                    open_id = %incoming.from_user_name,
                    invite_code = %invite.code,
                    "issued invite code for wechat text command"
                );

                let encrypted = self.build_text_reply(
                    message_mode,
                    &incoming.from_user_name,
                    &incoming.to_user_name,
                    &input.timestamp,
                    &input.nonce,
                    &incoming.app_id,
                    self.invite_code_message(&invite.code).await,
                )?;
                return Ok(Some(encrypted));
            }

            if normalized_command == "下一场id" || normalized_command == "下一场" {
                tracing::info!(
                    open_id = %incoming.from_user_name,
                    "wechat next match id text command received"
                );

                let reply_content = self.next_match_reply(&incoming.from_user_name).await;

                let encrypted = self.build_text_reply(
                    message_mode,
                    &incoming.from_user_name,
                    &incoming.to_user_name,
                    &input.timestamp,
                    &input.nonce,
                    &incoming.app_id,
                    reply_content,
                )?;
                return Ok(Some(encrypted));
            }

            tracing::info!(
                msg_type = %incoming.msg_type,
                content = normalized_content,
                from_user = %incoming.from_user_name,
                "ignoring unsupported wechat text command"
            );
            return Ok(None);
        }

        if incoming.msg_type != "event" {
            tracing::info!(
                msg_type = %incoming.msg_type,
                from_user = %incoming.from_user_name,
                "ignoring unsupported wechat message type"
            );
            return Ok(None);
        }

        match incoming.event.as_deref() {
            Some("subscribe") => {
                tracing::info!(
                    open_id = %incoming.from_user_name,
                    "wechat subscribe event received"
                );
                let invite = self
                    .repository
                    .issue_invite_code_for_wechat_follower(&incoming.from_user_name)
                    .await?;

                tracing::info!(
                    open_id = %incoming.from_user_name,
                    invite_code = %invite.code,
                    "issued invite code for wechat follower"
                );

                Ok(Some(self.build_text_reply(
                    message_mode,
                    &incoming.from_user_name,
                    &incoming.to_user_name,
                    &input.timestamp,
                    &input.nonce,
                    &incoming.app_id,
                    self.invite_code_message(&invite.code).await,
                )?))
            }
            Some("unsubscribe") => {
                tracing::info!(
                    open_id = %incoming.from_user_name,
                    "wechat unsubscribe event received"
                );
                self.repository
                    .mark_wechat_follower_unsubscribed(&incoming.from_user_name)
                    .await?;
                tracing::info!(
                    open_id = %incoming.from_user_name,
                    "marked wechat follower unsubscribed"
                );
                Ok(None)
            }
            Some(event) => {
                tracing::info!(
                    open_id = %incoming.from_user_name,
                    event = %event,
                    "ignoring unsupported wechat event"
                );
                Ok(None)
            }
            None => {
                tracing::warn!(
                    open_id = %incoming.from_user_name,
                    "wechat event message missing event type"
                );
                Ok(None)
            }
        }
    }

    fn build_text_reply(
        &self,
        message_mode: WechatMessageMode,
        to_user_name: &str,
        from_user_name: &str,
        timestamp: &str,
        nonce: &str,
        app_id: &str,
        content: String,
    ) -> anyhow::Result<String> {
        let reply = WechatTextReply {
            to_user_name: to_user_name.to_string(),
            from_user_name: from_user_name.to_string(),
            content,
        };

        match message_mode {
            WechatMessageMode::Encrypted => self
                .crypto_port
                .encrypt_reply(&reply, timestamp, nonce, app_id),
            WechatMessageMode::Plain => self.crypto_port.build_plain_reply(&reply, timestamp),
        }
    }

    async fn official_bind_reply(&self, official_open_id: &str, code: &str) -> String {
        use crate::auth::domain::official_bind::OfficialBindDecision;
        let decision = match &self.official_bind_use_case {
            Some(use_case) => use_case.execute(official_open_id, code).await,
            None => OfficialBindDecision::Error,
        };
        tracing::info!(decision = %decision.as_str(), "wechat_bind");
        official_bind_message(decision)
    }

    async fn next_match_reply(&self, official_open_id: &str) -> String {
        let (decision, content) = if !self.next_match_reply_enabled().await {
            ("disabled", next_match_missing_message())
        } else {
            match self
                .current_standard_match_port
                .fetch_current_match_id()
                .await
            {
                Ok(Some(match_id)) => {
                    let access = self
                        .official_account_access_port
                        .check(official_open_id, match_id.parse::<i64>().ok())
                        .await;
                    match access {
                        Ok(access) => {
                            self.match_id_access_reply(official_open_id, &match_id, access)
                                .await
                        }
                        Err(_) => next_match_access_error(),
                    }
                }
                Ok(None) => ("no_match", next_match_missing_message()),
                Err(_) => {
                    tracing::warn!("failed to fetch current-standard match id");
                    ("no_match", next_match_missing_message())
                }
            }
        };
        tracing::info!(decision = %decision, "next_match_id_reply");
        content
    }

    async fn match_id_access_reply(
        &self,
        official_open_id: &str,
        match_id: &str,
        access: crate::auth::domain::official_account_match_id_access::OfficialAccountMatchIdAccess,
    ) -> (&'static str, String) {
        use crate::auth::domain::official_account_match_id_access::OfficialAccountMatchIdAccess;
        let entitled = access == OfficialAccountMatchIdAccess::Granted;
        // Invalid source IDs cannot consume a first acquisition. Preserve existing member replies.
        let can_record = entitled || match_id.parse::<i64>().is_ok_and(|id| id > 0);
        if can_record {
            match self
                .official_account_access_port
                .record_acquisition(official_open_id, !entitled)
                .await
            {
                Ok(true) if entitled => return ("granted", next_match_id_message(match_id)),
                Ok(true) => return ("first_acquisition", next_match_first_id_message(match_id)),
                Ok(false) if !entitled => {}
                // Any storage failure must withhold the ID, including a refused entitled write.
                _ => return next_match_access_error(),
            }
        }
        match access {
            OfficialAccountMatchIdAccess::Unrecognized => {
                ("unrecognized", next_match_unrecognized_message())
            }
            _ => ("not_member", next_match_not_member_message()),
        }
    }

    async fn invite_code_message(&self, invite_code: &str) -> String {
        let custom_template = self
            .system_config_port
            .get_config_value(WECHAT_INVITE_REPLY_TEMPLATE_KEY)
            .await
            .ok()
            .flatten();

        build_invite_code_message(invite_code, custom_template.as_deref())
    }

    async fn next_match_reply_enabled(&self) -> bool {
        match self
            .system_config_port
            .get_config_value(WECHAT_NEXT_MATCH_REPLY_ENABLED_KEY)
            .await
        {
            Ok(value) => parse_switch(value.as_deref()),
            Err(_) => {
                tracing::warn!(
                    "failed to read wechat next match reply switch, treating as disabled"
                );
                false
            }
        }
    }
}

fn parse_switch(value: Option<&str>) -> bool {
    value.is_some_and(|item| {
        matches!(
            item.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    })
}

fn normalize_text_command(input: &str) -> String {
    let normalized: String = input
        .chars()
        .filter(|char| !char.is_whitespace())
        .flat_map(|char| char.to_lowercase())
        .collect();

    match normalized.as_str() {
        "1" => "邀请码".to_string(),
        "2" => "下一场".to_string(),
        "3" => "下一场id".to_string(),
        _ => normalized,
    }
}

fn default_invite_code_message(invite_code: &str) -> String {
    with_product_intro(format!(
        "【足球洞察】专属邀请码\n\n邀请码：{invite_code}\n\n使用方式：\n1. 微信搜索小程序【洞察足球集散地】\n2. 输入邀请码完成注册\n3. 注册后即可开始使用\n\n常用指令：\n- 回复“邀请码”或“1”可再次获取邀请码\n- 回复“下一场”或“2”可查询当前比赛 id\n- 回复“下一场id”或“3”可查询当前比赛 id"
    ))
}

fn build_invite_code_message(invite_code: &str, template: Option<&str>) -> String {
    let Some(template) = template.map(str::trim).filter(|item| !item.is_empty()) else {
        return default_invite_code_message(invite_code);
    };

    if !template.contains("{invite_code}") {
        return default_invite_code_message(invite_code);
    }

    with_product_intro(template.replace("{invite_code}", invite_code))
}

fn official_bind_message(
    decision: crate::auth::domain::official_bind::OfficialBindDecision,
) -> String {
    use crate::auth::domain::official_bind::OfficialBindDecision::*;
    let message = match decision {
        Success => {
            r#"【足球洞察】绑定成功

已将本公众号与你的小程序账号关联。请保持关注本公众号，取关后会员权益会暂停。

现在可以回复“下一场”查询比赛 ID（V6 及以上会员可用）。"#
        }
        Invalid | Error => {
            r#"【足球洞察】绑定失败

绑定码无效或已过期。请回到小程序「我的」页，点「绑定公众号」重新获取绑定码，再回复“绑定 + 绑定码”。"#
        }
        AlreadyBound => {
            r#"【足球洞察】已绑定

你的账号已经绑定过本公众号，无需重复操作。可以直接回复“下一场”查询比赛 ID（V6 及以上会员可用）。"#
        }
        MissingCode => {
            r#"【足球洞察】绑定公众号

请先打开小程序「足球洞察」，在「我的」页点「绑定公众号」获取绑定码，然后回复“绑定 + 绑定码”，例如：绑定 ABC234"#
        }
        RateLimited => r#"【足球洞察】操作过于频繁，请稍后再试。"#,
    };
    with_product_intro(message.to_string())
}

fn next_match_id_message(match_id: &str) -> String {
    with_product_intro(format!(
        "【足球洞察】当前比赛信息\n\n当前下一场比赛 match_id：{match_id}\n\n常用指令：\n- 回复“邀请码”或“1”获取注册邀请码\n- 回复“下一场”或“2”再次查询\n- 回复“下一场id”或“3”再次查询"
    ))
}

fn next_match_first_id_message(match_id: &str) -> String {
    with_product_intro(format!(
        "【足球洞察】当前比赛信息\n\n当前下一场比赛 match_id：{match_id}\n\n这是你首次获取比赛 ID，本次免费。每位用户仅有一次免费机会，后续查询需开通 V6 及以上会员。\n\n开通方式：打开小程序「足球洞察」，在「我的」页点会员卡上的「升级」。\n\n常用指令：\n- 回复“邀请码”或“1”获取注册邀请码"
    ))
}

fn next_match_access_error() -> (&'static str, String) {
    // Provider errors may contain identifiers; never log their raw text.
    tracing::warn!("failed to confirm or record official-account match id access");
    ("error", next_match_identity_error_message())
}

fn next_match_not_member_message() -> String {
    with_product_intro(
        r#"【足球洞察】当前比赛信息

下一场比赛 ID 是会员权益，V6 及以上会员可直接查询。

开通方式：打开小程序「足球洞察」，在「我的」页点会员卡上的「升级」，选择 V6 及以上档位即可。

常用指令：
- 回复“邀请码”或“1”获取注册邀请码"#
            .to_string(),
    )
}

fn next_match_unrecognized_message() -> String {
    with_product_intro(r#"【足球洞察】当前比赛信息

下一场比赛 ID 是会员权益，V6 及以上会员可直接查询。

暂未识别到你的会员账号。如果你已经是会员，请打开小程序「足球洞察」，在「我的」页点「绑定公众号」获取绑定码，再回复“绑定 + 绑定码”（例如：绑定 ABC234），只需绑定一次，之后就可以直接在这里回复“下一场”查询。也可以直接在小程序「余票监控」页点「获取比赛 ID」查看。

还不是会员的话，在小程序「我的」页点会员卡上的「升级」即可开通。

常用指令：
- 回复“邀请码”或“1”获取注册邀请码"#.to_string())
}

fn next_match_identity_error_message() -> String {
    with_product_intro(
        r#"【足球洞察】当前比赛信息

暂时无法确认你的会员身份，请稍后再回复“下一场”重试。"#
            .to_string(),
    )
}

fn next_match_missing_message() -> String {
    with_product_intro(
        "【足球洞察】当前比赛信息\n\n暂未获取到下一场比赛 id。\n\n你可以稍后再回复“下一场”或“下一场id”重试。".to_string(),
    )
}

fn with_product_intro(message: String) -> String {
    format!("{message}\n\n产品介绍：\n{PRODUCT_INTRO_ARTICLE_URL}")
}

fn detect_message_mode(body: &str) -> WechatMessageMode {
    let trimmed = body.trim_start();

    if trimmed.contains("<Encrypt>") || trimmed.contains("<Encrypt><![CDATA[") {
        WechatMessageMode::Encrypted
    } else if trimmed.starts_with("<xml") {
        WechatMessageMode::Plain
    } else {
        WechatMessageMode::Encrypted
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;

    use crate::auth::{
        application::handle_wechat_webhook::{
            HandleWechatWebhookUseCase, WechatMessageInput, WechatVerificationInput,
        },
        domain::{
            user::{AuthUser, StoredAuthUser},
            wechat::{
                IssuedInviteCode, WechatIncomingMessage, WechatOauthProfile, WechatTextReply,
            },
        },
        ports::{
            auth_repository::AuthRepository, current_standard_match_port::CurrentStandardMatchPort,
            wechat_crypto_port::WechatCryptoPort,
        },
    };
    use crate::system_config::{
        domain::{
            ai_chat_config::AiChatSystemConfig,
            public_system_config::{
                AiChatMode, HomeBriefingMarquees, PublicSystemConfig, default_membership_tier_rules,
            },
        },
        ports::system_config_port::SystemConfigPort,
    };

    #[derive(Default)]
    struct FakeRepository {
        events: Mutex<Vec<String>>,
    }

    #[async_trait]
    impl AuthRepository for FakeRepository {
        async fn create_user_with_invite(
            &self,
            _invite_code: &str,
            _account_identifier: &str,
            _password_hash: &str,
        ) -> anyhow::Result<AuthUser> {
            unreachable!()
        }

        async fn find_user_by_account_identifier(
            &self,
            _account_identifier: &str,
        ) -> anyhow::Result<Option<StoredAuthUser>> {
            unreachable!()
        }

        async fn get_user_by_id(&self, _user_id: uuid::Uuid) -> anyhow::Result<Option<AuthUser>> {
            unreachable!()
        }

        async fn find_user_by_wechat_open_id(
            &self,
            _open_id: &str,
        ) -> anyhow::Result<Option<AuthUser>> {
            Ok(None)
        }

        async fn bind_wechat_to_user(
            &self,
            _user_id: uuid::Uuid,
            _profile: &WechatOauthProfile,
        ) -> anyhow::Result<AuthUser> {
            unreachable!()
        }

        async fn create_user_with_invite_and_wechat(
            &self,
            _invite_code: &str,
            _phone_number: &str,
            _password_hash: &str,
            _profile: &WechatOauthProfile,
        ) -> anyhow::Result<AuthUser> {
            unreachable!()
        }

        async fn create_user_with_invite_and_mini_program_wechat(
            &self,
            _invite_code: &str,
            _profile: &WechatOauthProfile,
        ) -> anyhow::Result<AuthUser> {
            unreachable!()
        }

        async fn issue_invite_code_for_wechat_follower(
            &self,
            open_id: &str,
        ) -> anyhow::Result<IssuedInviteCode> {
            self.events.lock().unwrap().push(format!("issue:{open_id}"));
            Ok(IssuedInviteCode {
                code: "FI-TEST-01".to_string(),
            })
        }

        async fn mark_wechat_follower_unsubscribed(&self, open_id: &str) -> anyhow::Result<()> {
            self.events
                .lock()
                .unwrap()
                .push(format!("unsubscribe:{open_id}"));
            Ok(())
        }

        async fn reset_password_with_invite(
            &self,
            _invite_code: &str,
            _account_identifier: &str,
            _password_hash: &str,
        ) -> anyhow::Result<()> {
            unreachable!()
        }
    }

    struct FakeCryptoPort;

    impl WechatCryptoPort for FakeCryptoPort {
        fn verify_url(
            &self,
            _signature: &str,
            _timestamp: &str,
            _nonce: &str,
            echostr: &str,
        ) -> anyhow::Result<String> {
            Ok(format!("verified::{echostr}"))
        }

        fn decrypt_message(
            &self,
            _signature: &str,
            _timestamp: &str,
            _nonce: &str,
            request_body: &str,
        ) -> anyhow::Result<WechatIncomingMessage> {
            if request_body.starts_with("<xml>") {
                return Err(anyhow::anyhow!("encrypted parser cannot read plain xml"));
            }

            let is_text_message = !matches!(request_body, "subscribe" | "unsubscribe");
            Ok(WechatIncomingMessage {
                to_user_name: "gh_123".to_string(),
                from_user_name: "openid_123".to_string(),
                create_time: 1,
                msg_type: if is_text_message {
                    "text".to_string()
                } else {
                    "event".to_string()
                },
                event: if is_text_message {
                    None
                } else {
                    Some(request_body.to_string())
                },
                content: if is_text_message {
                    Some(request_body.to_string())
                } else {
                    None
                },
                app_id: "wx_app".to_string(),
            })
        }

        fn parse_plain_message(
            &self,
            _signature: &str,
            _timestamp: &str,
            _nonce: &str,
            request_body: &str,
        ) -> anyhow::Result<WechatIncomingMessage> {
            let content = request_body
                .split("<Content><![CDATA[")
                .nth(1)
                .and_then(|s| s.split("]]></Content>").next())
                .map(str::to_owned);

            Ok(WechatIncomingMessage {
                to_user_name: "gh_123".to_string(),
                from_user_name: "openid_123".to_string(),
                create_time: 1,
                msg_type: if content.is_some() {
                    "text".to_string()
                } else {
                    "event".to_string()
                },
                event: None,
                content,
                app_id: String::new(),
            })
        }

        fn encrypt_reply(
            &self,
            reply: &WechatTextReply,
            _timestamp: &str,
            _nonce: &str,
            _app_id: &str,
        ) -> anyhow::Result<String> {
            Ok(format!("encrypted::{}", reply.content))
        }

        fn build_plain_reply(
            &self,
            reply: &WechatTextReply,
            _timestamp: &str,
        ) -> anyhow::Result<String> {
            Ok(format!("plain::{}", reply.content))
        }
    }

    struct FakeCurrentStandardMatchPort;

    #[async_trait]
    impl CurrentStandardMatchPort for FakeCurrentStandardMatchPort {
        async fn fetch_current_match_id(&self) -> anyhow::Result<Option<String>> {
            Ok(Some("MATCH_2026_001".to_string()))
        }
    }

    struct FakeMissingCurrentStandardMatchPort;

    #[async_trait]
    impl CurrentStandardMatchPort for FakeMissingCurrentStandardMatchPort {
        async fn fetch_current_match_id(&self) -> anyhow::Result<Option<String>> {
            Ok(None)
        }
    }

    #[derive(Default)]
    struct FakeSystemConfigPort {
        invite_reply_template: Option<String>,
        next_match_reply_enabled: Option<String>,
    }

    fn next_match_enabled_config() -> FakeSystemConfigPort {
        FakeSystemConfigPort {
            next_match_reply_enabled: Some("true".to_string()),
            ..FakeSystemConfigPort::default()
        }
    }

    #[async_trait]
    impl SystemConfigPort for FakeSystemConfigPort {
        async fn get_public_config(&self) -> anyhow::Result<PublicSystemConfig> {
            Ok(PublicSystemConfig::new(
                false,
                AiChatMode::BackendProxy,
                HomeBriefingMarquees::default(),
                default_membership_tier_rules(),
            ))
        }

        async fn get_ai_chat_config(&self) -> anyhow::Result<AiChatSystemConfig> {
            Ok(AiChatSystemConfig::default())
        }

        async fn get_config_value(&self, config_key: &str) -> anyhow::Result<Option<String>> {
            if config_key == "wechat_invite_reply_template" {
                return Ok(self.invite_reply_template.clone());
            }

            if config_key == "wechat_next_match_reply_enabled" {
                return Ok(self.next_match_reply_enabled.clone());
            }

            Ok(None)
        }
    }

    struct FailingSystemConfigPort;

    #[async_trait]
    impl SystemConfigPort for FailingSystemConfigPort {
        async fn get_public_config(&self) -> anyhow::Result<PublicSystemConfig> {
            Ok(PublicSystemConfig::new(
                false,
                AiChatMode::BackendProxy,
                HomeBriefingMarquees::default(),
                default_membership_tier_rules(),
            ))
        }

        async fn get_ai_chat_config(&self) -> anyhow::Result<AiChatSystemConfig> {
            Ok(AiChatSystemConfig::default())
        }

        async fn get_config_value(&self, _config_key: &str) -> anyhow::Result<Option<String>> {
            Err(anyhow::anyhow!("config store unavailable"))
        }
    }

    struct RecordingCurrentStandardMatchPort {
        calls: std::sync::atomic::AtomicUsize,
    }

    impl RecordingCurrentStandardMatchPort {
        fn new() -> Self {
            Self {
                calls: std::sync::atomic::AtomicUsize::new(0),
            }
        }

        fn calls(&self) -> usize {
            self.calls.load(std::sync::atomic::Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl CurrentStandardMatchPort for RecordingCurrentStandardMatchPort {
        async fn fetch_current_match_id(&self) -> anyhow::Result<Option<String>> {
            self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(Some("MATCH_2026_001".to_string()))
        }
    }

    #[tokio::test]
    async fn verify_endpoint_delegates_to_crypto_port() {
        let use_case = HandleWechatWebhookUseCase::new(
            Arc::new(FakeRepository::default()),
            Arc::new(FakeCryptoPort),
            Arc::new(FakeCurrentStandardMatchPort),
            Arc::new(FakeSystemConfigPort::default()),
            Arc::new(GateAccessFake::granted()),
        );

        let result = use_case
            .verify_endpoint(WechatVerificationInput {
                signature: "sig".to_string(),
                timestamp: "1".to_string(),
                nonce: "2".to_string(),
                echostr: "echo".to_string(),
            })
            .unwrap();

        assert_eq!(result, "verified::echo");
    }

    #[tokio::test]
    async fn handle_subscribe_event_issues_invite_code_and_builds_reply() {
        let repository = Arc::new(FakeRepository::default());
        let use_case = HandleWechatWebhookUseCase::new(
            repository.clone(),
            Arc::new(FakeCryptoPort),
            Arc::new(FakeCurrentStandardMatchPort),
            Arc::new(FakeSystemConfigPort::default()),
            Arc::new(GateAccessFake::granted()),
        );

        let result = use_case
            .handle_message(WechatMessageInput {
                signature: "sig".to_string(),
                timestamp: "1".to_string(),
                nonce: "2".to_string(),
                body: "subscribe".to_string(),
            })
            .await
            .unwrap()
            .unwrap();

        assert!(result.contains("【足球洞察】专属邀请码"));
        assert!(result.contains("邀请码：FI-TEST-01"));
        assert!(result.contains("FI-TEST-01"));
        assert!(result.contains("微信搜索小程序【洞察足球集散地】"));
        assert!(result.contains("输入邀请码完成注册"));
        assert!(result.contains("常用指令"));
        assert!(result.contains("回复“邀请码”或“1”"));
        assert!(result.contains("回复“下一场”或“2”"));
        assert!(result.contains("回复“下一场id”或“3”"));
        assert!(result.contains("产品介绍"));
        assert!(result.contains("mp.weixin.qq.com/s?__biz=MzA3Nzc1NTk1Mg==&mid=2247483892&idx=1&sn=c41ff0a1987269db51417a11d0c7b885"));
        assert_eq!(
            repository.events.lock().unwrap().as_slice(),
            ["issue:openid_123"]
        );
    }

    #[tokio::test]
    async fn handle_invite_code_text_message_issues_invite_code_and_builds_reply() {
        let repository = Arc::new(FakeRepository::default());
        let use_case = HandleWechatWebhookUseCase::new(
            repository.clone(),
            Arc::new(FakeCryptoPort),
            Arc::new(FakeCurrentStandardMatchPort),
            Arc::new(FakeSystemConfigPort::default()),
            Arc::new(GateAccessFake::granted()),
        );

        let result = use_case
            .handle_message(WechatMessageInput {
                signature: "sig".to_string(),
                timestamp: "1".to_string(),
                nonce: "2".to_string(),
                body: "邀请码".to_string(),
            })
            .await
            .unwrap()
            .unwrap();

        assert!(result.contains("【足球洞察】专属邀请码"));
        assert!(result.contains("邀请码：FI-TEST-01"));
        assert!(result.contains("FI-TEST-01"));
        assert!(result.contains("微信搜索小程序【洞察足球集散地】"));
        assert!(result.contains("常用指令"));
        assert!(result.contains("回复“邀请码”或“1”"));
        assert!(result.contains("产品介绍"));
        assert!(result.contains("mp.weixin.qq.com/s?__biz=MzA3Nzc1NTk1Mg==&mid=2247483892&idx=1&sn=c41ff0a1987269db51417a11d0c7b885"));
        assert_eq!(
            repository.events.lock().unwrap().as_slice(),
            ["issue:openid_123"]
        );
    }

    #[tokio::test]
    async fn handle_next_match_id_text_message_returns_match_id_reply() {
        let use_case = HandleWechatWebhookUseCase::new(
            Arc::new(FakeRepository::default()),
            Arc::new(FakeCryptoPort),
            Arc::new(FakeCurrentStandardMatchPort),
            Arc::new(next_match_enabled_config()),
            Arc::new(GateAccessFake::granted()),
        );

        let result = use_case
            .handle_message(WechatMessageInput {
                signature: "sig".to_string(),
                timestamp: "1".to_string(),
                nonce: "2".to_string(),
                body: "下一场id".to_string(),
            })
            .await
            .unwrap()
            .unwrap();

        assert!(result.contains("当前下一场比赛 match_id：MATCH_2026_001"));
        assert!(result.contains("回复“下一场”或“2”"));
        assert!(result.contains("回复“下一场id”或“3”"));
        assert!(result.contains("产品介绍"));
        assert!(result.contains("mp.weixin.qq.com/s?__biz=MzA3Nzc1NTk1Mg==&mid=2247483892&idx=1&sn=c41ff0a1987269db51417a11d0c7b885"));
    }

    #[tokio::test]
    async fn handle_next_match_id_text_message_returns_fallback_when_match_missing() {
        let use_case = HandleWechatWebhookUseCase::new(
            Arc::new(FakeRepository::default()),
            Arc::new(FakeCryptoPort),
            Arc::new(FakeMissingCurrentStandardMatchPort),
            Arc::new(next_match_enabled_config()),
            Arc::new(GateAccessFake::granted()),
        );

        let result = use_case
            .handle_message(WechatMessageInput {
                signature: "sig".to_string(),
                timestamp: "1".to_string(),
                nonce: "2".to_string(),
                body: "下一场id".to_string(),
            })
            .await
            .unwrap()
            .unwrap();

        assert!(result.contains("暂未获取到下一场比赛 id"));
    }

    #[tokio::test]
    async fn handle_next_match_id_text_message_returns_missing_reply_when_switch_disabled() {
        let current_match_port = Arc::new(RecordingCurrentStandardMatchPort::new());
        let use_case = HandleWechatWebhookUseCase::new(
            Arc::new(FakeRepository::default()),
            Arc::new(FakeCryptoPort),
            current_match_port.clone(),
            Arc::new(FakeSystemConfigPort::default()),
            Arc::new(GateAccessFake::granted()),
        );

        let result = use_case
            .handle_message(WechatMessageInput {
                signature: "sig".to_string(),
                timestamp: "1".to_string(),
                nonce: "2".to_string(),
                body: "下一场id".to_string(),
            })
            .await
            .unwrap()
            .unwrap();

        assert!(result.contains("暂未获取到下一场比赛 id"));
        assert_eq!(current_match_port.calls(), 0);
    }

    #[tokio::test]
    async fn handle_next_match_id_text_message_returns_missing_reply_when_switch_explicitly_false()
    {
        let current_match_port = Arc::new(RecordingCurrentStandardMatchPort::new());
        let use_case = HandleWechatWebhookUseCase::new(
            Arc::new(FakeRepository::default()),
            Arc::new(FakeCryptoPort),
            current_match_port.clone(),
            Arc::new(FakeSystemConfigPort {
                next_match_reply_enabled: Some("false".to_string()),
                ..FakeSystemConfigPort::default()
            }),
            Arc::new(GateAccessFake::granted()),
        );

        let result = use_case
            .handle_message(WechatMessageInput {
                signature: "sig".to_string(),
                timestamp: "1".to_string(),
                nonce: "2".to_string(),
                body: "下一场id".to_string(),
            })
            .await
            .unwrap()
            .unwrap();

        assert!(result.contains("暂未获取到下一场比赛 id"));
        assert_eq!(current_match_port.calls(), 0);
    }

    #[tokio::test]
    async fn handle_next_match_id_text_message_returns_match_id_when_switch_value_is_one() {
        let use_case = HandleWechatWebhookUseCase::new(
            Arc::new(FakeRepository::default()),
            Arc::new(FakeCryptoPort),
            Arc::new(FakeCurrentStandardMatchPort),
            Arc::new(FakeSystemConfigPort {
                next_match_reply_enabled: Some("1".to_string()),
                ..FakeSystemConfigPort::default()
            }),
            Arc::new(GateAccessFake::granted()),
        );

        let result = use_case
            .handle_message(WechatMessageInput {
                signature: "sig".to_string(),
                timestamp: "1".to_string(),
                nonce: "2".to_string(),
                body: "下一场id".to_string(),
            })
            .await
            .unwrap()
            .unwrap();

        assert!(result.contains("MATCH_2026_001"));
    }

    #[tokio::test]
    async fn handle_next_match_id_text_message_treats_config_error_as_disabled() {
        let current_match_port = Arc::new(RecordingCurrentStandardMatchPort::new());
        let use_case = HandleWechatWebhookUseCase::new(
            Arc::new(FakeRepository::default()),
            Arc::new(FakeCryptoPort),
            current_match_port.clone(),
            Arc::new(FailingSystemConfigPort),
            Arc::new(GateAccessFake::granted()),
        );

        let result = use_case
            .handle_message(WechatMessageInput {
                signature: "sig".to_string(),
                timestamp: "1".to_string(),
                nonce: "2".to_string(),
                body: "下一场id".to_string(),
            })
            .await
            .unwrap()
            .unwrap();

        assert!(result.contains("暂未获取到下一场比赛 id"));
        assert_eq!(current_match_port.calls(), 0);
    }

    #[tokio::test]
    async fn handle_next_match_id_text_message_accepts_whitespace_and_uppercase() {
        let use_case = HandleWechatWebhookUseCase::new(
            Arc::new(FakeRepository::default()),
            Arc::new(FakeCryptoPort),
            Arc::new(FakeCurrentStandardMatchPort),
            Arc::new(next_match_enabled_config()),
            Arc::new(GateAccessFake::granted()),
        );

        let result = use_case
            .handle_message(WechatMessageInput {
                signature: "sig".to_string(),
                timestamp: "1".to_string(),
                nonce: "2".to_string(),
                body: "下一场 ID".to_string(),
            })
            .await
            .unwrap()
            .unwrap();

        assert!(result.contains("MATCH_2026_001"));
    }

    #[tokio::test]
    async fn handle_next_match_text_message_returns_match_id_reply() {
        let use_case = HandleWechatWebhookUseCase::new(
            Arc::new(FakeRepository::default()),
            Arc::new(FakeCryptoPort),
            Arc::new(FakeCurrentStandardMatchPort),
            Arc::new(next_match_enabled_config()),
            Arc::new(GateAccessFake::granted()),
        );

        let result = use_case
            .handle_message(WechatMessageInput {
                signature: "sig".to_string(),
                timestamp: "1".to_string(),
                nonce: "2".to_string(),
                body: "下一场".to_string(),
            })
            .await
            .unwrap()
            .unwrap();

        assert!(result.contains("MATCH_2026_001"));
    }

    #[tokio::test]
    async fn handle_numeric_one_text_message_returns_invite_code_reply() {
        let repository = Arc::new(FakeRepository::default());
        let use_case = HandleWechatWebhookUseCase::new(
            repository.clone(),
            Arc::new(FakeCryptoPort),
            Arc::new(FakeCurrentStandardMatchPort),
            Arc::new(FakeSystemConfigPort::default()),
            Arc::new(GateAccessFake::granted()),
        );

        let result = use_case
            .handle_message(WechatMessageInput {
                signature: "sig".to_string(),
                timestamp: "1".to_string(),
                nonce: "2".to_string(),
                body: "1".to_string(),
            })
            .await
            .unwrap()
            .unwrap();

        assert!(result.contains("邀请码：FI-TEST-01"));
        assert_eq!(
            repository.events.lock().unwrap().as_slice(),
            ["issue:openid_123"]
        );
    }

    #[tokio::test]
    async fn handle_numeric_two_text_message_returns_match_id_reply() {
        let use_case = HandleWechatWebhookUseCase::new(
            Arc::new(FakeRepository::default()),
            Arc::new(FakeCryptoPort),
            Arc::new(FakeCurrentStandardMatchPort),
            Arc::new(next_match_enabled_config()),
            Arc::new(GateAccessFake::granted()),
        );

        let result = use_case
            .handle_message(WechatMessageInput {
                signature: "sig".to_string(),
                timestamp: "1".to_string(),
                nonce: "2".to_string(),
                body: "2".to_string(),
            })
            .await
            .unwrap()
            .unwrap();

        assert!(result.contains("MATCH_2026_001"));
    }

    #[tokio::test]
    async fn handle_numeric_three_text_message_returns_match_id_reply() {
        let use_case = HandleWechatWebhookUseCase::new(
            Arc::new(FakeRepository::default()),
            Arc::new(FakeCryptoPort),
            Arc::new(FakeCurrentStandardMatchPort),
            Arc::new(next_match_enabled_config()),
            Arc::new(GateAccessFake::granted()),
        );

        let result = use_case
            .handle_message(WechatMessageInput {
                signature: "sig".to_string(),
                timestamp: "1".to_string(),
                nonce: "2".to_string(),
                body: "3".to_string(),
            })
            .await
            .unwrap()
            .unwrap();

        assert!(result.contains("MATCH_2026_001"));
    }

    #[tokio::test]
    async fn handle_plaintext_invite_code_message_issues_invite_code_and_builds_reply() {
        let repository = Arc::new(FakeRepository::default());
        let use_case = HandleWechatWebhookUseCase::new(
            repository.clone(),
            Arc::new(FakeCryptoPort),
            Arc::new(FakeCurrentStandardMatchPort),
            Arc::new(FakeSystemConfigPort::default()),
            Arc::new(GateAccessFake::granted()),
        );

        let result = use_case
            .handle_message(WechatMessageInput {
                signature: "sig".to_string(),
                timestamp: "1".to_string(),
                nonce: "2".to_string(),
                body: "<xml><ToUserName><![CDATA[gh_123]]></ToUserName><FromUserName><![CDATA[openid_123]]></FromUserName><CreateTime>1</CreateTime><MsgType><![CDATA[text]]></MsgType><Content><![CDATA[邀请码]]></Content></xml>".to_string(),
            })
            .await
            .unwrap()
            .unwrap();

        assert!(result.contains("【足球洞察】专属邀请码"));
        assert!(result.contains("邀请码：FI-TEST-01"));
        assert!(result.contains("FI-TEST-01"));
        assert!(result.contains("输入邀请码完成注册"));
        assert!(result.contains("回复“邀请码”或“1”"));
        assert_eq!(
            repository.events.lock().unwrap().as_slice(),
            ["issue:openid_123"]
        );
    }

    #[tokio::test]
    async fn handle_plaintext_next_match_message_returns_match_id_reply() {
        let use_case = HandleWechatWebhookUseCase::new(
            Arc::new(FakeRepository::default()),
            Arc::new(FakeCryptoPort),
            Arc::new(FakeCurrentStandardMatchPort),
            Arc::new(next_match_enabled_config()),
            Arc::new(GateAccessFake::granted()),
        );

        let result = use_case
            .handle_message(WechatMessageInput {
                signature: "sig".to_string(),
                timestamp: "1".to_string(),
                nonce: "2".to_string(),
                body: "<xml><ToUserName><![CDATA[gh_123]]></ToUserName><FromUserName><![CDATA[openid_123]]></FromUserName><CreateTime>1</CreateTime><MsgType><![CDATA[text]]></MsgType><Content><![CDATA[下一场]]></Content></xml>".to_string(),
            })
            .await
            .unwrap()
            .unwrap();

        assert!(result.contains("MATCH_2026_001"));
    }

    #[tokio::test]
    async fn handle_unknown_text_message_returns_none() {
        let use_case = HandleWechatWebhookUseCase::new(
            Arc::new(FakeRepository::default()),
            Arc::new(FakeCryptoPort),
            Arc::new(FakeCurrentStandardMatchPort),
            Arc::new(FakeSystemConfigPort::default()),
            Arc::new(GateAccessFake::granted()),
        );

        let result = use_case
            .handle_message(WechatMessageInput {
                signature: "sig".to_string(),
                timestamp: "1".to_string(),
                nonce: "2".to_string(),
                body: "你好".to_string(),
            })
            .await
            .unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn handle_unknown_plaintext_message_returns_none() {
        let use_case = HandleWechatWebhookUseCase::new(
            Arc::new(FakeRepository::default()),
            Arc::new(FakeCryptoPort),
            Arc::new(FakeCurrentStandardMatchPort),
            Arc::new(FakeSystemConfigPort::default()),
            Arc::new(GateAccessFake::granted()),
        );

        let result = use_case
            .handle_message(WechatMessageInput {
                signature: "sig".to_string(),
                timestamp: "1".to_string(),
                nonce: "2".to_string(),
                body: "<xml><ToUserName><![CDATA[gh_123]]></ToUserName><FromUserName><![CDATA[openid_123]]></FromUserName><CreateTime>1</CreateTime><MsgType><![CDATA[text]]></MsgType><Content><![CDATA[你好]]></Content></xml>".to_string(),
            })
            .await
            .unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn handle_subscribe_event_prefers_reply_template_from_system_config() {
        let use_case = HandleWechatWebhookUseCase::new(
            Arc::new(FakeRepository::default()),
            Arc::new(FakeCryptoPort),
            Arc::new(FakeCurrentStandardMatchPort),
            Arc::new(FakeSystemConfigPort {
                invite_reply_template: Some(
                    "自定义入口\n邀请码：{invite_code}\n微信搜索小程序【洞察足球集散地】"
                        .to_string(),
                ),
                ..FakeSystemConfigPort::default()
            }),
            Arc::new(GateAccessFake::granted()),
        );

        let result = use_case
            .handle_message(WechatMessageInput {
                signature: "sig".to_string(),
                timestamp: "1".to_string(),
                nonce: "2".to_string(),
                body: "subscribe".to_string(),
            })
            .await
            .unwrap()
            .unwrap();

        assert!(result.contains("自定义入口"));
        assert!(result.contains("邀请码：FI-TEST-01"));
        assert!(result.contains("微信搜索小程序【洞察足球集散地】"));
    }

    #[tokio::test]
    async fn handle_subscribe_event_falls_back_when_reply_template_misses_invite_placeholder() {
        let use_case = HandleWechatWebhookUseCase::new(
            Arc::new(FakeRepository::default()),
            Arc::new(FakeCryptoPort),
            Arc::new(FakeCurrentStandardMatchPort),
            Arc::new(FakeSystemConfigPort {
                invite_reply_template: Some("只有入口，没有邀请码".to_string()),
                ..FakeSystemConfigPort::default()
            }),
            Arc::new(GateAccessFake::granted()),
        );

        let result = use_case
            .handle_message(WechatMessageInput {
                signature: "sig".to_string(),
                timestamp: "1".to_string(),
                nonce: "2".to_string(),
                body: "subscribe".to_string(),
            })
            .await
            .unwrap()
            .unwrap();

        assert!(result.contains("【足球洞察】专属邀请码"));
        assert!(result.contains("FI-TEST-01"));
        assert!(!result.contains("只有入口，没有邀请码"));
    }
    use crate::auth::{
        domain::official_account_match_id_access::OfficialAccountMatchIdAccess,
        ports::official_account_match_id_access_port::OfficialAccountMatchIdAccessPort,
    };

    struct GateAccessFake {
        decision: OfficialAccountMatchIdAccess,
        requires_match: bool,
        fail: bool,
        calls: Mutex<Vec<Option<i64>>>,
        first_acquisition: Mutex<bool>,
        record_fail: bool,
    }

    impl GateAccessFake {
        fn granted() -> Self {
            Self::result(OfficialAccountMatchIdAccess::Granted, false)
        }
        fn result(decision: OfficialAccountMatchIdAccess, requires_match: bool) -> Self {
            Self {
                decision,
                requires_match,
                fail: false,
                calls: Mutex::new(Vec::new()),
                first_acquisition: Mutex::new(false),
                record_fail: false,
            }
        }
    }

    #[async_trait]
    impl OfficialAccountMatchIdAccessPort for GateAccessFake {
        async fn check(
            &self,
            official_open_id: &str,
            match_id: Option<i64>,
        ) -> anyhow::Result<OfficialAccountMatchIdAccess> {
            assert_eq!(official_open_id, "openid_123");
            self.calls.lock().unwrap().push(match_id);
            if self.fail {
                anyhow::bail!("provider failure: 987654321 openid_123");
            }
            Ok(if self.requires_match && match_id != Some(987654321) {
                OfficialAccountMatchIdAccess::NotMember
            } else {
                self.decision
            })
        }

        async fn record_acquisition(
            &self,
            official_open_id: &str,
            require_first: bool,
        ) -> anyhow::Result<bool> {
            assert_eq!(official_open_id, "openid_123");
            if self.record_fail {
                anyhow::bail!("storage failure: 987654321 openid_123");
            }
            let mut first = self.first_acquisition.lock().unwrap();
            let granted = !require_first || *first;
            if granted {
                *first = false;
            }
            Ok(granted)
        }
    }

    struct GateMatchFake {
        id: Option<&'static str>,
        fail: bool,
    }
    #[async_trait]
    impl CurrentStandardMatchPort for GateMatchFake {
        async fn fetch_current_match_id(&self) -> anyhow::Result<Option<String>> {
            if self.fail {
                anyhow::bail!("source failure: 987654321 openid_123");
            }
            Ok(self.id.map(str::to_owned))
        }
    }

    #[derive(Clone)]
    struct GateLogWriter(Arc<Mutex<Vec<u8>>>);
    impl std::io::Write for GateLogWriter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn next_match_gate_covers_all_commands_modes_decisions_and_never_leaks_id() {
        let commands = ["下一场", "下一场id", "2", "3", " 下 一 场 ID ", "\t3\n"];
        for command in commands {
            for plain in [false, true] {
                for (outcome, requires_match, fail, expected, decision) in [
                    (
                        OfficialAccountMatchIdAccess::NotMember,
                        false,
                        false,
                        false,
                        "not_member",
                    ),
                    (
                        OfficialAccountMatchIdAccess::Granted,
                        false,
                        false,
                        true,
                        "granted",
                    ),
                    (
                        OfficialAccountMatchIdAccess::Granted,
                        true,
                        false,
                        true,
                        "granted",
                    ),
                    (
                        OfficialAccountMatchIdAccess::Unrecognized,
                        false,
                        false,
                        false,
                        "unrecognized",
                    ),
                    (
                        OfficialAccountMatchIdAccess::Granted,
                        false,
                        true,
                        false,
                        "error",
                    ),
                ] {
                    let mut access = GateAccessFake::result(outcome, requires_match);
                    access.fail = fail;
                    let access = Arc::new(access);
                    let use_case = HandleWechatWebhookUseCase::new(
                        Arc::new(FakeRepository::default()),
                        Arc::new(FakeCryptoPort),
                        Arc::new(GateMatchFake {
                            id: Some("987654321"),
                            fail: false,
                        }),
                        Arc::new(next_match_enabled_config()),
                        access.clone(),
                    );
                    let body = if plain {
                        format!("<xml><Content><![CDATA[{command}]]></Content></xml>")
                    } else {
                        command.to_owned()
                    };
                    let result = use_case
                        .handle_message(WechatMessageInput {
                            signature: "sig".into(),
                            timestamp: "1".into(),
                            nonce: "2".into(),
                            body,
                        })
                        .await
                        .unwrap()
                        .unwrap();
                    assert_eq!(
                        result.contains("987654321"),
                        expected,
                        "{command} {decision}"
                    );
                    assert!(result.starts_with(if plain { "plain::" } else { "encrypted::" }));
                    assert!(result.contains("产品介绍"));
                    assert!(!result.contains("¥5"));
                    if !expected {
                        let expected_copy = match decision {
                            "not_member" => {
                                "【足球洞察】当前比赛信息\n\n下一场比赛 ID 是会员权益，V6 及以上会员可直接查询。\n\n开通方式：打开小程序「足球洞察」，在「我的」页点会员卡上的「升级」，选择 V6 及以上档位即可。\n\n常用指令：\n- 回复“邀请码”或“1”获取注册邀请码"
                            }
                            "unrecognized" => {
                                "【足球洞察】当前比赛信息\n\n下一场比赛 ID 是会员权益，V6 及以上会员可直接查询。\n\n暂未识别到你的会员账号。如果你已经是会员，请打开小程序「足球洞察」，在「我的」页点「绑定公众号」获取绑定码，再回复“绑定 + 绑定码”（例如：绑定 ABC234），只需绑定一次，之后就可以直接在这里回复“下一场”查询。也可以直接在小程序「余票监控」页点「获取比赛 ID」查看。\n\n还不是会员的话，在小程序「我的」页点会员卡上的「升级」即可开通。\n\n常用指令：\n- 回复“邀请码”或“1”获取注册邀请码"
                            }
                            _ => {
                                "【足球洞察】当前比赛信息\n\n暂时无法确认你的会员身份，请稍后再回复“下一场”重试。"
                            }
                        };
                        let prefix = if plain { "plain::" } else { "encrypted::" };
                        assert_eq!(
                            result,
                            format!(
                                "{prefix}{}",
                                super::with_product_intro(expected_copy.to_owned())
                            )
                        );
                    }
                    assert_eq!(*access.calls.lock().unwrap(), [Some(987654321)]);
                }
            }
        }
    }

    #[tokio::test]
    async fn first_time_follower_can_receive_one_match_id_without_membership() {
        let access = Arc::new(GateAccessFake::result(
            OfficialAccountMatchIdAccess::Unrecognized,
            false,
        ));
        *access.first_acquisition.lock().unwrap() = true;
        let use_case = HandleWechatWebhookUseCase::new(
            Arc::new(FakeRepository::default()),
            Arc::new(FakeCryptoPort),
            Arc::new(GateMatchFake {
                id: Some("987654321"),
                fail: false,
            }),
            Arc::new(next_match_enabled_config()),
            access.clone(),
        );
        let reply = use_case.next_match_reply("openid_123").await;
        assert!(
            reply.contains("987654321"),
            "a first-time follower should receive the ID"
        );
        assert!(reply.contains("免费"));
        assert!(reply.contains("V6"));
        for command in ["下一场", "下一场id", "2", "3"] {
            let reply = use_case
                .handle_message(WechatMessageInput {
                    signature: "sig".into(),
                    timestamp: "1".into(),
                    nonce: "2".into(),
                    body: command.into(),
                })
                .await
                .unwrap()
                .unwrap();
            assert!(
                !reply.contains("987654321"),
                "a second query cannot receive the ID: {command}"
            );
        }
        let next_match = HandleWechatWebhookUseCase::new(
            Arc::new(FakeRepository::default()),
            Arc::new(FakeCryptoPort),
            Arc::new(GateMatchFake {
                id: Some("987654322"),
                fail: false,
            }),
            Arc::new(next_match_enabled_config()),
            access,
        );
        assert!(
            !next_match
                .next_match_reply("openid_123")
                .await
                .contains("987654322")
        );
    }

    #[tokio::test]
    async fn first_id_is_available_for_bound_nonmembers_and_both_message_modes() {
        for outcome in [
            OfficialAccountMatchIdAccess::Unrecognized,
            OfficialAccountMatchIdAccess::NotMember,
        ] {
            for command in ["下一场", "下一场id", "2", "3", " 下 一 场 ID "] {
                for plain in [true, false] {
                    let access = Arc::new(GateAccessFake::result(outcome, false));
                    *access.first_acquisition.lock().unwrap() = true;
                    let use_case = HandleWechatWebhookUseCase::new(
                        Arc::new(FakeRepository::default()),
                        Arc::new(FakeCryptoPort),
                        Arc::new(GateMatchFake {
                            id: Some("987654321"),
                            fail: false,
                        }),
                        Arc::new(next_match_enabled_config()),
                        access,
                    );
                    let body = if plain {
                        format!("<xml><Content><![CDATA[{command}]]></Content></xml>")
                    } else {
                        command.into()
                    };
                    let reply = use_case
                        .handle_message(WechatMessageInput {
                            signature: "sig".into(),
                            timestamp: "1".into(),
                            nonce: "2".into(),
                            body,
                        })
                        .await
                        .unwrap()
                        .unwrap();
                    assert!(
                        reply.contains("987654321")
                            && reply.contains("首次")
                            && reply.contains("免费")
                    );
                    assert!(reply.starts_with(if plain { "plain::" } else { "encrypted::" }));
                    assert!(reply.contains("产品介绍"));
                }
            }
        }
    }

    #[tokio::test]
    async fn acquisition_errors_withhold_id_and_invalid_source_does_not_consume_first_id() {
        for outcome in [
            OfficialAccountMatchIdAccess::Granted,
            OfficialAccountMatchIdAccess::NotMember,
            OfficialAccountMatchIdAccess::Unrecognized,
        ] {
            let mut access = GateAccessFake::result(outcome, false);
            access.record_fail = true;
            *access.first_acquisition.lock().unwrap() = true;
            let use_case = HandleWechatWebhookUseCase::new(
                Arc::new(FakeRepository::default()),
                Arc::new(FakeCryptoPort),
                Arc::new(GateMatchFake {
                    id: Some("987654321"),
                    fail: false,
                }),
                Arc::new(next_match_enabled_config()),
                Arc::new(access),
            );
            let reply = use_case.next_match_reply("openid_123").await;
            assert!(!reply.contains("987654321") && !reply.contains("openid_123"));
            assert!(reply.contains("稍后"));
        }
        let access = Arc::new(GateAccessFake::result(
            OfficialAccountMatchIdAccess::NotMember,
            false,
        ));
        *access.first_acquisition.lock().unwrap() = true;
        for id in ["not-numeric-match", "", "0", "-1"] {
            let use_case = HandleWechatWebhookUseCase::new(
                Arc::new(FakeRepository::default()),
                Arc::new(FakeCryptoPort),
                Arc::new(GateMatchFake {
                    id: Some(id),
                    fail: false,
                }),
                Arc::new(next_match_enabled_config()),
                access.clone(),
            );
            assert!(
                !use_case
                    .next_match_reply("openid_123")
                    .await
                    .contains("本次免费")
            );
            assert!(*access.first_acquisition.lock().unwrap());
        }
    }

    #[tokio::test]
    async fn next_match_gate_skips_identity_for_disabled_missing_and_fetch_failure() {
        for scenario in ["disabled", "config_error", "missing", "fetch_error"] {
            let access = Arc::new(GateAccessFake::granted());
            let match_port = Arc::new(RecordingCurrentStandardMatchPort::new());
            let current: Arc<dyn CurrentStandardMatchPort> = match scenario {
                "missing" => Arc::new(GateMatchFake {
                    id: None,
                    fail: false,
                }),
                "fetch_error" => Arc::new(GateMatchFake {
                    id: None,
                    fail: true,
                }),
                _ => match_port.clone(),
            };
            let config: Arc<dyn SystemConfigPort> = match scenario {
                "disabled" => Arc::new(FakeSystemConfigPort::default()),
                "config_error" => Arc::new(FailingSystemConfigPort),
                _ => Arc::new(next_match_enabled_config()),
            };
            let use_case = HandleWechatWebhookUseCase::new(
                Arc::new(FakeRepository::default()),
                Arc::new(FakeCryptoPort),
                current,
                config,
                access.clone(),
            );
            let result = use_case
                .handle_message(WechatMessageInput {
                    signature: "sig".into(),
                    timestamp: "1".into(),
                    nonce: "2".into(),
                    body: "2".into(),
                })
                .await
                .unwrap()
                .unwrap();
            assert_eq!(
                result,
                format!("encrypted::{}", super::next_match_missing_message())
            );
            assert!(!result.contains("987654321") && !result.contains("MATCH_2026_001"));
            assert!(!result.contains("会员"));
            assert!(access.calls.lock().unwrap().is_empty(), "{scenario}");
            assert_eq!(match_port.calls(), 0, "{scenario}");
        }
    }

    #[tokio::test]
    async fn missing_match_and_failed_lookup_leave_the_first_acquisition_available() {
        let access = Arc::new(GateAccessFake::result(
            OfficialAccountMatchIdAccess::Unrecognized,
            false,
        ));
        *access.first_acquisition.lock().unwrap() = true;
        for (id, fail) in [(None, false), (None, true)] {
            let use_case = HandleWechatWebhookUseCase::new(
                Arc::new(FakeRepository::default()),
                Arc::new(FakeCryptoPort),
                Arc::new(GateMatchFake { id, fail }),
                Arc::new(next_match_enabled_config()),
                access.clone(),
            );
            let reply = use_case.next_match_reply("openid_123").await;
            assert!(reply.contains("暂未获取到"));
        }
        let use_case = HandleWechatWebhookUseCase::new(
            Arc::new(FakeRepository::default()),
            Arc::new(FakeCryptoPort),
            Arc::new(GateMatchFake {
                id: Some("987654321"),
                fail: false,
            }),
            Arc::new(next_match_enabled_config()),
            access,
        );
        let reply = use_case.next_match_reply("openid_123").await;
        assert!(reply.contains("987654321") && reply.contains("本次免费"));
    }

    #[tokio::test]
    async fn next_match_gate_unparseable_id_only_allows_membership_not_purchase() {
        for (outcome, requires_match, allowed) in [
            (OfficialAccountMatchIdAccess::Granted, false, true),
            (OfficialAccountMatchIdAccess::Granted, true, false),
            (OfficialAccountMatchIdAccess::NotMember, false, false),
        ] {
            let access = Arc::new(GateAccessFake::result(outcome, requires_match));
            let use_case = HandleWechatWebhookUseCase::new(
                Arc::new(FakeRepository::default()),
                Arc::new(FakeCryptoPort),
                Arc::new(GateMatchFake {
                    id: Some("not-numeric-match"),
                    fail: false,
                }),
                Arc::new(next_match_enabled_config()),
                access.clone(),
            );
            let result = use_case
                .handle_message(WechatMessageInput {
                    signature: "sig".into(),
                    timestamp: "1".into(),
                    nonce: "2".into(),
                    body: "3".into(),
                })
                .await
                .unwrap()
                .unwrap();
            assert_eq!(result.contains("not-numeric-match"), allowed);
            assert_eq!(*access.calls.lock().unwrap(), [None]);
        }
    }
    // tracing's process-wide callsite cache changes while other webhook tests run without
    // a subscriber. Exercise log capture explicitly in isolation, alongside the full suite.
    #[tokio::test(flavor = "current_thread")]
    #[ignore = "run log capture separately with --ignored --test-threads=1"]
    async fn next_match_decision_logs_are_single_and_identifier_free() {
        let bytes = Arc::new(Mutex::new(Vec::new()));
        let sink = bytes.clone();
        let _guard = tracing::subscriber::set_default(
            tracing_subscriber::fmt()
                .without_time()
                .with_ansi(false)
                .with_writer(move || GateLogWriter(sink.clone()))
                .finish(),
        );
        for scenario in [
            "granted",
            "first_acquisition",
            "not_member",
            "unrecognized",
            "error",
            "record_error",
            "disabled",
            "no_match",
            "fetch_error",
            "config_error",
        ] {
            bytes.lock().unwrap().clear();
            let outcome = match scenario {
                "not_member" => OfficialAccountMatchIdAccess::NotMember,
                "unrecognized" | "first_acquisition" => OfficialAccountMatchIdAccess::Unrecognized,
                _ => OfficialAccountMatchIdAccess::Granted,
            };
            let mut access = GateAccessFake::result(outcome, false);
            access.fail = scenario == "error";
            access.record_fail = scenario == "record_error";
            *access.first_acquisition.lock().unwrap() = scenario == "first_acquisition";
            let current = Arc::new(GateMatchFake {
                id: if scenario == "no_match" {
                    None
                } else {
                    Some("987654321")
                },
                fail: scenario == "fetch_error",
            });
            let config: Arc<dyn SystemConfigPort> = match scenario {
                "disabled" => Arc::new(FakeSystemConfigPort::default()),
                "config_error" => Arc::new(FailingSystemConfigPort),
                _ => Arc::new(next_match_enabled_config()),
            };
            let use_case = HandleWechatWebhookUseCase::new(
                Arc::new(FakeRepository::default()),
                Arc::new(FakeCryptoPort),
                current,
                config,
                Arc::new(access),
            );
            let reply = use_case.next_match_reply("openid_123").await;
            assert_eq!(
                reply.contains("987654321"),
                matches!(scenario, "granted" | "first_acquisition"),
                "{scenario}"
            );
            let logs = String::from_utf8(bytes.lock().unwrap().clone()).unwrap();
            let decisions: Vec<_> = logs
                .lines()
                .filter(|l| l.contains("next_match_id_reply"))
                .collect();
            assert_eq!(decisions.len(), 1, "{logs}");
            let expected = match scenario {
                "fetch_error" => "no_match",
                "config_error" => "disabled",
                "record_error" => "error",
                _ => scenario,
            };
            assert!(
                decisions[0].contains(&format!("decision={expected}")),
                "{logs}"
            );
            assert!(
                !logs.contains("987654321") && !logs.contains("openid_123"),
                "{logs}"
            );
        }
    }
    use crate::auth::{
        application::official_bind::CompleteOfficialBindUseCase,
        domain::official_bind::{ConsumeBindCodeOutcome, IssueBindCodeOutcome},
        ports::official_bind_repository::OfficialBindRepository,
    };
    struct WebhookBindFake {
        outcome: ConsumeBindCodeOutcome,
        fail: bool,
        calls: std::sync::atomic::AtomicUsize,
    }
    #[async_trait]
    impl OfficialBindRepository for WebhookBindFake {
        async fn issue(
            &self,
            _: uuid::Uuid,
            _: &str,
            _: chrono::DateTime<chrono::Utc>,
            _: chrono::DateTime<chrono::Utc>,
        ) -> anyhow::Result<IssueBindCodeOutcome> {
            unreachable!()
        }
        async fn consume(
            &self,
            hash: &str,
            openid: &str,
        ) -> anyhow::Result<ConsumeBindCodeOutcome> {
            self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            assert_eq!(
                hash,
                crate::auth::domain::official_bind::hash_bind_code("ABC234")
            );
            assert_eq!(openid, "openid_123");
            if self.fail {
                anyhow::bail!("ABC234 openid_123 user_id=fixture");
            }
            Ok(self.outcome)
        }
    }
    fn binding_use_case(scenario: &str) -> HandleWechatWebhookUseCase {
        let repo = Arc::new(WebhookBindFake {
            outcome: match scenario {
                "success" | "override" => ConsumeBindCodeOutcome::Success,
                "already_bound" => ConsumeBindCodeOutcome::AlreadyBound,
                _ => ConsumeBindCodeOutcome::Invalid,
            },
            fail: scenario == "error",
            calls: std::sync::atomic::AtomicUsize::new(0),
        });
        HandleWechatWebhookUseCase::new(
            Arc::new(FakeRepository::default()),
            Arc::new(FakeCryptoPort),
            Arc::new(FakeCurrentStandardMatchPort),
            Arc::new(next_match_enabled_config()),
            Arc::new(GateAccessFake::granted()),
        )
        .with_official_bind(Arc::new(CompleteOfficialBindUseCase::new(repo)))
    }
    fn bind_expected(scenario: &str) -> String {
        let text = match scenario {
            "success" | "override" => {
                "【足球洞察】绑定成功\n\n已将本公众号与你的小程序账号关联。请保持关注本公众号，取关后会员权益会暂停。\n\n现在可以回复“下一场”查询比赛 ID（V6 及以上会员可用）。"
            }
            "already_bound" => {
                "【足球洞察】已绑定\n\n你的账号已经绑定过本公众号，无需重复操作。可以直接回复“下一场”查询比赛 ID（V6 及以上会员可用）。"
            }
            "missing_code" => {
                "【足球洞察】绑定公众号\n\n请先打开小程序「足球洞察」，在「我的」页点「绑定公众号」获取绑定码，然后回复“绑定 + 绑定码”，例如：绑定 ABC234"
            }
            "rate_limited" => "【足球洞察】操作过于频繁，请稍后再试。",
            _ => {
                "【足球洞察】绑定失败\n\n绑定码无效或已过期。请回到小程序「我的」页，点「绑定公众号」重新获取绑定码，再回复“绑定 + 绑定码”。"
            }
        };
        super::with_product_intro(text.into())
    }
    #[tokio::test]
    async fn official_bind_commands_have_uniform_failures_exact_copy_and_both_reply_modes() {
        for scenario in [
            "success",
            "override",
            "already_bound",
            "invalid",
            "expired",
            "used",
            "inactive",
            "error",
            "missing_code",
            "malformed",
        ] {
            for plain in [false, true] {
                for command in [
                    "绑定ABC234",
                    "绑定 abc234",
                    "绑定  ABC234",
                    "绑 定 a b c 2 3 4",
                ] {
                    let use_case = binding_use_case(scenario);
                    let command = match scenario {
                        "missing_code" => "绑定",
                        "malformed" => "绑定000000",
                        _ => command,
                    };
                    let body = if plain {
                        format!("<xml><Content><![CDATA[{command}]]></Content></xml>")
                    } else {
                        command.into()
                    };
                    let result = use_case
                        .handle_message(WechatMessageInput {
                            signature: "sig".into(),
                            timestamp: "1".into(),
                            nonce: "2".into(),
                            body,
                        })
                        .await
                        .unwrap()
                        .unwrap();
                    assert_eq!(
                        result,
                        format!(
                            "{}{}",
                            if plain { "plain::" } else { "encrypted::" },
                            bind_expected(scenario)
                        )
                    );
                    assert!(!result.contains("MATCH_2026_001"));
                    assert!(!result.contains("openid_123"));
                }
            }
        }
    }
    #[tokio::test(flavor = "current_thread")]
    #[ignore = "run log capture separately with --ignored --test-threads=1"]
    async fn official_bind_logs_redact_every_input_and_have_one_decision_per_command() {
        let bytes = Arc::new(Mutex::new(Vec::new()));
        let sink = bytes.clone();
        let _guard = tracing::subscriber::set_default(
            tracing_subscriber::fmt()
                .without_time()
                .with_ansi(false)
                .with_writer(move || GateLogWriter(sink.clone()))
                .finish(),
        );
        for scenario in [
            "success",
            "already_bound",
            "invalid",
            "error",
            "missing_code",
            "rate_limited",
        ] {
            let use_case = binding_use_case(scenario);
            if scenario == "rate_limited" {
                for _ in 0..5 {
                    use_case
                        .handle_message(WechatMessageInput {
                            signature: "sig".into(),
                            timestamp: "1".into(),
                            nonce: "2".into(),
                            body: "绑定 ABC234".into(),
                        })
                        .await
                        .unwrap();
                }
            }
            bytes.lock().unwrap().clear();
            let body = if scenario == "missing_code" {
                "绑定"
            } else {
                "绑定 abc234"
            };
            let reply = use_case
                .handle_message(WechatMessageInput {
                    signature: "sig".into(),
                    timestamp: "1".into(),
                    nonce: "2".into(),
                    body: body.into(),
                })
                .await
                .unwrap()
                .unwrap();
            assert_eq!(reply, format!("encrypted::{}", bind_expected(scenario)));
            let logs = String::from_utf8(bytes.lock().unwrap().clone()).unwrap();
            let decisions: Vec<_> = logs
                .lines()
                .filter(|l| l.contains("wechat_bind decision="))
                .collect();
            assert_eq!(decisions.len(), 1, "{logs}");
            assert!(decisions[0].contains(&format!("decision={scenario}")));
            assert!(
                !logs.contains("ABC234")
                    && !logs.contains("abc234")
                    && !logs.contains("openid_123")
                    && !logs.contains("user_id")
            );
        }
    }
}
