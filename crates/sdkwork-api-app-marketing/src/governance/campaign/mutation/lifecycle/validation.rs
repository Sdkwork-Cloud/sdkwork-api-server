use super::super::super::audit::{
    build_marketing_campaign_lifecycle_audit_record,
    persist_marketing_campaign_lifecycle_audit_record,
};
use super::super::super::types::MarketingCampaignActionDecision;
use crate::MarketingGovernanceError;
use sdkwork_api_domain_marketing::{
    MarketingCampaignLifecycleAction, MarketingCampaignLifecycleAuditOutcome,
    MarketingCampaignRecord,
};
use sdkwork_api_storage_core::AdminStore;

#[allow(clippy::too_many_arguments)]
pub(super) async fn ensure_marketing_campaign_lifecycle_allowed(
    store: &dyn AdminStore,
    campaign: &MarketingCampaignRecord,
    action: MarketingCampaignLifecycleAction,
    decision: &MarketingCampaignActionDecision,
    operator_id: &str,
    request_id: &str,
    reason: &str,
    now_ms: u64,
) -> Result<(), MarketingGovernanceError> {
    if decision.allowed {
        return Ok(());
    }

    let audit = build_marketing_campaign_lifecycle_audit_record(
        campaign,
        None,
        None,
        action,
        MarketingCampaignLifecycleAuditOutcome::Rejected,
        operator_id,
        request_id,
        reason,
        now_ms,
        decision.reasons.clone(),
    );
    persist_marketing_campaign_lifecycle_audit_record(store, &audit).await?;
    Err(MarketingGovernanceError::invalid_input(
        decision
            .reasons
            .first()
            .cloned()
            .unwrap_or_else(|| "campaign lifecycle action is not allowed".to_owned()),
    ))
}
