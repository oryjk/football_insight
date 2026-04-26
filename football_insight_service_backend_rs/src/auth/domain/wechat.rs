#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssuedInviteCode {
    pub code: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WechatIncomingMessage {
    pub to_user_name: String,
    pub from_user_name: String,
    pub create_time: i64,
    pub msg_type: String,
    pub event: Option<String>,
    pub content: Option<String>,
    pub app_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WechatMessageMode {
    Plain,
    Encrypted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WechatTextReply {
    pub to_user_name: String,
    pub from_user_name: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WechatOauthProfile {
    pub open_id: String,
    pub union_id: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}
