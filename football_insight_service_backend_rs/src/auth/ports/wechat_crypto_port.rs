use crate::auth::domain::wechat::{WechatIncomingMessage, WechatTextReply};

pub trait WechatCryptoPort: Send + Sync {
    fn verify_url(
        &self,
        signature: &str,
        timestamp: &str,
        nonce: &str,
        echostr: &str,
    ) -> anyhow::Result<String>;
    fn decrypt_message(
        &self,
        signature: &str,
        timestamp: &str,
        nonce: &str,
        request_body: &str,
    ) -> anyhow::Result<WechatIncomingMessage>;
    fn parse_plain_message(
        &self,
        signature: &str,
        timestamp: &str,
        nonce: &str,
        request_body: &str,
    ) -> anyhow::Result<WechatIncomingMessage>;
    fn encrypt_reply(
        &self,
        reply: &WechatTextReply,
        timestamp: &str,
        nonce: &str,
        app_id: &str,
    ) -> anyhow::Result<String>;
    fn build_plain_reply(&self, reply: &WechatTextReply, timestamp: &str)
    -> anyhow::Result<String>;
}
