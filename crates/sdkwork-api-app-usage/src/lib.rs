use anyhow::Result;
use sdkwork_api_domain_usage::UsageRecord;

pub fn service_name() -> &'static str {
    "usage-service"
}

pub fn record_usage(project_id: &str, model: &str, provider: &str) -> Result<UsageRecord> {
    Ok(UsageRecord::new(project_id, model, provider))
}
