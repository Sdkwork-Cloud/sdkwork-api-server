use anyhow::Result;
use sdkwork_api_domain_billing::LedgerEntry;

pub fn service_name() -> &'static str {
    "billing-service"
}

pub fn check_quota(_project_id: &str, _units: u64) -> Result<bool> {
    Ok(true)
}

pub fn book_usage_cost(project_id: &str, units: u64, amount: f64) -> Result<LedgerEntry> {
    Ok(LedgerEntry::new(project_id, units, amount))
}
