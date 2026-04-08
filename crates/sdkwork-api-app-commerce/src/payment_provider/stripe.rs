use crate::error::{CommerceError, CommerceResult};
use crate::payment_provider::{PaymentMethodConfig, render_template};
use crate::types::{AdminCommerceRefundCreateRequest, PortalCommercePaymentAttemptCreateRequest};
use hmac::{Hmac, Mac};
use reqwest::Client;
use sdkwork_api_domain_commerce::{
    CommerceOrderRecord, CommercePaymentAttemptRecord, CommerceRefundRecord, PaymentMethodRecord,
};
use serde_json::Value;
use sha2_010::Sha256;

const STRIPE_API_BASE: &str = "https://api.stripe.com/v1";

#[derive(Debug, Clone)]
pub(crate) struct StripeCheckoutSessionCreateResult {
    pub provider_checkout_session_id: String,
    pub provider_payment_intent_id: Option<String>,
    pub checkout_url: Option<String>,
    pub provider_reference: Option<String>,
    pub status: String,
    pub expires_at_ms: Option<u64>,
    pub request_payload_json: String,
    pub response_payload_json: String,
}

#[derive(Debug, Clone)]
pub(crate) struct StripeRefundCreateResult {
    pub provider_refund_id: String,
    pub status: String,
    pub request_payload_json: String,
    pub response_payload_json: String,
}

#[derive(Debug, Clone)]
pub(crate) struct StripeCheckoutSessionSnapshot {
    pub status: String,
    pub payment_status: Option<String>,
    pub amount_total_minor: Option<u64>,
    pub response_payload_json: String,
}

#[derive(Debug, Clone)]
pub(crate) struct StripeRefundSnapshot {
    pub status: String,
    pub amount_minor: u64,
    pub response_payload_json: String,
}

#[derive(Debug, Clone)]
pub(crate) enum StripeWebhookEvent {
    CheckoutCompleted {
        event_id: String,
        order_id: String,
        project_id: String,
        user_id: String,
        payment_attempt_id: Option<String>,
        payment_method_id: Option<String>,
        checkout_session_id: String,
        payment_intent_id: Option<String>,
        amount_minor: u64,
    },
    CheckoutExpired {
        event_id: String,
        order_id: String,
        project_id: String,
        user_id: String,
        payment_attempt_id: Option<String>,
        payment_method_id: Option<String>,
        checkout_session_id: String,
        payment_intent_id: Option<String>,
    },
    PaymentFailed {
        event_id: String,
        order_id: String,
        project_id: String,
        user_id: String,
        payment_attempt_id: Option<String>,
        payment_method_id: Option<String>,
        payment_intent_id: Option<String>,
        error_message: Option<String>,
    },
    RefundUpdated {
        event_id: String,
        order_id: Option<String>,
        payment_attempt_id: Option<String>,
        local_refund_id: Option<String>,
        provider_refund_id: String,
        amount_minor: u64,
        status: String,
    },
    Unsupported {
        event_id: String,
    },
}

impl StripeWebhookEvent {
    pub(crate) fn event_id(&self) -> &str {
        match self {
            Self::CheckoutCompleted { event_id, .. }
            | Self::CheckoutExpired { event_id, .. }
            | Self::PaymentFailed { event_id, .. }
            | Self::RefundUpdated { event_id, .. }
            | Self::Unsupported { event_id, .. } => event_id,
        }
    }
}

