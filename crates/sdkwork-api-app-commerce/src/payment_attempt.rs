use crate::constants::COMMERCE_PAYMENT_PROVIDER_STRIPE;
use crate::error::{CommerceError, CommerceResult};
use crate::payment_provider::stripe;
use crate::payment_provider::{
    ensure_payment_method_supports_order, load_payment_method, parse_payment_method_config,
    resolve_payment_method_secret_bundle,
};
use crate::types::PortalCommercePaymentAttemptCreateRequest;
use reqwest::Client;
use sdkwork_api_app_credential::CredentialSecretManager;
use sdkwork_api_domain_commerce::CommercePaymentAttemptRecord;
use sdkwork_api_storage_core::AdminStore;

pub async fn create_portal_commerce_payment_attempt(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    user_id: &str,
    project_id: &str,
    order_id: &str,
    request: &PortalCommercePaymentAttemptCreateRequest,
) -> CommerceResult<CommercePaymentAttemptRecord> {
    let payment_method_id = request.payment_method_id.trim();
    if payment_method_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "payment_method_id is required".to_owned(),
        ));
    }

    let mut order =
        crate::load_project_commerce_order(store, user_id, project_id, order_id).await?;
    if order.payable_price_cents == 0 {
        return Err(CommerceError::Conflict(format!(
            "order {} does not require external payment",
            order.order_id
        )));
    }
    if order.status != "pending_payment" {
        return Err(CommerceError::Conflict(format!(
            "order {} cannot start a payment attempt from status {}",
            order.order_id, order.status
        )));
    }

    let payment_method = load_payment_method(store, payment_method_id).await?;
    ensure_payment_method_supports_order(&payment_method, &order, request.country_code.as_deref())?;

    let idempotency_key = match request
        .idempotency_key
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(idempotency_key) => idempotency_key.to_owned(),
        None => crate::generate_entity_id("commerce_payment_attempt_idem")?,
    };
    if let Some(existing_attempt) = store
        .find_commerce_payment_attempt_by_idempotency_key(&idempotency_key)
        .await
        .map_err(CommerceError::from)?
    {
        if existing_attempt.order_id != order.order_id {
            return Err(CommerceError::Conflict(format!(
                "payment attempt idempotency_key {} already belongs to order {}",
                idempotency_key, existing_attempt.order_id
            )));
        }
        return Ok(existing_attempt);
    }

    let existing_attempts = list_payment_attempts_for_order(store, &order.order_id).await?;
    let attempt_sequence = existing_attempts
        .iter()
        .map(|attempt| attempt.attempt_sequence)
        .max()
        .unwrap_or(0)
        .saturating_add(1);
    let initiated_at_ms = crate::current_time_ms()?;
    let payment_attempt_id = crate::generate_entity_id("commerce_payment_attempt")?;
    let mut payment_attempt = CommercePaymentAttemptRecord::new(
        payment_attempt_id,
        order.order_id.clone(),
        order.project_id.clone(),
        order.user_id.clone(),
        payment_method.payment_method_id.clone(),
        payment_method.provider.clone(),
        payment_method.channel.clone(),
        idempotency_key,
        attempt_sequence,
        order.payable_price_cents,
        order.currency_code.clone(),
        initiated_at_ms,
    )
    .with_status("initiating")
    .with_request_payload_json(
        serde_json::to_string(request).map_err(|error| CommerceError::Storage(error.into()))?,
    )
    .with_updated_at_ms(initiated_at_ms);

    match payment_method.provider.as_str() {
        COMMERCE_PAYMENT_PROVIDER_STRIPE => {
            let payment_method_config = parse_payment_method_config(&payment_method)?;
            let secrets = resolve_payment_method_secret_bundle(
                store,
                secret_manager,
                &payment_method.payment_method_id,
            )
            .await?;
            let client = Client::new();
            let checkout_session = stripe::create_checkout_session(
                &client,
                &secrets.api_secret,
                &payment_method,
                &payment_method_config,
                &order,
                &payment_attempt,
                request,
            )
            .await?;
            payment_attempt = payment_attempt
                .with_status(map_stripe_checkout_status(&checkout_session.status))
                .with_provider_checkout_session_id_option(Some(
                    checkout_session.provider_checkout_session_id,
                ))
                .with_provider_payment_intent_id_option(checkout_session.provider_payment_intent_id)
                .with_provider_reference_option(checkout_session.provider_reference)
                .with_checkout_url_option(checkout_session.checkout_url)
                .with_expires_at_ms_option(checkout_session.expires_at_ms)
                .with_request_payload_json(checkout_session.request_payload_json)
                .with_response_payload_json(checkout_session.response_payload_json)
                .with_updated_at_ms(crate::current_time_ms()?);
        }
        provider => {
            return Err(CommerceError::InvalidInput(format!(
                "payment provider {provider} is not yet wired for real payment attempts"
            )));
        }
    }

    payment_attempt = store
        .upsert_commerce_payment_attempt(&payment_attempt)
        .await
        .map_err(CommerceError::from)?;

    order.payment_method_id = Some(payment_method.payment_method_id);
    order.latest_payment_attempt_id = Some(payment_attempt.payment_attempt_id.clone());
    order.settlement_status = "requires_action".to_owned();
    order.updated_at_ms = crate::current_time_ms()?;
    let _ = store
        .insert_commerce_order(&order)
        .await
        .map_err(CommerceError::from)?;

    Ok(payment_attempt)
}

