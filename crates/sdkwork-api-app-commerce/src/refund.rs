use crate::constants::{COMMERCE_PAYMENT_PROVIDER_MANUAL_LAB, COMMERCE_PAYMENT_PROVIDER_STRIPE};
use crate::error::{CommerceError, CommerceResult};
use crate::payment_attempt::{
    list_payment_attempts_for_order, persist_payment_attempt,
    resolve_payment_attempt_for_provider_reference,
};
use crate::payment_provider::stripe;
use crate::payment_provider::{load_payment_method, resolve_payment_method_secret_bundle};
use crate::types::AdminCommerceRefundCreateRequest;
use reqwest::Client;
use sdkwork_api_app_billing::CommercialBillingAdminKernel;
use sdkwork_api_app_credential::CredentialSecretManager;
use sdkwork_api_domain_commerce::{
    CommerceOrderRecord, CommercePaymentAttemptRecord, CommerceRefundRecord,
};
use sdkwork_api_storage_core::AdminStore;

pub async fn create_admin_commerce_refund(
    store: &dyn AdminStore,
    commercial_billing: Option<&dyn CommercialBillingAdminKernel>,
    secret_manager: &CredentialSecretManager,
    order_id: &str,
    request: &AdminCommerceRefundCreateRequest,
) -> CommerceResult<CommerceRefundRecord> {
    let order = find_order_by_id(store, order_id).await?;
    if order.payable_price_cents == 0 {
        return Err(CommerceError::Conflict(format!(
            "order {} does not support financial refunds",
            order.order_id
        )));
    }
    if order.refundable_amount_minor == 0 {
        return Err(CommerceError::Conflict(format!(
            "order {} has no refundable amount remaining",
            order.order_id
        )));
    }

    let refund_amount_minor = request
        .amount_minor
        .unwrap_or(order.refundable_amount_minor);
    if refund_amount_minor == 0 {
        return Err(CommerceError::InvalidInput(
            "refund amount_minor must be greater than zero".to_owned(),
        ));
    }
    if refund_amount_minor > order.refundable_amount_minor {
        return Err(CommerceError::Conflict(format!(
            "refund amount {} exceeds refundable amount {} for order {}",
            refund_amount_minor, order.refundable_amount_minor, order.order_id
        )));
    }

    let payment_attempt = resolve_payment_attempt_for_refund(store, &order, request).await?;
    let provider = payment_attempt
        .as_ref()
        .map(|attempt| attempt.provider.clone())
        .unwrap_or_else(|| COMMERCE_PAYMENT_PROVIDER_MANUAL_LAB.to_owned());
    let payment_method_id = payment_attempt
        .as_ref()
        .map(|attempt| attempt.payment_method_id.clone())
        .or_else(|| order.payment_method_id.clone());
    let idempotency_key = request
        .idempotency_key
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| {
            format!(
                "commerce-refund:{}:{}:{}",
                order.order_id, refund_amount_minor, order.refunded_amount_minor
            )
        });
    if let Some(existing_refund) = store
        .find_commerce_refund_by_idempotency_key(&idempotency_key)
        .await
        .map_err(CommerceError::from)?
    {
        if existing_refund.order_id != order.order_id {
            return Err(CommerceError::Conflict(format!(
                "refund idempotency_key {} already belongs to order {}",
                idempotency_key, existing_refund.order_id
            )));
        }
        return Ok(existing_refund);
    }

    let created_at_ms = crate::current_time_ms()?;
    let refund_id = crate::generate_entity_id("commerce_refund")?;
    let mut refund = CommerceRefundRecord::new(
        refund_id,
        order.order_id.clone(),
        provider.clone(),
        idempotency_key,
        refund_amount_minor,
        order.currency_code.clone(),
        created_at_ms,
    )
    .with_payment_attempt_id_option(
        payment_attempt
            .as_ref()
            .map(|attempt| attempt.payment_attempt_id.clone()),
    )
    .with_payment_method_id_option(payment_method_id.clone())
    .with_reason_option(request.reason.clone())
    .with_request_payload_json(
        serde_json::to_string(request).map_err(|error| CommerceError::Storage(error.into()))?,
    )
    .with_updated_at_ms(created_at_ms);
    refund = store
        .upsert_commerce_refund(&refund)
        .await
        .map_err(CommerceError::from)?;
    let base_refund = refund.clone();

    let refund_result = match provider.as_str() {
        COMMERCE_PAYMENT_PROVIDER_STRIPE => {
            let payment_attempt = payment_attempt.as_ref().ok_or_else(|| {
                CommerceError::Conflict(format!(
                    "order {} has no provider-backed payment attempt for refund",
                    order.order_id
                ))
            })?;
            let payment_method_id = payment_method_id.as_deref().ok_or_else(|| {
                CommerceError::Conflict(format!(
                    "order {} is missing payment_method_id for stripe refund",
                    order.order_id
                ))
            })?;
            let payment_method = load_payment_method(store, payment_method_id).await?;
            let secrets =
                resolve_payment_method_secret_bundle(store, secret_manager, payment_method_id)
                    .await?;
            let client = Client::new();
            let provider_refund = stripe::create_refund(
                &client,
                &secrets.api_secret,
                payment_attempt,
                &base_refund,
                request,
            )
            .await?;
            let mut next_refund = base_refund
                .clone()
                .with_payment_method_id_option(Some(payment_method.payment_method_id))
                .with_provider_refund_id_option(Some(provider_refund.provider_refund_id))
                .with_status(provider_refund.status)
                .with_request_payload_json(provider_refund.request_payload_json)
                .with_response_payload_json(provider_refund.response_payload_json)
                .with_updated_at_ms(crate::current_time_ms()?);
            if next_refund.status == "succeeded" {
                next_refund =
                    next_refund.with_completed_at_ms_option(Some(crate::current_time_ms()?));
            }
            Ok(next_refund)
        }
        COMMERCE_PAYMENT_PROVIDER_MANUAL_LAB => Ok(base_refund
            .clone()
            .with_status("succeeded")
            .with_response_payload_json("{\"provider\":\"manual_lab\",\"status\":\"succeeded\"}")
            .with_updated_at_ms(crate::current_time_ms()?)
            .with_completed_at_ms_option(Some(crate::current_time_ms()?))),
        provider => Err(CommerceError::InvalidInput(format!(
            "refund flow is not wired for provider {provider}"
        ))),
    };

    match refund_result {
        Ok(mut refund) => {
            refund = store
                .upsert_commerce_refund(&refund)
                .await
                .map_err(CommerceError::from)?;

            if refund.status == "succeeded" {
                let payment_attempt_ref = payment_attempt.as_ref();
                let _ = apply_refund_completion_side_effects(
                    store,
                    commercial_billing,
                    &order,
                    payment_attempt_ref,
                    refund.amount_minor,
                    crate::current_time_ms()?,
                )
                .await?;
            }

            Ok(refund)
        }
        Err(error) => {
            let failed_refund = base_refund
                .with_status("failed")
                .with_response_payload_json(
                    serde_json::json!({
                        "error": error.to_string(),
                    })
                    .to_string(),
                )
                .with_updated_at_ms(crate::current_time_ms()?);
            let _ = store.upsert_commerce_refund(&failed_refund).await;
            Err(error)
        }
    }
}

