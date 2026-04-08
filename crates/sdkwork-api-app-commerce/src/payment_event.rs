use sdkwork_api_app_billing::CommercialBillingAdminKernel;
use sdkwork_api_domain_commerce::{
    CommerceOrderRecord, CommercePaymentEventProcessingStatus, CommercePaymentEventRecord,
};
use sdkwork_api_storage_core::AdminStore;

use crate::constants::{
    COMMERCE_PAYMENT_PROVIDER_ALIPAY, COMMERCE_PAYMENT_PROVIDER_MANUAL_LAB,
    COMMERCE_PAYMENT_PROVIDER_NO_PAYMENT_REQUIRED, COMMERCE_PAYMENT_PROVIDER_STRIPE,
    COMMERCE_PAYMENT_PROVIDER_WECHAT_PAY,
};
use crate::error::{CommerceError, CommerceResult};
use crate::payment_method::resolve_checkout_method_for_payment_event;
use crate::types::{PortalCommerceCheckoutSessionMethod, PortalCommercePaymentEventRequest};

pub async fn apply_portal_commerce_payment_event_with_billing(
    store: &dyn AdminStore,
    commercial_billing: Option<&dyn CommercialBillingAdminKernel>,
    user_id: &str,
    project_id: &str,
    order_id: &str,
    request: &PortalCommercePaymentEventRequest,
) -> CommerceResult<CommerceOrderRecord> {
    let event_type = request.event_type.trim();
    if event_type.is_empty() {
        return Err(CommerceError::InvalidInput(
            "event_type is required".to_owned(),
        ));
    }

    if !matches!(event_type, "settled" | "canceled" | "failed" | "refunded") {
        return Err(CommerceError::InvalidInput(format!(
            "unsupported payment event_type: {event_type}"
        )));
    }

    let normalized_user_id = user_id.trim();
    let normalized_project_id = project_id.trim();
    let normalized_order_id = order_id.trim();

    if normalized_user_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "user_id is required".to_owned(),
        ));
    }
    if normalized_project_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "project_id is required".to_owned(),
        ));
    }
    if normalized_order_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "order_id is required".to_owned(),
        ));
    }

    let current_order = super::load_project_commerce_order(
        store,
        normalized_user_id,
        normalized_project_id,
        normalized_order_id,
    )
    .await?;
    let checkout_method =
        resolve_checkout_method_for_payment_event(store, &current_order, request).await?;
    let (provider, provider_event_id, dedupe_key) =
        resolve_commerce_payment_event_identity(&current_order.order_id, request, checkout_method)?;

    let existing_event = store
        .find_commerce_payment_event_by_dedupe_key(&dedupe_key)
        .await
        .map_err(CommerceError::from)?;
    if let Some(existing_event) = existing_event.as_ref() {
        if existing_event.order_id != current_order.order_id {
            return Err(CommerceError::Conflict(format!(
                "payment event {dedupe_key} already belongs to order {}",
                existing_event.order_id
            )));
        }
        if matches!(
            existing_event.processing_status,
            CommercePaymentEventProcessingStatus::Processed
                | CommercePaymentEventProcessingStatus::Ignored
        ) {
            return super::load_project_commerce_order(
                store,
                normalized_user_id,
                normalized_project_id,
                normalized_order_id,
            )
            .await;
        }
    }

    let received_at_ms = super::current_time_ms()?;
    let mut payment_event = build_commerce_payment_event_record(
        &current_order,
        request,
        provider,
        provider_event_id,
        dedupe_key.clone(),
        received_at_ms,
        existing_event
            .as_ref()
            .map(|event| event.payment_event_id.as_str()),
    )?;
    payment_event = persist_commerce_payment_event(store, payment_event).await?;

    let order_result = match event_type {
        "settled" => {
            super::settle_portal_commerce_order_with_payment_event(
                store,
                commercial_billing,
                normalized_user_id,
                normalized_project_id,
                normalized_order_id,
                Some(payment_event.payment_event_id.as_str()),
            )
            .await
        }
        "canceled" => {
            super::cancel_portal_commerce_order(
                store,
                normalized_user_id,
                normalized_project_id,
                normalized_order_id,
            )
            .await
        }
        "failed" => {
            super::fail_portal_commerce_order(
                store,
                normalized_user_id,
                normalized_project_id,
                normalized_order_id,
            )
            .await
        }
        "refunded" => {
            super::refund_portal_commerce_order(
                store,
                commercial_billing,
                normalized_user_id,
                normalized_project_id,
                normalized_order_id,
                payment_event.provider.as_str(),
            )
            .await
        }
        _ => unreachable!(),
    };

    match order_result {
        Ok(order) => {
            let _ = persist_commerce_payment_event(
                store,
                finalize_commerce_payment_event(
                    payment_event,
                    CommercePaymentEventProcessingStatus::Processed,
                    None,
                    Some(order.status.clone()),
                    Some(super::current_time_ms()?),
                ),
            )
            .await;
            Ok(order)
        }
        Err(error) => {
            let rejection_status = commerce_payment_event_status_for_error(&error);
            let order_status_after = load_payment_event_order_status_after_error(
                store,
                normalized_user_id,
                normalized_project_id,
                normalized_order_id,
                &current_order,
            )
            .await;
            let _ = persist_commerce_payment_event(
                store,
                finalize_commerce_payment_event(
                    payment_event,
                    rejection_status,
                    Some(error.to_string()),
                    order_status_after,
                    Some(super::current_time_ms()?),
                ),
            )
            .await;
            Err(error)
        }
    }
}

