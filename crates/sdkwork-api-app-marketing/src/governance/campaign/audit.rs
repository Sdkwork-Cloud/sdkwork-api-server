use super::super::MarketingGovernanceError;
use super::lookup::marketing_campaign_revision;
use sdkwork_api_domain_marketing::{
    MarketingCampaignLifecycleAction, MarketingCampaignLifecycleAuditOutcome,
    MarketingCampaignLifecycleAuditRecord, MarketingCampaignRecord,
};
use sdkwork_api_storage_core::AdminStore;

#[allow(clippy::too_many_arguments)]
pub(super) fn build_marketing_campaign_lifecycle_audit_record(
    before: &MarketingCampaignRecord,
    after: Option<&MarketingCampaignRecord>,
    source_marketing_campaign_id: Option<String>,
    action: MarketingCampaignLifecycleAction,
    outcome: MarketingCampaignLifecycleAuditOutcome,
    operator_id: &str,
    request_id: &str,
    reason: &str,
    requested_at_ms: u64,
    decision_reasons: Vec<String>,
) -> MarketingCampaignLifecycleAuditRecord {
    let after_campaign = after.unwrap_or(before);
    let audit_marketing_campaign_id = if action == MarketingCampaignLifecycleAction::Clone {
        after_campaign.marketing_campaign_id.clone()
    } else {
        before.marketing_campaign_id.clone()
    };
    MarketingCampaignLifecycleAuditRecord::new(
        format!(
            "marketing_campaign_audit:{request_id}:{}:{}",
            audit_marketing_campaign_id,
            action.as_str()
        ),
        audit_marketing_campaign_id,
        before.coupon_template_id.clone(),
        action,
        outcome,
        before.status,
        after_campaign.status,
        operator_id.to_owned(),
        request_id.to_owned(),
        reason.to_owned(),
        requested_at_ms,
    )
    .with_source_marketing_campaign_id(source_marketing_campaign_id)
    .with_approval_states(before.approval_state, after_campaign.approval_state)
    .with_revisions(
        marketing_campaign_revision(before),
        marketing_campaign_revision(after_campaign),
    )
    .with_decision_reasons(decision_reasons)
}

pub(super) async fn persist_marketing_campaign_lifecycle_audit_record(
    store: &dyn AdminStore,
    record: &MarketingCampaignLifecycleAuditRecord,
) -> Result<MarketingCampaignLifecycleAuditRecord, MarketingGovernanceError> {
    store
        .insert_marketing_campaign_lifecycle_audit_record(record)
        .await
        .map_err(MarketingGovernanceError::storage)
}
