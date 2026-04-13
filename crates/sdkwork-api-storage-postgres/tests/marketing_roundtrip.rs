use sdkwork_api_domain_marketing::{
    AttributionSourceKind, CouponBenefitKind, CouponBenefitRuleRecord, CouponClaimRecord,
    CouponClaimStatus, CouponCodeBatchRecord, CouponCodeBatchStatus, CouponCodeGenerationMode,
    CouponCodeKind, CouponCodeRecord, CouponCodeStatus, CouponDistributionKind,
    CouponRedemptionRecord, CouponRedemptionStatus, CouponStackingPolicy, CouponTemplateRecord,
    CouponTemplateStatus, MarketingAttributionTouchRecord, MarketingCampaignKind,
    MarketingCampaignRecord, MarketingCampaignStatus, ReferralInviteRecord, ReferralInviteStatus,
    ReferralProgramRecord, ReferralProgramStatus,
};
use sdkwork_api_storage_postgres::{run_migrations, PostgresAdminStore};
use sqlx::PgPool;

#[tokio::test]
async fn postgres_store_creates_marketing_kernel_tables_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();

    for table_name in [
        "ai_marketing_coupon_template",
        "ai_marketing_coupon_benefit_rule",
        "ai_marketing_campaign",
        "ai_marketing_coupon_code_batch",
        "ai_marketing_coupon_code",
        "ai_marketing_coupon_claim",
        "ai_marketing_coupon_redemption",
        "ai_marketing_referral_program",
        "ai_marketing_referral_invite",
        "ai_marketing_attribution_touch",
    ] {
        let row: (String,) = sqlx::query_as(
            "select tablename
             from pg_tables
             where schemaname = 'public' and tablename = $1",
        )
        .bind(table_name)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(row.0, table_name);
    }

    assert_pg_column(
        &pool,
        "ai_marketing_coupon_code",
        "code_lookup_hash",
        "text",
        false,
        None,
    )
    .await;
    assert_pg_column(
        &pool,
        "ai_marketing_coupon_code",
        "claim_subject_id",
        "text",
        true,
        None,
    )
    .await;
    assert_pg_column(
        &pool,
        "ai_marketing_coupon_claim",
        "subject_type",
        "text",
        false,
        None,
    )
    .await;
    assert_pg_column(
        &pool,
        "ai_marketing_coupon_claim",
        "subject_id",
        "text",
        false,
        None,
    )
    .await;
    assert_pg_column(
        &pool,
        "ai_marketing_coupon_redemption",
        "subject_type",
        "text",
        false,
        None,
    )
    .await;
    assert_pg_column(
        &pool,
        "ai_marketing_coupon_redemption",
        "subject_id",
        "text",
        false,
        None,
    )
    .await;
    assert_pg_column(
        &pool,
        "ai_marketing_coupon_redemption",
        "idempotency_key",
        "text",
        true,
        None,
    )
    .await;

    let index_names: Vec<(String,)> = sqlx::query_as(
        "select indexname
         from pg_indexes
         where schemaname = 'public'
           and tablename in (
             'ai_marketing_coupon_template',
             'ai_marketing_campaign',
             'ai_marketing_coupon_code_batch',
             'ai_marketing_coupon_code',
             'ai_marketing_coupon_redemption'
           )
         order by indexname",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    let index_names = index_names
        .into_iter()
        .map(|(name,)| name)
        .collect::<std::collections::HashSet<_>>();

    for index_name in [
        "idx_ai_marketing_coupon_template_scope_status",
        "idx_ai_marketing_campaign_scope_status",
        "idx_ai_marketing_coupon_code_batch_template_status",
        "idx_ai_marketing_coupon_code_lookup_hash",
        "idx_ai_marketing_coupon_code_subject",
        "idx_ai_marketing_coupon_claim_subject_status",
        "idx_ai_marketing_coupon_redemption_lineage",
        "idx_ai_marketing_coupon_redemption_idempotency",
    ] {
        assert!(
            index_names.contains(index_name),
            "missing index {index_name}"
        );
    }
}

