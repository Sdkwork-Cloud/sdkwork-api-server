use super::*;

pub type AccountId = u64;
pub type BenefitLotId = u64;
pub type HoldId = u64;
pub type RequestId = u64;
pub type PricingPlanId = u64;
pub type PricingRateId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    Primary,
    Grant,
    Postpaid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AccountStatus {
    Active,
    Suspended,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct AccountRecord {
    pub account_id: AccountId,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub user_id: u64,
    pub account_type: AccountType,
    pub currency_code: String,
    pub credit_unit_code: String,
    pub status: AccountStatus,
    pub allow_overdraft: bool,
    pub overdraft_limit: f64,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl AccountRecord {
    pub fn new(
        account_id: AccountId,
        tenant_id: u64,
        organization_id: u64,
        user_id: u64,
        account_type: AccountType,
    ) -> Self {
        Self {
            account_id,
            tenant_id,
            organization_id,
            user_id,
            account_type,
            currency_code: "USD".to_owned(),
            credit_unit_code: "credit".to_owned(),
            status: AccountStatus::Active,
            allow_overdraft: false,
            overdraft_limit: 0.0,
            created_at_ms: 0,
            updated_at_ms: 0,
        }
    }

    pub fn with_currency_code(mut self, currency_code: impl Into<String>) -> Self {
        self.currency_code = currency_code.into();
        self
    }

    pub fn with_credit_unit_code(mut self, credit_unit_code: impl Into<String>) -> Self {
        self.credit_unit_code = credit_unit_code.into();
        self
    }

    pub fn with_status(mut self, status: AccountStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_allow_overdraft(mut self, allow_overdraft: bool) -> Self {
        self.allow_overdraft = allow_overdraft;
        self
    }

    pub fn with_overdraft_limit(mut self, overdraft_limit: f64) -> Self {
        self.overdraft_limit = overdraft_limit;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AccountBenefitType {
    CashCredit,
    PromoCredit,
    RequestAllowance,
    TokenAllowance,
    ImageAllowance,
    AudioAllowance,
    VideoAllowance,
    MusicAllowance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AccountBenefitSourceType {
    Recharge,
    Coupon,
    Grant,
    Order,
    ManualAdjustment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AccountBenefitLotStatus {
    Active,
    Exhausted,
    Expired,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct AccountBenefitLotRecord {
    pub lot_id: BenefitLotId,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub account_id: AccountId,
    pub user_id: u64,
    pub benefit_type: AccountBenefitType,
    pub source_type: AccountBenefitSourceType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_json: Option<String>,
    pub original_quantity: f64,
    pub remaining_quantity: f64,
    pub held_quantity: f64,
    pub priority: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acquired_unit_cost: Option<f64>,
    pub issued_at_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at_ms: Option<u64>,
    pub status: AccountBenefitLotStatus,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl AccountBenefitLotRecord {
    pub fn new(
        lot_id: BenefitLotId,
        tenant_id: u64,
        organization_id: u64,
        account_id: AccountId,
        user_id: u64,
        benefit_type: AccountBenefitType,
    ) -> Self {
        Self {
            lot_id,
            tenant_id,
            organization_id,
            account_id,
            user_id,
            benefit_type,
            source_type: AccountBenefitSourceType::Grant,
            source_id: None,
            scope_json: None,
            original_quantity: 0.0,
            remaining_quantity: 0.0,
            held_quantity: 0.0,
            priority: 0,
            acquired_unit_cost: None,
            issued_at_ms: 0,
            expires_at_ms: None,
            status: AccountBenefitLotStatus::Active,
            created_at_ms: 0,
            updated_at_ms: 0,
        }
    }

    pub fn with_source_type(mut self, source_type: AccountBenefitSourceType) -> Self {
        self.source_type = source_type;
        self
    }

    pub fn with_source_id(mut self, source_id: Option<u64>) -> Self {
        self.source_id = source_id;
        self
    }

    pub fn with_scope_json(mut self, scope_json: Option<String>) -> Self {
        self.scope_json = scope_json;
        self
    }

    pub fn with_original_quantity(mut self, original_quantity: f64) -> Self {
        self.original_quantity = original_quantity;
        self
    }

    pub fn with_remaining_quantity(mut self, remaining_quantity: f64) -> Self {
        self.remaining_quantity = remaining_quantity;
        self
    }

    pub fn with_held_quantity(mut self, held_quantity: f64) -> Self {
        self.held_quantity = held_quantity;
        self
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_acquired_unit_cost(mut self, acquired_unit_cost: Option<f64>) -> Self {
        self.acquired_unit_cost = acquired_unit_cost;
        self
    }

    pub fn with_issued_at_ms(mut self, issued_at_ms: u64) -> Self {
        self.issued_at_ms = issued_at_ms;
        self
    }

    pub fn with_expires_at_ms(mut self, expires_at_ms: Option<u64>) -> Self {
        self.expires_at_ms = expires_at_ms;
        self
    }

    pub fn with_status(mut self, status: AccountBenefitLotStatus) -> Self {
        self.status = status;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AccountHoldStatus {
    Held,
    Captured,
    PartiallyReleased,
    Released,
    Expired,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct AccountHoldRecord {
    pub hold_id: HoldId,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub account_id: AccountId,
    pub user_id: u64,
    pub request_id: RequestId,
    pub status: AccountHoldStatus,
    pub estimated_quantity: f64,
    pub captured_quantity: f64,
    pub released_quantity: f64,
    pub expires_at_ms: u64,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl AccountHoldRecord {
    pub fn new(
        hold_id: HoldId,
        tenant_id: u64,
        organization_id: u64,
        account_id: AccountId,
        user_id: u64,
        request_id: RequestId,
    ) -> Self {
        Self {
            hold_id,
            tenant_id,
            organization_id,
            account_id,
            user_id,
            request_id,
            status: AccountHoldStatus::Held,
            estimated_quantity: 0.0,
            captured_quantity: 0.0,
            released_quantity: 0.0,
            expires_at_ms: 0,
            created_at_ms: 0,
            updated_at_ms: 0,
        }
    }

    pub fn with_status(mut self, status: AccountHoldStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_estimated_quantity(mut self, estimated_quantity: f64) -> Self {
        self.estimated_quantity = estimated_quantity;
        self
    }

    pub fn with_captured_quantity(mut self, captured_quantity: f64) -> Self {
        self.captured_quantity = captured_quantity;
        self
    }

    pub fn with_released_quantity(mut self, released_quantity: f64) -> Self {
        self.released_quantity = released_quantity;
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountHoldAllocationRecord {
    pub hold_allocation_id: u64,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub hold_id: HoldId,
    pub lot_id: BenefitLotId,
    pub allocated_quantity: f64,
    pub captured_quantity: f64,
    pub released_quantity: f64,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl AccountHoldAllocationRecord {
    pub fn new(
        hold_allocation_id: u64,
        tenant_id: u64,
        organization_id: u64,
        hold_id: HoldId,
        lot_id: BenefitLotId,
    ) -> Self {
        Self {
            hold_allocation_id,
            tenant_id,
            organization_id,
            hold_id,
            lot_id,
            allocated_quantity: 0.0,
            captured_quantity: 0.0,
            released_quantity: 0.0,
            created_at_ms: 0,
            updated_at_ms: 0,
        }
    }

    pub fn with_allocated_quantity(mut self, allocated_quantity: f64) -> Self {
        self.allocated_quantity = allocated_quantity;
        self
    }

    pub fn with_captured_quantity(mut self, captured_quantity: f64) -> Self {
        self.captured_quantity = captured_quantity;
        self
    }

    pub fn with_released_quantity(mut self, released_quantity: f64) -> Self {
        self.released_quantity = released_quantity;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AccountLedgerEntryType {
    HoldCreate,
    HoldRelease,
    SettlementCapture,
    GrantIssue,
    ManualAdjustment,
    Refund,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct AccountLedgerEntryRecord {
    pub ledger_entry_id: u64,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub account_id: AccountId,
    pub user_id: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<RequestId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hold_id: Option<HoldId>,
    pub entry_type: AccountLedgerEntryType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub benefit_type: Option<String>,
    pub quantity: f64,
    pub amount: f64,
    pub created_at_ms: u64,
}

impl AccountLedgerEntryRecord {
    pub fn new(
        ledger_entry_id: u64,
        tenant_id: u64,
        organization_id: u64,
        account_id: AccountId,
        user_id: u64,
        entry_type: AccountLedgerEntryType,
    ) -> Self {
        Self {
            ledger_entry_id,
            tenant_id,
            organization_id,
            account_id,
            user_id,
            request_id: None,
            hold_id: None,
            entry_type,
            benefit_type: None,
            quantity: 0.0,
            amount: 0.0,
            created_at_ms: 0,
        }
    }

    pub fn with_request_id(mut self, request_id: Option<RequestId>) -> Self {
        self.request_id = request_id;
        self
    }

    pub fn with_hold_id(mut self, hold_id: Option<HoldId>) -> Self {
        self.hold_id = hold_id;
        self
    }

    pub fn with_benefit_type(mut self, benefit_type: Option<String>) -> Self {
        self.benefit_type = benefit_type;
        self
    }

    pub fn with_quantity(mut self, quantity: f64) -> Self {
        self.quantity = quantity;
        self
    }

    pub fn with_amount(mut self, amount: f64) -> Self {
        self.amount = amount;
        self
    }

    pub fn with_created_at_ms(mut self, created_at_ms: u64) -> Self {
        self.created_at_ms = created_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct AccountLedgerAllocationRecord {
    pub ledger_allocation_id: u64,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub ledger_entry_id: u64,
    pub lot_id: BenefitLotId,
    pub quantity_delta: f64,
    pub created_at_ms: u64,
}

impl AccountLedgerAllocationRecord {
    pub fn new(
        ledger_allocation_id: u64,
        tenant_id: u64,
        organization_id: u64,
        ledger_entry_id: u64,
        lot_id: BenefitLotId,
    ) -> Self {
        Self {
            ledger_allocation_id,
            tenant_id,
            organization_id,
            ledger_entry_id,
            lot_id,
            quantity_delta: 0.0,
            created_at_ms: 0,
        }
    }

    pub fn with_quantity_delta(mut self, quantity_delta: f64) -> Self {
        self.quantity_delta = quantity_delta;
        self
    }

    pub fn with_created_at_ms(mut self, created_at_ms: u64) -> Self {
        self.created_at_ms = created_at_ms;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RequestSettlementStatus {
    Pending,
    Captured,
    PartiallyReleased,
    Released,
    Refunded,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct RequestSettlementRecord {
    pub request_settlement_id: u64,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub request_id: RequestId,
    pub account_id: AccountId,
    pub user_id: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hold_id: Option<HoldId>,
    pub status: RequestSettlementStatus,
    pub estimated_credit_hold: f64,
    pub released_credit_amount: f64,
    pub captured_credit_amount: f64,
    pub provider_cost_amount: f64,
    pub retail_charge_amount: f64,
    pub shortfall_amount: f64,
    pub refunded_amount: f64,
    pub settled_at_ms: u64,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl RequestSettlementRecord {
    pub fn new(
        request_settlement_id: u64,
        tenant_id: u64,
        organization_id: u64,
        request_id: RequestId,
        account_id: AccountId,
        user_id: u64,
    ) -> Self {
        Self {
            request_settlement_id,
            tenant_id,
            organization_id,
            request_id,
            account_id,
            user_id,
            hold_id: None,
            status: RequestSettlementStatus::Pending,
            estimated_credit_hold: 0.0,
            released_credit_amount: 0.0,
            captured_credit_amount: 0.0,
            provider_cost_amount: 0.0,
            retail_charge_amount: 0.0,
            shortfall_amount: 0.0,
            refunded_amount: 0.0,
            settled_at_ms: 0,
            created_at_ms: 0,
            updated_at_ms: 0,
        }
    }

    pub fn with_hold_id(mut self, hold_id: Option<HoldId>) -> Self {
        self.hold_id = hold_id;
        self
    }

    pub fn with_status(mut self, status: RequestSettlementStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_estimated_credit_hold(mut self, estimated_credit_hold: f64) -> Self {
        self.estimated_credit_hold = estimated_credit_hold;
        self
    }

    pub fn with_released_credit_amount(mut self, released_credit_amount: f64) -> Self {
        self.released_credit_amount = released_credit_amount;
        self
    }

    pub fn with_captured_credit_amount(mut self, captured_credit_amount: f64) -> Self {
        self.captured_credit_amount = captured_credit_amount;
        self
    }

    pub fn with_provider_cost_amount(mut self, provider_cost_amount: f64) -> Self {
        self.provider_cost_amount = provider_cost_amount;
        self
    }

    pub fn with_retail_charge_amount(mut self, retail_charge_amount: f64) -> Self {
        self.retail_charge_amount = retail_charge_amount;
        self
    }

    pub fn with_shortfall_amount(mut self, shortfall_amount: f64) -> Self {
        self.shortfall_amount = shortfall_amount;
        self
    }

    pub fn with_refunded_amount(mut self, refunded_amount: f64) -> Self {
        self.refunded_amount = refunded_amount;
        self
    }

    pub fn with_settled_at_ms(mut self, settled_at_ms: u64) -> Self {
        self.settled_at_ms = settled_at_ms;
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
