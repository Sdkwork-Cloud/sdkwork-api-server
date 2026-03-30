use sdkwork_api_app_coupon::{list_active_coupons, persist_coupon};
use sdkwork_api_domain_coupon::CouponCampaign;
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[tokio::test]
async fn active_coupon_listing_excludes_inactive_and_empty_inventory() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    persist_coupon(
        &store,
        &CouponCampaign::new(
            "coupon-active",
            "ACTIVE10",
            "10% launch discount",
            "new_workspace",
            12,
            true,
            "Active campaign",
            "rolling",
        ),
    )
    .await
    .unwrap();
    persist_coupon(
        &store,
        &CouponCampaign::new(
            "coupon-inactive",
            "INACTIVE10",
            "10% off legacy plan",
            "legacy_workspace",
            6,
            false,
            "Inactive campaign",
            "rolling",
        ),
    )
    .await
    .unwrap();
    persist_coupon(
        &store,
        &CouponCampaign::new(
            "coupon-empty",
            "EMPTY10",
            "10% off starter pack",
            "starter_workspace",
            0,
            true,
            "Exhausted campaign",
            "rolling",
        ),
    )
    .await
    .unwrap();

    let active = list_active_coupons(&store).await.unwrap();
    let codes = active
        .into_iter()
        .map(|coupon| coupon.code)
        .collect::<Vec<_>>();

    assert_eq!(codes, vec!["ACTIVE10"]);
}
