use football_insight_service_backend_rs::payment::domain::payment_event::{
    PaymentEventKind, attempt_id, sanitize_detail, sanitize_message, terminal_stage,
};
use serde_json::json;

#[test]
fn client_cannot_forge_server_events() {
    for name in [
        "order_created",
        "pay_params_failed",
        "pay_params_issued",
        "delivered",
        "refunded",
        "unknown",
    ] {
        assert!(PaymentEventKind::from_client_str(name).is_none(), "{name}");
    }
    for name in [
        "purchase_tapped",
        "agreement_prompted",
        "session_prepare_failed",
        "order_create_requested",
        "order_create_failed",
        "pay_sheet_invoked",
        "pay_cancelled",
        "pay_failed",
        "pay_success_client",
        "confirm_timeout",
    ] {
        assert_eq!(
            PaymentEventKind::from_client_str(name).unwrap().as_str(),
            name
        );
    }
}

#[test]
fn attributes_every_terminal_stage_and_prioritizes_delivery() {
    for (event, expected) in [
        ("agreement_prompted", "agreement_not_accepted"),
        ("session_prepare_failed", "session_prepare_failed"),
        ("order_create_failed", "order_failed"),
        ("pay_params_failed", "order_failed"),
        ("pay_params_issued", "params_without_pay_sheet"),
        ("pay_sheet_invoked", "pay_sheet_without_callback"),
        ("pay_cancelled", "cancelled"),
        ("pay_failed", "payment_failed"),
        ("pay_success_client", "paid_awaiting_confirmation"),
        ("confirm_timeout", "confirmation_timeout"),
        ("delivered", "success"),
        ("refunded", "refunded"),
        ("order_created", "unknown"),
    ] {
        assert_eq!(terminal_stage(&[event.into()]), expected, "{event}");
    }
    assert_eq!(
        terminal_stage(&["delivered".into(), "confirm_timeout".into()]),
        "success"
    );
    assert_eq!(
        terminal_stage(&["refunded".into(), "delivered".into()]),
        "refunded"
    );
    assert_eq!(terminal_stage(&[]), "unknown");
}

#[test]
fn sanitizes_sensitive_keys_and_limits_utf8_by_chars_and_bytes() {
    let detail = sanitize_detail(
        &json!({"platform":"ios", "elapsed_ms":12, "openid":"secret", "token":"secret", "phone":"123", "model":{"token":"nested"}}),
    );
    assert_eq!(detail, json!({"platform":"ios", "elapsed_ms":12}));
    let huge = sanitize_detail(&json!({"model":"中".repeat(2000)}));
    assert!(serde_json::to_vec(&huge).unwrap().len() <= 1024);
    assert_eq!(sanitize_message(&"中".repeat(400)).chars().count(), 300);
}

#[test]
fn optional_attempt_header_never_rejects_old_clients() {
    let id = uuid::Uuid::new_v4();
    assert_eq!(attempt_id(Some(&id.to_string())), id);
    assert!(!attempt_id(None).is_nil());
    assert!(!attempt_id(Some("bad")).is_nil());
}

#[test]
fn delayed_batched_precursors_cannot_move_attempt_backwards() {
    assert_eq!(
        terminal_stage(&[
            "pay_params_issued".into(),
            "purchase_tapped".into(),
            "order_create_requested".into()
        ]),
        "params_without_pay_sheet"
    );
    assert_eq!(
        terminal_stage(&["pay_sheet_invoked".into(), "order_created".into()]),
        "pay_sheet_without_callback"
    );
}

#[test]
fn reversed_report_batches_cannot_erase_terminal_payment_evidence() {
    for (event, expected) in [
        ("pay_cancelled", "cancelled"),
        ("pay_failed", "payment_failed"),
        ("pay_success_client", "paid_awaiting_confirmation"),
        ("confirm_timeout", "confirmation_timeout"),
    ] {
        assert_eq!(
            terminal_stage(&[
                event.into(),
                "pay_sheet_invoked".into(),
                "pay_params_issued".into()
            ]),
            expected
        );
    }
    assert_eq!(
        terminal_stage(&["confirm_timeout".into(), "pay_success_client".into()]),
        "confirmation_timeout"
    );
}