pub(crate) async fn create_checkout_session(
    client: &Client,
    api_secret: &str,
    payment_method: &PaymentMethodRecord,
    payment_method_config: &PaymentMethodConfig,
    order: &CommerceOrderRecord,
    attempt: &CommercePaymentAttemptRecord,
    request: &PortalCommercePaymentAttemptCreateRequest,
) -> CommerceResult<StripeCheckoutSessionCreateResult> {
    let success_url = request
        .success_url
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(str::to_owned)
        .or_else(|| payment_method_config.checkout.success_url.clone())
        .map(|template| render_template(&template, order, &attempt.payment_attempt_id))
        .unwrap_or_else(|| {
            format!(
                "https://portal.sdkwork.local/commerce/orders/{}/success?payment_attempt_id={}",
                order.order_id, attempt.payment_attempt_id
            )
        });
    let cancel_url = request
        .cancel_url
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(str::to_owned)
        .or_else(|| payment_method_config.checkout.cancel_url.clone())
        .map(|template| render_template(&template, order, &attempt.payment_attempt_id))
        .unwrap_or_else(|| {
            format!(
                "https://portal.sdkwork.local/commerce/orders/{}/cancel?payment_attempt_id={}",
                order.order_id, attempt.payment_attempt_id
            )
        });
    let product_name = payment_method_config
        .checkout
        .product_name
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(|template| render_template(template, order, &attempt.payment_attempt_id))
        .unwrap_or_else(|| order.target_name.clone());
    let payment_method_types = if payment_method_config
        .checkout
        .payment_method_types
        .is_empty()
    {
        vec!["card".to_owned(), "link".to_owned()]
    } else {
        payment_method_config.checkout.payment_method_types.clone()
    };

    let mut form = vec![
        ("mode".to_owned(), "payment".to_owned()),
        ("success_url".to_owned(), success_url),
        ("cancel_url".to_owned(), cancel_url),
        ("client_reference_id".to_owned(), order.order_id.clone()),
        (
            "line_items[0][price_data][currency]".to_owned(),
            order.currency_code.to_ascii_lowercase(),
        ),
        (
            "line_items[0][price_data][product_data][name]".to_owned(),
            product_name,
        ),
        (
            "line_items[0][price_data][unit_amount]".to_owned(),
            order.payable_price_cents.to_string(),
        ),
        ("line_items[0][quantity]".to_owned(), "1".to_owned()),
        ("metadata[order_id]".to_owned(), order.order_id.clone()),
        ("metadata[project_id]".to_owned(), order.project_id.clone()),
        ("metadata[user_id]".to_owned(), order.user_id.clone()),
        (
            "metadata[payment_attempt_id]".to_owned(),
            attempt.payment_attempt_id.clone(),
        ),
        (
            "metadata[payment_method_id]".to_owned(),
            payment_method.payment_method_id.clone(),
        ),
        (
            "payment_intent_data[metadata][order_id]".to_owned(),
            order.order_id.clone(),
        ),
        (
            "payment_intent_data[metadata][project_id]".to_owned(),
            order.project_id.clone(),
        ),
        (
            "payment_intent_data[metadata][user_id]".to_owned(),
            order.user_id.clone(),
        ),
        (
            "payment_intent_data[metadata][payment_attempt_id]".to_owned(),
            attempt.payment_attempt_id.clone(),
        ),
        (
            "payment_intent_data[metadata][payment_method_id]".to_owned(),
            payment_method.payment_method_id.clone(),
        ),
    ];
    if let Some(customer_email) = request
        .customer_email
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        form.push(("customer_email".to_owned(), customer_email.to_owned()));
    }
    if let Some(customer_creation) = payment_method_config
        .checkout
        .customer_creation
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        form.push(("customer_creation".to_owned(), customer_creation.to_owned()));
    }
    if let Some(expires_in_minutes) = payment_method_config.checkout.expires_in_minutes {
        let expires_at_seconds =
            (attempt.initiated_at_ms / 1_000).saturating_add(expires_in_minutes.saturating_mul(60));
        form.push(("expires_at".to_owned(), expires_at_seconds.to_string()));
    }
    for (index, payment_method_type) in payment_method_types.iter().enumerate() {
        form.push((
            format!("payment_method_types[{index}]"),
            payment_method_type.clone(),
        ));
    }

    let request_payload_json =
        serde_json::to_string(&form).map_err(|error| CommerceError::Storage(error.into()))?;
    let form_body =
        serde_urlencoded::to_string(&form).map_err(|error| CommerceError::Storage(error.into()))?;
    let response = client
        .post(format!("{STRIPE_API_BASE}/checkout/sessions"))
        .bearer_auth(api_secret)
        .header("Idempotency-Key", &attempt.idempotency_key)
        .header(
            reqwest::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(form_body)
        .send()
        .await
        .map_err(|error| CommerceError::Storage(error.into()))?;
    let status_code = response.status();
    let response_text = response
        .text()
        .await
        .map_err(|error| CommerceError::Storage(error.into()))?;
    let payload = parse_json_payload(&response_text)?;
    if !status_code.is_success() {
        return Err(CommerceError::Conflict(
            stripe_error_message(&payload).unwrap_or_else(|| {
                format!("stripe checkout session creation failed with status {status_code}")
            }),
        ));
    }

    let provider_checkout_session_id =
        required_string(&payload, "id", "stripe checkout session id")?.to_owned();

    Ok(StripeCheckoutSessionCreateResult {
        provider_payment_intent_id: optional_string(&payload, "payment_intent").map(str::to_owned),
        checkout_url: optional_string(&payload, "url").map(str::to_owned),
        provider_reference: Some(provider_checkout_session_id.clone()),
        status: optional_string(&payload, "status")
            .unwrap_or("open")
            .to_owned(),
        expires_at_ms: payload
            .get("expires_at")
            .and_then(Value::as_u64)
            .map(|value| value.saturating_mul(1_000)),
        request_payload_json,
        response_payload_json: response_text,
        provider_checkout_session_id,
    })
}

