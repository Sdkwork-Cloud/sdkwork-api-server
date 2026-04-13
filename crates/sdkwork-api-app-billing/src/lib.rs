use anyhow::{ensure, Result};
use async_trait::async_trait;
use sdkwork_api_domain_billing::{
    AccountBenefitLotRecord, AccountBenefitLotStatus, AccountBenefitType,
    AccountHoldAllocationRecord, AccountHoldRecord, AccountHoldStatus,
    AccountLedgerAllocationRecord, AccountLedgerEntryRecord, AccountLedgerEntryType, AccountRecord,
    AccountStatus, AccountType, BillingEventAccountingModeSummary, BillingEventCapabilitySummary,
    BillingEventGroupSummary, BillingEventProjectSummary, BillingEventRecord, BillingEventSummary,
    BillingSummary, LedgerEntry, ProjectBillingSummary, RequestSettlementRecord,
    RequestSettlementStatus,
};
use sdkwork_api_domain_identity::GatewayAuthSubject;
use sdkwork_api_domain_usage::{RequestMeterFactRecord, RequestStatus, UsageCaptureStatus};
use sdkwork_api_policy_quota::{
    builtin_quota_policy_registry, QuotaPolicyExecutionInput, STRICTEST_LIMIT_QUOTA_POLICY_ID,
};
use sdkwork_api_storage_core::{AccountKernelCommandBatch, AccountKernelStore, AdminStore};
use std::collections::{BTreeMap, BTreeSet};

