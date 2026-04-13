use sdkwork_api_app_payment::{
    approve_refund_order_request, ensure_commerce_payment_checkout, finalize_refund_order_success,
    ingest_payment_callback, request_payment_order_refund, request_portal_commerce_order_refund,
    PaymentCallbackIntakeDisposition, PaymentCallbackIntakeRequest,
    PaymentCallbackNormalizedOutcome, PaymentSubjectScope,
};
use sdkwork_api_domain_billing::{
    AccountBenefitSourceType, AccountBenefitType, AccountLedgerEntryType, AccountType,
};
use sdkwork_api_domain_commerce::CommerceOrderRecord;
use sdkwork_api_domain_payment::{
    FinanceDirection, FinanceEntryCode, PaymentAttemptStatus, PaymentCallbackProcessingStatus,
    PaymentChannelPolicyRecord, PaymentGatewayAccountRecord, PaymentOrderStatus,
    PaymentProviderCode, PaymentRefundStatus, PaymentSessionStatus, PaymentTransactionKind,
    RefundOrderStatus,
};
use sdkwork_api_storage_core::{AccountKernelStore, PaymentKernelStore};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

fn build_pending_commerce_order(
    order_id: &str,
    project_id: &str,
    user_id: &str,
    created_at_ms: u64,
) -> CommerceOrderRecord {
    CommerceOrderRecord::new(
        order_id,
        project_id,
        user_id,
        "recharge_pack",
        "pack-100k",
        "Boost 100k",
        4_000,
        4_000,
        "$40.00",
        "$40.00",
        100_000,
        0,
        "pending_payment",
        "workspace_seed",
        created_at_ms,
    )
}

async fn prepare_checkout(
    store: &SqliteAdminStore,
    scope: &PaymentSubjectScope,
    order_id: &str,
    project_id: &str,
    user_id: &str,
    created_at_ms: u64,
) -> (
    CommerceOrderRecord,
    sdkwork_api_domain_payment::PaymentOrderRecord,
    sdkwork_api_domain_payment::PaymentAttemptRecord,
    sdkwork_api_domain_payment::PaymentSessionRecord,
) {
    let order = build_pending_commerce_order(order_id, project_id, user_id, created_at_ms);
    store.insert_commerce_order(&order).await.unwrap();
    let checkout = ensure_commerce_payment_checkout(store, scope, &order, "portal_web")
        .await
        .unwrap();

    (
        order,
        checkout.payment_order_opt.unwrap(),
        checkout.payment_attempt_opt.unwrap(),
        checkout.payment_session_opt.unwrap(),
    )
}

async fn prepare_checkout_without_persisted_order(
    store: &SqliteAdminStore,
    scope: &PaymentSubjectScope,
    order_id: &str,
    project_id: &str,
    user_id: &str,
    created_at_ms: u64,
) -> (
    CommerceOrderRecord,
    sdkwork_api_domain_payment::PaymentOrderRecord,
    sdkwork_api_domain_payment::PaymentAttemptRecord,
    sdkwork_api_domain_payment::PaymentSessionRecord,
) {
    let order = build_pending_commerce_order(order_id, project_id, user_id, created_at_ms);
    let checkout = ensure_commerce_payment_checkout(store, scope, &order, "portal_web")
        .await
        .unwrap();

    (
        order,
        checkout.payment_order_opt.unwrap(),
        checkout.payment_attempt_opt.unwrap(),
        checkout.payment_session_opt.unwrap(),
    )
}

async fn settle_checkout(
    store: &SqliteAdminStore,
    scope: &PaymentSubjectScope,
    order_id: &str,
    project_id: &str,
    user_id: &str,
    created_at_ms: u64,
) -> (
    CommerceOrderRecord,
    sdkwork_api_domain_payment::PaymentOrderRecord,
    sdkwork_api_domain_payment::PaymentAttemptRecord,
) {
    let (commerce_order, payment_order, payment_attempt, _payment_session) =
        prepare_checkout(store, scope, order_id, project_id, user_id, created_at_ms).await;

    let callback = ingest_payment_callback(
        store,
        &PaymentCallbackIntakeRequest::new(
            scope.clone(),
            PaymentProviderCode::Stripe,
            "stripe-main",
            "checkout.session.completed",
            &format!("evt_settled_{order_id}"),
            &format!("dedupe_evt_settled_{order_id}"),
            created_at_ms.saturating_add(120),
        )
        .with_payment_order_id(Some(payment_order.payment_order_id.clone()))
        .with_payment_attempt_id(Some(payment_attempt.payment_attempt_id.clone()))
        .with_provider_transaction_id(Some(format!("pi_{order_id}")))
        .with_signature_status("verified")
        .with_provider_status(Some("succeeded".to_owned()))
        .with_amount_minor(Some(payment_order.payable_minor))
        .with_currency_code(Some(payment_order.currency_code.clone()))
        .with_payload_json(Some(format!("{{\"id\":\"evt_settled_{order_id}\"}}"))),
    )
    .await
    .unwrap();

    assert_eq!(
        callback.disposition,
        PaymentCallbackIntakeDisposition::Processed
    );
    assert_eq!(
        callback.normalized_outcome,
        Some(PaymentCallbackNormalizedOutcome::Settled)
    );

    (
        commerce_order,
        store
            .find_payment_order_record(&payment_order.payment_order_id)
            .await
            .unwrap()
            .unwrap(),
        payment_attempt,
    )
}

