use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use sdkwork_api_app_payment::{ensure_commerce_payment_checkout, PaymentSubjectScope};
use sdkwork_api_domain_commerce::CommerceOrderRecord;
use sdkwork_api_domain_payment::PaymentOrderStatus;
use sdkwork_api_storage_core::{AdminStore, CommercialKernelStore, PaymentKernelStore, Reloadable};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};
use serde_json::Value;
use std::sync::Arc;
use tower::ServiceExt;

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
    sdkwork_api_domain_payment::PaymentOrderRecord,
    sdkwork_api_domain_payment::PaymentAttemptRecord,
) {
    let order = build_pending_commerce_order(order_id, project_id, user_id, created_at_ms);
    store.insert_commerce_order(&order).await.unwrap();
    let checkout = ensure_commerce_payment_checkout(store, scope, &order, "portal_web")
        .await
        .unwrap();
    (
        checkout.payment_order_opt.unwrap(),
        checkout.payment_attempt_opt.unwrap(),
    )
}

async fn prepare_live_checkout(
    tenant_id: u64,
    user_id: u64,
    order_id: &str,
    project_id: &str,
    user_identity: &str,
    created_at_ms: u64,
) -> (
    Arc<dyn AdminStore>,
    Arc<dyn CommercialKernelStore>,
    sdkwork_api_domain_payment::PaymentOrderRecord,
    sdkwork_api_domain_payment::PaymentAttemptRecord,
) {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = Arc::new(SqliteAdminStore::new(pool));
    let scope = PaymentSubjectScope::new(tenant_id, 0, user_id);
    let (payment_order, payment_attempt) = prepare_checkout(
        store.as_ref(),
        &scope,
        order_id,
        project_id,
        user_identity,
        created_at_ms,
    )
    .await;
    let admin_store: Arc<dyn AdminStore> = store.clone();
    let payment_store: Arc<dyn CommercialKernelStore> = store;
    (admin_store, payment_store, payment_order, payment_attempt)
}

async fn read_json(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn payment_callback_route_processes_verified_settlement_into_canonical_payment_state() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool.clone());
    let scope = PaymentSubjectScope::new(31, 0, 77);
    let (payment_order, payment_attempt) = prepare_checkout(
        &store,
        &scope,
        "commerce-order-http-callback-1",
        "project-http-callback-1",
        "user-http-callback-1",
        1_730_000_000,
    )
    .await;
    let app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/_sdkwork/payments/providers/stripe/gateway-accounts/stripe-main/callbacks")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    concat!(
                        "{{",
                        "\"tenant_id\":31,",
                        "\"organization_id\":0,",
                        "\"user_id\":77,",
                        "\"event_type\":\"checkout.session.completed\",",
                        "\"event_id\":\"evt_http_settled_1\",",
                        "\"dedupe_key\":\"dedupe_evt_http_settled_1\",",
                        "\"payment_order_id\":\"{}\",",
                        "\"payment_attempt_id\":\"{}\",",
                        "\"provider_transaction_id\":\"pi_http_settled_1\",",
                        "\"signature_status\":\"verified\",",
                        "\"provider_status\":\"succeeded\",",
                        "\"currency_code\":\"USD\",",
                        "\"amount_minor\":4000,",
                        "\"payload_json\":\"{{\\\"id\\\":\\\"evt_http_settled_1\\\"}}\"",
                        "}}"
                    ),
                    payment_order.payment_order_id, payment_attempt.payment_attempt_id
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["disposition"], "processed");
    assert_eq!(json["normalized_outcome"], "settled");
    assert_eq!(json["payment_order_id"], payment_order.payment_order_id);

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
            .list_payment_callback_event_records()
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        store
            .list_payment_transaction_records_for_order(&payment_order.payment_order_id)
            .await
            .unwrap()
            .len(),
        1
    );
}