pub use sdkwork_api_domain_billing::{BillingAccountingMode, QuotaCheckResult, QuotaPolicy};

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

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct AccountBalanceSnapshot {
    pub account_id: u64,
    pub available_balance: f64,
    pub held_balance: f64,
    pub consumed_balance: f64,
    pub grant_balance: f64,
    pub active_lot_count: u64,
    pub lots: Vec<AccountLotBalanceSnapshot>,
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

#[derive(Debug, Clone, PartialEq)]
pub struct CreateAccountHoldInput {
    pub hold_id: u64,
    pub account_id: u64,
    pub request_id: u64,
    pub estimated_quantity: f64,
    pub expires_at_ms: u64,
    pub now_ms: u64,
    pub request_meter_fact: RequestMeterFactRecord,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CaptureAccountHoldInput {
    pub hold_id: u64,
    pub captured_quantity: f64,
    pub provider_cost_amount: f64,
    pub retail_charge_amount: f64,
    pub settled_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseAccountHoldInput {
    pub hold_id: u64,
    pub settled_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreatedAccountHold {
    pub hold: AccountHoldRecord,
    pub hold_allocations: Vec<AccountHoldAllocationRecord>,
    pub request_meter_fact: RequestMeterFactRecord,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SettledAccountHold {
    pub hold: AccountHoldRecord,
    pub settlement: RequestSettlementRecord,
    pub request_meter_fact: RequestMeterFactRecord,
}

pub fn create_quota_policy(
    policy_id: &str,
    project_id: &str,
    max_units: u64,
    enabled: bool,
) -> Result<QuotaPolicy> {
    ensure!(!policy_id.trim().is_empty(), "policy_id must not be empty");
    ensure!(
        !project_id.trim().is_empty(),
        "project_id must not be empty"
    );
    ensure!(max_units > 0, "max_units must be greater than 0");

    Ok(QuotaPolicy::new(policy_id, project_id, max_units).with_enabled(enabled))
}

pub fn book_usage_cost(project_id: &str, units: u64, amount: f64) -> Result<LedgerEntry> {
    Ok(LedgerEntry::new(project_id, units, amount))
}

pub fn create_billing_event(input: CreateBillingEventInput<'_>) -> Result<BillingEventRecord> {
    ensure!(
        !input.event_id.trim().is_empty(),
        "event_id must not be empty"
    );
    ensure!(
        !input.tenant_id.trim().is_empty(),
        "tenant_id must not be empty"
    );
    ensure!(
        !input.project_id.trim().is_empty(),
        "project_id must not be empty"
    );
    ensure!(
        !input.capability.trim().is_empty(),
        "capability must not be empty"
    );
    ensure!(
        !input.usage_model.trim().is_empty(),
        "usage_model must not be empty"
    );
    ensure!(
        !input.provider_id.trim().is_empty(),
        "provider_id must not be empty"
    );
    ensure!(
        !input.operation_kind.trim().is_empty(),
        "operation_kind must not be empty"
    );
    ensure!(
        !input.modality.trim().is_empty(),
        "modality must not be empty"
    );
    ensure!(
        input.upstream_cost >= 0.0,
        "upstream_cost must not be negative"
    );
    ensure!(
        input.customer_charge >= 0.0,
        "customer_charge must not be negative"
    );

    let route_key = if input.route_key.trim().is_empty() {
        input.usage_model.trim()
    } else {
        input.route_key.trim()
    };
    let request_count = input.request_count.max(1);
    let total_tokens = if input.total_tokens == 0 {
        input.input_tokens.saturating_add(input.output_tokens)
    } else {
        input.total_tokens
    };

    let mut event = BillingEventRecord::new(
        input.event_id.trim(),
        input.tenant_id.trim(),
        input.project_id.trim(),
        input.capability.trim(),
        route_key,
        input.usage_model.trim(),
        input.provider_id.trim(),
        input.accounting_mode,
        input.created_at_ms,
    )
    .with_operation(input.operation_kind.trim(), input.modality.trim())
    .with_request_facts(
        input.api_key_hash.map(str::trim),
        input.channel_id.map(str::trim),
        input.reference_id.map(str::trim),
        input.latency_ms,
    )
    .with_units(input.units)
    .with_request_count(request_count)
    .with_token_usage(input.input_tokens, input.output_tokens, total_tokens)
    .with_cache_token_usage(input.cache_read_tokens, input.cache_write_tokens)
    .with_media_usage(
        input.image_count,
        input.audio_seconds,
        input.video_seconds,
        input.music_seconds,
    )
    .with_financials(input.upstream_cost, input.customer_charge)
    .with_routing_evidence(
        input.applied_routing_profile_id.map(str::trim),
        input.compiled_routing_snapshot_id.map(str::trim),
        input.fallback_reason.map(str::trim),
    );

    if let Some(api_key_group_id) = input.api_key_group_id.map(str::trim) {
        if !api_key_group_id.is_empty() {
            event = event.with_api_key_group_id(api_key_group_id);
        }
    }

    Ok(event)
}

pub async fn summarize_account_balance<S>(
    store: &S,
    account_id: u64,
    now_ms: u64,
) -> Result<AccountBalanceSnapshot>
where
    S: AccountKernelStore + ?Sized,
{
    ensure!(
        store.find_account_record(account_id).await?.is_some(),
        "account {account_id} does not exist"
    );

    let account_lots = store
        .list_account_benefit_lots()
        .await?
        .into_iter()
        .filter(|lot| lot.account_id == account_id)
        .collect::<Vec<_>>();
    let active_lots = eligible_lots_for_hold(&account_lots, now_ms);

    let available_balance = active_lots.iter().map(|lot| free_quantity(lot)).sum();
    let held_balance = account_lots.iter().map(|lot| lot.held_quantity).sum();
    let consumed_balance = account_lots
        .iter()
        .map(|lot| (lot.original_quantity - lot.remaining_quantity).max(0.0))
        .sum();
    let grant_balance = account_lots.iter().map(|lot| lot.original_quantity).sum();
    let lots = active_lots
        .into_iter()
        .map(|lot| AccountLotBalanceSnapshot {
            lot_id: lot.lot_id,
            benefit_type: lot.benefit_type,
            scope_json: lot.scope_json.clone(),
            expires_at_ms: lot.expires_at_ms,
            original_quantity: lot.original_quantity,
            remaining_quantity: lot.remaining_quantity,
            held_quantity: lot.held_quantity,
            available_quantity: free_quantity(lot),
        })
        .collect::<Vec<_>>();

    Ok(AccountBalanceSnapshot {
        account_id,
        available_balance,
        held_balance,
        consumed_balance,
        grant_balance,
        active_lot_count: lots.len() as u64,
        lots,
    })
}

pub async fn plan_account_hold<S>(
    store: &S,
    account_id: u64,
    requested_quantity: f64,
    now_ms: u64,
) -> Result<AccountHoldPlan>
where
    S: AccountKernelStore + ?Sized,
{
    ensure!(
        requested_quantity > 0.0,
        "requested_quantity must be positive"
    );

    ensure!(
        store.find_account_record(account_id).await?.is_some(),
        "account {account_id} does not exist"
    );

    let lots = store
        .list_account_benefit_lots()
        .await?
        .into_iter()
        .filter(|lot| lot.account_id == account_id)
        .collect::<Vec<_>>();
    let eligible_lots = eligible_lots_for_hold(&lots, now_ms);
    let mut remaining = requested_quantity;
    let mut allocations = Vec::new();

    for lot in eligible_lots {
        if remaining <= 0.0 {
            break;
        }
        let quantity = free_quantity(lot).min(remaining);
        if quantity <= 0.0 {
            continue;
        }
        allocations.push(PlannedHoldAllocation {
            lot_id: lot.lot_id,
            quantity,
        });
        remaining -= quantity;
    }

    let covered_quantity = requested_quantity - remaining.max(0.0);
    let shortfall_quantity = remaining.max(0.0);

    Ok(AccountHoldPlan {
        account_id,
        requested_quantity,
        covered_quantity,
        shortfall_quantity,
        sufficient_balance: shortfall_quantity <= f64::EPSILON,
        allocations,
    })
}

pub async fn create_account_hold<S>(
    store: &S,
    input: CreateAccountHoldInput,
) -> Result<CreatedAccountHold>
where
    S: AccountKernelStore + ?Sized,
{
    ensure!(
        input.estimated_quantity > 0.0,
        "estimated_quantity must be positive"
    );
    ensure!(
        input.request_meter_fact.request_id == input.request_id,
        "request meter fact request_id must match hold request_id"
    );

    let account = store
        .find_account_record(input.account_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("account {} does not exist", input.account_id))?;
    ensure!(
        account.status == AccountStatus::Active,
        "account {} is not active",
        account.account_id
    );

    if let Some(existing_hold) = store
        .list_account_holds()
        .await?
        .into_iter()
        .find(|hold| hold.hold_id == input.hold_id || hold.request_id == input.request_id)
    {
        let existing_allocations = store
            .list_account_hold_allocations()
            .await?
            .into_iter()
            .filter(|allocation| allocation.hold_id == existing_hold.hold_id)
            .collect::<Vec<_>>();
        let existing_request_meter_fact = store
            .list_request_meter_facts()
            .await?
            .into_iter()
            .find(|record| record.request_id == existing_hold.request_id)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "request meter fact {} missing for existing hold {}",
                    existing_hold.request_id,
                    existing_hold.hold_id
                )
            })?;
        return Ok(CreatedAccountHold {
            hold: existing_hold,
            hold_allocations: existing_allocations,
            request_meter_fact: existing_request_meter_fact,
        });
    }

    let hold_plan = plan_account_hold(
        store,
        input.account_id,
        input.estimated_quantity,
        input.now_ms,
    )
    .await?;
    ensure!(
        hold_plan.sufficient_balance,
        "account {} has insufficient balance for hold {}",
        input.account_id,
        input.estimated_quantity
    );

    let lots_by_id = store
        .list_account_benefit_lots()
        .await?
        .into_iter()
        .filter(|lot| lot.account_id == input.account_id)
        .map(|lot| (lot.lot_id, lot))
        .collect::<BTreeMap<_, _>>();

    let mut updated_lots = Vec::new();
    let mut hold_allocations = Vec::new();
    for (index, allocation) in hold_plan.allocations.iter().enumerate() {
        let mut lot = lots_by_id
            .get(&allocation.lot_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("benefit lot {} does not exist", allocation.lot_id))?;
        lot.held_quantity += allocation.quantity;
        lot.updated_at_ms = input.now_ms;
        updated_lots.push(lot);

        hold_allocations.push(
            AccountHoldAllocationRecord::new(
                derived_allocation_id(input.hold_id, index)?,
                account.tenant_id,
                account.organization_id,
                input.hold_id,
                allocation.lot_id,
            )
            .with_allocated_quantity(allocation.quantity)
            .with_created_at_ms(input.now_ms)
            .with_updated_at_ms(input.now_ms),
        );
    }

    let hold = AccountHoldRecord::new(
        input.hold_id,
        account.tenant_id,
        account.organization_id,
        account.account_id,
        account.user_id,
        input.request_id,
    )
    .with_status(AccountHoldStatus::Held)
    .with_estimated_quantity(input.estimated_quantity)
    .with_expires_at_ms(input.expires_at_ms)
    .with_created_at_ms(input.now_ms)
    .with_updated_at_ms(input.now_ms);

    let request_meter_fact = prepare_request_meter_fact_for_hold(
        &account,
        input.request_meter_fact,
        input.estimated_quantity,
        input.now_ms,
    )?;
    let hold_create_ledger_entry = AccountLedgerEntryRecord::new(
        hold_create_ledger_entry_id(input.request_id)?,
        account.tenant_id,
        account.organization_id,
        account.account_id,
        account.user_id,
        AccountLedgerEntryType::HoldCreate,
    )
    .with_request_id(Some(input.request_id))
    .with_hold_id(Some(input.hold_id))
    .with_quantity(input.estimated_quantity)
    .with_amount(input.estimated_quantity)
    .with_created_at_ms(input.now_ms);

    let batch = AccountKernelCommandBatch {
        benefit_lot_records: updated_lots,
        hold_records: vec![hold.clone()],
        hold_allocation_records: hold_allocations.clone(),
        ledger_entry_records: vec![hold_create_ledger_entry],
        request_meter_fact_records: vec![request_meter_fact.clone()],
        ..AccountKernelCommandBatch::default()
    };
    store.commit_account_kernel_batch(&batch).await?;

    Ok(CreatedAccountHold {
        hold,
        hold_allocations,
        request_meter_fact,
    })
}

pub async fn capture_account_hold<S>(
    store: &S,
    input: CaptureAccountHoldInput,
) -> Result<SettledAccountHold>
where
    S: AccountKernelStore + ?Sized,
{
    settle_account_hold(
        store,
        input.hold_id,
        input.captured_quantity,
        input.provider_cost_amount,
        input.retail_charge_amount,
        RequestStatus::Succeeded,
        UsageCaptureStatus::Captured,
        input.settled_at_ms,
    )
    .await
}

pub async fn reconcile_account_hold<S>(
    store: &S,
    input: CaptureAccountHoldInput,
) -> Result<SettledAccountHold>
where
    S: AccountKernelStore + ?Sized,
{
    settle_account_hold(
        store,
        input.hold_id,
        input.captured_quantity,
        input.provider_cost_amount,
        input.retail_charge_amount,
        RequestStatus::Succeeded,
        UsageCaptureStatus::Reconciled,
        input.settled_at_ms,
    )
    .await
}

pub async fn release_account_hold<S>(
    store: &S,
    input: ReleaseAccountHoldInput,
) -> Result<SettledAccountHold>
where
    S: AccountKernelStore + ?Sized,
{
    settle_account_hold(
        store,
        input.hold_id,
        0.0,
        0.0,
        0.0,
        RequestStatus::Failed,
        UsageCaptureStatus::Failed,
        input.settled_at_ms,
    )
    .await
}

pub async fn resolve_payable_account_for_gateway_subject<S>(
    store: &S,
    subject: &GatewayAuthSubject,
) -> Result<Option<AccountRecord>>
where
    S: AccountKernelStore + ?Sized,
{
    let Some(account) = store
        .find_account_record_by_owner(
            subject.tenant_id,
            subject.organization_id,
            subject.user_id,
            AccountType::Primary,
        )
        .await?
    else {
        return Ok(None);
    };

    ensure!(
        account.status == AccountStatus::Active,
        "primary account {} is not active",
        account.account_id
    );

    Ok(Some(account))
}

pub async fn persist_ledger_entry(
    store: &dyn AdminStore,
    project_id: &str,
    units: u64,
    amount: f64,
) -> Result<LedgerEntry> {
    let entry = book_usage_cost(project_id, units, amount)?;
    store.insert_ledger_entry(&entry).await
}

pub async fn persist_billing_event(
    store: &dyn AdminStore,
    event: &BillingEventRecord,
) -> Result<BillingEventRecord> {
    store.insert_billing_event(event).await
}

pub async fn list_ledger_entries(store: &dyn AdminStore) -> Result<Vec<LedgerEntry>> {
    store.list_ledger_entries().await
}

pub async fn list_billing_events(store: &dyn AdminStore) -> Result<Vec<BillingEventRecord>> {
    store.list_billing_events().await
}

pub async fn persist_quota_policy(
    store: &dyn AdminStore,
    policy: &QuotaPolicy,
) -> Result<QuotaPolicy> {
    store.insert_quota_policy(policy).await
}

pub async fn list_quota_policies(store: &dyn AdminStore) -> Result<Vec<QuotaPolicy>> {
    store.list_quota_policies().await
}

#[async_trait]
pub trait BillingQuotaStore: Send + Sync {
    async fn list_ledger_entries_for_project(&self, project_id: &str) -> Result<Vec<LedgerEntry>>;
    async fn list_quota_policies_for_project(&self, project_id: &str) -> Result<Vec<QuotaPolicy>>;
}

#[async_trait]
impl<T> BillingQuotaStore for T
where
    T: AdminStore + ?Sized,
{
    async fn list_ledger_entries_for_project(&self, project_id: &str) -> Result<Vec<LedgerEntry>> {
        AdminStore::list_ledger_entries_for_project(self, project_id).await
    }

    async fn list_quota_policies_for_project(&self, project_id: &str) -> Result<Vec<QuotaPolicy>> {
        AdminStore::list_quota_policies_for_project(self, project_id).await
    }
}

pub async fn check_quota<S>(
    store: &S,
    project_id: &str,
    requested_units: u64,
) -> Result<QuotaCheckResult>
where
    S: BillingQuotaStore + ?Sized,
{
    let used_units = store
        .list_ledger_entries_for_project(project_id)
        .await?
        .into_iter()
        .map(|entry| entry.units)
        .sum();
    let policies = store.list_quota_policies_for_project(project_id).await?;
    let registry = builtin_quota_policy_registry();
    let plugin = registry
        .resolve(STRICTEST_LIMIT_QUOTA_POLICY_ID)
        .expect("builtin strictest-limit quota policy plugin must exist");

    Ok(plugin.execute(QuotaPolicyExecutionInput {
        policies: &policies,
        used_units,
        requested_units,
    }))
}

pub fn summarize_billing_snapshot(
    entries: &[LedgerEntry],
    policies: &[QuotaPolicy],
) -> BillingSummary {
    if entries.is_empty() && policies.is_empty() {
        return BillingSummary::empty();
    }

    let mut projects = BTreeMap::<String, ProjectBillingSummary>::new();

    for entry in entries {
        let summary = projects
            .entry(entry.project_id.clone())
            .or_insert_with(|| ProjectBillingSummary::new(entry.project_id.clone()));
        summary.entry_count += 1;
        summary.used_units += entry.units;
        summary.booked_amount += entry.amount;
    }

    let active_policies = policies
        .iter()
        .filter(|policy| policy.enabled)
        .collect::<Vec<_>>();

    for policy in &active_policies {
        let summary = projects
            .entry(policy.project_id.clone())
            .or_insert_with(|| ProjectBillingSummary::new(policy.project_id.clone()));
        let replace_policy = match (
            summary.quota_limit_units,
            summary.quota_policy_id.as_deref(),
        ) {
            (None, _) => true,
            (Some(current_limit), Some(current_policy_id)) => {
                policy.max_units < current_limit
                    || (policy.max_units == current_limit
                        && policy.policy_id.as_str() < current_policy_id)
            }
            (Some(_), None) => true,
        };

        if replace_policy {
            summary.quota_policy_id = Some(policy.policy_id.clone());
            summary.quota_limit_units = Some(policy.max_units);
        }
    }

    let total_entries = entries.len() as u64;
    let total_units = entries.iter().map(|entry| entry.units).sum();
    let total_amount = entries.iter().map(|entry| entry.amount).sum();

    let mut project_summaries = projects
        .into_values()
        .map(|mut summary| {
            if let Some(limit_units) = summary.quota_limit_units {
                let remaining_units = limit_units.saturating_sub(summary.used_units);
                summary.balance_source = Some("quota_policy".to_owned());
                summary.quota_remaining_units = Some(remaining_units);
                summary.remaining_units = Some(remaining_units);
                summary.exhausted = summary.used_units >= limit_units;
            }
            summary
        })
        .collect::<Vec<_>>();

    project_summaries.sort_by(|left, right| {
        right
            .quota_limit_units
            .is_some()
            .cmp(&left.quota_limit_units.is_some())
            .then_with(|| right.exhausted.cmp(&left.exhausted))
            .then_with(|| right.used_units.cmp(&left.used_units))
            .then_with(|| left.project_id.cmp(&right.project_id))
    });

    let exhausted_project_count = project_summaries
        .iter()
        .filter(|summary| summary.exhausted)
        .count() as u64;

    BillingSummary {
        total_entries,
        project_count: project_summaries.len() as u64,
        total_units,
        total_amount,
        active_quota_policy_count: active_policies.len() as u64,
        exhausted_project_count,
        projects: project_summaries,
    }
}

pub fn summarize_billing_events(events: &[BillingEventRecord]) -> BillingEventSummary {
    if events.is_empty() {
        return BillingEventSummary::empty();
    }

    #[derive(Default)]
    struct ProjectAccumulator {
        event_count: u64,
        request_count: u64,
        total_units: u64,
        total_input_tokens: u64,
        total_output_tokens: u64,
        total_tokens: u64,
        total_image_count: u64,
        total_audio_seconds: f64,
        total_video_seconds: f64,
        total_music_seconds: f64,
        total_upstream_cost: f64,
        total_customer_charge: f64,
    }

    #[derive(Default)]
    struct GroupAccumulator {
        project_ids: BTreeSet<String>,
        event_count: u64,
        request_count: u64,
        total_upstream_cost: f64,
        total_customer_charge: f64,
    }

    #[derive(Default)]
    struct CapabilityAccumulator {
        event_count: u64,
        request_count: u64,
        total_tokens: u64,
        image_count: u64,
        audio_seconds: f64,
        video_seconds: f64,
        music_seconds: f64,
        total_upstream_cost: f64,
        total_customer_charge: f64,
    }

    #[derive(Default)]
    struct AccountingModeAccumulator {
        event_count: u64,
        request_count: u64,
        total_upstream_cost: f64,
        total_customer_charge: f64,
    }

    let mut projects = BTreeMap::<String, ProjectAccumulator>::new();
    let mut groups = BTreeMap::<Option<String>, GroupAccumulator>::new();
    let mut capabilities = BTreeMap::<String, CapabilityAccumulator>::new();
    let mut accounting_modes = BTreeMap::<BillingAccountingMode, AccountingModeAccumulator>::new();

    for event in events {
        let project = projects.entry(event.project_id.clone()).or_default();
        project.event_count += 1;
        project.request_count += event.request_count;
        project.total_units += event.units;
        project.total_input_tokens += event.input_tokens;
        project.total_output_tokens += event.output_tokens;
        project.total_tokens += event.total_tokens;
        project.total_image_count += event.image_count;
        project.total_audio_seconds += event.audio_seconds;
        project.total_video_seconds += event.video_seconds;
        project.total_music_seconds += event.music_seconds;
        project.total_upstream_cost += event.upstream_cost;
        project.total_customer_charge += event.customer_charge;

        let group = groups.entry(event.api_key_group_id.clone()).or_default();
        group.project_ids.insert(event.project_id.clone());
        group.event_count += 1;
        group.request_count += event.request_count;
        group.total_upstream_cost += event.upstream_cost;
        group.total_customer_charge += event.customer_charge;

        let capability = capabilities.entry(event.capability.clone()).or_default();
        capability.event_count += 1;
        capability.request_count += event.request_count;
        capability.total_tokens += event.total_tokens;
        capability.image_count += event.image_count;
        capability.audio_seconds += event.audio_seconds;
        capability.video_seconds += event.video_seconds;
        capability.music_seconds += event.music_seconds;
        capability.total_upstream_cost += event.upstream_cost;
        capability.total_customer_charge += event.customer_charge;

        let mode = accounting_modes.entry(event.accounting_mode).or_default();
        mode.event_count += 1;
        mode.request_count += event.request_count;
        mode.total_upstream_cost += event.upstream_cost;
        mode.total_customer_charge += event.customer_charge;
    }

    let mut project_summaries = projects
        .into_iter()
        .map(|(project_id, summary)| BillingEventProjectSummary {
            project_id,
            event_count: summary.event_count,
            request_count: summary.request_count,
            total_units: summary.total_units,
            total_input_tokens: summary.total_input_tokens,
            total_output_tokens: summary.total_output_tokens,
            total_tokens: summary.total_tokens,
            total_image_count: summary.total_image_count,
            total_audio_seconds: summary.total_audio_seconds,
            total_video_seconds: summary.total_video_seconds,
            total_music_seconds: summary.total_music_seconds,
            total_upstream_cost: summary.total_upstream_cost,
            total_customer_charge: summary.total_customer_charge,
        })
        .collect::<Vec<_>>();
    project_summaries.sort_by(|left, right| {
        right
            .total_customer_charge
            .total_cmp(&left.total_customer_charge)
            .then_with(|| right.request_count.cmp(&left.request_count))
            .then_with(|| left.project_id.cmp(&right.project_id))
    });

    let mut group_summaries = groups
        .into_iter()
        .map(|(api_key_group_id, summary)| BillingEventGroupSummary {
            api_key_group_id,
            project_count: summary.project_ids.len() as u64,
            event_count: summary.event_count,
            request_count: summary.request_count,
            total_upstream_cost: summary.total_upstream_cost,
            total_customer_charge: summary.total_customer_charge,
        })
        .collect::<Vec<_>>();
    group_summaries.sort_by(|left, right| {
        right
            .total_customer_charge
            .total_cmp(&left.total_customer_charge)
            .then_with(|| right.request_count.cmp(&left.request_count))
            .then_with(|| left.api_key_group_id.cmp(&right.api_key_group_id))
    });

    let mut capability_summaries = capabilities
        .into_iter()
        .map(|(capability, summary)| BillingEventCapabilitySummary {
            capability,
            event_count: summary.event_count,
            request_count: summary.request_count,
            total_tokens: summary.total_tokens,
            image_count: summary.image_count,
            audio_seconds: summary.audio_seconds,
            video_seconds: summary.video_seconds,
            music_seconds: summary.music_seconds,
            total_upstream_cost: summary.total_upstream_cost,
            total_customer_charge: summary.total_customer_charge,
        })
        .collect::<Vec<_>>();
    capability_summaries.sort_by(|left, right| {
        right
            .request_count
            .cmp(&left.request_count)
            .then_with(|| left.capability.cmp(&right.capability))
    });

    let mut accounting_mode_summaries = accounting_modes
        .into_iter()
        .map(
            |(accounting_mode, summary)| BillingEventAccountingModeSummary {
                accounting_mode,
                event_count: summary.event_count,
                request_count: summary.request_count,
                total_upstream_cost: summary.total_upstream_cost,
                total_customer_charge: summary.total_customer_charge,
            },
        )
        .collect::<Vec<_>>();
    accounting_mode_summaries.sort_by(|left, right| {
        right
            .total_customer_charge
            .total_cmp(&left.total_customer_charge)
            .then_with(|| right.event_count.cmp(&left.event_count))
            .then_with(|| left.accounting_mode.cmp(&right.accounting_mode))
    });

    BillingEventSummary {
        total_events: events.len() as u64,
        project_count: project_summaries.len() as u64,
        group_count: group_summaries.len() as u64,
        capability_count: capability_summaries.len() as u64,
        total_request_count: events.iter().map(|event| event.request_count).sum(),
        total_units: events.iter().map(|event| event.units).sum(),
        total_input_tokens: events.iter().map(|event| event.input_tokens).sum(),
        total_output_tokens: events.iter().map(|event| event.output_tokens).sum(),
        total_tokens: events.iter().map(|event| event.total_tokens).sum(),
        total_image_count: events.iter().map(|event| event.image_count).sum(),
        total_audio_seconds: events.iter().map(|event| event.audio_seconds).sum(),
        total_video_seconds: events.iter().map(|event| event.video_seconds).sum(),
        total_music_seconds: events.iter().map(|event| event.music_seconds).sum(),
        total_upstream_cost: events.iter().map(|event| event.upstream_cost).sum(),
        total_customer_charge: events.iter().map(|event| event.customer_charge).sum(),
        projects: project_summaries,
        groups: group_summaries,
        capabilities: capability_summaries,
        accounting_modes: accounting_mode_summaries,
    }
}

pub async fn summarize_billing_from_store(store: &dyn AdminStore) -> Result<BillingSummary> {
    let entries = list_ledger_entries(store).await?;
    let policies = list_quota_policies(store).await?;
    Ok(summarize_billing_snapshot(&entries, &policies))
}

pub async fn summarize_billing_events_from_store(
    store: &dyn AdminStore,
) -> Result<BillingEventSummary> {
    let events = list_billing_events(store).await?;
    Ok(summarize_billing_events(&events))
}

async fn settle_account_hold<S>(
    store: &S,
    hold_id: u64,
    captured_quantity: f64,
    provider_cost_amount: f64,
    retail_charge_amount: f64,
    request_status: RequestStatus,
    usage_capture_status: UsageCaptureStatus,
    settled_at_ms: u64,
) -> Result<SettledAccountHold>
where
    S: AccountKernelStore + ?Sized,
{
    ensure!(
        captured_quantity >= 0.0,
        "captured_quantity must not be negative"
    );
    ensure!(
        provider_cost_amount >= 0.0,
        "provider_cost_amount must not be negative"
    );
    ensure!(
        retail_charge_amount >= 0.0,
        "retail_charge_amount must not be negative"
    );

    let holds = store.list_account_holds().await?;
    let current_hold = holds
        .into_iter()
        .find(|hold| hold.hold_id == hold_id)
        .ok_or_else(|| anyhow::anyhow!("hold {} does not exist", hold_id))?;

    if let Some(existing_settlement) = store
        .list_request_settlement_records()
        .await?
        .into_iter()
        .find(|settlement| settlement.request_id == current_hold.request_id)
    {
        let request_meter_fact = store
            .list_request_meter_facts()
            .await?
            .into_iter()
            .find(|record| record.request_id == current_hold.request_id)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "request meter fact {} missing for hold {}",
                    current_hold.request_id,
                    current_hold.hold_id
                )
            })?;
        return Ok(SettledAccountHold {
            hold: current_hold,
            settlement: existing_settlement,
            request_meter_fact,
        });
    }

    ensure!(
        captured_quantity <= current_hold.estimated_quantity + f64::EPSILON,
        "captured_quantity exceeds held quantity for hold {}",
        current_hold.hold_id
    );

    let account = store
        .find_account_record(current_hold.account_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("account {} does not exist", current_hold.account_id))?;
    let request_meter_fact = store
        .list_request_meter_facts()
        .await?
        .into_iter()
        .find(|record| record.request_id == current_hold.request_id)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "request meter fact {} does not exist",
                current_hold.request_id
            )
        })?;
    let hold_allocations = store
        .list_account_hold_allocations()
        .await?
        .into_iter()
        .filter(|allocation| allocation.hold_id == current_hold.hold_id)
        .collect::<Vec<_>>();
    ensure!(
        !hold_allocations.is_empty(),
        "hold {} has no allocation records",
        current_hold.hold_id
    );

    let lots_by_id = store
        .list_account_benefit_lots()
        .await?
        .into_iter()
        .filter(|lot| lot.account_id == current_hold.account_id)
        .map(|lot| (lot.lot_id, lot))
        .collect::<BTreeMap<_, _>>();

    let mut remaining_capture = captured_quantity;
    let mut updated_allocations = Vec::new();
    let mut updated_lots = Vec::new();
    let mut capture_allocations = Vec::new();
    let mut release_allocations = Vec::new();

    for (index, allocation) in hold_allocations.iter().enumerate() {
        let capture_amount = allocation.allocated_quantity.min(remaining_capture);
        remaining_capture = (remaining_capture - capture_amount).max(0.0);
        let release_amount = (allocation.allocated_quantity - capture_amount).max(0.0);

        updated_allocations.push(
            allocation
                .clone()
                .with_captured_quantity(capture_amount)
                .with_released_quantity(release_amount)
                .with_updated_at_ms(settled_at_ms),
        );

        let mut lot = lots_by_id
            .get(&allocation.lot_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("benefit lot {} does not exist", allocation.lot_id))?;
        ensure!(
            lot.held_quantity + f64::EPSILON >= allocation.allocated_quantity,
            "benefit lot {} does not have enough held balance for release",
            lot.lot_id
        );
        lot.held_quantity = (lot.held_quantity - allocation.allocated_quantity).max(0.0);
        lot.remaining_quantity = (lot.remaining_quantity - capture_amount).max(0.0);
        if lot.remaining_quantity <= f64::EPSILON {
            lot.remaining_quantity = 0.0;
            lot.status = AccountBenefitLotStatus::Exhausted;
        }
        lot.updated_at_ms = settled_at_ms;
        updated_lots.push(lot);

        if capture_amount > f64::EPSILON {
            capture_allocations.push(
                AccountLedgerAllocationRecord::new(
                    derived_allocation_id(
                        settlement_capture_ledger_entry_id(current_hold.request_id)?,
                        index,
                    )?,
                    account.tenant_id,
                    account.organization_id,
                    settlement_capture_ledger_entry_id(current_hold.request_id)?,
                    allocation.lot_id,
                )
                .with_quantity_delta(-capture_amount)
                .with_created_at_ms(settled_at_ms),
            );
        }
        if release_amount > f64::EPSILON {
            release_allocations.push(
                AccountLedgerAllocationRecord::new(
                    derived_allocation_id(
                        settlement_release_ledger_entry_id(current_hold.request_id)?,
                        index,
                    )?,
                    account.tenant_id,
                    account.organization_id,
                    settlement_release_ledger_entry_id(current_hold.request_id)?,
                    allocation.lot_id,
                )
                .with_quantity_delta(release_amount)
                .with_created_at_ms(settled_at_ms),
            );
        }
    }

    ensure!(
        remaining_capture <= f64::EPSILON,
        "captured_quantity exceeds allocated quantities for hold {}",
        current_hold.hold_id
    );

    let released_quantity = (current_hold.estimated_quantity - captured_quantity).max(0.0);
    let hold_status = hold_status_for_settlement(captured_quantity, released_quantity);
    let settlement_status = settlement_status_for_settlement(captured_quantity, released_quantity);

    let updated_hold = current_hold
        .clone()
        .with_status(hold_status)
        .with_captured_quantity(captured_quantity)
        .with_released_quantity(released_quantity)
        .with_updated_at_ms(settled_at_ms);
    let updated_request_meter_fact = request_meter_fact
        .with_request_status(request_status)
        .with_usage_capture_status(usage_capture_status)
        .with_actual_credit_charge(Some(captured_quantity))
        .with_actual_provider_cost(Some(provider_cost_amount))
        .with_finished_at_ms(Some(settled_at_ms))
        .with_updated_at_ms(settled_at_ms);
    let settlement = RequestSettlementRecord::new(
        request_settlement_id(current_hold.request_id),
        account.tenant_id,
        account.organization_id,
        current_hold.request_id,
        account.account_id,
        account.user_id,
    )
    .with_hold_id(Some(current_hold.hold_id))
    .with_status(settlement_status)
    .with_estimated_credit_hold(current_hold.estimated_quantity)
    .with_released_credit_amount(released_quantity)
    .with_captured_credit_amount(captured_quantity)
    .with_provider_cost_amount(provider_cost_amount)
    .with_retail_charge_amount(retail_charge_amount)
    .with_settled_at_ms(settled_at_ms)
    .with_created_at_ms(settled_at_ms)
    .with_updated_at_ms(settled_at_ms);

    let mut ledger_entries = Vec::new();
    let mut ledger_allocations = Vec::new();
    if captured_quantity > f64::EPSILON {
        ledger_entries.push(
            AccountLedgerEntryRecord::new(
                settlement_capture_ledger_entry_id(current_hold.request_id)?,
                account.tenant_id,
                account.organization_id,
                account.account_id,
                account.user_id,
                AccountLedgerEntryType::SettlementCapture,
            )
            .with_request_id(Some(current_hold.request_id))
            .with_hold_id(Some(current_hold.hold_id))
            .with_quantity(captured_quantity)
            .with_amount(captured_quantity)
            .with_created_at_ms(settled_at_ms),
        );
        ledger_allocations.extend(capture_allocations);
    }
    if released_quantity > f64::EPSILON {
        ledger_entries.push(
            AccountLedgerEntryRecord::new(
                settlement_release_ledger_entry_id(current_hold.request_id)?,
                account.tenant_id,
                account.organization_id,
                account.account_id,
                account.user_id,
                AccountLedgerEntryType::HoldRelease,
            )
            .with_request_id(Some(current_hold.request_id))
            .with_hold_id(Some(current_hold.hold_id))
            .with_quantity(released_quantity)
            .with_amount(released_quantity)
            .with_created_at_ms(settled_at_ms),
        );
        ledger_allocations.extend(release_allocations);
    }

    let batch = AccountKernelCommandBatch {
        benefit_lot_records: updated_lots,
        hold_records: vec![updated_hold.clone()],
        hold_allocation_records: updated_allocations,
        ledger_entry_records: ledger_entries,
        ledger_allocation_records: ledger_allocations,
        request_meter_fact_records: vec![updated_request_meter_fact.clone()],
        request_settlement_records: vec![settlement.clone()],
        ..AccountKernelCommandBatch::default()
    };
    store.commit_account_kernel_batch(&batch).await?;

    Ok(SettledAccountHold {
        hold: updated_hold,
        settlement,
        request_meter_fact: updated_request_meter_fact,
    })
}

