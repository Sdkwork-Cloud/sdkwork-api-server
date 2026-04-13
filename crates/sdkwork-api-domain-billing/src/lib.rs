use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub project_id: String,
    pub units: u64,
    pub amount: f64,
}

impl LedgerEntry {
    pub fn new(project_id: impl Into<String>, units: u64, amount: f64) -> Self {
        Self {
            project_id: project_id.into(),
            units,
            amount,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BillingAccountingMode {
    PlatformCredit,
    Byok,
    Passthrough,
}

impl Default for BillingAccountingMode {
    fn default() -> Self {
        Self::PlatformCredit
    }
}

impl BillingAccountingMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PlatformCredit => "platform_credit",
            Self::Byok => "byok",
            Self::Passthrough => "passthrough",
        }
    }
}

impl FromStr for BillingAccountingMode {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "platform_credit" => Ok(Self::PlatformCredit),
            "byok" => Ok(Self::Byok),
            "passthrough" => Ok(Self::Passthrough),
            other => Err(format!("unknown billing accounting mode: {other}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BillingEventRecord {
    pub event_id: String,
    pub tenant_id: String,
    pub project_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_group_id: Option<String>,
    pub capability: String,
    pub route_key: String,
    pub usage_model: String,
    pub provider_id: String,
    pub accounting_mode: BillingAccountingMode,
    pub operation_kind: String,
    pub modality: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    #[serde(default)]
    pub units: u64,
    #[serde(default = "default_request_count")]
    pub request_count: u64,
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    #[serde(default)]
    pub total_tokens: u64,
    #[serde(default)]
    pub cache_read_tokens: u64,
    #[serde(default)]
    pub cache_write_tokens: u64,
    #[serde(default)]
    pub image_count: u64,
    #[serde(default)]
    pub audio_seconds: f64,
    #[serde(default)]
    pub video_seconds: f64,
    #[serde(default)]
    pub music_seconds: f64,
    #[serde(default)]
    pub upstream_cost: f64,
    #[serde(default)]
    pub customer_charge: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applied_routing_profile_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compiled_routing_snapshot_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_reason: Option<String>,
    pub created_at_ms: u64,
}

impl BillingEventRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        event_id: impl Into<String>,
        tenant_id: impl Into<String>,
        project_id: impl Into<String>,
        capability: impl Into<String>,
        route_key: impl Into<String>,
        usage_model: impl Into<String>,
        provider_id: impl Into<String>,
        accounting_mode: BillingAccountingMode,
        created_at_ms: u64,
    ) -> Self {
        Self {
            event_id: event_id.into(),
            tenant_id: tenant_id.into(),
            project_id: project_id.into(),
            api_key_group_id: None,
            capability: capability.into(),
            route_key: route_key.into(),
            usage_model: usage_model.into(),
            provider_id: provider_id.into(),
            accounting_mode,
            operation_kind: "request".to_owned(),
            modality: "text".to_owned(),
            api_key_hash: None,
            channel_id: None,
            reference_id: None,
            latency_ms: None,
            units: 0,
            request_count: default_request_count(),
            input_tokens: 0,
            output_tokens: 0,
            total_tokens: 0,
            cache_read_tokens: 0,
            cache_write_tokens: 0,
            image_count: 0,
            audio_seconds: 0.0,
            video_seconds: 0.0,
            music_seconds: 0.0,
            upstream_cost: 0.0,
            customer_charge: 0.0,
            applied_routing_profile_id: None,
            compiled_routing_snapshot_id: None,
            fallback_reason: None,
            created_at_ms,
        }
    }

    pub fn with_api_key_group_id(mut self, api_key_group_id: impl Into<String>) -> Self {
        self.api_key_group_id = Some(api_key_group_id.into());
        self
    }

    pub fn with_operation(
        mut self,
        operation_kind: impl Into<String>,
        modality: impl Into<String>,
    ) -> Self {
        self.operation_kind = operation_kind.into();
        self.modality = modality.into();
        self
    }

    pub fn with_request_facts(
        mut self,
        api_key_hash: Option<&str>,
        channel_id: Option<&str>,
        reference_id: Option<&str>,
        latency_ms: Option<u64>,
    ) -> Self {
        self.api_key_hash = api_key_hash.map(ToOwned::to_owned);
        self.channel_id = channel_id.map(ToOwned::to_owned);
        self.reference_id = reference_id.map(ToOwned::to_owned);
        self.latency_ms = latency_ms;
        self
    }

