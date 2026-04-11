use sdkwork_api_app_marketing::{
    confirm_coupon_redemption, reserve_coupon_redemption, rollback_coupon_redemption,
    validate_coupon_stack,
};
use sdkwork_api_domain_marketing::{
    CampaignBudgetRecord, CampaignBudgetStatus, CouponCodeRecord, CouponCodeStatus,
    CouponRedemptionStatus, CouponReservationStatus, CouponRollbackStatus, CouponRollbackType,
    CouponTemplateRecord, CouponTemplateStatus, MarketingBenefitKind, MarketingCampaignRecord,
    MarketingCampaignStatus, MarketingSubjectScope,
};

#[test]
fn validate_coupon_stack_rejects_inactive_campaigns() {
    let template = CouponTemplateRecord::new(
        "tpl_launch_20",
        "launch-20",
        MarketingBenefitKind::PercentageOff,
    )
    .with_status(CouponTemplateStatus::Active);
    let campaign = MarketingCampaignRecord::new("camp_launch", "tpl_launch_20")
        .with_status(MarketingCampaignStatus::Paused)
        .with_start_at_ms(Some(1_000))
        .with_end_at_ms(Some(5_000));
    let budget = CampaignBudgetRecord::new("budget_launch", "camp_launch")
        .with_status(CampaignBudgetStatus::Active)
        .with_total_budget_minor(10_000);
    let code = CouponCodeRecord::new("code_launch_20", "tpl_launch_20", "LAUNCH20")
        .with_status(CouponCodeStatus::Available)
        .with_expires_at_ms(Some(5_000));

    let decision = validate_coupon_stack(&template, &campaign, &budget, &code, 2_000, 3_000, 500);

    assert!(!decision.eligible);
    assert_eq!(
        decision.rejection_reason.as_deref(),
        Some("campaign_not_effective")
    );
}

#[test]
fn reserve_coupon_redemption_transitions_code_and_creates_reservation() {
    let code = CouponCodeRecord::new("code_launch_20", "tpl_launch_20", "LAUNCH20")
        .with_status(CouponCodeStatus::Available)
        .with_expires_at_ms(Some(5_000));

    let (reserved_code, reservation) = reserve_coupon_redemption(
        &code,
        "res_launch_20",
        MarketingSubjectScope::Project,
        "project_demo",
        700,
        2_000,
        1_000,
    )
    .expect("reservation should succeed");

    assert_eq!(reserved_code.status, CouponCodeStatus::Reserved);
    assert_eq!(
        reservation.reservation_status,
        CouponReservationStatus::Reserved
    );
    assert_eq!(reservation.subject_id, "project_demo");
    assert_eq!(reservation.budget_reserved_minor, 700);
    assert!(reservation.is_active_at(2_500));
}

#[test]
fn confirm_and_rollback_coupon_redemption_build_finance_safe_records() {
    let code = CouponCodeRecord::new("code_launch_20", "tpl_launch_20", "LAUNCH20")
        .with_status(CouponCodeStatus::Available);
    let (_, reservation) = reserve_coupon_redemption(
        &code,
        "res_launch_20",
        MarketingSubjectScope::Project,
        "project_demo",
        700,
        2_000,
        1_000,
    )
    .expect("reservation should succeed");

    let (confirmed_reservation, redemption) = confirm_coupon_redemption(
        &reservation,
        "red_launch_20",
        "code_launch_20",
        "tpl_launch_20",
        700,
        Some("order_demo".to_owned()),
        Some("payevt_demo".to_owned()),
        2_300,
    )
    .expect("confirmation should succeed");

    assert_eq!(
        confirmed_reservation.reservation_status,
        CouponReservationStatus::Confirmed
    );
    assert_eq!(
        redemption.redemption_status,
        CouponRedemptionStatus::Redeemed
    );
    assert_eq!(redemption.order_id.as_deref(), Some("order_demo"));

    let (rolled_back_redemption, rollback) = rollback_coupon_redemption(
        &redemption,
        "rollback_launch_20",
        CouponRollbackType::Refund,
        700,
        1,
        2_600,
    )
    .expect("rollback should succeed");

    assert_eq!(
        rolled_back_redemption.redemption_status,
        CouponRedemptionStatus::RolledBack
    );
    assert_eq!(rollback.rollback_status, CouponRollbackStatus::Completed);
    assert_eq!(rollback.restored_budget_minor, 700);
    assert_eq!(rollback.restored_inventory_count, 1);
}

#[test]
fn confirm_coupon_redemption_rejects_subsidy_amount_above_reserved_budget() {
    let code = CouponCodeRecord::new("code_launch_20", "tpl_launch_20", "LAUNCH20")
        .with_status(CouponCodeStatus::Available);
    let (_, reservation) = reserve_coupon_redemption(
        &code,
        "res_launch_20",
        MarketingSubjectScope::Project,
        "project_demo",
        700,
        2_000,
        1_000,
    )
    .expect("reservation should succeed");

    let error = confirm_coupon_redemption(
        &reservation,
        "red_launch_20",
        "code_launch_20",
        "tpl_launch_20",
        701,
        Some("order_demo".to_owned()),
        Some("payevt_demo".to_owned()),
        2_300,
    )
    .expect_err("confirmation should reject subsidy above reserved budget");

    assert_eq!(
        error.to_string(),
        "subsidy amount exceeds reserved coupon budget"
    );
}

#[test]
fn rollback_coupon_redemption_rejects_restored_budget_above_redeemed_subsidy() {
    let code = CouponCodeRecord::new("code_launch_20", "tpl_launch_20", "LAUNCH20")
        .with_status(CouponCodeStatus::Available);
    let (_, reservation) = reserve_coupon_redemption(
        &code,
        "res_launch_20",
        MarketingSubjectScope::Project,
        "project_demo",
        700,
        2_000,
        1_000,
    )
    .expect("reservation should succeed");
    let (_, redemption) = confirm_coupon_redemption(
        &reservation,
        "red_launch_20",
        "code_launch_20",
        "tpl_launch_20",
        700,
        Some("order_demo".to_owned()),
        Some("payevt_demo".to_owned()),
        2_300,
    )
    .expect("confirmation should succeed");

    let error = rollback_coupon_redemption(
        &redemption,
        "rollback_launch_20",
        CouponRollbackType::Refund,
        701,
        1,
        2_600,
    )
    .expect_err("rollback should reject restored budget above redeemed subsidy");

    assert_eq!(
        error.to_string(),
        "restored budget exceeds redeemed coupon subsidy"
    );
}
