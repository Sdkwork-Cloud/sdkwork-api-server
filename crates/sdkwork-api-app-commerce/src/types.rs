use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PortalSubscriptionPlan {
    pub id: String,
    pub name: String,
    pub price_label: String,
    pub cadence: String,
    pub included_units: u64,
    pub highlight: String,
    pub features: Vec<String>,
    pub cta: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PortalRechargePack {
    pub id: String,
    pub label: String,
    pub points: u64,
    pub price_label: String,
    pub note: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PortalRechargeOption {
    pub id: String,
    pub label: String,
    pub amount_cents: u64,
    pub amount_label: String,
    pub granted_units: u64,
    pub effective_ratio_label: String,
    pub note: String,
    pub recommended: bool,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PortalCustomRechargeRule {
    pub id: String,
    pub label: String,
    pub min_amount_cents: u64,
    pub max_amount_cents: u64,
    pub units_per_cent: u64,
    pub effective_ratio_label: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PortalCustomRechargePolicy {
    pub enabled: bool,
    pub min_amount_cents: u64,
    pub max_amount_cents: u64,
    pub step_amount_cents: u64,
    pub suggested_amount_cents: u64,
    pub rules: Vec<PortalCustomRechargeRule>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PortalCommerceCoupon {
    pub id: String,
    pub code: String,
    pub discount_label: String,
    pub audience: String,
    pub remaining: u64,
    pub active: bool,
    pub note: String,
    pub expires_on: String,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discount_percent: Option<u8>,
    #[serde(default)]
    pub bonus_units: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PortalCommerceCatalog {
    pub plans: Vec<PortalSubscriptionPlan>,
    pub packs: Vec<PortalRechargePack>,
    pub recharge_options: Vec<PortalRechargeOption>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_recharge_policy: Option<PortalCustomRechargePolicy>,
    pub coupons: Vec<PortalCommerceCoupon>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PortalCommerceQuoteRequest {
    pub target_kind: String,
    pub target_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coupon_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_remaining_units: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_amount_cents: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PortalAppliedCoupon {
    pub code: String,
    pub discount_label: String,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discount_percent: Option<u8>,
    #[serde(default)]
    pub bonus_units: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PortalCommerceQuote {
    pub target_kind: String,
    pub target_id: String,
    pub target_name: String,
    pub list_price_cents: u64,
    pub payable_price_cents: u64,
    pub list_price_label: String,
    pub payable_price_label: String,
    pub granted_units: u64,
    pub bonus_units: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount_cents: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projected_remaining_units: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applied_coupon: Option<PortalAppliedCoupon>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pricing_rule_label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_ratio_label: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PortalCommerceCheckoutSessionMethod {
    pub id: String,
    pub label: String,
    pub detail: String,
    pub action: String,
    pub availability: String,
    pub provider: String,
    pub channel: String,
    pub session_kind: String,
    pub session_reference: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub qr_code_payload: Option<String>,
    pub webhook_verification: String,
    pub supports_refund: bool,
    pub supports_partial_refund: bool,
    pub recommended: bool,
    pub supports_webhook: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PortalCommerceCheckoutSession {
    pub order_id: String,
    pub order_status: String,
    pub session_status: String,
    pub provider: String,
    pub mode: String,
    pub reference: String,
    pub payable_price_label: String,
    pub guidance: String,
    pub payment_simulation_enabled: bool,
    pub methods: Vec<PortalCommerceCheckoutSessionMethod>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PortalCommercePaymentEventRequest {
    pub event_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_event_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkout_method_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PortalCommercePaymentAttemptCreateRequest {
    pub payment_method_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub success_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cancel_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub customer_email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PortalCommerceWebhookAck {
    pub webhook_inbox_id: String,
    pub delivery_attempt_id: String,
    pub processing_status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_event_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payment_attempt_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct AdminCommerceRefundCreateRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payment_attempt_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount_minor: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct AdminCommerceReconciliationRunCreateRequest {
    pub provider: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payment_method_id: Option<String>,
    pub scope_started_at_ms: u64,
    pub scope_ended_at_ms: u64,
}
