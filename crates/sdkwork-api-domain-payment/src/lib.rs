use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentProviderCode {
    Unspecified,
    Stripe,
    WeChatPay,
    Alipay,
}

impl PaymentProviderCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unspecified => "unspecified",
            Self::Stripe => "stripe",
            Self::WeChatPay => "wechat_pay",
            Self::Alipay => "alipay",
        }
    }
}

impl Default for PaymentProviderCode {
    fn default() -> Self {
        Self::Unspecified
    }
}

impl FromStr for PaymentProviderCode {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "unspecified" => Ok(Self::Unspecified),
            "stripe" => Ok(Self::Stripe),
            "wechat_pay" => Ok(Self::WeChatPay),
            "alipay" => Ok(Self::Alipay),
            other => Err(format!("unknown payment provider code: {other}")),
        }
    }
}

macro_rules! impl_payment_enum_str {
    ($ty:ty, $label:literal, { $($variant:ident => $value:literal),+ $(,)? }) => {
        impl $ty {
            pub fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $value,)+
                }
            }
        }

        impl FromStr for $ty {
            type Err = String;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                match value {
                    $($value => Ok(Self::$variant),)+
                    other => Err(format!("unknown {}: {other}", $label)),
                }
            }
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentOrderStatus {
    Created,
    AwaitingCustomer,
    Processing,
    Authorized,
    PartiallyCaptured,
    Captured,
    Failed,
    Expired,
    Canceled,
}

