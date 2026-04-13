use sdkwork_api_app_marketing::{
    claim_coupon_code, expire_due_coupon_codes, issue_coupon_code, redeem_coupon_code,
    release_coupon_redemption_reservation, reserve_coupon_redemption, validate_coupon_for_quote,
    void_coupon_code, ClaimCouponCodeInput, ExpireDueCouponCodesInput, IssueCouponCodeInput,
    RedeemCouponCodeInput, ReleaseCouponRedemptionReservationInput,
    ReserveCouponRedemptionInput, ValidateCouponForQuoteInput, VoidCouponCodeInput,
};
use sdkwork_api_domain_marketing::{
    CouponBenefitKind, CouponBenefitRuleRecord, CouponClaimRecord, CouponClaimStatus,
    CouponCodeBatchRecord, CouponCodeBatchStatus, CouponCodeGenerationMode, CouponCodeKind,
    CouponCodeRecord, CouponCodeStatus, CouponDistributionKind, CouponRedemptionStatus,
    CouponTemplateRecord, CouponTemplateStatus, MarketingCampaignKind, MarketingCampaignRecord,
    MarketingCampaignStatus,
};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[tokio::test]
async fn issue_coupon_code_inherits_batch_expiry_and_campaign_context() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let template = CouponTemplateRecord::new(
        100,
        1001,
        2002,
        "spring-launch",
        "Spring launch 20% off",
        CouponBenefitKind::PercentageDiscount,
        CouponDistributionKind::UniqueCode,
        1_710_000_000,
    )
    .with_status(CouponTemplateStatus::Active)
    .with_updated_at_ms(1_710_000_100);
    store
        .insert_coupon_template_record(&template)
        .await
        .unwrap();

    let campaign = MarketingCampaignRecord::new(
        300,
        1001,
        2002,
        "campaign-spring",
        "Spring launch campaign",
        MarketingCampaignKind::Launch,
        1_710_000_000,
    )
    .with_status(MarketingCampaignStatus::Active)
    .with_starts_at_ms(Some(1_709_999_000))
    .with_ends_at_ms(Some(1_710_100_000))
    .with_updated_at_ms(1_710_000_101);
    store
        .insert_marketing_campaign_record(&campaign)
        .await
        .unwrap();

    let batch = CouponCodeBatchRecord::new(
        400,
        1001,
        2002,
        template.coupon_template_id,
        Some(campaign.marketing_campaign_id),
        CouponCodeGenerationMode::BulkRandom,
        1_710_000_010,
    )
    .with_status(CouponCodeBatchStatus::Active)
    .with_expires_at_ms(Some(1_710_050_000))
    .with_updated_at_ms(1_710_000_102);
    store.insert_coupon_code_batch_record(&batch).await.unwrap();

    let issued = issue_coupon_code(
        &store,
        IssueCouponCodeInput::new(
            500,
            batch.coupon_code_batch_id,
            "hash:spring-code-001",
            CouponCodeKind::SingleUseUnique,
            1_710_000_200,
        ),
    )
    .await
    .unwrap();

    assert_eq!(issued.coupon_template_id, template.coupon_template_id);
    assert_eq!(
        issued.marketing_campaign_id,
        Some(campaign.marketing_campaign_id)
    );
    assert_eq!(issued.expires_at_ms, Some(1_710_050_000));
    assert_eq!(issued.status, CouponCodeStatus::Issued);

    let stored_code = store.find_coupon_code_record(500).await.unwrap().unwrap();
    assert_eq!(stored_code.marketing_campaign_id, Some(300));

    let stored_batches = store.list_coupon_code_batch_records().await.unwrap();
    assert_eq!(stored_batches[0].issued_count, 1);
}

