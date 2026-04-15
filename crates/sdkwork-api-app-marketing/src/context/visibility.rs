use super::types::MarketingCouponContext;
use sdkwork_api_domain_marketing::{
    CouponCodeRecord, CouponCodeStatus, CouponDistributionKind, CouponTemplateRecord,
    CouponTemplateStatus,
};

pub fn coupon_context_is_catalog_visible(context: &MarketingCouponContext, now_ms: u64) -> bool {
    context.template.status == CouponTemplateStatus::Active
        && context.campaign.is_effective_at(now_ms)
        && context.budget.available_budget_minor() > 0
        && coupon_code_is_available_for_template(&context.template, &context.code, now_ms)
}

pub fn marketing_coupon_context_remaining_inventory(
    context: &MarketingCouponContext,
    now_ms: u64,
) -> u64 {
    match context.template.distribution_kind {
        CouponDistributionKind::SharedCode => context.budget.available_budget_minor(),
        CouponDistributionKind::UniqueCode | CouponDistributionKind::AutoClaim => {
            if coupon_code_is_available_for_template(&context.template, &context.code, now_ms) {
                1
            } else {
                0
            }
        }
    }
}

fn coupon_code_is_available_for_template(
    template: &CouponTemplateRecord,
    code: &CouponCodeRecord,
    now_ms: u64,
) -> bool {
    match template.distribution_kind {
        CouponDistributionKind::SharedCode => {
            !matches!(
                code.status,
                CouponCodeStatus::Disabled | CouponCodeStatus::Expired
            ) && code.expires_at_ms.map_or(true, |value| now_ms <= value)
        }
        CouponDistributionKind::UniqueCode | CouponDistributionKind::AutoClaim => {
            code.is_redeemable_at(now_ms)
        }
    }
}
