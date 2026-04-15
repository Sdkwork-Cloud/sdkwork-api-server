use super::*;
use crate::gateway_commercial::current_billing_timestamp_ms;
use sdkwork_api_app_payment::{
    ingest_payment_callback, PaymentCallbackIntakeDisposition, PaymentCallbackIntakeRequest,
    PaymentCallbackNormalizedOutcome, PaymentSubjectScope,
};
use sdkwork_api_domain_payment::PaymentProviderCode;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PaymentCallbackHttpRequest {
    tenant_id: u64,
    #[serde(default)]
    organization_id: u64,
    #[serde(default)]
    user_id: u64,
    event_type: String,
    event_id: String,
    dedupe_key: String,
    #[serde(default)]
    payment_order_id: Option<String>,
    #[serde(default)]
    payment_attempt_id: Option<String>,
    #[serde(default)]
    provider_transaction_id: Option<String>,
    #[serde(default)]
    signature_status: Option<String>,
    #[serde(default)]
    provider_status: Option<String>,
    #[serde(default)]
    currency_code: Option<String>,
    #[serde(default)]
    amount_minor: Option<u64>,
    #[serde(default)]
    fee_minor: Option<u64>,
    #[serde(default)]
    net_amount_minor: Option<u64>,
    #[serde(default)]
    payload_json: Option<String>,
    #[serde(default)]
    received_at_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
struct PaymentCallbackHttpResponse {
    disposition: PaymentCallbackIntakeDisposition,
    normalized_outcome: Option<PaymentCallbackNormalizedOutcome>,
    callback_event_id: String,
    processing_status: String,
    signature_status: String,
    payment_order_id: Option<String>,
    payment_attempt_id: Option<String>,
    payment_session_id: Option<String>,
    payment_transaction_id: Option<String>,
}

pub(crate) async fn payment_callbacks_with_state_handler(
    Path((provider_code, gateway_account_id)): Path<(String, String)>,
    State(state): State<GatewayApiState>,
    ExtractJson(payload): ExtractJson<PaymentCallbackHttpRequest>,
) -> Response {
    let Some(payment_store) = state.payment_store_snapshot() else {
        return payment_callback_error_response(
            StatusCode::SERVICE_UNAVAILABLE,
            "gateway state does not expose canonical payment callback processing",
        );
    };

    let provider_code = match PaymentProviderCode::from_str(provider_code.as_str()) {
        Ok(provider_code) => provider_code,
        Err(error) => return payment_callback_error_response(StatusCode::BAD_REQUEST, error),
    };

    let intake = PaymentCallbackIntakeRequest::new(
        PaymentSubjectScope::new(payload.tenant_id, payload.organization_id, payload.user_id),
        provider_code,
        gateway_account_id,
        payload.event_type,
        payload.event_id,
        payload.dedupe_key,
        payload
            .received_at_ms
            .unwrap_or_else(|| current_billing_timestamp_ms().unwrap_or(0)),
    )
    .with_payment_order_id(payload.payment_order_id)
    .with_payment_attempt_id(payload.payment_attempt_id)
    .with_provider_transaction_id(payload.provider_transaction_id)
    .with_signature_status(
        payload
            .signature_status
            .unwrap_or_else(|| "pending".to_owned()),
    )
    .with_provider_status(payload.provider_status)
    .with_currency_code(payload.currency_code)
    .with_amount_minor(payload.amount_minor)
    .with_fee_minor(payload.fee_minor)
    .with_net_amount_minor(payload.net_amount_minor)
    .with_payload_json(payload.payload_json);

    let result = match ingest_payment_callback(payment_store.as_ref(), &intake).await {
        Ok(result) => result,
        Err(error) => {
            let message = error.to_string();
            let status = if message.contains("payment_order_id")
                || message.contains("payment order not found")
            {
                StatusCode::ACCEPTED
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            return payment_callback_error_response(status, message);
        }
    };

    let response = PaymentCallbackHttpResponse {
        disposition: result.disposition,
        normalized_outcome: result.normalized_outcome,
        callback_event_id: result.callback_event.callback_event_id,
        processing_status: result.callback_event.processing_status.as_str().to_owned(),
        signature_status: result.callback_event.signature_status,
        payment_order_id: result
            .payment_order_opt
            .as_ref()
            .map(|payment_order| payment_order.payment_order_id.clone())
            .or(result.callback_event.payment_order_id),
        payment_attempt_id: result
            .payment_attempt_opt
            .as_ref()
            .map(|payment_attempt| payment_attempt.payment_attempt_id.clone())
            .or(result.callback_event.payment_attempt_id),
        payment_session_id: result
            .payment_session_opt
            .as_ref()
            .map(|payment_session| payment_session.payment_session_id.clone()),
        payment_transaction_id: result
            .payment_transaction_opt
            .as_ref()
            .map(|payment_transaction| payment_transaction.payment_transaction_id.clone()),
    };

    (
        payment_callback_status_code(result.disposition),
        Json(response),
    )
        .into_response()
}

fn payment_callback_status_code(disposition: PaymentCallbackIntakeDisposition) -> StatusCode {
    match disposition {
        PaymentCallbackIntakeDisposition::RequiresProviderQuery => StatusCode::ACCEPTED,
        _ => StatusCode::OK,
    }
}

fn payment_callback_error_response(status: StatusCode, message: impl Into<String>) -> Response {
    (status, Json(serde_json::json!({ "error": message.into() }))).into_response()
}
