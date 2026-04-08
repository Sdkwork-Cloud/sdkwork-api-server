use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct PaymentMethodRecord {
    pub payment_method_id: String,
    pub display_name: String,
    pub description: String,
    pub provider: String,
    pub channel: String,
    pub mode: String,
    pub enabled: bool,
    pub sort_order: i32,
    pub capability_codes: Vec<String>,
    pub supported_currency_codes: Vec<String>,
    pub supported_country_codes: Vec<String>,
    pub supported_order_kinds: Vec<String>,
    pub callback_strategy: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub webhook_path: Option<String>,
    pub webhook_tolerance_seconds: u64,
    pub replay_window_seconds: u64,
    pub max_retry_count: u32,
    pub config_json: String,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl PaymentMethodRecord {
    pub fn new(
        payment_method_id: impl Into<String>,
        display_name: impl Into<String>,
        provider: impl Into<String>,
        channel: impl Into<String>,
        created_at_ms: u64,
    ) -> Self {
        Self {
            payment_method_id: payment_method_id.into(),
            display_name: display_name.into(),
            description: String::new(),
            provider: provider.into(),
            channel: channel.into(),
            mode: "live".to_owned(),
            enabled: true,
            sort_order: 0,
            capability_codes: vec!["checkout".to_owned()],
            supported_currency_codes: Vec::new(),
            supported_country_codes: Vec::new(),
            supported_order_kinds: Vec::new(),
            callback_strategy: "webhook_signed".to_owned(),
            webhook_path: None,
            webhook_tolerance_seconds: 300,
            replay_window_seconds: 300,
            max_retry_count: 8,
            config_json: "{}".to_owned(),
            created_at_ms,
            updated_at_ms: created_at_ms,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn with_mode(mut self, mode: impl Into<String>) -> Self {
        self.mode = mode.into();
        self
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn with_sort_order(mut self, sort_order: i32) -> Self {
        self.sort_order = sort_order;
        self
    }

    pub fn with_capability_codes(mut self, capability_codes: Vec<String>) -> Self {
        self.capability_codes = capability_codes;
        self
    }

    pub fn with_supported_currency_codes(
        mut self,
        supported_currency_codes: Vec<String>,
    ) -> Self {
        self.supported_currency_codes = supported_currency_codes;
        self
    }

    pub fn with_supported_country_codes(
        mut self,
        supported_country_codes: Vec<String>,
    ) -> Self {
        self.supported_country_codes = supported_country_codes;
        self
    }

    pub fn with_supported_order_kinds(mut self, supported_order_kinds: Vec<String>) -> Self {
        self.supported_order_kinds = supported_order_kinds;
        self
    }

    pub fn with_callback_strategy(mut self, callback_strategy: impl Into<String>) -> Self {
        self.callback_strategy = callback_strategy.into();
        self
    }

    pub fn with_webhook_path_option(mut self, webhook_path: Option<String>) -> Self {
        self.webhook_path = webhook_path;
        self
    }

    pub fn with_webhook_tolerance_seconds(mut self, webhook_tolerance_seconds: u64) -> Self {
        self.webhook_tolerance_seconds = webhook_tolerance_seconds;
        self
    }

    pub fn with_replay_window_seconds(mut self, replay_window_seconds: u64) -> Self {
        self.replay_window_seconds = replay_window_seconds;
        self
    }

    pub fn with_max_retry_count(mut self, max_retry_count: u32) -> Self {
        self.max_retry_count = max_retry_count;
        self
    }

    pub fn with_config_json(mut self, config_json: impl Into<String>) -> Self {
        self.config_json = config_json.into();
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct PaymentMethodCredentialBindingRecord {
    pub binding_id: String,
    pub payment_method_id: String,
    pub usage_kind: String,
    pub credential_tenant_id: String,
    pub credential_provider_id: String,
    pub credential_key_reference: String,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl PaymentMethodCredentialBindingRecord {
    pub fn new(
        binding_id: impl Into<String>,
        payment_method_id: impl Into<String>,
        usage_kind: impl Into<String>,
        credential_tenant_id: impl Into<String>,
        credential_provider_id: impl Into<String>,
        credential_key_reference: impl Into<String>,
        created_at_ms: u64,
    ) -> Self {
        Self {
            binding_id: binding_id.into(),
            payment_method_id: payment_method_id.into(),
            usage_kind: usage_kind.into(),
            credential_tenant_id: credential_tenant_id.into(),
            credential_provider_id: credential_provider_id.into(),
            credential_key_reference: credential_key_reference.into(),
            created_at_ms,
            updated_at_ms: created_at_ms,
        }
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CommercePaymentAttemptRecord {
    pub payment_attempt_id: String,
    pub order_id: String,
    pub project_id: String,
    pub user_id: String,
    pub payment_method_id: String,
    pub provider: String,
    pub channel: String,
    pub status: String,
    pub idempotency_key: String,
    pub attempt_sequence: u32,
    pub amount_minor: u64,
    pub currency_code: String,
    pub captured_amount_minor: u64,
    pub refunded_amount_minor: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_payment_intent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_checkout_session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_reference: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkout_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub qr_code_payload: Option<String>,
    pub request_payload_json: String,
    pub response_payload_json: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    pub initiated_at_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at_ms: Option<u64>,
    pub updated_at_ms: u64,
}

impl CommercePaymentAttemptRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        payment_attempt_id: impl Into<String>,
        order_id: impl Into<String>,
        project_id: impl Into<String>,
        user_id: impl Into<String>,
        payment_method_id: impl Into<String>,
        provider: impl Into<String>,
        channel: impl Into<String>,
        idempotency_key: impl Into<String>,
        attempt_sequence: u32,
        amount_minor: u64,
        currency_code: impl Into<String>,
        initiated_at_ms: u64,
    ) -> Self {
        Self {
            payment_attempt_id: payment_attempt_id.into(),
            order_id: order_id.into(),
            project_id: project_id.into(),
            user_id: user_id.into(),
            payment_method_id: payment_method_id.into(),
            provider: provider.into(),
            channel: channel.into(),
            status: "created".to_owned(),
            idempotency_key: idempotency_key.into(),
            attempt_sequence,
            amount_minor,
            currency_code: currency_code.into(),
            captured_amount_minor: 0,
            refunded_amount_minor: 0,
            provider_payment_intent_id: None,
            provider_checkout_session_id: None,
            provider_reference: None,
            checkout_url: None,
            qr_code_payload: None,
            request_payload_json: "{}".to_owned(),
            response_payload_json: "{}".to_owned(),
            error_code: None,
            error_message: None,
            initiated_at_ms,
            expires_at_ms: None,
            completed_at_ms: None,
            updated_at_ms: initiated_at_ms,
        }
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = status.into();
        self
    }

    pub fn with_captured_amount_minor(mut self, captured_amount_minor: u64) -> Self {
        self.captured_amount_minor = captured_amount_minor;
        self
    }

    pub fn with_refunded_amount_minor(mut self, refunded_amount_minor: u64) -> Self {
        self.refunded_amount_minor = refunded_amount_minor;
        self
    }

    pub fn with_provider_payment_intent_id_option(
        mut self,
        provider_payment_intent_id: Option<String>,
    ) -> Self {
        self.provider_payment_intent_id = provider_payment_intent_id;
        self
    }

    pub fn with_provider_checkout_session_id_option(
        mut self,
        provider_checkout_session_id: Option<String>,
    ) -> Self {
        self.provider_checkout_session_id = provider_checkout_session_id;
        self
    }

    pub fn with_provider_reference_option(mut self, provider_reference: Option<String>) -> Self {
        self.provider_reference = provider_reference;
        self
    }

    pub fn with_checkout_url_option(mut self, checkout_url: Option<String>) -> Self {
        self.checkout_url = checkout_url;
        self
    }

    pub fn with_qr_code_payload_option(mut self, qr_code_payload: Option<String>) -> Self {
        self.qr_code_payload = qr_code_payload;
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

    pub fn with_error_code_option(mut self, error_code: Option<String>) -> Self {
        self.error_code = error_code;
        self
    }

    pub fn with_error_message_option(mut self, error_message: Option<String>) -> Self {
        self.error_message = error_message;
        self
    }

    pub fn with_expires_at_ms_option(mut self, expires_at_ms: Option<u64>) -> Self {
        self.expires_at_ms = expires_at_ms;
        self
    }

    pub fn with_completed_at_ms_option(mut self, completed_at_ms: Option<u64>) -> Self {
        self.completed_at_ms = completed_at_ms;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}
