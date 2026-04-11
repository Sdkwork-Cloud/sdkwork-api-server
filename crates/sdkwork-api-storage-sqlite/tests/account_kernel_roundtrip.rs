use sdkwork_api_domain_billing::{
    AccountBenefitLotRecord, AccountBenefitSourceType, AccountBenefitType,
    AccountHoldAllocationRecord, AccountHoldRecord, AccountLedgerAllocationRecord,
    AccountLedgerEntryRecord, AccountLedgerEntryType, AccountRecord, AccountType,
    PricingPlanOwnershipScope, PricingPlanRecord, PricingRateRecord, RequestSettlementRecord,
};
use sdkwork_api_domain_usage::{RequestMeterFactRecord, RequestMeterMetricRecord};
use sdkwork_api_storage_core::AccountKernelStore;
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[tokio::test]
async fn sqlite_store_round_trips_canonical_account_kernel_records() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let account = AccountRecord::new(7001, 1001, 2002, 9001, AccountType::Primary)
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_created_at_ms(10)
        .with_updated_at_ms(20);
    let lot =
        AccountBenefitLotRecord::new(8001, 1001, 2002, 7001, 9001, AccountBenefitType::CashCredit)
            .with_source_type(AccountBenefitSourceType::Recharge)
            .with_original_quantity(1200.0)
            .with_remaining_quantity(1200.0)
            .with_issued_at_ms(30)
            .with_created_at_ms(30)
            .with_updated_at_ms(30);
    let hold = AccountHoldRecord::new(8101, 1001, 2002, 7001, 9001, 6001)
        .with_estimated_quantity(42.5)
        .with_expires_at_ms(40)
        .with_created_at_ms(35)
        .with_updated_at_ms(35);
    let ledger_entry = AccountLedgerEntryRecord::new(
        8201,
        1001,
        2002,
        7001,
        9001,
        AccountLedgerEntryType::HoldCreate,
    )
    .with_request_id(Some(6001))
    .with_hold_id(Some(8101))
    .with_benefit_type(Some("cash_credit".to_owned()))
    .with_quantity(42.5)
    .with_amount(42.5)
    .with_created_at_ms(36);
    let hold_allocation = AccountHoldAllocationRecord::new(8401, 1001, 2002, 8101, 8001)
        .with_allocated_quantity(42.5)
        .with_captured_quantity(40.0)
        .with_released_quantity(2.5)
        .with_created_at_ms(36)
        .with_updated_at_ms(41);
    let ledger_allocation = AccountLedgerAllocationRecord::new(8501, 1001, 2002, 8201, 8001)
        .with_quantity_delta(-40.0)
        .with_created_at_ms(41);
    let fact = RequestMeterFactRecord::new(
        6001,
        1001,
        2002,
        9001,
        7001,
        "api_key",
        "responses",
        "openai",
        "gpt-4.1",
        "provider-openai-official",
    )
    .with_api_key_id(Some(778899))
    .with_api_key_hash(Some("key_hash_live".to_owned()))
    .with_protocol_family("openai")
    .with_estimated_credit_hold(24.0)
    .with_created_at_ms(35)
    .with_updated_at_ms(36);
    let metric = RequestMeterMetricRecord::new(7001001, 1001, 2002, 6001, "token.input", 128.0)
        .with_provider_field(Some("prompt_tokens".to_owned()))
        .with_captured_at_ms(37);
    let plan = PricingPlanRecord::new(9101, 1001, 2002, "default-retail", 3)
        .with_display_name("Default Retail v3")
        .with_status("active")
        .with_ownership_scope(PricingPlanOwnershipScope::PlatformShared)
        .with_effective_from_ms(38)
        .with_effective_to_ms(Some(138))
        .with_created_at_ms(38)
        .with_updated_at_ms(38);
    let rate = PricingRateRecord::new(9201, 1001, 2002, 9101, "token.input")
        .with_model_code(Some("gpt-4.1".to_owned()))
        .with_provider_code(Some("provider-openai-official".to_owned()))
        .with_capability_code(Some("responses".to_owned()))
        .with_charge_unit("input_token")
        .with_pricing_method("per_unit")
        .with_unit_price(0.0025)
        .with_display_price_unit("USD / 1M input tokens")
        .with_minimum_billable_quantity(0.0)
        .with_minimum_charge(0.0)
        .with_rounding_increment(1.0)
        .with_rounding_mode("ceil")
        .with_included_quantity(0.0)
        .with_priority(100)
        .with_notes(Some("Retail input pricing".to_owned()))
        .with_status("active")
        .with_created_at_ms(39)
        .with_updated_at_ms(40);
    let settlement = RequestSettlementRecord::new(8301, 1001, 2002, 6001, 7001, 9001)
        .with_hold_id(Some(8101))
        .with_estimated_credit_hold(42.5)
        .with_captured_credit_amount(40.0)
        .with_provider_cost_amount(18.0)
        .with_retail_charge_amount(40.0)
        .with_settled_at_ms(41)
        .with_created_at_ms(41)
        .with_updated_at_ms(41);

    store.insert_account_record(&account).await.unwrap();
    store.insert_account_benefit_lot(&lot).await.unwrap();
    store.insert_account_hold(&hold).await.unwrap();
    store
        .insert_account_ledger_entry_record(&ledger_entry)
        .await
        .unwrap();
    store
        .insert_account_hold_allocation(&hold_allocation)
        .await
        .unwrap();
    store
        .insert_account_ledger_allocation(&ledger_allocation)
        .await
        .unwrap();
    store.insert_request_meter_fact(&fact).await.unwrap();
    store.insert_request_meter_metric(&metric).await.unwrap();
    store.insert_pricing_plan_record(&plan).await.unwrap();
    store.insert_pricing_rate_record(&rate).await.unwrap();
    store
        .insert_request_settlement_record(&settlement)
        .await
        .unwrap();

    assert_eq!(
        store.find_account_record(7001).await.unwrap(),
        Some(account)
    );
    assert_eq!(store.list_account_records().await.unwrap().len(), 1);
    assert_eq!(store.list_account_benefit_lots().await.unwrap().len(), 1);
    assert_eq!(store.list_account_holds().await.unwrap().len(), 1);
    assert_eq!(
        store.list_account_hold_allocations().await.unwrap().len(),
        1
    );
    assert_eq!(
        store
            .list_account_ledger_entry_records()
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        store.list_account_ledger_allocations().await.unwrap().len(),
        1
    );
    assert_eq!(store.list_request_meter_facts().await.unwrap().len(), 1);
    assert_eq!(store.list_request_meter_metrics().await.unwrap().len(), 1);
    assert_eq!(store.list_pricing_plan_records().await.unwrap(), vec![plan]);
    assert_eq!(store.list_pricing_rate_records().await.unwrap(), vec![rate]);
    assert_eq!(
        store.list_request_settlement_records().await.unwrap().len(),
        1
    );
}

