use crate::auth::domain::acquisition::AcquisitionSource;
use serde::{Deserialize, Deserializer};
use serde_json::Value;

/// Ancillary metadata must never reject an otherwise valid authentication request.
pub fn optional_acquisition<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<AcquisitionSource>, D::Error> {
    let value = Value::deserialize(deserializer)?;
    Ok(AcquisitionSource::new(
        value.get("src").and_then(Value::as_str),
        value.get("cid").and_then(Value::as_str),
        value.get("landing_scene").and_then(Value::as_i64),
    ))
}

#[cfg(test)]
mod tests {
    use super::super::handlers::{MiniWechatBindRequest, RegisterRequest, WechatBindRequest};
    use serde_json::json;
    #[test]
    fn acquisition_request_metadata_is_optional_sanitized_and_cannot_select_user_identity() {
        let base = json!({"account_identifier":"tester","password":"secret123"});
        assert!(
            serde_json::from_value::<RegisterRequest>(base.clone())
                .unwrap()
                .acquisition
                .is_none()
        );
        for invalid in [
            json!(null),
            json!(1),
            json!([]),
            json!("x"),
            json!({"src":[]}),
            json!({"src":"<script>"}),
        ] {
            let mut request = base.clone();
            request["acquisition"] = invalid;
            assert!(
                serde_json::from_value::<RegisterRequest>(request)
                    .unwrap()
                    .acquisition
                    .is_none()
            );
        }
        let mut request = base;
        request["acquisition"] =
            json!({"src":"douyin","cid":"test1","landing_scene":1047,"user_id":"forged"});
        let source = serde_json::from_value::<RegisterRequest>(request)
            .unwrap()
            .acquisition
            .unwrap();
        assert_eq!(source.src, "douyin");
        assert_eq!(source.cid.as_deref(), Some("test1"));
        assert_eq!(source.landing_scene, Some(1047));
        let mini = json!({"bind_token":"token","display_name":"name","avatar_data_url":"avatar","acquisition":{"src":"share","cid":12,"landing_scene":"bad"}});
        let source = serde_json::from_value::<MiniWechatBindRequest>(mini)
            .unwrap()
            .acquisition
            .unwrap();
        assert_eq!(source.src, "share");
        assert!(source.cid.is_none());
        assert!(source.landing_scene.is_none());
        assert!(
            serde_json::from_value::<WechatBindRequest>(
                json!({"bind_token":"token","phone_number":"123","password":"secret"})
            )
            .unwrap()
            .acquisition
            .is_none()
        );
    }
}
