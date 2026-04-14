use super::super::super::actionability::build_marketing_campaign_actionability;
use super::super::super::audit::{
    build_marketing_campaign_lifecycle_audit_record,
    persist_marketing_campaign_lifecycle_audit_record,
};
use super::super::super::types::CloneMarketingCampaignRevisionInput;
use crate::MarketingGovernanceError;
use sdkwork_api_domain_marketing::{
    CouponTemplateRecord, MarketingCampaignLifecycleAction, MarketingCampaignLifecycleAuditOutcome,
    MarketingCampaignRecord,
};
use sdkwork_api_storage_core::AdminStore;

pub(super) async fn ensure_marketing_campaign_clone_allowed(
    store: &dyn AdminStore,
    source_campaign: &MarketingCampaignRecord,
    coupon_template: &CouponTemplateRecord,
    input: &CloneMarketingCampaignRevisionInput,
    operator_id: &str,
    request_id: &str,
    reason: &str,
    now_ms: u64,
) -> Result<(), MarketingGovernanceError> {
    let actionability =
        build_marketing_campaign_actionability(source_campaign, coupon_template, now_ms);
    if !actionability.clone.allowed {
        let audit = build_marketing_campaign_lifecycle_audit_record(
            source_campaign,
            None,
            None,
            MarketingCampaignLifecycleAction::Clone,
            MarketingCampaignLifecycleAuditOutcome::Rejected,
            operator_id,
            request_id,
            reason,
            now_ms,
            actionability.clone.reasons.clone(),
        );
        persist_marketing_campaign_lifecycle_audit_record(store, &audit).await?;
        return Err(MarketingGovernanceError::invalid_input(
            actionability
                .clone
                .reasons
                .first()
                .cloned()
                .unwrap_or_else(|| "campaign clone is not allowed".to_owned()),
        ));
    }

    if input.marketing_campaign_id == source_campaign.marketing_campaign_id {
        return Err(MarketingGovernanceError::invalid_input(
            "cloned marketing campaign must use a new marketing_campaign_id",
        ));
    }
    if store
        .find_marketing_campaign_record(&input.marketing_campaign_id)
        .await
        .map_err(MarketingGovernanceError::storage)?
        .is_some()
    {
        return Err(MarketingGovernanceError::conflict(format!(
            "marketing campaign {} already exists",
            input.marketing_campaign_id
        )));
    }

    Ok(())
}
