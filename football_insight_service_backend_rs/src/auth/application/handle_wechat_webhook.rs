use std::sync::Arc;

use crate::auth::{
    domain::wechat::{WechatMessageMode, WechatTextReply},
    ports::{
        auth_repository::AuthRepository, current_standard_match_port::CurrentStandardMatchPort,
        wechat_crypto_port::WechatCryptoPort,
    },
};
use crate::system_config::ports::system_config_port::SystemConfigPort;

const PRODUCT_INTRO_ARTICLE_URL: &str = "https://mp.weixin.qq.com/s?__biz=MzA3Nzc1NTk1Mg==&mid=2247483892&idx=1&sn=c41ff0a1987269db51417a11d0c7b885";
const WECHAT_INVITE_REPLY_TEMPLATE_KEY: &str = "wechat_invite_reply_template";

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
}

impl HandleWechatWebhookUseCase {
    pub fn new(
        repository: Arc<dyn AuthRepository>,
        crypto_port: Arc<dyn WechatCryptoPort>,
        current_standard_match_port: Arc<dyn CurrentStandardMatchPort>,
        system_config_port: Arc<dyn SystemConfigPort>,
    ) -> Self {
        Self {
            repository,
            crypto_port,
            current_standard_match_port,
            system_config_port,
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

    pub async fn handle_message(
        &self,
        input: WechatMessageInput,
    ) -> anyhow::Result<Option<String>> {
        tracing::info!(
            timestamp = %input.timestamp,
            nonce = %input.nonce,
            "received wechat webhook message"
        );

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

        if incoming.msg_type == "text" {
            let normalized_content = incoming
                .content
                .as_deref()
                .map(str::trim)
                .unwrap_or_default();
            let normalized_command = normalize_text_command(normalized_content);

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

                let reply_content = match self
                    .current_standard_match_port
                    .fetch_current_match_id()
                    .await
                {
                    Ok(Some(match_id)) => next_match_id_message(&match_id),
                    Ok(None) => next_match_missing_message(),
                    Err(error) => {
                        tracing::warn!(error = %error, "failed to fetch current-standard match id");
                        next_match_missing_message()
                    }
                };

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

    async fn invite_code_message(&self, invite_code: &str) -> String {
        let custom_template = self
            .system_config_port
            .get_config_value(WECHAT_INVITE_REPLY_TEMPLATE_KEY)
            .await
            .ok()
            .flatten();

        build_invite_code_message(invite_code, custom_template.as_deref())
    }
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

fn next_match_id_message(match_id: &str) -> String {
    with_product_intro(format!(
        "【足球洞察】当前比赛信息\n\n当前下一场比赛 match_id：{match_id}\n\n常用指令：\n- 回复“邀请码”或“1”获取注册邀请码\n- 回复“下一场”或“2”再次查询\n- 回复“下一场id”或“3”再次查询"
    ))
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
            let content = if request_body.contains("<Content><![CDATA[邀请码]]></Content>") {
                Some("邀请码".to_string())
            } else if request_body.contains("<Content><![CDATA[下一场]]></Content>") {
                Some("下一场".to_string())
            } else if request_body.contains("<Content><![CDATA[你好]]></Content>") {
                Some("你好".to_string())
            } else {
                None
            };

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

            Ok(None)
        }
    }

    #[tokio::test]
    async fn verify_endpoint_delegates_to_crypto_port() {
        let use_case = HandleWechatWebhookUseCase::new(
            Arc::new(FakeRepository::default()),
            Arc::new(FakeCryptoPort),
            Arc::new(FakeCurrentStandardMatchPort),
            Arc::new(FakeSystemConfigPort::default()),
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
            Arc::new(FakeSystemConfigPort::default()),
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
            Arc::new(FakeSystemConfigPort::default()),
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
    async fn handle_next_match_id_text_message_accepts_whitespace_and_uppercase() {
        let use_case = HandleWechatWebhookUseCase::new(
            Arc::new(FakeRepository::default()),
            Arc::new(FakeCryptoPort),
            Arc::new(FakeCurrentStandardMatchPort),
            Arc::new(FakeSystemConfigPort::default()),
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
            Arc::new(FakeSystemConfigPort::default()),
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
            Arc::new(FakeSystemConfigPort::default()),
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
            Arc::new(FakeSystemConfigPort::default()),
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
            Arc::new(FakeSystemConfigPort::default()),
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
            }),
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
            }),
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
}
