use sdkwork_api_app_billing::{
    capture_account_hold, create_account_hold, release_account_hold, summarize_account_balance,
    CaptureAccountHoldInput, CreateAccountHoldInput, ReleaseAccountHoldInput,
};
use sdkwork_api_domain_billing::{
    AccountBenefitLotRecord, AccountBenefitSourceType, AccountBenefitType, AccountHoldStatus,
    AccountLedgerEntryType, AccountRecord, AccountType, RequestSettlementStatus,
};
use sdkwork_api_domain_usage::{RequestMeterFactRecord, RequestStatus, UsageCaptureStatus};
use sdkwork_api_storage_core::AccountKernelStore;
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[tokio::test]
async fn creates_hold_and_reserves_lot_balance_through_command_batch() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    seed_account_with_lots(
        &store,
        9001,
        &[(9101, 50.0, AccountBenefitType::PromoCredit), (9102, 70.0, AccountBenefitType::CashCredit)],
    )
    .await;

    let created = create_account_hold(
        &store,
        CreateAccountHoldInput {
            hold_id: 9201,
            account_id: 9001,
            request_id: 9301,
            estimated_quantity: 80.0,
            expires_at_ms: 500,
            now_ms: 100,
            request_meter_fact: request_meter_fact(9301, 9001, 100),
        },
    )
    .await
    .unwrap();

    assert_eq!(created.hold.status, AccountHoldStatus::Held);
    assert_eq!(created.hold.estimated_quantity, 80.0);
    assert_eq!(created.hold_allocations.len(), 2);

    let snapshot = summarize_account_balance(&store, 9001, 100).await.unwrap();
    assert_eq!(snapshot.available_balance, 40.0);
    assert_eq!(snapshot.held_balance, 80.0);
    assert_eq!(snapshot.consumed_balance, 0.0);

    let lots = store.list_account_benefit_lots().await.unwrap();
    let promo = lots.iter().find(|lot| lot.lot_id == 9101).unwrap();
    let cash = lots.iter().find(|lot| lot.lot_id == 9102).unwrap();
    assert_eq!(promo.held_quantity, 50.0);
    assert_eq!(cash.held_quantity, 30.0);
    assert_eq!(promo.remaining_quantity, 50.0);
    assert_eq!(cash.remaining_quantity, 70.0);

    let request_facts = store.list_request_meter_facts().await.unwrap();
    assert_eq!(request_facts.len(), 1);
    assert_eq!(request_facts[0].request_status, RequestStatus::Pending);
    assert_eq!(request_facts[0].usage_capture_status, UsageCaptureStatus::Pending);
    assert_eq!(request_facts[0].estimated_credit_hold, 80.0);

    let ledger_entries = store.list_account_ledger_entry_records().await.unwrap();
    assert_eq!(ledger_entries.len(), 1);
    assert_eq!(ledger_entries[0].entry_type, AccountLedgerEntryType::HoldCreate);
    assert_eq!(ledger_entries[0].quantity, 80.0);
}

#[tokio::test]
async fn captures_hold_once_and_reuses_existing_settlement_for_duplicate_requests() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    seed_account_with_lots(
        &store,
        9002,
        &[(9111, 50.0, AccountBenefitType::PromoCredit), (9112, 50.0, AccountBenefitType::CashCredit)],
    )
    .await;

    create_account_hold(
        &store,
        CreateAccountHoldInput {
            hold_id: 9202,
            account_id: 9002,
            request_id: 9302,
            estimated_quantity: 90.0,
            expires_at_ms: 500,
            now_ms: 100,
            request_meter_fact: request_meter_fact(9302, 9002, 100),
        },
    )
    .await
    .unwrap();

    let settled = capture_account_hold(
        &store,
        CaptureAccountHoldInput {
            hold_id: 9202,
            captured_quantity: 65.0,
            provider_cost_amount: 22.5,
            retail_charge_amount: 65.0,
            settled_at_ms: 120,
        },
    )
    .await
    .unwrap();

    assert_eq!(settled.settlement.status, RequestSettlementStatus::PartiallyReleased);
    assert_eq!(settled.settlement.captured_credit_amount, 65.0);
    assert_eq!(settled.settlement.released_credit_amount, 25.0);

    let duplicate = capture_account_hold(
        &store,
        CaptureAccountHoldInput {
            hold_id: 9202,
            captured_quantity: 90.0,
            provider_cost_amount: 99.0,
            retail_charge_amount: 90.0,
            settled_at_ms: 150,
        },
    )
    .await
    .unwrap();

    assert_eq!(duplicate.settlement, settled.settlement);

    let lots = store.list_account_benefit_lots().await.unwrap();
    let promo = lots.iter().find(|lot| lot.lot_id == 9111).unwrap();
    let cash = lots.iter().find(|lot| lot.lot_id == 9112).unwrap();
    assert_eq!(promo.remaining_quantity, 0.0);
    assert_eq!(promo.held_quantity, 0.0);
    assert_eq!(cash.remaining_quantity, 35.0);
    assert_eq!(cash.held_quantity, 0.0);

    let hold_allocations = store.list_account_hold_allocations().await.unwrap();
    assert_eq!(hold_allocations.len(), 2);
    let promo_allocation = hold_allocations
        .iter()
        .find(|allocation| allocation.lot_id == 9111)
        .unwrap();
    let cash_allocation = hold_allocations
        .iter()
        .find(|allocation| allocation.lot_id == 9112)
        .unwrap();
    assert_eq!(promo_allocation.captured_quantity, 50.0);
    assert_eq!(promo_allocation.released_quantity, 0.0);
    assert_eq!(cash_allocation.captured_quantity, 15.0);
    assert_eq!(cash_allocation.released_quantity, 25.0);

    let request_facts = store.list_request_meter_facts().await.unwrap();
    assert_eq!(request_facts.len(), 1);
    assert_eq!(request_facts[0].request_status, RequestStatus::Succeeded);
    assert_eq!(request_facts[0].usage_capture_status, UsageCaptureStatus::Captured);
    assert_eq!(request_facts[0].actual_credit_charge, Some(65.0));
    assert_eq!(request_facts[0].actual_provider_cost, Some(22.5));

    let settlements = store.list_request_settlement_records().await.unwrap();
    assert_eq!(settlements.len(), 1);

    let ledger_entries = store.list_account_ledger_entry_records().await.unwrap();
    assert_eq!(ledger_entries.len(), 3);

    let ledger_allocations = store.list_account_ledger_allocations().await.unwrap();
    assert_eq!(ledger_allocations.len(), 3);
    assert!(
        ledger_allocations
            .iter()
            .any(|allocation| allocation.lot_id == 9112 && allocation.quantity_delta == 25.0)
    );

    let snapshot = summarize_account_balance(&store, 9002, 200).await.unwrap();
    assert_eq!(snapshot.available_balance, 35.0);
    assert_eq!(snapshot.held_balance, 0.0);
    assert_eq!(snapshot.consumed_balance, 65.0);
}