#[tokio::test]
async fn claim_coupon_code_uses_lookup_hash_and_updates_subject_ownership() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let template = CouponTemplateRecord::new(
        101,
        1001,
        2002,
        "spring-launch",
        "Spring launch 20% off",
        CouponBenefitKind::PercentageDiscount,
        CouponDistributionKind::UniqueCode,
        1_710_000_000,
    )
    .with_status(CouponTemplateStatus::Active)
    .with_updated_at_ms(1_710_000_100);
    store
        .insert_coupon_template_record(&template)
        .await
        .unwrap();

    let batch = CouponCodeBatchRecord::new(
        401,
        1001,
        2002,
        template.coupon_template_id,
        None,
        CouponCodeGenerationMode::BulkRandom,
        1_710_000_010,
    )
    .with_status(CouponCodeBatchStatus::Active)
    .with_updated_at_ms(1_710_000_101);
    store.insert_coupon_code_batch_record(&batch).await.unwrap();

    let code = CouponCodeRecord::new(
        501,
        1001,
        2002,
        batch.coupon_code_batch_id,
        template.coupon_template_id,
        None,
        "hash:spring-code-001",
        CouponCodeKind::SingleUseUnique,
        1_710_000_020,
    )
    .with_status(CouponCodeStatus::Issued)
    .with_updated_at_ms(1_710_000_102);
    store.insert_coupon_code_record(&code).await.unwrap();

    let claim = claim_coupon_code(
        &store,
        ClaimCouponCodeInput::new(601, "user", "9002", "hash:spring-code-001", 1_710_000_200)
            .with_account_id(Some(7001))
            .with_project_id(Some("project-live".to_owned())),
    )
    .await
    .unwrap();

    assert_eq!(claim.coupon_code_id, 501);
    assert_eq!(claim.subject_type, "user");
    assert_eq!(claim.subject_id, "9002");
    assert_eq!(claim.account_id, Some(7001));

    let stored_code = store
        .find_coupon_code_record_by_lookup_hash("hash:spring-code-001")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored_code.status, CouponCodeStatus::Claimed);
    assert_eq!(stored_code.claim_subject_type.as_deref(), Some("user"));
    assert_eq!(
        stored_code.claim_subject_id.as_deref(),
        Some("1001:2002:9002")
    );
    assert_eq!(store.list_coupon_claim_records().await.unwrap().len(), 1);
}

#[tokio::test]
async fn claim_coupon_code_rejects_campaigns_outside_active_window() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let template = CouponTemplateRecord::new(
        106,
        1001,
        2002,
        "spring-future",
        "Future spring launch",
        CouponBenefitKind::PercentageDiscount,
        CouponDistributionKind::UniqueCode,
        1_710_000_000,
    )
    .with_status(CouponTemplateStatus::Active)
    .with_updated_at_ms(1_710_000_100);
    store
        .insert_coupon_template_record(&template)
        .await
        .unwrap();

    let campaign = MarketingCampaignRecord::new(
        306,
        1001,
        2002,
        "campaign-future",
        "Future launch campaign",
        MarketingCampaignKind::Launch,
        1_710_000_000,
    )
    .with_status(MarketingCampaignStatus::Active)
    .with_starts_at_ms(Some(1_710_100_000))
    .with_ends_at_ms(Some(1_710_200_000))
    .with_updated_at_ms(1_710_000_101);
    store
        .insert_marketing_campaign_record(&campaign)
        .await
        .unwrap();

    let batch = CouponCodeBatchRecord::new(
        406,
        1001,
        2002,
        template.coupon_template_id,
        Some(campaign.marketing_campaign_id),
        CouponCodeGenerationMode::BulkRandom,
        1_710_000_010,
    )
    .with_status(CouponCodeBatchStatus::Active)
    .with_updated_at_ms(1_710_000_102);
    store.insert_coupon_code_batch_record(&batch).await.unwrap();

    let code = CouponCodeRecord::new(
        506,
        1001,
        2002,
        batch.coupon_code_batch_id,
        template.coupon_template_id,
        Some(campaign.marketing_campaign_id),
        "hash:future-campaign-code",
        CouponCodeKind::SingleUseUnique,
        1_710_000_020,
    )
    .with_status(CouponCodeStatus::Issued)
    .with_updated_at_ms(1_710_000_103);
    store.insert_coupon_code_record(&code).await.unwrap();

    let error = claim_coupon_code(
        &store,
        ClaimCouponCodeInput::new(
            606,
            "user",
            "9002",
            "hash:future-campaign-code",
            1_710_000_200,
        ),
    )
    .await
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("marketing campaign is not active yet"));
}

