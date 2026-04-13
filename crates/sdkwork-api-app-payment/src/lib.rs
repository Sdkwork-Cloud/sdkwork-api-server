use anyhow::{anyhow, Result};
use reqwest::StatusCode;
use sdkwork_api_app_commerce::{
    apply_portal_commerce_payment_event, CommerceError, PortalCommerceOrderRecord,
    PortalCommercePaymentEventRequest,
};
use sdkwork_api_domain_commerce::CommerceOrderRecord;
use sdkwork_api_domain_payment::{
    PaymentAttemptRecord, PaymentOrderRecord, PaymentWebhookEventRecord,
};
use sdkwork_api_observability::{
    annotate_current_http_metrics, record_current_commercial_event,
    record_current_payment_callback, CommercialEventDimensions, CommercialEventKind,
    PaymentMetricDimensions,
};
use sdkwork_api_storage_core::AdminStore;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

type PaymentResult<T> = std::result::Result<T, PaymentError>;

const ALIPAY_PROVIDER: &str = "alipay";
const STRIPE_PROVIDER: &str = "stripe";
const WECHATPAY_PROVIDER: &str = "wechatpay";
const STRIPE_SIGNATURE_HEADER: &str = "stripe-signature";
const WECHATPAY_SIGNATURE_HEADER: &str = "wechatpay-signature";
const WECHATPAY_TIMESTAMP_HEADER: &str = "wechatpay-timestamp";
const WECHATPAY_NONCE_HEADER: &str = "wechatpay-nonce";
const STRIPE_SIGNATURE_TOLERANCE_SECS: u64 = 300;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlipayPaymentConfig {
    pub notify_secret: String,
}

