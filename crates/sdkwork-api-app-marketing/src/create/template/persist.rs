use super::super::support::{
    marketing_create_invalid_input, marketing_create_storage, PersistMode,
};
use super::prepare::prepare_coupon_template_record_for_create;
use crate::governance::MarketingGovernanceError;
use sdkwork_api_domain_marketing::CouponTemplateRecord;
use sdkwork_api_storage_core::AdminStore;

async fn persist_coupon_template_record(
    store: &dyn AdminStore,
    record: CouponTemplateRecord,
    mode: PersistMode,
) -> Result<CouponTemplateRecord, MarketingGovernanceError> {
    let record = prepare_coupon_template_record_for_create(record)
        .map_err(marketing_create_invalid_input)?;

    if let Some(existing_template) = store
        .find_coupon_template_record(&record.coupon_template_id)
        .await
        .map_err(marketing_create_storage)?
    {
        if coupon_template_seed_matches(&existing_template, &record) {
            return Ok(existing_template);
        }
        if matches!(mode, PersistMode::Ensure) {
            let reconciled =
                reconcile_coupon_template_seed_record(store, &existing_template, &record).await?;
            return store
                .insert_coupon_template_record(&reconciled)
                .await
                .map_err(marketing_create_storage);
        }
        return mode.resolve_existing_primary_with(
            "coupon template",
            &record.coupon_template_id,
            existing_template,
            &record,
            coupon_template_seed_matches,
        );
    }
    if let Some(existing_template) = store
        .find_coupon_template_record_by_template_key(&record.template_key)
        .await
        .map_err(marketing_create_storage)?
    {
        return mode.resolve_existing_unique(existing_template, &record, |existing_template| {
            MarketingGovernanceError::Conflict(format!(
                "coupon template key {} already exists on {}",
                record.template_key, existing_template.coupon_template_id
            ))
        });
    }

    store
        .insert_coupon_template_record(&record)
        .await
        .map_err(marketing_create_storage)
}

pub async fn create_coupon_template_record(
    store: &dyn AdminStore,
    record: CouponTemplateRecord,
) -> Result<CouponTemplateRecord, MarketingGovernanceError> {
    persist_coupon_template_record(store, record, PersistMode::Create).await
}

pub async fn ensure_coupon_template_record(
    store: &dyn AdminStore,
    record: CouponTemplateRecord,
) -> Result<CouponTemplateRecord, MarketingGovernanceError> {
    persist_coupon_template_record(store, record, PersistMode::Ensure).await
}

fn coupon_template_seed_matches(
    existing: &CouponTemplateRecord,
    desired: &CouponTemplateRecord,
) -> bool {
    existing.coupon_template_id == desired.coupon_template_id
        && existing.template_key == desired.template_key
        && existing.display_name == desired.display_name
        && existing.distribution_kind == desired.distribution_kind
        && existing.benefit == desired.benefit
        && existing.restriction == desired.restriction
}

async fn reconcile_coupon_template_seed_record(
    store: &dyn AdminStore,
    existing: &CouponTemplateRecord,
    desired: &CouponTemplateRecord,
) -> Result<CouponTemplateRecord, MarketingGovernanceError> {
    if existing.template_key != desired.template_key {
        if let Some(conflicting_template) = store
            .find_coupon_template_record_by_template_key(&desired.template_key)
            .await
            .map_err(marketing_create_storage)?
        {
            if conflicting_template.coupon_template_id != existing.coupon_template_id {
                return Err(MarketingGovernanceError::Conflict(format!(
                    "coupon template key {} already exists on {}",
                    desired.template_key, conflicting_template.coupon_template_id
                )));
            }
        }
    }

    Ok(desired
        .clone()
        .with_status(existing.status)
        .with_approval_state(existing.approval_state)
        .with_revision(existing.revision)
        .with_root_coupon_template_id(existing.root_coupon_template_id.clone())
        .with_parent_coupon_template_id(existing.parent_coupon_template_id.clone())
        .with_activation_at_ms(existing.activation_at_ms)
        .with_created_at_ms(existing.created_at_ms)
        .with_updated_at_ms(existing.updated_at_ms.max(desired.updated_at_ms)))
}