#[tokio::test]
async fn void_coupon_code_blocks_future_redemption() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let template = CouponTemplateRecord::new(
        103,
        1001,
        2002,
        "member-claim",
        "Member claim reward",
        CouponBenefitKind::CreditGrant,
        CouponDistributionKind::UniqueCode,
        1_710_000_000,
    )
    .with_status(CouponTemplateStatus::Active)
    .with_updated_at_ms(1_710_000_100);
    store
        .insert_coupon_template_record(&template)
        .await
        .unwrap();

    let batch = CouponCodeBatchRecord::new(
        403,
        1001,
        2002,
        template.coupon_template_id,
        None,
        CouponCodeGenerationMode::BulkRandom,
        1_710_000_010,
    )
    .with_status(CouponCodeBatchStatus::Active)
    .with_updated_at_ms(1_710_000_101);
    store.insert_coupon_code_batch_record(&batch).await.unwrap();

    let code = CouponCodeRecord::new(
        503,
        1001,
        2002,
        batch.coupon_code_batch_id,
        template.coupon_template_id,
        None,
        "hash:void-code-001",
        CouponCodeKind::SingleUseUnique,
        1_710_000_020,
    )
    .with_status(CouponCodeStatus::Issued)
    .with_updated_at_ms(1_710_000_102);
    store.insert_coupon_code_record(&code).await.unwrap();

    claim_coupon_code(
        &store,
        ClaimCouponCodeInput::new(602, "user", "9002", "hash:void-code-001", 1_710_000_200),
    )
    .await
    .unwrap();

    let voided = void_coupon_code(&store, VoidCouponCodeInput::new(503, 1_710_000_250))
        .await
        .unwrap();
    assert_eq!(voided.status, CouponCodeStatus::Voided);

    let stored_batch = store.list_coupon_code_batch_records().await.unwrap();
    assert_eq!(stored_batch[0].voided_count, 1);

    let redeem_error = redeem_coupon_code(
        &store,
        RedeemCouponCodeInput::new(
            702,
            "user",
            "9002",
            "hash:void-code-001",
            "redeem:void-code-001:9002",
            1_710_000_300,
        ),
    )
    .await
    .unwrap_err();
    assert!(redeem_error
        .to_string()
        .contains("coupon code has been voided"));
}

#[tokio::test]
async fn expire_due_coupon_codes_marks_codes_and_claims_expired() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let template = CouponTemplateRecord::new(
        104,
        1001,
        2002,
        "limited-window",
        "Limited window reward",
        CouponBenefitKind::CreditGrant,
        CouponDistributionKind::UniqueCode,
        1_710_000_000,
    )
    .with_status(CouponTemplateStatus::Active)
    .with_updated_at_ms(1_710_000_100);
    store
        .insert_coupon_template_record(&template)
        .await
        .unwrap();

    let batch = CouponCodeBatchRecord::new(
        404,
        1001,
        2002,
        template.coupon_template_id,
        None,
        CouponCodeGenerationMode::BulkRandom,
        1_710_000_010,
    )
    .with_status(CouponCodeBatchStatus::Active)
    .with_updated_at_ms(1_710_000_101);
    store.insert_coupon_code_batch_record(&batch).await.unwrap();

    let code = CouponCodeRecord::new(
        504,
        1001,
        2002,
        batch.coupon_code_batch_id,
        template.coupon_template_id,
        None,
        "hash:expired-code-001",
        CouponCodeKind::SingleUseUnique,
        1_710_000_020,
    )
    .with_status(CouponCodeStatus::Claimed)
    .with_claim_subject_type(Some("user".to_owned()))
    .with_claim_subject_id(Some("1001:2002:9002".to_owned()))
    .with_claimed_at_ms(Some(1_710_000_120))
    .with_expires_at_ms(Some(1_710_000_150))
    .with_updated_at_ms(1_710_000_120);
    store.insert_coupon_code_record(&code).await.unwrap();

    let claim = CouponClaimRecord::new(
        603,
        1001,
        2002,
        code.coupon_code_id,
        template.coupon_template_id,
        "user",
        "9002",
        1_710_000_120,
    )
    .with_status(CouponClaimStatus::Claimed)
    .with_expires_at_ms(Some(1_710_000_150))
    .with_updated_at_ms(1_710_000_120);
    store.insert_coupon_claim_record(&claim).await.unwrap();

    let expired = expire_due_coupon_codes(&store, ExpireDueCouponCodesInput::new(1_710_000_200))
        .await
        .unwrap();
    assert_eq!(expired.expired_code_ids, vec![504]);
    assert_eq!(expired.expired_claim_ids, vec![603]);

    let stored_code = store.find_coupon_code_record(504).await.unwrap().unwrap();
    assert_eq!(stored_code.status, CouponCodeStatus::Expired);

    let stored_claims = store.list_coupon_claim_records().await.unwrap();
    assert_eq!(stored_claims[0].status, CouponClaimStatus::Expired);
}

