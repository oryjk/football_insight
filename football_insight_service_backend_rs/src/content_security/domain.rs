use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ModerationScene {
    Profile = 1,
    Comment = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentSecurityError {
    Rejected,
    Unavailable,
}

impl Display for ContentSecurityError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rejected => f.write_str("内容未通过安全检测"),
            Self::Unavailable => f.write_str("内容安全校验服务暂不可用，请稍后重试"),
        }
    }
}

impl std::error::Error for ContentSecurityError {}
