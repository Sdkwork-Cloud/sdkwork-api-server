use sdkwork_api_domain_billing::{
    AccountBenefitLotRecord, AccountBenefitLotStatus, AccountBenefitSourceType, AccountBenefitType,
    AccountLedgerAllocationRecord, AccountLedgerEntryRecord, AccountLedgerEntryType, AccountRecord,
    AccountType,
};
use sdkwork_api_domain_payment::{
    FinanceDirection, FinanceEntryCode, FinanceJournalEntryRecord, FinanceJournalLineRecord,
    PaymentAttemptRecord, PaymentAttemptStatus, PaymentCallbackEventRecord,
    PaymentCallbackProcessingStatus, PaymentChannelPolicyRecord, PaymentGatewayAccountRecord,
    PaymentOrderRecord, PaymentOrderStatus, PaymentProviderCode, PaymentSessionKind,
    PaymentSessionRecord, PaymentSessionStatus, PaymentTransactionKind, PaymentTransactionRecord,
    ReconciliationMatchStatus, ReconciliationMatchSummaryRecord, RefundOrderRecord,
    RefundOrderStatus,
};
use sdkwork_api_storage_core::{AccountKernelStore, PaymentKernelStore};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[tokio::test]
async fn sqlite_payment_store_round_trips_order_attempt_session_and_transaction() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let order = PaymentOrderRecord::new(
        "payment-order-1",
        "commerce-order-1",
        1,
        0,
        7,
        "project-1",
        "recharge",
        "workspace",
        "project-1",
        "CNY",
        12_000,
    )
    .with_discount_minor(1_000)
    .with_subsidy_minor(500)
    .with_payable_minor(10_500)
    .with_captured_amount_minor(4_200)
    .with_provider_code(PaymentProviderCode::WeChatPay)
    .with_method_code("native_qr")
    .with_payment_status(PaymentOrderStatus::AwaitingCustomer)
    .with_created_at_ms(1_700_000_000)
    .with_updated_at_ms(1_700_000_001);
    store.insert_payment_order_record(&order).await.unwrap();

    let attempt = PaymentAttemptRecord::new(
        "payment-attempt-1",
        1,
        0,
        "payment-order-1",
        1,
        "gateway-account-1",
        PaymentProviderCode::WeChatPay,
        "native_qr",
        "portal_web",
        "idem-1",
    )
    .with_provider_payment_reference(Some("wx-prepay-1".to_owned()))
    .with_attempt_status(PaymentAttemptStatus::HandoffReady)
    .with_request_payload_hash("hash-1")
    .with_expires_at_ms(Some(1_700_000_600))
    .with_created_at_ms(1_700_000_010)
    .with_updated_at_ms(1_700_000_020);
    store.insert_payment_attempt_record(&attempt).await.unwrap();

    let session = PaymentSessionRecord::new(
        "payment-session-1",
        1,
        0,
        "payment-attempt-1",
        PaymentSessionKind::QrCode,
        PaymentSessionStatus::Open,
    )
    .with_display_reference(Some("ORDER-1".to_owned()))
    .with_qr_payload(Some("weixin://wxpay/demo".to_owned()))
    .with_expires_at_ms(1_700_000_600)
    .with_created_at_ms(1_700_000_030)
    .with_updated_at_ms(1_700_000_031);
    store.insert_payment_session_record(&session).await.unwrap();

    let transaction = PaymentTransactionRecord::new(
        "payment-tx-1",
        1,
        0,
        "payment-order-1",
        PaymentTransactionKind::Sale,
        PaymentProviderCode::WeChatPay,
        "provider-tx-1",
        "CNY",
        10_500,
        1_700_000_040,
    )
    .with_payment_attempt_id(Some("payment-attempt-1".to_owned()))
    .with_fee_minor(Some(30))
    .with_net_amount_minor(Some(10_470))
    .with_provider_status("succeeded")
    .with_created_at_ms(1_700_000_041);
    store
        .insert_payment_transaction_record(&transaction)
        .await
        .unwrap();

    let stored_order = store
        .find_payment_order_record("payment-order-1")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored_order, order);

    let attempts = store
        .list_payment_attempt_records_for_order("payment-order-1")
        .await
        .unwrap();
    assert_eq!(attempts, vec![attempt]);

    let sessions = store
        .list_payment_session_records_for_attempt("payment-attempt-1")
        .await
        .unwrap();
    assert_eq!(sessions, vec![session]);

    let transactions = store
        .list_payment_transaction_records_for_order("payment-order-1")
        .await
        .unwrap();
    assert_eq!(transactions, vec![transaction]);
}