async fn seed_failover_route(
    store: &SqliteAdminStore,
    scope: &PaymentSubjectScope,
    provider_code: PaymentProviderCode,
    gateway_account_id: &str,
    priority: i32,
    method_code: &str,
    client_kind: &str,
) {
    store
        .insert_payment_gateway_account_record(
            &PaymentGatewayAccountRecord::new(
                gateway_account_id,
                scope.tenant_id,
                scope.organization_id,
                provider_code,
            )
            .with_environment("production")
            .with_merchant_id(format!("merchant_{gateway_account_id}"))
            .with_app_id(format!("app_{gateway_account_id}"))
            .with_status("active")
            .with_priority(priority)
            .with_created_at_ms(1_720_100_000)
            .with_updated_at_ms(1_720_100_001),
        )
        .await
        .unwrap();

    store
        .insert_payment_channel_policy_record(
            &PaymentChannelPolicyRecord::new(
                format!("channel_policy_{gateway_account_id}"),
                scope.tenant_id,
                scope.organization_id,
                provider_code,
                method_code,
            )
            .with_scene_code("recharge_pack")
            .with_currency_code("USD")
            .with_client_kind(client_kind)
            .with_status("active")
            .with_priority(priority)
            .with_created_at_ms(1_720_100_002)
            .with_updated_at_ms(1_720_100_003),
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn verified_settlement_callback_is_idempotent_and_captures_canonical_payment_state() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let scope = PaymentSubjectScope::new(9, 0, 42);
    let (_commerce_order, payment_order, payment_attempt, _payment_session) = prepare_checkout(
        &store,
        &scope,
        "commerce-order-callback-1",
        "project-callback-1",
        "user-callback-1",
        1_720_000_000,
    )
    .await;

    let request = PaymentCallbackIntakeRequest::new(
        scope.clone(),
        PaymentProviderCode::Stripe,
        "stripe-main",
        "checkout.session.completed",
        "evt_settled_1",
        "dedupe_evt_settled_1",
        1_720_000_120,
    )
    .with_payment_order_id(Some(payment_order.payment_order_id.clone()))
    .with_payment_attempt_id(Some(payment_attempt.payment_attempt_id.clone()))
    .with_provider_transaction_id(Some("pi_123456789".to_owned()))
    .with_signature_status("verified")
    .with_provider_status(Some("succeeded".to_owned()))
    .with_amount_minor(Some(4_000))
    .with_currency_code(Some("USD".to_owned()))
    .with_payload_json(Some("{\"id\":\"evt_settled_1\"}".to_owned()));

    let first = ingest_payment_callback(&store, &request).await.unwrap();
    let duplicate = ingest_payment_callback(&store, &request).await.unwrap();

    assert_eq!(
        first.disposition,
        PaymentCallbackIntakeDisposition::Processed
    );
    assert_eq!(
        first.normalized_outcome,
        Some(PaymentCallbackNormalizedOutcome::Settled)
    );
    assert_eq!(
        duplicate.disposition,
        PaymentCallbackIntakeDisposition::Duplicate
    );
    assert_eq!(
        first.callback_event.processing_status,
        PaymentCallbackProcessingStatus::Processed
    );
    assert_eq!(
        first.payment_order_opt.as_ref().unwrap().payment_status,
        PaymentOrderStatus::Captured
    );
    assert_eq!(
        first.payment_order_opt.as_ref().unwrap().fulfillment_status,
        "fulfilled"
    );
    assert_eq!(
        first.payment_attempt_opt.as_ref().unwrap().attempt_status,
        PaymentAttemptStatus::Succeeded
    );
    assert_eq!(
        first.payment_session_opt.as_ref().unwrap().session_status,
        PaymentSessionStatus::Settled
    );
    assert_eq!(
        first
            .payment_transaction_opt
            .as_ref()
            .unwrap()
            .provider_transaction_id,
        "pi_123456789"
    );

    let callbacks = store.list_payment_callback_event_records().await.unwrap();
    assert_eq!(callbacks.len(), 1);
    assert_eq!(
        store
            .list_payment_transaction_records_for_order(&payment_order.payment_order_id)
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        store
            .find_payment_order_record(&payment_order.payment_order_id)
            .await
            .unwrap()
            .unwrap()
            .payment_status,
        PaymentOrderStatus::Captured
    );
    assert_eq!(
        store
            .find_payment_order_record(&payment_order.payment_order_id)
            .await
            .unwrap()
            .unwrap()
            .fulfillment_status,
        "fulfilled"
    );
    assert_eq!(
        store
            .list_commerce_orders()
            .await
            .unwrap()
            .into_iter()
            .find(|order| order.order_id == payment_order.commerce_order_id)
            .unwrap()
            .status,
        "fulfilled"
    );

    let account = store
        .find_account_record_by_owner(
            scope.tenant_id,
            scope.organization_id,
            scope.user_id,
            AccountType::Primary,
        )
        .await
        .unwrap()
        .expect("primary account should be created for settled recharge");
    let lots = store.list_account_benefit_lots().await.unwrap();
    assert_eq!(lots.len(), 1);
    assert_eq!(lots[0].account_id, account.account_id);
    assert_eq!(lots[0].benefit_type, AccountBenefitType::RequestAllowance);
    assert_eq!(lots[0].source_type, AccountBenefitSourceType::Order);
    assert_eq!(lots[0].original_quantity, 100_000.0);
    assert_eq!(lots[0].remaining_quantity, 100_000.0);

    let ledger_entries = store.list_account_ledger_entry_records().await.unwrap();
    assert_eq!(ledger_entries.len(), 1);
    assert_eq!(ledger_entries[0].account_id, account.account_id);
    assert_eq!(
        ledger_entries[0].entry_type,
        AccountLedgerEntryType::GrantIssue
    );
    assert_eq!(ledger_entries[0].quantity, 100_000.0);

    let ledger_allocations = store.list_account_ledger_allocations().await.unwrap();
    assert_eq!(ledger_allocations.len(), 1);
    assert_eq!(ledger_allocations[0].lot_id, lots[0].lot_id);
    assert_eq!(
        ledger_allocations[0].ledger_entry_id,
        ledger_entries[0].ledger_entry_id
    );
    assert_eq!(ledger_allocations[0].quantity_delta, 100_000.0);
}

#[tokio::test]
async fn verified_authorization_callback_marks_payment_authorized_without_fulfillment_side_effects()
{
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let scope = PaymentSubjectScope::new(10, 0, 43);
    let (_commerce_order, payment_order, payment_attempt, _payment_session) = prepare_checkout(
        &store,
        &scope,
        "commerce-order-callback-auth-1",
        "project-callback-auth-1",
        "user-callback-auth-1",
        1_720_000_150,
    )
    .await;

    let result = ingest_payment_callback(
        &store,
        &PaymentCallbackIntakeRequest::new(
            scope.clone(),
            PaymentProviderCode::Stripe,
            "stripe-main",
            "payment_intent.amount_capturable_updated",
            "evt_authorized_1",
            "dedupe_evt_authorized_1",
            1_720_000_210,
        )
        .with_payment_order_id(Some(payment_order.payment_order_id.clone()))
        .with_payment_attempt_id(Some(payment_attempt.payment_attempt_id.clone()))
        .with_provider_transaction_id(Some("pi_auth_1".to_owned()))
        .with_signature_status("verified")
        .with_provider_status(Some("requires_capture".to_owned()))
        .with_amount_minor(Some(4_000))
        .with_currency_code(Some("USD".to_owned()))
        .with_payload_json(Some(
            "{\"id\":\"evt_authorized_1\",\"status\":\"requires_capture\"}".to_owned(),
        )),
    )
    .await
    .unwrap();

    assert_eq!(
        result.disposition,
        PaymentCallbackIntakeDisposition::Processed
    );
    assert!(format!("{:?}", result.normalized_outcome)
        .to_ascii_lowercase()
        .contains("authorized"));
    assert_eq!(
        result
            .payment_order_opt
            .as_ref()
            .unwrap()
            .payment_status
            .as_str(),
        "authorized"
    );
    assert_eq!(
        result
            .payment_order_opt
            .as_ref()
            .unwrap()
            .fulfillment_status,
        "authorized_pending_capture"
    );
    assert_eq!(
        result
            .payment_attempt_opt
            .as_ref()
            .unwrap()
            .attempt_status
            .as_str(),
        "authorized"
    );
    assert_eq!(
        result
            .payment_session_opt
            .as_ref()
            .unwrap()
            .session_status
            .as_str(),
        "authorized"
    );
    assert_eq!(
        result
            .payment_transaction_opt
            .as_ref()
            .unwrap()
            .transaction_kind
            .as_str(),
        "authorization"
    );

    let commerce_order = store
        .list_commerce_orders()
        .await
        .unwrap()
        .into_iter()
        .find(|order| order.order_id == payment_order.commerce_order_id)
        .unwrap();
    assert_eq!(commerce_order.status, "pending_payment");
    assert!(store
        .find_account_record_by_owner(
            scope.tenant_id,
            scope.organization_id,
            scope.user_id,
            AccountType::Primary,
        )
        .await
        .unwrap()
        .is_none());
    assert!(store.list_account_benefit_lots().await.unwrap().is_empty());
    assert!(store
        .list_account_ledger_entry_records()
        .await
        .unwrap()
        .is_empty());
    assert!(store
        .list_payment_transaction_records_for_order(&payment_order.payment_order_id)
        .await
        .unwrap()
        .iter()
        .all(|transaction| transaction.transaction_kind.as_str() != "sale"));
}

#[tokio::test]
async fn verified_partial_settlement_keeps_order_unfulfilled_and_limits_refunds() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let scope = PaymentSubjectScope::new(11, 0, 44);
    let (_commerce_order, payment_order, payment_attempt, _payment_session) = prepare_checkout(
        &store,
        &scope,
        "commerce-order-callback-partial-1",
        "project-callback-partial-1",
        "user-callback-partial-1",
        1_720_000_240,
    )
    .await;

    let result = ingest_payment_callback(
        &store,
        &PaymentCallbackIntakeRequest::new(
            scope.clone(),
            PaymentProviderCode::Stripe,
            "stripe-main",
            "checkout.session.completed",
            "evt_partial_settled_1",
            "dedupe_evt_partial_settled_1",
            1_720_000_300,
        )
        .with_payment_order_id(Some(payment_order.payment_order_id.clone()))
        .with_payment_attempt_id(Some(payment_attempt.payment_attempt_id.clone()))
        .with_provider_transaction_id(Some("pi_partial_settled_1".to_owned()))
        .with_signature_status("verified")
        .with_provider_status(Some("succeeded".to_owned()))
        .with_amount_minor(Some(1_000))
        .with_currency_code(Some("USD".to_owned()))
        .with_payload_json(Some("{\"id\":\"evt_partial_settled_1\"}".to_owned())),
    )
    .await
    .unwrap();

    assert_eq!(
        result.disposition,
        PaymentCallbackIntakeDisposition::Processed
    );
    assert_eq!(
        result.normalized_outcome,
        Some(PaymentCallbackNormalizedOutcome::Settled)
    );
    assert_eq!(
        result
            .payment_order_opt
            .as_ref()
            .unwrap()
            .payment_status
            .as_str(),
        "partially_captured"
    );
    assert_eq!(
        result
            .payment_order_opt
            .as_ref()
            .unwrap()
            .fulfillment_status,
        "partial_capture_pending_review"
    );

    let commerce_order = store
        .list_commerce_orders()
        .await
        .unwrap()
        .into_iter()
        .find(|order| order.order_id == payment_order.commerce_order_id)
        .unwrap();
    assert_eq!(commerce_order.status, "pending_payment");
    assert!(store
        .find_account_record_by_owner(
            scope.tenant_id,
            scope.organization_id,
            scope.user_id,
            AccountType::Primary,
        )
        .await
        .unwrap()
        .is_none());
    assert!(store.list_account_benefit_lots().await.unwrap().is_empty());
    assert!(store
        .list_account_ledger_entry_records()
        .await
        .unwrap()
        .is_empty());

    let transactions = store
        .list_payment_transaction_records_for_order(&payment_order.payment_order_id)
        .await
        .unwrap();
    assert_eq!(transactions.len(), 1);
    assert_eq!(
        transactions[0].transaction_kind,
        PaymentTransactionKind::Sale
    );
    assert_eq!(transactions[0].amount_minor, 1_000);

    let oversized_refund_error = request_payment_order_refund(
        &store,
        &scope,
        &payment_order.payment_order_id,
        "customer_request",
        1_500,
        "portal_user",
        "portal-user-partial-1",
        1_720_000_360,
    )
    .await
    .unwrap_err();
    assert!(oversized_refund_error
        .to_string()
        .contains("remaining refundable amount 1000"));

    let refund_order = request_payment_order_refund(
        &store,
        &scope,
        &payment_order.payment_order_id,
        "customer_request",
        1_000,
        "portal_user",
        "portal-user-partial-1",
        1_720_000_420,
    )
    .await
    .unwrap();
    assert_eq!(refund_order.requested_amount_minor, 1_000);
}

