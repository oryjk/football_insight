use async_trait::async_trait;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeatSwapEvidenceUpload {
    pub file_name: String,
    pub content_type: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeatSwapEvidenceObject {
    pub object_key: String,
    pub public_url: String,
}

#[async_trait]
pub trait SeatSwapEvidenceStoragePort: Send + Sync {
    async fn upload_cancel_evidence(
        &self,
        match_id: i64,
        request_id: uuid::Uuid,
        upload: SeatSwapEvidenceUpload,
    ) -> anyhow::Result<SeatSwapEvidenceObject>;
}
