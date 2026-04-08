use super::*;

#[tokio::test]
async fn paid_checkout_session_exposes_structured_payment_rails_and_normalizes_provider_aliases() {
    let store = build_store().await;

    let order = submit_portal_commerce_order(
        &store,
        "user-3",
        "project-3",
        &PortalCommerceQuoteRequest {
            target_kind: "recharge_pack".to_owned(),
            target_id: "pack-100k".to_owned(),
            coupon_code: None,
            current_remaining_units: Some(1_250),
            custom_amount_cents: None,
        },
    )
    .await
    .expect("order should be created");

    let session =
        load_portal_commerce_checkout_session(&store, "user-3", "project-3", &order.order_id)
            .await
            .expect("checkout session");

    assert_eq!(session.provider, "manual_lab");
    assert_eq!(session.mode, "operator_settlement");
    assert!(session.methods.iter().any(|method| {
        method.id == "manual_settlement"
            && method.provider == "manual_lab"
            && method.channel == "operator_settlement"
            && method.session_kind == "operator_action"
            && method.session_reference.starts_with("MANUAL-")
            && method.qr_code_payload.is_none()
            && method.webhook_verification == "manual"
            && method.supports_refund
            && !method.recommended
            && !method.supports_webhook
    }));
    assert!(session.methods.iter().any(|method| {
        method.id == "stripe_checkout"
            && method.provider == "stripe"
            && method.channel == "hosted_checkout"
            && method.session_kind == "hosted_checkout"
            && method.session_reference.starts_with("STRIPE-")
            && method.qr_code_payload.is_none()
            && method.webhook_verification == "stripe_signature"
            && method.supports_refund
            && method.supports_partial_refund
            && method.recommended
            && method.supports_webhook
    }));
    assert!(session.methods.iter().any(|method| {
        method.id == "alipay_qr"
            && method.provider == "alipay"
            && method.channel == "scan_qr"
            && method.session_kind == "qr_code"
            && method.session_reference.starts_with("ALIPAY-")
            && method
                .qr_code_payload
                .as_deref()
                .is_some_and(|payload| payload.contains("sdkworkpay://alipay_qr/"))
            && method.webhook_verification == "alipay_rsa_sha256"
            && method.supports_refund
            && !method.supports_partial_refund
            && !method.recommended
            && method.supports_webhook
    }));
    assert!(session.methods.iter().any(|method| {
        method.id == "wechat_pay_qr"
            && method.provider == "wechat_pay"
            && method.channel == "scan_qr"
            && method.session_kind == "qr_code"
            && method.session_reference.starts_with("WECHAT-")
            && method
                .qr_code_payload
                .as_deref()
                .is_some_and(|payload| payload.contains("sdkworkpay://wechat_pay_qr/"))
            && method.webhook_verification == "wechatpay_rsa_sha256"
            && method.supports_refund
            && !method.supports_partial_refund
            && !method.recommended
            && method.supports_webhook
    }));

    let settled = apply_portal_commerce_payment_event(
        &store,
        "user-3",
        "project-3",
        &order.order_id,
        &PortalCommercePaymentEventRequest {
            event_type: "settled".to_owned(),
            provider: Some("wechat".to_owned()),
            provider_event_id: Some("evt_alias_paid".to_owned()),
            checkout_method_id: Some("wechat_pay_qr".to_owned()),
            message: None,
        },
    )
    .await
    .expect("settled order");

    assert_eq!(settled.status, "fulfilled");

    let events = AdminStore::list_commerce_payment_events_for_order(&store, &order.order_id)
        .await
        .expect("payment events");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].provider, "wechat_pay");
    assert_eq!(events[0].dedupe_key, "wechat_pay:evt_alias_paid");
    assert!(
        events[0]
            .payload_json
            .contains("\"checkout_method_id\":\"wechat_pay_qr\""),
        "payload should preserve the originating checkout method: {}",
        events[0].payload_json
    );
}

#[tokio::test]
async fn payment_events_reject_unsupported_provider_values() {
    let store = build_store().await;

    let order = submit_portal_commerce_order(
        &store,
        "user-4",
        "project-4",
        &PortalCommerceQuoteRequest {
            target_kind: "recharge_pack".to_owned(),
            target_id: "pack-100k".to_owned(),
            coupon_code: None,
            current_remaining_units: Some(0),
            custom_amount_cents: None,
        },
    )
    .await
    .expect("order should be created");

    let error = apply_portal_commerce_payment_event(
        &store,
        "user-4",
        "project-4",
        &order.order_id,
        &PortalCommercePaymentEventRequest {
            event_type: "settled".to_owned(),
            provider: Some("paypal".to_owned()),
            provider_event_id: Some("evt_unsupported".to_owned()),
            checkout_method_id: None,
            message: None,
        },
    )
    .await
    .expect_err("unsupported providers should be rejected");

    assert!(
        error
            .to_string()
            .contains("unsupported commerce payment provider"),
        "unexpected error: {error}"
    );
}

