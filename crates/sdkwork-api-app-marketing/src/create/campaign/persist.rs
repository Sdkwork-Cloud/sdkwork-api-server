use super::super::support::{
    load_coupon_template_record, marketing_create_invalid_input, marketing_create_storage,
    PersistMode,
};
use super::prepare::prepare_marketing_campaign_record_for_create;
use crate::governance::MarketingGovernanceError;
use sdkwork_api_domain_marketing::MarketingCampaignRecord;
use sdkwork_api_storage_core::AdminStore;

async fn persist_marketing_campaign_record(
    store: &dyn AdminStore,
    record: MarketingCampaignRecord,
    mode: PersistMode,
) -> Result<MarketingCampaignRecord, MarketingGovernanceError> {
    let record = prepare_marketing_campaign_record_for_create(record)
        .map_err(marketing_create_invalid_input)?;

    if let Some(existing_campaign) = store
        .find_marketing_campaign_record(&record.marketing_campaign_id)
        .await
        .map_err(marketing_create_storage)?
    {
        if marketing_campaign_seed_matches(&existing_campaign, &record) {
            return Ok(existing_campaign);
        }
        if matches!(mode, PersistMode::Ensure) {
            let reconciled =
                reconcile_marketing_campaign_seed_record(store, &existing_campaign, &record)
                    .await?;
            return store
                .insert_marketing_campaign_record(&reconciled)
                .await
                .map_err(marketing_create_storage);
        }
        return mode.resolve_existing_primary_with(
            "marketing campaign",
            &record.marketing_campaign_id,
            existing_campaign,
            &record,
            marketing_campaign_seed_matches,
        );
    }

    load_coupon_template_record(store, &record.coupon_template_id).await?;

    store
        .insert_marketing_campaign_record(&record)
        .await
        .map_err(marketing_create_storage)
}

pub async fn create_marketing_campaign_record(
    store: &dyn AdminStore,
    record: MarketingCampaignRecord,
) -> Result<MarketingCampaignRecord, MarketingGovernanceError> {
    persist_marketing_campaign_record(store, record, PersistMode::Create).await
}

pub async fn ensure_marketing_campaign_record(
    store: &dyn AdminStore,
    record: MarketingCampaignRecord,
) -> Result<MarketingCampaignRecord, MarketingGovernanceError> {
    persist_marketing_campaign_record(store, record, PersistMode::Ensure).await
}

fn marketing_campaign_seed_matches(
    existing: &MarketingCampaignRecord,
    desired: &MarketingCampaignRecord,
) -> bool {
    existing.marketing_campaign_id == desired.marketing_campaign_id
        && existing.coupon_template_id == desired.coupon_template_id
        && existing.display_name == desired.display_name
        && existing.start_at_ms == desired.start_at_ms
        && existing.end_at_ms == desired.end_at_ms
}

async fn reconcile_marketing_campaign_seed_record(
    store: &dyn AdminStore,
    existing: &MarketingCampaignRecord,
    desired: &MarketingCampaignRecord,
) -> Result<MarketingCampaignRecord, MarketingGovernanceError> {
    load_coupon_template_record(store, &desired.coupon_template_id).await?;

    Ok(desired
        .clone()
        .with_status(existing.status)
        .with_approval_state(existing.approval_state)
        .with_revision(existing.revision)
        .with_root_marketing_campaign_id(existing.root_marketing_campaign_id.clone())
        .with_parent_marketing_campaign_id(existing.parent_marketing_campaign_id.clone())
        .with_created_at_ms(existing.created_at_ms)
        .with_updated_at_ms(existing.updated_at_ms.max(desired.updated_at_ms)))
}
