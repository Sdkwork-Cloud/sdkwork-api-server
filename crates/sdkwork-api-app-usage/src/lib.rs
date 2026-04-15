use anyhow::Result;
use sdkwork_api_domain_usage::{
    UsageModelSummary, UsageProjectSummary, UsageProviderSummary, UsageRecord, UsageSummary,
};
use sdkwork_api_storage_core::AdminStore;
use std::collections::{BTreeMap, BTreeSet};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn service_name() -> &'static str {
    "usage-service"
}

pub fn record_usage(
    project_id: &str,
    model: &str,
    provider: &str,
    units: u64,
    amount: f64,
) -> Result<UsageRecord> {
    record_usage_with_tokens(project_id, model, provider, units, amount, 0, 0, 0)
}

#[allow(clippy::too_many_arguments)]
pub fn record_usage_with_tokens(
    project_id: &str,
    model: &str,
    provider: &str,
    units: u64,
    amount: f64,
    input_tokens: u64,
    output_tokens: u64,
    total_tokens: u64,
) -> Result<UsageRecord> {
    Ok(UsageRecord::new(project_id, model, provider)
        .with_metering(units, amount, current_time_ms()?)
        .with_token_usage(input_tokens, output_tokens, total_tokens))
}

#[allow(clippy::too_many_arguments)]
pub fn record_usage_with_tokens_and_facts(
    project_id: &str,
    model: &str,
    provider: &str,
    units: u64,
    amount: f64,
    input_tokens: u64,
    output_tokens: u64,
    total_tokens: u64,
    api_key_hash: Option<&str>,
    channel_id: Option<&str>,
    latency_ms: Option<u64>,
    reference_amount: Option<f64>,
) -> Result<UsageRecord> {
    Ok(record_usage_with_tokens(
        project_id,
        model,
        provider,
        units,
        amount,
        input_tokens,
        output_tokens,
        total_tokens,
    )?
    .with_request_facts(api_key_hash, channel_id, latency_ms, reference_amount))
}

pub async fn persist_usage_record(
    store: &dyn AdminStore,
    project_id: &str,
    model: &str,
    provider: &str,
    units: u64,
    amount: f64,
) -> Result<UsageRecord> {
    let usage = record_usage(project_id, model, provider, units, amount)?;
    store.insert_usage_record(&usage).await
}

#[allow(clippy::too_many_arguments)]
pub async fn persist_usage_record_with_tokens(
    store: &dyn AdminStore,
    project_id: &str,
    model: &str,
    provider: &str,
    units: u64,
    amount: f64,
    input_tokens: u64,
    output_tokens: u64,
    total_tokens: u64,
) -> Result<UsageRecord> {
    let usage = record_usage_with_tokens(
        project_id,
        model,
        provider,
        units,
        amount,
        input_tokens,
        output_tokens,
        total_tokens,
    )?;
    store.insert_usage_record(&usage).await
}

#[allow(clippy::too_many_arguments)]
pub async fn persist_usage_record_with_tokens_and_facts(
    store: &dyn AdminStore,
    project_id: &str,
    model: &str,
    provider: &str,
    units: u64,
    amount: f64,
    input_tokens: u64,
    output_tokens: u64,
    total_tokens: u64,
    api_key_hash: Option<&str>,
    channel_id: Option<&str>,
    latency_ms: Option<u64>,
    reference_amount: Option<f64>,
) -> Result<UsageRecord> {
    let usage = record_usage_with_tokens_and_facts(
        project_id,
        model,
        provider,
        units,
        amount,
        input_tokens,
        output_tokens,
        total_tokens,
        api_key_hash,
        channel_id,
        latency_ms,
        reference_amount,
    )?;
    store.insert_usage_record(&usage).await
}

pub async fn list_usage_records(store: &dyn AdminStore) -> Result<Vec<UsageRecord>> {
    store.list_usage_records().await
}

pub fn summarize_usage_records(records: &[UsageRecord]) -> UsageSummary {
    if records.is_empty() {
        return UsageSummary::empty();
    }

    let mut projects = BTreeMap::<String, u64>::new();
    let mut providers = BTreeMap::<String, (u64, BTreeSet<String>)>::new();
    let mut models = BTreeMap::<String, (u64, BTreeSet<String>)>::new();
    let mut distinct_projects = BTreeSet::new();
    let mut distinct_models = BTreeSet::new();
    let mut distinct_providers = BTreeSet::new();

    for record in records {
        *projects.entry(record.project_id.clone()).or_default() += 1;

        let provider = providers.entry(record.provider.clone()).or_default();
        provider.0 += 1;
        provider.1.insert(record.project_id.clone());

        let model = models.entry(record.model.clone()).or_default();
        model.0 += 1;
        model.1.insert(record.provider.clone());

        distinct_projects.insert(record.project_id.clone());
        distinct_models.insert(record.model.clone());
        distinct_providers.insert(record.provider.clone());
    }

    let mut project_summaries = projects
        .into_iter()
        .map(|(project_id, request_count)| UsageProjectSummary::new(project_id, request_count))
        .collect::<Vec<_>>();
    project_summaries.sort_by(|left, right| {
        right
            .request_count
            .cmp(&left.request_count)
            .then_with(|| left.project_id.cmp(&right.project_id))
    });

    let mut provider_summaries = providers
        .into_iter()
        .map(|(provider, (request_count, project_ids))| {
            UsageProviderSummary::new(provider, request_count, project_ids.len() as u64)
        })
        .collect::<Vec<_>>();
    provider_summaries.sort_by(|left, right| {
        right
            .request_count
            .cmp(&left.request_count)
            .then_with(|| left.provider.cmp(&right.provider))
    });

    let mut model_summaries = models
        .into_iter()
        .map(|(model, (request_count, providers))| {
            UsageModelSummary::new(model, request_count, providers.len() as u64)
        })
        .collect::<Vec<_>>();
    model_summaries.sort_by(|left, right| {
        right
            .request_count
            .cmp(&left.request_count)
            .then_with(|| left.model.cmp(&right.model))
    });

    UsageSummary {
        total_requests: records.len() as u64,
        project_count: distinct_projects.len() as u64,
        model_count: distinct_models.len() as u64,
        provider_count: distinct_providers.len() as u64,
        projects: project_summaries,
        providers: provider_summaries,
        models: model_summaries,
    }
}

pub async fn summarize_usage_records_from_store(store: &dyn AdminStore) -> Result<UsageSummary> {
    let records = list_usage_records(store).await?;
    Ok(summarize_usage_records(&records))
}

fn current_time_ms() -> Result<u64> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64)
}