#[tokio::test]
async fn validate_coupon_for_quote_returns_percentage_discount_rule() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let template = CouponTemplateRecord::new(
        105,
        1001,
        2002,
        "growth-discount",
        "Growth pack discount",
        CouponBenefitKind::PercentageDiscount,
        CouponDistributionKind::UniqueCode,
        1_710_000_000,
    )
    .with_status(CouponTemplateStatus::Active)
    .with_claim_required(false)
    .with_updated_at_ms(1_710_000_100);
    store
        .insert_coupon_template_record(&template)
        .await
        .unwrap();

    let rule = CouponBenefitRuleRecord::new(
        205,
        1001,
        2002,
        template.coupon_template_id,
        CouponBenefitKind::PercentageDiscount,
        1_710_000_010,
    )
    .with_target_order_kind(Some("recharge_pack".to_owned()))
    .with_percentage_off(Some(15.0))
    .with_maximum_subsidy_amount(Some(20.0))
    .with_currency_code(Some("USD".to_owned()))
    .with_updated_at_ms(1_710_000_101);
    store
        .insert_coupon_benefit_rule_record(&rule)
        .await
        .unwrap();

    let batch = CouponCodeBatchRecord::new(
        405,
        1001,
        2002,
        template.coupon_template_id,
        None,
        CouponCodeGenerationMode::BulkRandom,
        1_710_000_020,
    )
    .with_status(CouponCodeBatchStatus::Active)
    .with_updated_at_ms(1_710_000_102);
    store.insert_coupon_code_batch_record(&batch).await.unwrap();

    let code = CouponCodeRecord::new(
        505,
        1001,
        2002,
        batch.coupon_code_batch_id,
        template.coupon_template_id,
        None,
        "hash:quote-code-001",
        CouponCodeKind::SingleUseUnique,
        1_710_000_030,
    )
    .with_status(CouponCodeStatus::Issued)
    .with_updated_at_ms(1_710_000_103);
    store.insert_coupon_code_record(&code).await.unwrap();

    let validation = validate_coupon_for_quote(
        &store,
        ValidateCouponForQuoteInput::new("user", "9002", "hash:quote-code-001", 1_710_000_200)
            .with_target_order_kind(Some("recharge_pack".to_owned())),
    )
    .await
    .unwrap();

    assert_eq!(validation.coupon_code_id, 505);
    assert_eq!(validation.coupon_benefit_rule_id, 205);
    assert_eq!(validation.percentage_off, Some(15.0));
    assert_eq!(validation.maximum_subsidy_amount, Some(20.0));
    assert_eq!(validation.currency_code.as_deref(), Some("USD"));
}

#[tokio::test]
async fn validate_coupon_for_quote_supports_string_subject_ids_for_claim_required_codes() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let template = CouponTemplateRecord::new(
        106,
        1001,
        2002,
        "workspace-discount",
        "Workspace claimed discount",
        CouponBenefitKind::PercentageDiscount,
        CouponDistributionKind::UniqueCode,
        1_710_000_000,
    )
    .with_status(CouponTemplateStatus::Active)
    .with_claim_required(true)
    .with_updated_at_ms(1_710_000_100);
    store
        .insert_coupon_template_record(&template)
        .await
        .unwrap();

    let rule = CouponBenefitRuleRecord::new(
        206,
        1001,
        2002,
        template.coupon_template_id,
        CouponBenefitKind::PercentageDiscount,
        1_710_000_010,
    )
    .with_target_order_kind(Some("recharge_pack".to_owned()))
    .with_percentage_off(Some(12.0))
    .with_updated_at_ms(1_710_000_101);
    store
        .insert_coupon_benefit_rule_record(&rule)
        .await
        .unwrap();

    let batch = CouponCodeBatchRecord::new(
        406,
        1001,
        2002,
        template.coupon_template_id,
        None,
        CouponCodeGenerationMode::BulkRandom,
        1_710_000_020,
    )
    .with_status(CouponCodeBatchStatus::Active)
    .with_updated_at_ms(1_710_000_102);
    store.insert_coupon_code_batch_record(&batch).await.unwrap();

    let code = CouponCodeRecord::new(
        506,
        1001,
        2002,
        batch.coupon_code_batch_id,
        template.coupon_template_id,
        None,
        "hash:workspace-quote-code-001",
        CouponCodeKind::SingleUseUnique,
        1_710_000_030,
    )
    .with_status(CouponCodeStatus::Claimed)
    .with_claim_subject_type(Some("user".to_owned()))
    .with_claim_subject_id(Some("1001:2002:user_workspace_owner".to_owned()))
    .with_claimed_at_ms(Some(1_710_000_120))
    .with_updated_at_ms(1_710_000_120);
    store.insert_coupon_code_record(&code).await.unwrap();

    let validation = validate_coupon_for_quote(
        &store,
        ValidateCouponForQuoteInput::new(
            "user",
            "user_workspace_owner",
            "hash:workspace-quote-code-001",
            1_710_000_200,
        )
        .with_target_order_kind(Some("recharge_pack".to_owned())),
    )
    .await
    .unwrap();

    assert_eq!(validation.coupon_code_id, 506);
    assert_eq!(validation.coupon_benefit_rule_id, 206);
    assert_eq!(validation.percentage_off, Some(12.0));
}

