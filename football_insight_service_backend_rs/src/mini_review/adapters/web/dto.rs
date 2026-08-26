use serde::{Deserialize, Serialize};

use crate::mini_review::domain::mini_review_status::MiniReviewStatus;

#[derive(Debug, Deserialize)]
pub struct ReviewStatusQuery {
    pub project_code: String,
    pub version: String,
}

#[derive(Debug, Deserialize)]
pub struct AllocateRequest {
    pub project_code: String,
    pub current_version: String,
    pub version: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SetReviewStatusRequest {
    pub project_code: String,
    pub version: String,
    pub is_reviewing: bool,
    pub status_text: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ReviewStatusDto {
    pub project_code: String,
    pub version: String,
    pub version_code: i64,
    pub is_reviewing: bool,
    pub status_text: String,
}

impl From<MiniReviewStatus> for ReviewStatusDto {
    fn from(status: MiniReviewStatus) -> Self {
        Self {
            project_code: status.project_code,
            version: status.version,
            version_code: status.version_code,
            is_reviewing: status.is_reviewing,
            status_text: status.status_text,
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::ReviewStatusDto;
    use crate::mini_review::domain::mini_review_status::{MiniReviewStatus, Version};

    #[test]
    fn review_status_dto_serializes_flat_payload() {
        let now = Utc.with_ymd_and_hms(2026, 8, 26, 12, 0, 0).unwrap();
        let dto = ReviewStatusDto::from(MiniReviewStatus::new_reviewing(
            "football_insight_mini",
            Version::parse("1.0.55").unwrap(),
            now,
        ));

        let payload = serde_json::to_value(dto).expect("serialize dto");

        assert_eq!(payload["project_code"], "football_insight_mini");
        assert_eq!(payload["version"], "1.0.55");
        assert_eq!(payload["version_code"], 10055);
        assert_eq!(payload["is_reviewing"], true);
        assert_eq!(payload["status_text"], "正在审核");
    }
}
