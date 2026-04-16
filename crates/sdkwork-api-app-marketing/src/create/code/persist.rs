use super::super::support::{
    load_coupon_template_record, marketing_create_invalid_input, marketing_create_storage,
    validate_coupon_code_template_compatibility, PersistMode,
};
use super::prepare::prepare_coupon_code_record_for_create;
use crate::governance::MarketingGovernanceError;
use sdkwork_api_domain_marketing::CouponCodeRecord;
use sdkwork_api_storage_core::AdminStore;

async fn persist_coupon_code_record(
    store: &dyn AdminStore,
    record: CouponCodeRecord,
    mode: PersistMode,
) -> Result<CouponCodeRecord, MarketingGovernanceError> {
    let record =
        prepare_coupon_code_record_for_create(record).map_err(marketing_create_invalid_input)?;

    if let Some(existing_code) = store
        .find_coupon_code_record(&record.coupon_code_id)
        .await
        .map_err(marketing_create_storage)?
    {
        if coupon_code_seed_matches(&existing_code, &record) {
            return Ok(existing_code);
        }
        if matches!(mode, PersistMode::Ensure) && coupon_code_seed_can_reconcile(&existing_code) {
            let coupon_template =
                load_coupon_template_record(store, &record.coupon_template_id).await?;
            validate_coupon_code_template_compatibility(&coupon_template, &record)?;
            return store
                .insert_coupon_code_record(&record)
                .await
                .map_err(marketing_create_storage);
        }
        return mode.resolve_existing_primary_with(
            "coupon code",
            &record.coupon_code_id,
            existing_code,
            &record,
            coupon_code_seed_matches,
        );
    }
    if let Some(existing_code) = store
        .find_coupon_code_record_by_value(&record.code_value)
        .await
        .map_err(marketing_create_storage)?
    {
        return mode.resolve_existing_unique(existing_code, &record, |existing_code| {
            MarketingGovernanceError::Conflict(format!(
                "coupon code value {} already exists on {}",
                record.code_value, existing_code.coupon_code_id
            ))
        });
    }

    let coupon_template = load_coupon_template_record(store, &record.coupon_template_id).await?;
    validate_coupon_code_template_compatibility(&coupon_template, &record)?;

    store
        .insert_coupon_code_record(&record)
        .await
        .map_err(marketing_create_storage)
}

pub async fn create_coupon_code_record(
    store: &dyn AdminStore,
    record: CouponCodeRecord,
) -> Result<CouponCodeRecord, MarketingGovernanceError> {
    persist_coupon_code_record(store, record, PersistMode::Create).await
}

pub async fn ensure_coupon_code_record(
    store: &dyn AdminStore,
    record: CouponCodeRecord,
) -> Result<CouponCodeRecord, MarketingGovernanceError> {
    persist_coupon_code_record(store, record, PersistMode::Ensure).await
}

fn coupon_code_seed_matches(existing: &CouponCodeRecord, desired: &CouponCodeRecord) -> bool {
    existing.coupon_code_id == desired.coupon_code_id
        && existing.coupon_template_id == desired.coupon_template_id
        && existing.code_value == desired.code_value
        && existing.expires_at_ms == desired.expires_at_ms
}

fn coupon_code_seed_can_reconcile(existing: &CouponCodeRecord) -> bool {
    existing.status == sdkwork_api_domain_marketing::CouponCodeStatus::Available
        && existing.claimed_subject_scope.is_none()
        && existing.claimed_subject_id.is_none()
}
