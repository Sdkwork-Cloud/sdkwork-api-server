use super::super::types::CouponTemplateDetail;
use super::decision::build_coupon_template_actionability;
use sdkwork_api_domain_marketing::CouponTemplateRecord;

pub(in crate::governance::template) fn build_coupon_template_detail(
    coupon_template: CouponTemplateRecord,
    now_ms: u64,
) -> CouponTemplateDetail {
    let actionability = build_coupon_template_actionability(&coupon_template, now_ms);
    CouponTemplateDetail {
        coupon_template,
        actionability,
    }
}
