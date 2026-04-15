use super::super::MarketingGovernanceError;
use sdkwork_api_domain_marketing::{
    CouponCodeLifecycleAction, CouponCodeLifecycleAuditOutcome, CouponCodeLifecycleAuditRecord,
    CouponCodeRecord,
};
use sdkwork_api_storage_core::AdminStore;

#[allow(clippy::too_many_arguments)]
pub(super) fn build_coupon_code_lifecycle_audit_record(
    before: &CouponCodeRecord,
    after: Option<&CouponCodeRecord>,
    action: CouponCodeLifecycleAction,
    outcome: CouponCodeLifecycleAuditOutcome,
    operator_id: &str,
    request_id: &str,
    reason: &str,
    requested_at_ms: u64,
    decision_reasons: Vec<String>,
) -> CouponCodeLifecycleAuditRecord {
    let after_code = after.unwrap_or(before);
    CouponCodeLifecycleAuditRecord::new(
        format!(
            "coupon_code_audit:{request_id}:{}:{}",
            before.coupon_code_id,
            action.as_str()
        ),
        before.coupon_code_id.clone(),
        before.coupon_template_id.clone(),
        action,
        outcome,
        before.status,
        after_code.status,
        operator_id.to_owned(),
        request_id.to_owned(),
        reason.to_owned(),
        requested_at_ms,
    )
    .with_decision_reasons(decision_reasons)
}

pub(super) async fn persist_coupon_code_lifecycle_audit_record(
    store: &dyn AdminStore,
    record: &CouponCodeLifecycleAuditRecord,
) -> Result<CouponCodeLifecycleAuditRecord, MarketingGovernanceError> {
    store
        .insert_coupon_code_lifecycle_audit_record(record)
        .await
        .map_err(MarketingGovernanceError::storage)
}
