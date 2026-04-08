use super::*;

#[derive(Debug, Deserialize, Default)]
pub(crate) struct PortalMarketingRedemptionsQuery {
    #[serde(default)]
    pub(crate) status: Option<CouponRedemptionStatus>,
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct PortalMarketingCodesQuery {
    #[serde(default)]
    pub(crate) status: Option<CouponCodeStatus>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PortalCouponValidationRequest {
    pub(crate) coupon_code: String,
    pub(crate) subject_scope: MarketingSubjectScope,
    pub(crate) target_kind: String,
    pub(crate) order_amount_minor: u64,
    pub(crate) reserve_amount_minor: u64,
}

#[derive(Debug, Serialize)]
pub(crate) struct PortalCouponValidationDecisionResponse {
    pub(crate) eligible: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) rejection_reason: Option<String>,
    pub(crate) reservable_budget_minor: u64,
}

#[derive(Debug, Serialize)]
pub(crate) struct PortalCouponValidationResponse {
    pub(crate) decision: PortalCouponValidationDecisionResponse,
    pub(crate) template: CouponTemplateRecord,
    pub(crate) campaign: MarketingCampaignRecord,
    pub(crate) budget: CampaignBudgetRecord,
    pub(crate) code: CouponCodeRecord,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PortalCouponReservationRequest {
    pub(crate) coupon_code: String,
    pub(crate) subject_scope: MarketingSubjectScope,
    pub(crate) target_kind: String,
    pub(crate) reserve_amount_minor: u64,
    pub(crate) ttl_ms: u64,
    #[serde(default)]
    pub(crate) idempotency_key: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct PortalCouponReservationResponse {
    pub(crate) reservation: CouponReservationRecord,
    pub(crate) template: CouponTemplateRecord,
    pub(crate) campaign: MarketingCampaignRecord,
    pub(crate) budget: CampaignBudgetRecord,
    pub(crate) code: CouponCodeRecord,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PortalCouponRedemptionConfirmRequest {
    pub(crate) coupon_reservation_id: String,
    pub(crate) subsidy_amount_minor: u64,
    #[serde(default)]
    pub(crate) order_id: Option<String>,
    #[serde(default)]
    pub(crate) payment_event_id: Option<String>,
    #[serde(default)]
    pub(crate) idempotency_key: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct PortalCouponRedemptionConfirmResponse {
    pub(crate) reservation: CouponReservationRecord,
    pub(crate) redemption: CouponRedemptionRecord,
    pub(crate) budget: CampaignBudgetRecord,
    pub(crate) code: CouponCodeRecord,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PortalCouponRedemptionRollbackRequest {
    pub(crate) coupon_redemption_id: String,
    pub(crate) rollback_type: CouponRollbackType,
    pub(crate) restored_budget_minor: u64,
    pub(crate) restored_inventory_count: u64,
    #[serde(default)]
    pub(crate) idempotency_key: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct PortalCouponRedemptionRollbackResponse {
    pub(crate) redemption: CouponRedemptionRecord,
    pub(crate) rollback: CouponRollbackRecord,
    pub(crate) budget: CampaignBudgetRecord,
    pub(crate) code: CouponCodeRecord,
}

#[derive(Debug, Serialize, Default)]
pub(crate) struct PortalMarketingRedemptionSummary {
    pub(crate) total_count: usize,
    pub(crate) redeemed_count: usize,
    pub(crate) partially_rolled_back_count: usize,
    pub(crate) rolled_back_count: usize,
    pub(crate) failed_count: usize,
}

#[derive(Debug, Serialize, Default)]
pub(crate) struct PortalMarketingCodeSummary {
    pub(crate) total_count: usize,
    pub(crate) available_count: usize,
    pub(crate) reserved_count: usize,
    pub(crate) redeemed_count: usize,
    pub(crate) disabled_count: usize,
    pub(crate) expired_count: usize,
}

#[derive(Debug, Serialize)]
pub(crate) struct PortalMarketingRedemptionsResponse {
    pub(crate) summary: PortalMarketingRedemptionSummary,
    pub(crate) items: Vec<CouponRedemptionRecord>,
}

#[derive(Debug, Serialize)]
pub(crate) struct PortalMarketingCodeItem {
    pub(crate) code: CouponCodeRecord,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) latest_reservation: Option<CouponReservationRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) latest_redemption: Option<CouponRedemptionRecord>,
}

#[derive(Debug, Serialize)]
pub(crate) struct PortalMarketingCodesResponse {
    pub(crate) summary: PortalMarketingCodeSummary,
    pub(crate) items: Vec<PortalMarketingCodeItem>,
}

#[derive(Debug, Serialize)]
pub(crate) struct PortalMarketingRewardHistoryItem {
    pub(crate) redemption: CouponRedemptionRecord,
    pub(crate) code: CouponCodeRecord,
    #[serde(default)]
    pub(crate) rollbacks: Vec<CouponRollbackRecord>,
}