impl AlipayPaymentConfig {
    pub fn from_env() -> Option<Self> {
        Some(Self {
            notify_secret: optional_env_value("SDKWORK_ALIPAY_NOTIFY_SECRET")?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StripePaymentConfig {
    pub api_base_url: String,
    pub secret_key: String,
    pub webhook_secret: String,
    pub return_base_url: String,
}

impl StripePaymentConfig {
    pub fn from_env() -> Option<Self> {
        let api_base_url = optional_env_value("SDKWORK_STRIPE_API_BASE_URL")?;
        let secret_key = optional_env_value("SDKWORK_STRIPE_SECRET_KEY")?;
        let webhook_secret = optional_env_value("SDKWORK_STRIPE_WEBHOOK_SECRET")?;
        let return_base_url = optional_env_value("SDKWORK_PORTAL_CHECKOUT_RETURN_BASE_URL")?;
        Some(Self {
            api_base_url,
            secret_key,
            webhook_secret,
            return_base_url,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeChatPayPaymentConfig {
    pub notify_secret: String,
}

impl WeChatPayPaymentConfig {
    pub fn from_env() -> Option<Self> {
        Some(Self {
            notify_secret: optional_env_value("SDKWORK_WECHATPAY_NOTIFY_SECRET")?,
        })
    }
}

#[derive(Debug)]
pub enum PaymentError {
    InvalidInput(String),
    Unauthorized(String),
    NotFound(String),
    Conflict(String),
    Storage(anyhow::Error),
}

impl std::fmt::Display for PaymentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput(message) => write!(f, "{message}"),
            Self::Unauthorized(message) => write!(f, "{message}"),
            Self::NotFound(message) => write!(f, "{message}"),
            Self::Conflict(message) => write!(f, "{message}"),
            Self::Storage(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for PaymentError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error.as_ref()),
            _ => None,
        }
    }
}

impl From<anyhow::Error> for PaymentError {
    fn from(value: anyhow::Error) -> Self {
        Self::Storage(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StripeCheckoutSessionProjection {
    pub payment_order_id: String,
    pub provider_reference_id: String,
    pub checkout_url: String,
}

pub async fn provision_stripe_checkout_session(
    store: &dyn AdminStore,
    order: &CommerceOrderRecord,
    config: &StripePaymentConfig,
) -> PaymentResult<StripeCheckoutSessionProjection> {
    if order.payable_price_cents == 0 {
        return Err(PaymentError::InvalidInput(
            "stripe checkout requires a paid order".to_owned(),
        ));
    }
    if order.status != "pending_payment" {
        return Err(PaymentError::Conflict(format!(
            "order {} cannot provision checkout from status {}",
            order.order_id, order.status
        )));
    }

    if let Some(existing) = store
        .find_payment_order_record_by_commerce_order_id(&order.order_id)
        .await
        .map_err(PaymentError::from)?
    {
        if existing.provider != STRIPE_PROVIDER {
            return Err(PaymentError::Conflict(format!(
                "commerce order {} is already bound to payment provider {}",
                order.order_id, existing.provider
            )));
        }
        return Ok(StripeCheckoutSessionProjection {
            payment_order_id: existing.payment_order_id,
            provider_reference_id: existing.provider_reference_id,
            checkout_url: existing.checkout_url,
        });
    }

    let payment_order_id = generate_entity_id("payment_order")?;
    let request_body = json!({
        "mode": "payment",
        "amount_cents": order.payable_price_cents,
        "currency": "usd",
        "success_url": format!(
            "{}/billing/checkout/success?order_id={}",
            config.return_base_url, order.order_id
        ),
        "cancel_url": format!(
            "{}/billing/checkout/cancel?order_id={}",
            config.return_base_url, order.order_id
        ),
        "client_reference_id": order.order_id,
        "metadata": {
            "order_id": order.order_id,
            "payment_order_id": payment_order_id
        }
    });
    let response = reqwest::Client::new()
        .post(format!(
            "{}/v1/checkout/sessions",
            config.api_base_url.trim_end_matches('/')
        ))
        .bearer_auth(&config.secret_key)
        .json(&request_body)
        .send()
        .await
        .map_err(|error| PaymentError::Storage(error.into()))?;
    if response.status() != StatusCode::OK {
        return Err(PaymentError::Storage(anyhow!(
            "stripe checkout session creation failed with status {}",
            response.status()
        )));
    }
    let response_json: Value = response
        .json()
        .await
        .map_err(|error| PaymentError::Storage(error.into()))?;
    let provider_reference_id = response_json["id"]
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            PaymentError::Storage(anyhow!("stripe checkout response missing session id"))
        })?
        .to_owned();
    let checkout_url = response_json["url"]
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            PaymentError::Storage(anyhow!("stripe checkout response missing checkout url"))
        })?
        .to_owned();
    let created_at_ms = current_time_millis()?;
    let payment_order = PaymentOrderRecord::new(
        &payment_order_id,
        &order.order_id,
        &order.project_id,
        &order.user_id,
        STRIPE_PROVIDER,
        "usd",
        order.payable_price_cents,
        "checkout_open",
        created_at_ms,
    )
    .with_provider_reference_id(&provider_reference_id)
    .with_checkout_url(&checkout_url);
    let saved = store
        .insert_payment_order_record(&payment_order)
        .await
        .map_err(PaymentError::from)?;
    Ok(StripeCheckoutSessionProjection {
        payment_order_id: saved.payment_order_id,
        provider_reference_id: saved.provider_reference_id,
        checkout_url: saved.checkout_url,
    })
}

pub fn stripe_signature_header_name() -> &'static str {
    STRIPE_SIGNATURE_HEADER
}

pub fn wechatpay_signature_header_name() -> &'static str {
    WECHATPAY_SIGNATURE_HEADER
}

pub fn wechatpay_timestamp_header_name() -> &'static str {
    WECHATPAY_TIMESTAMP_HEADER
}

pub fn wechatpay_nonce_header_name() -> &'static str {
    WECHATPAY_NONCE_HEADER
}

