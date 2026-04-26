use crate::system_config::adapters::persistence::postgres_system_config_port::{
    parse_ai_chat_mode, parse_home_briefing_marquees, parse_optional_config_value, parse_switch,
};
use crate::system_config::domain::public_system_config::{AiChatMode, parse_membership_tier_rules};

#[test]
fn parse_switch_accepts_truthy_values() {
    assert!(parse_switch(Some(" true ")));
    assert!(parse_switch(Some("1")));
    assert!(parse_switch(Some("YES")));
    assert!(!parse_switch(Some("off")));
    assert!(!parse_switch(None));
}

#[test]
fn parse_home_briefing_marquees_reads_json_arrays() {
    let value = parse_home_briefing_marquees(Some(
        r#"{"leader":["成都这波抢分含金量很高","申花还在给榜首施压"],"scorer":["头部火力仍然集中"],"assist":["约尼查的输送稳定性被低估了"]}"#,
    ));

    assert_eq!(
        value.leader,
        vec!["成都这波抢分含金量很高", "申花还在给榜首施压"]
    );
    assert_eq!(value.scorer, vec!["头部火力仍然集中"]);
    assert_eq!(value.assist, vec!["约尼查的输送稳定性被低估了"]);
}

#[test]
fn parse_home_briefing_marquees_ignores_invalid_or_blank_values() {
    let value = parse_home_briefing_marquees(Some(
        r#"{"leader":["   ",""],"scorer":null,"assist":"not-array"}"#,
    ));

    assert!(value.leader.is_empty());
    assert!(value.scorer.is_empty());
    assert!(value.assist.is_empty());
    assert!(
        parse_home_briefing_marquees(Some("not-json"))
            .leader
            .is_empty()
    );
}

#[test]
fn parse_optional_config_value_trims_and_filters_blank_values() {
    assert_eq!(
        parse_optional_config_value(Some("  glm-5.1-fast  ")),
        Some("glm-5.1-fast".to_string())
    );
    assert_eq!(parse_optional_config_value(Some("   ")), None);
    assert_eq!(parse_optional_config_value(None), None);
}

#[test]
fn parse_ai_chat_mode_reads_known_values_and_defaults_safely() {
    assert_eq!(
        parse_ai_chat_mode(Some("frontend_direct")),
        AiChatMode::FrontendDirect
    );
    assert_eq!(
        parse_ai_chat_mode(Some(" backend_proxy ")),
        AiChatMode::BackendProxy
    );
    assert_eq!(
        parse_ai_chat_mode(Some("unknown")),
        AiChatMode::BackendProxy
    );
    assert_eq!(parse_ai_chat_mode(None), AiChatMode::BackendProxy);
}

#[test]
fn parse_membership_tier_rules_reads_json_arrays() {
    let rules = parse_membership_tier_rules(Some(
        r#"[{"code":"V1","kind":"standard","ticket_watch_poll_interval_seconds":900},{"code":"v4","kind":"referral","min_referrals":12,"ticket_watch_poll_interval_seconds":45}]"#,
    ));

    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0].code, "V1");
    assert_eq!(rules[0].ticket_watch_poll_interval_seconds, 900);
    assert_eq!(rules[1].code, "V4");
    assert_eq!(rules[1].min_referrals, Some(12));
}

#[test]
fn parse_membership_tier_rules_falls_back_to_defaults_when_invalid() {
    let rules = parse_membership_tier_rules(Some("not-json"));

    assert_eq!(rules.first().map(|item| item.code.as_str()), Some("V1"));
    assert_eq!(rules.last().map(|item| item.code.as_str()), Some("V9"));
}
