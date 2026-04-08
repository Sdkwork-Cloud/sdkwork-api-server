use serde::{Deserialize, Serialize};
use std::str::FromStr;
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CommercePaymentEventProcessingStatus {
    Received,
    Processed,
    Ignored,
    Rejected,
    Failed,
}

impl CommercePaymentEventProcessingStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Received => "received",
            Self::Processed => "processed",
            Self::Ignored => "ignored",
            Self::Rejected => "rejected",
            Self::Failed => "failed",
        }
    }
}

impl FromStr for CommercePaymentEventProcessingStatus {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "received" => Ok(Self::Received),
            "processed" => Ok(Self::Processed),
            "ignored" => Ok(Self::Ignored),
            "rejected" => Ok(Self::Rejected),
            "failed" => Ok(Self::Failed),
            other => Err(format!(
                "unknown commerce payment event processing status: {other}"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CommercePaymentEventRecord {
    pub payment_event_id: String,
    pub order_id: String,
    pub project_id: String,
    pub user_id: String,
    pub provider: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_event_id: Option<String>,
    pub dedupe_key: String,
    pub event_type: String,
    pub payload_json: String,
    pub processing_status: CommercePaymentEventProcessingStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub processing_message: Option<String>,
    pub received_at_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub processed_at_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order_status_after: Option<String>,
}

impl CommercePaymentEventRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        payment_event_id: impl Into<String>,
        order_id: impl Into<String>,
        project_id: impl Into<String>,
        user_id: impl Into<String>,
        provider: impl Into<String>,
        dedupe_key: impl Into<String>,
        event_type: impl Into<String>,
        payload_json: impl Into<String>,
        received_at_ms: u64,
    ) -> Self {
        Self {
            payment_event_id: payment_event_id.into(),
            order_id: order_id.into(),
            project_id: project_id.into(),
            user_id: user_id.into(),
            provider: provider.into(),
            provider_event_id: None,
            dedupe_key: dedupe_key.into(),
            event_type: event_type.into(),
            payload_json: payload_json.into(),
            processing_status: CommercePaymentEventProcessingStatus::Received,
            processing_message: None,
            received_at_ms,
            processed_at_ms: None,
            order_status_after: None,
        }
    }

    pub fn with_provider_event_id(mut self, provider_event_id: Option<String>) -> Self {
        self.provider_event_id = provider_event_id;
        self
    }

    pub fn with_processing_status(
        mut self,
        processing_status: CommercePaymentEventProcessingStatus,
    ) -> Self {
        self.processing_status = processing_status;
        self
    }

    pub fn with_processing_message(mut self, processing_message: Option<String>) -> Self {
        self.processing_message = processing_message;
        self
    }

    pub fn with_processed_at_ms(mut self, processed_at_ms: Option<u64>) -> Self {
        self.processed_at_ms = processed_at_ms;
        self
    }

    pub fn with_order_status_after(mut self, order_status_after: Option<String>) -> Self {
        self.order_status_after = order_status_after;
        self
    }
}
