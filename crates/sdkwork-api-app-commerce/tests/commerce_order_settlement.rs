use sdkwork_api_app_commerce::{
    settle_portal_commerce_order, settle_portal_commerce_order_from_verified_payment,
    submit_portal_commerce_order, PortalCommerceQuoteRequest,
};
use sdkwork_api_domain_coupon::CouponCampaign;
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[tokio::test]
async fn restored_pending_order_replay_does_not_reapply_quota_or_reconsume_coupon() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    store
        .insert_coupon(
            &CouponCampaign::new(
                "coupon-live-10",
                "LIVE10",
                "10% off Growth",
                "all",
                1,
                true,
                "Live settlement replay guard",
                "2099-12-31",
            )
            .with_created_at_ms(1_700_000_000),
        )
        .await
        .unwrap();

    let order = submit_portal_commerce_order(
        &store,
        "user-1",
        "project-1",
        &PortalCommerceQuoteRequest {
            target_kind: "recharge_pack".to_owned(),
            target_id: "pack-100k".to_owned(),
            coupon_code: Some("LIVE10".to_owned()),
            current_remaining_units: None,
            custom_amount_cents: None,
        },
    )
    .await
    .unwrap();

    assert_eq!(order.status, "pending_payment");

    let settled = settle_portal_commerce_order_from_verified_payment(
        &store,
        "user-1",
        "project-1",
        &order.order_id,
    )
    .await
    .unwrap();
    assert_eq!(settled.status, "fulfilled");

    let quota_after_first = store
        .list_quota_policies_for_project("project-1")
        .await
        .unwrap();
    assert_eq!(quota_after_first.len(), 1);
    assert_eq!(quota_after_first[0].max_units, 100_000);

    let coupon_after_first = store.find_coupon("coupon-live-10").await.unwrap().unwrap();
    assert_eq!(coupon_after_first.remaining, 0);

    let restored = settled.clone();
    store
        .insert_commerce_order(&sdkwork_api_domain_commerce::CommerceOrderRecord {
            status: "pending_payment".to_owned(),
            ..restored
        })
        .await
        .unwrap();

    let replayed = settle_portal_commerce_order_from_verified_payment(
        &store,
        "user-1",
        "project-1",
        &order.order_id,
    )
    .await
    .unwrap();
    assert_eq!(replayed.status, "fulfilled");

    let quota_after_replay = store
        .list_quota_policies_for_project("project-1")
        .await
        .unwrap();
    assert_eq!(quota_after_replay.len(), 1);
    assert_eq!(quota_after_replay[0].max_units, 100_000);

    let coupon_after_replay = store.find_coupon("coupon-live-10").await.unwrap().unwrap();
    assert_eq!(coupon_after_replay.remaining, 0);
}

#[tokio::test]
async fn instant_coupon_redemption_replay_does_not_double_apply_quota() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let order = submit_portal_commerce_order(
        &store,
        "user-2",
        "project-2",
        &PortalCommerceQuoteRequest {
            target_kind: "coupon_redemption".to_owned(),
            target_id: "WELCOME100".to_owned(),
            coupon_code: None,
            current_remaining_units: None,
            custom_amount_cents: None,
        },
    )
    .await
    .unwrap();

    assert_eq!(order.status, "fulfilled");

    let quota_after_first = store
        .list_quota_policies_for_project("project-2")
        .await
        .unwrap();
    assert_eq!(quota_after_first.len(), 1);
    assert_eq!(quota_after_first[0].max_units, 100);

    let restored = order.clone();
    store
        .insert_commerce_order(&sdkwork_api_domain_commerce::CommerceOrderRecord {
            status: "pending_payment".to_owned(),
            ..restored
        })
        .await
        .unwrap();

    let replayed = settle_portal_commerce_order(&store, "user-2", "project-2", &order.order_id)
        .await
        .unwrap();
    assert_eq!(replayed.status, "fulfilled");

    let quota_after_replay = store
        .list_quota_policies_for_project("project-2")
        .await
        .unwrap();
    assert_eq!(quota_after_replay.len(), 1);
    assert_eq!(quota_after_replay[0].max_units, 100);
}

#[tokio::test]
async fn repeated_identical_payable_submission_reuses_pending_order() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let first = submit_portal_commerce_order(
        &store,
        "user-3",
        "project-3",
        &PortalCommerceQuoteRequest {
            target_kind: "subscription_plan".to_owned(),
            target_id: "growth".to_owned(),
            coupon_code: None,
            current_remaining_units: None,
            custom_amount_cents: None,
        },
    )
    .await
    .unwrap();

    let second = submit_portal_commerce_order(
        &store,
        "user-3",
        "project-3",
        &PortalCommerceQuoteRequest {
            target_kind: "subscription_plan".to_owned(),
            target_id: "growth".to_owned(),
            coupon_code: None,
            current_remaining_units: None,
            custom_amount_cents: None,
        },
    )
    .await
    .unwrap();

    assert_eq!(first.status, "pending_payment");
    assert_eq!(second.status, "pending_payment");
    assert_eq!(second.order_id, first.order_id);

    let orders = store
        .list_commerce_orders_for_project("project-3")
        .await
        .unwrap();
    assert_eq!(orders.len(), 1);
}
