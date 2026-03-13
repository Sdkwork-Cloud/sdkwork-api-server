use anyhow::Result;
use sdkwork_api_domain_usage::UsageRecord;
use sdkwork_api_storage_core::AdminStore;

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