pub async fn list_portal_commerce_payment_attempts(
    store: &dyn AdminStore,
    user_id: &str,
    project_id: &str,
    order_id: &str,
) -> CommerceResult<Vec<CommercePaymentAttemptRecord>> {
    let order = crate::load_project_commerce_order(store, user_id, project_id, order_id).await?;
    list_payment_attempts_for_order(store, &order.order_id).await
}

pub async fn load_portal_commerce_payment_attempt(
    store: &dyn AdminStore,
    user_id: &str,
    project_id: &str,
    payment_attempt_id: &str,
) -> CommerceResult<CommercePaymentAttemptRecord> {
    let payment_attempt_id = payment_attempt_id.trim();
    if payment_attempt_id.is_empty() {
        return Err(CommerceError::InvalidInput(
            "payment_attempt_id is required".to_owned(),
        ));
    }

    let attempt = store
        .find_commerce_payment_attempt(payment_attempt_id)
        .await
        .map_err(CommerceError::from)?
        .ok_or_else(|| {
            CommerceError::NotFound(format!("payment attempt {payment_attempt_id} not found"))
        })?;
    if attempt.user_id != user_id || attempt.project_id != project_id {
        return Err(CommerceError::NotFound(format!(
            "payment attempt {payment_attempt_id} not found"
        )));
    }

    crate::load_project_commerce_order(store, user_id, project_id, &attempt.order_id).await?;
    Ok(attempt)
}

pub async fn list_payment_attempts_for_order(
    store: &dyn AdminStore,
    order_id: &str,
) -> CommerceResult<Vec<CommercePaymentAttemptRecord>> {
    let mut attempts = store
        .list_commerce_payment_attempts_for_order(order_id)
        .await
        .map_err(CommerceError::from)?;
    attempts.sort_by(|left, right| {
        right
            .initiated_at_ms
            .cmp(&left.initiated_at_ms)
            .then_with(|| right.attempt_sequence.cmp(&left.attempt_sequence))
            .then_with(|| right.payment_attempt_id.cmp(&left.payment_attempt_id))
    });
    Ok(attempts)
}

pub(crate) async fn resolve_payment_attempt_for_provider_reference(
    store: &dyn AdminStore,
    order_id: &str,
    payment_attempt_id: Option<&str>,
    provider_checkout_session_id: Option<&str>,
    provider_payment_intent_id: Option<&str>,
) -> CommerceResult<Option<CommercePaymentAttemptRecord>> {
    if let Some(payment_attempt_id) = payment_attempt_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let attempt = store
            .find_commerce_payment_attempt(payment_attempt_id)
            .await
            .map_err(CommerceError::from)?;
        if let Some(attempt) = attempt {
            if attempt.order_id != order_id {
                return Err(CommerceError::Conflict(format!(
                    "payment attempt {} does not belong to order {}",
                    payment_attempt_id, order_id
                )));
            }
            return Ok(Some(attempt));
        }
    }

    let attempts = list_payment_attempts_for_order(store, order_id).await?;
    Ok(attempts.into_iter().find(|attempt| {
        provider_checkout_session_id
            .and_then(|provider_checkout_session_id| {
                attempt
                    .provider_checkout_session_id
                    .as_deref()
                    .map(|value| value == provider_checkout_session_id)
            })
            .unwrap_or(false)
            || provider_payment_intent_id
                .and_then(|provider_payment_intent_id| {
                    attempt
                        .provider_payment_intent_id
                        .as_deref()
                        .map(|value| value == provider_payment_intent_id)
                })
                .unwrap_or(false)
    }))
}

pub(crate) async fn persist_payment_attempt(
    store: &dyn AdminStore,
    payment_attempt: &CommercePaymentAttemptRecord,
) -> CommerceResult<CommercePaymentAttemptRecord> {
    store
        .upsert_commerce_payment_attempt(payment_attempt)
        .await
        .map_err(CommerceError::from)
}

fn map_stripe_checkout_status(status: &str) -> &'static str {
    match status {
        "open" => "requires_action",
        "complete" => "succeeded",
        "expired" => "expired",
        _ => "pending",
    }
}