pub async fn apply_alipay_notification(
    store: &dyn AdminStore,
    config: &AlipayPaymentConfig,
    payload: &str,
) -> PaymentResult<PortalCommerceOrderRecord> {
    let mut fields = parse_form_body(payload)?;
    let signature = take_required_form_field(&mut fields, "sign", "missing alipay signature")?;
    let sign_type = fields
        .get("sign_type")
        .map(String::as_str)
        .unwrap_or("HMAC-SHA256");
    if sign_type != "HMAC-SHA256" {
        return Err(PaymentError::InvalidInput(format!(
            "unsupported alipay sign_type: {sign_type}"
        )));
    }
    verify_alipay_signature(&fields, &signature, &config.notify_secret)?;

    let provider_event_id = required_form_field(&fields, "notify_id", "alipay notify_id")?;
    if let Some(existing) =
        load_processed_webhook_result(store, ALIPAY_PROVIDER, provider_event_id).await?
    {
        return Ok(existing);
    }

    let event_type = required_form_field(&fields, "trade_status", "alipay trade_status")?;
    let payment_order_id = required_form_field(&fields, "out_trade_no", "alipay out_trade_no")?;
    let provider_reference_id = required_form_field(&fields, "trade_no", "alipay trade_no")?;
    let (commerce_event_type, next_payment_status) = match event_type {
        "TRADE_SUCCESS" | "TRADE_FINISHED" => ("settled", "settled"),
        "TRADE_CLOSED" => ("canceled", "canceled"),
        "TRADE_FAILED" => ("failed", "failed"),
        other => {
            return Err(PaymentError::InvalidInput(format!(
                "unsupported alipay trade_status: {other}"
            )))
        }
    };

    let payment_order = resolve_payment_order_for_provider(
        store,
        ALIPAY_PROVIDER,
        Some(payment_order_id),
        None,
        Some(provider_reference_id),
    )
    .await?;
    apply_payment_callback_outcome(
        store,
        ALIPAY_PROVIDER,
        provider_event_id,
        event_type,
        payload,
        payment_order,
        provider_reference_id,
        next_payment_status,
        commerce_event_type,
    )
    .await
}

pub async fn apply_stripe_webhook(
    store: &dyn AdminStore,
    config: &StripePaymentConfig,
    signature_header: &str,
    payload: &str,
) -> PaymentResult<PortalCommerceOrderRecord> {
    let signature = signature_header.trim();
    if signature.is_empty() {
        return Err(PaymentError::Unauthorized(
            "missing stripe signature".to_owned(),
        ));
    }
    verify_stripe_signature(signature, payload, &config.webhook_secret)?;

    let event: Value = serde_json::from_str(payload)
        .map_err(|error| PaymentError::InvalidInput(error.to_string()))?;
    let provider_event_id = event["id"]
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| PaymentError::InvalidInput("stripe event id is required".to_owned()))?;
    if let Some(existing) =
        load_processed_webhook_result(store, STRIPE_PROVIDER, provider_event_id).await?
    {
        return Ok(existing);
    }

    let event_type = event["type"]
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| PaymentError::InvalidInput("stripe event type is required".to_owned()))?;
    let event_object = event
        .get("data")
        .and_then(|value| value.get("object"))
        .and_then(Value::as_object)
        .ok_or_else(|| PaymentError::InvalidInput("stripe event object is required".to_owned()))?;
    let provider_reference_id = event_object
        .get("id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| PaymentError::InvalidInput("stripe session id is required".to_owned()))?;
    let metadata = event_object.get("metadata").and_then(Value::as_object);
    let payment_order_id = metadata
        .and_then(|value| value.get("payment_order_id"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let commerce_order_id = metadata
        .and_then(|value| value.get("order_id"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty());

    let payment_order = resolve_payment_order_for_provider(
        store,
        STRIPE_PROVIDER,
        payment_order_id,
        commerce_order_id,
        Some(provider_reference_id),
    )
    .await?;
    let commerce_event_type = match event_type {
        "checkout.session.completed" => "settled",
        "checkout.session.expired" => "canceled",
        "checkout.session.async_payment_failed" | "payment_intent.payment_failed" => "failed",
        other => {
            return Err(PaymentError::InvalidInput(format!(
                "unsupported stripe event type: {other}"
            )))
        }
    };
    let next_payment_status = match commerce_event_type {
        "settled" => "settled",
        "canceled" => "canceled",
        "failed" => "failed",
        _ => "received",
    };
    apply_payment_callback_outcome(
        store,
        STRIPE_PROVIDER,
        provider_event_id,
        event_type,
        payload,
        payment_order,
        provider_reference_id,
        next_payment_status,
        commerce_event_type,
    )
    .await
}

pub async fn apply_wechatpay_notification(
    store: &dyn AdminStore,
    config: &WeChatPayPaymentConfig,
    signature_header: &str,
    timestamp_header: &str,
    nonce_header: &str,
    payload: &str,
) -> PaymentResult<PortalCommerceOrderRecord> {
    verify_wechatpay_signature(
        signature_header,
        timestamp_header,
        nonce_header,
        payload,
        &config.notify_secret,
    )?;

    let event: Value = serde_json::from_str(payload)
        .map_err(|error| PaymentError::InvalidInput(error.to_string()))?;
    let provider_event_id = event["id"]
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| PaymentError::InvalidInput("wechatpay event id is required".to_owned()))?;
    if let Some(existing) =
        load_processed_webhook_result(store, WECHATPAY_PROVIDER, provider_event_id).await?
    {
        return Ok(existing);
    }

    let event_type = event["event_type"]
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| PaymentError::InvalidInput("wechatpay event_type is required".to_owned()))?;
    let resource = event
        .get("resource")
        .and_then(Value::as_object)
        .ok_or_else(|| PaymentError::InvalidInput("wechatpay resource is required".to_owned()))?;
    let payment_order_id = resource
        .get("out_trade_no")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            PaymentError::InvalidInput("wechatpay resource.out_trade_no is required".to_owned())
        })?;
    let provider_reference_id = resource
        .get("transaction_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            PaymentError::InvalidInput("wechatpay resource.transaction_id is required".to_owned())
        })?;
    let trade_state = resource
        .get("trade_state")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(event_type);
    let (commerce_event_type, next_payment_status) = match (event_type, trade_state) {
        ("TRANSACTION.SUCCESS", _) | (_, "SUCCESS") => ("settled", "settled"),
        ("TRANSACTION.CLOSED", _) | (_, "CLOSED") => ("canceled", "canceled"),
        ("TRANSACTION.FAIL", _) | (_, "PAYERROR") => ("failed", "failed"),
        _ => {
            return Err(PaymentError::InvalidInput(format!(
            "unsupported wechatpay event_type/trade_state combination: {event_type}/{trade_state}"
        )))
        }
    };

    let payment_order = resolve_payment_order_for_provider(
        store,
        WECHATPAY_PROVIDER,
        Some(payment_order_id),
        None,
        Some(provider_reference_id),
    )
    .await?;
    apply_payment_callback_outcome(
        store,
        WECHATPAY_PROVIDER,
        provider_event_id,
        event_type,
        payload,
        payment_order,
        provider_reference_id,
        next_payment_status,
        commerce_event_type,
    )
    .await
}

