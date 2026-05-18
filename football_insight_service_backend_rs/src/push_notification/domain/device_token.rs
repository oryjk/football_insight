#[derive(Debug, Clone)]
pub struct DeviceToken {
    pub id: i64,
    pub user_id: uuid::Uuid,
    pub device_token: String,
    pub platform: String,
}