#[tokio::test]
async fn claim_and_redeem_update_batch_counters() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let template = CouponTemplateRecord::new(
        107,
        1001,
        2002,
        "counter-check",
        "Counter check reward",
        CouponBenefitKind::CreditGrant,
        CouponDistributionKind::UniqueCode,
        1_710_000_000,
    )
    .with_status(CouponTemplateStatus::Active)
    .with_updated_at_ms(1_710_000_100);
    store
        .insert_coupon_template_record(&template)
        .await
        .unwrap();

    let batch = CouponCodeBatchRecord::new(
        407,
        1001,
        2002,
        template.coupon_template_id,
        None,
        CouponCodeGenerationMode::BulkRandom,
        1_710_000_010,
    )
    .with_status(CouponCodeBatchStatus::Active)
    .with_issued_count(1)
    .with_updated_at_ms(1_710_000_101);
    store.insert_coupon_code_batch_record(&batch).await.unwrap();

    let code = CouponCodeRecord::new(
        507,
        1001,
        2002,
        batch.coupon_code_batch_id,
        template.coupon_template_id,
        None,
        "hash:counter-code-001",
        CouponCodeKind::SingleUseUnique,
        1_710_000_020,
    )
    .with_status(CouponCodeStatus::Issued)
    .with_updated_at_ms(1_710_000_102);
    store.insert_coupon_code_record(&code).await.unwrap();

    claim_coupon_code(
        &store,
        ClaimCouponCodeInput::new(607, "user", "9002", "hash:counter-code-001", 1_710_000_200),
    )
    .await
    .unwrap();

    let batch_after_claim = store.list_coupon_code_batch_records().await.unwrap();
    assert_eq!(batch_after_claim[0].claimed_count, 1);

    redeem_coupon_code(
        &store,
        RedeemCouponCodeInput::new(
            707,
            "user",
            "9002",
            "hash:counter-code-001",
            "redeem:counter-code-001:9002",
            1_710_000_300,
        ),
    )
    .await
    .unwrap();

    let batch_after_redeem = store.list_coupon_code_batch_records().await.unwrap();
    assert_eq!(batch_after_redeem[0].redeemed_count, 1);
}

