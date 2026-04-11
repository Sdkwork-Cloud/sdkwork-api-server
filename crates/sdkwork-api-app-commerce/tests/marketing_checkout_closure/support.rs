use super::*;

pub(super) async fn build_store() -> SqliteAdminStore {
    let (store, _) = build_store_with_pool().await;
    store
}

pub(super) async fn build_store_with_pool() -> (SqliteAdminStore, sqlx::SqlitePool) {
    let pool = run_migrations("sqlite::memory:")
        .await
        .expect("sqlite migrations");
    (SqliteAdminStore::new(pool.clone()), pool)
}

pub(super) async fn seed_percent_off_coupon(
    store: &SqliteAdminStore,
    code: &str,
    discount_percent: u8,
) {
    seed_percent_off_coupon_for_targets(store, code, discount_percent, &[]).await;
}

pub(super) async fn seed_percent_off_coupon_for_targets(
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
        sdkwork_api_domain_marketing::CouponRestrictionSpec::new(
            sdkwork_api_domain_marketing::MarketingSubjectScope::Project,
        )
        .with_eligible_target_kinds(
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
        .expect("insert budget");

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

pub(super) async fn seed_pricing_plan(
    store: &SqliteAdminStore,
    pricing_plan_id: u64,
    plan_code: &str,
    plan_version: u64,
    status: &str,
) {
    let pricing_plan = PricingPlanRecord::new(pricing_plan_id, 1001, 2002, plan_code, plan_version)
        .with_display_name(format!("{plan_code} v{plan_version}"))
        .with_status(status.to_owned())
        .with_effective_from_ms(1_710_000_000_000)
        .with_created_at_ms(1_710_000_000_000 + pricing_plan_id)
        .with_updated_at_ms(1_710_000_000_000 + pricing_plan_id);
    AccountKernelStore::insert_pricing_plan_record(store, &pricing_plan)
        .await
        .expect("insert pricing plan");
}
