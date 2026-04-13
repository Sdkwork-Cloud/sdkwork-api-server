use sdkwork_api_app_marketing::{
    find_coupon_redemption, list_coupon_codes, list_coupon_redemptions,
    summarize_coupon_codes, summarize_marketing_overview, summarize_coupon_redemptions,
    ListCouponCodesInput, ListCouponRedemptionsInput,
};
use sdkwork_api_domain_marketing::{
    CouponClaimRecord, CouponClaimStatus, CouponCodeBatchRecord, CouponCodeBatchStatus,
    CouponCodeGenerationMode, CouponTemplateRecord, CouponTemplateStatus, MarketingCampaignKind,
    MarketingCampaignRecord, MarketingCampaignStatus,
    CouponCodeKind, CouponCodeRecord, CouponCodeStatus, CouponRedemptionRecord,
    CouponRedemptionStatus,
};
use sdkwork_api_storage_sqlite::SqliteAdminStore;

async fn memory_store() -> SqliteAdminStore {
    let pool = sdkwork_api_storage_sqlite::run_migrations("sqlite::memory:")
        .await
        .unwrap();
    SqliteAdminStore::new(pool)
}

#[tokio::test]
async fn list_and_summarize_coupon_redemptions_support_subject_filters() {
    let store = memory_store().await;
    store
        .insert_coupon_redemption_record(
            &CouponRedemptionRecord::new(
                701,
                1001,
                2002,
                501,
                101,
                Some(301),
                "user",
                "user_alpha",
                1_710_000_100,
            )
            .with_status(CouponRedemptionStatus::Fulfilled)
            .with_project_id(Some("project_alpha".to_owned()))
            .with_order_id(Some("order_alpha".to_owned()))
            .with_payment_order_id(Some("stripe_pi_alpha".to_owned()))
            .with_subsidy_amount(Some(12.5))
            .with_currency_code(Some("USD".to_owned()))
            .with_updated_at_ms(1_710_000_200),
        )
        .await
        .unwrap();
    store
        .insert_coupon_redemption_record(
            &CouponRedemptionRecord::new(
                702,
                1001,
                2002,
                502,
                101,
                Some(301),
                "user",
                "user_alpha",
                1_710_000_110,
            )
            .with_status(CouponRedemptionStatus::Pending)
            .with_project_id(Some("project_alpha".to_owned()))
            .with_order_id(Some("order_beta".to_owned()))
            .with_updated_at_ms(1_710_000_210),
        )
        .await
        .unwrap();
    store
        .insert_coupon_redemption_record(
            &CouponRedemptionRecord::new(
                703,
                1001,
                2002,
                503,
                102,
                Some(302),
                "user",
                "user_beta",
                1_710_000_120,
            )
            .with_status(CouponRedemptionStatus::Failed)
            .with_project_id(Some("project_beta".to_owned()))
            .with_order_id(Some("order_gamma".to_owned()))
            .with_updated_at_ms(1_710_000_220),
        )
        .await
        .unwrap();

    let filtered = list_coupon_redemptions(
        &store,
        &ListCouponRedemptionsInput::new()
            .with_subject("user", "user_alpha")
            .with_project_id(Some("project_alpha".to_owned()))
            .with_status(Some(CouponRedemptionStatus::Fulfilled)),
    )
    .await
    .unwrap();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].coupon_redemption_id, 701);

    let summary = summarize_coupon_redemptions(
        &store,
        &ListCouponRedemptionsInput::new()
            .with_subject("user", "user_alpha")
            .with_project_id(Some("project_alpha".to_owned())),
    )
    .await
    .unwrap();
    assert_eq!(summary.total_count, 2);
    assert_eq!(summary.pending_count, 1);
    assert_eq!(summary.fulfilled_count, 1);
    assert_eq!(summary.failed_count, 0);
    assert_eq!(summary.payment_linked_count, 1);
    assert_eq!(summary.subsidized_count, 1);
    assert_eq!(summary.total_subsidy_amount, 12.5);
    assert_eq!(summary.latest_created_at_ms, Some(1_710_000_110));

    let detail = find_coupon_redemption(&store, 701).await.unwrap().unwrap();
    assert_eq!(detail.subject_id, "user_alpha");
    assert_eq!(detail.payment_order_id.as_deref(), Some("stripe_pi_alpha"));
}