fn verify_stripe_signature(
    signature_header: &str,
    payload: &str,
    secret: &str,
) -> PaymentResult<()> {
    let mut timestamp = None;
    let mut signatures = Vec::new();
    for part in signature_header.split(',') {
        let part = part.trim();
        if let Some(value) = part.strip_prefix("t=") {
            timestamp = value.parse::<u64>().ok();
        } else if let Some(value) = part.strip_prefix("v1=") {
            let value = value.trim();
            if !value.is_empty() {
                signatures.push(value.to_owned());
            }
        }
    }

    let timestamp = timestamp.ok_or_else(|| {
        PaymentError::Unauthorized("stripe signature timestamp is required".to_owned())
    })?;
    if signatures.is_empty() {
        return Err(PaymentError::Unauthorized(
            "stripe signature digest is required".to_owned(),
        ));
    }
    let now = current_time_secs()?;
    if !timestamp_within_tolerance(now, timestamp, STRIPE_SIGNATURE_TOLERANCE_SECS) {
        return Err(PaymentError::Unauthorized(
            "stripe signature timestamp is outside the allowed tolerance".to_owned(),
        ));
    }

    let signed_payload = format!("{timestamp}.{payload}");
    let expected_signature = hmac_sha256_hex(secret.as_bytes(), signed_payload.as_bytes());
    if signatures
        .iter()
        .any(|signature| constant_time_eq(signature.as_bytes(), expected_signature.as_bytes()))
    {
        Ok(())
    } else {
        Err(PaymentError::Unauthorized(
            "invalid stripe signature".to_owned(),
        ))
    }
}

fn verify_alipay_signature(
    fields: &BTreeMap<String, String>,
    signature: &str,
    secret: &str,
) -> PaymentResult<()> {
    if signature.trim().is_empty() {
        return Err(PaymentError::Unauthorized(
            "missing alipay signature".to_owned(),
        ));
    }
    let expected_signature = hmac_sha256_hex(
        secret.as_bytes(),
        canonical_form_signature_payload(fields).as_bytes(),
    );
    if constant_time_eq(signature.trim().as_bytes(), expected_signature.as_bytes()) {
        Ok(())
    } else {
        Err(PaymentError::Unauthorized(
            "invalid alipay signature".to_owned(),
        ))
    }
}