#[tokio::test]
async fn sqlite_store_lists_account_benefit_lots_with_account_cursor_pagination() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let account = AccountRecord::new(7001, 1001, 2002, 9001, AccountType::Primary)
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_created_at_ms(10)
        .with_updated_at_ms(20);
    let other_account = AccountRecord::new(7002, 1001, 2002, 9002, AccountType::Primary)
        .with_currency_code("USD")
        .with_credit_unit_code("credit")
        .with_created_at_ms(10)
        .with_updated_at_ms(20);
    store.insert_account_record(&account).await.unwrap();
    store.insert_account_record(&other_account).await.unwrap();

    for lot_id in [8001_u64, 8002, 8003] {
        store
            .insert_account_benefit_lot(
                &AccountBenefitLotRecord::new(
                    lot_id,
                    account.tenant_id,
                    account.organization_id,
                    account.account_id,
                    account.user_id,
                    AccountBenefitType::CashCredit,
                )
                .with_source_type(AccountBenefitSourceType::Recharge)
                .with_original_quantity(100.0)
                .with_remaining_quantity(100.0)
                .with_issued_at_ms(lot_id)
                .with_created_at_ms(lot_id)
                .with_updated_at_ms(lot_id),
            )
            .await
            .unwrap();
    }
    store
        .insert_account_benefit_lot(
            &AccountBenefitLotRecord::new(
                8100,
                other_account.tenant_id,
                other_account.organization_id,
                other_account.account_id,
                other_account.user_id,
                AccountBenefitType::CashCredit,
            )
            .with_source_type(AccountBenefitSourceType::Recharge)
            .with_original_quantity(50.0)
            .with_remaining_quantity(50.0)
            .with_issued_at_ms(8100)
            .with_created_at_ms(8100)
            .with_updated_at_ms(8100),
        )
        .await
        .unwrap();

    let first_page = store
        .list_account_benefit_lots_for_account(account.account_id, None, 2)
        .await
        .unwrap();
    assert_eq!(
        first_page.iter().map(|lot| lot.lot_id).collect::<Vec<_>>(),
        vec![8001, 8002]
    );

    let second_page = store
        .list_account_benefit_lots_for_account(account.account_id, Some(8002), 2)
        .await
        .unwrap();
    assert_eq!(
        second_page.iter().map(|lot| lot.lot_id).collect::<Vec<_>>(),
        vec![8003]
    );
}
