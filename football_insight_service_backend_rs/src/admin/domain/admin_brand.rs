use crate::system_config::domain::public_system_config::MAX_BRAND_SUBTITLE_CHARS;

#[derive(Debug, thiserror::Error)]
pub enum AdminBrandError {
    #[error("{0}")]
    InvalidInput(String),
    #[error("brand config query failed")]
    Internal,
}

/// 首页品牌栏副标题；None 表示恢复客户端内置默认文案。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrandSubtitle(Option<String>);

impl BrandSubtitle {
    pub fn new(raw: &str) -> Result<Self, AdminBrandError> {
        let value = raw.trim();
        if value.is_empty() {
            return Ok(Self(None));
        }
        if value.chars().any(char::is_control) {
            return Err(AdminBrandError::InvalidInput(
                "subtitle must not contain control characters".into(),
            ));
        }
        if value.chars().count() > MAX_BRAND_SUBTITLE_CHARS {
            return Err(AdminBrandError::InvalidInput(format!(
                "subtitle must be at most {MAX_BRAND_SUBTITLE_CHARS} characters"
            )));
        }
        Ok(Self(Some(value.to_string())))
    }

    pub fn value(&self) -> Option<&str> {
        self.0.as_deref()
    }

    pub fn into_value(self) -> Option<String> {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subtitle_is_trimmed_and_blank_means_default() {
        assert_eq!(BrandSubtitle::new("  成都蓉城球迷的数据主场 ").unwrap().value(), Some("成都蓉城球迷的数据主场"));
        assert_eq!(BrandSubtitle::new("   ").unwrap().value(), None);
    }

    #[test]
    fn subtitle_rejects_overlong_and_control_characters() {
        assert!(BrandSubtitle::new(&"蓉".repeat(MAX_BRAND_SUBTITLE_CHARS)).is_ok());
        assert!(BrandSubtitle::new(&"蓉".repeat(MAX_BRAND_SUBTITLE_CHARS + 1)).is_err());
        assert!(BrandSubtitle::new("a\nb").is_err());
    }
}