fn verify_wechatpay_signature(
    signature_header: &str,
    timestamp_header: &str,
    nonce_header: &str,
    payload: &str,
    secret: &str,
) -> PaymentResult<()> {
    let signature = signature_header.trim();
    if signature.is_empty() {
        return Err(PaymentError::Unauthorized(
            "missing wechatpay signature".to_owned(),
        ));
    }
    let timestamp = timestamp_header
        .trim()
        .parse::<u64>()
        .map_err(|_| PaymentError::Unauthorized("wechatpay timestamp is required".to_owned()))?;
    let nonce = nonce_header.trim();
    if nonce.is_empty() {
        return Err(PaymentError::Unauthorized(
            "wechatpay nonce is required".to_owned(),
        ));
    }
    let now = current_time_secs()?;
    if !timestamp_within_tolerance(now, timestamp, STRIPE_SIGNATURE_TOLERANCE_SECS) {
        return Err(PaymentError::Unauthorized(
            "wechatpay signature timestamp is outside the allowed tolerance".to_owned(),
        ));
    }

    let signed_payload = format!("{timestamp}\n{nonce}\n{payload}\n");
    let expected_signature = hmac_sha256_hex(secret.as_bytes(), signed_payload.as_bytes());
    if constant_time_eq(signature.as_bytes(), expected_signature.as_bytes()) {
        Ok(())
    } else {
        Err(PaymentError::Unauthorized(
            "invalid wechatpay signature".to_owned(),
        ))
    }
}

async fn resolve_payment_order_for_provider(
    store: &dyn AdminStore,
    provider: &str,
    payment_order_id: Option<&str>,
    commerce_order_id: Option<&str>,
    provider_reference_id: Option<&str>,
) -> PaymentResult<PaymentOrderRecord> {
    if let Some(payment_order_id) = payment_order_id {
        if let Some(record) = store
            .find_payment_order_record(payment_order_id)
            .await
            .map_err(PaymentError::from)?
        {
            return ensure_payment_order_provider(record, provider);
        }
    }
    if let Some(commerce_order_id) = commerce_order_id {
        if let Some(record) = store
            .find_payment_order_record_by_commerce_order_id(commerce_order_id)
            .await
            .map_err(PaymentError::from)?
        {
            return ensure_payment_order_provider(record, provider);
        }
    }
    if let Some(provider_reference_id) = provider_reference_id {
        if let Some(record) = store
            .find_payment_order_record_by_provider_reference(provider, provider_reference_id)
            .await
            .map_err(PaymentError::from)?
        {
            return ensure_payment_order_provider(record, provider);
        }
    }

    let payment_hint = payment_order_id
        .map(str::to_owned)
        .or_else(|| commerce_order_id.map(str::to_owned))
        .or_else(|| provider_reference_id.map(str::to_owned))
        .unwrap_or_else(|| "unknown".to_owned());
    Err(PaymentError::NotFound(format!(
        "payment order not found for provider {provider} and reference {payment_hint}"
    )))
}

fn ensure_payment_order_provider(
    record: PaymentOrderRecord,
    provider: &str,
) -> PaymentResult<PaymentOrderRecord> {
    if record.provider == provider {
        Ok(record)
    } else {
        Err(PaymentError::Conflict(format!(
            "payment order {} belongs to provider {}",
            record.payment_order_id, record.provider
        )))
    }
}

async fn load_processed_webhook_result(
    store: &dyn AdminStore,
    provider: &str,
    provider_event_id: &str,
) -> PaymentResult<Option<PortalCommerceOrderRecord>> {
    let Some(existing) = store
        .find_payment_webhook_event_record(provider, provider_event_id)
        .await
        .map_err(PaymentError::from)?
    else {
        return Ok(None);
    };
    if existing.status != "processed" {
        return Ok(None);
    }
    let order_id = existing.commerce_order_id.ok_or_else(|| {
        PaymentError::Storage(anyhow!(
            "{provider} webhook {provider_event_id} was persisted without commerce order linkage"
        ))
    })?;
    let order = load_portal_commerce_order(store, &order_id).await?;
    record_payment_callback_observability(provider, &order, "duplicate");
    record_current_commercial_event(
        CommercialEventKind::CallbackReplay,
        CommercialEventDimensions::default()
            .with_tenant(order.project_id.clone())
            .with_provider(provider.to_owned())
            .with_payment_outcome("duplicate")
            .with_result("ignored"),
    );
    Ok(Some(order))
}

