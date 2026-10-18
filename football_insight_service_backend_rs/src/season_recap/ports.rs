use super::domain::{RecapError, RecapMatch};
use async_trait::async_trait;
use uuid::Uuid;
#[derive(Clone, Debug)]
pub struct Snapshot {
    pub matches: Vec<RecapMatch>,
    pub selected: Vec<i64>,
    pub home_one_way_km: Option<f64>,
}
/// Executed against the locked, current snapshot before the repository commits.
/// Calculation/validation stays in application; adapters only manage transaction lifetime.
pub type ValidateSnapshot<'a> = dyn Fn(&Snapshot) -> Result<(), RecapError> + Send + Sync + 'a;
#[async_trait]
pub trait RecapRepository: Send + Sync {
    async fn load(&self, user: Uuid, season: i32) -> Result<Snapshot, RecapError>;
    async fn save(
        &self,
        user: Uuid,
        season: i32,
        selected: &[i64],
        home_km: Option<f64>,
        validate: &ValidateSnapshot<'_>,
    ) -> Result<Snapshot, RecapError>;
}
#[async_trait]
pub trait RecapCodePort: Send + Sync {
    async fn png_base64(&self, season: i32) -> Result<super::domain::CodeImage, RecapError>;
}
