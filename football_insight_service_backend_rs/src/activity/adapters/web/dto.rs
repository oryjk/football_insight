use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RecordPageActivityRequest {
    pub page_key: String,
}
