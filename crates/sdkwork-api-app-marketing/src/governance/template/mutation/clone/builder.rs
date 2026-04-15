use super::super::super::super::normalize_optional_display_name;
use super::super::super::lookup::{coupon_template_root_id, next_coupon_template_revision};
use super::super::super::types::CloneCouponTemplateRevisionInput;
use crate::MarketingGovernanceError;
use sdkwork_api_domain_marketing::{
    CouponTemplateApprovalState, CouponTemplateRecord, CouponTemplateStatus,
};
use sdkwork_api_storage_core::AdminStore;

pub(super) async fn build_cloned_coupon_template(
    store: &dyn AdminStore,
    source_coupon_template: &CouponTemplateRecord,
    input: CloneCouponTemplateRevisionInput,
    now_ms: u64,
) -> Result<CouponTemplateRecord, MarketingGovernanceError> {
    let cloned_display_name = input
        .display_name
        .and_then(normalize_optional_display_name)
        .unwrap_or_else(|| source_coupon_template.display_name.clone());
    let root_coupon_template_id = coupon_template_root_id(source_coupon_template);
    let cloned_coupon_template = source_coupon_template
        .clone()
        .with_status(CouponTemplateStatus::Draft)
        .with_approval_state(CouponTemplateApprovalState::Draft)
        .with_revision(next_coupon_template_revision(store, source_coupon_template).await?)
        .with_root_coupon_template_id(Some(root_coupon_template_id))
        .with_parent_coupon_template_id(Some(source_coupon_template.coupon_template_id.clone()))
        .with_activation_at_ms(None)
        .with_created_at_ms(now_ms)
        .with_updated_at_ms(now_ms);

    Ok(CouponTemplateRecord {
        coupon_template_id: input.coupon_template_id,
        template_key: input.template_key,
        display_name: cloned_display_name,
        ..cloned_coupon_template
    })
}
