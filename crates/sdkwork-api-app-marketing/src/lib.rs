use sdkwork_api_domain_marketing::{
    CampaignBudgetRecord, CouponCodeRecord, CouponCodeStatus, CouponRedemptionRecord,
    CouponRedemptionStatus, CouponReservationRecord, CouponReservationStatus,
    CouponRollbackRecord, CouponRollbackStatus, CouponRollbackType, CouponTemplateRecord,
    CouponTemplateStatus, MarketingCampaignRecord,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CouponValidationDecision {
    pub eligible: bool,
    pub rejection_reason: Option<String>,
    pub reservable_budget_minor: u64,
}

impl CouponValidationDecision {
    pub fn eligible(reservable_budget_minor: u64) -> Self {
        Self {
            eligible: true,
            rejection_reason: None,
            reservable_budget_minor,
        }
    }

    pub fn rejected(reason: &'static str) -> Self {
        Self {
            eligible: false,
            rejection_reason: Some(reason.to_owned()),
            reservable_budget_minor: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarketingServiceError {
    InvalidState(&'static str),
}

impl std::fmt::Display for MarketingServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidState(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for MarketingServiceError {}

pub fn validate_coupon_stack(
    template: &CouponTemplateRecord,
    campaign: &MarketingCampaignRecord,
    budget: &CampaignBudgetRecord,
    code: &CouponCodeRecord,
    now_ms: u64,
    order_amount_minor: u64,
    reserve_amount_minor: u64,
) -> CouponValidationDecision {
    if template.status != CouponTemplateStatus::Active {
        return CouponValidationDecision::rejected("template_not_active");
    }
    if !campaign.is_effective_at(now_ms) {
        return CouponValidationDecision::rejected("campaign_not_effective");
    }
    if let Some(min_order_amount_minor) = template.restriction.min_order_amount_minor {
        if order_amount_minor < min_order_amount_minor {
            return CouponValidationDecision::rejected("order_amount_below_minimum");
        }
    }
    if !budget.can_reserve(reserve_amount_minor) {
        return CouponValidationDecision::rejected("budget_unavailable");
    }
    if !code.is_redeemable_at(now_ms) {
        return CouponValidationDecision::rejected("coupon_code_unavailable");
    }

    CouponValidationDecision::eligible(reserve_amount_minor)
}

pub fn reserve_coupon_redemption(
    code: &CouponCodeRecord,
    coupon_reservation_id: impl Into<String>,
    subject_scope: sdkwork_api_domain_marketing::MarketingSubjectScope,
    subject_id: impl Into<String>,
    budget_reserved_minor: u64,
    now_ms: u64,
    ttl_ms: u64,
) -> Result<(CouponCodeRecord, CouponReservationRecord), MarketingServiceError> {
    if ttl_ms == 0 {
        return Err(MarketingServiceError::InvalidState(
            "reservation ttl must be positive",
        ));
    }
    if !code.is_redeemable_at(now_ms) {
        return Err(MarketingServiceError::InvalidState(
            "coupon code is not redeemable",
        ));
    }

    let reserved_code = code
        .clone()
        .with_status(CouponCodeStatus::Reserved)
        .with_updated_at_ms(now_ms);
    let reservation = CouponReservationRecord::new(
        coupon_reservation_id,
        reserved_code.coupon_code_id.clone(),
        subject_scope,
        subject_id,
        now_ms.saturating_add(ttl_ms),
    )
    .with_budget_reserved_minor(budget_reserved_minor)
    .with_created_at_ms(now_ms)
    .with_updated_at_ms(now_ms);

    Ok((reserved_code, reservation))
}

pub fn confirm_coupon_redemption(
    reservation: &CouponReservationRecord,
    coupon_redemption_id: impl Into<String>,
    coupon_code_id: impl Into<String>,
    coupon_template_id: impl Into<String>,
    subsidy_amount_minor: u64,
    order_id: Option<String>,
    payment_event_id: Option<String>,
    now_ms: u64,
) -> Result<(CouponReservationRecord, CouponRedemptionRecord), MarketingServiceError> {
    if reservation.reservation_status != CouponReservationStatus::Reserved {
        return Err(MarketingServiceError::InvalidState(
            "reservation is not in reserved state",
        ));
    }
    if !reservation.is_active_at(now_ms) {
        return Err(MarketingServiceError::InvalidState(
            "reservation is no longer active",
        ));
    }
    if subsidy_amount_minor > reservation.budget_reserved_minor {
        return Err(MarketingServiceError::InvalidState(
            "subsidy amount exceeds reserved coupon budget",
        ));
    }

    let confirmed_reservation = reservation
        .clone()
        .with_status(CouponReservationStatus::Confirmed)
        .with_updated_at_ms(now_ms);
    let redemption = CouponRedemptionRecord::new(
        coupon_redemption_id,
        confirmed_reservation.coupon_reservation_id.clone(),
        coupon_code_id,
        coupon_template_id,
        now_ms,
    )
    .with_status(CouponRedemptionStatus::Redeemed)
    .with_subsidy_amount_minor(subsidy_amount_minor)
    .with_order_id(order_id)
    .with_payment_event_id(payment_event_id)
    .with_updated_at_ms(now_ms);

    Ok((confirmed_reservation, redemption))
}

pub fn rollback_coupon_redemption(
    redemption: &CouponRedemptionRecord,
    coupon_rollback_id: impl Into<String>,
    rollback_type: CouponRollbackType,
    restored_budget_minor: u64,
    restored_inventory_count: u64,
    now_ms: u64,
) -> Result<(CouponRedemptionRecord, CouponRollbackRecord), MarketingServiceError> {
    if redemption.redemption_status != CouponRedemptionStatus::Redeemed {
        return Err(MarketingServiceError::InvalidState(
            "redemption is not in redeemed state",
        ));
    }
    if restored_budget_minor > redemption.subsidy_amount_minor {
        return Err(MarketingServiceError::InvalidState(
            "restored budget exceeds redeemed coupon subsidy",
        ));
    }

    let next_redemption_status = match rollback_type {
        CouponRollbackType::PartialRefund => CouponRedemptionStatus::PartiallyRolledBack,
        CouponRollbackType::Cancel | CouponRollbackType::Refund | CouponRollbackType::Manual => {
            CouponRedemptionStatus::RolledBack
        }
    };

    let rolled_back_redemption = redemption
        .clone()
        .with_status(next_redemption_status)
        .with_updated_at_ms(now_ms);
    let rollback = CouponRollbackRecord::new(
        coupon_rollback_id,
        rolled_back_redemption.coupon_redemption_id.clone(),
        rollback_type,
        now_ms,
    )
    .with_status(CouponRollbackStatus::Completed)
    .with_restored_budget_minor(restored_budget_minor)
    .with_restored_inventory_count(restored_inventory_count)
    .with_updated_at_ms(now_ms);

    Ok((rolled_back_redemption, rollback))
}
