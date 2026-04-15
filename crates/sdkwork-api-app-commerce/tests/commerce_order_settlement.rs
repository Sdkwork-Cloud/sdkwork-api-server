use sdkwork_api_app_commerce::{
    settle_portal_commerce_order, settle_portal_commerce_order_from_verified_payment,
    submit_portal_commerce_order, PortalCommerceQuoteRequest,
};
use sdkwork_api_domain_marketing::{
    CampaignBudgetRecord, CampaignBudgetStatus, CouponBenefitSpec, CouponCodeRecord,
    CouponCodeStatus, CouponDistributionKind, CouponRedemptionStatus, CouponReservationStatus,
    CouponRestrictionSpec, CouponTemplateRecord, CouponTemplateStatus, MarketingBenefitKind,
    MarketingCampaignRecord, MarketingCampaignStatus, MarketingSubjectScope,
};
use sdkwork_api_storage_core::{AdminStore, MarketingStore};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

async fn seed_percent_off_coupon(
    store: &SqliteAdminStore,
    code: &str,
    discount_percent: u8,
    eligible_target_kinds: &[&str],
) {
    let template = CouponTemplateRecord::new(
        format!("template_{code}"),
        code,
        MarketingBenefitKind::PercentageOff,
    )
    .with_display_name(format!("{code} launch coupon"))
    .with_status(CouponTemplateStatus::Active)
    .with_distribution_kind(CouponDistributionKind::UniqueCode)
    .with_restriction(
        CouponRestrictionSpec::new(MarketingSubjectScope::Project).with_eligible_target_kinds(
            eligible_target_kinds
                .iter()
                .map(|kind| (*kind).to_owned())
                .collect(),
        ),
    )
    .with_benefit(
        CouponBenefitSpec::new(MarketingBenefitKind::PercentageOff)
            .with_discount_percent(Some(discount_percent)),
    )
    .with_created_at_ms(1_710_000_000)
    .with_updated_at_ms(1_710_000_000);
    MarketingStore::insert_coupon_template_record(store, &template)
        .await
        .expect("insert coupon template");

    let campaign = MarketingCampaignRecord::new(
        format!("campaign_{code}"),
        template.coupon_template_id.clone(),
    )
    .with_display_name(format!("{code} campaign"))
    .with_status(MarketingCampaignStatus::Active)
    .with_created_at_ms(1_710_000_000)
    .with_updated_at_ms(1_710_000_000);
    MarketingStore::insert_marketing_campaign_record(store, &campaign)
        .await
        .expect("insert marketing campaign");

    let budget = CampaignBudgetRecord::new(
        format!("budget_{code}"),
        campaign.marketing_campaign_id.clone(),
    )
    .with_status(CampaignBudgetStatus::Active)
    .with_total_budget_minor(10_000)
    .with_created_at_ms(1_710_000_000)
    .with_updated_at_ms(1_710_000_000);
    MarketingStore::insert_campaign_budget_record(store, &budget)
        .await
        .expect("insert campaign budget");

    let coupon_code = CouponCodeRecord::new(
        format!("coupon_code_{code}"),
        template.coupon_template_id.clone(),
        code,
    )
    .with_status(CouponCodeStatus::Available)
    .with_created_at_ms(1_710_000_000)
    .with_updated_at_ms(1_710_000_000);
    MarketingStore::insert_coupon_code_record(store, &coupon_code)
        .await
        .expect("insert coupon code");
}