#[tokio::test]
async fn sqlite_payment_store_round_trips_callback_and_refund_records() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let order = PaymentOrderRecord::new(
        "payment-order-2",
        "commerce-order-2",
        2,
        0,
        9,
        "project-2",
        "recharge",
        "workspace",
        "project-2",
        "USD",
        5_000,
    )
    .with_provider_code(PaymentProviderCode::Stripe)
    .with_captured_amount_minor(5_000)
    .with_payment_status(PaymentOrderStatus::Captured)
    .with_created_at_ms(1_710_000_000)
    .with_updated_at_ms(1_710_000_001);
    store.insert_payment_order_record(&order).await.unwrap();

    let callback = PaymentCallbackEventRecord::new(
        "callback-1",
        2,
        0,
        PaymentProviderCode::Stripe,
        "gateway-account-2",
        "checkout.session.completed",
        "evt_123",
        "dedupe_evt_123",
        1_710_000_010,
    )
    .with_payment_order_id(Some("payment-order-2".to_owned()))
    .with_signature_status("verified")
    .with_processing_status(PaymentCallbackProcessingStatus::Processed)
    .with_payload_json(Some("{\"id\":\"evt_123\"}".to_owned()))
    .with_processed_at_ms(Some(1_710_000_011));
    store
        .insert_payment_callback_event_record(&callback)
        .await
        .unwrap();

    let refund = RefundOrderRecord::new(
        "refund-order-1",
        2,
        0,
        "payment-order-2",
        "commerce-order-2",
        "buyer_request",
        "portal_user",
        "user-9",
        "USD",
        2_000,
    )
    .with_approved_amount_minor(Some(2_000))
    .with_refunded_amount_minor(1_000)
    .with_refund_status(RefundOrderStatus::Processing)
    .with_created_at_ms(1_710_000_020)
    .with_updated_at_ms(1_710_000_021);
    store.insert_refund_order_record(&refund).await.unwrap();

    let callbacks = store.list_payment_callback_event_records().await.unwrap();
    assert_eq!(callbacks, vec![callback]);

    let refunds = store
        .list_refund_order_records_for_payment_order("payment-order-2")
        .await
        .unwrap();
    assert_eq!(refunds, vec![refund]);
}

#[tokio::test]
async fn sqlite_payment_store_round_trips_finance_and_reconciliation_records() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let journal = FinanceJournalEntryRecord::new(
        "journal-1",
        3,
        0,
        "payment_order",
        "payment-order-3",
        FinanceEntryCode::CustomerPrepaidLiabilityIncrease,
        "USD",
        1_720_000_000,
    )
    .with_entry_status("posted")
    .with_created_at_ms(1_720_000_001);
    store
        .insert_finance_journal_entry_record(&journal)
        .await
        .unwrap();

    let journal_line = FinanceJournalLineRecord::new(
        "journal-line-1",
        3,
        0,
        "journal-1",
        1,
        "gateway_clearing_asset",
        FinanceDirection::Debit,
        3_000,
    )
    .with_party_type(Some("payment_order".to_owned()))
    .with_party_id(Some("payment-order-3".to_owned()));
    store
        .insert_finance_journal_line_record(&journal_line)
        .await
        .unwrap();

    let reconciliation = ReconciliationMatchSummaryRecord::new(
        "recon-line-1",
        3,
        0,
        "recon-batch-1",
        "provider-tx-3",
        ReconciliationMatchStatus::MismatchAmount,
        3_000,
    )
    .with_local_amount_minor(Some(2_900))
    .with_payment_order_id(Some("payment-order-3".to_owned()))
    .with_reason_code(Some("provider_fee_pending".to_owned()))
    .with_created_at_ms(1_720_000_010)
    .with_updated_at_ms(1_720_000_011);
    store
        .insert_reconciliation_match_summary_record(&reconciliation)
        .await
        .unwrap();

    let journals = store.list_finance_journal_entry_records().await.unwrap();
    assert_eq!(journals, vec![journal]);

    let journal_lines = store
        .list_finance_journal_line_records("journal-1")
        .await
        .unwrap();
    assert_eq!(journal_lines, vec![journal_line]);

    let reconciliations = store
        .list_reconciliation_match_summary_records("recon-batch-1")
        .await
        .unwrap();
    assert_eq!(reconciliations, vec![reconciliation.clone()]);

    let fetched_reconciliation = store
        .find_reconciliation_match_summary_record("recon-line-1")
        .await
        .unwrap();
    assert_eq!(fetched_reconciliation, Some(reconciliation));

    let missing_reconciliation = store
        .find_reconciliation_match_summary_record("recon-line-missing")
        .await
        .unwrap();
    assert_eq!(missing_reconciliation, None);

    let gateway_account =
        PaymentGatewayAccountRecord::new("gateway-account-1", 3, 0, PaymentProviderCode::Stripe)
            .with_environment("production")
            .with_merchant_id("merchant-1")
            .with_app_id("app-1")
            .with_status("active")
            .with_priority(100)
            .with_created_at_ms(1_720_000_020)
            .with_updated_at_ms(1_720_000_021);
    store
        .insert_payment_gateway_account_record(&gateway_account)
        .await
        .unwrap();

    let channel_policy = PaymentChannelPolicyRecord::new(
        "channel-policy-1",
        3,
        0,
        PaymentProviderCode::Stripe,
        "hosted_checkout",
    )
    .with_scene_code("recharge_pack")
    .with_currency_code("USD")
    .with_client_kind("portal_web")
    .with_status("active")
    .with_priority(80)
    .with_created_at_ms(1_720_000_022)
    .with_updated_at_ms(1_720_000_023);
    store
        .insert_payment_channel_policy_record(&channel_policy)
        .await
        .unwrap();

    let gateway_accounts = store.list_payment_gateway_account_records().await.unwrap();
    assert_eq!(gateway_accounts, vec![gateway_account]);

    let channel_policies = store.list_payment_channel_policy_records().await.unwrap();
    assert_eq!(channel_policies, vec![channel_policy]);
}