#[tokio::test]
async fn redeem_coupon_code_is_idempotent_for_the_same_idempotency_key() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let template = CouponTemplateRecord::new(
        102,
        1001,
        2002,
        "grant-launch",
        "Launch reward",
        CouponBenefitKind::CreditGrant,
        CouponDistributionKind::UniqueCode,
        1_710_000_000,
    )
    .with_status(CouponTemplateStatus::Active)
    .with_claim_required(false)
    .with_updated_at_ms(1_710_000_100);
    store
        .insert_coupon_template_record(&template)
        .await
        .unwrap();

    let batch = CouponCodeBatchRecord::new(
        402,
        1001,
        2002,
        template.coupon_template_id,
        None,
        CouponCodeGenerationMode::BulkRandom,
        1_710_000_010,
    )
    .with_status(CouponCodeBatchStatus::Active)
    .with_updated_at_ms(1_710_000_101);
    store.insert_coupon_code_batch_record(&batch).await.unwrap();

    let code = CouponCodeRecord::new(
        502,
        1001,
        2002,
        batch.coupon_code_batch_id,
        template.coupon_template_id,
        None,
        "hash:grant-code-001",
        CouponCodeKind::SingleUseUnique,
        1_710_000_020,
    )
    .with_status(CouponCodeStatus::Issued)
    .with_updated_at_ms(1_710_000_102);
    store.insert_coupon_code_record(&code).await.unwrap();

    let first = redeem_coupon_code(
        &store,
        RedeemCouponCodeInput::new(
            701,
            "user",
            "9002",
            "hash:grant-code-001",
            "redeem:grant-code-001:9002",
            1_710_000_300,
        )
        .with_account_id(Some(7001))
        .with_project_id(Some("project-live".to_owned()))
        .with_order_id(Some("order_123".to_owned()))
        .with_payment_order_id(Some("pay_123".to_owned())),
    )
    .await
    .unwrap();

    let second = redeem_coupon_code(
        &store,
        RedeemCouponCodeInput::new(
            999,
            "user",
            "9002",
            "hash:grant-code-001",
            "redeem:grant-code-001:9002",
            1_710_000_400,
        )
        .with_account_id(Some(7001))
        .with_project_id(Some("project-live".to_owned())),
    )
    .await
    .unwrap();

    assert_eq!(first.coupon_redemption_id, 701);
    assert_eq!(second.coupon_redemption_id, 701);
    assert_eq!(first.status, CouponRedemptionStatus::Fulfilled);
    assert_eq!(
        store.list_coupon_redemption_records().await.unwrap().len(),
        1
    );

    let stored_code = store
        .find_coupon_code_record_by_lookup_hash("hash:grant-code-001")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored_code.status, CouponCodeStatus::Redeemed);
}

#[tokio::test]
async fn reservation_blocks_conflicting_quote_and_redeem_fulfills_existing_pending_record() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let template = CouponTemplateRecord::new(
        150,
        1001,
        2002,
        "reserved-discount",
        "Reserved 15% off",
        CouponBenefitKind::PercentageDiscount,
        CouponDistributionKind::UniqueCode,
        1_710_000_000,
    )
    .with_status(CouponTemplateStatus::Active)
    .with_claim_required(true)
    .with_updated_at_ms(1_710_000_100);
    store
        .insert_coupon_template_record(&template)
        .await
        .unwrap();

    let rule = CouponBenefitRuleRecord::new(
        250,
        1001,
        2002,
        template.coupon_template_id,
        CouponBenefitKind::PercentageDiscount,
        1_710_000_010,
    )
    .with_target_order_kind(Some("recharge_pack".to_owned()))
    .with_percentage_off(Some(15.0))
    .with_updated_at_ms(1_710_000_101);
    store
        .insert_coupon_benefit_rule_record(&rule)
        .await
        .unwrap();

    let batch = CouponCodeBatchRecord::new(
        350,
        1001,
        2002,
        template.coupon_template_id,
        None,
        CouponCodeGenerationMode::BulkRandom,
        1_710_000_020,
    )
    .with_status(CouponCodeBatchStatus::Active)
    .with_updated_at_ms(1_710_000_102);
    store.insert_coupon_code_batch_record(&batch).await.unwrap();

    let code = CouponCodeRecord::new(
        450,
        1001,
        2002,
        batch.coupon_code_batch_id,
        template.coupon_template_id,
        None,
        "hash:reserve-code-001",
        CouponCodeKind::SingleUseUnique,
        1_710_000_030,
    )
    .with_status(CouponCodeStatus::Claimed)
    .with_claim_subject_type(Some("user".to_owned()))
    .with_claim_subject_id(Some("1001:2002:user_alpha".to_owned()))
    .with_claimed_at_ms(Some(1_710_000_031))
    .with_updated_at_ms(1_710_000_103);
    store.insert_coupon_code_record(&code).await.unwrap();

    let reserved = reserve_coupon_redemption(
        &store,
        ReserveCouponRedemptionInput::new(
            750,
            "user",
            "user_alpha",
            "hash:reserve-code-001",
            "reserve:order_alpha:RESERVE15",
            1_710_000_200,
        )
        .with_project_id(Some("project_alpha".to_owned()))
        .with_order_id(Some("order_alpha".to_owned())),
    )
    .await
    .unwrap();
    assert_eq!(reserved.status, CouponRedemptionStatus::Pending);

    let blocked = validate_coupon_for_quote(
        &store,
        ValidateCouponForQuoteInput::new("user", "user_alpha", "hash:reserve-code-001", 1_710_000_250)
            .with_target_order_kind(Some("recharge_pack".to_owned()))
            .with_reservation_idempotency_key(None),
    )
    .await
    .unwrap_err();
    assert!(
        blocked
            .to_string()
            .contains("already reserved for checkout")
    );

    let fulfilled = redeem_coupon_code(
        &store,
        RedeemCouponCodeInput::new(
            999,
            "user",
            "user_alpha",
            "hash:reserve-code-001",
            "reserve:order_alpha:RESERVE15",
            1_710_000_300,
        )
        .with_project_id(Some("project_alpha".to_owned()))
        .with_order_id(Some("order_alpha".to_owned()))
        .with_payment_order_id(Some("pay_alpha".to_owned())),
    )
    .await
    .unwrap();

    assert_eq!(fulfilled.coupon_redemption_id, 750);
    assert_eq!(fulfilled.status, CouponRedemptionStatus::Fulfilled);
    assert_eq!(fulfilled.payment_order_id.as_deref(), Some("pay_alpha"));

    let stored_code = store
        .find_coupon_code_record_by_lookup_hash("hash:reserve-code-001")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored_code.status, CouponCodeStatus::Redeemed);
}