#[tokio::test]
async fn full_capture_replay_upgrades_partial_capture_without_duplicate_sale_side_effects() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let scope = PaymentSubjectScope::new(12, 0, 45);
    let (_commerce_order, payment_order, payment_attempt, _payment_session) = prepare_checkout(
        &store,
        &scope,
        "commerce-order-callback-partial-2",
        "project-callback-partial-2",
        "user-callback-partial-2",
        1_720_000_460,
    )
    .await;

    let first = ingest_payment_callback(
        &store,
        &PaymentCallbackIntakeRequest::new(
            scope.clone(),
            PaymentProviderCode::Stripe,
            "stripe-main",
            "checkout.session.completed",
            "evt_partial_settled_2",
            "dedupe_evt_partial_settled_2",
            1_720_000_520,
        )
        .with_payment_order_id(Some(payment_order.payment_order_id.clone()))
        .with_payment_attempt_id(Some(payment_attempt.payment_attempt_id.clone()))
        .with_provider_transaction_id(Some("pi_partial_upgrade_1".to_owned()))
        .with_signature_status("verified")
        .with_provider_status(Some("succeeded".to_owned()))
        .with_amount_minor(Some(1_000))
        .with_currency_code(Some("USD".to_owned()))
        .with_payload_json(Some("{\"id\":\"evt_partial_settled_2\"}".to_owned())),
    )
    .await
    .unwrap();
    assert_eq!(
        first
            .payment_order_opt
            .as_ref()
            .unwrap()
            .payment_status
            .as_str(),
        "partially_captured"
    );

    let second = ingest_payment_callback(
        &store,
        &PaymentCallbackIntakeRequest::new(
            scope.clone(),
            PaymentProviderCode::Stripe,
            "stripe-main",
            "checkout.session.completed",
            "evt_full_settled_2",
            "dedupe_evt_full_settled_2",
            1_720_000_640,
        )
        .with_payment_order_id(Some(payment_order.payment_order_id.clone()))
        .with_payment_attempt_id(Some(payment_attempt.payment_attempt_id.clone()))
        .with_provider_transaction_id(Some("pi_partial_upgrade_1".to_owned()))
        .with_signature_status("verified")
        .with_provider_status(Some("succeeded".to_owned()))
        .with_amount_minor(Some(4_000))
        .with_currency_code(Some("USD".to_owned()))
        .with_payload_json(Some("{\"id\":\"evt_full_settled_2\"}".to_owned())),
    )
    .await
    .unwrap();

    assert_eq!(
        second.payment_order_opt.as_ref().unwrap().payment_status,
        PaymentOrderStatus::Captured
    );
    assert_eq!(
        second
            .payment_order_opt
            .as_ref()
            .unwrap()
            .fulfillment_status,
        "fulfilled"
    );

    let commerce_order = store
        .list_commerce_orders()
        .await
        .unwrap()
        .into_iter()
        .find(|order| order.order_id == payment_order.commerce_order_id)
        .unwrap();
    assert_eq!(commerce_order.status, "fulfilled");

    let transactions = store
        .list_payment_transaction_records_for_order(&payment_order.payment_order_id)
        .await
        .unwrap();
    assert_eq!(transactions.len(), 1);
    assert_eq!(
        transactions[0].transaction_kind,
        PaymentTransactionKind::Sale
    );
    assert_eq!(
        transactions[0].provider_transaction_id,
        "pi_partial_upgrade_1"
    );
    assert_eq!(transactions[0].amount_minor, 4_000);

    let account = store
        .find_account_record_by_owner(
            scope.tenant_id,
            scope.organization_id,
            scope.user_id,
            AccountType::Primary,
        )
        .await
        .unwrap()
        .expect("primary account should be created only after full capture");
    let lots = store.list_account_benefit_lots().await.unwrap();
    assert_eq!(lots.len(), 1);
    assert_eq!(lots[0].account_id, account.account_id);

    let ledger_entries = store.list_account_ledger_entry_records().await.unwrap();
    assert_eq!(ledger_entries.len(), 1);
    assert_eq!(ledger_entries[0].account_id, account.account_id);
}

#[tokio::test]
async fn distinct_partial_captures_accumulate_without_conflict_until_threshold() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let scope = PaymentSubjectScope::new(13, 0, 46);
    let (_commerce_order, payment_order, payment_attempt, _payment_session) = prepare_checkout(
        &store,
        &scope,
        "commerce-order-callback-multi-1",
        "project-callback-multi-1",
        "user-callback-multi-1",
        1_720_000_700,
    )
    .await;

    for (event_id, dedupe_key, provider_transaction_id, amount_minor, received_at_ms) in [
        (
            "evt_multi_capture_1",
            "dedupe_multi_capture_1",
            "pi_multi_capture_1",
            1_000_u64,
            1_720_000_760_u64,
        ),
        (
            "evt_multi_capture_2",
            "dedupe_multi_capture_2",
            "pi_multi_capture_2",
            1_500_u64,
            1_720_000_820_u64,
        ),
    ] {
        ingest_payment_callback(
            &store,
            &PaymentCallbackIntakeRequest::new(
                scope.clone(),
                PaymentProviderCode::Stripe,
                "stripe-main",
                "checkout.session.completed",
                event_id,
                dedupe_key,
                received_at_ms,
            )
            .with_payment_order_id(Some(payment_order.payment_order_id.clone()))
            .with_payment_attempt_id(Some(payment_attempt.payment_attempt_id.clone()))
            .with_provider_transaction_id(Some(provider_transaction_id.to_owned()))
            .with_signature_status("verified")
            .with_provider_status(Some("succeeded".to_owned()))
            .with_amount_minor(Some(amount_minor))
            .with_currency_code(Some("USD".to_owned()))
            .with_payload_json(Some(format!("{{\"id\":\"{event_id}\"}}"))),
        )
        .await
        .unwrap();
    }

    let stored_payment_order = store
        .find_payment_order_record(&payment_order.payment_order_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        stored_payment_order.payment_status.as_str(),
        "partially_captured"
    );
    assert_eq!(stored_payment_order.captured_amount_minor, 2_500);
    assert_eq!(
        stored_payment_order.fulfillment_status,
        "partial_capture_pending_review"
    );

    let transactions = store
        .list_payment_transaction_records_for_order(&payment_order.payment_order_id)
        .await
        .unwrap()
        .into_iter()
        .filter(|transaction| transaction.transaction_kind == PaymentTransactionKind::Sale)
        .collect::<Vec<_>>();
    assert_eq!(transactions.len(), 2);
    assert!(transactions
        .iter()
        .any(|transaction| transaction.provider_transaction_id == "pi_multi_capture_1"));
    assert!(transactions
        .iter()
        .any(|transaction| transaction.provider_transaction_id == "pi_multi_capture_2"));
    assert!(store
        .list_reconciliation_match_summary_records(&format!(
            "payment_conflict_batch_{}",
            payment_order.payment_order_id
        ))
        .await
        .unwrap()
        .is_empty());

    let commerce_order = store
        .list_commerce_orders()
        .await
        .unwrap()
        .into_iter()
        .find(|order| order.order_id == payment_order.commerce_order_id)
        .unwrap();
    assert_eq!(commerce_order.status, "pending_payment");
    assert!(store.list_account_benefit_lots().await.unwrap().is_empty());
    assert!(store
        .list_account_ledger_entry_records()
        .await
        .unwrap()
        .is_empty());

    let oversized_refund_error = request_payment_order_refund(
        &store,
        &scope,
        &payment_order.payment_order_id,
        "customer_request",
        2_600,
        "portal_user",
        "portal-user-multi-1",
        1_720_000_900,
    )
    .await
    .unwrap_err();
    assert!(oversized_refund_error
        .to_string()
        .contains("remaining refundable amount 2500"));
}

#[tokio::test]
async fn final_distinct_capture_crosses_threshold_and_fulfills_once() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let scope = PaymentSubjectScope::new(14, 0, 47);
    let (_commerce_order, payment_order, payment_attempt, _payment_session) = prepare_checkout(
        &store,
        &scope,
        "commerce-order-callback-multi-2",
        "project-callback-multi-2",
        "user-callback-multi-2",
        1_720_000_940,
    )
    .await;

    for (event_id, dedupe_key, provider_transaction_id, amount_minor, received_at_ms) in [
        (
            "evt_multi_threshold_1",
            "dedupe_multi_threshold_1",
            "pi_multi_threshold_1",
            1_000_u64,
            1_720_001_000_u64,
        ),
        (
            "evt_multi_threshold_2",
            "dedupe_multi_threshold_2",
            "pi_multi_threshold_2",
            1_500_u64,
            1_720_001_060_u64,
        ),
        (
            "evt_multi_threshold_3",
            "dedupe_multi_threshold_3",
            "pi_multi_threshold_3",
            1_500_u64,
            1_720_001_120_u64,
        ),
    ] {
        ingest_payment_callback(
            &store,
            &PaymentCallbackIntakeRequest::new(
                scope.clone(),
                PaymentProviderCode::Stripe,
                "stripe-main",
                "checkout.session.completed",
                event_id,
                dedupe_key,
                received_at_ms,
            )
            .with_payment_order_id(Some(payment_order.payment_order_id.clone()))
            .with_payment_attempt_id(Some(payment_attempt.payment_attempt_id.clone()))
            .with_provider_transaction_id(Some(provider_transaction_id.to_owned()))
            .with_signature_status("verified")
            .with_provider_status(Some("succeeded".to_owned()))
            .with_amount_minor(Some(amount_minor))
            .with_currency_code(Some("USD".to_owned()))
            .with_payload_json(Some(format!("{{\"id\":\"{event_id}\"}}"))),
        )
        .await
        .unwrap();
    }

    let stored_payment_order = store
        .find_payment_order_record(&payment_order.payment_order_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        stored_payment_order.payment_status,
        PaymentOrderStatus::Captured
    );
    assert_eq!(stored_payment_order.captured_amount_minor, 4_000);
    assert_eq!(stored_payment_order.fulfillment_status, "fulfilled");

    let transactions = store
        .list_payment_transaction_records_for_order(&payment_order.payment_order_id)
        .await
        .unwrap()
        .into_iter()
        .filter(|transaction| transaction.transaction_kind == PaymentTransactionKind::Sale)
        .collect::<Vec<_>>();
    assert_eq!(transactions.len(), 3);

    let commerce_order = store
        .list_commerce_orders()
        .await
        .unwrap()
        .into_iter()
        .find(|order| order.order_id == payment_order.commerce_order_id)
        .unwrap();
    assert_eq!(commerce_order.status, "fulfilled");

    let account = store
        .find_account_record_by_owner(
            scope.tenant_id,
            scope.organization_id,
            scope.user_id,
            AccountType::Primary,
        )
        .await
        .unwrap()
        .expect("primary account should be created once after aggregate capture reaches threshold");
    let lots = store.list_account_benefit_lots().await.unwrap();
    assert_eq!(lots.len(), 1);
    assert_eq!(lots[0].account_id, account.account_id);
    let ledger_entries = store.list_account_ledger_entry_records().await.unwrap();
    assert_eq!(ledger_entries.len(), 1);
    assert_eq!(ledger_entries[0].account_id, account.account_id);
}