impl PaymentOrderStatus {
    pub fn supports_refund(self) -> bool {
        matches!(self, Self::PartiallyCaptured | Self::Captured)
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Failed | Self::Expired | Self::Canceled)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentRefundStatus {
    NotRequested,
    Pending,
    PartiallyRefunded,
    Refunded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentAttemptStatus {
    Initiated,
    HandoffReady,
    Processing,
    Authorized,
    Succeeded,
    Failed,
    Expired,
    Canceled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentSessionKind {
    QrCode,
    Redirect,
    HostedCheckout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentSessionStatus {
    Open,
    Authorized,
    Settled,
    Expired,
    Failed,
    Canceled,
}

impl PaymentSessionStatus {
    pub fn is_terminal(self) -> bool {
        !matches!(self, Self::Open | Self::Authorized)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentTransactionKind {
    Authorization,
    Sale,
    Refund,
    Chargeback,
    Adjustment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentCallbackProcessingStatus {
    Pending,
    Processed,
    Ignored,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RefundOrderStatus {
    Requested,
    AwaitingApproval,
    Approved,
    Processing,
    PartiallySucceeded,
    Succeeded,
    Failed,
    Canceled,
}

impl RefundOrderStatus {
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::PartiallySucceeded | Self::Succeeded | Self::Failed | Self::Canceled
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinanceDirection {
    Debit,
    Credit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinanceEntryCode {
    CustomerPrepaidLiabilityIncrease,
    CustomerPrepaidLiabilityDecrease,
    RefundPayout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReconciliationMatchStatus {
    Matched,
    MismatchAmount,
    MismatchReference,
    MissingLocal,
    MissingProvider,
    Resolved,
}

impl_payment_enum_str!(PaymentOrderStatus, "payment order status", {
    Created => "created",
    AwaitingCustomer => "awaiting_customer",
    Processing => "processing",
    Authorized => "authorized",
    PartiallyCaptured => "partially_captured",
    Captured => "captured",
    Failed => "failed",
    Expired => "expired",
    Canceled => "canceled",
});

impl_payment_enum_str!(PaymentRefundStatus, "payment refund status", {
    NotRequested => "not_requested",
    Pending => "pending",
    PartiallyRefunded => "partially_refunded",
    Refunded => "refunded",
});

impl_payment_enum_str!(PaymentAttemptStatus, "payment attempt status", {
    Initiated => "initiated",
    HandoffReady => "handoff_ready",
    Processing => "processing",
    Authorized => "authorized",
    Succeeded => "succeeded",
    Failed => "failed",
    Expired => "expired",
    Canceled => "canceled",
});

impl_payment_enum_str!(PaymentSessionKind, "payment session kind", {
    QrCode => "qr_code",
    Redirect => "redirect",
    HostedCheckout => "hosted_checkout",
});

impl_payment_enum_str!(PaymentSessionStatus, "payment session status", {
    Open => "open",
    Authorized => "authorized",
    Settled => "settled",
    Expired => "expired",
    Failed => "failed",
    Canceled => "canceled",
});

impl_payment_enum_str!(PaymentTransactionKind, "payment transaction kind", {
    Authorization => "authorization",
    Sale => "sale",
    Refund => "refund",
    Chargeback => "chargeback",
    Adjustment => "adjustment",
});

impl_payment_enum_str!(
    PaymentCallbackProcessingStatus,
    "payment callback processing status",
    {
        Pending => "pending",
        Processed => "processed",
        Ignored => "ignored",
        Failed => "failed",
    }
);

impl_payment_enum_str!(RefundOrderStatus, "refund order status", {
    Requested => "requested",
    AwaitingApproval => "awaiting_approval",
    Approved => "approved",
    Processing => "processing",
    PartiallySucceeded => "partially_succeeded",
    Succeeded => "succeeded",
    Failed => "failed",
    Canceled => "canceled",
});

impl_payment_enum_str!(FinanceDirection, "finance direction", {
    Debit => "debit",
    Credit => "credit",
});

impl_payment_enum_str!(FinanceEntryCode, "finance entry code", {
    CustomerPrepaidLiabilityIncrease => "customer_prepaid_liability_increase",
    CustomerPrepaidLiabilityDecrease => "customer_prepaid_liability_decrease",
    RefundPayout => "refund_payout",
});

impl_payment_enum_str!(ReconciliationMatchStatus, "reconciliation match status", {
    Matched => "matched",
    MismatchAmount => "mismatch_amount",
    MismatchReference => "mismatch_reference",
    MissingLocal => "missing_local",
    MissingProvider => "missing_provider",
    Resolved => "resolved",
});

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaymentOrderRecord {
    pub payment_order_id: String,
    pub commerce_order_id: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub user_id: u64,
    pub project_id: String,
    pub order_kind: String,
    pub subject_type: String,
    pub subject_id: String,
    pub currency_code: String,
    pub amount_minor: u64,
    pub discount_minor: u64,
    pub subsidy_minor: u64,
    pub payable_minor: u64,
    pub captured_amount_minor: u64,
    pub provider_code: PaymentProviderCode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method_code: Option<String>,
    pub payment_status: PaymentOrderStatus,
    pub fulfillment_status: String,
    pub refund_status: PaymentRefundStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote_snapshot_json: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata_json: Option<String>,
    pub version: u64,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl PaymentOrderRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        payment_order_id: impl Into<String>,
        commerce_order_id: impl Into<String>,
        tenant_id: u64,
        organization_id: u64,
        user_id: u64,
        project_id: impl Into<String>,
        order_kind: impl Into<String>,
        subject_type: impl Into<String>,
        subject_id: impl Into<String>,
        currency_code: impl Into<String>,
        amount_minor: u64,
    ) -> Self {
        Self {
            payment_order_id: payment_order_id.into(),
            commerce_order_id: commerce_order_id.into(),
            tenant_id,
            organization_id,
            user_id,
            project_id: project_id.into(),
            order_kind: order_kind.into(),
            subject_type: subject_type.into(),
            subject_id: subject_id.into(),
            currency_code: currency_code.into(),
            amount_minor,
            discount_minor: 0,
            subsidy_minor: 0,
            payable_minor: amount_minor,
            captured_amount_minor: 0,
            provider_code: PaymentProviderCode::default(),
            method_code: None,
            payment_status: PaymentOrderStatus::Created,
            fulfillment_status: "pending".to_owned(),
            refund_status: PaymentRefundStatus::NotRequested,
            quote_snapshot_json: None,
            metadata_json: None,
            version: 0,
            created_at_ms: 0,
            updated_at_ms: 0,
        }
    }

    pub fn with_discount_minor(mut self, discount_minor: u64) -> Self {
        self.discount_minor = discount_minor;
        self
    }

    pub fn with_subsidy_minor(mut self, subsidy_minor: u64) -> Self {
        self.subsidy_minor = subsidy_minor;
        self
    }

    pub fn with_payable_minor(mut self, payable_minor: u64) -> Self {
        self.payable_minor = payable_minor;
        self
    }

    pub fn with_captured_amount_minor(mut self, captured_amount_minor: u64) -> Self {
        self.captured_amount_minor = captured_amount_minor;
        self
    }

    pub fn with_provider_code(mut self, provider_code: PaymentProviderCode) -> Self {
        self.provider_code = provider_code;
        self
    }

    pub fn with_method_code(mut self, method_code: impl Into<String>) -> Self {
        self.method_code = Some(method_code.into());
        self
    }

    pub fn with_payment_status(mut self, payment_status: PaymentOrderStatus) -> Self {
        self.payment_status = payment_status;
        self
    }

    pub fn with_fulfillment_status(mut self, fulfillment_status: impl Into<String>) -> Self {
        self.fulfillment_status = fulfillment_status.into();
        self
    }

    pub fn with_refund_status(mut self, refund_status: PaymentRefundStatus) -> Self {
        self.refund_status = refund_status;
        self
    }

    pub fn with_quote_snapshot_json(mut self, quote_snapshot_json: Option<String>) -> Self {
        self.quote_snapshot_json = quote_snapshot_json;
        self
    }

    pub fn with_metadata_json(mut self, metadata_json: Option<String>) -> Self {
        self.metadata_json = metadata_json;
        self
    }

    pub fn with_version(mut self, version: u64) -> Self {
        self.version = version;
        self
    }

    pub fn with_created_at_ms(mut self, created_at_ms: u64) -> Self {
        self.created_at_ms = created_at_ms;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaymentAttemptRecord {
    pub payment_attempt_id: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub payment_order_id: String,
    pub attempt_no: u32,
    pub gateway_account_id: String,
    pub provider_code: PaymentProviderCode,
    pub method_code: String,
    pub client_kind: String,
    pub idempotency_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_request_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_payment_reference: Option<String>,
    pub attempt_status: PaymentAttemptStatus,
    pub request_payload_hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at_ms: Option<u64>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl PaymentAttemptRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        payment_attempt_id: impl Into<String>,
        tenant_id: u64,
        organization_id: u64,
        payment_order_id: impl Into<String>,
        attempt_no: u32,
        gateway_account_id: impl Into<String>,
        provider_code: PaymentProviderCode,
        method_code: impl Into<String>,
        client_kind: impl Into<String>,
        idempotency_key: impl Into<String>,
    ) -> Self {
        Self {
            payment_attempt_id: payment_attempt_id.into(),
            tenant_id,
            organization_id,
            payment_order_id: payment_order_id.into(),
            attempt_no,
            gateway_account_id: gateway_account_id.into(),
            provider_code,
            method_code: method_code.into(),
            client_kind: client_kind.into(),
            idempotency_key: idempotency_key.into(),
            provider_request_id: None,
            provider_payment_reference: None,
            attempt_status: PaymentAttemptStatus::Initiated,
            request_payload_hash: String::new(),
            expires_at_ms: None,
            created_at_ms: 0,
            updated_at_ms: 0,
        }
    }

    pub fn with_provider_request_id(mut self, provider_request_id: Option<String>) -> Self {
        self.provider_request_id = provider_request_id;
        self
    }

    pub fn with_provider_payment_reference(
        mut self,
        provider_payment_reference: Option<String>,
    ) -> Self {
        self.provider_payment_reference = provider_payment_reference;
        self
    }

    pub fn with_attempt_status(mut self, attempt_status: PaymentAttemptStatus) -> Self {
        self.attempt_status = attempt_status;
        self
    }

    pub fn with_request_payload_hash(mut self, request_payload_hash: impl Into<String>) -> Self {
        self.request_payload_hash = request_payload_hash.into();
        self
    }

    pub fn with_expires_at_ms(mut self, expires_at_ms: Option<u64>) -> Self {
        self.expires_at_ms = expires_at_ms;
        self
    }

    pub fn with_created_at_ms(mut self, created_at_ms: u64) -> Self {
        self.created_at_ms = created_at_ms;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaymentSessionRecord {
    pub payment_session_id: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub payment_attempt_id: String,
    pub session_kind: PaymentSessionKind,
    pub session_status: PaymentSessionStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_reference: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub qr_payload: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redirect_url: Option<String>,
    pub expires_at_ms: u64,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl PaymentSessionRecord {
    pub fn new(
        payment_session_id: impl Into<String>,
        tenant_id: u64,
        organization_id: u64,
        payment_attempt_id: impl Into<String>,
        session_kind: PaymentSessionKind,
        session_status: PaymentSessionStatus,
    ) -> Self {
        Self {
            payment_session_id: payment_session_id.into(),
            tenant_id,
            organization_id,
            payment_attempt_id: payment_attempt_id.into(),
            session_kind,
            session_status,
            display_reference: None,
            qr_payload: None,
            redirect_url: None,
            expires_at_ms: 0,
            created_at_ms: 0,
            updated_at_ms: 0,
        }
    }

    pub fn with_display_reference(mut self, display_reference: Option<String>) -> Self {
        self.display_reference = display_reference;
        self
    }

    pub fn with_qr_payload(mut self, qr_payload: Option<String>) -> Self {
        self.qr_payload = qr_payload;
        self
    }

    pub fn with_redirect_url(mut self, redirect_url: Option<String>) -> Self {
        self.redirect_url = redirect_url;
        self
    }

    pub fn with_expires_at_ms(mut self, expires_at_ms: u64) -> Self {
        self.expires_at_ms = expires_at_ms;
        self
    }

    pub fn with_created_at_ms(mut self, created_at_ms: u64) -> Self {
        self.created_at_ms = created_at_ms;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaymentGatewayAccountRecord {
    pub gateway_account_id: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub provider_code: PaymentProviderCode,
    pub environment: String,
    pub merchant_id: String,
    pub app_id: String,
    pub status: String,
    pub priority: i32,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl PaymentGatewayAccountRecord {
    pub fn new(
        gateway_account_id: impl Into<String>,
        tenant_id: u64,
        organization_id: u64,
        provider_code: PaymentProviderCode,
    ) -> Self {
        Self {
            gateway_account_id: gateway_account_id.into(),
            tenant_id,
            organization_id,
            provider_code,
            environment: "production".to_owned(),
            merchant_id: String::new(),
            app_id: String::new(),
            status: "active".to_owned(),
            priority: 0,
            created_at_ms: 0,
            updated_at_ms: 0,
        }
    }

    pub fn with_environment(mut self, environment: impl Into<String>) -> Self {
        self.environment = environment.into();
        self
    }

    pub fn with_merchant_id(mut self, merchant_id: impl Into<String>) -> Self {
        self.merchant_id = merchant_id.into();
        self
    }

    pub fn with_app_id(mut self, app_id: impl Into<String>) -> Self {
        self.app_id = app_id.into();
        self
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = status.into();
        self
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_created_at_ms(mut self, created_at_ms: u64) -> Self {
        self.created_at_ms = created_at_ms;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaymentChannelPolicyRecord {
    pub channel_policy_id: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub scene_code: String,
    pub country_code: String,
    pub currency_code: String,
    pub client_kind: String,
    pub provider_code: PaymentProviderCode,
    pub method_code: String,
    pub priority: i32,
    pub status: String,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl PaymentChannelPolicyRecord {
    pub fn new(
        channel_policy_id: impl Into<String>,
        tenant_id: u64,
        organization_id: u64,
        provider_code: PaymentProviderCode,
        method_code: impl Into<String>,
    ) -> Self {
        Self {
            channel_policy_id: channel_policy_id.into(),
            tenant_id,
            organization_id,
            scene_code: String::new(),
            country_code: String::new(),
            currency_code: String::new(),
            client_kind: String::new(),
            provider_code,
            method_code: method_code.into(),
            priority: 0,
            status: "active".to_owned(),
            created_at_ms: 0,
            updated_at_ms: 0,
        }
    }

    pub fn with_scene_code(mut self, scene_code: impl Into<String>) -> Self {
        self.scene_code = scene_code.into();
        self
    }

    pub fn with_country_code(mut self, country_code: impl Into<String>) -> Self {
        self.country_code = country_code.into();
        self
    }

    pub fn with_currency_code(mut self, currency_code: impl Into<String>) -> Self {
        self.currency_code = currency_code.into();
        self
    }

    pub fn with_client_kind(mut self, client_kind: impl Into<String>) -> Self {
        self.client_kind = client_kind.into();
        self
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = status.into();
        self
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_created_at_ms(mut self, created_at_ms: u64) -> Self {
        self.created_at_ms = created_at_ms;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaymentCallbackEventRecord {
    pub callback_event_id: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub provider_code: PaymentProviderCode,
    pub gateway_account_id: String,
    pub event_type: String,
    pub event_identity: String,
    pub dedupe_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payment_order_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payment_attempt_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_transaction_id: Option<String>,
    pub signature_status: String,
    pub processing_status: PaymentCallbackProcessingStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload_json: Option<String>,
    pub received_at_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub processed_at_ms: Option<u64>,
}

impl PaymentCallbackEventRecord {
    pub fn new(
        callback_event_id: impl Into<String>,
        tenant_id: u64,
        organization_id: u64,
        provider_code: PaymentProviderCode,
        gateway_account_id: impl Into<String>,
        event_type: impl Into<String>,
        event_identity: impl Into<String>,
        dedupe_key: impl Into<String>,
        received_at_ms: u64,
    ) -> Self {
        Self {
            callback_event_id: callback_event_id.into(),
            tenant_id,
            organization_id,
            provider_code,
            gateway_account_id: gateway_account_id.into(),
            event_type: event_type.into(),
            event_identity: event_identity.into(),
            dedupe_key: dedupe_key.into(),
            payment_order_id: None,
            payment_attempt_id: None,
            provider_transaction_id: None,
            signature_status: "pending".to_owned(),
            processing_status: PaymentCallbackProcessingStatus::Pending,
            payload_json: None,
            received_at_ms,
            processed_at_ms: None,
        }
    }

    pub fn with_payment_order_id(mut self, payment_order_id: Option<String>) -> Self {
        self.payment_order_id = payment_order_id;
        self
    }

    pub fn with_payment_attempt_id(mut self, payment_attempt_id: Option<String>) -> Self {
        self.payment_attempt_id = payment_attempt_id;
        self
    }

    pub fn with_provider_transaction_id(mut self, provider_transaction_id: Option<String>) -> Self {
        self.provider_transaction_id = provider_transaction_id;
        self
    }

    pub fn with_signature_status(mut self, signature_status: impl Into<String>) -> Self {
        self.signature_status = signature_status.into();
        self
    }

    pub fn with_processing_status(
        mut self,
        processing_status: PaymentCallbackProcessingStatus,
    ) -> Self {
        self.processing_status = processing_status;
        self
    }

    pub fn with_payload_json(mut self, payload_json: Option<String>) -> Self {
        self.payload_json = payload_json;
        self
    }

    pub fn with_processed_at_ms(mut self, processed_at_ms: Option<u64>) -> Self {
        self.processed_at_ms = processed_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefundOrderRecord {
    pub refund_order_id: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub payment_order_id: String,
    pub commerce_order_id: String,
    pub refund_reason_code: String,
    pub requested_by_type: String,
    pub requested_by_id: String,
    pub currency_code: String,
    pub requested_amount_minor: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approved_amount_minor: Option<u64>,
    pub refunded_amount_minor: u64,
    pub refund_status: RefundOrderStatus,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl RefundOrderRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        refund_order_id: impl Into<String>,
        tenant_id: u64,
        organization_id: u64,
        payment_order_id: impl Into<String>,
        commerce_order_id: impl Into<String>,
        refund_reason_code: impl Into<String>,
        requested_by_type: impl Into<String>,
        requested_by_id: impl Into<String>,
        currency_code: impl Into<String>,
        requested_amount_minor: u64,
    ) -> Self {
        Self {
            refund_order_id: refund_order_id.into(),
            tenant_id,
            organization_id,
            payment_order_id: payment_order_id.into(),
            commerce_order_id: commerce_order_id.into(),
            refund_reason_code: refund_reason_code.into(),
            requested_by_type: requested_by_type.into(),
            requested_by_id: requested_by_id.into(),
            currency_code: currency_code.into(),
            requested_amount_minor,
            approved_amount_minor: None,
            refunded_amount_minor: 0,
            refund_status: RefundOrderStatus::Requested,
            created_at_ms: 0,
            updated_at_ms: 0,
        }
    }

    pub fn with_approved_amount_minor(mut self, approved_amount_minor: Option<u64>) -> Self {
        self.approved_amount_minor = approved_amount_minor;
        self
    }

    pub fn with_refunded_amount_minor(mut self, refunded_amount_minor: u64) -> Self {
        self.refunded_amount_minor = refunded_amount_minor;
        self
    }

    pub fn with_refund_status(mut self, refund_status: RefundOrderStatus) -> Self {
        self.refund_status = refund_status;
        self
    }

    pub fn with_created_at_ms(mut self, created_at_ms: u64) -> Self {
        self.created_at_ms = created_at_ms;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReconciliationMatchSummaryRecord {
    pub reconciliation_line_id: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub reconciliation_batch_id: String,
    pub provider_transaction_id: String,
    pub match_status: ReconciliationMatchStatus,
    pub provider_amount_minor: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_amount_minor: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payment_order_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refund_order_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<String>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl ReconciliationMatchSummaryRecord {
    pub fn new(
        reconciliation_line_id: impl Into<String>,
        tenant_id: u64,
        organization_id: u64,
        reconciliation_batch_id: impl Into<String>,
        provider_transaction_id: impl Into<String>,
        match_status: ReconciliationMatchStatus,
        provider_amount_minor: u64,
    ) -> Self {
        Self {
            reconciliation_line_id: reconciliation_line_id.into(),
            tenant_id,
            organization_id,
            reconciliation_batch_id: reconciliation_batch_id.into(),
            provider_transaction_id: provider_transaction_id.into(),
            match_status,
            provider_amount_minor,
            local_amount_minor: None,
            payment_order_id: None,
            refund_order_id: None,
            reason_code: None,
            created_at_ms: 0,
            updated_at_ms: 0,
        }
    }

    pub fn with_local_amount_minor(mut self, local_amount_minor: Option<u64>) -> Self {
        self.local_amount_minor = local_amount_minor;
        self
    }

    pub fn with_payment_order_id(mut self, payment_order_id: Option<String>) -> Self {
        self.payment_order_id = payment_order_id;
        self
    }

    pub fn with_refund_order_id(mut self, refund_order_id: Option<String>) -> Self {
        self.refund_order_id = refund_order_id;
        self
    }

    pub fn with_reason_code(mut self, reason_code: Option<String>) -> Self {
        self.reason_code = reason_code;
        self
    }

    pub fn with_created_at_ms(mut self, created_at_ms: u64) -> Self {
        self.created_at_ms = created_at_ms;
        if self.updated_at_ms == 0 {
            self.updated_at_ms = created_at_ms;
        }
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaymentTransactionRecord {
    pub payment_transaction_id: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub payment_order_id: String,
    pub transaction_kind: PaymentTransactionKind,
    pub provider_code: PaymentProviderCode,
    pub provider_transaction_id: String,
    pub currency_code: String,
    pub amount_minor: u64,
    pub occurred_at_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payment_attempt_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fee_minor: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub net_amount_minor: Option<u64>,
    pub provider_status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_event_id: Option<String>,
    pub created_at_ms: u64,
}

impl PaymentTransactionRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        payment_transaction_id: impl Into<String>,
        tenant_id: u64,
        organization_id: u64,
        payment_order_id: impl Into<String>,
        transaction_kind: PaymentTransactionKind,
        provider_code: PaymentProviderCode,
        provider_transaction_id: impl Into<String>,
        currency_code: impl Into<String>,
        amount_minor: u64,
        occurred_at_ms: u64,
    ) -> Self {
        Self {
            payment_transaction_id: payment_transaction_id.into(),
            tenant_id,
            organization_id,
            payment_order_id: payment_order_id.into(),
            transaction_kind,
            provider_code,
            provider_transaction_id: provider_transaction_id.into(),
            currency_code: currency_code.into(),
            amount_minor,
            occurred_at_ms,
            payment_attempt_id: None,
            fee_minor: None,
            net_amount_minor: None,
            provider_status: "pending".to_owned(),
            raw_event_id: None,
            created_at_ms: 0,
        }
    }

    pub fn with_payment_attempt_id(mut self, payment_attempt_id: Option<String>) -> Self {
        self.payment_attempt_id = payment_attempt_id;
        self
    }

    pub fn with_fee_minor(mut self, fee_minor: Option<u64>) -> Self {
        self.fee_minor = fee_minor;
        self
    }

    pub fn with_net_amount_minor(mut self, net_amount_minor: Option<u64>) -> Self {
        self.net_amount_minor = net_amount_minor;
        self
    }

    pub fn with_provider_status(mut self, provider_status: impl Into<String>) -> Self {
        self.provider_status = provider_status.into();
        self
    }

    pub fn with_raw_event_id(mut self, raw_event_id: Option<String>) -> Self {
        self.raw_event_id = raw_event_id;
        self
    }

    pub fn with_created_at_ms(mut self, created_at_ms: u64) -> Self {
        self.created_at_ms = created_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FinanceJournalEntryRecord {
    pub finance_journal_entry_id: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub source_kind: String,
    pub source_id: String,
    pub entry_code: FinanceEntryCode,
    pub currency_code: String,
    pub occurred_at_ms: u64,
    pub entry_status: String,
    pub created_at_ms: u64,
}

impl FinanceJournalEntryRecord {
    pub fn new(
        finance_journal_entry_id: impl Into<String>,
        tenant_id: u64,
        organization_id: u64,
        source_kind: impl Into<String>,
        source_id: impl Into<String>,
        entry_code: FinanceEntryCode,
        currency_code: impl Into<String>,
        occurred_at_ms: u64,
    ) -> Self {
        Self {
            finance_journal_entry_id: finance_journal_entry_id.into(),
            tenant_id,
            organization_id,
            source_kind: source_kind.into(),
            source_id: source_id.into(),
            entry_code,
            currency_code: currency_code.into(),
            occurred_at_ms,
            entry_status: "draft".to_owned(),
            created_at_ms: 0,
        }
    }

    pub fn with_entry_status(mut self, entry_status: impl Into<String>) -> Self {
        self.entry_status = entry_status.into();
        self
    }

    pub fn with_created_at_ms(mut self, created_at_ms: u64) -> Self {
        self.created_at_ms = created_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FinanceJournalLineRecord {
    pub finance_journal_line_id: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub finance_journal_entry_id: String,
    pub line_no: u32,
    pub account_code: String,
    pub direction: FinanceDirection,
    pub amount_minor: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub party_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub party_id: Option<String>,
}

impl FinanceJournalLineRecord {
    pub fn new(
        finance_journal_line_id: impl Into<String>,
        tenant_id: u64,
        organization_id: u64,
        finance_journal_entry_id: impl Into<String>,
        line_no: u32,
        account_code: impl Into<String>,
        direction: FinanceDirection,
        amount_minor: u64,
    ) -> Self {
        Self {
            finance_journal_line_id: finance_journal_line_id.into(),
            tenant_id,
            organization_id,
            finance_journal_entry_id: finance_journal_entry_id.into(),
            line_no,
            account_code: account_code.into(),
            direction,
            amount_minor,
            party_type: None,
            party_id: None,
        }
    }

    pub fn with_party_type(mut self, party_type: Option<String>) -> Self {
        self.party_type = party_type;
        self
    }

    pub fn with_party_id(mut self, party_id: Option<String>) -> Self {
        self.party_id = party_id;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::{
        FinanceDirection, FinanceEntryCode, FinanceJournalEntryRecord, FinanceJournalLineRecord,
        PaymentAttemptRecord, PaymentAttemptStatus, PaymentCallbackEventRecord,
        PaymentCallbackProcessingStatus, PaymentOrderRecord, PaymentOrderStatus,
        PaymentProviderCode, PaymentRefundStatus, PaymentSessionKind, PaymentSessionRecord,
        PaymentSessionStatus, PaymentTransactionKind, PaymentTransactionRecord,
        ReconciliationMatchStatus, ReconciliationMatchSummaryRecord, RefundOrderRecord,
        RefundOrderStatus,
    };

    #[test]
    fn payment_order_keeps_checkout_and_finance_fields() {
        let order = PaymentOrderRecord::new(
            "payment_order_1",
            "commerce_order_1",
            1,
            0,
            7,
            "project_demo",
            "recharge_pack",
            "workspace",
            "project_demo",
            "CNY",
            12_000,
        )
        .with_discount_minor(1_000)
        .with_subsidy_minor(500)
        .with_payable_minor(10_500)
        .with_provider_code(PaymentProviderCode::WeChatPay)
        .with_method_code("native_qr")
        .with_payment_status(PaymentOrderStatus::AwaitingCustomer)
        .with_fulfillment_status("pending")
        .with_refund_status(PaymentRefundStatus::NotRequested)
        .with_quote_snapshot_json(Some("{\"quote_id\":\"quote_1\"}".to_owned()))
        .with_metadata_json(Some("{\"scene\":\"portal\"}".to_owned()))
        .with_version(3)
        .with_created_at_ms(1_710_000_001)
        .with_updated_at_ms(1_710_000_002);

        assert_eq!(order.payable_minor, 10_500);
        assert_eq!(order.provider_code, PaymentProviderCode::WeChatPay);
        assert_eq!(order.method_code.as_deref(), Some("native_qr"));
        assert_eq!(order.payment_status, PaymentOrderStatus::AwaitingCustomer);
        assert_eq!(order.refund_status, PaymentRefundStatus::NotRequested);
        assert_eq!(order.version, 3);
    }

    #[test]
    fn payment_attempt_and_session_support_provider_handoff_payloads() {
        let attempt = PaymentAttemptRecord::new(
            "payment_attempt_1",
            1,
            0,
            "payment_order_1",
            1,
            "gateway_account_1",
            PaymentProviderCode::Stripe,
            "checkout",
            "desktop_web",
            "idem_attempt_1",
        )
        .with_provider_request_id(Some("req_123".to_owned()))
        .with_provider_payment_reference(Some("cs_test_123".to_owned()))
        .with_attempt_status(PaymentAttemptStatus::HandoffReady)
        .with_request_payload_hash("payload_hash")
        .with_expires_at_ms(Some(1_710_000_999))
        .with_created_at_ms(1_710_000_010)
        .with_updated_at_ms(1_710_000_020);

        let session = PaymentSessionRecord::new(
            "payment_session_1",
            1,
            0,
            "payment_attempt_1",
            PaymentSessionKind::QrCode,
            PaymentSessionStatus::Open,
        )
        .with_display_reference(Some("PAY-ORDER-1".to_owned()))
        .with_qr_payload(Some("weixin://wxpay/demo".to_owned()))
        .with_redirect_url(None)
        .with_expires_at_ms(1_710_000_999)
        .with_created_at_ms(1_710_000_030)
        .with_updated_at_ms(1_710_000_031);

        assert_eq!(attempt.tenant_id, 1);
        assert_eq!(attempt.organization_id, 0);
        assert_eq!(attempt.provider_code, PaymentProviderCode::Stripe);
        assert_eq!(session.tenant_id, 1);
        assert_eq!(session.organization_id, 0);
        assert_eq!(attempt.attempt_status, PaymentAttemptStatus::HandoffReady);
        assert_eq!(session.session_kind, PaymentSessionKind::QrCode);
        assert_eq!(session.qr_payload.as_deref(), Some("weixin://wxpay/demo"));
        assert_eq!(session.redirect_url, None);
    }

    #[test]
    fn callback_refund_and_reconciliation_records_keep_audit_state() {
        let callback = PaymentCallbackEventRecord::new(
            "callback_1",
            1,
            0,
            PaymentProviderCode::Alipay,
            "gateway_account_1",
            "trade_status_sync",
            "notify:trade_123",
            "dedupe_trade_123",
            1_710_100_000,
        )
        .with_payment_order_id(Some("payment_order_1".to_owned()))
        .with_payment_attempt_id(Some("payment_attempt_1".to_owned()))
        .with_provider_transaction_id(Some("trade_123".to_owned()))
        .with_signature_status("verified")
        .with_processing_status(PaymentCallbackProcessingStatus::Processed)
        .with_processed_at_ms(Some(1_710_100_020));

        let refund = RefundOrderRecord::new(
            "refund_order_1",
            1,
            0,
            "payment_order_1",
            "commerce_order_1",
            "buyer_request",
            "portal_user",
            "user_7",
            "CNY",
            3_000,
        )
        .with_approved_amount_minor(Some(2_000))
        .with_refunded_amount_minor(1_500)
        .with_refund_status(RefundOrderStatus::Processing)
        .with_created_at_ms(1_710_100_030)
        .with_updated_at_ms(1_710_100_031);

        let reconciliation = ReconciliationMatchSummaryRecord::new(
            "recon_line_1",
            1,
            0,
            "recon_batch_1",
            "trade_123",
            ReconciliationMatchStatus::MismatchAmount,
            10_500,
        )
        .with_local_amount_minor(Some(10_000))
        .with_payment_order_id(Some("payment_order_1".to_owned()))
        .with_reason_code(Some("provider_fee_not_split".to_owned()))
        .with_created_at_ms(1_710_100_040);

        assert_eq!(callback.tenant_id, 1);
        assert_eq!(callback.organization_id, 0);
        assert_eq!(
            callback.processing_status,
            PaymentCallbackProcessingStatus::Processed
        );
        assert_eq!(refund.tenant_id, 1);
        assert_eq!(refund.organization_id, 0);
        assert_eq!(refund.refund_status, RefundOrderStatus::Processing);
        assert_eq!(refund.refunded_amount_minor, 1_500);
        assert_eq!(reconciliation.tenant_id, 1);
        assert_eq!(reconciliation.organization_id, 0);
        assert_eq!(
            reconciliation.match_status,
            ReconciliationMatchStatus::MismatchAmount
        );
    }

    #[test]
    fn finance_journal_and_transaction_records_preserve_money_evidence() {
        let transaction = PaymentTransactionRecord::new(
            "payment_tx_1",
            1,
            0,
            "payment_order_1",
            PaymentTransactionKind::Sale,
            PaymentProviderCode::Stripe,
            "pi_123",
            "USD",
            2_500,
            1_710_100_100,
        )
        .with_payment_attempt_id(Some("payment_attempt_1".to_owned()))
        .with_fee_minor(Some(80))
        .with_net_amount_minor(Some(2_420))
        .with_provider_status("succeeded")
        .with_raw_event_id(Some("evt_123".to_owned()))
        .with_created_at_ms(1_710_100_101);

        let journal = FinanceJournalEntryRecord::new(
            "finance_journal_1",
            1,
            0,
            "payment_order",
            "payment_order_1",
            FinanceEntryCode::CustomerPrepaidLiabilityIncrease,
            "USD",
            1_710_100_110,
        )
        .with_entry_status("posted")
        .with_created_at_ms(1_710_100_111);

        let journal_line = FinanceJournalLineRecord::new(
            "finance_line_1",
            1,
            0,
            "finance_journal_1",
            1,
            "gateway_clearing_asset",
            FinanceDirection::Debit,
            2_500,
        )
        .with_party_type(Some("payment_order".to_owned()))
        .with_party_id(Some("payment_order_1".to_owned()));

        assert_eq!(transaction.tenant_id, 1);
        assert_eq!(transaction.organization_id, 0);
        assert_eq!(transaction.transaction_kind, PaymentTransactionKind::Sale);
        assert_eq!(transaction.net_amount_minor, Some(2_420));
        assert_eq!(journal.tenant_id, 1);
        assert_eq!(journal.organization_id, 0);
        assert_eq!(
            journal.entry_code,
            FinanceEntryCode::CustomerPrepaidLiabilityIncrease
        );
        assert_eq!(journal_line.tenant_id, 1);
        assert_eq!(journal_line.organization_id, 0);
        assert_eq!(journal_line.direction, FinanceDirection::Debit);
    }

    #[test]
    fn status_enums_normalize_and_gate_terminal_transitions() {
        assert_eq!(
            "wechat_pay".parse::<PaymentProviderCode>().unwrap(),
            PaymentProviderCode::WeChatPay
        );
        assert_eq!(
            "authorized".parse::<PaymentOrderStatus>().unwrap(),
            PaymentOrderStatus::Authorized
        );
        assert_eq!(
            "partially_captured".parse::<PaymentOrderStatus>().unwrap(),
            PaymentOrderStatus::PartiallyCaptured
        );
        assert_eq!(
            "captured".parse::<PaymentOrderStatus>().unwrap(),
            PaymentOrderStatus::Captured
        );
        assert_eq!(PaymentAttemptStatus::Authorized.as_str(), "authorized");
        assert_eq!(PaymentSessionStatus::Authorized.as_str(), "authorized");
        assert_eq!(
            PaymentTransactionKind::Authorization.as_str(),
            "authorization"
        );
        assert_eq!(PaymentAttemptStatus::Processing.as_str(), "processing");
        assert_eq!(
            "mismatch_amount"
                .parse::<ReconciliationMatchStatus>()
                .unwrap(),
            ReconciliationMatchStatus::MismatchAmount
        );
        assert!(PaymentOrderStatus::PartiallyCaptured.supports_refund());
        assert!(PaymentOrderStatus::Captured.supports_refund());
        assert!(!PaymentOrderStatus::Created.supports_refund());
        assert!(PaymentOrderStatus::Failed.is_terminal());
        assert!(!PaymentSessionStatus::Authorized.is_terminal());
        assert!(PaymentSessionStatus::Expired.is_terminal());
        assert!(RefundOrderStatus::Succeeded.is_terminal());
    }
}