#[tokio::test]
async fn releasing_pending_reservation_transitions_redemption_status() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let template = CouponTemplateRecord::new(
        151,
        1001,
        2002,
        "reserved-discount-release",
        "Reserved discount release",
        CouponBenefitKind::PercentageDiscount,
        CouponDistributionKind::UniqueCode,
        1_710_000_000,
    )
    .with_status(CouponTemplateStatus::Active)
    .with_claim_required(true)
    .with_updated_at_ms(1_710_000_100);
    store
        .insert_coupon_template_record(&template)
        .await
        .unwrap();

    let rule = CouponBenefitRuleRecord::new(
        251,
        1001,
        2002,
        template.coupon_template_id,
        CouponBenefitKind::PercentageDiscount,
        1_710_000_010,
    )
    .with_target_order_kind(Some("recharge_pack".to_owned()))
    .with_percentage_off(Some(15.0))
    .with_updated_at_ms(1_710_000_101);
    store
        .insert_coupon_benefit_rule_record(&rule)
        .await
        .unwrap();

    let batch = CouponCodeBatchRecord::new(
        351,
        1001,
        2002,
        template.coupon_template_id,
        None,
        CouponCodeGenerationMode::BulkRandom,
        1_710_000_020,
    )
    .with_status(CouponCodeBatchStatus::Active)
    .with_updated_at_ms(1_710_000_102);
    store.insert_coupon_code_batch_record(&batch).await.unwrap();

    let code = CouponCodeRecord::new(
        451,
        1001,
        2002,
        batch.coupon_code_batch_id,
        template.coupon_template_id,
        None,
        "hash:reserve-code-002",
        CouponCodeKind::SingleUseUnique,
        1_710_000_030,
    )
    .with_status(CouponCodeStatus::Claimed)
    .with_claim_subject_type(Some("user".to_owned()))
    .with_claim_subject_id(Some("1001:2002:user_alpha".to_owned()))
    .with_claimed_at_ms(Some(1_710_000_031))
    .with_updated_at_ms(1_710_000_103);
    store.insert_coupon_code_record(&code).await.unwrap();

    reserve_coupon_redemption(
        &store,
        ReserveCouponRedemptionInput::new(
            751,
            "user",
            "user_alpha",
            "hash:reserve-code-002",
            "reserve:order_beta:RESERVE15",
            1_710_000_200,
        )
        .with_project_id(Some("project_alpha".to_owned()))
        .with_order_id(Some("order_beta".to_owned())),
    )
    .await
    .unwrap();

    let released = release_coupon_redemption_reservation(
        &store,
        ReleaseCouponRedemptionReservationInput::new(
            "reserve:order_beta:RESERVE15",
            CouponRedemptionStatus::Voided,
            1_710_000_250,
        ),
    )
    .await
    .unwrap()
    .expect("released reservation");

    assert_eq!(released.status, CouponRedemptionStatus::Voided);
    let stored_code = store
        .find_coupon_code_record_by_lookup_hash("hash:reserve-code-002")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored_code.status, CouponCodeStatus::Claimed);
}