#[tokio::test]
async fn settled_overcapture_caps_local_capture_and_records_reconciliation() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let scope = PaymentSubjectScope::new(16, 0, 49);
    let (_commerce_order, payment_order, payment_attempt, _payment_session) = prepare_checkout(
        &store,
        &scope,
        "commerce-order-callback-overcap-1",
        "project-callback-overcap-1",
        "user-callback-overcap-1",
        1_720_001_180,
    )
    .await;

    let result = ingest_payment_callback(
        &store,
        &PaymentCallbackIntakeRequest::new(
            scope.clone(),
            PaymentProviderCode::Stripe,
            "stripe-main",
            "checkout.session.completed",
            "evt_overcap_1",
            "dedupe_evt_overcap_1",
            1_720_001_240,
        )
        .with_payment_order_id(Some(payment_order.payment_order_id.clone()))
        .with_payment_attempt_id(Some(payment_attempt.payment_attempt_id.clone()))
        .with_provider_transaction_id(Some("pi_overcap_1".to_owned()))
        .with_signature_status("verified")
        .with_provider_status(Some("succeeded".to_owned()))
        .with_amount_minor(Some(4_500))
        .with_currency_code(Some("USD".to_owned()))
        .with_payload_json(Some("{\"id\":\"evt_overcap_1\"}".to_owned())),
    )
    .await
    .unwrap();

    assert_eq!(
        result.disposition,
        PaymentCallbackIntakeDisposition::Processed
    );
    assert_eq!(
        result.normalized_outcome,
        Some(PaymentCallbackNormalizedOutcome::Settled)
    );

    let stored_payment_order = store
        .find_payment_order_record(&payment_order.payment_order_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        stored_payment_order.payment_status,
        PaymentOrderStatus::Captured
    );
    assert_eq!(stored_payment_order.captured_amount_minor, 4_000);
    assert_eq!(stored_payment_order.fulfillment_status, "fulfilled");

    let transactions = store
        .list_payment_transaction_records_for_order(&payment_order.payment_order_id)
        .await
        .unwrap();
    assert_eq!(transactions.len(), 1);
    assert_eq!(
        transactions[0].transaction_kind,
        PaymentTransactionKind::Sale
    );
    assert_eq!(transactions[0].provider_transaction_id, "pi_overcap_1");
    assert_eq!(transactions[0].amount_minor, 4_000);

    let reconciliation = store
        .list_reconciliation_match_summary_records(&format!(
            "payment_conflict_batch_{}",
            payment_order.payment_order_id
        ))
        .await
        .unwrap();
    assert_eq!(reconciliation.len(), 1);
    assert_eq!(reconciliation[0].provider_transaction_id, "pi_overcap_1");
    assert_eq!(reconciliation[0].provider_amount_minor, 4_500);
    assert_eq!(reconciliation[0].local_amount_minor, Some(4_000));
    assert_eq!(
        reconciliation[0].reason_code.as_deref(),
        Some("payment_capture_amount_capped")
    );
    assert_eq!(reconciliation[0].match_status.as_str(), "mismatch_amount");

    let account = store
        .find_account_record_by_owner(
            scope.tenant_id,
            scope.organization_id,
            scope.user_id,
            AccountType::Primary,
        )
        .await
        .unwrap()
        .expect("primary account should be created once for capped overcapture");
    let lots = store.list_account_benefit_lots().await.unwrap();
    assert_eq!(lots.len(), 1);
    assert_eq!(lots[0].account_id, account.account_id);
    let ledger_entries = store.list_account_ledger_entry_records().await.unwrap();
    assert_eq!(ledger_entries.len(), 1);
    assert_eq!(ledger_entries[0].account_id, account.account_id);
}

#[tokio::test]
async fn same_sale_replay_overcapture_caps_aggregate_capture_without_duplicate_fulfillment() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let scope = PaymentSubjectScope::new(17, 0, 50);
    let (_commerce_order, payment_order, payment_attempt, _payment_session) = prepare_checkout(
        &store,
        &scope,
        "commerce-order-callback-overcap-2",
        "project-callback-overcap-2",
        "user-callback-overcap-2",
        1_720_001_300,
    )
    .await;

    for (event_id, dedupe_key, amount_minor, received_at_ms) in [
        (
            "evt_overcap_replay_1",
            "dedupe_evt_overcap_replay_1",
            1_000_u64,
            1_720_001_360_u64,
        ),
        (
            "evt_overcap_replay_2",
            "dedupe_evt_overcap_replay_2",
            5_000_u64,
            1_720_001_420_u64,
        ),
    ] {
        ingest_payment_callback(
            &store,
            &PaymentCallbackIntakeRequest::new(
                scope.clone(),
                PaymentProviderCode::Stripe,
                "stripe-main",
                "checkout.session.completed",
                event_id,
                dedupe_key,
                received_at_ms,
            )
            .with_payment_order_id(Some(payment_order.payment_order_id.clone()))
            .with_payment_attempt_id(Some(payment_attempt.payment_attempt_id.clone()))
            .with_provider_transaction_id(Some("pi_overcap_replay_1".to_owned()))
            .with_signature_status("verified")
            .with_provider_status(Some("succeeded".to_owned()))
            .with_amount_minor(Some(amount_minor))
            .with_currency_code(Some("USD".to_owned()))
            .with_payload_json(Some(format!("{{\"id\":\"{event_id}\"}}"))),
        )
        .await
        .unwrap();
    }

    let stored_payment_order = store
        .find_payment_order_record(&payment_order.payment_order_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        stored_payment_order.payment_status,
        PaymentOrderStatus::Captured
    );
    assert_eq!(stored_payment_order.captured_amount_minor, 4_000);
    assert_eq!(stored_payment_order.fulfillment_status, "fulfilled");

    let transactions = store
        .list_payment_transaction_records_for_order(&payment_order.payment_order_id)
        .await
        .unwrap()
        .into_iter()
        .filter(|transaction| transaction.transaction_kind == PaymentTransactionKind::Sale)
        .collect::<Vec<_>>();
    assert_eq!(transactions.len(), 1);
    assert_eq!(
        transactions[0].provider_transaction_id,
        "pi_overcap_replay_1"
    );
    assert_eq!(transactions[0].amount_minor, 4_000);

    let reconciliation = store
        .list_reconciliation_match_summary_records(&format!(
            "payment_conflict_batch_{}",
            payment_order.payment_order_id
        ))
        .await
        .unwrap();
    assert_eq!(reconciliation.len(), 1);
    assert_eq!(
        reconciliation[0].provider_transaction_id,
        "pi_overcap_replay_1"
    );
    assert_eq!(reconciliation[0].provider_amount_minor, 5_000);
    assert_eq!(reconciliation[0].local_amount_minor, Some(4_000));
    assert_eq!(
        reconciliation[0].reason_code.as_deref(),
        Some("payment_capture_amount_capped")
    );
    assert_eq!(reconciliation[0].match_status.as_str(), "mismatch_amount");

    let account = store
        .find_account_record_by_owner(
            scope.tenant_id,
            scope.organization_id,
            scope.user_id,
            AccountType::Primary,
        )
        .await
        .unwrap()
        .expect("primary account should be created once for capped replay overcapture");
    let lots = store.list_account_benefit_lots().await.unwrap();
    assert_eq!(lots.len(), 1);
    assert_eq!(lots[0].account_id, account.account_id);
    let ledger_entries = store.list_account_ledger_entry_records().await.unwrap();
    assert_eq!(ledger_entries.len(), 1);
    assert_eq!(ledger_entries[0].account_id, account.account_id);
}