#[tokio::test]
async fn payment_callback_route_processes_authorization_without_capture() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool.clone());
    let scope = PaymentSubjectScope::new(34, 0, 80);
    let (payment_order, payment_attempt) = prepare_checkout(
        &store,
        &scope,
        "commerce-order-http-auth-1",
        "project-http-auth-1",
        "user-http-auth-1",
        1_730_000_050,
    )
    .await;
    let app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/_sdkwork/payments/providers/stripe/gateway-accounts/stripe-main/callbacks")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    concat!(
                        "{{",
                        "\"tenant_id\":34,",
                        "\"organization_id\":0,",
                        "\"user_id\":80,",
                        "\"event_type\":\"payment_intent.amount_capturable_updated\",",
                        "\"event_id\":\"evt_http_authorized_1\",",
                        "\"dedupe_key\":\"dedupe_evt_http_authorized_1\",",
                        "\"payment_order_id\":\"{}\",",
                        "\"payment_attempt_id\":\"{}\",",
                        "\"provider_transaction_id\":\"pi_http_authorized_1\",",
                        "\"signature_status\":\"verified\",",
                        "\"provider_status\":\"requires_capture\",",
                        "\"currency_code\":\"USD\",",
                        "\"amount_minor\":4000",
                        "}}"
                    ),
                    payment_order.payment_order_id, payment_attempt.payment_attempt_id
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["disposition"], "processed");
    assert_eq!(json["normalized_outcome"], "authorized");
    assert_eq!(
        store
            .find_payment_order_record(&payment_order.payment_order_id)
            .await
            .unwrap()
            .unwrap()
            .payment_status
            .as_str(),
        "authorized"
    );
}

#[tokio::test]
async fn payment_callback_route_returns_partial_capture_state_for_underpaid_settlement() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool.clone());
    let scope = PaymentSubjectScope::new(35, 0, 81);
    let (payment_order, payment_attempt) = prepare_checkout(
        &store,
        &scope,
        "commerce-order-http-partial-1",
        "project-http-partial-1",
        "user-http-partial-1",
        1_730_000_080,
    )
    .await;
    let app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/_sdkwork/payments/providers/stripe/gateway-accounts/stripe-main/callbacks")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    concat!(
                        "{{",
                        "\"tenant_id\":35,",
                        "\"organization_id\":0,",
                        "\"user_id\":81,",
                        "\"event_type\":\"checkout.session.completed\",",
                        "\"event_id\":\"evt_http_partial_1\",",
                        "\"dedupe_key\":\"dedupe_evt_http_partial_1\",",
                        "\"payment_order_id\":\"{}\",",
                        "\"payment_attempt_id\":\"{}\",",
                        "\"provider_transaction_id\":\"pi_http_partial_1\",",
                        "\"signature_status\":\"verified\",",
                        "\"provider_status\":\"succeeded\",",
                        "\"currency_code\":\"USD\",",
                        "\"amount_minor\":1000,",
                        "\"payload_json\":\"{{\\\"id\\\":\\\"evt_http_partial_1\\\"}}\"",
                        "}}"
                    ),
                    payment_order.payment_order_id, payment_attempt.payment_attempt_id
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["disposition"], "processed");
    assert_eq!(json["normalized_outcome"], "settled");

    let stored_payment_order = store
        .find_payment_order_record(&payment_order.payment_order_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        stored_payment_order.payment_status.as_str(),
        "partially_captured"
    );
    assert_eq!(
        stored_payment_order.fulfillment_status,
        "partial_capture_pending_review"
    );

    let transactions = store
        .list_payment_transaction_records_for_order(&payment_order.payment_order_id)
        .await
        .unwrap();
    assert_eq!(transactions.len(), 1);
    assert_eq!(transactions[0].amount_minor, 1_000);
}