async fn apply_payment_callback_outcome(
    store: &dyn AdminStore,
    provider: &str,
    provider_event_id: &str,
    event_type: &str,
    payload: &str,
    payment_order: PaymentOrderRecord,
    provider_reference_id: &str,
    next_payment_status: &str,
    commerce_event_type: &str,
) -> PaymentResult<PortalCommerceOrderRecord> {
    let updated_payment_order = payment_order
        .clone()
        .with_provider_reference_id(provider_reference_id)
        .with_status(next_payment_status)
        .with_updated_at_ms(current_time_millis()?);
    let updated_payment_order = store
        .insert_payment_order_record(&updated_payment_order)
        .await
        .map_err(PaymentError::from)?;
    let (attempt_kind, attempt_status) = match commerce_event_type {
        "settled" => ("capture", "succeeded"),
        "canceled" => ("cancel", "canceled"),
        "failed" => ("capture", "failed"),
        other => {
            return Err(PaymentError::InvalidInput(format!(
                "unsupported payment callback commerce_event_type: {other}"
            )))
        }
    };
    let payment_attempt = PaymentAttemptRecord::new(
        deterministic_payment_attempt_id(
            provider,
            &updated_payment_order.payment_order_id,
            provider_reference_id,
        ),
        &updated_payment_order.payment_order_id,
        provider,
        provider_reference_id,
        attempt_kind,
        attempt_status,
        &updated_payment_order.currency_code,
        updated_payment_order.amount_cents,
        current_time_millis()?,
    )
    .with_idempotency_key(provider_event_id)
    .with_updated_at_ms(current_time_millis()?);
    store
        .insert_payment_attempt_record(&payment_attempt)
        .await
        .map_err(PaymentError::from)?;

    let webhook_event_id = generate_entity_id("payment_webhook_event")?;
    let webhook_event = PaymentWebhookEventRecord::new(
        &webhook_event_id,
        provider,
        provider_event_id,
        event_type,
        payload,
        current_time_millis()?,
    )
    .with_payment_order_id(updated_payment_order.payment_order_id.clone())
    .with_commerce_order_id(updated_payment_order.commerce_order_id.clone())
    .with_status("processing");
    store
        .insert_payment_webhook_event_record(&webhook_event)
        .await
        .map_err(PaymentError::from)?;

    let order = apply_portal_commerce_payment_event(
        store,
        &updated_payment_order.commerce_order_id,
        &PortalCommercePaymentEventRequest {
            event_type: commerce_event_type.to_owned(),
            payment_order_id: Some(updated_payment_order.payment_order_id.clone()),
        },
    )
    .await
    .map_err(map_commerce_error)?;

    store
        .insert_payment_webhook_event_record(&webhook_event.with_status("processed"))
        .await
        .map_err(PaymentError::from)?;
    record_payment_callback_observability(provider, &order, commerce_event_type);
    Ok(order)
}

fn record_payment_callback_observability(
    provider: &str,
    order: &PortalCommerceOrderRecord,
    payment_outcome: &str,
) {
    annotate_current_http_metrics(|dimensions| {
        dimensions.tenant = Some(order.project_id.clone());
        dimensions.provider = Some(provider.to_owned());
        dimensions.payment_outcome = Some(payment_outcome.to_owned());
    });
    record_current_payment_callback(
        PaymentMetricDimensions::default()
            .with_provider(provider.to_owned())
            .with_tenant(order.project_id.clone())
            .with_payment_outcome(payment_outcome.to_owned()),
    );
}

