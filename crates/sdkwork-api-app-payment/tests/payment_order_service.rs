use sdkwork_api_app_payment::{
    ensure_commerce_payment_checkout, project_commerce_checkout_bridge, PaymentSubjectScope,
};
use sdkwork_api_domain_commerce::CommerceOrderRecord;
use sdkwork_api_domain_payment::{
    PaymentAttemptStatus, PaymentOrderStatus, PaymentProviderCode, PaymentSessionKind,
    PaymentSessionStatus,
};
use sdkwork_api_storage_core::PaymentKernelStore;
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[tokio::test]
async fn payable_commerce_order_creates_idempotent_canonical_payment_artifacts() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let scope = PaymentSubjectScope::new(1, 0, 7);
    let order = CommerceOrderRecord::new(
        "commerce-order-1",
        "project-1",
        "user-1",
        "recharge_pack",
        "pack-100k",
        "Boost 100k",
        4_000,
        4_000,
        "$40.00",
        "$40.00",
        100_000,
        0,
        "pending_payment",
        "workspace_seed",
        1_700_000_000,
    );

    let first = ensure_commerce_payment_checkout(&store, &scope, &order, "portal_web")
        .await
        .unwrap();
    let second = ensure_commerce_payment_checkout(&store, &scope, &order, "portal_web")
        .await
        .unwrap();
    let first_order = first.payment_order_opt.as_ref().unwrap();
    let second_order = second.payment_order_opt.as_ref().unwrap();
    let first_attempt = first.payment_attempt_opt.as_ref().unwrap();
    let first_session = first.payment_session_opt.as_ref().unwrap();

    assert_eq!(
        first_order.payment_status,
        PaymentOrderStatus::AwaitingCustomer
    );
    assert_eq!(first_order.payment_order_id, second_order.payment_order_id);
    assert_eq!(first_order.commerce_order_id, "commerce-order-1");
    assert_eq!(first_attempt.client_kind, "portal_web");
    assert_eq!(
        first_session.session_kind,
        PaymentSessionKind::HostedCheckout
    );
    assert_eq!(first_session.session_status, PaymentSessionStatus::Open);

    let orders = store.list_payment_order_records().await.unwrap();
    assert_eq!(orders.len(), 1);
    let attempts = store
        .list_payment_attempt_records_for_order(&first_order.payment_order_id)
        .await
        .unwrap();
    assert_eq!(attempts.len(), 1);
    let sessions = store
        .list_payment_session_records_for_attempt(&first_attempt.payment_attempt_id)
        .await
        .unwrap();
    assert_eq!(sessions.len(), 1);
}

#[tokio::test]
async fn zero_pay_coupon_order_skips_payment_artifact_persistence() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let scope = PaymentSubjectScope::new(1, 0, 8);
    let order = CommerceOrderRecord::new(
        "commerce-order-2",
        "project-2",
        "user-2",
        "coupon_redemption",
        "WELCOME100",
        "WELCOME100",
        0,
        0,
        "$0.00",
        "$0.00",
        0,
        100,
        "fulfilled",
        "workspace_seed",
        1_700_000_001,
    );

    let checkout = ensure_commerce_payment_checkout(&store, &scope, &order, "portal_web")
        .await
        .unwrap();

    assert!(checkout.payment_order_opt.is_none());
    assert_eq!(checkout.checkout.provider, "no_payment_required");
    assert_eq!(checkout.checkout.mode, "instant_fulfillment");
    assert!(store.list_payment_order_records().await.unwrap().is_empty());
}

#[tokio::test]
async fn preserves_advanced_payment_state_while_repairing_missing_checkout_artifacts() {
    let seeded_pool = run_migrations("sqlite::memory:").await.unwrap();
    let seeded_store = SqliteAdminStore::new(seeded_pool);
    let scope = PaymentSubjectScope::new(1, 0, 9);
    let order = CommerceOrderRecord::new(
        "commerce-order-4",
        "project-4",
        "user-4",
        "subscription_plan",
        "growth",
        "Growth",
        7_900,
        7_900,
        "$79.00",
        "$79.00",
        100_000,
        0,
        "pending_payment",
        "workspace_seed",
        1_700_000_003,
    );

    let seeded = ensure_commerce_payment_checkout(&seeded_store, &scope, &order, "portal_web")
        .await
        .unwrap();
    let seeded_order = seeded.payment_order_opt.as_ref().unwrap();
    let captured_order_id = seeded_order.payment_order_id.clone();
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let mut captured_order = seeded_order.clone();
    captured_order.payment_status = PaymentOrderStatus::Captured;
    captured_order.fulfillment_status = "captured_pending_fulfillment".to_owned();
    captured_order.provider_code = PaymentProviderCode::Stripe;
    captured_order.updated_at_ms = order.created_at_ms + 120;
    captured_order.version = captured_order.version.saturating_add(1);
    store
        .insert_payment_order_record(&captured_order)
        .await
        .unwrap();

    let repaired = ensure_commerce_payment_checkout(&store, &scope, &order, "portal_web")
        .await
        .unwrap();
    let repaired_order = repaired.payment_order_opt.as_ref().unwrap();
    let repaired_attempt = repaired.payment_attempt_opt.as_ref().unwrap();
    let repaired_session = repaired.payment_session_opt.as_ref().unwrap();

    assert_eq!(repaired_order.payment_status, PaymentOrderStatus::Captured);
    assert_eq!(
        repaired_order.fulfillment_status,
        "captured_pending_fulfillment"
    );
    assert_eq!(repaired_order.provider_code, PaymentProviderCode::Stripe);
    assert_eq!(
        repaired_attempt.attempt_status,
        PaymentAttemptStatus::Succeeded
    );
    assert_eq!(
        repaired_session.session_status,
        PaymentSessionStatus::Settled
    );

    let attempts = store
        .list_payment_attempt_records_for_order(&captured_order_id)
        .await
        .unwrap();
    assert_eq!(attempts.len(), 1);
    let sessions = store
        .list_payment_session_records_for_attempt(&attempts[0].payment_attempt_id)
        .await
        .unwrap();
    assert_eq!(sessions.len(), 1);
}

#[test]
fn pending_order_checkout_projection_exposes_provider_handoff_without_manual_settlement() {
    let order = CommerceOrderRecord::new(
        "commerce-order-3",
        "project-3",
        "user-3",
        "subscription_plan",
        "growth",
        "Growth",
        7_900,
        7_900,
        "$79.00",
        "$79.00",
        100_000,
        0,
        "pending_payment",
        "workspace_seed",
        1_700_000_002,
    );

    let checkout = project_commerce_checkout_bridge(&order).unwrap();

    assert_eq!(checkout.provider, "payment_orchestrator");
    assert_eq!(checkout.mode, "checkout_bridge");
    assert!(checkout
        .methods
        .iter()
        .any(|method| method.id == "provider_handoff"));
    assert!(!checkout
        .methods
        .iter()
        .any(|method| method.id == "manual_settlement"));
}
