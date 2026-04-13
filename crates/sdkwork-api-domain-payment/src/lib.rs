use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaymentOrderRecord {
    pub payment_order_id: String,
    pub commerce_order_id: String,
    pub project_id: String,
    pub user_id: String,
    pub provider: String,
    pub currency_code: String,
    pub amount_cents: u64,
    pub status: String,
    pub provider_reference_id: String,
    pub checkout_url: String,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl PaymentOrderRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        payment_order_id: impl Into<String>,
        commerce_order_id: impl Into<String>,
        project_id: impl Into<String>,
        user_id: impl Into<String>,
        provider: impl Into<String>,
        currency_code: impl Into<String>,
        amount_cents: u64,
        status: impl Into<String>,
        created_at_ms: u64,
    ) -> Self {
        Self {
            payment_order_id: payment_order_id.into(),
            commerce_order_id: commerce_order_id.into(),
            project_id: project_id.into(),
            user_id: user_id.into(),
            provider: provider.into(),
            currency_code: currency_code.into(),
            amount_cents,
            status: status.into(),
            provider_reference_id: String::new(),
            checkout_url: String::new(),
            created_at_ms,
            updated_at_ms: created_at_ms,
        }
    }

    pub fn with_provider_reference_id(mut self, provider_reference_id: impl Into<String>) -> Self {
        self.provider_reference_id = provider_reference_id.into();
        self
    }

    pub fn with_checkout_url(mut self, checkout_url: impl Into<String>) -> Self {
        self.checkout_url = checkout_url.into();
        self
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = status.into();
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaymentWebhookEventRecord {
    pub payment_webhook_event_id: String,
    pub provider: String,
    pub provider_event_id: String,
    pub payment_order_id: Option<String>,
    pub commerce_order_id: Option<String>,
    pub event_type: String,
    pub status: String,
    pub payload_json: String,
    pub created_at_ms: u64,
}

impl PaymentWebhookEventRecord {
    pub fn new(
        payment_webhook_event_id: impl Into<String>,
        provider: impl Into<String>,
        provider_event_id: impl Into<String>,
        event_type: impl Into<String>,
        payload_json: impl Into<String>,
        created_at_ms: u64,
    ) -> Self {
        Self {
            payment_webhook_event_id: payment_webhook_event_id.into(),
            provider: provider.into(),
            provider_event_id: provider_event_id.into(),
            payment_order_id: None,
            commerce_order_id: None,
            event_type: event_type.into(),
            status: "received".to_owned(),
            payload_json: payload_json.into(),
            created_at_ms,
        }
    }

    pub fn with_payment_order_id(mut self, payment_order_id: impl Into<String>) -> Self {
        self.payment_order_id = Some(payment_order_id.into());
        self
    }

    pub fn with_payment_order_id_option(mut self, payment_order_id: Option<String>) -> Self {
        self.payment_order_id = payment_order_id;
        self
    }

    pub fn with_commerce_order_id(mut self, commerce_order_id: impl Into<String>) -> Self {
        self.commerce_order_id = Some(commerce_order_id.into());
        self
    }

    pub fn with_commerce_order_id_option(mut self, commerce_order_id: Option<String>) -> Self {
        self.commerce_order_id = commerce_order_id;
        self
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = status.into();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaymentAttemptRecord {
    pub payment_attempt_id: String,
    pub payment_order_id: String,
    pub provider: String,
    pub provider_attempt_id: String,
    pub attempt_kind: String,
    pub status: String,
    pub currency_code: String,
    pub amount_cents: u64,
    pub idempotency_key: Option<String>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl PaymentAttemptRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        payment_attempt_id: impl Into<String>,
        payment_order_id: impl Into<String>,
        provider: impl Into<String>,
        provider_attempt_id: impl Into<String>,
        attempt_kind: impl Into<String>,
        status: impl Into<String>,
        currency_code: impl Into<String>,
        amount_cents: u64,
        created_at_ms: u64,
    ) -> Self {
        Self {
            payment_attempt_id: payment_attempt_id.into(),
            payment_order_id: payment_order_id.into(),
            provider: provider.into(),
            provider_attempt_id: provider_attempt_id.into(),
            attempt_kind: attempt_kind.into(),
            status: status.into(),
            currency_code: currency_code.into(),
            amount_cents,
            idempotency_key: None,
            error_code: None,
            error_message: None,
            created_at_ms,
            updated_at_ms: created_at_ms,
        }
    }

    pub fn with_idempotency_key(mut self, idempotency_key: impl Into<String>) -> Self {
        self.idempotency_key = Some(idempotency_key.into());
        self
    }

    pub fn with_error_code(mut self, error_code: impl Into<String>) -> Self {
        self.error_code = Some(error_code.into());
        self
    }

    pub fn with_error_message(mut self, error_message: impl Into<String>) -> Self {
        self.error_message = Some(error_message.into());
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefundRecord {
    pub refund_id: String,
    pub payment_order_id: String,
    pub provider: String,
    pub provider_refund_id: String,
    pub status: String,
    pub currency_code: String,
    pub amount_cents: u64,
    pub reason: Option<String>,
    pub failure_code: Option<String>,
    pub failure_message: Option<String>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl RefundRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        refund_id: impl Into<String>,
        payment_order_id: impl Into<String>,
        provider: impl Into<String>,
        provider_refund_id: impl Into<String>,
        status: impl Into<String>,
        currency_code: impl Into<String>,
        amount_cents: u64,
        created_at_ms: u64,
    ) -> Self {
        Self {
            refund_id: refund_id.into(),
            payment_order_id: payment_order_id.into(),
            provider: provider.into(),
            provider_refund_id: provider_refund_id.into(),
            status: status.into(),
            currency_code: currency_code.into(),
            amount_cents,
            reason: None,
            failure_code: None,
            failure_message: None,
            created_at_ms,
            updated_at_ms: created_at_ms,
        }
    }

    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    pub fn with_failure_code(mut self, failure_code: impl Into<String>) -> Self {
        self.failure_code = Some(failure_code.into());
        self
    }

    pub fn with_failure_message(mut self, failure_message: impl Into<String>) -> Self {
        self.failure_message = Some(failure_message.into());
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisputeRecord {
    pub dispute_id: String,
    pub payment_order_id: String,
    pub provider: String,
    pub provider_dispute_id: String,
    pub status: String,
    pub reason: String,
    pub currency_code: String,
    pub amount_cents: u64,
    pub evidence_due_at_ms: Option<u64>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl DisputeRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        dispute_id: impl Into<String>,
        payment_order_id: impl Into<String>,
        provider: impl Into<String>,
        provider_dispute_id: impl Into<String>,
        status: impl Into<String>,
        reason: impl Into<String>,
        currency_code: impl Into<String>,
        amount_cents: u64,
        created_at_ms: u64,
    ) -> Self {
        Self {
            dispute_id: dispute_id.into(),
            payment_order_id: payment_order_id.into(),
            provider: provider.into(),
            provider_dispute_id: provider_dispute_id.into(),
            status: status.into(),
            reason: reason.into(),
            currency_code: currency_code.into(),
            amount_cents,
            evidence_due_at_ms: None,
            created_at_ms,
            updated_at_ms: created_at_ms,
        }
    }

    pub fn with_evidence_due_at_ms(mut self, evidence_due_at_ms: Option<u64>) -> Self {
        self.evidence_due_at_ms = evidence_due_at_ms;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}