async fn seed_bonus_coupon(store: &SqliteAdminStore, code: &str, grant_units: u64) {
    let template = CouponTemplateRecord::new(
        format!("template_{code}"),
        code,
        MarketingBenefitKind::GrantUnits,
    )
    .with_display_name(format!("{code} bonus coupon"))
    .with_status(CouponTemplateStatus::Active)
    .with_distribution_kind(CouponDistributionKind::UniqueCode)
    .with_restriction(CouponRestrictionSpec::new(MarketingSubjectScope::Project))
    .with_benefit(
        CouponBenefitSpec::new(MarketingBenefitKind::GrantUnits)
            .with_grant_units(Some(grant_units)),
    )
    .with_created_at_ms(1_710_000_000)
    .with_updated_at_ms(1_710_000_000);
    MarketingStore::insert_coupon_template_record(store, &template)
        .await
        .expect("insert bonus coupon template");

    let campaign = MarketingCampaignRecord::new(
        format!("campaign_{code}"),
        template.coupon_template_id.clone(),
    )
    .with_display_name(format!("{code} bonus campaign"))
    .with_status(MarketingCampaignStatus::Active)
    .with_created_at_ms(1_710_000_000)
    .with_updated_at_ms(1_710_000_000);
    MarketingStore::insert_marketing_campaign_record(store, &campaign)
        .await
        .expect("insert bonus marketing campaign");

    let budget = CampaignBudgetRecord::new(
        format!("budget_{code}"),
        campaign.marketing_campaign_id.clone(),
    )
    .with_status(CampaignBudgetStatus::Active)
    .with_total_budget_minor(10_000)
    .with_created_at_ms(1_710_000_000)
    .with_updated_at_ms(1_710_000_000);
    MarketingStore::insert_campaign_budget_record(store, &budget)
        .await
        .expect("insert bonus campaign budget");

    let coupon_code = CouponCodeRecord::new(
        format!("coupon_code_{code}"),
        template.coupon_template_id.clone(),
        code,
    )
    .with_status(CouponCodeStatus::Available)
    .with_created_at_ms(1_710_000_000)
    .with_updated_at_ms(1_710_000_000);
    MarketingStore::insert_coupon_code_record(store, &coupon_code)
        .await
        .expect("insert bonus coupon code");
}

#[tokio::test]
async fn restored_pending_order_replay_does_not_reapply_quota_or_reconsume_coupon() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    seed_percent_off_coupon(&store, "LIVE10", 10, &["recharge_pack"]).await;

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

    let reservations_after_first = MarketingStore::list_coupon_reservation_records(&store)
        .await
        .expect("coupon reservations after first settlement");
    assert_eq!(reservations_after_first.len(), 1);
    assert_eq!(
        reservations_after_first[0].reservation_status,
        CouponReservationStatus::Confirmed
    );

    let coupon_after_first = MarketingStore::find_coupon_code_record_by_value(&store, "LIVE10")
        .await
        .expect("coupon code lookup after first settlement")
        .expect("coupon code exists after first settlement");
    assert_eq!(coupon_after_first.status, CouponCodeStatus::Redeemed);

    let redemptions_after_first = MarketingStore::list_coupon_redemption_records(&store)
        .await
        .expect("coupon redemptions after first settlement");
    assert_eq!(redemptions_after_first.len(), 1);
    assert_eq!(
        redemptions_after_first[0].redemption_status,
        CouponRedemptionStatus::Redeemed
    );
    assert_eq!(
        redemptions_after_first[0].order_id.as_deref(),
        Some(order.order_id.as_str())
    );

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

    let coupon_after_replay = MarketingStore::find_coupon_code_record_by_value(&store, "LIVE10")
        .await
        .expect("coupon code lookup after replay")
        .expect("coupon code exists after replay");
    assert_eq!(coupon_after_replay.status, CouponCodeStatus::Redeemed);

    let redemptions_after_replay = MarketingStore::list_coupon_redemption_records(&store)
        .await
        .expect("coupon redemptions after replay");
    assert_eq!(redemptions_after_replay.len(), 1);
    assert_eq!(
        redemptions_after_replay[0].coupon_redemption_id,
        redemptions_after_first[0].coupon_redemption_id
    );
}

#[tokio::test]
async fn instant_coupon_redemption_replay_does_not_double_apply_quota() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    seed_bonus_coupon(&store, "WELCOME100", 100).await;

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

    let orders = AdminStore::list_commerce_orders_for_project(&store, "project-2")
        .await
        .expect("orders for coupon redemption replay");
    assert_eq!(orders.len(), 1);
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