#[tokio::test]
async fn list_coupon_codes_supports_subject_suffix_and_status_filters() {
    let store = memory_store().await;
    store
        .insert_coupon_code_record(
            &CouponCodeRecord::new(
                801,
                1001,
                2002,
                301,
                101,
                Some(401),
                "hash:wallet-001",
                CouponCodeKind::SingleUseUnique,
                1_710_000_100,
            )
            .with_status(CouponCodeStatus::Claimed)
            .with_claim_subject_type(Some("user".to_owned()))
            .with_claim_subject_id(Some("1001:2002:user_alpha".to_owned()))
            .with_updated_at_ms(1_710_000_200),
        )
        .await
        .unwrap();
    store
        .insert_coupon_code_record(
            &CouponCodeRecord::new(
                802,
                1001,
                2002,
                301,
                101,
                Some(401),
                "hash:wallet-002",
                CouponCodeKind::SingleUseUnique,
                1_710_000_110,
            )
            .with_status(CouponCodeStatus::Redeemed)
            .with_claim_subject_type(Some("user".to_owned()))
            .with_claim_subject_id(Some("1001:2002:user_alpha".to_owned()))
            .with_updated_at_ms(1_710_000_210),
        )
        .await
        .unwrap();
    store
        .insert_coupon_code_record(
            &CouponCodeRecord::new(
                803,
                1001,
                2002,
                302,
                102,
                Some(402),
                "hash:wallet-003",
                CouponCodeKind::SingleUseUnique,
                1_710_000_120,
            )
            .with_status(CouponCodeStatus::Claimed)
            .with_claim_subject_type(Some("user".to_owned()))
            .with_claim_subject_id(Some("1001:2002:user_beta".to_owned()))
            .with_updated_at_ms(1_710_000_220),
        )
        .await
        .unwrap();

    let claimed = list_coupon_codes(
        &store,
        &ListCouponCodesInput::new()
            .with_subject("user", "user_alpha")
            .with_status(Some(CouponCodeStatus::Claimed)),
    )
    .await
    .unwrap();
    assert_eq!(claimed.len(), 1);
    assert_eq!(claimed[0].coupon_code_id, 801);

    let all_for_subject = list_coupon_codes(
        &store,
        &ListCouponCodesInput::new()
            .with_subject("user", "user_alpha")
            .with_marketing_campaign_id(Some(401)),
    )
    .await
    .unwrap();
    assert_eq!(all_for_subject.len(), 2);

    let summary = summarize_coupon_codes(
        &store,
        &ListCouponCodesInput::new().with_subject("user", "user_alpha"),
    )
    .await
    .unwrap();
    assert_eq!(summary.total_count, 2);
    assert_eq!(summary.claimed_count, 1);
    assert_eq!(summary.redeemed_count, 1);
}

