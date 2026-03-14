use anyhow::Result;
use sdkwork_api_domain_usage::{
    UsageModelSummary, UsageProjectSummary, UsageProviderSummary, UsageRecord, UsageSummary,
};
use sdkwork_api_storage_core::AdminStore;
use std::collections::{BTreeMap, BTreeSet};

pub fn service_name() -> &'static str {
    "usage-service"
}

pub fn record_usage(project_id: &str, model: &str, provider: &str) -> Result<UsageRecord> {
    Ok(UsageRecord::new(project_id, model, provider))
}

pub async fn persist_usage_record(
    store: &dyn AdminStore,
    project_id: &str,
    model: &str,
    provider: &str,
) -> Result<UsageRecord> {
    let usage = record_usage(project_id, model, provider)?;
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
