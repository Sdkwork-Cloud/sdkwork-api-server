use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PricingPlanOwnershipScope {
    Workspace,
    PlatformShared,
}

impl Default for PricingPlanOwnershipScope {
    fn default() -> Self {
        Self::Workspace
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct PricingPlanRecord {
    pub pricing_plan_id: PricingPlanId,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub plan_code: String,
    pub plan_version: u64,
    pub display_name: String,
    pub currency_code: String,
    pub credit_unit_code: String,
    pub status: String,
    #[serde(default)]
    pub ownership_scope: PricingPlanOwnershipScope,
    pub effective_from_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_to_ms: Option<u64>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl PricingPlanRecord {
    pub fn new(
        pricing_plan_id: PricingPlanId,
        tenant_id: u64,
        organization_id: u64,
        plan_code: impl Into<String>,
        plan_version: u64,
    ) -> Self {
        Self {
            pricing_plan_id,
            tenant_id,
            organization_id,
            plan_code: plan_code.into(),
            plan_version,
            display_name: String::new(),
            currency_code: "USD".to_owned(),
            credit_unit_code: "credit".to_owned(),
            status: "draft".to_owned(),
            ownership_scope: PricingPlanOwnershipScope::default(),
            effective_from_ms: 0,
            effective_to_ms: None,
            created_at_ms: 0,
            updated_at_ms: 0,
        }
    }

    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = display_name.into();
        self
    }

    pub fn with_currency_code(mut self, currency_code: impl Into<String>) -> Self {
        self.currency_code = currency_code.into();
        self
    }

    pub fn with_credit_unit_code(mut self, credit_unit_code: impl Into<String>) -> Self {
        self.credit_unit_code = credit_unit_code.into();
        self
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = status.into();
        self
    }

    pub fn with_ownership_scope(mut self, ownership_scope: PricingPlanOwnershipScope) -> Self {
        self.ownership_scope = ownership_scope;
        self
    }

    pub fn with_effective_from_ms(mut self, effective_from_ms: u64) -> Self {
        self.effective_from_ms = effective_from_ms;
        self
    }

    pub fn with_effective_to_ms(mut self, effective_to_ms: Option<u64>) -> Self {
        self.effective_to_ms = effective_to_ms;
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

    pub fn is_platform_shared(&self) -> bool {
        self.ownership_scope == PricingPlanOwnershipScope::PlatformShared
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct AccountCommerceReconciliationStateRecord {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub account_id: AccountId,
    pub project_id: String,
    pub last_order_updated_at_ms: u64,
    pub last_order_created_at_ms: u64,
    pub last_order_id: String,
    pub updated_at_ms: u64,
}

impl AccountCommerceReconciliationStateRecord {
    pub fn new(
        tenant_id: u64,
        organization_id: u64,
        account_id: AccountId,
        project_id: impl Into<String>,
        last_order_id: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id,
            organization_id,
            account_id,
            project_id: project_id.into(),
            last_order_updated_at_ms: 0,
            last_order_created_at_ms: 0,
            last_order_id: last_order_id.into(),
            updated_at_ms: 0,
        }
    }

    pub fn with_last_order_updated_at_ms(mut self, last_order_updated_at_ms: u64) -> Self {
        self.last_order_updated_at_ms = last_order_updated_at_ms;
        self
    }

    pub fn with_last_order_created_at_ms(mut self, last_order_created_at_ms: u64) -> Self {
        self.last_order_created_at_ms = last_order_created_at_ms;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct PricingRateRecord {
    pub pricing_rate_id: PricingRateId,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub pricing_plan_id: PricingPlanId,
    pub metric_code: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_code: Option<String>,
    pub charge_unit: String,
    pub pricing_method: String,
    pub quantity_step: f64,
    pub unit_price: f64,
    pub display_price_unit: String,
    pub minimum_billable_quantity: f64,
    pub minimum_charge: f64,
    pub rounding_increment: f64,
    pub rounding_mode: String,
    pub included_quantity: f64,
    pub priority: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    pub status: String,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl PricingRateRecord {
    pub fn new(
        pricing_rate_id: PricingRateId,
        tenant_id: u64,
        organization_id: u64,
        pricing_plan_id: PricingPlanId,
        metric_code: impl Into<String>,
    ) -> Self {
        Self {
            pricing_rate_id,
            tenant_id,
            organization_id,
            pricing_plan_id,
            metric_code: metric_code.into(),
            capability_code: None,
            model_code: None,
            provider_code: None,
            charge_unit: "unit".to_owned(),
            pricing_method: "per_unit".to_owned(),
            quantity_step: 1.0,
            unit_price: 0.0,
            display_price_unit: String::new(),
            minimum_billable_quantity: 0.0,
            minimum_charge: 0.0,
            rounding_increment: 1.0,
            rounding_mode: "none".to_owned(),
            included_quantity: 0.0,
            priority: 0,
            notes: None,
            status: "draft".to_owned(),
            created_at_ms: 0,
            updated_at_ms: 0,
        }
    }

    pub fn with_capability_code(mut self, capability_code: Option<String>) -> Self {
        self.capability_code = capability_code;
        self
    }

    pub fn with_model_code(mut self, model_code: Option<String>) -> Self {
        self.model_code = model_code;
        self
    }

    pub fn with_provider_code(mut self, provider_code: Option<String>) -> Self {
        self.provider_code = provider_code;
        self
    }

    pub fn with_charge_unit(mut self, charge_unit: impl Into<String>) -> Self {
        self.charge_unit = charge_unit.into();
        self
    }

    pub fn with_pricing_method(mut self, pricing_method: impl Into<String>) -> Self {
        self.pricing_method = pricing_method.into();
        self
    }

    pub fn with_quantity_step(mut self, quantity_step: f64) -> Self {
        self.quantity_step = quantity_step;
        self
    }

    pub fn with_unit_price(mut self, unit_price: f64) -> Self {
        self.unit_price = unit_price;
        self
    }

    pub fn with_display_price_unit(mut self, display_price_unit: impl Into<String>) -> Self {
        self.display_price_unit = display_price_unit.into();
        self
    }

    pub fn with_minimum_billable_quantity(mut self, minimum_billable_quantity: f64) -> Self {
        self.minimum_billable_quantity = minimum_billable_quantity;
        self
    }

    pub fn with_minimum_charge(mut self, minimum_charge: f64) -> Self {
        self.minimum_charge = minimum_charge;
        self
    }

    pub fn with_rounding_increment(mut self, rounding_increment: f64) -> Self {
        self.rounding_increment = rounding_increment;
        self
    }

    pub fn with_rounding_mode(mut self, rounding_mode: impl Into<String>) -> Self {
        self.rounding_mode = rounding_mode.into();
        self
    }

    pub fn with_included_quantity(mut self, included_quantity: f64) -> Self {
        self.included_quantity = included_quantity;
        self
    }

    pub fn with_priority(mut self, priority: u64) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_notes(mut self, notes: Option<String>) -> Self {
        self.notes = notes;
        self
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = status.into();
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
