use chrono::{DateTime, Utc};
use football_insight_service_backend_rs::payment::domain::{
    order::membership_tier_from_product_type,
    product_kind::{
        GrantSnapshot, MembershipExpiry, ProductKindRegistry, ProductSpec, UserContext,
        is_valid_membership_tier,
    },
    virtual_payment::product_key,
};
use serde_json::json;

fn spec() -> ProductSpec {
    serde_json::from_value(json!({"catalog_product_id":uuid::Uuid::new_v4(),"product_key":"membership:V4@900","kind":"membership","price_cents":900,"membership_duration_months":1,"grant_spec":{"tier":"V4","expiry":{"rule":"end_of_next_day","tz":"Asia/Shanghai"}},"pricing_rule":{"type":"fixed"},"eligibility":{"max_current_tier":"V3"},"requires_capability":"catalog-v2","title":"V4 场次体验卡","subtitle":"次日有效","description":"体验","sort_order":10,"group_key":"pass"})).unwrap()
}

#[test]
fn single_tier_validation_supports_all_v3_to_v9_and_rejects_unknown() {
    for tier in ["V3", "V4", "V5", "V6", "V7", "V8", "V9"] {
        assert!(is_valid_membership_tier(tier));
        assert!(product_key(&format!("membership:{tier}"), 900).is_ok());
        assert_eq!(
            membership_tier_from_product_type(&format!("membership:{tier}")).unwrap(),
            tier
        );
    }
    for tier in ["V2", "V10", "V04", "v4", "unknown"] {
        assert!(!is_valid_membership_tier(tier));
    }
    assert!(membership_tier_from_product_type("membership:V99").is_err());
    assert_eq!(
        membership_tier_from_product_type("membership").unwrap(),
        "V9"
    );
}
#[test]
fn validates_and_checks_capabilities_and_audience_from_data() {
    let registry = ProductKindRegistry::default();
    let s = spec();
    registry.validate(&s).unwrap();
    assert!(!registry.eligible(
        &s,
        &UserContext {
            current_tier: "V3".into(),
            capabilities: vec![]
        }
    ));
    for tier in ["V1", "V2", "V3"] {
        let user = UserContext {
            current_tier: tier.into(),
            capabilities: vec!["catalog-v2".into()],
        };
        assert!(registry.eligible(&s, &user));
        assert_eq!(registry.resolve_price(&s, &user).unwrap(), 900);
    }
    for tier in ["V4", "V5", "V6", "V7", "V8", "V9"] {
        assert!(!registry.eligible(
            &s,
            &UserContext {
                current_tier: tier.into(),
                capabilities: vec!["catalog-v2".into()]
            }
        ));
    }
    let mut invalid = s.clone();
    invalid.eligibility = json!({"max_current_tier":"V3","unknown":true});
    assert!(registry.validate(&invalid).is_err());
    invalid = s.clone();
    invalid.kind = "unknown".into();
    assert!(registry.validate(&invalid).is_err());
}
#[test]
fn day_expiry_is_next_beijing_day_last_second_at_all_boundaries() {
    let expiry = MembershipExpiry::EndOfNextDay {
        tz: "Asia/Shanghai".into(),
    };
    for (input, want) in [
        ("2026-10-16T15:59:59Z", "2026-10-17T15:59:59Z"),
        ("2026-10-16T16:00:00Z", "2026-10-18T15:59:59Z"),
        ("2026-10-31T15:59:59Z", "2026-11-01T15:59:59Z"),
        ("2026-12-31T15:59:59Z", "2027-01-01T15:59:59Z"),
    ] {
        let now: DateTime<Utc> = input.parse().unwrap();
        assert_eq!(
            expiry.expires_at(now).unwrap(),
            want.parse::<DateTime<Utc>>().unwrap()
        );
    }
    assert!(
        MembershipExpiry::EndOfNextDay { tz: "UTC".into() }
            .expires_at(Utc::now())
            .is_err()
    );
}
#[test]
fn snapshots_are_validated_independently_of_mutable_catalog() {
    let s = spec();
    let snapshot = GrantSnapshot::from_spec(&s).unwrap();
    assert_eq!(snapshot.membership().unwrap().tier, "V4");
    let mut invalid = snapshot;
    invalid.grant_spec = json!({"tier":"V99","expiry":{"rule":"months","months":12}});
    assert!(invalid.membership().is_err());
}

#[test]
fn data_backed_fixed_offer_does_not_inherit_legacy_annual_upgrade_credit() {
    use football_insight_service_backend_rs::payment::domain::order::{
        calculate_membership_checkout_price, default_membership_product_options,
    };
    let mut products = default_membership_product_options();
    let option = products.iter_mut().find(|p| p.target_tier == "V7").unwrap();
    let mut offer = spec();
    offer.product_key = "membership:V7@500".into();
    offer.price_cents = 500;
    offer.grant_spec = json!({"tier":"V7","expiry":{"rule":"months","months":1}});
    offer.eligibility = json!({});
    option.price_cents = 500;
    option.presentation.spec = Some(offer);
    let price = calculate_membership_checkout_price(&products, "V6", "V7").unwrap();
    assert_eq!(price.pay_price_cents, 500);
    assert_eq!(price.upgrade_fee_cents, 0);
}
