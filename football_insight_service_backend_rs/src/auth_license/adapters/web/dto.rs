use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct GenerateLicenseResponse {
    pub license_key: String,
    pub expires_at: String,
}

#[derive(Deserialize)]
pub struct BindLicenseRequest {
    pub license_key: String,
}

#[derive(Serialize)]
pub struct BindLicenseResponse {
    pub access_token: String,
    pub user: serde_json::Value,
}
