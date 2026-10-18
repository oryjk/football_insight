#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodeImage {
    pub image_base64: String,
    pub mime_type: String,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ShareCodeError {
    #[error("二维码暂时获取失败，请稍后重试")]
    Unavailable,
}
