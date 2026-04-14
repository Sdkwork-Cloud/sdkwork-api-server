use super::super::super::actionability::build_coupon_template_actionability;
use super::super::super::audit::{
    build_coupon_template_lifecycle_audit_record, persist_coupon_template_lifecycle_audit_record,
};
use super::super::super::types::CloneCouponTemplateRevisionInput;
use crate::MarketingGovernanceError;
use sdkwork_api_domain_marketing::{
    CouponTemplateLifecycleAction, CouponTemplateLifecycleAuditOutcome, CouponTemplateRecord,
};
use sdkwork_api_storage_core::AdminStore;

pub(super) async fn ensure_coupon_template_clone_allowed(
    store: &dyn AdminStore,
    source_coupon_template: &CouponTemplateRecord,
    input: &CloneCouponTemplateRevisionInput,
    operator_id: &str,
    request_id: &str,
    reason: &str,
    now_ms: u64,
) -> Result<(), MarketingGovernanceError> {
    let actionability = build_coupon_template_actionability(source_coupon_template, now_ms);
    if !actionability.clone.allowed {
        let audit = build_coupon_template_lifecycle_audit_record(
            source_coupon_template,
            None,
            None,
            CouponTemplateLifecycleAction::Clone,
            CouponTemplateLifecycleAuditOutcome::Rejected,
            operator_id,
            request_id,
            reason,
            now_ms,
            actionability.clone.reasons.clone(),
        );
        persist_coupon_template_lifecycle_audit_record(store, &audit).await?;
        return Err(MarketingGovernanceError::invalid_input(
            actionability
                .clone
                .reasons
                .first()
                .cloned()
                .unwrap_or_else(|| "coupon template clone is not allowed".to_owned()),
        ));
    }

    if input.coupon_template_id == source_coupon_template.coupon_template_id {
        return Err(MarketingGovernanceError::invalid_input(
            "cloned coupon template must use a new coupon_template_id",
        ));
    }
    if store
        .find_coupon_template_record(&input.coupon_template_id)
        .await
        .map_err(MarketingGovernanceError::storage)?
        .is_some()
    {
        return Err(MarketingGovernanceError::conflict(format!(
            "coupon template {} already exists",
            input.coupon_template_id
        )));
    }
    if store
        .find_coupon_template_record_by_template_key(&input.template_key)
        .await
        .map_err(MarketingGovernanceError::storage)?
        .is_some()
    {
        return Err(MarketingGovernanceError::conflict(format!(
            "coupon template key {} already exists",
            input.template_key
        )));
    }

    Ok(())
}
