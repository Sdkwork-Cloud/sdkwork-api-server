use super::error::MarketingOperationError;
use super::types::ValidatedCouponResult;
use crate::{load_marketing_coupon_context_by_value, validate_marketing_coupon_context};
use sdkwork_api_domain_marketing::MarketingSubjectScope;
use sdkwork_api_storage_core::AdminStore;

#[allow(clippy::too_many_arguments)]
pub async fn validate_coupon_for_subject(
    store: &dyn AdminStore,
    coupon_code: &str,
    subject_scope: MarketingSubjectScope,
    subject_id: &str,
    target_kind: &str,
    order_amount_minor: u64,
    reserve_amount_minor: u64,
    now_ms: u64,
) -> Result<ValidatedCouponResult, MarketingOperationError> {
    let target_kind = target_kind.trim();
    if target_kind.is_empty() {
        return Err(MarketingOperationError::invalid_input(
            "target_kind is required",
        ));
    }

    let Some(context) = load_marketing_coupon_context_by_value(store, coupon_code, now_ms)
        .await
        .map_err(MarketingOperationError::storage)?
    else {
        return Err(MarketingOperationError::not_found("coupon code not found"));
    };

    let decision = validate_marketing_coupon_context(
        &context,
        target_kind,
        now_ms,
        order_amount_minor,
        reserve_amount_minor,
        Some((subject_scope, subject_id)),
    );

    Ok(ValidatedCouponResult { context, decision })
}
