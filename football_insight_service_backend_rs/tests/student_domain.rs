use football_insight_service_backend_rs::payment::domain::virtual_payment::product_key;

#[test]
fn student_payment_keys_only_accept_the_two_fixed_prices() {
    for (code, price) in [("certificate_renewal", 9900), ("tool_upgrade", 69900)] {
        let kind = format!("student_service:{code}");
        assert_eq!(
            product_key(&kind, price).unwrap(),
            format!("{kind}@{price}")
        );
        assert!(product_key(&kind, price - 1).is_err());
    }
    assert!(product_key("student_service:unknown", 9900).is_err());
}
