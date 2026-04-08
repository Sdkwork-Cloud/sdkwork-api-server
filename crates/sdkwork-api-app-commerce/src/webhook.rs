use crate::constants::COMMERCE_PAYMENT_PROVIDER_STRIPE;
use crate::error::{CommerceError, CommerceResult};
use crate::payment_attempt::{
    persist_payment_attempt, resolve_payment_attempt_for_provider_reference,
};
use crate::payment_event::apply_portal_commerce_payment_event_with_billing;
use crate::payment_provider::stripe::{self, StripeWebhookEvent};
use crate::payment_provider::{load_payment_method, resolve_payment_method_secret_bundle};
use crate::refund::apply_refund_completion_side_effects;
use crate::types::{PortalCommercePaymentEventRequest, PortalCommerceWebhookAck};
use sdkwork_api_app_billing::CommercialBillingAdminKernel;
use sdkwork_api_app_credential::CredentialSecretManager;
use sdkwork_api_domain_commerce::{
    CommerceRefundRecord, CommerceWebhookDeliveryAttemptRecord, CommerceWebhookInboxRecord,
};
use sdkwork_api_storage_core::AdminStore;

pub async fn process_portal_stripe_webhook(
    store: &dyn AdminStore,
    commercial_billing: Option<&dyn CommercialBillingAdminKernel>,
    secret_manager: &CredentialSecretManager,
    payment_method_id: &str,
    signature_header: Option<&str>,
    payload: &str,
) -> CommerceResult<PortalCommerceWebhookAck> {
    let payment_method = load_payment_method(store, payment_method_id).await?;
    if payment_method.provider != COMMERCE_PAYMENT_PROVIDER_STRIPE {
        return Err(CommerceError::InvalidInput(format!(
            "payment method {} is not configured for stripe webhooks",
            payment_method.payment_method_id
        )));
    }

    let signature_header = signature_header
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            CommerceError::InvalidInput("Stripe-Signature header is required".to_owned())
        })?;
    let secrets = resolve_payment_method_secret_bundle(
        store,
        secret_manager,
        &payment_method.payment_method_id,
    )
    .await?;
    let webhook_secret = secrets.webhook_secret.as_deref().ok_or_else(|| {
        CommerceError::InvalidInput(format!(
            "payment method {} is missing webhook_secret binding",
            payment_method.payment_method_id
        ))
    })?;

    let now_ms = crate::current_time_ms()?;
    stripe::verify_webhook_signature(
        payload,
        signature_header,
        webhook_secret,
        payment_method.webhook_tolerance_seconds,
        now_ms,
    )?;

    let webhook_event = stripe::parse_webhook_event(payload)?;
    let dedupe_key = format!("stripe:{}", webhook_event.event_id());
    let existing_inbox = store
        .find_commerce_webhook_inbox_by_dedupe_key(&dedupe_key)
        .await
        .map_err(CommerceError::from)?;
    let mut inbox = existing_inbox.clone().unwrap_or_else(|| {
        CommerceWebhookInboxRecord::new(
            crate::generate_entity_id("commerce_webhook_inbox")
                .unwrap_or_else(|_| "commerce_webhook_inbox_fallback".to_owned()),
            COMMERCE_PAYMENT_PROVIDER_STRIPE,
            dedupe_key.clone(),
            payload.to_owned(),
            now_ms,
        )
    });
    inbox = inbox
        .with_payment_method_id_option(Some(payment_method.payment_method_id.clone()))
        .with_provider_event_id_option(Some(webhook_event.event_id().to_owned()))
        .with_signature_header_option(Some(signature_header.to_owned()))
        .with_last_received_at_ms(now_ms)
        .with_max_retry_count(payment_method.max_retry_count);
    if !matches!(inbox.processing_status.as_str(), "processed" | "ignored") {
        inbox = inbox
            .with_processing_status("processing")
            .with_last_error_message_option(None)
            .with_next_retry_at_ms_option(None);
    }
    inbox = store
        .upsert_commerce_webhook_inbox(&inbox)
        .await
        .map_err(CommerceError::from)?;

    let started_at_ms = crate::current_time_ms()?;
    let delivery_attempt_id = crate::generate_entity_id("commerce_webhook_delivery_attempt")?;
    let delivery_attempt = CommerceWebhookDeliveryAttemptRecord::new(
        delivery_attempt_id.clone(),
        inbox.webhook_inbox_id.clone(),
        started_at_ms,
    );

    if matches!(inbox.processing_status.as_str(), "processed" | "ignored")
        && existing_inbox.is_some()
    {
        let duplicate_attempt = delivery_attempt
            .with_processing_status("ignored")
            .with_response_code_option(Some(200))
            .with_error_message_option(Some("duplicate webhook delivery ignored".to_owned()))
            .with_finished_at_ms_option(Some(crate::current_time_ms()?));
        let _ = store
            .insert_commerce_webhook_delivery_attempt(&duplicate_attempt)
            .await
            .map_err(CommerceError::from)?;
        return Ok(PortalCommerceWebhookAck {
            webhook_inbox_id: inbox.webhook_inbox_id,
            delivery_attempt_id,
            processing_status: "ignored".to_owned(),
            provider_event_id: Some(webhook_event.event_id().to_owned()),
            order_id: extract_order_id_from_event(&webhook_event),
            payment_attempt_id: extract_payment_attempt_id_from_event(&webhook_event),
        });
    }

    let processing_result: CommerceResult<(String, Option<String>, Option<String>)> =
        match &webhook_event {
            StripeWebhookEvent::CheckoutCompleted {
                order_id,
                project_id,
                user_id,
                payment_attempt_id,
                payment_method_id,
                checkout_session_id,
                payment_intent_id,
                amount_minor,
                ..
            } => {
                let payment_attempt = resolve_payment_attempt_for_provider_reference(
                    store,
                    order_id,
                    payment_attempt_id.as_deref(),
                    Some(checkout_session_id),
                    payment_intent_id.as_deref(),
                )
                .await?;
                if let Some(payment_attempt) = payment_attempt {
                    let updated_attempt = payment_attempt
                        .with_status("succeeded")
                        .with_captured_amount_minor(*amount_minor)
                        .with_provider_checkout_session_id_option(Some(checkout_session_id.clone()))
                        .with_provider_payment_intent_id_option(payment_intent_id.clone())
                        .with_response_payload_json(payload.to_owned())
                        .with_completed_at_ms_option(Some(now_ms))
                        .with_updated_at_ms(now_ms);
                    let _ = persist_payment_attempt(store, &updated_attempt).await?;
                }

                let request = PortalCommercePaymentEventRequest {
                    event_type: "settled".to_owned(),
                    provider: Some(COMMERCE_PAYMENT_PROVIDER_STRIPE.to_owned()),
                    provider_event_id: Some(webhook_event.event_id().to_owned()),
                    checkout_method_id: payment_method_id.clone(),
                    message: Some("stripe checkout session completed".to_owned()),
                };
                let _ = apply_portal_commerce_payment_event_with_billing(
                    store,
                    commercial_billing,
                    user_id,
                    project_id,
                    order_id,
                    &request,
                )
                .await?;
                Ok((
                    "processed".to_owned(),
                    Some(order_id.clone()),
                    payment_attempt_id.clone(),
                ))
            }
            StripeWebhookEvent::CheckoutExpired {
                order_id,
                project_id,
                user_id,
                payment_attempt_id,
                payment_method_id,
                checkout_session_id,
                payment_intent_id,
                ..
            } => {
                let payment_attempt = resolve_payment_attempt_for_provider_reference(
                    store,
                    order_id,
                    payment_attempt_id.as_deref(),
                    Some(checkout_session_id),
                    payment_intent_id.as_deref(),
                )
                .await?;
                if let Some(payment_attempt) = payment_attempt {
                    let updated_attempt = payment_attempt
                        .with_status("expired")
                        .with_provider_checkout_session_id_option(Some(checkout_session_id.clone()))
                        .with_provider_payment_intent_id_option(payment_intent_id.clone())
                        .with_response_payload_json(payload.to_owned())
                        .with_completed_at_ms_option(Some(now_ms))
                        .with_updated_at_ms(now_ms);
                    let _ = persist_payment_attempt(store, &updated_attempt).await?;
                }

                let request = PortalCommercePaymentEventRequest {
                    event_type: "failed".to_owned(),
                    provider: Some(COMMERCE_PAYMENT_PROVIDER_STRIPE.to_owned()),
                    provider_event_id: Some(webhook_event.event_id().to_owned()),
                    checkout_method_id: payment_method_id.clone(),
                    message: Some("stripe checkout session expired".to_owned()),
                };
                let _ = apply_portal_commerce_payment_event_with_billing(
                    store,
                    commercial_billing,
                    user_id,
                    project_id,
                    order_id,
                    &request,
                )
                .await?;
                Ok((
                    "processed".to_owned(),
                    Some(order_id.clone()),
                    payment_attempt_id.clone(),
                ))
            }
            StripeWebhookEvent::PaymentFailed {
                order_id,
                project_id,
                user_id,
                payment_attempt_id,
                payment_method_id,
                payment_intent_id,
                error_message,
                ..
            } => {
                let payment_attempt = resolve_payment_attempt_for_provider_reference(
                    store,
                    order_id,
                    payment_attempt_id.as_deref(),
                    None,
                    payment_intent_id.as_deref(),
                )
                .await?;
                if let Some(payment_attempt) = payment_attempt {
                    let updated_attempt = payment_attempt
                        .with_status("failed")
                        .with_provider_payment_intent_id_option(payment_intent_id.clone())
                        .with_error_message_option(error_message.clone())
                        .with_response_payload_json(payload.to_owned())
                        .with_completed_at_ms_option(Some(now_ms))
                        .with_updated_at_ms(now_ms);
                    let _ = persist_payment_attempt(store, &updated_attempt).await?;
                }

                let request = PortalCommercePaymentEventRequest {
                    event_type: "failed".to_owned(),
                    provider: Some(COMMERCE_PAYMENT_PROVIDER_STRIPE.to_owned()),
                    provider_event_id: Some(webhook_event.event_id().to_owned()),
                    checkout_method_id: payment_method_id.clone(),
                    message: error_message.clone(),
                };
                let _ = apply_portal_commerce_payment_event_with_billing(
                    store,
                    commercial_billing,
                    user_id,
                    project_id,
                    order_id,
                    &request,
                )
                .await?;
                Ok((
                    "processed".to_owned(),
                    Some(order_id.clone()),
                    payment_attempt_id.clone(),
                ))
            }
            StripeWebhookEvent::RefundUpdated {
                local_refund_id,
                provider_refund_id,
                order_id,
                payment_attempt_id,
                amount_minor,
                status,
                ..
            } => {
                let mut refund =
                    load_refund_for_webhook(store, local_refund_id.as_deref(), provider_refund_id)
                        .await?;
                let normalized_status = stripe::normalize_refund_status(status).to_owned();
                let was_succeeded = refund.status == "succeeded";
                refund = refund
                    .with_provider_refund_id_option(Some(provider_refund_id.clone()))
                    .with_status(normalized_status.clone())
                    .with_response_payload_json(payload.to_owned())
                    .with_updated_at_ms(now_ms);
                if normalized_status == "succeeded" {
                    refund = refund.with_completed_at_ms_option(Some(now_ms));
                }
                refund = store
                    .upsert_commerce_refund(&refund)
                    .await
                    .map_err(CommerceError::from)?;

                if normalized_status == "succeeded" && !was_succeeded {
                    let order = store
                        .list_commerce_orders()
                        .await
                        .map_err(CommerceError::from)?
                        .into_iter()
                        .find(|order| order.order_id == refund.order_id)
                        .ok_or_else(|| {
                            CommerceError::NotFound(format!(
                                "order {} referenced by refund {} not found",
                                refund.order_id, refund.refund_id
                            ))
                        })?;
                    let payment_attempt = resolve_payment_attempt_for_provider_reference(
                        store,
                        &order.order_id,
                        refund
                            .payment_attempt_id
                            .as_deref()
                            .or(payment_attempt_id.as_deref()),
                        None,
                        None,
                    )
                    .await?;
                    let _ = apply_refund_completion_side_effects(
                        store,
                        commercial_billing,
                        &order,
                        payment_attempt.as_ref(),
                        *amount_minor,
                        now_ms,
                    )
                    .await?;
                }

                Ok((
                    if normalized_status == "failed" {
                        "ignored".to_owned()
                    } else {
                        "processed".to_owned()
                    },
                    order_id.clone().or(Some(refund.order_id.clone())),
                    payment_attempt_id
                        .clone()
                        .or(refund.payment_attempt_id.clone()),
                ))
            }
            StripeWebhookEvent::Unsupported { .. } => Ok((
                "ignored".to_owned(),
                extract_order_id_from_event(&webhook_event),
                extract_payment_attempt_id_from_event(&webhook_event),
            )),
        };

    match processing_result {
        Ok((processing_status, order_id, payment_attempt_id)) => {
            let finished_at_ms = crate::current_time_ms()?;
            let completed_attempt = delivery_attempt
                .with_processing_status(processing_status.clone())
                .with_response_code_option(Some(200))
                .with_finished_at_ms_option(Some(finished_at_ms));
            let _ = store
                .insert_commerce_webhook_delivery_attempt(&completed_attempt)
                .await
                .map_err(CommerceError::from)?;
            let finalized_inbox = inbox
                .with_processing_status(processing_status.clone())
                .with_processed_at_ms_option(Some(finished_at_ms))
                .with_last_error_message_option(None)
                .with_next_retry_at_ms_option(None);
            let _ = store
                .upsert_commerce_webhook_inbox(&finalized_inbox)
                .await
                .map_err(CommerceError::from)?;
            Ok(PortalCommerceWebhookAck {
                webhook_inbox_id: finalized_inbox.webhook_inbox_id,
                delivery_attempt_id,
                processing_status,
                provider_event_id: Some(webhook_event.event_id().to_owned()),
                order_id,
                payment_attempt_id,
            })
        }
        Err(error) => {
            let retry_count = inbox.retry_count.saturating_add(1);
            let should_dead_letter = retry_count >= inbox.max_retry_count;
            let next_retry_at_ms = (!should_dead_letter)
                .then_some(now_ms.saturating_add(compute_retry_delay_ms(retry_count)));
            let failed_attempt = delivery_attempt
                .with_processing_status("failed")
                .with_response_code_option(Some(500))
                .with_error_message_option(Some(error.to_string()))
                .with_finished_at_ms_option(Some(crate::current_time_ms()?));
            let _ = store
                .insert_commerce_webhook_delivery_attempt(&failed_attempt)
                .await
                .map_err(CommerceError::from)?;
            let failed_inbox = inbox
                .with_processing_status(if should_dead_letter {
                    "dead_letter"
                } else {
                    "retry_scheduled"
                })
                .with_retry_count(retry_count)
                .with_last_error_message_option(Some(error.to_string()))
                .with_next_retry_at_ms_option(next_retry_at_ms);
            let _ = store
                .upsert_commerce_webhook_inbox(&failed_inbox)
                .await
                .map_err(CommerceError::from)?;
            Err(error)
        }
    }
}

