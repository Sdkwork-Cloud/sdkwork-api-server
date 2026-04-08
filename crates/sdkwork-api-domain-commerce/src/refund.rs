use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CommerceRefundRecord {
    pub refund_id: String,
    pub order_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payment_attempt_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payment_method_id: Option<String>,
    pub provider: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_refund_id: Option<String>,
    pub idempotency_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub status: String,
    pub amount_minor: u64,
    pub currency_code: String,
    pub request_payload_json: String,
    pub response_payload_json: String,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at_ms: Option<u64>,
}

impl CommerceRefundRecord {
    pub fn new(
        refund_id: impl Into<String>,
        order_id: impl Into<String>,
        provider: impl Into<String>,
        idempotency_key: impl Into<String>,
        amount_minor: u64,
        currency_code: impl Into<String>,
        created_at_ms: u64,
    ) -> Self {
        Self {
            refund_id: refund_id.into(),
            order_id: order_id.into(),
            payment_attempt_id: None,
            payment_method_id: None,
            provider: provider.into(),
            provider_refund_id: None,
            idempotency_key: idempotency_key.into(),
            reason: None,
            status: "requested".to_owned(),
            amount_minor,
            currency_code: currency_code.into(),
            request_payload_json: "{}".to_owned(),
            response_payload_json: "{}".to_owned(),
            created_at_ms,
            updated_at_ms: created_at_ms,
            completed_at_ms: None,
        }
    }

    pub fn with_payment_attempt_id_option(mut self, payment_attempt_id: Option<String>) -> Self {
        self.payment_attempt_id = payment_attempt_id;
        self
    }

    pub fn with_payment_method_id_option(mut self, payment_method_id: Option<String>) -> Self {
        self.payment_method_id = payment_method_id;
        self
    }

    pub fn with_provider_refund_id_option(mut self, provider_refund_id: Option<String>) -> Self {
        self.provider_refund_id = provider_refund_id;
        self
    }

    pub fn with_reason_option(mut self, reason: Option<String>) -> Self {
        self.reason = reason;
        self
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = status.into();
        self
    }

    pub fn with_request_payload_json(mut self, request_payload_json: impl Into<String>) -> Self {
        self.request_payload_json = request_payload_json.into();
        self
    }

    pub fn with_response_payload_json(
        mut self,
        response_payload_json: impl Into<String>,
    ) -> Self {
        self.response_payload_json = response_payload_json.into();
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }

    pub fn with_completed_at_ms_option(mut self, completed_at_ms: Option<u64>) -> Self {
        self.completed_at_ms = completed_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CommerceReconciliationRunRecord {
    pub reconciliation_run_id: String,
    pub provider: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payment_method_id: Option<String>,
    pub scope_started_at_ms: u64,
    pub scope_ended_at_ms: u64,
    pub status: String,
    pub summary_json: String,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at_ms: Option<u64>,
}

impl CommerceReconciliationRunRecord {
    pub fn new(
        reconciliation_run_id: impl Into<String>,
        provider: impl Into<String>,
        scope_started_at_ms: u64,
        scope_ended_at_ms: u64,
        created_at_ms: u64,
    ) -> Self {
        Self {
            reconciliation_run_id: reconciliation_run_id.into(),
            provider: provider.into(),
            payment_method_id: None,
            scope_started_at_ms,
            scope_ended_at_ms,
            status: "running".to_owned(),
            summary_json: "{}".to_owned(),
            created_at_ms,
            updated_at_ms: created_at_ms,
            completed_at_ms: None,
        }
    }

    pub fn with_payment_method_id_option(mut self, payment_method_id: Option<String>) -> Self {
        self.payment_method_id = payment_method_id;
        self
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = status.into();
        self
    }

    pub fn with_summary_json(mut self, summary_json: impl Into<String>) -> Self {
        self.summary_json = summary_json.into();
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }

    pub fn with_completed_at_ms_option(mut self, completed_at_ms: Option<u64>) -> Self {
        self.completed_at_ms = completed_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CommerceReconciliationItemRecord {
    pub reconciliation_item_id: String,
    pub reconciliation_run_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payment_attempt_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refund_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_reference: Option<String>,
    pub discrepancy_type: String,
    pub status: String,
    pub expected_amount_minor: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_amount_minor: Option<i64>,
    pub detail_json: String,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl CommerceReconciliationItemRecord {
    pub fn new(
        reconciliation_item_id: impl Into<String>,
        reconciliation_run_id: impl Into<String>,
        discrepancy_type: impl Into<String>,
        expected_amount_minor: i64,
        created_at_ms: u64,
    ) -> Self {
        Self {
            reconciliation_item_id: reconciliation_item_id.into(),
            reconciliation_run_id: reconciliation_run_id.into(),
            order_id: None,
            payment_attempt_id: None,
            refund_id: None,
            external_reference: None,
            discrepancy_type: discrepancy_type.into(),
            status: "open".to_owned(),
            expected_amount_minor,
            provider_amount_minor: None,
            detail_json: "{}".to_owned(),
            created_at_ms,
            updated_at_ms: created_at_ms,
        }
    }

    pub fn with_order_id_option(mut self, order_id: Option<String>) -> Self {
        self.order_id = order_id;
        self
    }

    pub fn with_payment_attempt_id_option(mut self, payment_attempt_id: Option<String>) -> Self {
        self.payment_attempt_id = payment_attempt_id;
        self
    }

    pub fn with_refund_id_option(mut self, refund_id: Option<String>) -> Self {
        self.refund_id = refund_id;
        self
    }

    pub fn with_external_reference_option(
        mut self,
        external_reference: Option<String>,
    ) -> Self {
        self.external_reference = external_reference;
        self
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = status.into();
        self
    }

    pub fn with_provider_amount_minor_option(mut self, provider_amount_minor: Option<i64>) -> Self {
        self.provider_amount_minor = provider_amount_minor;
        self
    }

    pub fn with_detail_json(mut self, detail_json: impl Into<String>) -> Self {
        self.detail_json = detail_json.into();
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}