pub(crate) async fn create_refund(
    client: &Client,
    api_secret: &str,
    payment_attempt: &CommercePaymentAttemptRecord,
    refund: &CommerceRefundRecord,
    request: &AdminCommerceRefundCreateRequest,
) -> CommerceResult<StripeRefundCreateResult> {
    let payment_intent_id = payment_attempt
        .provider_payment_intent_id
        .as_deref()
        .or(payment_attempt.provider_reference.as_deref())
        .ok_or_else(|| {
            CommerceError::Conflict(format!(
                "payment attempt {} has no provider payment intent reference",
                payment_attempt.payment_attempt_id
            ))
        })?;

    let mut form = vec![
        ("payment_intent".to_owned(), payment_intent_id.to_owned()),
        ("amount".to_owned(), refund.amount_minor.to_string()),
        ("metadata[order_id]".to_owned(), refund.order_id.clone()),
        ("metadata[refund_id]".to_owned(), refund.refund_id.clone()),
        (
            "metadata[payment_attempt_id]".to_owned(),
            refund.payment_attempt_id.clone().unwrap_or_default(),
        ),
        (
            "metadata[payment_method_id]".to_owned(),
            refund.payment_method_id.clone().unwrap_or_default(),
        ),
    ];
    if let Some(reason) = request
        .reason
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        form.push(("metadata[reason]".to_owned(), reason.to_owned()));
    }

    let request_payload_json =
        serde_json::to_string(&form).map_err(|error| CommerceError::Storage(error.into()))?;
    let form_body =
        serde_urlencoded::to_string(&form).map_err(|error| CommerceError::Storage(error.into()))?;
    let response = client
        .post(format!("{STRIPE_API_BASE}/refunds"))
        .bearer_auth(api_secret)
        .header("Idempotency-Key", &refund.idempotency_key)
        .header(
            reqwest::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(form_body)
        .send()
        .await
        .map_err(|error| CommerceError::Storage(error.into()))?;
    let status_code = response.status();
    let response_text = response
        .text()
        .await
        .map_err(|error| CommerceError::Storage(error.into()))?;
    let payload = parse_json_payload(&response_text)?;
    if !status_code.is_success() {
        return Err(CommerceError::Conflict(
            stripe_error_message(&payload).unwrap_or_else(|| {
                format!("stripe refund creation failed with status {status_code}")
            }),
        ));
    }

    Ok(StripeRefundCreateResult {
        provider_refund_id: required_string(&payload, "id", "stripe refund id")?.to_owned(),
        status: optional_string(&payload, "status")
            .unwrap_or("pending")
            .to_owned(),
        request_payload_json,
        response_payload_json: response_text,
    })
}

pub(crate) async fn retrieve_checkout_session(
    client: &Client,
    api_secret: &str,
    provider_checkout_session_id: &str,
) -> CommerceResult<StripeCheckoutSessionSnapshot> {
    let response = client
        .get(format!(
            "{STRIPE_API_BASE}/checkout/sessions/{provider_checkout_session_id}"
        ))
        .bearer_auth(api_secret)
        .send()
        .await
        .map_err(|error| CommerceError::Storage(error.into()))?;
    let status_code = response.status();
    let response_text = response
        .text()
        .await
        .map_err(|error| CommerceError::Storage(error.into()))?;
    let payload = parse_json_payload(&response_text)?;
    if !status_code.is_success() {
        return Err(CommerceError::Conflict(
            stripe_error_message(&payload).unwrap_or_else(|| {
                format!("stripe checkout session lookup failed with status {status_code}")
            }),
        ));
    }

    Ok(StripeCheckoutSessionSnapshot {
        status: optional_string(&payload, "status")
            .unwrap_or("unknown")
            .to_owned(),
        payment_status: optional_string(&payload, "payment_status").map(str::to_owned),
        amount_total_minor: payload.get("amount_total").and_then(Value::as_u64),
        response_payload_json: response_text,
    })
}