#[tokio::test]
async fn summarize_marketing_overview_rolls_up_inventory_and_reservation_counts() {
    let store = memory_store().await;
    store
        .insert_marketing_campaign_record(
            &MarketingCampaignRecord::new(
                401,
                1001,
                2002,
                "campaign_alpha",
                "Campaign Alpha",
                MarketingCampaignKind::Launch,
                1_710_000_100,
            )
            .with_status(MarketingCampaignStatus::Active)
            .with_updated_at_ms(1_710_000_200),
        )
        .await
        .unwrap();
    store
        .insert_marketing_campaign_record(
            &MarketingCampaignRecord::new(
                402,
                1001,
                2002,
                "campaign_beta",
                "Campaign Beta",
                MarketingCampaignKind::Retention,
                1_710_000_101,
            )
            .with_status(MarketingCampaignStatus::Archived)
            .with_updated_at_ms(1_710_000_201),
        )
        .await
        .unwrap();
    store
        .insert_coupon_template_record(
            &CouponTemplateRecord::new(
                101,
                1001,
                2002,
                "template_alpha",
                "Template Alpha",
                sdkwork_api_domain_marketing::CouponBenefitKind::PercentageDiscount,
                sdkwork_api_domain_marketing::CouponDistributionKind::UniqueCode,
                1_710_000_110,
            )
            .with_status(CouponTemplateStatus::Active)
            .with_updated_at_ms(1_710_000_210),
        )
        .await
        .unwrap();
    store
        .insert_coupon_template_record(
            &CouponTemplateRecord::new(
                102,
                1001,
                2002,
                "template_beta",
                "Template Beta",
                sdkwork_api_domain_marketing::CouponBenefitKind::PercentageDiscount,
                sdkwork_api_domain_marketing::CouponDistributionKind::UniqueCode,
                1_710_000_111,
            )
            .with_status(CouponTemplateStatus::Draft)
            .with_updated_at_ms(1_710_000_211),
        )
        .await
        .unwrap();
    store
        .insert_coupon_code_batch_record(
            &CouponCodeBatchRecord::new(
                301,
                1001,
                2002,
                101,
                Some(401),
                CouponCodeGenerationMode::BulkRandom,
                1_710_000_120,
            )
            .with_status(CouponCodeBatchStatus::Active)
            .with_updated_at_ms(1_710_000_220),
        )
        .await
        .unwrap();
    store
        .insert_coupon_code_batch_record(
            &CouponCodeBatchRecord::new(
                302,
                1001,
                2002,
                102,
                Some(402),
                CouponCodeGenerationMode::BulkRandom,
                1_710_000_121,
            )
            .with_status(CouponCodeBatchStatus::Archived)
            .with_updated_at_ms(1_710_000_221),
        )
        .await
        .unwrap();
    store
        .insert_coupon_code_record(
            &CouponCodeRecord::new(
                801,
                1001,
                2002,
                301,
                101,
                Some(401),
                "hash:overview-1",
                CouponCodeKind::SingleUseUnique,
                1_710_000_130,
            )
            .with_status(CouponCodeStatus::Claimed)
            .with_claim_subject_type(Some("user".to_owned()))
            .with_claim_subject_id(Some("1001:2002:user_alpha".to_owned()))
            .with_updated_at_ms(1_710_000_230),
        )
        .await
        .unwrap();
    store
        .insert_coupon_code_record(
            &CouponCodeRecord::new(
                802,
                1001,
                2002,
                302,
                102,
                Some(402),
                "hash:overview-2",
                CouponCodeKind::SingleUseUnique,
                1_710_000_131,
            )
            .with_status(CouponCodeStatus::Redeemed)
            .with_claim_subject_type(Some("user".to_owned()))
            .with_claim_subject_id(Some("1001:2002:user_beta".to_owned()))
            .with_updated_at_ms(1_710_000_231),
        )
        .await
        .unwrap();
    store
        .insert_coupon_claim_record(
            &CouponClaimRecord::new(
                501,
                1001,
                2002,
                801,
                101,
                "user",
                "user_alpha",
                1_710_000_140,
            )
            .with_status(CouponClaimStatus::Claimed)
            .with_updated_at_ms(1_710_000_240),
        )
        .await
        .unwrap();
    store
        .insert_coupon_claim_record(
            &CouponClaimRecord::new(
                502,
                1001,
                2002,
                802,
                102,
                "user",
                "user_beta",
                1_710_000_141,
            )
            .with_status(CouponClaimStatus::Expired)
            .with_updated_at_ms(1_710_000_241),
        )
        .await
        .unwrap();
    store
        .insert_coupon_redemption_record(
            &CouponRedemptionRecord::new(
                701,
                1001,
                2002,
                801,
                101,
                Some(401),
                "user",
                "user_alpha",
                1_710_000_150,
            )
            .with_status(CouponRedemptionStatus::Pending)
            .with_updated_at_ms(1_710_000_250),
        )
        .await
        .unwrap();
    store
        .insert_coupon_redemption_record(
            &CouponRedemptionRecord::new(
                702,
                1001,
                2002,
                802,
                102,
                Some(402),
                "user",
                "user_beta",
                1_710_000_151,
            )
            .with_status(CouponRedemptionStatus::Fulfilled)
            .with_updated_at_ms(1_710_000_251),
        )
        .await
        .unwrap();

    let overview = summarize_marketing_overview(&store).await.unwrap();
    assert_eq!(overview.campaign_count, 2);
    assert_eq!(overview.active_campaign_count, 1);
    assert_eq!(overview.template_count, 2);
    assert_eq!(overview.active_template_count, 1);
    assert_eq!(overview.batch_count, 2);
    assert_eq!(overview.active_batch_count, 1);
    assert_eq!(overview.code_summary.total_count, 2);
    assert_eq!(overview.code_summary.reserved_count, 1);
    assert_eq!(overview.claim_count, 2);
    assert_eq!(overview.claimed_claim_count, 1);
    assert_eq!(overview.redemption_summary.total_count, 2);
    assert_eq!(overview.redemption_summary.pending_count, 1);
}