#[tokio::test]
async fn webhook_backed_checkout_methods_require_provider_event_id() {
    let store = build_store().await;

    let order = submit_portal_commerce_order(
        &store,
        "user-5",
        "project-5",
        &PortalCommerceQuoteRequest {
            target_kind: "recharge_pack".to_owned(),
            target_id: "pack-100k".to_owned(),
            coupon_code: None,
            current_remaining_units: Some(25),
            custom_amount_cents: None,
        },
    )
    .await
    .expect("order should be created");

    let error = apply_portal_commerce_payment_event(
        &store,
        "user-5",
        "project-5",
        &order.order_id,
        &PortalCommercePaymentEventRequest {
            event_type: "settled".to_owned(),
            provider: Some("stripe".to_owned()),
            provider_event_id: None,
            checkout_method_id: Some("stripe_checkout".to_owned()),
            message: None,
        },
    )
    .await
    .expect_err("webhook-backed methods should require provider_event_id");

    assert!(
        error
            .to_string()
            .contains("provider_event_id is required for webhook-backed checkout methods"),
        "unexpected error: {error}"
    );
}

#[tokio::test]
async fn refund_events_reject_provider_mismatch_against_processed_settlement_provider() {
    let store = build_store().await;

    let order = submit_portal_commerce_order(
        &store,
        "user-6",
        "project-6",
        &PortalCommerceQuoteRequest {
            target_kind: "recharge_pack".to_owned(),
            target_id: "pack-100k".to_owned(),
            coupon_code: None,
            current_remaining_units: Some(50),
            custom_amount_cents: None,
        },
    )
    .await
    .expect("order should be created");

    let settled = apply_portal_commerce_payment_event(
        &store,
        "user-6",
        "project-6",
        &order.order_id,
        &PortalCommercePaymentEventRequest {
            event_type: "settled".to_owned(),
            provider: Some("stripe".to_owned()),
            provider_event_id: Some("evt_provider_match_paid".to_owned()),
            checkout_method_id: Some("stripe_checkout".to_owned()),
            message: None,
        },
    )
    .await
    .expect("settled order");
    assert_eq!(settled.status, "fulfilled");

    let error = apply_portal_commerce_payment_event(
        &store,
        "user-6",
        "project-6",
        &order.order_id,
        &PortalCommercePaymentEventRequest {
            event_type: "refunded".to_owned(),
            provider: Some("alipay".to_owned()),
            provider_event_id: Some("evt_provider_mismatch_refund".to_owned()),
            checkout_method_id: Some("alipay_qr".to_owned()),
            message: None,
        },
    )
    .await
    .expect_err("refund provider mismatch should be rejected");

    assert!(
        error
            .to_string()
            .contains("refund provider alipay does not match settled provider stripe"),
        "unexpected error: {error}"
    );

    let events = AdminStore::list_commerce_payment_events_for_order(&store, &order.order_id)
        .await
        .expect("payment events");
    assert_eq!(events.len(), 2);
    assert!(events.iter().any(|event| {
        event.event_type == "refunded"
            && event.provider == "alipay"
            && event.provider_event_id.as_deref() == Some("evt_provider_mismatch_refund")
            && event.processing_status.as_str() == "rejected"
            && event.order_status_after.as_deref() == Some("fulfilled")
    }));
}

#[tokio::test]
async fn provider_backed_payment_events_require_provider_event_id_without_checkout_method_hint() {
    let store = build_store().await;

    let order = submit_portal_commerce_order(
        &store,
        "user-7",
        "project-7",
        &PortalCommerceQuoteRequest {
            target_kind: "recharge_pack".to_owned(),
            target_id: "pack-100k".to_owned(),
            coupon_code: None,
            current_remaining_units: Some(70),
            custom_amount_cents: None,
        },
    )
    .await
    .expect("order should be created");

    let error = apply_portal_commerce_payment_event(
        &store,
        "user-7",
        "project-7",
        &order.order_id,
        &PortalCommercePaymentEventRequest {
            event_type: "settled".to_owned(),
            provider: Some("stripe".to_owned()),
            provider_event_id: None,
            checkout_method_id: None,
            message: None,
        },
    )
    .await
    .expect_err("provider-backed events should require provider_event_id");

    assert!(
        error
            .to_string()
            .contains("provider_event_id is required for provider-backed payment events"),
        "unexpected error: {error}"
    );
}