pub async fn list_admin_commerce_refunds_for_order(
    store: &dyn AdminStore,
    order_id: &str,
) -> CommerceResult<Vec<CommerceRefundRecord>> {
    let mut refunds = store
        .list_commerce_refunds_for_order(order_id)
        .await
        .map_err(CommerceError::from)?;
    refunds.sort_by(|left, right| {
        right
            .updated_at_ms
            .cmp(&left.updated_at_ms)
            .then_with(|| right.refund_id.cmp(&left.refund_id))
    });
    Ok(refunds)
}

pub(crate) async fn apply_refund_completion_side_effects(
    store: &dyn AdminStore,
    commercial_billing: Option<&dyn CommercialBillingAdminKernel>,
    order: &CommerceOrderRecord,
    payment_attempt: Option<&CommercePaymentAttemptRecord>,
    refund_amount_minor: u64,
    now_ms: u64,
) -> CommerceResult<CommerceOrderRecord> {
    let mut current_order = find_order_by_id(store, &order.order_id).await?;
    let next_refunded_amount_minor = current_order
        .refunded_amount_minor
        .saturating_add(refund_amount_minor)
        .min(current_order.payable_price_cents);
    let applied_delta =
        next_refunded_amount_minor.saturating_sub(current_order.refunded_amount_minor);
    if applied_delta == 0 {
        return Ok(current_order);
    }

    let fully_refunded = next_refunded_amount_minor >= current_order.payable_price_cents;
    if fully_refunded && supports_safe_entitlement_rollback(&current_order) {
        current_order = crate::refund_portal_commerce_order(
            store,
            commercial_billing,
            &current_order.user_id,
            &current_order.project_id,
            &current_order.order_id,
            payment_attempt
                .map(|attempt| attempt.provider.as_str())
                .unwrap_or(COMMERCE_PAYMENT_PROVIDER_MANUAL_LAB),
        )
        .await?;
    } else {
        current_order.refunded_amount_minor = next_refunded_amount_minor;
        current_order.refundable_amount_minor = current_order
            .payable_price_cents
            .saturating_sub(next_refunded_amount_minor);
        current_order.settlement_status = if current_order.refundable_amount_minor == 0 {
            "refunded".to_owned()
        } else {
            "partially_refunded".to_owned()
        };
        current_order.updated_at_ms = now_ms;
        current_order = store
            .insert_commerce_order(&current_order)
            .await
            .map_err(CommerceError::from)?;
    }

    if let Some(payment_attempt) = payment_attempt {
        let mut updated_payment_attempt = payment_attempt.clone();
        updated_payment_attempt.refunded_amount_minor = updated_payment_attempt
            .refunded_amount_minor
            .saturating_add(applied_delta)
            .min(updated_payment_attempt.amount_minor);
        updated_payment_attempt.status = if updated_payment_attempt.refunded_amount_minor
            >= updated_payment_attempt
                .captured_amount_minor
                .max(updated_payment_attempt.amount_minor)
        {
            "refunded".to_owned()
        } else {
            "partially_refunded".to_owned()
        };
        updated_payment_attempt.updated_at_ms = now_ms;
        let _ = persist_payment_attempt(store, &updated_payment_attempt).await?;
    }

    Ok(current_order)
}

