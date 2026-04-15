use std::fs;

fn postgres_storage_sources() -> (String, String) {
    let account_kernel_store = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/account_kernel_store.rs"
    ))
    .expect("postgres account kernel store source");
    let account_support = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/account_support.rs"
    ))
    .expect("postgres account support source");
    (account_kernel_store, account_support)
}

#[test]
fn postgres_store_implements_account_kernel_pricing_surface() {
    let (account_kernel_store, account_support) = postgres_storage_sources();

    assert!(
        account_kernel_store.contains("async fn insert_pricing_plan_record("),
        "expected postgres store pricing plan insert implementation",
    );
    assert!(
        account_kernel_store.contains(
            "async fn list_pricing_plan_records(&self) -> Result<Vec<PricingPlanRecord>>"
        ),
        "expected postgres store pricing plan list implementation",
    );
    assert!(
        account_kernel_store.contains("async fn insert_pricing_rate_record("),
        "expected postgres store pricing rate insert implementation",
    );
    assert!(
        account_kernel_store.contains(
            "async fn list_pricing_rate_records(&self) -> Result<Vec<PricingRateRecord>>"
        ),
        "expected postgres store pricing rate list implementation",
    );
    assert!(
        account_support
            .contains("fn decode_pricing_plan_row(row: PgRow) -> Result<PricingPlanRecord>"),
        "expected postgres pricing plan row decoder",
    );
    assert!(
        account_support
            .contains("fn decode_pricing_rate_row(row: PgRow) -> Result<PricingRateRecord>"),
        "expected postgres pricing rate row decoder",
    );
    assert!(
        account_kernel_store.contains("ownership_scope"),
        "expected postgres pricing plan persistence to include ownership_scope",
    );
    assert!(
        account_support.contains("parse_pricing_plan_ownership_scope"),
        "expected postgres pricing plan decoder to parse ownership_scope",
    );
}

#[test]
fn postgres_store_implements_commercial_account_store_surface() {
    let (account_kernel_store, _) = postgres_storage_sources();

    for snippet in [
        "async fn insert_account_record(&self, record: &AccountRecord) -> Result<AccountRecord>",
        "async fn list_account_records(&self) -> Result<Vec<AccountRecord>>",
        "async fn find_account_record(&self, account_id: u64) -> Result<Option<AccountRecord>>",
        "async fn find_account_record_by_owner(",
        "async fn insert_account_benefit_lot(",
        "async fn list_account_benefit_lots(&self) -> Result<Vec<AccountBenefitLotRecord>>",
        "async fn list_account_benefit_lots_for_account(",
        "async fn insert_account_hold(&self, record: &AccountHoldRecord) -> Result<AccountHoldRecord>",
        "async fn list_account_holds(&self) -> Result<Vec<AccountHoldRecord>>",
        "async fn insert_request_settlement_record(",
        "async fn list_request_settlement_records(&self) -> Result<Vec<RequestSettlementRecord>>",
    ] {
        assert!(
            account_kernel_store.contains(snippet),
            "expected postgres commercial account surface to contain {snippet}",
        );
    }
}

#[test]
fn postgres_store_implements_remaining_account_kernel_persistence_surface() {
    let (account_kernel_store, account_support) = postgres_storage_sources();

    for snippet in [
        "async fn insert_account_hold_allocation(",
        "async fn list_account_hold_allocations(&self) -> Result<Vec<AccountHoldAllocationRecord>>",
        "async fn insert_account_ledger_entry_record(",
        "async fn list_account_ledger_entry_records(&self) -> Result<Vec<AccountLedgerEntryRecord>>",
        "async fn insert_account_ledger_allocation(",
        "async fn list_account_ledger_allocations(&self) -> Result<Vec<AccountLedgerAllocationRecord>>",
        "async fn insert_request_meter_fact(",
        "async fn list_request_meter_facts(&self) -> Result<Vec<RequestMeterFactRecord>>",
        "async fn insert_request_meter_metric(",
        "async fn list_request_meter_metrics(&self) -> Result<Vec<RequestMeterMetricRecord>>",
    ] {
        assert!(
            account_kernel_store.contains(snippet),
            "expected postgres remaining account-kernel surface to contain {snippet}",
        );
    }

    for snippet in [
        "fn decode_account_ledger_entry_row(",
        "fn decode_account_ledger_allocation_row(",
        "fn decode_request_meter_fact_row(",
        "fn decode_request_meter_metric_row(",
        "fn account_ledger_entry_type_as_str(",
        "fn parse_account_ledger_entry_type(",
        "fn request_status_as_str(",
        "fn parse_request_status(",
        "fn usage_capture_status_as_str(",
        "fn parse_usage_capture_status(",
    ] {
        assert!(
            account_support.contains(snippet),
            "expected postgres remaining account-kernel surface to contain {snippet}",
        );
    }
}
