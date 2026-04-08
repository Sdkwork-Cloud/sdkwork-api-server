use super::*;

pub(crate) fn normalize_coupon_code_value(code_value: &str) -> String {
    code_value.trim().to_ascii_uppercase()
}

pub(crate) fn coupon_template_status_as_str(status: CouponTemplateStatus) -> &'static str {
    match status {
        CouponTemplateStatus::Draft => "draft",
        CouponTemplateStatus::Active => "active",
        CouponTemplateStatus::Archived => "archived",
    }
}

pub(crate) fn coupon_distribution_kind_as_str(kind: CouponDistributionKind) -> &'static str {
    match kind {
        CouponDistributionKind::SharedCode => "shared_code",
        CouponDistributionKind::UniqueCode => "unique_code",
        CouponDistributionKind::AutoClaim => "auto_claim",
    }
}

pub(crate) fn marketing_campaign_status_as_str(status: MarketingCampaignStatus) -> &'static str {
    match status {
        MarketingCampaignStatus::Draft => "draft",
        MarketingCampaignStatus::Scheduled => "scheduled",
        MarketingCampaignStatus::Active => "active",
        MarketingCampaignStatus::Paused => "paused",
        MarketingCampaignStatus::Ended => "ended",
        MarketingCampaignStatus::Archived => "archived",
    }
}

pub(crate) fn campaign_budget_status_as_str(status: CampaignBudgetStatus) -> &'static str {
    match status {
        CampaignBudgetStatus::Draft => "draft",
        CampaignBudgetStatus::Active => "active",
        CampaignBudgetStatus::Exhausted => "exhausted",
        CampaignBudgetStatus::Closed => "closed",
    }
}

pub(crate) fn coupon_code_status_as_str(status: CouponCodeStatus) -> &'static str {
    match status {
        CouponCodeStatus::Available => "available",
        CouponCodeStatus::Reserved => "reserved",
        CouponCodeStatus::Redeemed => "redeemed",
        CouponCodeStatus::Expired => "expired",
        CouponCodeStatus::Disabled => "disabled",
    }
}

pub(crate) fn marketing_subject_scope_as_str(scope: MarketingSubjectScope) -> &'static str {
    match scope {
        MarketingSubjectScope::User => "user",
        MarketingSubjectScope::Project => "project",
        MarketingSubjectScope::Workspace => "workspace",
        MarketingSubjectScope::Account => "account",
    }
}

pub(crate) fn coupon_reservation_status_as_str(status: CouponReservationStatus) -> &'static str {
    match status {
        CouponReservationStatus::Reserved => "reserved",
        CouponReservationStatus::Released => "released",
        CouponReservationStatus::Confirmed => "confirmed",
        CouponReservationStatus::Expired => "expired",
    }
}

pub(crate) fn coupon_redemption_status_as_str(status: CouponRedemptionStatus) -> &'static str {
    match status {
        CouponRedemptionStatus::Pending => "pending",
        CouponRedemptionStatus::Redeemed => "redeemed",
        CouponRedemptionStatus::PartiallyRolledBack => "partially_rolled_back",
        CouponRedemptionStatus::RolledBack => "rolled_back",
        CouponRedemptionStatus::Failed => "failed",
    }
}

pub(crate) fn coupon_rollback_type_as_str(rollback_type: CouponRollbackType) -> &'static str {
    match rollback_type {
        CouponRollbackType::Cancel => "cancel",
        CouponRollbackType::Refund => "refund",
        CouponRollbackType::PartialRefund => "partial_refund",
        CouponRollbackType::Manual => "manual",
    }
}

pub(crate) fn coupon_rollback_status_as_str(status: CouponRollbackStatus) -> &'static str {
    match status {
        CouponRollbackStatus::Pending => "pending",
        CouponRollbackStatus::Completed => "completed",
        CouponRollbackStatus::Failed => "failed",
    }
}

pub(crate) fn marketing_outbox_event_status_as_str(
    status: MarketingOutboxEventStatus,
) -> &'static str {
    match status {
        MarketingOutboxEventStatus::Pending => "pending",
        MarketingOutboxEventStatus::Delivered => "delivered",
        MarketingOutboxEventStatus::Failed => "failed",
    }
}
