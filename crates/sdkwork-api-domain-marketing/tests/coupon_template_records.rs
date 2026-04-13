use sdkwork_api_domain_marketing::{
    AttributionSourceKind, CouponBenefitKind, CouponBenefitRuleRecord, CouponClaimRecord,
    CouponClaimStatus, CouponCodeBatchRecord, CouponCodeBatchStatus, CouponCodeGenerationMode,
    CouponCodeKind, CouponCodeRecord, CouponCodeStatus, CouponDistributionKind,
    CouponRedemptionRecord, CouponRedemptionStatus, CouponStackingPolicy, CouponTemplateRecord,
    CouponTemplateStatus, MarketingAttributionTouchRecord, MarketingCampaignKind,
    MarketingCampaignRecord, MarketingCampaignStatus, ReferralInviteRecord, ReferralInviteStatus,
    ReferralProgramRecord, ReferralProgramStatus,
};

#[test]
fn coupon_template_and_benefit_rule_keep_marketing_semantics() {
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
    .with_updated_at_ms(1_710_000_123);

    let benefit_rule = CouponBenefitRuleRecord::new(
        201,
        1001,
        2002,
        template.coupon_template_id,
        CouponBenefitKind::PercentageDiscount,
        1_710_000_000,
    )
    .with_target_order_kind(Some("recharge_pack".to_owned()))
    .with_target_product_id(Some("pack-100k".to_owned()))
    .with_percentage_off(Some(20.0))
    .with_maximum_subsidy_amount(Some(50.0))
    .with_currency_code(Some("USD".to_owned()))
    .with_updated_at_ms(1_710_000_456);

    assert_eq!(template.template_code, "spring-launch");
    assert_eq!(template.status, CouponTemplateStatus::Active);
    assert_eq!(
        template.stacking_policy,
        CouponStackingPolicy::ExclusiveWithinGroup
    );
    assert_eq!(template.max_redemptions_per_subject, Some(1));
    assert_eq!(benefit_rule.coupon_template_id, template.coupon_template_id);
    assert_eq!(
        benefit_rule.benefit_kind,
        CouponBenefitKind::PercentageDiscount
    );
    assert_eq!(benefit_rule.target_product_id.as_deref(), Some("pack-100k"));
    assert_eq!(benefit_rule.percentage_off, Some(20.0));
}

#[test]
fn campaign_batch_and_code_support_one_template_many_codes() {
    let campaign = MarketingCampaignRecord::new(
        301,
        1001,
        2002,
        "launch-2026-q2",
        "Q2 Launch Push",
        MarketingCampaignKind::Launch,
        1_710_000_000,
    )
    .with_status(MarketingCampaignStatus::Active)
    .with_channel_source(Some("partner-marketplace".to_owned()))
    .with_budget_amount(Some(20_000.0))
    .with_currency_code(Some("USD".to_owned()))
    .with_subsidy_cap_amount(Some(15_000.0))
    .with_owner_user_id(Some(9001))
    .with_ends_at_ms(Some(1_710_999_999));

    let batch = CouponCodeBatchRecord::new(
        401,
        1001,
        2002,
        101,
        Some(campaign.marketing_campaign_id),
        CouponCodeGenerationMode::BulkRandom,
        1_710_000_000,
    )
    .with_status(CouponCodeBatchStatus::Active)
    .with_batch_kind(Some("partner-distribution".to_owned()))
    .with_code_prefix(Some("SPR".to_owned()))
    .with_issued_count(5000)
    .with_claimed_count(120)
    .with_redeemed_count(75)
    .with_voided_count(3)
    .with_expires_at_ms(Some(1_710_999_999));

    let code = CouponCodeRecord::new(
        501,
        1001,
        2002,
        batch.coupon_code_batch_id,
        101,
        Some(campaign.marketing_campaign_id),
        "hash:spring-code-001",
        CouponCodeKind::SingleUseUnique,
        1_710_000_000,
    )
    .with_status(CouponCodeStatus::Issued)
    .with_display_code_prefix(Some("SPR".to_owned()))
    .with_display_code_suffix(Some("001".to_owned()))
    .with_claim_subject_type(Some("user".to_owned()))
    .with_claim_subject_id(Some("1001:2002:9002".to_owned()))
    .with_claimed_at_ms(Some(1_710_000_123))
    .with_expires_at_ms(Some(1_710_999_999));

    assert_eq!(campaign.status, MarketingCampaignStatus::Active);
    assert_eq!(batch.coupon_template_id, 101);
    assert_eq!(batch.generation_mode, CouponCodeGenerationMode::BulkRandom);
    assert_eq!(batch.issued_count, 5000);
    assert_eq!(code.coupon_code_batch_id, batch.coupon_code_batch_id);
    assert_eq!(code.code_kind, CouponCodeKind::SingleUseUnique);
    assert_eq!(code.display_code_suffix.as_deref(), Some("001"));
    assert_eq!(code.claim_subject_id.as_deref(), Some("1001:2002:9002"));
}

