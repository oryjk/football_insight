use super::{
    domain::{CodeImage, ShareCodeError},
    ports::TicketWatchCodePort,
};
use std::sync::Arc;

pub struct ShareCodeService {
    code: Arc<dyn TicketWatchCodePort>,
}

impl ShareCodeService {
    pub fn new(code: Arc<dyn TicketWatchCodePort>) -> Self {
        Self { code }
    }

    pub async fn ticket_watch(&self) -> Result<CodeImage, ShareCodeError> {
        self.code.image().await
    }
}