fn prepare_request_meter_fact_for_hold(
    account: &AccountRecord,
    request_meter_fact: RequestMeterFactRecord,
    estimated_credit_hold: f64,
    now_ms: u64,
) -> Result<RequestMeterFactRecord> {
    ensure!(
        request_meter_fact.account_id == account.account_id,
        "request meter fact account_id must match hold account"
    );
    ensure!(
        request_meter_fact.tenant_id == account.tenant_id,
        "request meter fact tenant_id must match hold account"
    );
    ensure!(
        request_meter_fact.organization_id == account.organization_id,
        "request meter fact organization_id must match hold account"
    );
    ensure!(
        request_meter_fact.user_id == account.user_id,
        "request meter fact user_id must match hold account"
    );

    let started_at_ms = if request_meter_fact.started_at_ms == 0 {
        now_ms
    } else {
        request_meter_fact.started_at_ms
    };
    let created_at_ms = if request_meter_fact.created_at_ms == 0 {
        now_ms
    } else {
        request_meter_fact.created_at_ms
    };

    Ok(request_meter_fact
        .with_request_status(RequestStatus::Pending)
        .with_usage_capture_status(UsageCaptureStatus::Pending)
        .with_estimated_credit_hold(estimated_credit_hold)
        .with_actual_credit_charge(None)
        .with_actual_provider_cost(None)
        .with_started_at_ms(started_at_ms)
        .with_created_at_ms(created_at_ms)
        .with_updated_at_ms(now_ms))
}

