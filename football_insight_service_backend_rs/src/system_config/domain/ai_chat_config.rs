#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AiChatSystemConfig {
    pub model: Option<String>,
    pub base_url: Option<String>,
}

impl AiChatSystemConfig {
    pub fn new(model: Option<String>, base_url: Option<String>) -> Self {
        Self { model, base_url }
    }
}
