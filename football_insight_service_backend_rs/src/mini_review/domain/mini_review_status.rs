use std::fmt;

use chrono::{DateTime, Utc};

/// 新登记版本的默认状态文案，与小程序审核态提示保持一致。
pub const DEFAULT_REVIEWING_STATUS_TEXT: &str = "正在审核";
/// 小程序运行时查询未登记版本时返回的状态文案。
pub const UNREGISTERED_STATUS_TEXT: &str = "未登记版本";

/// 每段的取值上限：minor/patch 超过 99 会让 major*10000+minor*100+patch
/// 编码进位冲突，因此在解析与递增时都限制在两位内。
const VERSION_SEGMENT_LIMIT: i64 = 99;

/// 三段式版本号 x.y.z，minor/patch 限制在 0-99 保证数值编码唯一。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version {
    pub major: i64,
    pub minor: i64,
    pub patch: i64,
}

impl Version {
    pub fn parse(raw: &str) -> anyhow::Result<Self> {
        let segments: Vec<&str> = raw.trim().split('.').collect();
        if segments.len() != 3 {
            anyhow::bail!("版本号格式必须是 x.y.z");
        }

        let mut values = [0i64; 3];
        for (index, segment) in segments.iter().enumerate() {
            let value: i64 = segment
                .parse()
                .map_err(|_| anyhow::anyhow!("版本号段无效: {raw}"))?;
            if value < 0 || (index > 0 && value > VERSION_SEGMENT_LIMIT) {
                anyhow::bail!("版本号段无效: {raw}");
            }
            values[index] = value;
        }

        Ok(Self {
            major: values[0],
            minor: values[1],
            patch: values[2],
        })
    }

    /// 与小程序构建脚本一致的数值编码（major*10000+minor*100+patch），
    /// 用于排序取最大版本。
    pub fn code(self) -> i64 {
        self.major * 10000 + self.minor * 100 + self.patch
    }

