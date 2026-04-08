use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CommerceOrderRecord {
    pub order_id: String,
    pub project_id: String,
    pub user_id: String,
    pub target_kind: String,
    pub target_id: String,
    pub target_name: String,
    pub list_price_cents: u64,
    pub payable_price_cents: u64,
    pub list_price_label: String,
    pub payable_price_label: String,
    pub granted_units: u64,
    pub bonus_units: u64,
    pub currency_code: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pricing_plan_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pricing_plan_version: Option<u64>,
    pub pricing_snapshot_json: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applied_coupon_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coupon_reservation_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coupon_redemption_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marketing_campaign_id: Option<String>,
    #[serde(default)]
    pub subsidy_amount_minor: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payment_method_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_payment_attempt_id: Option<String>,
    pub status: String,
    pub settlement_status: String,
    pub source: String,
    #[serde(default)]
    pub refundable_amount_minor: u64,
    #[serde(default)]
    pub refunded_amount_minor: u64,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl CommerceOrderRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        order_id: impl Into<String>,
        project_id: impl Into<String>,
        user_id: impl Into<String>,
        target_kind: impl Into<String>,
        target_id: impl Into<String>,
        target_name: impl Into<String>,
        list_price_cents: u64,
        payable_price_cents: u64,
        list_price_label: impl Into<String>,
        payable_price_label: impl Into<String>,
        granted_units: u64,
        bonus_units: u64,
        status: impl Into<String>,
        source: impl Into<String>,
        created_at_ms: u64,
    ) -> Self {
        let status = status.into();
        let source = source.into();
        Self {
            order_id: order_id.into(),
            project_id: project_id.into(),
            user_id: user_id.into(),
            target_kind: target_kind.into(),
            target_id: target_id.into(),
            target_name: target_name.into(),
            list_price_cents,
            payable_price_cents,
            list_price_label: list_price_label.into(),
            payable_price_label: payable_price_label.into(),
            granted_units,
            bonus_units,
            currency_code: "USD".to_owned(),
            pricing_plan_id: None,
            pricing_plan_version: None,
            pricing_snapshot_json: "{}".to_owned(),
            applied_coupon_code: None,
            coupon_reservation_id: None,
            coupon_redemption_id: None,
            marketing_campaign_id: None,
            subsidy_amount_minor: 0,
            payment_method_id: None,
            latest_payment_attempt_id: None,
            status: status.clone(),
            settlement_status: default_settlement_status_for_order_status(
                payable_price_cents,
                &status,
            )
            .to_owned(),
            source,
            refundable_amount_minor: payable_price_cents,
            refunded_amount_minor: 0,
            created_at_ms,
            updated_at_ms: created_at_ms,
        }
    }

    pub fn with_currency_code(mut self, currency_code: impl Into<String>) -> Self {
        self.currency_code = currency_code.into();
        self
    }

    pub fn with_pricing_plan_id_option(mut self, pricing_plan_id: Option<String>) -> Self {
        self.pricing_plan_id = pricing_plan_id;
        self
    }

    pub fn with_pricing_plan_version_option(mut self, pricing_plan_version: Option<u64>) -> Self {
        self.pricing_plan_version = pricing_plan_version;
        self
    }

    pub fn with_pricing_snapshot_json(mut self, pricing_snapshot_json: impl Into<String>) -> Self {
        self.pricing_snapshot_json = pricing_snapshot_json.into();
        self
    }

    pub fn with_applied_coupon_code_option(mut self, applied_coupon_code: Option<String>) -> Self {
        self.applied_coupon_code = applied_coupon_code;
        self
    }

    pub fn with_coupon_reservation_id_option(
        mut self,
        coupon_reservation_id: Option<String>,
    ) -> Self {
        self.coupon_reservation_id = coupon_reservation_id;
        self
    }

    pub fn with_coupon_redemption_id_option(
        mut self,
        coupon_redemption_id: Option<String>,
    ) -> Self {
        self.coupon_redemption_id = coupon_redemption_id;
        self
    }

    pub fn with_marketing_campaign_id_option(
        mut self,
        marketing_campaign_id: Option<String>,
    ) -> Self {
        self.marketing_campaign_id = marketing_campaign_id;
        self
    }

    pub fn with_subsidy_amount_minor(mut self, subsidy_amount_minor: u64) -> Self {
        self.subsidy_amount_minor = subsidy_amount_minor;
        self
    }

    pub fn with_payment_method_id_option(mut self, payment_method_id: Option<String>) -> Self {
        self.payment_method_id = payment_method_id;
        self
    }

    pub fn with_latest_payment_attempt_id_option(
        mut self,
        latest_payment_attempt_id: Option<String>,
    ) -> Self {
        self.latest_payment_attempt_id = latest_payment_attempt_id;
        self
    }

    pub fn with_settlement_status(mut self, settlement_status: impl Into<String>) -> Self {
        self.settlement_status = settlement_status.into();
        self
    }

    pub fn with_refundable_amount_minor(mut self, refundable_amount_minor: u64) -> Self {
        self.refundable_amount_minor = refundable_amount_minor;
        self
    }

    pub fn with_refunded_amount_minor(mut self, refunded_amount_minor: u64) -> Self {
        self.refunded_amount_minor = refunded_amount_minor;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}

fn default_settlement_status_for_order_status(
    payable_price_cents: u64,
    status: &str,
) -> &'static str {
    match status {
        "pending_payment" => "pending",
        "fulfilled" if payable_price_cents == 0 => "not_required",
        "fulfilled" => "settled",
        "refunded" => "refunded",
        "failed" => "failed",
        "canceled" => "canceled",
        _ => "pending",
    }
}
