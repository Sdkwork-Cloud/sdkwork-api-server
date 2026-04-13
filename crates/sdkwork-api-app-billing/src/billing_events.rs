use super::*;

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