#[tokio::test]
async fn settled_payment_replay_keeps_single_sale_transaction_and_records_provider_conflict() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let scope = PaymentSubjectScope::new(10, 0, 43);
    let (_commerce_order, payment_order, payment_attempt, _payment_session) = prepare_checkout(
        &store,
        &scope,
        "commerce-order-callback-replay-1",
        "project-callback-replay-1",
        "user-callback-replay-1",
        1_720_000_180,
    )
    .await;

    let first_request = PaymentCallbackIntakeRequest::new(
        scope.clone(),
        PaymentProviderCode::Stripe,
        "stripe-main",
        "checkout.session.completed",
        "evt_settled_replay_1",
        "dedupe_evt_settled_replay_1",
        1_720_000_300,
    )
    .with_payment_order_id(Some(payment_order.payment_order_id.clone()))
    .with_payment_attempt_id(Some(payment_attempt.payment_attempt_id.clone()))
    .with_provider_transaction_id(Some("pi_replay_original".to_owned()))
    .with_signature_status("verified")
    .with_provider_status(Some("succeeded".to_owned()))
    .with_amount_minor(Some(4_000))
    .with_currency_code(Some("USD".to_owned()))
    .with_payload_json(Some("{\"id\":\"evt_settled_replay_1\"}".to_owned()));

    let second_request = PaymentCallbackIntakeRequest::new(
        scope.clone(),
        PaymentProviderCode::Stripe,
        "stripe-main",
        "checkout.session.completed",
        "evt_settled_replay_2",
        "dedupe_evt_settled_replay_2",
        1_720_000_420,
    )
    .with_payment_order_id(Some(payment_order.payment_order_id.clone()))
    .with_payment_attempt_id(Some(payment_attempt.payment_attempt_id.clone()))
    .with_provider_transaction_id(Some("pi_replay_changed".to_owned()))
    .with_signature_status("verified")
    .with_provider_status(Some("succeeded".to_owned()))
    .with_amount_minor(Some(4_000))
    .with_currency_code(Some("USD".to_owned()))
    .with_payload_json(Some("{\"id\":\"evt_settled_replay_2\"}".to_owned()));

    let first = ingest_payment_callback(&store, &first_request)
        .await
        .unwrap();
    let replay = ingest_payment_callback(&store, &second_request)
        .await
        .unwrap();

    assert_eq!(
        first.disposition,
        PaymentCallbackIntakeDisposition::Processed
    );
    assert_eq!(
        replay.disposition,
        PaymentCallbackIntakeDisposition::Processed
    );
    assert_eq!(
        replay.normalized_outcome,
        Some(PaymentCallbackNormalizedOutcome::Settled)
    );

    let transactions = store
        .list_payment_transaction_records_for_order(&payment_order.payment_order_id)
        .await
        .unwrap();
    assert_eq!(
        transactions
            .iter()
            .filter(|transaction| transaction.transaction_kind == PaymentTransactionKind::Sale)
            .count(),
        1
    );
    let sale_transaction = transactions
        .iter()
        .find(|transaction| transaction.transaction_kind == PaymentTransactionKind::Sale)
        .unwrap();
    assert_eq!(
        sale_transaction.provider_transaction_id,
        "pi_replay_original"
    );
    assert_eq!(
        replay
            .payment_transaction_opt
            .as_ref()
            .unwrap()
            .provider_transaction_id,
        "pi_replay_original"
    );

    let reconciliation = store
        .list_reconciliation_match_summary_records(&format!(
            "payment_conflict_batch_{}",
            payment_order.payment_order_id
        ))
        .await
        .unwrap();
    assert_eq!(reconciliation.len(), 1);
    assert_eq!(
        reconciliation[0].provider_transaction_id,
        "pi_replay_changed"
    );
    assert_eq!(
        reconciliation[0].payment_order_id.as_deref(),
        Some(payment_order.payment_order_id.as_str())
    );
    assert_eq!(
        reconciliation[0].reason_code.as_deref(),
        Some("payment_provider_transaction_conflict")
    );
    assert_eq!(
        reconciliation[0].match_status.as_str(),
        "mismatch_reference"
    );
    assert_eq!(reconciliation[0].local_amount_minor, Some(4_000));
}