pub async fn list_order_commerce_payment_events(
    store: &dyn AdminStore,
    user_id: &str,
    project_id: &str,
    order_id: &str,
) -> CommerceResult<Vec<CommercePaymentEventRecord>> {
    let normalized_user_id = user_id.trim();
    let normalized_project_id = project_id.trim();
    let normalized_order_id = order_id.trim();

    if normalized_user_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "user_id is required".to_owned(),
        ));
    }
    if normalized_project_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "project_id is required".to_owned(),
        ));
    }
    if normalized_order_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "order_id is required".to_owned(),
        ));
    }

    let order = super::load_project_commerce_order(
        store,
        normalized_user_id,
        normalized_project_id,
        normalized_order_id,
    )
    .await?;
    let mut events = store
        .list_commerce_payment_events_for_order(&order.order_id)
        .await
        .map_err(CommerceError::from)?;
    events.sort_by(|left, right| {
        right
            .received_at_ms
            .cmp(&left.received_at_ms)
            .then_with(|| right.payment_event_id.cmp(&left.payment_event_id))
    });
    Ok(events)
}

async fn load_payment_event_order_status_after_error(
    store: &dyn AdminStore,
    user_id: &str,
    project_id: &str,
    order_id: &str,
    current_order: &CommerceOrderRecord,
) -> Option<String> {
    super::load_project_commerce_order(store, user_id, project_id, order_id)
        .await
        .map(|order| order.status)
        .ok()
        .or_else(|| Some(current_order.status.clone()))
}

fn resolve_commerce_payment_event_identity(
    order_id: &str,
    request: &PortalCommercePaymentEventRequest,
    checkout_method: Option<PortalCommerceCheckoutSessionMethod>,
) -> CommerceResult<(String, Option<String>, String)> {
    let event_type = request.event_type.trim();
    let provider = request
        .provider
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(normalize_commerce_payment_provider)
        .transpose()?;
    let provider_event_id = request
        .provider_event_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned);
    let checkout_method_provider = checkout_method
        .as_ref()
        .map(|method| method.provider.clone());

    if checkout_method
        .as_ref()
        .is_some_and(|method| method.supports_webhook)
        && provider_event_id.is_none()
    {
        return Err(CommerceError::InvalidInput(
            "provider_event_id is required for webhook-backed checkout methods".to_owned(),
        ));
    }

    let provider = match (provider, checkout_method_provider) {
        (Some(provider), Some(method_provider)) => {
            if provider != method_provider {
                return Err(CommerceError::InvalidInput(format!(
                    "checkout_method_id belongs to provider {method_provider}, but request provider is {provider}"
                )));
            }
            Some(provider)
        }
        (Some(provider), None) => Some(provider),
        (None, Some(method_provider)) => Some(method_provider),
        (None, None) => None,
    };

    if provider.as_ref().is_some_and(|provider| {
        provider != COMMERCE_PAYMENT_PROVIDER_MANUAL_LAB
            && provider != COMMERCE_PAYMENT_PROVIDER_NO_PAYMENT_REQUIRED
    }) && provider_event_id.is_none()
    {
        return Err(CommerceError::InvalidInput(
            "provider_event_id is required for provider-backed payment events".to_owned(),
        ));
    }

    match (provider, provider_event_id) {
        (Some(provider), Some(provider_event_id)) => Ok((
            provider.clone(),
            Some(provider_event_id.clone()),
            format!("{provider}:{provider_event_id}"),
        )),
        (Some(provider), None) => Ok((
            provider.clone(),
            None,
            format!("{provider}:{order_id}:{event_type}"),
        )),
        (None, Some(_)) => Err(CommerceError::InvalidInput(
            "provider is required when provider_event_id is set".to_owned(),
        )),
        (None, None) => Ok((
            COMMERCE_PAYMENT_PROVIDER_MANUAL_LAB.to_owned(),
            None,
            format!("{COMMERCE_PAYMENT_PROVIDER_MANUAL_LAB}:{order_id}:{event_type}"),
        )),
    }
}