#[tokio::test]
async fn payment_callback_route_accumulates_distinct_partial_captures_for_same_order() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool.clone());
    let scope = PaymentSubjectScope::new(36, 0, 82);
    let (payment_order, payment_attempt) = prepare_checkout(
        &store,
        &scope,
        "commerce-order-http-multi-1",
        "project-http-multi-1",
        "user-http-multi-1",
        1_730_000_120,
    )
    .await;
    let app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    for (event_id, dedupe_key, provider_transaction_id, amount_minor) in [
        (
            "evt_http_multi_1",
            "dedupe_evt_http_multi_1",
            "pi_http_multi_1",
            1_000_u64,
        ),
        (
            "evt_http_multi_2",
            "dedupe_evt_http_multi_2",
            "pi_http_multi_2",
            1_500_u64,
        ),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(
                        "/_sdkwork/payments/providers/stripe/gateway-accounts/stripe-main/callbacks",
                    )
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        concat!(
                            "{{",
                            "\"tenant_id\":36,",
                            "\"organization_id\":0,",
                            "\"user_id\":82,",
                            "\"event_type\":\"checkout.session.completed\",",
                            "\"event_id\":\"{}\",",
                            "\"dedupe_key\":\"{}\",",
                            "\"payment_order_id\":\"{}\",",
                            "\"payment_attempt_id\":\"{}\",",
                            "\"provider_transaction_id\":\"{}\",",
                            "\"signature_status\":\"verified\",",
                            "\"provider_status\":\"succeeded\",",
                            "\"currency_code\":\"USD\",",
                            "\"amount_minor\":{}",
                            "}}"
                        ),
                        event_id,
                        dedupe_key,
                        payment_order.payment_order_id,
                        payment_attempt.payment_attempt_id,
                        provider_transaction_id,
                        amount_minor
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
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

    let transactions = store
        .list_payment_transaction_records_for_order(&payment_order.payment_order_id)
        .await
        .unwrap();
    assert_eq!(transactions.len(), 2);
    assert!(transactions
        .iter()
        .any(|transaction| transaction.provider_transaction_id == "pi_http_multi_1"));
    assert!(transactions
        .iter()
        .any(|transaction| transaction.provider_transaction_id == "pi_http_multi_2"));
}

#[tokio::test]
async fn payment_callback_route_caps_overcapture_to_payable_amount() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool.clone());
    let scope = PaymentSubjectScope::new(37, 0, 83);
    let (payment_order, payment_attempt) = prepare_checkout(
        &store,
        &scope,
        "commerce-order-http-overcap-1",
        "project-http-overcap-1",
        "user-http-overcap-1",
        1_730_000_180,
    )
    .await;
    let app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/_sdkwork/payments/providers/stripe/gateway-accounts/stripe-main/callbacks")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    concat!(
                        "{{",
                        "\"tenant_id\":37,",
                        "\"organization_id\":0,",
                        "\"user_id\":83,",
                        "\"event_type\":\"checkout.session.completed\",",
                        "\"event_id\":\"evt_http_overcap_1\",",
                        "\"dedupe_key\":\"dedupe_evt_http_overcap_1\",",
                        "\"payment_order_id\":\"{}\",",
                        "\"payment_attempt_id\":\"{}\",",
                        "\"provider_transaction_id\":\"pi_http_overcap_1\",",
                        "\"signature_status\":\"verified\",",
                        "\"provider_status\":\"succeeded\",",
                        "\"currency_code\":\"USD\",",
                        "\"amount_minor\":4500",
                        "}}"
                    ),
                    payment_order.payment_order_id, payment_attempt.payment_attempt_id
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["disposition"], "processed");
    assert_eq!(json["normalized_outcome"], "settled");

    let stored_payment_order = store
        .find_payment_order_record(&payment_order.payment_order_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored_payment_order.payment_status.as_str(), "captured");
    assert_eq!(stored_payment_order.captured_amount_minor, 4_000);

    let transactions = store
        .list_payment_transaction_records_for_order(&payment_order.payment_order_id)
        .await
        .unwrap();
    assert_eq!(transactions.len(), 1);
    assert_eq!(transactions[0].amount_minor, 4_000);

    let reconciliation = store
        .list_reconciliation_match_summary_records(&format!(
            "payment_conflict_batch_{}",
            payment_order.payment_order_id
        ))
        .await
        .unwrap();
    assert_eq!(reconciliation.len(), 1);
    assert_eq!(reconciliation[0].provider_amount_minor, 4_500);
    assert_eq!(reconciliation[0].local_amount_minor, Some(4_000));
    assert_eq!(
        reconciliation[0].reason_code.as_deref(),
        Some("payment_capture_amount_capped")
    );
    assert_eq!(reconciliation[0].match_status.as_str(), "mismatch_amount");
}