pub(crate) async fn retrieve_refund(
    client: &Client,
    api_secret: &str,
    provider_refund_id: &str,
) -> CommerceResult<StripeRefundSnapshot> {
    let response = client
        .get(format!("{STRIPE_API_BASE}/refunds/{provider_refund_id}"))
        .bearer_auth(api_secret)
        .send()
        .await
        .map_err(|error| CommerceError::Storage(error.into()))?;
    let status_code = response.status();
    let response_text = response
        .text()
        .await
        .map_err(|error| CommerceError::Storage(error.into()))?;
    let payload = parse_json_payload(&response_text)?;
    if !status_code.is_success() {
        return Err(CommerceError::Conflict(
            stripe_error_message(&payload).unwrap_or_else(|| {
                format!("stripe refund lookup failed with status {status_code}")
            }),
        ));
    }

    Ok(StripeRefundSnapshot {
        status: optional_string(&payload, "status")
            .unwrap_or("unknown")
            .to_owned(),
        amount_minor: payload.get("amount").and_then(Value::as_u64).unwrap_or(0),
        response_payload_json: response_text,
    })
}

pub(crate) fn verify_webhook_signature(
    payload: &str,
    signature_header: &str,
    webhook_secret: &str,
    tolerance_seconds: u64,
    now_ms: u64,
) -> CommerceResult<()> {
    let mut timestamp: Option<u64> = None;
    let mut signatures = Vec::new();

    for segment in signature_header.split(',') {
        let Some((key, value)) = segment.trim().split_once('=') else {
            continue;
        };
        match key.trim() {
            "t" => {
                timestamp = value.trim().parse::<u64>().ok();
            }
            "v1" => signatures.push(value.trim().to_owned()),
            _ => {}
        }
    }

    let timestamp = timestamp.ok_or_else(|| {
        CommerceError::InvalidInput("stripe webhook signature timestamp is missing".to_owned())
    })?;
    if tolerance_seconds > 0 {
        let now_seconds = now_ms / 1_000;
        let delta = now_seconds.abs_diff(timestamp);
        if delta > tolerance_seconds {
            return Err(CommerceError::Conflict(
                "stripe webhook signature is outside the configured tolerance window".to_owned(),
            ));
        }
    }
    if signatures.is_empty() {
        return Err(CommerceError::InvalidInput(
            "stripe webhook v1 signature is missing".to_owned(),
        ));
    }

    let signed_payload = format!("{timestamp}.{payload}");
    let mut mac = Hmac::<Sha256>::new_from_slice(webhook_secret.as_bytes())
        .map_err(|error| CommerceError::Storage(error.into()))?;
    mac.update(signed_payload.as_bytes());
    let computed_signature = encode_hex(&mac.finalize().into_bytes());

    if signatures
        .iter()
        .any(|signature| signature.eq_ignore_ascii_case(&computed_signature))
    {
        return Ok(());
    }

    Err(CommerceError::Conflict(
        "stripe webhook signature verification failed".to_owned(),
    ))
}

