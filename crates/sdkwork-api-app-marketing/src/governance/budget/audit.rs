use super::super::MarketingGovernanceError;
use sdkwork_api_domain_marketing::{
    CampaignBudgetLifecycleAction, CampaignBudgetLifecycleAuditOutcome,
    CampaignBudgetLifecycleAuditRecord, CampaignBudgetRecord,
};
use sdkwork_api_storage_core::AdminStore;

#[allow(clippy::too_many_arguments)]
pub(super) fn build_campaign_budget_lifecycle_audit_record(
    before: &CampaignBudgetRecord,
    after: Option<&CampaignBudgetRecord>,
    action: CampaignBudgetLifecycleAction,
    outcome: CampaignBudgetLifecycleAuditOutcome,
    operator_id: &str,
    request_id: &str,
    reason: &str,
    requested_at_ms: u64,
    decision_reasons: Vec<String>,
) -> CampaignBudgetLifecycleAuditRecord {
    let after_budget = after.unwrap_or(before);
    CampaignBudgetLifecycleAuditRecord::new(
        format!(
            "campaign_budget_audit:{request_id}:{}:{}",
            before.campaign_budget_id,
            action.as_str()
        ),
        before.campaign_budget_id.clone(),
        before.marketing_campaign_id.clone(),
        action,
        outcome,
        before.status,
        after_budget.status,
        operator_id.to_owned(),
        request_id.to_owned(),
        reason.to_owned(),
        requested_at_ms,
    )
    .with_decision_reasons(decision_reasons)
}

pub(super) async fn persist_campaign_budget_lifecycle_audit_record(
    store: &dyn AdminStore,
    record: &CampaignBudgetLifecycleAuditRecord,
) -> Result<CampaignBudgetLifecycleAuditRecord, MarketingGovernanceError> {
    store
        .insert_campaign_budget_lifecycle_audit_record(record)
        .await
        .map_err(MarketingGovernanceError::storage)
}