async fn resolve_payment_attempt_for_refund(
    store: &dyn AdminStore,
    order: &CommerceOrderRecord,
    request: &AdminCommerceRefundCreateRequest,
) -> CommerceResult<Option<CommercePaymentAttemptRecord>> {
    if let Some(payment_attempt_id) = request
        .payment_attempt_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return resolve_payment_attempt_for_provider_reference(
            store,
            &order.order_id,
            Some(payment_attempt_id),
            None,
            None,
        )
        .await;
    }

    let attempts = list_payment_attempts_for_order(store, &order.order_id).await?;
    Ok(attempts.into_iter().find(|attempt| {
        matches!(
            attempt.status.as_str(),
            "succeeded" | "partially_refunded" | "refunded"
        )
    }))
}

async fn find_order_by_id(
    store: &dyn AdminStore,
    order_id: &str,
) -> CommerceResult<CommerceOrderRecord> {
    let order_id = order_id.trim();
    if order_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "order_id is required".to_owned(),
        ));
    }

    store
        .list_commerce_orders()
        .await
        .map_err(CommerceError::from)?
        .into_iter()
        .find(|order| order.order_id == order_id)
        .ok_or_else(|| CommerceError::NotFound(format!("order {order_id} not found")))
}

fn supports_safe_entitlement_rollback(order: &CommerceOrderRecord) -> bool {
    matches!(
        order.target_kind.as_str(),
        "recharge_pack" | "custom_recharge"
    )
}
