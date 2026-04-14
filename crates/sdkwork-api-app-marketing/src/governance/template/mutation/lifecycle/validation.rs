use super::super::super::audit::{
    build_coupon_template_lifecycle_audit_record, persist_coupon_template_lifecycle_audit_record,
};
use super::super::super::types::CouponTemplateActionDecision;
use crate::MarketingGovernanceError;
use sdkwork_api_domain_marketing::{
    CouponTemplateLifecycleAction, CouponTemplateLifecycleAuditOutcome, CouponTemplateRecord,
};
use sdkwork_api_storage_core::AdminStore;

pub(super) async fn ensure_coupon_template_lifecycle_allowed(
    store: &dyn AdminStore,
    coupon_template: &CouponTemplateRecord,
    action: CouponTemplateLifecycleAction,
    decision: &CouponTemplateActionDecision,
    operator_id: &str,
    request_id: &str,
    reason: &str,
    now_ms: u64,
) -> Result<(), MarketingGovernanceError> {
    if decision.allowed {
        return Ok(());
    }

    let audit = build_coupon_template_lifecycle_audit_record(
        coupon_template,
        None,
        None,
        action,
        CouponTemplateLifecycleAuditOutcome::Rejected,
        operator_id,
        request_id,
        reason,
        now_ms,
        decision.reasons.clone(),
    );
    persist_coupon_template_lifecycle_audit_record(store, &audit).await?;
    Err(MarketingGovernanceError::invalid_input(
        decision
            .reasons
            .first()
            .cloned()
            .unwrap_or_else(|| "coupon template lifecycle action is not allowed".to_owned()),
    ))
}