pub(crate) fn parse_webhook_event(payload: &str) -> CommerceResult<StripeWebhookEvent> {
    let envelope = parse_json_payload(payload)?;
    let event_id = required_string(&envelope, "id", "stripe event id")?.to_owned();
    let event_type = required_string(&envelope, "type", "stripe event type")?.to_owned();
    let object = envelope
        .get("data")
        .and_then(|data| data.get("object"))
        .cloned()
        .unwrap_or(Value::Null);
    let metadata = object
        .get("metadata")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let order_id = metadata
        .get("order_id")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let project_id = metadata
        .get("project_id")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let user_id = metadata
        .get("user_id")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let payment_attempt_id = metadata
        .get("payment_attempt_id")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let payment_method_id = metadata
        .get("payment_method_id")
        .and_then(Value::as_str)
        .map(str::to_owned);

    match event_type.as_str() {
        "checkout.session.completed" => Ok(StripeWebhookEvent::CheckoutCompleted {
            event_id,
            order_id: required_owned(order_id, "stripe webhook metadata.order_id")?,
            project_id: required_owned(project_id, "stripe webhook metadata.project_id")?,
            user_id: required_owned(user_id, "stripe webhook metadata.user_id")?,
            payment_attempt_id,
            payment_method_id,
            checkout_session_id: required_string(&object, "id", "stripe checkout session id")?
                .to_owned(),
            payment_intent_id: optional_string(&object, "payment_intent").map(str::to_owned),
            amount_minor: object
                .get("amount_total")
                .and_then(Value::as_u64)
                .unwrap_or(0),
        }),
        "checkout.session.expired" => Ok(StripeWebhookEvent::CheckoutExpired {
            event_id,
            order_id: required_owned(order_id, "stripe webhook metadata.order_id")?,
            project_id: required_owned(project_id, "stripe webhook metadata.project_id")?,
            user_id: required_owned(user_id, "stripe webhook metadata.user_id")?,
            payment_attempt_id,
            payment_method_id,
            checkout_session_id: required_string(&object, "id", "stripe checkout session id")?
                .to_owned(),
            payment_intent_id: optional_string(&object, "payment_intent").map(str::to_owned),
        }),
        "payment_intent.payment_failed" => {
            let error_message = object
                .get("last_payment_error")
                .and_then(|value| value.get("message"))
                .and_then(Value::as_str)
                .map(str::to_owned);
            Ok(StripeWebhookEvent::PaymentFailed {
                event_id,
                order_id: required_owned(order_id, "stripe webhook metadata.order_id")?,
                project_id: required_owned(project_id, "stripe webhook metadata.project_id")?,
                user_id: required_owned(user_id, "stripe webhook metadata.user_id")?,
                payment_attempt_id,
                payment_method_id,
                payment_intent_id: optional_string(&object, "id").map(str::to_owned),
                error_message,
            })
        }
        "refund.updated" => Ok(StripeWebhookEvent::RefundUpdated {
            event_id,
            order_id,
            payment_attempt_id,
            local_refund_id: metadata
                .get("refund_id")
                .and_then(Value::as_str)
                .map(str::to_owned),
            provider_refund_id: required_string(&object, "id", "stripe refund id")?.to_owned(),
            amount_minor: object.get("amount").and_then(Value::as_u64).unwrap_or(0),
            status: optional_string(&object, "status")
                .unwrap_or("unknown")
                .to_owned(),
        }),
        _ => Ok(StripeWebhookEvent::Unsupported { event_id }),
    }
}

pub(crate) fn normalize_refund_status(status: &str) -> &str {
    match status {
        "succeeded" => "succeeded",
        "failed" | "canceled" => "failed",
        _ => "pending",
    }
}

fn parse_json_payload(payload: &str) -> CommerceResult<Value> {
    serde_json::from_str(payload).map_err(|error| CommerceError::Storage(error.into()))
}

fn stripe_error_message(payload: &Value) -> Option<String> {
    payload
        .get("error")
        .and_then(|error| error.get("message"))
        .and_then(Value::as_str)
        .map(str::to_owned)
}

fn required_string<'a>(payload: &'a Value, key: &str, label: &str) -> CommerceResult<&'a str> {
    optional_string(payload, key).ok_or_else(|| {
        CommerceError::InvalidInput(format!("{label} is missing from stripe payload"))
    })
}

fn optional_string<'a>(payload: &'a Value, key: &str) -> Option<&'a str> {
    payload.get(key).and_then(Value::as_str)
}

fn required_owned(value: Option<String>, label: &str) -> CommerceResult<String> {
    value.ok_or_else(|| CommerceError::InvalidInput(format!("{label} is required")))
}

fn encode_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(nibble_to_hex((byte >> 4) & 0x0f));
        output.push(nibble_to_hex(byte & 0x0f));
    }
    output
}

fn nibble_to_hex(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        10..=15 => (b'a' + (value - 10)) as char,
        _ => '0',
    }
}
