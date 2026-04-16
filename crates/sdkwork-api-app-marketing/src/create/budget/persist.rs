use super::super::support::{
    marketing_create_invalid_input, marketing_create_storage, require_marketing_campaign_record,
    PersistMode,
};
use super::prepare::prepare_campaign_budget_record_for_create;
use crate::governance::MarketingGovernanceError;
use sdkwork_api_domain_marketing::CampaignBudgetRecord;
use sdkwork_api_storage_core::AdminStore;

async fn persist_campaign_budget_record(
    store: &dyn AdminStore,
    record: CampaignBudgetRecord,
    mode: PersistMode,
) -> Result<CampaignBudgetRecord, MarketingGovernanceError> {
    let record = prepare_campaign_budget_record_for_create(record)
        .map_err(marketing_create_invalid_input)?;

    if let Some(existing_budget) = store
        .find_campaign_budget_record(&record.campaign_budget_id)
        .await
        .map_err(marketing_create_storage)?
    {
        if campaign_budget_seed_matches(&existing_budget, &record) {
            return Ok(existing_budget);
        }
        if matches!(mode, PersistMode::Ensure) {
            let reconciled =
                reconcile_campaign_budget_seed_record(store, &existing_budget, &record).await?;
            return store
                .insert_campaign_budget_record(&reconciled)
                .await
                .map_err(marketing_create_storage);
        }
        return mode.resolve_existing_primary_with(
            "campaign budget",
            &record.campaign_budget_id,
            existing_budget,
            &record,
            campaign_budget_seed_matches,
        );
    }

    require_marketing_campaign_record(store, &record.marketing_campaign_id).await?;

    store
        .insert_campaign_budget_record(&record)
        .await
        .map_err(marketing_create_storage)
}

pub async fn create_campaign_budget_record(
    store: &dyn AdminStore,
    record: CampaignBudgetRecord,
) -> Result<CampaignBudgetRecord, MarketingGovernanceError> {
    persist_campaign_budget_record(store, record, PersistMode::Create).await
}

pub async fn ensure_campaign_budget_record(
    store: &dyn AdminStore,
    record: CampaignBudgetRecord,
) -> Result<CampaignBudgetRecord, MarketingGovernanceError> {
    persist_campaign_budget_record(store, record, PersistMode::Ensure).await
}

fn campaign_budget_seed_matches(
    existing: &CampaignBudgetRecord,
    desired: &CampaignBudgetRecord,
) -> bool {
    existing.campaign_budget_id == desired.campaign_budget_id
        && existing.marketing_campaign_id == desired.marketing_campaign_id
        && existing.total_budget_minor == desired.total_budget_minor
}

async fn reconcile_campaign_budget_seed_record(
    store: &dyn AdminStore,
    existing: &CampaignBudgetRecord,
    desired: &CampaignBudgetRecord,
) -> Result<CampaignBudgetRecord, MarketingGovernanceError> {
    require_marketing_campaign_record(store, &desired.marketing_campaign_id).await?;

    let committed_usage_minor = existing
        .reserved_budget_minor
        .saturating_add(existing.consumed_budget_minor);
    if desired.total_budget_minor < committed_usage_minor {
        return Err(MarketingGovernanceError::Conflict(format!(
            "campaign budget {} cannot shrink total budget below committed usage {}",
            desired.campaign_budget_id, committed_usage_minor
        )));
    }

    Ok(desired
        .clone()
        .with_status(existing.status)
        .with_reserved_budget_minor(existing.reserved_budget_minor)
        .with_consumed_budget_minor(existing.consumed_budget_minor)
        .with_created_at_ms(existing.created_at_ms)
        .with_updated_at_ms(existing.updated_at_ms.max(desired.updated_at_ms)))
}