#[tokio::test]
async fn sqlite_payment_store_applies_account_grant_reversal_idempotently() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool.clone());

    let account = AccountRecord::new(7001, 1001, 0, 9001, AccountType::Primary)
        .with_created_at_ms(10)
        .with_updated_at_ms(10);
    store.insert_account_record(&account).await.unwrap();

    let granted_lot =
        AccountBenefitLotRecord::new(8001, 1001, 0, 7001, 9001, AccountBenefitType::CashCredit)
            .with_source_type(AccountBenefitSourceType::Recharge)
            .with_original_quantity(100.0)
            .with_remaining_quantity(100.0)
            .with_status(AccountBenefitLotStatus::Active)
            .with_issued_at_ms(20)
            .with_created_at_ms(20)
            .with_updated_at_ms(20);
    store.insert_account_benefit_lot(&granted_lot).await.unwrap();

    let reversal_entry = AccountLedgerEntryRecord::new(
        8201,
        1001,
        0,
        7001,
        9001,
        AccountLedgerEntryType::Refund,
    )
    .with_benefit_type(Some("request_allowance".to_owned()))
    .with_quantity(-40.0)
    .with_amount(-16.0)
    .with_created_at_ms(30);
    let reversal_allocation = AccountLedgerAllocationRecord::new(8301, 1001, 0, 8201, 8001)
        .with_quantity_delta(-40.0)
        .with_created_at_ms(30);

    let applied = store
        .apply_refund_order_account_grant_reversal(
            "refund-order-account-1",
            8001,
            40.0,
            30,
            &reversal_entry,
            &reversal_allocation,
        )
        .await
        .unwrap();
    assert!(applied);

    let replayed = store
        .apply_refund_order_account_grant_reversal(
            "refund-order-account-1",
            8001,
            40.0,
            35,
            &reversal_entry,
            &reversal_allocation,
        )
        .await
        .unwrap();
    assert!(!replayed);

    let stored_lot = store
        .list_account_benefit_lots()
        .await
        .unwrap()
        .into_iter()
        .find(|lot| lot.lot_id == 8001)
        .unwrap();
    assert_eq!(stored_lot.remaining_quantity, 60.0);
    assert_eq!(stored_lot.status, AccountBenefitLotStatus::Active);

    let ledger_entries = store.list_account_ledger_entry_records().await.unwrap();
    assert_eq!(ledger_entries, vec![reversal_entry]);

    let ledger_allocations = store.list_account_ledger_allocations().await.unwrap();
    assert_eq!(ledger_allocations, vec![reversal_allocation]);

    let processing_steps: Vec<(String, String)> = sqlx::query_as(
        "SELECT refund_order_id, step_key
         FROM ai_refund_order_processing_steps
         ORDER BY refund_order_id, step_key",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        processing_steps,
        vec![("refund-order-account-1".to_owned(), "account".to_owned())]
    );
}