#[tokio::test]
async fn unverified_callback_is_persisted_but_ignored_without_payment_mutation() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let scope = PaymentSubjectScope::new(10, 0, 43);
    let (_commerce_order, payment_order, _payment_attempt, _payment_session) = prepare_checkout(
        &store,
        &scope,
        "commerce-order-callback-2",
        "project-callback-2",
        "user-callback-2",
        1_720_000_300,
    )
    .await;

    let result = ingest_payment_callback(
        &store,
        &PaymentCallbackIntakeRequest::new(
            scope.clone(),
            PaymentProviderCode::Stripe,
            "stripe-main",
            "checkout.session.completed",
            "evt_unverified_1",
            "dedupe_evt_unverified_1",
            1_720_000_360,
        )
        .with_payment_order_id(Some(payment_order.payment_order_id.clone()))
        .with_signature_status("rejected")
        .with_provider_transaction_id(Some("pi_rejected_1".to_owned()))
        .with_payload_json(Some("{\"id\":\"evt_unverified_1\"}".to_owned())),
    )
    .await
    .unwrap();

    assert_eq!(
        result.disposition,
        PaymentCallbackIntakeDisposition::Ignored
    );
    assert_eq!(result.normalized_outcome, None);
    assert_eq!(
        result.callback_event.processing_status,
        PaymentCallbackProcessingStatus::Ignored
    );
    assert_eq!(
        store
            .find_payment_order_record(&payment_order.payment_order_id)
            .await
            .unwrap()
            .unwrap()
            .payment_status,
        PaymentOrderStatus::AwaitingCustomer
    );
    assert!(store
        .list_payment_transaction_records_for_order(&payment_order.payment_order_id)
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn ambiguous_callback_stays_pending_for_provider_query_followup() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let scope = PaymentSubjectScope::new(11, 0, 44);
    let (_commerce_order, payment_order, payment_attempt, _payment_session) = prepare_checkout(
        &store,
        &scope,
        "commerce-order-callback-3",
        "project-callback-3",
        "user-callback-3",
        1_720_000_600,
    )
    .await;

    let result = ingest_payment_callback(
        &store,
        &PaymentCallbackIntakeRequest::new(
            scope.clone(),
            PaymentProviderCode::Stripe,
            "stripe-main",
            "payment.status_unknown",
            "evt_pending_1",
            "dedupe_evt_pending_1",
            1_720_000_660,
        )
        .with_payment_order_id(Some(payment_order.payment_order_id.clone()))
        .with_payment_attempt_id(Some(payment_attempt.payment_attempt_id.clone()))
        .with_signature_status("verified")
        .with_provider_transaction_id(Some("pi_pending_1".to_owned()))
        .with_payload_json(Some("{\"id\":\"evt_pending_1\"}".to_owned())),
    )
    .await
    .unwrap();

    assert_eq!(
        result.disposition,
        PaymentCallbackIntakeDisposition::RequiresProviderQuery
    );
    assert_eq!(result.normalized_outcome, None);
    assert_eq!(
        result.callback_event.processing_status,
        PaymentCallbackProcessingStatus::Pending
    );
    assert_eq!(
        store
            .find_payment_order_record(&payment_order.payment_order_id)
            .await
            .unwrap()
            .unwrap()
            .payment_status,
        PaymentOrderStatus::AwaitingCustomer
    );
    assert!(store
        .list_payment_transaction_records_for_order(&payment_order.payment_order_id)
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn failed_settlement_callback_can_be_replayed_after_order_restoration_without_duplicate_side_effects(
) {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let scope = PaymentSubjectScope::new(12, 0, 45);
    let (commerce_order, payment_order, payment_attempt, _payment_session) =
        prepare_checkout_without_persisted_order(
            &store,
            &scope,
            "commerce-order-callback-4",
            "project-callback-4",
            "user-callback-4",
            1_720_000_900,
        )
        .await;

    let first_request = PaymentCallbackIntakeRequest::new(
        scope.clone(),
        PaymentProviderCode::Stripe,
        "stripe-main",
        "checkout.session.completed",
        "evt_retry_1",
        "dedupe_evt_retry_1",
        1_720_000_960,
    )
    .with_payment_order_id(Some(payment_order.payment_order_id.clone()))
    .with_payment_attempt_id(Some(payment_attempt.payment_attempt_id.clone()))
    .with_provider_transaction_id(Some("pi_retry_1".to_owned()))
    .with_signature_status("verified")
    .with_provider_status(Some("succeeded".to_owned()))
    .with_amount_minor(Some(4_000))
    .with_currency_code(Some("USD".to_owned()))
    .with_payload_json(Some("{\"id\":\"evt_retry_1\",\"attempt\":1}".to_owned()));

    let first_error = ingest_payment_callback(&store, &first_request)
        .await
        .expect_err("missing commerce order should fail fulfillment");
    assert!(first_error.to_string().contains("commerce order"));

    let callbacks_after_failure = store.list_payment_callback_event_records().await.unwrap();
    assert_eq!(callbacks_after_failure.len(), 1);
    assert_eq!(
        callbacks_after_failure[0].processing_status,
        PaymentCallbackProcessingStatus::Failed
    );
    assert_eq!(
        store
            .find_payment_order_record(&payment_order.payment_order_id)
            .await
            .unwrap()
            .unwrap()
            .payment_status,
        PaymentOrderStatus::Captured
    );
    assert_eq!(
        store
            .find_payment_order_record(&payment_order.payment_order_id)
            .await
            .unwrap()
            .unwrap()
            .fulfillment_status,
        "captured_pending_fulfillment"
    );
    assert_eq!(
        store
            .list_payment_transaction_records_for_order(&payment_order.payment_order_id)
            .await
            .unwrap()
            .len(),
        1
    );
    assert!(store.list_account_benefit_lots().await.unwrap().is_empty());
    assert!(store
        .list_account_ledger_entry_records()
        .await
        .unwrap()
        .is_empty());
    assert!(store
        .list_account_ledger_allocations()
        .await
        .unwrap()
        .is_empty());

    store.insert_commerce_order(&commerce_order).await.unwrap();

    let replay_request = PaymentCallbackIntakeRequest::new(
        scope.clone(),
        PaymentProviderCode::Stripe,
        "stripe-main",
        "checkout.session.completed",
        "evt_retry_1",
        "dedupe_evt_retry_1",
        1_720_001_020,
    )
    .with_payment_order_id(Some(payment_order.payment_order_id.clone()))
    .with_payment_attempt_id(Some(payment_attempt.payment_attempt_id.clone()))
    .with_provider_transaction_id(Some("pi_retry_1".to_owned()))
    .with_signature_status("verified")
    .with_provider_status(Some("succeeded".to_owned()))
    .with_amount_minor(Some(4_000))
    .with_currency_code(Some("USD".to_owned()))
    .with_payload_json(Some("{\"id\":\"evt_retry_1\",\"attempt\":2}".to_owned()));

    let replay = ingest_payment_callback(&store, &replay_request)
        .await
        .unwrap();
    assert_eq!(
        replay.disposition,
        PaymentCallbackIntakeDisposition::Processed
    );
    assert_eq!(
        replay.normalized_outcome,
        Some(PaymentCallbackNormalizedOutcome::Settled)
    );
    assert_eq!(
        replay.callback_event.processing_status,
        PaymentCallbackProcessingStatus::Processed
    );
    assert_eq!(
        replay
            .payment_order_opt
            .as_ref()
            .unwrap()
            .fulfillment_status,
        "fulfilled"
    );

    let callbacks = store.list_payment_callback_event_records().await.unwrap();
    assert_eq!(callbacks.len(), 1);
    assert_eq!(
        callbacks[0].processing_status,
        PaymentCallbackProcessingStatus::Processed
    );
    assert_eq!(
        store
            .list_payment_transaction_records_for_order(&payment_order.payment_order_id)
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(store.list_account_benefit_lots().await.unwrap().len(), 1);
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
}

#[tokio::test]
async fn recharge_refund_success_reverses_quota_and_account_history_once() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let scope = PaymentSubjectScope::new(13, 0, 46);
    let (_commerce_order, payment_order, _payment_attempt) = settle_checkout(
        &store,
        &scope,
        "commerce-order-refund-1",
        "project-refund-1",
        "user-refund-1",
        1_720_001_200,
    )
    .await;

    let refund_order = request_payment_order_refund(
        &store,
        &scope,
        &payment_order.payment_order_id,
        "buyer_request",
        4_000,
        "portal_user",
        "user-refund-1",
        1_720_001_260,
    )
    .await
    .unwrap();

    assert_eq!(
        refund_order.payment_order_id,
        payment_order.payment_order_id
    );
    assert_eq!(refund_order.requested_amount_minor, 4_000);
    assert_eq!(refund_order.approved_amount_minor, Some(4_000));
    assert_eq!(refund_order.refund_status, RefundOrderStatus::Requested);
    assert_eq!(
        store
            .find_payment_order_record(&payment_order.payment_order_id)
            .await
            .unwrap()
            .unwrap()
            .refund_status,
        PaymentRefundStatus::Pending
    );

    let first = finalize_refund_order_success(
        &store,
        &refund_order.refund_order_id,
        "re_refund_1",
        4_000,
        1_720_001_320,
    )
    .await
    .unwrap();
    let replay = finalize_refund_order_success(
        &store,
        &refund_order.refund_order_id,
        "re_refund_1",
        4_000,
        1_720_001_380,
    )
    .await
    .unwrap();

    assert_eq!(first.refund_order_id, refund_order.refund_order_id);
    assert_eq!(replay.refund_order_id, refund_order.refund_order_id);

    let payment_order = store
        .find_payment_order_record(&payment_order.payment_order_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(payment_order.refund_status, PaymentRefundStatus::Refunded);

    let stored_refunds = store
        .list_refund_order_records_for_payment_order(&payment_order.payment_order_id)
        .await
        .unwrap();
    assert_eq!(stored_refunds.len(), 1);
    assert_eq!(
        stored_refunds[0].refund_status,
        RefundOrderStatus::Succeeded
    );
    assert_eq!(stored_refunds[0].refunded_amount_minor, 4_000);

    let quota_policies = store
        .list_quota_policies_for_project("project-refund-1")
        .await
        .unwrap();
    assert_eq!(quota_policies.len(), 1);
    assert_eq!(quota_policies[0].max_units, 0);

    let account = store
        .find_account_record_by_owner(
            scope.tenant_id,
            scope.organization_id,
            scope.user_id,
            AccountType::Primary,
        )
        .await
        .unwrap()
        .unwrap();
    let lots = store.list_account_benefit_lots().await.unwrap();
    assert_eq!(lots.len(), 1);
    assert_eq!(lots[0].account_id, account.account_id);
    assert_eq!(lots[0].source_type, AccountBenefitSourceType::Order);
    assert_eq!(lots[0].original_quantity, 100_000.0);
    assert_eq!(lots[0].remaining_quantity, 0.0);

    let ledger_entries = store.list_account_ledger_entry_records().await.unwrap();
    assert_eq!(ledger_entries.len(), 2);
    assert!(ledger_entries.iter().any(|entry| {
        entry.entry_type == AccountLedgerEntryType::GrantIssue && entry.quantity == 100_000.0
    }));
    assert!(ledger_entries.iter().any(|entry| {
        entry.entry_type == AccountLedgerEntryType::Refund && entry.quantity == -100_000.0
    }));

    let ledger_allocations = store.list_account_ledger_allocations().await.unwrap();
    assert_eq!(ledger_allocations.len(), 2);
    assert!(ledger_allocations
        .iter()
        .any(|allocation| allocation.quantity_delta == 100_000.0));
    assert!(ledger_allocations
        .iter()
        .any(|allocation| allocation.quantity_delta == -100_000.0));

    let transactions = store
        .list_payment_transaction_records_for_order(&payment_order.payment_order_id)
        .await
        .unwrap();
    assert_eq!(transactions.len(), 2);
    assert!(transactions
        .iter()
        .any(|transaction| transaction.transaction_kind == PaymentTransactionKind::Sale));
    assert!(transactions.iter().any(|transaction| {
        transaction.transaction_kind == PaymentTransactionKind::Refund
            && transaction.provider_transaction_id == "re_refund_1"
            && transaction.amount_minor == 4_000
    }));

    let journals = store.list_finance_journal_entry_records().await.unwrap();
    assert_eq!(journals.len(), 1);
    assert_eq!(journals[0].entry_code, FinanceEntryCode::RefundPayout);
    assert_eq!(journals[0].source_kind, "refund_order");
    assert_eq!(journals[0].source_id, refund_order.refund_order_id);

    let journal_lines = store
        .list_finance_journal_line_records(&journals[0].finance_journal_entry_id)
        .await
        .unwrap();
    assert_eq!(journal_lines.len(), 2);
    assert_eq!(journal_lines[0].account_code, "customer_prepaid_liability");
    assert_eq!(journal_lines[0].direction, FinanceDirection::Debit);
    assert_eq!(journal_lines[0].amount_minor, 4_000);
    assert_eq!(journal_lines[1].account_code, "payment_refund_clearing");
    assert_eq!(journal_lines[1].direction, FinanceDirection::Credit);
    assert_eq!(journal_lines[1].amount_minor, 4_000);
}

#[tokio::test]
async fn partial_recharge_refund_marks_payment_order_partially_refunded() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let scope = PaymentSubjectScope::new(14, 0, 47);
    let (_commerce_order, payment_order, _payment_attempt) = settle_checkout(
        &store,
        &scope,
        "commerce-order-refund-2",
        "project-refund-2",
        "user-refund-2",
        1_720_001_500,
    )
    .await;

    let refund_order = request_payment_order_refund(
        &store,
        &scope,
        &payment_order.payment_order_id,
        "buyer_request",
        1_000,
        "portal_user",
        "user-refund-2",
        1_720_001_560,
    )
    .await
    .unwrap();

    finalize_refund_order_success(
        &store,
        &refund_order.refund_order_id,
        "re_refund_partial_1",
        1_000,
        1_720_001_620,
    )
    .await
    .unwrap();

    let payment_order = store
        .find_payment_order_record(&payment_order.payment_order_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        payment_order.refund_status,
        PaymentRefundStatus::PartiallyRefunded
    );

    let refunds = store
        .list_refund_order_records_for_payment_order(&payment_order.payment_order_id)
        .await
        .unwrap();
    assert_eq!(refunds.len(), 1);
    assert_eq!(refunds[0].refund_status, RefundOrderStatus::Succeeded);
    assert_eq!(refunds[0].refunded_amount_minor, 1_000);

    let quota_policies = store
        .list_quota_policies_for_project("project-refund-2")
        .await
        .unwrap();
    assert_eq!(quota_policies.len(), 1);
    assert_eq!(quota_policies[0].max_units, 75_000);

    let lots = store.list_account_benefit_lots().await.unwrap();
    assert_eq!(lots.len(), 1);
    assert_eq!(lots[0].remaining_quantity, 75_000.0);

    let ledger_entries = store.list_account_ledger_entry_records().await.unwrap();
    assert_eq!(ledger_entries.len(), 2);
    assert!(ledger_entries.iter().any(|entry| {
        entry.entry_type == AccountLedgerEntryType::Refund && entry.quantity == -25_000.0
    }));
}

#[tokio::test]
async fn repeated_identical_partial_refund_request_reuses_pending_refund_order() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let scope = PaymentSubjectScope::new(15, 0, 48);
    let (_commerce_order, payment_order, _payment_attempt) = settle_checkout(
        &store,
        &scope,
        "commerce-order-refund-request-reuse-1",
        "project-refund-request-reuse-1",
        "user-refund-request-reuse-1",
        1_720_001_700,
    )
    .await;

    let first = request_payment_order_refund(
        &store,
        &scope,
        &payment_order.payment_order_id,
        "buyer_request",
        1_000,
        "portal_user",
        "user-refund-request-reuse-1",
        1_720_001_760,
    )
    .await
    .unwrap();
    let replay = request_payment_order_refund(
        &store,
        &scope,
        &payment_order.payment_order_id,
        "buyer_request",
        1_000,
        "portal_user",
        "user-refund-request-reuse-1",
        1_720_001_820,
    )
    .await
    .unwrap();

    assert_eq!(replay.refund_order_id, first.refund_order_id);
    assert_eq!(replay.requested_amount_minor, 1_000);
    assert_eq!(replay.refund_status, RefundOrderStatus::Requested);

    let refunds = store
        .list_refund_order_records_for_payment_order(&payment_order.payment_order_id)
        .await
        .unwrap();
    assert_eq!(refunds.len(), 1);
    assert_eq!(refunds[0].refund_order_id, first.refund_order_id);

    let payment_order = store
        .find_payment_order_record(&payment_order.payment_order_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(payment_order.refund_status, PaymentRefundStatus::Pending);
}

#[tokio::test]
async fn refund_processing_replay_keeps_single_refund_transaction() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let scope = PaymentSubjectScope::new(15, 0, 48);
    let (_commerce_order, payment_order, _payment_attempt) = settle_checkout(
        &store,
        &scope,
        "commerce-order-refund-replay-1",
        "project-refund-replay-1",
        "user-refund-replay-1",
        1_720_001_800,
    )
    .await;

    let refund_order = request_payment_order_refund(
        &store,
        &scope,
        &payment_order.payment_order_id,
        "buyer_request",
        4_000,
        "portal_user",
        "user-refund-replay-1",
        1_720_001_860,
    )
    .await
    .unwrap();

    finalize_refund_order_success(
        &store,
        &refund_order.refund_order_id,
        "re_refund_replay_original",
        4_000,
        1_720_001_920,
    )
    .await
    .unwrap();

    let mut rewound_refund = store
        .find_refund_order_record(&refund_order.refund_order_id)
        .await
        .unwrap()
        .unwrap();
    rewound_refund.refund_status = RefundOrderStatus::Processing;
    rewound_refund.updated_at_ms = 1_720_001_980;
    store
        .insert_refund_order_record(&rewound_refund)
        .await
        .unwrap();

    finalize_refund_order_success(
        &store,
        &refund_order.refund_order_id,
        "re_refund_replay_changed",
        4_000,
        1_720_002_040,
    )
    .await
    .unwrap();

    let payment_order = store
        .find_payment_order_record(&payment_order.payment_order_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(payment_order.refund_status, PaymentRefundStatus::Refunded);

    let refunds = store
        .list_refund_order_records_for_payment_order(&payment_order.payment_order_id)
        .await
        .unwrap();
    assert_eq!(refunds.len(), 1);
    assert_eq!(refunds[0].refund_status, RefundOrderStatus::Succeeded);
    assert_eq!(refunds[0].refunded_amount_minor, 4_000);

    let transactions = store
        .list_payment_transaction_records_for_order(&payment_order.payment_order_id)
        .await
        .unwrap();
    assert_eq!(
        transactions
            .iter()
            .filter(|transaction| transaction.transaction_kind == PaymentTransactionKind::Refund)
            .count(),
        1
    );
    let refund_transaction = transactions
        .iter()
        .find(|transaction| transaction.transaction_kind == PaymentTransactionKind::Refund)
        .unwrap();
    assert_eq!(
        refund_transaction.provider_transaction_id,
        "re_refund_replay_original"
    );
    assert_eq!(refund_transaction.amount_minor, 4_000);

    let journals = store.list_finance_journal_entry_records().await.unwrap();
    assert_eq!(journals.len(), 1);
    let journal_lines = store
        .list_finance_journal_line_records(&journals[0].finance_journal_entry_id)
        .await
        .unwrap();
    assert_eq!(journal_lines.len(), 2);

    let reconciliation = store
        .list_reconciliation_match_summary_records(&format!(
            "refund_conflict_batch_{}",
            refund_order.refund_order_id
        ))
        .await
        .unwrap();
    assert_eq!(reconciliation.len(), 1);
    assert_eq!(
        reconciliation[0].provider_transaction_id,
        "re_refund_replay_changed"
    );
    assert_eq!(
        reconciliation[0].payment_order_id.as_deref(),
        Some(payment_order.payment_order_id.as_str())
    );
    assert_eq!(
        reconciliation[0].refund_order_id.as_deref(),
        Some(refund_order.refund_order_id.as_str())
    );
    assert_eq!(
        reconciliation[0].reason_code.as_deref(),
        Some("refund_provider_transaction_conflict")
    );
    assert_eq!(
        reconciliation[0].match_status.as_str(),
        "mismatch_reference"
    );
    assert_eq!(reconciliation[0].local_amount_minor, Some(4_000));
}

#[tokio::test]
async fn portal_refund_request_requires_execution_start_after_approval_before_success_finalization()
{
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let scope = PaymentSubjectScope::new(16, 0, 49);
    let (commerce_order, _payment_order, _payment_attempt) = settle_checkout(
        &store,
        &scope,
        "commerce-order-refund-approval-1",
        "project-refund-approval-1",
        "user-refund-approval-1",
        1_720_002_140,
    )
    .await;

    let refund_order = request_portal_commerce_order_refund(
        &store,
        &commerce_order.user_id,
        &commerce_order.project_id,
        &commerce_order.order_id,
        "customer_request",
        4_000,
        1_720_002_200,
    )
    .await
    .unwrap();
    assert_eq!(
        refund_order.refund_status,
        RefundOrderStatus::AwaitingApproval
    );
    assert_eq!(refund_order.approved_amount_minor, None);

    let approved_refund = approve_refund_order_request(
        &store,
        &refund_order.refund_order_id,
        Some(4_000),
        1_720_002_230,
    )
    .await
    .unwrap();
    assert_eq!(approved_refund.refund_status, RefundOrderStatus::Approved);

    let error = finalize_refund_order_success(
        &store,
        &refund_order.refund_order_id,
        "re_portal_refund_needs_approval",
        4_000,
        1_720_002_260,
    )
    .await
    .unwrap_err();
    assert!(error.to_string().contains("cannot be finalized as success"));
}

#[tokio::test]
async fn subscription_refund_request_is_rejected() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let scope = PaymentSubjectScope::new(15, 0, 48);
    let order = CommerceOrderRecord::new(
        "commerce-order-refund-subscription-1",
        "project-refund-subscription-1",
        "user-refund-subscription-1",
        "subscription_plan",
        "growth",
        "Growth",
        9_900,
        9_900,
        "$99.00",
        "$99.00",
        300_000,
        0,
        "pending_payment",
        "workspace_seed",
        1_720_001_800,
    );
    store.insert_commerce_order(&order).await.unwrap();
    let checkout = ensure_commerce_payment_checkout(&store, &scope, &order, "portal_web")
        .await
        .unwrap();
    let payment_order = checkout.payment_order_opt.unwrap();
    let payment_attempt = checkout.payment_attempt_opt.unwrap();

    ingest_payment_callback(
        &store,
        &PaymentCallbackIntakeRequest::new(
            scope.clone(),
            PaymentProviderCode::Stripe,
            "stripe-main",
            "checkout.session.completed",
            "evt_settled_subscription_refund_1",
            "dedupe_evt_settled_subscription_refund_1",
            1_720_001_920,
        )
        .with_payment_order_id(Some(payment_order.payment_order_id.clone()))
        .with_payment_attempt_id(Some(payment_attempt.payment_attempt_id.clone()))
        .with_provider_transaction_id(Some("pi_subscription_refund_1".to_owned()))
        .with_signature_status("verified")
        .with_provider_status(Some("succeeded".to_owned()))
        .with_amount_minor(Some(9_900))
        .with_currency_code(Some("USD".to_owned()))
        .with_payload_json(Some(
            "{\"id\":\"evt_settled_subscription_refund_1\"}".to_owned(),
        )),
    )
    .await
    .unwrap();

    let error = request_payment_order_refund(
        &store,
        &scope,
        &payment_order.payment_order_id,
        "buyer_request",
        9_900,
        "portal_user",
        "user-refund-subscription-1",
        1_720_001_980,
    )
    .await
    .expect_err("subscription refunds should remain unsupported in this tranche");

    assert!(error.to_string().contains("subscription"));
}

#[tokio::test]
async fn failed_payment_attempt_failsover_to_next_active_route_without_duplicate_retry_artifacts() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let scope = PaymentSubjectScope::new(60, 0, 120);
    let (_commerce_order, payment_order, payment_attempt, payment_session) = prepare_checkout(
        &store,
        &scope,
        "commerce-order-failover-failed-1",
        "project-failover-failed-1",
        "user-failover-failed-1",
        1_720_100_100,
    )
    .await;

    seed_failover_route(
        &store,
        &scope,
        PaymentProviderCode::Stripe,
        "stripe-main",
        100,
        "hosted_checkout",
        "portal_web",
    )
    .await;
    seed_failover_route(
        &store,
        &scope,
        PaymentProviderCode::Stripe,
        "stripe-backup",
        90,
        "hosted_checkout",
        "portal_web",
    )
    .await;

    let request = PaymentCallbackIntakeRequest::new(
        scope.clone(),
        PaymentProviderCode::Stripe,
        "stripe-main",
        "payment_intent.payment_failed",
        "evt_failover_failed_1",
        "dedupe_failover_failed_1",
        1_720_100_220,
    )
    .with_payment_order_id(Some(payment_order.payment_order_id.clone()))
    .with_payment_attempt_id(Some(payment_attempt.payment_attempt_id.clone()))
    .with_provider_transaction_id(Some("pi_failover_failed_1".to_owned()))
    .with_signature_status("verified")
    .with_provider_status(Some("failed".to_owned()))
    .with_amount_minor(Some(4_000))
    .with_currency_code(Some("USD".to_owned()))
    .with_payload_json(Some(
        "{\"id\":\"evt_failover_failed_1\",\"status\":\"failed\"}".to_owned(),
    ));

    let first = ingest_payment_callback(&store, &request).await.unwrap();
    let duplicate = ingest_payment_callback(&store, &request).await.unwrap();

    assert_eq!(
        first.disposition,
        PaymentCallbackIntakeDisposition::Processed
    );
    assert_eq!(
        first.normalized_outcome,
        Some(PaymentCallbackNormalizedOutcome::Failed)
    );
    assert_eq!(
        first.payment_order_opt.as_ref().unwrap().payment_status,
        PaymentOrderStatus::AwaitingCustomer
    );
    assert_eq!(
        first.payment_order_opt.as_ref().unwrap().fulfillment_status,
        "pending"
    );
    assert_eq!(
        first.payment_order_opt.as_ref().unwrap().provider_code,
        PaymentProviderCode::Stripe
    );
    assert_eq!(
        first
            .payment_order_opt
            .as_ref()
            .unwrap()
            .method_code
            .as_deref(),
        Some("hosted_checkout")
    );
    assert_eq!(first.payment_attempt_opt.as_ref().unwrap().attempt_no, 2);
    assert_eq!(
        first
            .payment_attempt_opt
            .as_ref()
            .unwrap()
            .gateway_account_id,
        "stripe-backup"
    );
    assert_eq!(
        first.payment_attempt_opt.as_ref().unwrap().attempt_status,
        PaymentAttemptStatus::HandoffReady
    );
    assert_eq!(
        first.payment_session_opt.as_ref().unwrap().session_status,
        PaymentSessionStatus::Open
    );
    assert_eq!(
        duplicate.disposition,
        PaymentCallbackIntakeDisposition::Duplicate
    );

    let attempts = store
        .list_payment_attempt_records_for_order(&payment_order.payment_order_id)
        .await
        .unwrap();
    assert_eq!(attempts.len(), 2);
    assert_eq!(attempts[0].attempt_no, 2);
    assert_eq!(attempts[0].gateway_account_id, "stripe-backup");
    assert_eq!(
        attempts[0].attempt_status,
        PaymentAttemptStatus::HandoffReady
    );
    assert_eq!(attempts[1].attempt_no, 1);
    assert_eq!(attempts[1].gateway_account_id, "stripe-main");
    assert_eq!(attempts[1].attempt_status, PaymentAttemptStatus::Failed);

    let original_sessions = store
        .list_payment_session_records_for_attempt(&payment_attempt.payment_attempt_id)
        .await
        .unwrap();
    assert_eq!(original_sessions.len(), 1);
    assert_eq!(
        original_sessions[0].payment_session_id,
        payment_session.payment_session_id
    );
    assert_eq!(
        original_sessions[0].session_status,
        PaymentSessionStatus::Failed
    );

    let retry_sessions = store
        .list_payment_session_records_for_attempt(&attempts[0].payment_attempt_id)
        .await
        .unwrap();
    assert_eq!(retry_sessions.len(), 1);
    assert_eq!(retry_sessions[0].session_status, PaymentSessionStatus::Open);
}