#[tokio::test]
async fn postgres_store_round_trips_marketing_kernel_records_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);

    let template = CouponTemplateRecord::new(
        101,
        1001,
        2002,
        "spring-launch",
        "Spring launch 20% off",
        CouponBenefitKind::PercentageDiscount,
        CouponDistributionKind::SharedCode,
        1_710_000_000,
    )
    .with_status(CouponTemplateStatus::Active)
    .with_stacking_policy(CouponStackingPolicy::ExclusiveWithinGroup)
    .with_exclusive_group(Some("seasonal-discount".to_owned()))
    .with_ends_at_ms(Some(1_710_999_999))
    .with_max_total_redemptions(Some(20_000))
    .with_max_redemptions_per_subject(Some(1))
    .with_claim_required(false)
    .with_updated_at_ms(1_710_000_100);

    let benefit_rule = CouponBenefitRuleRecord::new(
        201,
        1001,
        2002,
        template.coupon_template_id,
        CouponBenefitKind::PercentageDiscount,
        1_710_000_001,
    )
    .with_target_order_kind(Some("recharge_pack".to_owned()))
    .with_target_product_id(Some("pack-100k".to_owned()))
    .with_percentage_off(Some(20.0))
    .with_maximum_subsidy_amount(Some(50.0))
    .with_currency_code(Some("USD".to_owned()))
    .with_updated_at_ms(1_710_000_101);

    let campaign = MarketingCampaignRecord::new(
        301,
        1001,
        2002,
        "launch-2026-q2",
        "Q2 Launch Push",
        MarketingCampaignKind::Launch,
        1_710_000_002,
    )
    .with_status(MarketingCampaignStatus::Active)
    .with_channel_source(Some("partner-marketplace".to_owned()))
    .with_budget_amount(Some(20_000.0))
    .with_currency_code(Some("USD".to_owned()))
    .with_subsidy_cap_amount(Some(15_000.0))
    .with_owner_user_id(Some(9001))
    .with_ends_at_ms(Some(1_710_999_999))
    .with_updated_at_ms(1_710_000_102);

    let batch = CouponCodeBatchRecord::new(
        401,
        1001,
        2002,
        template.coupon_template_id,
        Some(campaign.marketing_campaign_id),
        CouponCodeGenerationMode::BulkRandom,
        1_710_000_003,
    )
    .with_status(CouponCodeBatchStatus::Active)
    .with_batch_kind(Some("partner-distribution".to_owned()))
    .with_code_prefix(Some("SPR".to_owned()))
    .with_issued_count(5000)
    .with_claimed_count(120)
    .with_redeemed_count(75)
    .with_voided_count(3)
    .with_expires_at_ms(Some(1_710_999_999))
    .with_updated_at_ms(1_710_000_103);

    let code = CouponCodeRecord::new(
        501,
        1001,
        2002,
        batch.coupon_code_batch_id,
        template.coupon_template_id,
        Some(campaign.marketing_campaign_id),
        "hash:spring-code-001",
        CouponCodeKind::SingleUseUnique,
        1_710_000_004,
    )
    .with_status(CouponCodeStatus::Claimed)
    .with_display_code_prefix(Some("SPR".to_owned()))
    .with_display_code_suffix(Some("001".to_owned()))
    .with_claim_subject_type(Some("user".to_owned()))
    .with_claim_subject_id(Some("1001:2002:9002".to_owned()))
    .with_claimed_at_ms(Some(1_710_000_123))
    .with_expires_at_ms(Some(1_710_999_999))
    .with_updated_at_ms(1_710_000_104);

    let claim = CouponClaimRecord::new(
        601,
        1001,
        2002,
        code.coupon_code_id,
        template.coupon_template_id,
        "user",
        "9002",
        1_710_000_005,
    )
    .with_status(CouponClaimStatus::Claimed)
    .with_account_id(Some(7001))
    .with_project_id(Some("project-live".to_owned()))
    .with_expires_at_ms(Some(1_710_999_999))
    .with_updated_at_ms(1_710_000_105);

    let redemption = CouponRedemptionRecord::new(
        701,
        1001,
        2002,
        code.coupon_code_id,
        template.coupon_template_id,
        Some(campaign.marketing_campaign_id),
        "user",
        "9002",
        1_710_000_006,
    )
    .with_status(CouponRedemptionStatus::Fulfilled)
    .with_account_id(Some(7001))
    .with_project_id(Some("project-live".to_owned()))
    .with_order_id(Some("order_123".to_owned()))
    .with_payment_order_id(Some("pay_123".to_owned()))
    .with_benefit_lot_id(Some(8801))
    .with_pricing_adjustment_id(Some("pricing_adjustment_1".to_owned()))
    .with_subsidy_amount(Some(18.5))
    .with_currency_code(Some("USD".to_owned()))
    .with_idempotency_key(Some("redeem:501:9002".to_owned()))
    .with_updated_at_ms(1_710_000_106);

    let referral_program = ReferralProgramRecord::new(
        801,
        1001,
        2002,
        "invite-growth",
        "Invite Growth",
        1_710_000_007,
    )
    .with_status(ReferralProgramStatus::Active)
    .with_invite_reward_template_id(Some(111))
    .with_referee_reward_template_id(Some(112))
    .with_ends_at_ms(Some(1_710_999_999))
    .with_updated_at_ms(1_710_000_107);

    let referral_invite = ReferralInviteRecord::new(
        901,
        1001,
        2002,
        referral_program.referral_program_id,
        9002,
        1_710_000_008,
    )
    .with_status(ReferralInviteStatus::Rewarded)
    .with_coupon_code_id(Some(code.coupon_code_id))
    .with_source_code(Some("INVITE-A1B2".to_owned()))
    .with_referred_user_id(Some(9003))
    .with_accepted_at_ms(Some(1_710_000_050))
    .with_rewarded_at_ms(Some(1_710_000_200))
    .with_updated_at_ms(1_710_000_108);

    let touch = MarketingAttributionTouchRecord::new(
        1001,
        1001,
        2002,
        AttributionSourceKind::Referral,
        1_710_000_009,
    )
    .with_source_code(Some("INVITE-A1B2".to_owned()))
    .with_utm_source(Some("partner".to_owned()))
    .with_utm_campaign(Some("launch-2026-q2".to_owned()))
    .with_utm_medium(Some("invite".to_owned()))
    .with_partner_id(Some("partner-01".to_owned()))
    .with_referrer_user_id(Some(9002))
    .with_invite_code_id(Some(code.coupon_code_id))
    .with_conversion_subject_id(Some("workspace:9003".to_owned()))
    .with_converted_at_ms(Some(1_710_000_250))
    .with_updated_at_ms(1_710_000_109);

    store
        .insert_coupon_template_record(&template)
        .await
        .unwrap();
    store
        .insert_coupon_benefit_rule_record(&benefit_rule)
        .await
        .unwrap();
    store
        .insert_marketing_campaign_record(&campaign)
        .await
        .unwrap();
    store.insert_coupon_code_batch_record(&batch).await.unwrap();
    store.insert_coupon_code_record(&code).await.unwrap();
    store.insert_coupon_claim_record(&claim).await.unwrap();
    store
        .insert_coupon_redemption_record(&redemption)
        .await
        .unwrap();
    store
        .insert_referral_program_record(&referral_program)
        .await
        .unwrap();
    store
        .insert_referral_invite_record(&referral_invite)
        .await
        .unwrap();
    store
        .insert_marketing_attribution_touch_record(&touch)
        .await
        .unwrap();

    assert_eq!(
        store.find_coupon_template_record(101).await.unwrap(),
        Some(template)
    );
    assert_eq!(
        store
            .list_coupon_benefit_rule_records()
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        store.list_marketing_campaign_records().await.unwrap().len(),
        1
    );
    assert_eq!(
        store.list_coupon_code_batch_records().await.unwrap().len(),
        1
    );
    assert_eq!(
        store
            .find_coupon_code_record_by_lookup_hash("hash:spring-code-001")
            .await
            .unwrap()
            .map(|record| record.coupon_code_id),
        Some(501)
    );
    assert_eq!(
        store
            .list_coupon_code_records_for_subject("user", "1001:2002:9002")
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(store.list_coupon_claim_records().await.unwrap().len(), 1);
    assert_eq!(
        store.list_coupon_claim_records().await.unwrap()[0].subject_id,
        "9002"
    );
    assert_eq!(
        store
            .find_coupon_redemption_record_by_idempotency_key("redeem:501:9002")
            .await
            .unwrap()
            .map(|record| record.coupon_redemption_id),
        Some(701)
    );
    assert_eq!(
        store
            .find_coupon_redemption_record_by_idempotency_key("redeem:501:9002")
            .await
            .unwrap()
            .map(|record| record.subject_id),
        Some("9002".to_owned())
    );
    assert_eq!(
        store.list_referral_program_records().await.unwrap().len(),
        1
    );
    assert_eq!(store.list_referral_invite_records().await.unwrap().len(), 1);
    assert_eq!(
        store
            .list_marketing_attribution_touch_records()
            .await
            .unwrap()
            .len(),
        1
    );
}

async fn assert_pg_column(
    pool: &PgPool,
    table_name: &str,
    column_name: &str,
    data_type: &str,
    nullable: bool,
    default_contains: Option<&str>,
) {
    let row: (String, String, Option<String>) = sqlx::query_as(
        "select data_type, is_nullable, column_default
         from information_schema.columns
         where table_schema = 'public'
           and table_name = $1
           and column_name = $2",
    )
    .bind(table_name)
    .bind(column_name)
    .fetch_one(pool)
    .await
    .unwrap();

    assert_eq!(row.0, data_type);
    assert_eq!(row.1 == "YES", nullable);
    match default_contains {
        Some(expected) => assert!(
            row.2
                .as_deref()
                .is_some_and(|value| value.contains(expected)),
            "expected default for {table_name}.{column_name} to contain {expected:?}, got {:?}",
            row.2
        ),
        None => {}
    }
}