#[tokio::test]
async fn payment_callback_route_returns_duplicate_for_replayed_dedupe_key() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool.clone());
    let scope = PaymentSubjectScope::new(32, 0, 78);
    let (payment_order, payment_attempt) = prepare_checkout(
        &store,
        &scope,
        "commerce-order-http-callback-2",
        "project-http-callback-2",
        "user-http-callback-2",
        1_730_000_100,
    )
    .await;
    let app = sdkwork_api_interface_http::gateway_router_with_pool(pool);
    let body = format!(
        concat!(
            "{{",
            "\"tenant_id\":32,",
            "\"organization_id\":0,",
            "\"user_id\":78,",
            "\"event_type\":\"checkout.session.completed\",",
            "\"event_id\":\"evt_http_duplicate_1\",",
            "\"dedupe_key\":\"dedupe_evt_http_duplicate_1\",",
            "\"payment_order_id\":\"{}\",",
            "\"payment_attempt_id\":\"{}\",",
            "\"provider_transaction_id\":\"pi_http_duplicate_1\",",
            "\"signature_status\":\"verified\",",
            "\"provider_status\":\"succeeded\"",
            "}}"
        ),
        payment_order.payment_order_id, payment_attempt.payment_attempt_id
    );

    let first = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/_sdkwork/payments/providers/stripe/gateway-accounts/stripe-main/callbacks")
                .header("content-type", "application/json")
                .body(Body::from(body.clone()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::OK);

    let duplicate = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/_sdkwork/payments/providers/stripe/gateway-accounts/stripe-main/callbacks")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(duplicate.status(), StatusCode::OK);
    let json = read_json(duplicate).await;
    assert_eq!(json["disposition"], "duplicate");
    assert_eq!(
        store
            .list_payment_callback_event_records()
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        store
            .list_payment_transaction_records_for_order(&payment_order.payment_order_id)
            .await
            .unwrap()
            .len(),
        1
    );
}

#[tokio::test]
async fn payment_callback_route_reuses_sale_transaction_on_provider_conflict_replay() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool.clone());
    let scope = PaymentSubjectScope::new(33, 0, 79);
    let (payment_order, payment_attempt) = prepare_checkout(
        &store,
        &scope,
        "commerce-order-http-callback-3",
        "project-http-callback-3",
        "user-http-callback-3",
        1_730_000_200,
    )
    .await;
    let app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let first_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/_sdkwork/payments/providers/stripe/gateway-accounts/stripe-main/callbacks")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    concat!(
                        "{{",
                        "\"tenant_id\":33,",
                        "\"organization_id\":0,",
                        "\"user_id\":79,",
                        "\"event_type\":\"checkout.session.completed\",",
                        "\"event_id\":\"evt_http_conflict_1\",",
                        "\"dedupe_key\":\"dedupe_evt_http_conflict_1\",",
                        "\"payment_order_id\":\"{}\",",
                        "\"payment_attempt_id\":\"{}\",",
                        "\"provider_transaction_id\":\"pi_http_conflict_original\",",
                        "\"signature_status\":\"verified\",",
                        "\"provider_status\":\"succeeded\",",
                        "\"currency_code\":\"USD\",",
                        "\"amount_minor\":4000",
                        "}}"
                    ),
                    payment_order.payment_order_id, payment_attempt.payment_attempt_id
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(first_response.status(), StatusCode::OK);
    let first_json = read_json(first_response).await;

    let replay_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/_sdkwork/payments/providers/stripe/gateway-accounts/stripe-main/callbacks")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    concat!(
                        "{{",
                        "\"tenant_id\":33,",
                        "\"organization_id\":0,",
                        "\"user_id\":79,",
                        "\"event_type\":\"checkout.session.completed\",",
                        "\"event_id\":\"evt_http_conflict_2\",",
                        "\"dedupe_key\":\"dedupe_evt_http_conflict_2\",",
                        "\"payment_order_id\":\"{}\",",
                        "\"payment_attempt_id\":\"{}\",",
                        "\"provider_transaction_id\":\"pi_http_conflict_changed\",",
                        "\"signature_status\":\"verified\",",
                        "\"provider_status\":\"succeeded\",",
                        "\"currency_code\":\"USD\",",
                        "\"amount_minor\":4000",
                        "}}"
                    ),
                    payment_order.payment_order_id, payment_attempt.payment_attempt_id
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(replay_response.status(), StatusCode::OK);
    let replay_json = read_json(replay_response).await;
    assert_eq!(replay_json["disposition"], "processed");
    assert_eq!(
        replay_json["payment_transaction_id"],
        first_json["payment_transaction_id"]
    );
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
            .list_reconciliation_match_summary_records(&format!(
                "payment_conflict_batch_{}",
                payment_order.payment_order_id
            ))
            .await
            .unwrap()
            .len(),
        1
    );
}