#[tokio::test]
async fn expired_payment_attempt_failsover_to_next_active_route() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let scope = PaymentSubjectScope::new(61, 0, 121);
    let (_commerce_order, payment_order, payment_attempt, _payment_session) = prepare_checkout(
        &store,
        &scope,
        "commerce-order-failover-expired-1",
        "project-failover-expired-1",
        "user-failover-expired-1",
        1_720_100_300,
    )
    .await;

    seed_failover_route(
        &store,
        &scope,
        PaymentProviderCode::Stripe,
        "stripe-main-expired",
        100,
        "hosted_checkout",
        "portal_web",
    )
    .await;
    seed_failover_route(
        &store,
        &scope,
        PaymentProviderCode::Alipay,
        "alipay-backup-expired",
        90,
        "native_qr",
        "portal_web",
    )
    .await;

    let result = ingest_payment_callback(
        &store,
        &PaymentCallbackIntakeRequest::new(
            scope.clone(),
            PaymentProviderCode::Stripe,
            "stripe-main-expired",
            "checkout.session.expired",
            "evt_failover_expired_1",
            "dedupe_failover_expired_1",
            1_720_100_420,
        )
        .with_payment_order_id(Some(payment_order.payment_order_id.clone()))
        .with_payment_attempt_id(Some(payment_attempt.payment_attempt_id.clone()))
        .with_provider_transaction_id(Some("pi_failover_expired_1".to_owned()))
        .with_signature_status("verified")
        .with_provider_status(Some("expired".to_owned()))
        .with_amount_minor(Some(4_000))
        .with_currency_code(Some("USD".to_owned()))
        .with_payload_json(Some(
            "{\"id\":\"evt_failover_expired_1\",\"status\":\"expired\"}".to_owned(),
        )),
    )
    .await
    .unwrap();

    assert_eq!(
        result.normalized_outcome,
        Some(PaymentCallbackNormalizedOutcome::Expired)
    );
    assert_eq!(
        result.payment_order_opt.as_ref().unwrap().payment_status,
        PaymentOrderStatus::AwaitingCustomer
    );
    assert_eq!(
        result
            .payment_attempt_opt
            .as_ref()
            .unwrap()
            .gateway_account_id,
        "alipay-backup-expired"
    );
    assert_eq!(
        result.payment_attempt_opt.as_ref().unwrap().provider_code,
        PaymentProviderCode::Alipay
    );
    assert_eq!(
        result.payment_attempt_opt.as_ref().unwrap().method_code,
        "native_qr"
    );
}

