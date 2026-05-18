use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct UserLicense {
    pub id: i64,
    pub user_id: uuid::Uuid,
    pub license_key: String,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl UserLicense {
    pub fn is_used(&self) -> bool {
        self.used_at.is_some()
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at <= chrono::Utc::now()
    }

    pub fn is_valid(&self) -> bool {
        !self.is_used() && !self.is_expired()
    }
}

pub fn generate_license_key() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::rng();
    (0..12)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
