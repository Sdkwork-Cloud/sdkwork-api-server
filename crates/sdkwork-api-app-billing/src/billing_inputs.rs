use super::*;

pub fn service_name() -> &'static str {
    "billing-service"
}

pub struct CreateBillingEventInput<'a> {
    pub event_id: &'a str,
    pub tenant_id: &'a str,
    pub project_id: &'a str,
    pub api_key_group_id: Option<&'a str>,
    pub capability: &'a str,
    pub route_key: &'a str,
    pub usage_model: &'a str,
    pub provider_id: &'a str,
    pub accounting_mode: BillingAccountingMode,
    pub operation_kind: &'a str,
    pub modality: &'a str,
    pub api_key_hash: Option<&'a str>,
    pub channel_id: Option<&'a str>,
    pub reference_id: Option<&'a str>,
    pub latency_ms: Option<u64>,
    pub units: u64,
    pub request_count: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
    pub image_count: u64,
    pub audio_seconds: f64,
    pub video_seconds: f64,
    pub music_seconds: f64,
    pub upstream_cost: f64,
    pub customer_charge: f64,
    pub applied_routing_profile_id: Option<&'a str>,
    pub compiled_routing_snapshot_id: Option<&'a str>,
    pub fallback_reason: Option<&'a str>,
    pub created_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct AccountLotBalanceSnapshot {
    pub lot_id: u64,
    pub benefit_type: AccountBenefitType,
    pub scope_json: Option<String>,
    pub expires_at_ms: Option<u64>,
    pub original_quantity: f64,
    pub remaining_quantity: f64,
    pub held_quantity: f64,
    pub available_quantity: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct AccountBalanceSnapshot {
    pub account_id: u64,
    pub available_balance: f64,
    pub held_balance: f64,
    pub consumed_balance: f64,
    pub grant_balance: f64,
    pub active_lot_count: u64,
    pub lots: Vec<AccountLotBalanceSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct AccountLedgerHistoryEntry {
    pub entry: AccountLedgerEntryRecord,
    pub allocations: Vec<AccountLedgerAllocationRecord>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlannedHoldAllocation {
    pub lot_id: u64,
    pub quantity: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AccountHoldPlan {
    pub account_id: u64,
    pub requested_quantity: f64,
    pub covered_quantity: f64,
    pub shortfall_quantity: f64,
    pub sufficient_balance: bool,
    pub allocations: Vec<PlannedHoldAllocation>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CreateAccountHoldInput {
    pub hold_id: u64,
    pub hold_allocation_start_id: u64,
    pub request_id: u64,
    pub account_id: u64,
    pub requested_quantity: f64,
    pub expires_at_ms: u64,
    pub now_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReleaseAccountHoldInput {
    pub request_id: u64,
    pub released_at_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CaptureAccountHoldInput {
    pub request_settlement_id: u64,
    pub request_id: u64,
    pub captured_quantity: f64,
    pub provider_cost_amount: f64,
    pub retail_charge_amount: f64,
    pub settled_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AccountHoldMutationResult {
    pub idempotent_replay: bool,
    pub hold: AccountHoldRecord,
    pub allocations: Vec<AccountHoldAllocationRecord>,
    pub updated_lots: Vec<AccountBenefitLotRecord>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CaptureAccountHoldResult {
    pub idempotent_replay: bool,
    pub hold: AccountHoldRecord,
    pub allocations: Vec<AccountHoldAllocationRecord>,
    pub updated_lots: Vec<AccountBenefitLotRecord>,
    pub settlement: RequestSettlementRecord,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RefundAccountSettlementInput {
    pub request_settlement_id: u64,
    pub refund_ledger_entry_id: u64,
    pub refund_ledger_allocation_start_id: u64,
    pub refunded_amount: f64,
    pub refunded_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RefundAccountSettlementResult {
    pub idempotent_replay: bool,
    pub settlement: RequestSettlementRecord,
    pub updated_lots: Vec<AccountBenefitLotRecord>,
    pub ledger_entry: AccountLedgerEntryRecord,
    pub ledger_allocations: Vec<AccountLedgerAllocationRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IssueCommerceOrderCreditsInput<'a> {
    pub account_id: u64,
    pub order_id: &'a str,
    pub project_id: &'a str,
    pub target_kind: &'a str,
    pub granted_quantity: f64,
    pub payable_amount: f64,
    pub issued_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IssueCommerceOrderCreditsResult {
    pub idempotent_replay: bool,
    pub lot: AccountBenefitLotRecord,
    pub ledger_entry: AccountLedgerEntryRecord,
    pub ledger_allocations: Vec<AccountLedgerAllocationRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RefundCommerceOrderCreditsInput<'a> {
    pub account_id: u64,
    pub order_id: &'a str,
    pub refunded_quantity: f64,
    pub refunded_amount: f64,
    pub refunded_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RefundCommerceOrderCreditsResult {
    pub idempotent_replay: bool,
    pub lot: AccountBenefitLotRecord,
    pub ledger_entry: AccountLedgerEntryRecord,
    pub ledger_allocations: Vec<AccountLedgerAllocationRecord>,
}