#[test]
fn claim_redemption_referral_and_attribution_preserve_lineage() {
    let claim = CouponClaimRecord::new(601, 1001, 2002, 501, 101, "user", "9002", 1_710_000_000)
        .with_status(CouponClaimStatus::Claimed)
        .with_account_id(Some(7001))
        .with_project_id(Some("project-live".to_owned()))
        .with_expires_at_ms(Some(1_710_999_999));

    let redemption = CouponRedemptionRecord::new(
        701,
        1001,
        2002,
        501,
        101,
        Some(301),
        "user",
        "9002",
        1_710_000_111,
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
    .with_idempotency_key(Some("redeem:501:9002".to_owned()));

    let referral_program = ReferralProgramRecord::new(
        801,
        1001,
        2002,
        "invite-growth",
        "Invite Growth",
        1_710_000_000,
    )
    .with_status(ReferralProgramStatus::Active)
    .with_invite_reward_template_id(Some(111))
    .with_referee_reward_template_id(Some(112))
    .with_ends_at_ms(Some(1_710_999_999));

    let referral_invite = ReferralInviteRecord::new(
        901,
        1001,
        2002,
        referral_program.referral_program_id,
        9002,
        1_710_000_000,
    )
    .with_status(ReferralInviteStatus::Rewarded)
    .with_coupon_code_id(Some(501))
    .with_source_code(Some("INVITE-A1B2".to_owned()))
    .with_referred_user_id(Some(9003))
    .with_accepted_at_ms(Some(1_710_000_050))
    .with_rewarded_at_ms(Some(1_710_000_200));

    let touch = MarketingAttributionTouchRecord::new(
        1001,
        1001,
        2002,
        AttributionSourceKind::Referral,
        1_710_000_000,
    )
    .with_source_code(Some("INVITE-A1B2".to_owned()))
    .with_utm_source(Some("partner".to_owned()))
    .with_utm_campaign(Some("launch-2026-q2".to_owned()))
    .with_utm_medium(Some("invite".to_owned()))
    .with_partner_id(Some("partner-01".to_owned()))
    .with_referrer_user_id(Some(9002))
    .with_invite_code_id(Some(501))
    .with_conversion_subject_id(Some("workspace:9003".to_owned()))
    .with_converted_at_ms(Some(1_710_000_250));

    assert_eq!(claim.status, CouponClaimStatus::Claimed);
    assert_eq!(claim.subject_type, "user");
    assert_eq!(claim.subject_id, "9002");
    assert_eq!(claim.project_id.as_deref(), Some("project-live"));
    assert_eq!(redemption.status, CouponRedemptionStatus::Fulfilled);
    assert_eq!(redemption.subject_type, "user");
    assert_eq!(redemption.subject_id, "9002");
    assert_eq!(
        redemption.idempotency_key.as_deref(),
        Some("redeem:501:9002")
    );
    assert_eq!(referral_program.status, ReferralProgramStatus::Active);
    assert_eq!(referral_invite.status, ReferralInviteStatus::Rewarded);
    assert_eq!(touch.source_kind, AttributionSourceKind::Referral);
    assert_eq!(touch.partner_id.as_deref(), Some("partner-01"));
}