    /// 递增 patch；patch 到 99 时进位到 minor，minor 到 99 时进位到 major。
    pub fn next_patch(self) -> Self {
        if self.patch < VERSION_SEGMENT_LIMIT {
            return Self {
                major: self.major,
                minor: self.minor,
                patch: self.patch + 1,
            };
        }
        if self.minor < VERSION_SEGMENT_LIMIT {
            return Self {
                major: self.major,
                minor: self.minor + 1,
                patch: 0,
            };
        }
        Self {
            major: self.major + 1,
            minor: 0,
            patch: 0,
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// 记录小程序某个版本的提审状态：
/// 生产构建登记新版本（is_reviewing=true），过审后标记通过（false）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MiniReviewStatus {
    pub id: i64,
    pub project_code: String,
    pub version: String,
    pub version_code: i64,
    pub is_reviewing: bool,
    pub status_text: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl MiniReviewStatus {
    /// 登记一个进入审核的新版本记录。
    pub fn new_reviewing(project_code: &str, version: Version, now: DateTime<Utc>) -> Self {
        Self {
            id: 0,
            project_code: project_code.trim().to_string(),
            version: version.to_string(),
            version_code: version.code(),
            is_reviewing: true,
            status_text: DEFAULT_REVIEWING_STATUS_TEXT.to_string(),
            created_at: now,
            updated_at: now,
        }
    }

    /// 已过审的版本重新提审（重复构建同一显式版本的场景）。
    pub fn restart_reviewing(&mut self, now: DateTime<Utc>) {
        self.is_reviewing = true;
        self.status_text = DEFAULT_REVIEWING_STATUS_TEXT.to_string();
        self.updated_at = now;
    }

    /// 更新审核结论：标记通过（is_reviewing=false，附结论文案）或重新打开审核。
    pub fn set_status(&mut self, is_reviewing: bool, status_text: &str, now: DateTime<Utc>) -> anyhow::Result<()> {
        let trimmed = status_text.trim();
        if trimmed.is_empty() {
            anyhow::bail!("审核状态文案不能为空");
        }
        self.is_reviewing = is_reviewing;
        self.status_text = trimmed.to_string();
        self.updated_at = now;
        Ok(())
    }
}

/// 生产构建自动分配版本号的依据。
pub struct AllocationInput {
    /// 该项目当前版本号最大的记录；项目首次登记时为 None。
    pub latest: Option<MiniReviewStatus>,
    /// 构建侧传来的 manifest 当前版本，仅用于库内无任何记录时的首次起点。
    pub seed: Version,
}

/// 决定本次构建应使用的版本号，登记库是唯一权威：
/// 最新记录仍在审核中 → 复用它（重复构建不递增，任何构建机一致）；
/// 最新记录已出审核 → 在它基础上递增 patch（删库重置后版本号随库回落）；
/// 库内无记录 → 以构建侧 manifest 为起点递增。
/// 本地 manifest 不参与后续分配，避免多台构建机因各自 manifest 状态不同而分叉。
pub fn decide_next_version(input: &AllocationInput) -> Version {
    let Some(latest) = input.latest.as_ref() else {
        return input.seed.next_patch();
    };
    let Ok(latest_version) = Version::parse(&latest.version) else {
        return input.seed.next_patch();
    };
    if latest.is_reviewing {
        return latest_version;
    }
    latest_version.next_patch()
}

#[cfg(test)]
mod tests {
    use super::{
        DEFAULT_REVIEWING_STATUS_TEXT, MiniReviewStatus, Version, decide_next_version,
        AllocationInput,
    };
    use chrono::{TimeZone, Utc};

    fn fixed_now() -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 8, 26, 12, 0, 0).unwrap()
    }

    #[test]
    fn version_parse_accepts_three_segments() {
        let version = Version::parse("1.2.3").expect("valid version");
        assert_eq!(version.to_string(), "1.2.3");
        assert_eq!(version.code(), 10203);
    }

    #[test]
    fn version_parse_rejects_invalid_input() {
        for raw in ["1.2", "1.2.3.4", "a.b.c", "1.100.0", "1.2.100", "-1.2.3", ""] {
            assert!(Version::parse(raw).is_err(), "should reject {raw}");
        }
    }

    #[test]
    fn version_next_patch_carries_over_at_99() {
        assert_eq!(Version::parse("1.2.3").unwrap().next_patch().to_string(), "1.2.4");
        assert_eq!(Version::parse("1.2.99").unwrap().next_patch().to_string(), "1.3.0");
        assert_eq!(Version::parse("1.99.99").unwrap().next_patch().to_string(), "2.0.0");
    }

    fn reviewing_status(version: &str, is_reviewing: bool) -> MiniReviewStatus {
        let mut status = MiniReviewStatus::new_reviewing(
            "football_insight_mini",
            Version::parse(version).unwrap(),
            fixed_now(),
        );
        status.is_reviewing = is_reviewing;
        status
    }

    #[test]
    fn decide_next_version_reuses_latest_reviewing_version() {
        let input = AllocationInput {
            latest: Some(reviewing_status("1.0.54", true)),
            seed: Version::parse("9.9.9").unwrap(),
        };
        assert_eq!(decide_next_version(&input).to_string(), "1.0.54");
    }

    #[test]
    fn decide_next_version_increments_after_review_finished() {
        let input = AllocationInput {
            latest: Some(reviewing_status("1.0.54", false)),
            seed: Version::parse("9.9.9").unwrap(),
        };
        assert_eq!(decide_next_version(&input).to_string(), "1.0.55");
    }

    #[test]
    fn decide_next_version_seeds_from_manifest_when_registry_empty() {
        let input = AllocationInput {
            latest: None,
            seed: Version::parse("1.0.54").unwrap(),
        };
        assert_eq!(decide_next_version(&input).to_string(), "1.0.55");
    }

    #[test]
    fn new_reviewing_status_marks_version_reviewing() {
        let status = MiniReviewStatus::new_reviewing(
            "football_insight_mini",
            Version::parse("1.0.55").unwrap(),
            fixed_now(),
        );

        assert_eq!(status.version, "1.0.55");
        assert_eq!(status.version_code, 10055);
        assert!(status.is_reviewing);
        assert_eq!(status.status_text, DEFAULT_REVIEWING_STATUS_TEXT);
    }

    #[test]
    fn set_status_updates_text_and_flag() {
        let mut status = reviewing_status("1.0.55", true);
        status.set_status(false, "已过审", fixed_now());

        assert!(!status.is_reviewing);
        assert_eq!(status.status_text, "已过审");
    }

    #[test]
    fn set_status_rejects_blank_text() {
        let mut status = reviewing_status("1.0.55", true);
        assert!(status.set_status(false, "  ", fixed_now()).is_err());
    }

    #[test]
    fn restart_reviewing_resets_defaults() {
        let mut status = reviewing_status("1.0.55", false);
        status.status_text = "已过审".to_string();
        status.restart_reviewing(fixed_now());

        assert!(status.is_reviewing);
        assert_eq!(status.status_text, DEFAULT_REVIEWING_STATUS_TEXT);
    }
}