    pub fn with_units(mut self, units: u64) -> Self {
        self.units = units;
        self
    }

    pub fn with_request_count(mut self, request_count: u64) -> Self {
        self.request_count = request_count;
        self
    }

    pub fn with_token_usage(
        mut self,
        input_tokens: u64,
        output_tokens: u64,
        total_tokens: u64,
    ) -> Self {
        self.input_tokens = input_tokens;
        self.output_tokens = output_tokens;
        self.total_tokens = total_tokens;
        self
    }

    pub fn with_cache_token_usage(
        mut self,
        cache_read_tokens: u64,
        cache_write_tokens: u64,
    ) -> Self {
        self.cache_read_tokens = cache_read_tokens;
        self.cache_write_tokens = cache_write_tokens;
        self
    }

    pub fn with_media_usage(
        mut self,
        image_count: u64,
        audio_seconds: f64,
        video_seconds: f64,
        music_seconds: f64,
    ) -> Self {
        self.image_count = image_count;
        self.audio_seconds = audio_seconds;
        self.video_seconds = video_seconds;
        self.music_seconds = music_seconds;
        self
    }

    pub fn with_financials(mut self, upstream_cost: f64, customer_charge: f64) -> Self {
        self.upstream_cost = upstream_cost;
        self.customer_charge = customer_charge;
        self
    }

