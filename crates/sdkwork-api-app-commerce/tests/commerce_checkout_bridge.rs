use sdkwork_api_app_commerce::load_portal_commerce_checkout_session;
use sdkwork_api_domain_commerce::CommerceOrderRecord;
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[tokio::test]
async fn pending_checkout_uses_canonical_payment_bridge_without_manual_portal_settlement() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let order = CommerceOrderRecord::new(
        "order-bridge-1",
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
    store.insert_commerce_order(&order).await.unwrap();

    let session = load_portal_commerce_checkout_session(&store, "user-1", "project-1", "order-bridge-1")
        .await
        .unwrap();

    assert_eq!(session.session_status, "open");
    assert_eq!(session.provider, "payment_orchestrator");
    assert_eq!(session.mode, "checkout_bridge");
    assert!(session.methods.iter().any(|method| method.id == "provider_handoff"));
    assert!(!session
        .methods
        .iter()
        .any(|method| method.id == "manual_settlement"));
}

#[tokio::test]
async fn zero_pay_coupon_order_keeps_not_required_checkout_behavior() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let order = CommerceOrderRecord::new(
        "order-bridge-2",
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
    )
    .with_applied_coupon_code_option(Some("WELCOME100".to_owned()));
    store.insert_commerce_order(&order).await.unwrap();

    let session = load_portal_commerce_checkout_session(&store, "user-2", "project-2", "order-bridge-2")
        .await
        .unwrap();

    assert_eq!(session.session_status, "not_required");
    assert_eq!(session.provider, "no_payment_required");
    assert_eq!(session.mode, "instant_fulfillment");
    assert!(session.methods.is_empty());
}