async fn load_refund_for_webhook(
    store: &dyn AdminStore,
    local_refund_id: Option<&str>,
    provider_refund_id: &str,
) -> CommerceResult<CommerceRefundRecord> {
    if let Some(local_refund_id) = local_refund_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        if let Some(refund) = store
            .find_commerce_refund(local_refund_id)
            .await
            .map_err(CommerceError::from)?
        {
            return Ok(refund);
        }
    }

    store
        .list_commerce_refunds()
        .await
        .map_err(CommerceError::from)?
        .into_iter()
        .find(|refund| refund.provider_refund_id.as_deref() == Some(provider_refund_id))
        .ok_or_else(|| {
            CommerceError::NotFound(format!(
                "refund referenced by provider_refund_id {} not found",
                provider_refund_id
            ))
        })
}

fn extract_order_id_from_event(webhook_event: &StripeWebhookEvent) -> Option<String> {
    match webhook_event {
        StripeWebhookEvent::CheckoutCompleted { order_id, .. }
        | StripeWebhookEvent::CheckoutExpired { order_id, .. }
        | StripeWebhookEvent::PaymentFailed { order_id, .. } => Some(order_id.clone()),
        StripeWebhookEvent::RefundUpdated { order_id, .. } => order_id.clone(),
        StripeWebhookEvent::Unsupported { .. } => None,
    }
}

fn extract_payment_attempt_id_from_event(webhook_event: &StripeWebhookEvent) -> Option<String> {
    match webhook_event {
        StripeWebhookEvent::CheckoutCompleted {
            payment_attempt_id, ..
        }
        | StripeWebhookEvent::CheckoutExpired {
            payment_attempt_id, ..
        }
        | StripeWebhookEvent::PaymentFailed {
            payment_attempt_id, ..
        }
        | StripeWebhookEvent::RefundUpdated {
            payment_attempt_id, ..
        } => payment_attempt_id.clone(),
        StripeWebhookEvent::Unsupported { .. } => None,
    }
}

fn compute_retry_delay_ms(retry_count: u32) -> u64 {
    let exponent = retry_count.min(8);
    5_000u64.saturating_mul(2u64.saturating_pow(exponent))
}