fn parse_form_body(payload: &str) -> PaymentResult<BTreeMap<String, String>> {
    let mut fields = BTreeMap::new();
    for pair in payload.split('&') {
        let pair = pair.trim();
        if pair.is_empty() {
            continue;
        }
        let (raw_key, raw_value) = pair.split_once('=').unwrap_or((pair, ""));
        let key = raw_key.trim();
        if key.is_empty() {
            return Err(PaymentError::InvalidInput(
                "form field name is required".to_owned(),
            ));
        }
        fields.insert(key.to_owned(), raw_value.trim().to_owned());
    }
    Ok(fields)
}

fn take_required_form_field(
    fields: &mut BTreeMap<String, String>,
    key: &str,
    message: &str,
) -> PaymentResult<String> {
    fields
        .remove(key)
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| PaymentError::Unauthorized(message.to_owned()))
}

fn required_form_field<'a>(
    fields: &'a BTreeMap<String, String>,
    key: &str,
    label: &str,
) -> PaymentResult<&'a str> {
    fields
        .get(key)
        .map(String::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| PaymentError::InvalidInput(format!("{label} is required")))
}

fn canonical_form_signature_payload(fields: &BTreeMap<String, String>) -> String {
    fields
        .iter()
        .filter_map(|(key, value)| {
            let value = value.trim();
            if key == "sign" || value.is_empty() {
                None
            } else {
                Some(format!("{key}={value}"))
            }
        })
        .collect::<Vec<_>>()
        .join("&")
}

fn timestamp_within_tolerance(now: u64, timestamp: u64, tolerance_secs: u64) -> bool {
    now.abs_diff(timestamp) <= tolerance_secs
}

fn deterministic_payment_attempt_id(
    provider: &str,
    payment_order_id: &str,
    provider_reference_id: &str,
) -> String {
    format!(
        "payment_attempt_{}_{}_{}",
        sanitize_identifier_component(provider),
        sanitize_identifier_component(payment_order_id),
        sanitize_identifier_component(provider_reference_id)
    )
}

fn sanitize_identifier_component(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

async fn load_portal_commerce_order(
    store: &dyn AdminStore,
    commerce_order_id: &str,
) -> PaymentResult<PortalCommerceOrderRecord> {
    store
        .list_commerce_orders()
        .await
        .map_err(PaymentError::from)?
        .into_iter()
        .find(|order| order.order_id == commerce_order_id)
        .ok_or_else(|| {
            PaymentError::NotFound(format!("commerce order {commerce_order_id} not found"))
        })
}

fn map_commerce_error(error: CommerceError) -> PaymentError {
    match error {
        CommerceError::InvalidInput(message) => PaymentError::InvalidInput(message),
        CommerceError::NotFound(message) => PaymentError::NotFound(message),
        CommerceError::Conflict(message) => PaymentError::Conflict(message),
        CommerceError::Forbidden(message) => PaymentError::Conflict(message),
        CommerceError::Storage(error) => PaymentError::Storage(error),
    }
}

fn optional_env_value(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn generate_entity_id(prefix: &str) -> Result<String> {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| anyhow!("system clock error"))?
        .as_nanos();
    Ok(format!("{prefix}_{nonce:x}"))
}

fn current_time_secs() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| anyhow!("system clock error"))?
        .as_secs())
}

fn current_time_millis() -> Result<u64> {
    Ok(u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| anyhow!("system clock error"))?
            .as_millis(),
    )?)
}

fn hmac_sha256_hex(key: &[u8], message: &[u8]) -> String {
    let block_size = 64;
    let mut normalized_key = if key.len() > block_size {
        Sha256::digest(key).to_vec()
    } else {
        key.to_vec()
    };
    normalized_key.resize(block_size, 0);

    let mut outer = vec![0x5c; block_size];
    let mut inner = vec![0x36; block_size];
    for (index, key_byte) in normalized_key.iter().enumerate() {
        outer[index] ^= key_byte;
        inner[index] ^= key_byte;
    }

    let mut inner_hasher = Sha256::new();
    inner_hasher.update(&inner);
    inner_hasher.update(message);
    let inner_hash = inner_hasher.finalize();

    let mut outer_hasher = Sha256::new();
    outer_hasher.update(&outer);
    outer_hasher.update(inner_hash);
    format!("{:x}", outer_hasher.finalize())
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut diff = 0_u8;
    for (left_byte, right_byte) in left.iter().zip(right.iter()) {
        diff |= left_byte ^ right_byte;
    }
    diff == 0
}