    pub fn with_routing_evidence(
        mut self,
        applied_routing_profile_id: Option<&str>,
        compiled_routing_snapshot_id: Option<&str>,
        fallback_reason: Option<&str>,
    ) -> Self {
        self.applied_routing_profile_id = applied_routing_profile_id.map(ToOwned::to_owned);
        self.compiled_routing_snapshot_id = compiled_routing_snapshot_id.map(ToOwned::to_owned);
        self.fallback_reason = fallback_reason.map(ToOwned::to_owned);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuotaPolicy {
    pub policy_id: String,
    pub project_id: String,
    pub max_units: u64,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

impl QuotaPolicy {
    pub fn new(
        policy_id: impl Into<String>,
        project_id: impl Into<String>,
        max_units: u64,
    ) -> Self {
        Self {
            policy_id: policy_id.into(),
            project_id: project_id.into(),
            max_units,
            enabled: true,
        }
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuotaCheckResult {
    pub allowed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_id: Option<String>,
    pub requested_units: u64,
    pub used_units: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit_units: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remaining_units: Option<u64>,
}

impl QuotaCheckResult {
    pub fn allowed_without_policy(requested_units: u64, used_units: u64) -> Self {
        Self {
            allowed: true,
            policy_id: None,
            requested_units,
            used_units,
            limit_units: None,
            remaining_units: None,
        }
    }

    pub fn from_policy(policy: &QuotaPolicy, used_units: u64, requested_units: u64) -> Self {
        let remaining_units = policy.max_units.saturating_sub(used_units);
        Self {
            allowed: used_units.saturating_add(requested_units) <= policy.max_units,
            policy_id: Some(policy.policy_id.clone()),
            requested_units,
            used_units,
            limit_units: Some(policy.max_units),
            remaining_units: Some(remaining_units),
        }
    }
}

fn default_enabled() -> bool {
    true
}

fn default_request_count() -> u64 {
    1
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectBillingSummary {
    pub project_id: String,
    pub entry_count: u64,
    pub used_units: u64,
    pub booked_amount: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub balance_source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quota_policy_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quota_limit_units: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quota_remaining_units: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remaining_units: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_account_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_available_balance: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_held_balance: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_grant_balance: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_consumed_balance: Option<f64>,
    #[serde(default)]
    pub exhausted: bool,
}

impl ProjectBillingSummary {
    pub fn new(project_id: impl Into<String>) -> Self {
        Self {
            project_id: project_id.into(),
            entry_count: 0,
            used_units: 0,
            booked_amount: 0.0,
            balance_source: None,
            quota_policy_id: None,
            quota_limit_units: None,
            quota_remaining_units: None,
            remaining_units: None,
            canonical_account_id: None,
            canonical_available_balance: None,
            canonical_held_balance: None,
            canonical_grant_balance: None,
            canonical_consumed_balance: None,
            exhausted: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BillingSummary {
    pub total_entries: u64,
    pub project_count: u64,
    pub total_units: u64,
    pub total_amount: f64,
    pub active_quota_policy_count: u64,
    pub exhausted_project_count: u64,
    pub projects: Vec<ProjectBillingSummary>,
}

impl BillingSummary {
    pub fn empty() -> Self {
        Self {
            total_entries: 0,
            project_count: 0,
            total_units: 0,
            total_amount: 0.0,
            active_quota_policy_count: 0,
            exhausted_project_count: 0,
            projects: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BillingEventProjectSummary {
    pub project_id: String,
    pub event_count: u64,
    pub request_count: u64,
    pub total_units: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_tokens: u64,
    pub total_image_count: u64,
    pub total_audio_seconds: f64,
    pub total_video_seconds: f64,
    pub total_music_seconds: f64,
    pub total_upstream_cost: f64,
    pub total_customer_charge: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BillingEventGroupSummary {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_group_id: Option<String>,
    pub project_count: u64,
    pub event_count: u64,
    pub request_count: u64,
    pub total_upstream_cost: f64,
    pub total_customer_charge: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BillingEventCapabilitySummary {
    pub capability: String,
    pub event_count: u64,
    pub request_count: u64,
    pub total_tokens: u64,
    pub image_count: u64,
    pub audio_seconds: f64,
    pub video_seconds: f64,
    pub music_seconds: f64,
    pub total_upstream_cost: f64,
    pub total_customer_charge: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BillingEventAccountingModeSummary {
    pub accounting_mode: BillingAccountingMode,
    pub event_count: u64,
    pub request_count: u64,
    pub total_upstream_cost: f64,
    pub total_customer_charge: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BillingEventSummary {
    pub total_events: u64,
    pub project_count: u64,
    pub group_count: u64,
    pub capability_count: u64,
    pub total_request_count: u64,
    pub total_units: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_tokens: u64,
    pub total_image_count: u64,
    pub total_audio_seconds: f64,
    pub total_video_seconds: f64,
    pub total_music_seconds: f64,
    pub total_upstream_cost: f64,
    pub total_customer_charge: f64,
    pub projects: Vec<BillingEventProjectSummary>,
    pub groups: Vec<BillingEventGroupSummary>,
    pub capabilities: Vec<BillingEventCapabilitySummary>,
    pub accounting_modes: Vec<BillingEventAccountingModeSummary>,
}

impl BillingEventSummary {
    pub fn empty() -> Self {
        Self {
            total_events: 0,
            project_count: 0,
            group_count: 0,
            capability_count: 0,
            total_request_count: 0,
            total_units: 0,
            total_input_tokens: 0,
            total_output_tokens: 0,
            total_tokens: 0,
            total_image_count: 0,
            total_audio_seconds: 0.0,
            total_video_seconds: 0.0,
            total_music_seconds: 0.0,
            total_upstream_cost: 0.0,
            total_customer_charge: 0.0,
            projects: Vec::new(),
            groups: Vec::new(),
            capabilities: Vec::new(),
            accounting_modes: Vec::new(),
        }
    }
}

pub type AccountId = u64;
pub type BenefitLotId = u64;
pub type HoldId = u64;
pub type RequestId = u64;
pub type PricingPlanId = u64;
pub type PricingRateId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    Primary,
    Grant,
    Postpaid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountStatus {
    Active,
    Suspended,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountBenefitSourceType {
    Recharge,
    Coupon,
    Grant,
    Order,
    ManualAdjustment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountBenefitLotStatus {
    Active,
    Exhausted,
    Expired,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountHoldStatus {
    Held,
    Captured,
    PartiallyReleased,
    Released,
    Expired,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountLedgerEntryType {
    HoldCreate,
    HoldRelease,
    SettlementCapture,
    GrantIssue,
    ManualAdjustment,
    Refund,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestSettlementStatus {
    Pending,
    Captured,
    PartiallyReleased,
    Released,
    Refunded,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
pub struct PricingRateRecord {
    pub pricing_rate_id: PricingRateId,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub pricing_plan_id: PricingPlanId,
    pub metric_code: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_code: Option<String>,
    pub quantity_step: f64,
    pub unit_price: f64,
    pub created_at_ms: u64,
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
            model_code: None,
            provider_code: None,
            quantity_step: 1.0,
            unit_price: 0.0,
            created_at_ms: 0,
        }
    }

    pub fn with_model_code(mut self, model_code: Option<String>) -> Self {
        self.model_code = model_code;
        self
    }

    pub fn with_provider_code(mut self, provider_code: Option<String>) -> Self {
        self.provider_code = provider_code;
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

    pub fn with_created_at_ms(mut self, created_at_ms: u64) -> Self {
        self.created_at_ms = created_at_ms;
        self
    }
}