#[tokio::test]
async fn canceled_payment_attempt_does_not_autofailover_to_next_route() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let scope = PaymentSubjectScope::new(62, 0, 122);
    let (_commerce_order, payment_order, payment_attempt, _payment_session) = prepare_checkout(
        &store,
        &scope,
        "commerce-order-failover-canceled-1",
        "project-failover-canceled-1",
        "user-failover-canceled-1",
        1_720_100_500,
    )
    .await;

    seed_failover_route(
        &store,
        &scope,
        PaymentProviderCode::Stripe,
        "stripe-main-canceled",
        100,
        "hosted_checkout",
        "portal_web",
    )
    .await;
    seed_failover_route(
        &store,
        &scope,
        PaymentProviderCode::Stripe,
        "stripe-backup-canceled",
        90,
        "hosted_checkout",
        "portal_web",
    )
    .await;

    let result = ingest_payment_callback(
        &store,
        &PaymentCallbackIntakeRequest::new(
            scope.clone(),
            PaymentProviderCode::Stripe,
            "stripe-main-canceled",
            "checkout.session.canceled",
            "evt_failover_canceled_1",
            "dedupe_failover_canceled_1",
            1_720_100_620,
        )
        .with_payment_order_id(Some(payment_order.payment_order_id.clone()))
        .with_payment_attempt_id(Some(payment_attempt.payment_attempt_id.clone()))
        .with_provider_transaction_id(Some("pi_failover_canceled_1".to_owned()))
        .with_signature_status("verified")
        .with_provider_status(Some("canceled".to_owned()))
        .with_amount_minor(Some(4_000))
        .with_currency_code(Some("USD".to_owned()))
        .with_payload_json(Some(
            "{\"id\":\"evt_failover_canceled_1\",\"status\":\"canceled\"}".to_owned(),
        )),
    )
    .await
    .unwrap();

    assert_eq!(
        result.normalized_outcome,
        Some(PaymentCallbackNormalizedOutcome::Canceled)
    );
    assert_eq!(
        result.payment_order_opt.as_ref().unwrap().payment_status,
        PaymentOrderStatus::Canceled
    );
    assert_eq!(
        result.payment_attempt_opt.as_ref().unwrap().attempt_status,
        PaymentAttemptStatus::Canceled
    );
    assert_eq!(
        store
            .list_payment_attempt_records_for_order(&payment_order.payment_order_id)
            .await
            .unwrap()
            .len(),
        1
    );
}
