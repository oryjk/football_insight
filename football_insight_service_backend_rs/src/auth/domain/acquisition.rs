#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcquisitionSource {
    pub src: String,
    pub cid: Option<String>,
    pub landing_scene: Option<i32>,
}

impl AcquisitionSource {
    pub fn new(src: Option<&str>, cid: Option<&str>, scene: Option<i64>) -> Option<Self> {
        fn label(value: &str, max: usize) -> bool {
            !value.is_empty()
                && value.len() <= max
                && value
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
        }
        Some(Self {
            src: src.filter(|value| label(value, 32))?.to_owned(),
            cid: cid.filter(|value| label(value, 64)).map(str::to_owned),
            landing_scene: scene
                .filter(|value| (1..=65535).contains(value))
                .map(|value| value as i32),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acquisition_labels_require_valid_channel_and_drop_only_invalid_optional_fields() {
        let value = AcquisitionSource::new(Some("douyin"), Some("test1"), Some(1047)).unwrap();
        assert_eq!(value.src, "douyin");
        assert_eq!(value.cid.as_deref(), Some("test1"));
        assert_eq!(value.landing_scene, Some(1047));
        assert!(
            AcquisitionSource::new(Some(&"a".repeat(32)), Some(&"b".repeat(64)), None).is_some()
        );
        for src in [
            "",
            "Douyin",
            "x/y",
            " x",
            "抖音",
            "<script>",
            &"x".repeat(33),
        ] {
            assert!(AcquisitionSource::new(Some(src), Some("test"), None).is_none());
        }
        assert!(AcquisitionSource::new(None, Some("test"), Some(1047)).is_none());
        for cid in ["", "invalid value", &"x".repeat(65)] {
            assert_eq!(
                AcquisitionSource::new(Some("share"), Some(cid), Some(-1)).unwrap(),
                AcquisitionSource {
                    src: "share".into(),
                    cid: None,
                    landing_scene: None,
                }
            );
        }
        assert_eq!(
            AcquisitionSource::new(Some("xhs"), None, Some(65536))
                .unwrap()
                .landing_scene,
            None
        );
    }
}
