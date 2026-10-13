use football_insight_service_backend_rs::payment::domain::virtual_payment::{
    Delivery, VirtualOrder, VirtualProduct, build_sign_data, product_key, validate_delivery,
};

#[test]
fn catalog_preserves_upgrade_amount_and_removes_match_specific_id() {
    assert_eq!(product_key("membership:V3", 1).unwrap(), "membership:V3@1");
    assert_eq!(
        product_key("membership:V7", 3500).unwrap(),
        "membership:V7@3500"
    );
    assert_eq!(
        product_key("match_id_unlock:123", 500).unwrap(),
        "match_id_unlock@500"
    );
    assert_eq!(
        product_key("reflux_subscription:single:chengdu:123", 100).unwrap(),
        "reflux_subscription:single:chengdu@100"
    );
    assert!(product_key("unknown", 100).is_err());
    assert!(product_key("membership:V7", 0).is_err());
}

#[test]
fn payment_data_contains_exact_price_and_no_secrets() {
    let product = VirtualProduct {
        product_id: "v7_upgrade".into(),
        price_cents: 3500,
    };
    let data = build_sign_data("123", "ORDER123", &product).unwrap();
    let value: serde_json::Value = serde_json::from_str(&data).unwrap();
    assert_eq!(value["goodsPrice"], 3500);
    assert_eq!(value["buyQuantity"], 1);
    assert_eq!(value["env"], 0);
    assert_eq!(value["productId"], "v7_upgrade");
    assert!(value.get("appKey").is_none());
    assert!(build_sign_data("123", "_bad", &product).is_err());
}

fn order() -> VirtualOrder {
    VirtualOrder {
        order_no: "ORDER123".into(),
        open_id: "buyer".into(),
        product_id: "v6".into(),
        amount_cents: 3900,
        wx_order_id: None,
    }
}
fn delivery() -> Delivery {
    Delivery {
        order_no: "ORDER123".into(),
        open_id: "buyer".into(),
        product_id: "v6".into(),
        amount_cents: 3900,
        original_price_cents: 3900,
        quantity: 1,
        env: 0,
        wx_order_id: "WX123".into(),
    }
}

#[test]
fn forged_delivery_cannot_grant_entitlement() {
    assert!(validate_delivery(&order(), &delivery()).is_ok());
    let mut d = delivery();
    d.open_id = "other".into();
    assert!(validate_delivery(&order(), &d).is_err());
    let mut d = delivery();
    d.amount_cents = 1;
    assert!(validate_delivery(&order(), &d).is_err());
    let mut d = delivery();
    d.product_id = "v9".into();
    assert!(validate_delivery(&order(), &d).is_err());
    let mut d = delivery();
    d.env = 1;
    assert!(validate_delivery(&order(), &d).is_err());
    let mut d = delivery();
    d.quantity = 2;
    assert!(validate_delivery(&order(), &d).is_err());
    let mut d = delivery();
    d.original_price_cents = 1;
    assert!(validate_delivery(&order(), &d).is_err());
    let mut o = order();
    o.wx_order_id = Some("WXother".into());
    assert!(validate_delivery(&o, &delivery()).is_err());
}
