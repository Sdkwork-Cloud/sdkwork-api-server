use super::super::error::MarketingOperationError;
use super::super::support::{find_coupon_rollback_record, load_coupon_redemption_for_subject};
use super::super::types::{RollbackCouponInput, RollbackCouponResult};
use crate::load_marketing_coupon_context_for_code_id;
use sdkwork_api_domain_marketing::CouponRollbackStatus;
use sdkwork_api_storage_core::AdminStore;

pub(crate) async fn try_replay_rolled_back_coupon(
    store: &dyn AdminStore,
    input: &RollbackCouponInput<'_>,
    coupon_rollback_id: &str,
) -> Result<Option<RollbackCouponResult>, MarketingOperationError> {
    if input.idempotency_key.is_none() {
        return Ok(None);
    }

    let Some(existing_rollback) = find_coupon_rollback_record(store, coupon_rollback_id).await?
    else {
        return Ok(None);
    };
    if existing_rollback.rollback_status == CouponRollbackStatus::Failed {
        return Ok(None);
    }

    let redemption = load_coupon_redemption_for_subject(
        store,
        input.subject_scope,
        input.subject_id,
        &existing_rollback.coupon_redemption_id,
    )
    .await?;
    if existing_rollback.coupon_redemption_id != input.coupon_redemption_id
        || existing_rollback.rollback_type != input.rollback_type
        || existing_rollback.restored_budget_minor != input.restored_budget_minor
        || existing_rollback.restored_inventory_count != input.restored_inventory_count
    {
        return Err(MarketingOperationError::conflict(
            "idempotent rollback replay does not match the original request",
        ));
    }

    let context =
        load_marketing_coupon_context_for_code_id(store, &redemption.coupon_code_id, input.now_ms)
            .await
            .map_err(MarketingOperationError::storage)?
            .ok_or_else(|| {
                MarketingOperationError::not_found(
                    "coupon context is unavailable for rollback replay",
                )
            })?;

    Ok(Some(RollbackCouponResult {
        context,
        redemption,
        rollback: existing_rollback,
        created: false,
    }))
}