fn hold_status_for_settlement(captured_quantity: f64, released_quantity: f64) -> AccountHoldStatus {
    if captured_quantity <= f64::EPSILON {
        AccountHoldStatus::Released
    } else if released_quantity <= f64::EPSILON {
        AccountHoldStatus::Captured
    } else {
        AccountHoldStatus::PartiallyReleased
    }
}

fn settlement_status_for_settlement(
    captured_quantity: f64,
    released_quantity: f64,
) -> RequestSettlementStatus {
    if captured_quantity <= f64::EPSILON {
        RequestSettlementStatus::Released
    } else if released_quantity <= f64::EPSILON {
        RequestSettlementStatus::Captured
    } else {
        RequestSettlementStatus::PartiallyReleased
    }
}

fn request_settlement_id(request_id: u64) -> u64 {
    request_id
}

fn hold_create_ledger_entry_id(request_id: u64) -> Result<u64> {
    derived_child_id(request_id, 1)
}

fn settlement_capture_ledger_entry_id(request_id: u64) -> Result<u64> {
    derived_child_id(request_id, 2)
}

fn settlement_release_ledger_entry_id(request_id: u64) -> Result<u64> {
    derived_child_id(request_id, 3)
}

fn derived_child_id(parent_id: u64, suffix: u64) -> Result<u64> {
    parent_id
        .checked_mul(10)
        .and_then(|value| value.checked_add(suffix))
        .ok_or_else(|| anyhow::anyhow!("derived identifier overflow for {}", parent_id))
}

