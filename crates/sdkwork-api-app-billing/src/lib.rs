use anyhow::Result;
use sdkwork_api_domain_billing::LedgerEntry;
use sdkwork_api_storage_core::AdminStore;

pub fn service_name() -> &'static str {
    "billing-service"
}

pub fn check_quota(_project_id: &str, _units: u64) -> Result<bool> {
    Ok(true)
}

pub fn book_usage_cost(project_id: &str, units: u64, amount: f64) -> Result<LedgerEntry> {
    Ok(LedgerEntry::new(project_id, units, amount))
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

pub async fn list_ledger_entries(store: &dyn AdminStore) -> Result<Vec<LedgerEntry>> {
    store.list_ledger_entries().await
}