#[tokio::test]
async fn releases_hold_without_consuming_lot_quantity() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    seed_account_with_lots(
        &store,
        9003,
        &[(9121, 30.0, AccountBenefitType::PromoCredit), (9122, 40.0, AccountBenefitType::CashCredit)],
    )
    .await;

    create_account_hold(
        &store,
        CreateAccountHoldInput {
            hold_id: 9203,
            account_id: 9003,
            request_id: 9303,
            estimated_quantity: 60.0,
            expires_at_ms: 500,
            now_ms: 100,
            request_meter_fact: request_meter_fact(9303, 9003, 100),
        },
    )
    .await
    .unwrap();

    let released = release_account_hold(
        &store,
        ReleaseAccountHoldInput {
            hold_id: 9203,
            settled_at_ms: 140,
        },
    )
    .await
    .unwrap();

    assert_eq!(released.settlement.status, RequestSettlementStatus::Released);
    assert_eq!(released.settlement.captured_credit_amount, 0.0);
    assert_eq!(released.settlement.released_credit_amount, 60.0);

    let lots = store.list_account_benefit_lots().await.unwrap();
    let promo = lots.iter().find(|lot| lot.lot_id == 9121).unwrap();
    let cash = lots.iter().find(|lot| lot.lot_id == 9122).unwrap();
    assert_eq!(promo.remaining_quantity, 30.0);
    assert_eq!(promo.held_quantity, 0.0);
    assert_eq!(cash.remaining_quantity, 40.0);
    assert_eq!(cash.held_quantity, 0.0);

    let request_facts = store.list_request_meter_facts().await.unwrap();
    assert_eq!(request_facts.len(), 1);
    assert_eq!(request_facts[0].request_status, RequestStatus::Failed);
    assert_eq!(request_facts[0].usage_capture_status, UsageCaptureStatus::Failed);
    assert_eq!(request_facts[0].actual_credit_charge, Some(0.0));

    let ledger_entries = store.list_account_ledger_entry_records().await.unwrap();
    assert_eq!(ledger_entries.len(), 2);
    assert!(
        ledger_entries
            .iter()
            .any(|entry| entry.entry_type == AccountLedgerEntryType::HoldRelease)
    );

    let ledger_allocations = store.list_account_ledger_allocations().await.unwrap();
    assert_eq!(ledger_allocations.len(), 2);

    let snapshot = summarize_account_balance(&store, 9003, 200).await.unwrap();
    assert_eq!(snapshot.available_balance, 70.0);
    assert_eq!(snapshot.held_balance, 0.0);
    assert_eq!(snapshot.consumed_balance, 0.0);
}

async fn seed_account_with_lots(
    store: &SqliteAdminStore,
    account_id: u64,
    lots: &[(u64, f64, AccountBenefitType)],
) {
    let account = AccountRecord::new(account_id, 1001, 2002, 9001, AccountType::Primary)
        .with_created_at_ms(10)
        .with_updated_at_ms(10);
    store.insert_account_record(&account).await.unwrap();

    for (index, (lot_id, quantity, benefit_type)) in lots.iter().enumerate() {
        let lot = AccountBenefitLotRecord::new(
            *lot_id,
            1001,
            2002,
            account_id,
            9001,
            *benefit_type,
        )
        .with_source_type(match benefit_type {
            AccountBenefitType::CashCredit => AccountBenefitSourceType::Recharge,
            _ => AccountBenefitSourceType::Coupon,
        })
        .with_original_quantity(*quantity)
        .with_remaining_quantity(*quantity)
        .with_created_at_ms(20 + u64::try_from(index).unwrap())
        .with_updated_at_ms(20 + u64::try_from(index).unwrap());
        store.insert_account_benefit_lot(&lot).await.unwrap();
    }
}

fn request_meter_fact(request_id: u64, account_id: u64, now_ms: u64) -> RequestMeterFactRecord {
    RequestMeterFactRecord::new(
        request_id,
        1001,
        2002,
        9001,
        account_id,
        "api_key",
        "responses",
        "openai",
        "gpt-4.1",
        "provider-openai-official",
    )
    .with_protocol_family("openai")
    .with_started_at_ms(now_ms)
    .with_created_at_ms(now_ms)
    .with_updated_at_ms(now_ms)
}
