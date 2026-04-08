use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CommerceWebhookInboxRecord {
    pub webhook_inbox_id: String,
    pub provider: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payment_method_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_event_id: Option<String>,
    pub dedupe_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature_header: Option<String>,
    pub payload_json: String,
    pub processing_status: String,
    pub retry_count: u32,
    pub max_retry_count: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error_message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_retry_at_ms: Option<u64>,
    pub first_received_at_ms: u64,
    pub last_received_at_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub processed_at_ms: Option<u64>,
}

impl CommerceWebhookInboxRecord {
    pub fn new(
        webhook_inbox_id: impl Into<String>,
        provider: impl Into<String>,
        dedupe_key: impl Into<String>,
        payload_json: impl Into<String>,
        first_received_at_ms: u64,
    ) -> Self {
        Self {
            webhook_inbox_id: webhook_inbox_id.into(),
            provider: provider.into(),
            payment_method_id: None,
            provider_event_id: None,
            dedupe_key: dedupe_key.into(),
            signature_header: None,
            payload_json: payload_json.into(),
            processing_status: "received".to_owned(),
            retry_count: 0,
            max_retry_count: 8,
            last_error_message: None,
            next_retry_at_ms: None,
            first_received_at_ms,
            last_received_at_ms: first_received_at_ms,
            processed_at_ms: None,
        }
    }

    pub fn with_payment_method_id_option(mut self, payment_method_id: Option<String>) -> Self {
        self.payment_method_id = payment_method_id;
        self
    }

    pub fn with_provider_event_id_option(mut self, provider_event_id: Option<String>) -> Self {
        self.provider_event_id = provider_event_id;
        self
    }

    pub fn with_signature_header_option(mut self, signature_header: Option<String>) -> Self {
        self.signature_header = signature_header;
        self
    }

    pub fn with_processing_status(mut self, processing_status: impl Into<String>) -> Self {
        self.processing_status = processing_status.into();
        self
    }

    pub fn with_retry_count(mut self, retry_count: u32) -> Self {
        self.retry_count = retry_count;
        self
    }

    pub fn with_max_retry_count(mut self, max_retry_count: u32) -> Self {
        self.max_retry_count = max_retry_count;
        self
    }

    pub fn with_last_error_message_option(
        mut self,
        last_error_message: Option<String>,
    ) -> Self {
        self.last_error_message = last_error_message;
        self
    }

    pub fn with_next_retry_at_ms_option(mut self, next_retry_at_ms: Option<u64>) -> Self {
        self.next_retry_at_ms = next_retry_at_ms;
        self
    }

    pub fn with_last_received_at_ms(mut self, last_received_at_ms: u64) -> Self {
        self.last_received_at_ms = last_received_at_ms;
        self
    }

    pub fn with_processed_at_ms_option(mut self, processed_at_ms: Option<u64>) -> Self {
        self.processed_at_ms = processed_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CommerceWebhookDeliveryAttemptRecord {
    pub delivery_attempt_id: String,
    pub webhook_inbox_id: String,
    pub processing_status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_code: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    pub started_at_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished_at_ms: Option<u64>,
}

impl CommerceWebhookDeliveryAttemptRecord {
    pub fn new(
        delivery_attempt_id: impl Into<String>,
        webhook_inbox_id: impl Into<String>,
        started_at_ms: u64,
    ) -> Self {
        Self {
            delivery_attempt_id: delivery_attempt_id.into(),
            webhook_inbox_id: webhook_inbox_id.into(),
            processing_status: "processing".to_owned(),
            response_code: None,
            error_message: None,
            started_at_ms,
            finished_at_ms: None,
        }
    }

    pub fn with_processing_status(mut self, processing_status: impl Into<String>) -> Self {
        self.processing_status = processing_status.into();
        self
    }

    pub fn with_response_code_option(mut self, response_code: Option<u16>) -> Self {
        self.response_code = response_code;
        self
    }

    pub fn with_error_message_option(mut self, error_message: Option<String>) -> Self {
        self.error_message = error_message;
        self
    }

    pub fn with_finished_at_ms_option(mut self, finished_at_ms: Option<u64>) -> Self {
        self.finished_at_ms = finished_at_ms;
        self
    }
}