fn derived_allocation_id(parent_id: u64, index: usize) -> Result<u64> {
    let offset = u64::try_from(index)
        .ok()
        .and_then(|value| value.checked_add(1))
        .ok_or_else(|| anyhow::anyhow!("allocation index overflow for {}", parent_id))?;
    parent_id
        .checked_mul(100)
        .and_then(|value| value.checked_add(offset))
        .ok_or_else(|| anyhow::anyhow!("derived allocation identifier overflow for {}", parent_id))
}

fn eligible_lots_for_hold(
    lots: &[AccountBenefitLotRecord],
    now_ms: u64,
) -> Vec<&AccountBenefitLotRecord> {
    let mut eligible = lots
        .iter()
        .filter(|lot| {
            lot.status == AccountBenefitLotStatus::Active
                && lot
                    .expires_at_ms
                    .map(|expires_at_ms| expires_at_ms > now_ms)
                    .unwrap_or(true)
                && free_quantity(lot) > 0.0
        })
        .collect::<Vec<_>>();
    eligible.sort_by(|left, right| {
        left.expires_at_ms
            .unwrap_or(u64::MAX)
            .cmp(&right.expires_at_ms.unwrap_or(u64::MAX))
            .then_with(|| right.scope_json.is_some().cmp(&left.scope_json.is_some()))
            .then_with(|| {
                benefit_cash_rank(left.benefit_type).cmp(&benefit_cash_rank(right.benefit_type))
            })
            .then_with(|| {
                left.acquired_unit_cost
                    .unwrap_or(f64::INFINITY)
                    .total_cmp(&right.acquired_unit_cost.unwrap_or(f64::INFINITY))
            })
            .then_with(|| left.lot_id.cmp(&right.lot_id))
    });
    eligible
}

fn free_quantity(lot: &AccountBenefitLotRecord) -> f64 {
    (lot.remaining_quantity - lot.held_quantity).max(0.0)
}

fn benefit_cash_rank(benefit_type: AccountBenefitType) -> u8 {
    match benefit_type {
        AccountBenefitType::CashCredit => 1,
        _ => 0,
    }
}