#[tokio::test]
async fn payment_callback_route_rejects_unknown_provider_codes() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/_sdkwork/payments/providers/not_real/gateway-accounts/stripe-main/callbacks")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"tenant_id\":1,\"event_type\":\"checkout.session.completed\",\"event_id\":\"evt_bad_provider\",\"dedupe_key\":\"dedupe_bad_provider\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn payment_callback_route_uses_replaced_live_payment_store_for_new_requests() {
    let (
        initial_admin_store,
        initial_payment_store,
        initial_payment_order,
        initial_payment_attempt,
    ) = prepare_live_checkout(
        41,
        91,
        "commerce-order-http-live-callback-1",
        "project-http-live-callback-1",
        "user-http-live-callback-1",
        1_730_000_200,
    )
    .await;
    let (
        rotated_admin_store,
        rotated_payment_store,
        rotated_payment_order,
        rotated_payment_attempt,
    ) = prepare_live_checkout(
        42,
        92,
        "commerce-order-http-live-callback-2",
        "project-http-live-callback-2",
        "user-http-live-callback-2",
        1_730_000_300,
    )
    .await;
    let live_store = Reloadable::new(initial_admin_store.clone());
    let live_payment_store = Reloadable::new(initial_payment_store.clone());
    let app = sdkwork_api_interface_http::gateway_router_with_state(
        sdkwork_api_interface_http::GatewayApiState::with_live_store_payment_store_and_secret_manager_handle(
            live_store.clone(),
            live_payment_store.clone(),
            Reloadable::new(
                sdkwork_api_app_credential::CredentialSecretManager::database_encrypted(
                    "local-dev-master-key",
                ),
            ),
        ),
    );

    let initial_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/_sdkwork/payments/providers/stripe/gateway-accounts/stripe-main/callbacks")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    concat!(
                        "{{",
                        "\"tenant_id\":41,",
                        "\"organization_id\":0,",
                        "\"user_id\":91,",
                        "\"event_type\":\"checkout.session.completed\",",
                        "\"event_id\":\"evt_http_live_store_1\",",
                        "\"dedupe_key\":\"dedupe_evt_http_live_store_1\",",
                        "\"payment_order_id\":\"{}\",",
                        "\"payment_attempt_id\":\"{}\",",
                        "\"provider_transaction_id\":\"pi_http_live_store_1\",",
                        "\"signature_status\":\"verified\",",
                        "\"provider_status\":\"succeeded\"",
                        "}}"
                    ),
                    initial_payment_order.payment_order_id,
                    initial_payment_attempt.payment_attempt_id
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(initial_response.status(), StatusCode::OK);
    assert_eq!(
        initial_payment_store
            .find_payment_order_record(&initial_payment_order.payment_order_id)
            .await
            .unwrap()
            .unwrap()
            .payment_status,
        PaymentOrderStatus::Captured
    );

    live_store.replace(rotated_admin_store.clone());
    live_payment_store.replace(rotated_payment_store.clone());

    let rotated_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/_sdkwork/payments/providers/stripe/gateway-accounts/stripe-main/callbacks")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    concat!(
                        "{{",
                        "\"tenant_id\":42,",
                        "\"organization_id\":0,",
                        "\"user_id\":92,",
                        "\"event_type\":\"checkout.session.completed\",",
                        "\"event_id\":\"evt_http_live_store_2\",",
                        "\"dedupe_key\":\"dedupe_evt_http_live_store_2\",",
                        "\"payment_order_id\":\"{}\",",
                        "\"payment_attempt_id\":\"{}\",",
                        "\"provider_transaction_id\":\"pi_http_live_store_2\",",
                        "\"signature_status\":\"verified\",",
                        "\"provider_status\":\"succeeded\"",
                        "}}"
                    ),
                    rotated_payment_order.payment_order_id,
                    rotated_payment_attempt.payment_attempt_id
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(rotated_response.status(), StatusCode::OK);
    assert_eq!(
        rotated_payment_store
            .find_payment_order_record(&rotated_payment_order.payment_order_id)
            .await
            .unwrap()
            .unwrap()
            .payment_status,
        PaymentOrderStatus::Captured
    );
}
