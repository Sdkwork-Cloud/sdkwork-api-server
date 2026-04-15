use super::super::MarketingGovernanceError;
use super::lookup::coupon_template_revision;
use sdkwork_api_domain_marketing::{
    CouponTemplateLifecycleAction, CouponTemplateLifecycleAuditOutcome,
    CouponTemplateLifecycleAuditRecord, CouponTemplateRecord,
};
use sdkwork_api_storage_core::AdminStore;

#[allow(clippy::too_many_arguments)]
pub(super) fn build_coupon_template_lifecycle_audit_record(
    before: &CouponTemplateRecord,
    after: Option<&CouponTemplateRecord>,
    source_coupon_template_id: Option<String>,
    action: CouponTemplateLifecycleAction,
    outcome: CouponTemplateLifecycleAuditOutcome,
    operator_id: &str,
    request_id: &str,
    reason: &str,
    requested_at_ms: u64,
    decision_reasons: Vec<String>,
) -> CouponTemplateLifecycleAuditRecord {
    let after_template = after.unwrap_or(before);
    let audit_coupon_template_id = if action == CouponTemplateLifecycleAction::Clone {
        after_template.coupon_template_id.clone()
    } else {
        before.coupon_template_id.clone()
    };
    CouponTemplateLifecycleAuditRecord::new(
        format!(
            "coupon_template_audit:{request_id}:{}:{}",
            audit_coupon_template_id,
            action.as_str()
        ),
        audit_coupon_template_id,
        action,
        outcome,
        before.status,
        after_template.status,
        operator_id.to_owned(),
        request_id.to_owned(),
        reason.to_owned(),
        requested_at_ms,
    )
    .with_source_coupon_template_id(source_coupon_template_id)
    .with_approval_states(before.approval_state, after_template.approval_state)
    .with_revisions(
        coupon_template_revision(before),
        coupon_template_revision(after_template),
    )
    .with_decision_reasons(decision_reasons)
}

pub(super) async fn persist_coupon_template_lifecycle_audit_record(
    store: &dyn AdminStore,
    record: &CouponTemplateLifecycleAuditRecord,
) -> Result<CouponTemplateLifecycleAuditRecord, MarketingGovernanceError> {
    store
        .insert_coupon_template_lifecycle_audit_record(record)
        .await
        .map_err(MarketingGovernanceError::storage)
}