fn normalize_commerce_payment_provider(value: &str) -> CommerceResult<String> {
    let provider = match value.trim().to_ascii_lowercase().as_str() {
        "manual" | "manual_lab" | "operator_settlement" => COMMERCE_PAYMENT_PROVIDER_MANUAL_LAB,
        "stripe" | "stripe_checkout" | "stripe_hosted" => COMMERCE_PAYMENT_PROVIDER_STRIPE,
        "alipay" | "alipay_qr" | "alipay_scan" => COMMERCE_PAYMENT_PROVIDER_ALIPAY,
        "wechat" | "wechat_pay" | "wechatpay" | "wxpay" | "wechat_qr" | "wechat_pay_qr" => {
            COMMERCE_PAYMENT_PROVIDER_WECHAT_PAY
        }
        _ => {
            return Err(CommerceError::InvalidInput(format!(
                "unsupported commerce payment provider: {value}"
            )));
        }
    };

    Ok(provider.to_owned())
}

fn build_commerce_payment_event_record(
    order: &CommerceOrderRecord,
    request: &PortalCommercePaymentEventRequest,
    provider: String,
    provider_event_id: Option<String>,
    dedupe_key: String,
    received_at_ms: u64,
    existing_payment_event_id: Option<&str>,
) -> CommerceResult<CommercePaymentEventRecord> {
    let payment_event_id = match existing_payment_event_id {
        Some(payment_event_id) => payment_event_id.to_owned(),
        None => super::generate_entity_id("commerce_payment_event")?,
    };
    let payload_json =
        serde_json::to_string(request).map_err(|error| CommerceError::Storage(error.into()))?;

    Ok(CommercePaymentEventRecord::new(
        payment_event_id,
        order.order_id.clone(),
        order.project_id.clone(),
        order.user_id.clone(),
        provider,
        dedupe_key,
        request.event_type.trim(),
        payload_json,
        received_at_ms,
    )
    .with_provider_event_id(provider_event_id))
}

fn finalize_commerce_payment_event(
    event: CommercePaymentEventRecord,
    processing_status: CommercePaymentEventProcessingStatus,
    processing_message: Option<String>,
    order_status_after: Option<String>,
    processed_at_ms: Option<u64>,
) -> CommercePaymentEventRecord {
    event
        .with_processing_status(processing_status)
        .with_processing_message(processing_message)
        .with_order_status_after(order_status_after)
        .with_processed_at_ms(processed_at_ms)
}

fn commerce_payment_event_status_for_error(
    error: &CommerceError,
) -> CommercePaymentEventProcessingStatus {
    match error {
        CommerceError::Storage(_) => CommercePaymentEventProcessingStatus::Failed,
        CommerceError::InvalidInput(_)
        | CommerceError::NotFound(_)
        | CommerceError::Conflict(_) => CommercePaymentEventProcessingStatus::Rejected,
    }
}

async fn persist_commerce_payment_event(
    store: &dyn AdminStore,
    payment_event: CommercePaymentEventRecord,
) -> CommerceResult<CommercePaymentEventRecord> {
    match store.upsert_commerce_payment_event(&payment_event).await {
        Ok(event) => Ok(event),
        Err(error) => {
            if let Some(existing_event) = store
                .find_commerce_payment_event_by_dedupe_key(&payment_event.dedupe_key)
                .await
                .map_err(CommerceError::from)?
            {
                if existing_event.order_id != payment_event.order_id {
                    return Err(CommerceError::Conflict(format!(
                        "payment event {} already belongs to order {}",
                        payment_event.dedupe_key, existing_event.order_id
                    )));
                }
            }
            Err(CommerceError::Storage(error))
        }
    }
}
