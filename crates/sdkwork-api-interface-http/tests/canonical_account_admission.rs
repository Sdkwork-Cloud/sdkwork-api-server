use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router};
use sdkwork_api_app_identity::persist_gateway_api_key_with_metadata;
use sdkwork_api_domain_billing::{
    AccountBenefitLotRecord, AccountBenefitSourceType, AccountBenefitType, AccountHoldStatus,
    AccountLedgerEntryType, AccountRecord, AccountType, RequestSettlementStatus,
};
use sdkwork_api_domain_identity::{CanonicalApiKeyRecord, IdentityUserRecord};
use sdkwork_api_domain_usage::{RequestStatus, UsageCaptureStatus};
use sdkwork_api_storage_core::{AccountKernelStore, IdentityKernelStore};
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use serde_json::Value;
use sqlx::SqlitePool;
use tower::ServiceExt;

mod support;

async fn read_json(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn memory_pool() -> SqlitePool {
    sdkwork_api_storage_sqlite::run_migrations("sqlite::memory:")
        .await
        .unwrap()
}

#[tokio::test]
async fn stateful_chat_route_captures_canonical_account_hold_from_actual_usage_and_price() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new().route("/v1/chat/completions", post(upstream_chat_with_usage));
    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let seeded = seed_dual_scoped_gateway_account(&pool, 7001, 8001, 1.0).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    configure_openai_provider_with_price(
        admin_app,
        &admin_token,
        &format!("http://{address}"),
        0.01,
        2.5,
        10.0,
    )
    .await;

    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let store = SqliteAdminStore::new(pool);
    let expected_charge = 0.01 + (120.0 * 2.5 / 1_000_000.0) + (80.0 * 10.0 / 1_000_000.0);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {}", seeded.plaintext))
                .header("x-request-id", "req-canonical-capture")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"hi\"}],\"max_completion_tokens\":100}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let holds = store.list_account_holds().await.unwrap();
    assert_eq!(holds.len(), 1);
    assert_eq!(holds[0].status, AccountHoldStatus::PartiallyReleased);
    assert!(holds[0].estimated_quantity > expected_charge);
    assert_approx_eq(holds[0].captured_quantity, expected_charge);
    assert_approx_eq(
        holds[0].released_quantity,
        holds[0].estimated_quantity - expected_charge,
    );

    let settlements = store.list_request_settlement_records().await.unwrap();
    assert_eq!(settlements.len(), 1);
    assert_eq!(
        settlements[0].status,
        RequestSettlementStatus::PartiallyReleased
    );
    assert_approx_eq(settlements[0].captured_credit_amount, expected_charge);
    assert_approx_eq(
        settlements[0].released_credit_amount,
        holds[0].estimated_quantity - expected_charge,
    );
    assert_approx_eq(settlements[0].retail_charge_amount, expected_charge);

    let request_facts = store.list_request_meter_facts().await.unwrap();
    assert_eq!(request_facts.len(), 1);
    assert_eq!(request_facts[0].request_status, RequestStatus::Succeeded);
    assert_eq!(
        request_facts[0].usage_capture_status,
        UsageCaptureStatus::Captured
    );
    assert_eq!(request_facts[0].api_key_id, Some(778899));
    assert_eq!(
        request_facts[0].api_key_hash.as_deref(),
        Some(seeded.hashed.as_str())
    );
    assert_eq!(
        request_facts[0].gateway_request_ref.as_deref(),
        Some("req-canonical-capture")
    );
    assert_approx_eq(
        request_facts[0].actual_credit_charge.unwrap_or_default(),
        expected_charge,
    );
    assert_approx_eq(
        request_facts[0].actual_provider_cost.unwrap_or_default(),
        0.0,
    );

    let lots = store.list_account_benefit_lots().await.unwrap();
    assert_eq!(lots.len(), 1);
    assert_approx_eq(lots[0].remaining_quantity, 1.0 - expected_charge);
    assert_approx_eq(lots[0].held_quantity, 0.0);

    let ledger_entries = store.list_account_ledger_entry_records().await.unwrap();
    assert_eq!(ledger_entries.len(), 3);
    assert!(ledger_entries
        .iter()
        .any(|entry| entry.entry_type == AccountLedgerEntryType::SettlementCapture));
    assert!(ledger_entries
        .iter()
        .any(|entry| entry.entry_type == AccountLedgerEntryType::HoldRelease));
}

#[tokio::test]
async fn stateful_chat_route_rejects_canonical_account_when_balance_is_insufficient() {
    let pool = memory_pool().await;
    let seeded = seed_dual_scoped_gateway_account(&pool, 7002, 8002, 0.005).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    configure_openai_provider_with_price(
        admin_app,
        &admin_token,
        "http://127.0.0.1:1",
        0.01,
        2.5,
        10.0,
    )
    .await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let store = SqliteAdminStore::new(pool);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {}", seeded.plaintext))
                .header("x-request-id", "req-canonical-insufficient")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"hi\"}],\"max_completion_tokens\":100}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    let json = read_json(response).await;
    assert_eq!(json["error"]["type"], "insufficient_quota");
    assert_eq!(json["error"]["code"], "account_balance_insufficient");

    assert!(store.list_account_holds().await.unwrap().is_empty());
    assert!(store.list_request_meter_facts().await.unwrap().is_empty());
    assert!(store
        .list_request_settlement_records()
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn stateful_chat_route_releases_canonical_hold_when_upstream_relay_fails() {
    let pool = memory_pool().await;
    let seeded = seed_dual_scoped_gateway_account(&pool, 7003, 8003, 1.0).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    configure_broken_openai_provider(admin_app, &admin_token).await;

    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let store = SqliteAdminStore::new(pool);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {}", seeded.plaintext))
                .header("x-request-id", "req-canonical-release")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"relay me\"}],\"max_completion_tokens\":100}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);

    let holds = store.list_account_holds().await.unwrap();
    assert_eq!(holds.len(), 1);
    assert_eq!(holds[0].status, AccountHoldStatus::Released);
    assert_approx_eq(holds[0].captured_quantity, 0.0);
    assert_approx_eq(holds[0].released_quantity, holds[0].estimated_quantity);

    let settlements = store.list_request_settlement_records().await.unwrap();
    assert_eq!(settlements.len(), 1);
    assert_eq!(settlements[0].status, RequestSettlementStatus::Released);

    let request_facts = store.list_request_meter_facts().await.unwrap();
    assert_eq!(request_facts.len(), 1);
    assert_eq!(request_facts[0].request_status, RequestStatus::Failed);
    assert_eq!(
        request_facts[0].usage_capture_status,
        UsageCaptureStatus::Failed
    );

    let lots = store.list_account_benefit_lots().await.unwrap();
    assert_eq!(lots.len(), 1);
    assert_approx_eq(lots[0].remaining_quantity, 1.0);
    assert_approx_eq(lots[0].held_quantity, 0.0);

    let ledger_entries = store.list_account_ledger_entry_records().await.unwrap();
    assert_eq!(ledger_entries.len(), 2);
    assert!(ledger_entries
        .iter()
        .any(|entry| entry.entry_type == AccountLedgerEntryType::HoldRelease));
}

#[tokio::test]
async fn stateful_chat_stream_route_captures_canonical_account_hold_from_stream_usage() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new().route(
        "/v1/chat/completions",
        post(upstream_chat_stream_with_usage),
    );
    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let seeded = seed_dual_scoped_gateway_account(&pool, 7007, 8007, 1.0).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    configure_openai_provider_with_price(
        admin_app,
        &admin_token,
        &format!("http://{address}"),
        0.01,
        2.5,
        10.0,
    )
    .await;

    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let store = SqliteAdminStore::new(pool);
    let expected_charge = 0.01 + (120.0 * 2.5 / 1_000_000.0) + (80.0 * 10.0 / 1_000_000.0);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {}", seeded.plaintext))
                .header("x-request-id", "req-canonical-chat-stream-capture")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"hi\"}],\"max_completion_tokens\":100,\"stream\":true}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = String::from_utf8(body.to_vec()).unwrap();
    assert!(body.contains("chatcmpl_stream_upstream"));
    assert!(body.contains("[DONE]"));

    let holds = store.list_account_holds().await.unwrap();
    assert_eq!(holds.len(), 1);
    assert_eq!(holds[0].status, AccountHoldStatus::PartiallyReleased);
    assert!(holds[0].estimated_quantity > expected_charge);
    assert_approx_eq(holds[0].captured_quantity, expected_charge);

    let settlements = store.list_request_settlement_records().await.unwrap();
    assert_eq!(settlements.len(), 1);
    assert_eq!(
        settlements[0].status,
        RequestSettlementStatus::PartiallyReleased
    );
    assert_approx_eq(settlements[0].captured_credit_amount, expected_charge);
    assert_approx_eq(settlements[0].retail_charge_amount, expected_charge);

    let request_facts = store.list_request_meter_facts().await.unwrap();
    assert_eq!(request_facts.len(), 1);
    assert_eq!(request_facts[0].request_status, RequestStatus::Succeeded);
    assert_eq!(
        request_facts[0].usage_capture_status,
        UsageCaptureStatus::Captured
    );
    assert_eq!(
        request_facts[0].gateway_request_ref.as_deref(),
        Some("req-canonical-chat-stream-capture")
    );
    assert_approx_eq(
        request_facts[0].actual_credit_charge.unwrap_or_default(),
        expected_charge,
    );

    let lots = store.list_account_benefit_lots().await.unwrap();
    assert_eq!(lots.len(), 1);
    assert_approx_eq(lots[0].remaining_quantity, 1.0 - expected_charge);
    assert_approx_eq(lots[0].held_quantity, 0.0);
}

#[tokio::test]
async fn stateful_chat_stream_route_releases_canonical_hold_when_stream_finishes_without_done() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new().route(
        "/v1/chat/completions",
        post(upstream_chat_stream_without_done),
    );
    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let seeded = seed_dual_scoped_gateway_account(&pool, 7008, 8008, 1.0).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    configure_openai_provider_with_price(
        admin_app,
        &admin_token,
        &format!("http://{address}"),
        0.01,
        2.5,
        10.0,
    )
    .await;

    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let store = SqliteAdminStore::new(pool);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {}", seeded.plaintext))
                .header("x-request-id", "req-canonical-chat-stream-release")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"relay me\"}],\"max_completion_tokens\":100,\"stream\":true}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = String::from_utf8(body.to_vec()).unwrap();
    assert!(body.contains("chatcmpl_stream_upstream"));
    assert!(!body.contains("[DONE]"));

    let holds = store.list_account_holds().await.unwrap();
    assert_eq!(holds.len(), 1);
    assert_eq!(holds[0].status, AccountHoldStatus::Released);
    assert_approx_eq(holds[0].captured_quantity, 0.0);
    assert_approx_eq(holds[0].released_quantity, holds[0].estimated_quantity);

    let settlements = store.list_request_settlement_records().await.unwrap();
    assert_eq!(settlements.len(), 1);
    assert_eq!(settlements[0].status, RequestSettlementStatus::Released);

    let request_facts = store.list_request_meter_facts().await.unwrap();
    assert_eq!(request_facts.len(), 1);
    assert_eq!(request_facts[0].request_status, RequestStatus::Failed);
    assert_eq!(
        request_facts[0].usage_capture_status,
        UsageCaptureStatus::Failed
    );

    let lots = store.list_account_benefit_lots().await.unwrap();
    assert_eq!(lots.len(), 1);
    assert_approx_eq(lots[0].remaining_quantity, 1.0);
    assert_approx_eq(lots[0].held_quantity, 0.0);
}

#[tokio::test]
async fn stateful_responses_route_captures_canonical_account_hold_from_actual_usage_and_price() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new().route("/v1/responses", post(upstream_responses_with_usage));
    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let seeded = seed_dual_scoped_gateway_account(&pool, 7101, 8101, 1.0).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    configure_openai_provider_with_price(
        admin_app,
        &admin_token,
        &format!("http://{address}"),
        0.02,
        2.5,
        10.0,
    )
    .await;

    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let store = SqliteAdminStore::new(pool);
    let expected_charge = 0.02 + (160.0 * 2.5 / 1_000_000.0) + (40.0 * 10.0 / 1_000_000.0);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("authorization", format!("Bearer {}", seeded.plaintext))
                .header("x-request-id", "req-canonical-responses-capture")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"input\":\"count response tokens\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let holds = store.list_account_holds().await.unwrap();
    assert_eq!(holds.len(), 1);
    assert_eq!(holds[0].status, AccountHoldStatus::PartiallyReleased);
    assert!(holds[0].estimated_quantity > expected_charge);
    assert_approx_eq(holds[0].captured_quantity, expected_charge);

    let settlements = store.list_request_settlement_records().await.unwrap();
    assert_eq!(settlements.len(), 1);
    assert_eq!(
        settlements[0].status,
        RequestSettlementStatus::PartiallyReleased
    );
    assert_approx_eq(settlements[0].captured_credit_amount, expected_charge);
    assert_approx_eq(settlements[0].retail_charge_amount, expected_charge);

    let request_facts = store.list_request_meter_facts().await.unwrap();
    assert_eq!(request_facts.len(), 1);
    assert_eq!(request_facts[0].request_status, RequestStatus::Succeeded);
    assert_eq!(
        request_facts[0].usage_capture_status,
        UsageCaptureStatus::Captured
    );
    assert_eq!(
        request_facts[0].gateway_request_ref.as_deref(),
        Some("req-canonical-responses-capture")
    );
    assert_approx_eq(
        request_facts[0].actual_credit_charge.unwrap_or_default(),
        expected_charge,
    );

    let lots = store.list_account_benefit_lots().await.unwrap();
    assert_eq!(lots.len(), 1);
    assert_approx_eq(lots[0].remaining_quantity, 1.0 - expected_charge);
    assert_approx_eq(lots[0].held_quantity, 0.0);
}

#[tokio::test]
async fn stateful_responses_route_releases_canonical_hold_when_upstream_relay_fails() {
    let pool = memory_pool().await;
    let seeded = seed_dual_scoped_gateway_account(&pool, 7102, 8102, 1.0).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    configure_openai_provider_with_price(
        admin_app,
        &admin_token,
        "http://127.0.0.1:1",
        0.02,
        2.5,
        10.0,
    )
    .await;

    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let store = SqliteAdminStore::new(pool);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("authorization", format!("Bearer {}", seeded.plaintext))
                .header("x-request-id", "req-canonical-responses-release")
                .header("content-type", "application/json")
                .body(Body::from("{\"model\":\"gpt-4.1\",\"input\":\"relay me\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);

    let holds = store.list_account_holds().await.unwrap();
    assert_eq!(holds.len(), 1);
    assert_eq!(holds[0].status, AccountHoldStatus::Released);
    assert_approx_eq(holds[0].captured_quantity, 0.0);
    assert_approx_eq(holds[0].released_quantity, holds[0].estimated_quantity);

    let settlements = store.list_request_settlement_records().await.unwrap();
    assert_eq!(settlements.len(), 1);
    assert_eq!(settlements[0].status, RequestSettlementStatus::Released);

    let request_facts = store.list_request_meter_facts().await.unwrap();
    assert_eq!(request_facts.len(), 1);
    assert_eq!(request_facts[0].request_status, RequestStatus::Failed);
    assert_eq!(
        request_facts[0].usage_capture_status,
        UsageCaptureStatus::Failed
    );

    let lots = store.list_account_benefit_lots().await.unwrap();
    assert_eq!(lots.len(), 1);
    assert_approx_eq(lots[0].remaining_quantity, 1.0);
    assert_approx_eq(lots[0].held_quantity, 0.0);
}

#[tokio::test]
async fn stateful_responses_stream_route_captures_canonical_account_hold_from_completion_usage() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new().route(
        "/v1/responses",
        post(upstream_responses_stream_with_completion_usage),
    );
    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let seeded = seed_dual_scoped_gateway_account(&pool, 7107, 8107, 1.0).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    configure_openai_provider_with_price(
        admin_app,
        &admin_token,
        &format!("http://{address}"),
        0.02,
        2.5,
        10.0,
    )
    .await;

    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let store = SqliteAdminStore::new(pool);
    let expected_charge = 0.02 + (160.0 * 2.5 / 1_000_000.0) + (40.0 * 10.0 / 1_000_000.0);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("authorization", format!("Bearer {}", seeded.plaintext))
                .header("x-request-id", "req-canonical-responses-stream-capture")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"input\":\"count streamed response tokens\",\"stream\":true}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = String::from_utf8(body.to_vec()).unwrap();
    assert!(body.contains("response.completed"));
    assert!(body.contains("[DONE]"));

    let holds = store.list_account_holds().await.unwrap();
    assert_eq!(holds.len(), 1);
    assert_eq!(holds[0].status, AccountHoldStatus::PartiallyReleased);
    assert!(holds[0].estimated_quantity > expected_charge);
    assert_approx_eq(holds[0].captured_quantity, expected_charge);

    let settlements = store.list_request_settlement_records().await.unwrap();
    assert_eq!(settlements.len(), 1);
    assert_eq!(
        settlements[0].status,
        RequestSettlementStatus::PartiallyReleased
    );
    assert_approx_eq(settlements[0].captured_credit_amount, expected_charge);
    assert_approx_eq(settlements[0].retail_charge_amount, expected_charge);

    let request_facts = store.list_request_meter_facts().await.unwrap();
    assert_eq!(request_facts.len(), 1);
    assert_eq!(request_facts[0].request_status, RequestStatus::Succeeded);
    assert_eq!(
        request_facts[0].usage_capture_status,
        UsageCaptureStatus::Captured
    );
    assert_eq!(
        request_facts[0].gateway_request_ref.as_deref(),
        Some("req-canonical-responses-stream-capture")
    );
    assert_approx_eq(
        request_facts[0].actual_credit_charge.unwrap_or_default(),
        expected_charge,
    );

    let lots = store.list_account_benefit_lots().await.unwrap();
    assert_eq!(lots.len(), 1);
    assert_approx_eq(lots[0].remaining_quantity, 1.0 - expected_charge);
    assert_approx_eq(lots[0].held_quantity, 0.0);
}

#[tokio::test]
async fn stateful_responses_stream_route_releases_canonical_hold_when_stream_finishes_without_completion(
) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new().route(
        "/v1/responses",
        post(upstream_responses_stream_without_completion),
    );
    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let seeded = seed_dual_scoped_gateway_account(&pool, 7108, 8108, 1.0).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    configure_openai_provider_with_price(
        admin_app,
        &admin_token,
        &format!("http://{address}"),
        0.02,
        2.5,
        10.0,
    )
    .await;

    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let store = SqliteAdminStore::new(pool);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("authorization", format!("Bearer {}", seeded.plaintext))
                .header("x-request-id", "req-canonical-responses-stream-release")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"input\":\"relay me\",\"stream\":true}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = String::from_utf8(body.to_vec()).unwrap();
    assert!(body.contains("response.output_text.delta"));
    assert!(body.contains("[DONE]"));

    let holds = store.list_account_holds().await.unwrap();
    assert_eq!(holds.len(), 1);
    assert_eq!(holds[0].status, AccountHoldStatus::Released);
    assert_approx_eq(holds[0].captured_quantity, 0.0);
    assert_approx_eq(holds[0].released_quantity, holds[0].estimated_quantity);

    let settlements = store.list_request_settlement_records().await.unwrap();
    assert_eq!(settlements.len(), 1);
    assert_eq!(settlements[0].status, RequestSettlementStatus::Released);

    let request_facts = store.list_request_meter_facts().await.unwrap();
    assert_eq!(request_facts.len(), 1);
    assert_eq!(request_facts[0].request_status, RequestStatus::Failed);
    assert_eq!(
        request_facts[0].usage_capture_status,
        UsageCaptureStatus::Failed
    );

    let lots = store.list_account_benefit_lots().await.unwrap();
    assert_eq!(lots.len(), 1);
    assert_approx_eq(lots[0].remaining_quantity, 1.0);
    assert_approx_eq(lots[0].held_quantity, 0.0);
}

#[tokio::test]
async fn stateful_completions_route_captures_canonical_account_hold_from_actual_usage_and_price() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new().route("/v1/completions", post(upstream_completions_with_usage));
    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let seeded = seed_dual_scoped_gateway_account(&pool, 7103, 8103, 1.0).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    configure_openai_model_provider_with_price(
        admin_app,
        &admin_token,
        &format!("http://{address}"),
        "gpt-3.5-turbo-instruct",
        0.005,
        1.5,
        2.0,
    )
    .await;

    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let store = SqliteAdminStore::new(pool);
    let expected_charge = 0.005 + (90.0 * 1.5 / 1_000_000.0) + (30.0 * 2.0 / 1_000_000.0);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/completions")
                .header("authorization", format!("Bearer {}", seeded.plaintext))
                .header("x-request-id", "req-canonical-completions-capture")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-3.5-turbo-instruct\",\"prompt\":\"complete me\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let holds = store.list_account_holds().await.unwrap();
    assert_eq!(holds.len(), 1);
    assert_eq!(holds[0].status, AccountHoldStatus::PartiallyReleased);
    assert!(holds[0].estimated_quantity > expected_charge);
    assert_approx_eq(holds[0].captured_quantity, expected_charge);

    let settlements = store.list_request_settlement_records().await.unwrap();
    assert_eq!(settlements.len(), 1);
    assert_eq!(
        settlements[0].status,
        RequestSettlementStatus::PartiallyReleased
    );
    assert_approx_eq(settlements[0].captured_credit_amount, expected_charge);
    assert_approx_eq(settlements[0].retail_charge_amount, expected_charge);

    let request_facts = store.list_request_meter_facts().await.unwrap();
    assert_eq!(request_facts.len(), 1);
    assert_eq!(request_facts[0].request_status, RequestStatus::Succeeded);
    assert_eq!(
        request_facts[0].usage_capture_status,
        UsageCaptureStatus::Captured
    );

    let lots = store.list_account_benefit_lots().await.unwrap();
    assert_eq!(lots.len(), 1);
    assert_approx_eq(lots[0].remaining_quantity, 1.0 - expected_charge);
    assert_approx_eq(lots[0].held_quantity, 0.0);
}

#[tokio::test]
async fn stateful_embeddings_route_captures_canonical_account_hold_from_actual_usage_and_price() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new().route("/v1/embeddings", post(upstream_embeddings_with_usage));
    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let seeded = seed_dual_scoped_gateway_account(&pool, 7104, 8104, 1.0).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    configure_openai_model_provider_with_price(
        admin_app,
        &admin_token,
        &format!("http://{address}"),
        "text-embedding-3-large",
        0.0,
        0.13,
        0.0,
    )
    .await;

    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let store = SqliteAdminStore::new(pool);
    let expected_charge = 75.0 * 0.13 / 1_000_000.0;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/embeddings")
                .header("authorization", format!("Bearer {}", seeded.plaintext))
                .header("x-request-id", "req-canonical-embeddings-capture")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"text-embedding-3-large\",\"input\":\"embed me\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let holds = store.list_account_holds().await.unwrap();
    assert_eq!(holds.len(), 1);
    assert_eq!(holds[0].status, AccountHoldStatus::PartiallyReleased);
    assert!(holds[0].estimated_quantity > expected_charge);
    assert_approx_eq(holds[0].captured_quantity, expected_charge);

    let settlements = store.list_request_settlement_records().await.unwrap();
    assert_eq!(settlements.len(), 1);
    assert_eq!(
        settlements[0].status,
        RequestSettlementStatus::PartiallyReleased
    );
    assert_approx_eq(settlements[0].captured_credit_amount, expected_charge);
    assert_approx_eq(settlements[0].retail_charge_amount, expected_charge);

    let request_facts = store.list_request_meter_facts().await.unwrap();
    assert_eq!(request_facts.len(), 1);
    assert_eq!(request_facts[0].request_status, RequestStatus::Succeeded);
    assert_eq!(
        request_facts[0].usage_capture_status,
        UsageCaptureStatus::Captured
    );

    let lots = store.list_account_benefit_lots().await.unwrap();
    assert_eq!(lots.len(), 1);
    assert_approx_eq(lots[0].remaining_quantity, 1.0 - expected_charge);
    assert_approx_eq(lots[0].held_quantity, 0.0);
}

#[tokio::test]
async fn stateful_moderations_route_captures_canonical_account_hold_from_estimated_usage_and_price()
{
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new().route("/v1/moderations", post(upstream_moderations));
    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let seeded = seed_dual_scoped_gateway_account(&pool, 7105, 8105, 1.0).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    configure_openai_model_provider_with_price(
        admin_app,
        &admin_token,
        &format!("http://{address}"),
        "omni-moderation-latest",
        0.02,
        0.0,
        0.0,
    )
    .await;

    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let store = SqliteAdminStore::new(pool);
    let expected_charge = 0.02;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/moderations")
                .header("authorization", format!("Bearer {}", seeded.plaintext))
                .header("x-request-id", "req-canonical-moderations-capture")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"omni-moderation-latest\",\"input\":\"flag this\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let holds = store.list_account_holds().await.unwrap();
    assert_eq!(holds.len(), 1);
    assert_eq!(holds[0].status, AccountHoldStatus::Captured);
    assert_approx_eq(holds[0].estimated_quantity, expected_charge);
    assert_approx_eq(holds[0].captured_quantity, expected_charge);
    assert_approx_eq(holds[0].released_quantity, 0.0);

    let settlements = store.list_request_settlement_records().await.unwrap();
    assert_eq!(settlements.len(), 1);
    assert_eq!(settlements[0].status, RequestSettlementStatus::Captured);
    assert_approx_eq(settlements[0].captured_credit_amount, expected_charge);
    assert_approx_eq(settlements[0].released_credit_amount, 0.0);
    assert_approx_eq(settlements[0].retail_charge_amount, expected_charge);

    let request_facts = store.list_request_meter_facts().await.unwrap();
    assert_eq!(request_facts.len(), 1);
    assert_eq!(request_facts[0].request_status, RequestStatus::Succeeded);
    assert_eq!(
        request_facts[0].usage_capture_status,
        UsageCaptureStatus::Captured
    );
    assert_eq!(
        request_facts[0].gateway_request_ref.as_deref(),
        Some("req-canonical-moderations-capture")
    );
    assert_approx_eq(
        request_facts[0].actual_credit_charge.unwrap_or_default(),
        expected_charge,
    );

    let lots = store.list_account_benefit_lots().await.unwrap();
    assert_eq!(lots.len(), 1);
    assert_approx_eq(lots[0].remaining_quantity, 1.0 - expected_charge);
    assert_approx_eq(lots[0].held_quantity, 0.0);
}

#[tokio::test]
async fn stateful_moderations_route_releases_canonical_hold_when_upstream_relay_fails() {
    let pool = memory_pool().await;
    let seeded = seed_dual_scoped_gateway_account(&pool, 7106, 8106, 1.0).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    configure_openai_model_provider_with_price(
        admin_app,
        &admin_token,
        "http://127.0.0.1:1",
        "omni-moderation-latest",
        0.02,
        0.0,
        0.0,
    )
    .await;

    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let store = SqliteAdminStore::new(pool);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/moderations")
                .header("authorization", format!("Bearer {}", seeded.plaintext))
                .header("x-request-id", "req-canonical-moderations-release")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"omni-moderation-latest\",\"input\":\"flag this\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);

    let holds = store.list_account_holds().await.unwrap();
    assert_eq!(holds.len(), 1);
    assert_eq!(holds[0].status, AccountHoldStatus::Released);
    assert_approx_eq(holds[0].captured_quantity, 0.0);
    assert_approx_eq(holds[0].released_quantity, holds[0].estimated_quantity);

    let settlements = store.list_request_settlement_records().await.unwrap();
    assert_eq!(settlements.len(), 1);
    assert_eq!(settlements[0].status, RequestSettlementStatus::Released);

    let request_facts = store.list_request_meter_facts().await.unwrap();
    assert_eq!(request_facts.len(), 1);
    assert_eq!(request_facts[0].request_status, RequestStatus::Failed);
    assert_eq!(
        request_facts[0].usage_capture_status,
        UsageCaptureStatus::Failed
    );

    let lots = store.list_account_benefit_lots().await.unwrap();
    assert_eq!(lots.len(), 1);
    assert_approx_eq(lots[0].remaining_quantity, 1.0);
    assert_approx_eq(lots[0].held_quantity, 0.0);
}

#[tokio::test]
async fn stateful_anthropic_messages_route_captures_canonical_account_hold_from_actual_usage_and_price(
) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new().route("/v1/chat/completions", post(upstream_chat_with_usage));
    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let seeded = seed_dual_scoped_gateway_account(&pool, 7107, 8107, 1.0).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    configure_openai_provider_with_price(
        admin_app,
        &admin_token,
        &format!("http://{address}"),
        0.01,
        2.5,
        10.0,
    )
    .await;

    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let store = SqliteAdminStore::new(pool);
    let expected_charge = 0.01 + (120.0 * 2.5 / 1_000_000.0) + (80.0 * 10.0 / 1_000_000.0);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("x-api-key", &seeded.plaintext)
                .header("x-request-id", "req-canonical-anthropic-capture")
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "model": "gpt-4.1",
                        "max_tokens": 256,
                        "messages": [
                            {
                                "role": "user",
                                "content": "route by anthropic canonical billing"
                            }
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let holds = store.list_account_holds().await.unwrap();
    assert_eq!(holds.len(), 1);
    assert_eq!(holds[0].status, AccountHoldStatus::PartiallyReleased);
    assert!(holds[0].estimated_quantity > expected_charge);
    assert_approx_eq(holds[0].captured_quantity, expected_charge);
    assert_approx_eq(
        holds[0].released_quantity,
        holds[0].estimated_quantity - expected_charge,
    );

    let settlements = store.list_request_settlement_records().await.unwrap();
    assert_eq!(settlements.len(), 1);
    assert_eq!(
        settlements[0].status,
        RequestSettlementStatus::PartiallyReleased
    );
    assert_approx_eq(settlements[0].captured_credit_amount, expected_charge);
    assert_approx_eq(
        settlements[0].released_credit_amount,
        holds[0].estimated_quantity - expected_charge,
    );
    assert_approx_eq(settlements[0].retail_charge_amount, expected_charge);

    let request_facts = store.list_request_meter_facts().await.unwrap();
    assert_eq!(request_facts.len(), 1);
    assert_eq!(request_facts[0].request_status, RequestStatus::Succeeded);
    assert_eq!(
        request_facts[0].usage_capture_status,
        UsageCaptureStatus::Captured
    );
    assert_eq!(request_facts[0].api_key_id, Some(778899));
    assert_eq!(
        request_facts[0].api_key_hash.as_deref(),
        Some(seeded.hashed.as_str())
    );
    assert_eq!(
        request_facts[0].gateway_request_ref.as_deref(),
        Some("req-canonical-anthropic-capture")
    );
    assert_approx_eq(
        request_facts[0].actual_credit_charge.unwrap_or_default(),
        expected_charge,
    );

    let lots = store.list_account_benefit_lots().await.unwrap();
    assert_eq!(lots.len(), 1);
    assert_approx_eq(lots[0].remaining_quantity, 1.0 - expected_charge);
    assert_approx_eq(lots[0].held_quantity, 0.0);
}

#[tokio::test]
async fn stateful_anthropic_messages_route_releases_canonical_hold_when_upstream_relay_fails() {
    let pool = memory_pool().await;
    let seeded = seed_dual_scoped_gateway_account(&pool, 7108, 8108, 1.0).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    configure_broken_openai_provider(admin_app, &admin_token).await;

    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let store = SqliteAdminStore::new(pool);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("x-api-key", &seeded.plaintext)
                .header("x-request-id", "req-canonical-anthropic-release")
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "model": "gpt-4.1",
                        "max_tokens": 256,
                        "messages": [
                            {
                                "role": "user",
                                "content": "relay anthropic canonical release"
                            }
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);

    let holds = store.list_account_holds().await.unwrap();
    assert_eq!(holds.len(), 1);
    assert_eq!(holds[0].status, AccountHoldStatus::Released);
    assert_approx_eq(holds[0].captured_quantity, 0.0);
    assert_approx_eq(holds[0].released_quantity, holds[0].estimated_quantity);

    let settlements = store.list_request_settlement_records().await.unwrap();
    assert_eq!(settlements.len(), 1);
    assert_eq!(settlements[0].status, RequestSettlementStatus::Released);

    let request_facts = store.list_request_meter_facts().await.unwrap();
    assert_eq!(request_facts.len(), 1);
    assert_eq!(request_facts[0].request_status, RequestStatus::Failed);
    assert_eq!(
        request_facts[0].usage_capture_status,
        UsageCaptureStatus::Failed
    );
    assert_eq!(
        request_facts[0].gateway_request_ref.as_deref(),
        Some("req-canonical-anthropic-release")
    );

    let lots = store.list_account_benefit_lots().await.unwrap();
    assert_eq!(lots.len(), 1);
    assert_approx_eq(lots[0].remaining_quantity, 1.0);
    assert_approx_eq(lots[0].held_quantity, 0.0);
}

#[tokio::test]
async fn stateful_anthropic_messages_stream_route_captures_canonical_account_hold_from_stream_usage(
) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new().route(
        "/v1/chat/completions",
        post(upstream_chat_stream_with_usage),
    );
    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let seeded = seed_dual_scoped_gateway_account(&pool, 7111, 8111, 1.0).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    configure_openai_provider_with_price(
        admin_app,
        &admin_token,
        &format!("http://{address}"),
        0.01,
        2.5,
        10.0,
    )
    .await;

    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let store = SqliteAdminStore::new(pool);
    let expected_charge = 0.01 + (120.0 * 2.5 / 1_000_000.0) + (80.0 * 10.0 / 1_000_000.0);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("x-api-key", &seeded.plaintext)
                .header("x-request-id", "req-canonical-anthropic-stream-capture")
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "model": "gpt-4.1",
                        "max_tokens": 256,
                        "stream": true,
                        "messages": [
                            {
                                "role": "user",
                                "content": "route anthropic stream canonical billing"
                            }
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = String::from_utf8(body.to_vec()).unwrap();
    assert!(body.contains("event: message_start"));
    assert!(body.contains("event: message_stop"));

    let holds = store.list_account_holds().await.unwrap();
    assert_eq!(holds.len(), 1);
    assert_eq!(holds[0].status, AccountHoldStatus::PartiallyReleased);
    assert!(holds[0].estimated_quantity > expected_charge);
    assert_approx_eq(holds[0].captured_quantity, expected_charge);

    let settlements = store.list_request_settlement_records().await.unwrap();
    assert_eq!(settlements.len(), 1);
    assert_eq!(
        settlements[0].status,
        RequestSettlementStatus::PartiallyReleased
    );
    assert_approx_eq(settlements[0].captured_credit_amount, expected_charge);
    assert_approx_eq(settlements[0].retail_charge_amount, expected_charge);

    let request_facts = store.list_request_meter_facts().await.unwrap();
    assert_eq!(request_facts.len(), 1);
    assert_eq!(request_facts[0].request_status, RequestStatus::Succeeded);
    assert_eq!(
        request_facts[0].usage_capture_status,
        UsageCaptureStatus::Captured
    );
    assert_eq!(
        request_facts[0].gateway_request_ref.as_deref(),
        Some("req-canonical-anthropic-stream-capture")
    );
    assert_approx_eq(
        request_facts[0].actual_credit_charge.unwrap_or_default(),
        expected_charge,
    );

    let lots = store.list_account_benefit_lots().await.unwrap();
    assert_eq!(lots.len(), 1);
    assert_approx_eq(lots[0].remaining_quantity, 1.0 - expected_charge);
    assert_approx_eq(lots[0].held_quantity, 0.0);
}

#[tokio::test]
async fn stateful_anthropic_messages_stream_route_releases_canonical_hold_when_stream_finishes_without_done(
) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new().route(
        "/v1/chat/completions",
        post(upstream_chat_stream_without_done),
    );
    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let seeded = seed_dual_scoped_gateway_account(&pool, 7112, 8112, 1.0).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    configure_openai_provider_with_price(
        admin_app,
        &admin_token,
        &format!("http://{address}"),
        0.01,
        2.5,
        10.0,
    )
    .await;

    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let store = SqliteAdminStore::new(pool);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("x-api-key", &seeded.plaintext)
                .header("x-request-id", "req-canonical-anthropic-stream-release")
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "model": "gpt-4.1",
                        "max_tokens": 256,
                        "stream": true,
                        "messages": [
                            {
                                "role": "user",
                                "content": "relay anthropic stream canonical release"
                            }
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = String::from_utf8(body.to_vec()).unwrap();
    assert!(body.contains("event: message_start"));
    assert!(!body.contains("event: message_stop"));

    let holds = store.list_account_holds().await.unwrap();
    assert_eq!(holds.len(), 1);
    assert_eq!(holds[0].status, AccountHoldStatus::Released);
    assert_approx_eq(holds[0].captured_quantity, 0.0);
    assert_approx_eq(holds[0].released_quantity, holds[0].estimated_quantity);

    let settlements = store.list_request_settlement_records().await.unwrap();
    assert_eq!(settlements.len(), 1);
    assert_eq!(settlements[0].status, RequestSettlementStatus::Released);

    let request_facts = store.list_request_meter_facts().await.unwrap();
    assert_eq!(request_facts.len(), 1);
    assert_eq!(request_facts[0].request_status, RequestStatus::Failed);
    assert_eq!(
        request_facts[0].usage_capture_status,
        UsageCaptureStatus::Failed
    );

    let lots = store.list_account_benefit_lots().await.unwrap();
    assert_eq!(lots.len(), 1);
    assert_approx_eq(lots[0].remaining_quantity, 1.0);
    assert_approx_eq(lots[0].held_quantity, 0.0);
}

#[tokio::test]
async fn stateful_gemini_generate_content_route_captures_canonical_account_hold_from_actual_usage_and_price(
) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new().route("/v1/chat/completions", post(upstream_chat_with_usage));
    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let seeded = seed_dual_scoped_gateway_account(&pool, 7109, 8109, 1.0).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    configure_openai_model_provider_with_price(
        admin_app,
        &admin_token,
        &format!("http://{address}"),
        "gemini-2.5-pro",
        0.02,
        3.0,
        12.0,
    )
    .await;

    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let store = SqliteAdminStore::new(pool);
    let expected_charge = 0.02 + (120.0 * 3.0 / 1_000_000.0) + (80.0 * 12.0 / 1_000_000.0);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/v1beta/models/gemini-2.5-pro:generateContent?key={}",
                    seeded.plaintext
                ))
                .header("x-request-id", "req-canonical-gemini-capture")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "contents": [
                            {
                                "role": "user",
                                "parts": [
                                    { "text": "route by gemini canonical billing" }
                                ]
                            }
                        ],
                        "generationConfig": {
                            "maxOutputTokens": 256
                        }
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let holds = store.list_account_holds().await.unwrap();
    assert_eq!(holds.len(), 1);
    assert_eq!(holds[0].status, AccountHoldStatus::PartiallyReleased);
    assert!(holds[0].estimated_quantity > expected_charge);
    assert_approx_eq(holds[0].captured_quantity, expected_charge);
    assert_approx_eq(
        holds[0].released_quantity,
        holds[0].estimated_quantity - expected_charge,
    );

    let settlements = store.list_request_settlement_records().await.unwrap();
    assert_eq!(settlements.len(), 1);
    assert_eq!(
        settlements[0].status,
        RequestSettlementStatus::PartiallyReleased
    );
    assert_approx_eq(settlements[0].captured_credit_amount, expected_charge);
    assert_approx_eq(
        settlements[0].released_credit_amount,
        holds[0].estimated_quantity - expected_charge,
    );
    assert_approx_eq(settlements[0].retail_charge_amount, expected_charge);

    let request_facts = store.list_request_meter_facts().await.unwrap();
    assert_eq!(request_facts.len(), 1);
    assert_eq!(request_facts[0].request_status, RequestStatus::Succeeded);
    assert_eq!(
        request_facts[0].usage_capture_status,
        UsageCaptureStatus::Captured
    );
    assert_eq!(request_facts[0].api_key_id, Some(778899));
    assert_eq!(
        request_facts[0].api_key_hash.as_deref(),
        Some(seeded.hashed.as_str())
    );
    assert_eq!(
        request_facts[0].gateway_request_ref.as_deref(),
        Some("req-canonical-gemini-capture")
    );
    assert_approx_eq(
        request_facts[0].actual_credit_charge.unwrap_or_default(),
        expected_charge,
    );

    let lots = store.list_account_benefit_lots().await.unwrap();
    assert_eq!(lots.len(), 1);
    assert_approx_eq(lots[0].remaining_quantity, 1.0 - expected_charge);
    assert_approx_eq(lots[0].held_quantity, 0.0);
}

#[tokio::test]
async fn stateful_gemini_generate_content_route_releases_canonical_hold_when_upstream_relay_fails()
{
    let pool = memory_pool().await;
    let seeded = seed_dual_scoped_gateway_account(&pool, 7110, 8110, 1.0).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    configure_openai_model_provider_with_price(
        admin_app,
        &admin_token,
        "http://127.0.0.1:1",
        "gemini-2.5-pro",
        0.02,
        3.0,
        12.0,
    )
    .await;

    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let store = SqliteAdminStore::new(pool);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/v1beta/models/gemini-2.5-pro:generateContent?key={}",
                    seeded.plaintext
                ))
                .header("x-request-id", "req-canonical-gemini-release")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "contents": [
                            {
                                "role": "user",
                                "parts": [
                                    { "text": "relay gemini canonical release" }
                                ]
                            }
                        ],
                        "generationConfig": {
                            "maxOutputTokens": 256
                        }
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);

    let holds = store.list_account_holds().await.unwrap();
    assert_eq!(holds.len(), 1);
    assert_eq!(holds[0].status, AccountHoldStatus::Released);
    assert_approx_eq(holds[0].captured_quantity, 0.0);
    assert_approx_eq(holds[0].released_quantity, holds[0].estimated_quantity);

    let settlements = store.list_request_settlement_records().await.unwrap();
    assert_eq!(settlements.len(), 1);
    assert_eq!(settlements[0].status, RequestSettlementStatus::Released);

    let request_facts = store.list_request_meter_facts().await.unwrap();
    assert_eq!(request_facts.len(), 1);
    assert_eq!(request_facts[0].request_status, RequestStatus::Failed);
    assert_eq!(
        request_facts[0].usage_capture_status,
        UsageCaptureStatus::Failed
    );
    assert_eq!(
        request_facts[0].gateway_request_ref.as_deref(),
        Some("req-canonical-gemini-release")
    );

    let lots = store.list_account_benefit_lots().await.unwrap();
    assert_eq!(lots.len(), 1);
    assert_approx_eq(lots[0].remaining_quantity, 1.0);
    assert_approx_eq(lots[0].held_quantity, 0.0);
}

#[tokio::test]
async fn stateful_gemini_stream_generate_content_route_captures_canonical_account_hold_from_stream_usage(
) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new().route(
        "/v1/chat/completions",
        post(upstream_chat_stream_with_usage),
    );
    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let seeded = seed_dual_scoped_gateway_account(&pool, 7113, 8113, 1.0).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    configure_openai_model_provider_with_price(
        admin_app,
        &admin_token,
        &format!("http://{address}"),
        "gemini-2.5-pro",
        0.02,
        3.0,
        12.0,
    )
    .await;

    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let store = SqliteAdminStore::new(pool);
    let expected_charge = 0.02 + (120.0 * 3.0 / 1_000_000.0) + (80.0 * 12.0 / 1_000_000.0);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/v1beta/models/gemini-2.5-pro:streamGenerateContent?alt=sse&key={}",
                    seeded.plaintext
                ))
                .header("x-request-id", "req-canonical-gemini-stream-capture")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "contents": [
                            {
                                "role": "user",
                                "parts": [
                                    { "text": "route gemini stream canonical billing" }
                                ]
                            }
                        ],
                        "generationConfig": {
                            "maxOutputTokens": 256
                        }
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = String::from_utf8(body.to_vec()).unwrap();
    assert!(body.contains("\"candidates\""));

    let holds = store.list_account_holds().await.unwrap();
    assert_eq!(holds.len(), 1);
    assert_eq!(holds[0].status, AccountHoldStatus::PartiallyReleased);
    assert!(holds[0].estimated_quantity > expected_charge);
    assert_approx_eq(holds[0].captured_quantity, expected_charge);

    let settlements = store.list_request_settlement_records().await.unwrap();
    assert_eq!(settlements.len(), 1);
    assert_eq!(
        settlements[0].status,
        RequestSettlementStatus::PartiallyReleased
    );
    assert_approx_eq(settlements[0].captured_credit_amount, expected_charge);
    assert_approx_eq(settlements[0].retail_charge_amount, expected_charge);

    let request_facts = store.list_request_meter_facts().await.unwrap();
    assert_eq!(request_facts.len(), 1);
    assert_eq!(request_facts[0].request_status, RequestStatus::Succeeded);
    assert_eq!(
        request_facts[0].usage_capture_status,
        UsageCaptureStatus::Captured
    );
    assert_eq!(
        request_facts[0].gateway_request_ref.as_deref(),
        Some("req-canonical-gemini-stream-capture")
    );
    assert_approx_eq(
        request_facts[0].actual_credit_charge.unwrap_or_default(),
        expected_charge,
    );

    let lots = store.list_account_benefit_lots().await.unwrap();
    assert_eq!(lots.len(), 1);
    assert_approx_eq(lots[0].remaining_quantity, 1.0 - expected_charge);
    assert_approx_eq(lots[0].held_quantity, 0.0);
}

#[tokio::test]
async fn stateful_gemini_stream_generate_content_route_releases_canonical_hold_when_stream_finishes_without_done(
) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new().route(
        "/v1/chat/completions",
        post(upstream_chat_stream_without_done),
    );
    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let seeded = seed_dual_scoped_gateway_account(&pool, 7114, 8114, 1.0).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    configure_openai_model_provider_with_price(
        admin_app,
        &admin_token,
        &format!("http://{address}"),
        "gemini-2.5-pro",
        0.02,
        3.0,
        12.0,
    )
    .await;

    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let store = SqliteAdminStore::new(pool);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/v1beta/models/gemini-2.5-pro:streamGenerateContent?alt=sse&key={}",
                    seeded.plaintext
                ))
                .header("x-request-id", "req-canonical-gemini-stream-release")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "contents": [
                            {
                                "role": "user",
                                "parts": [
                                    { "text": "relay gemini stream canonical release" }
                                ]
                            }
                        ],
                        "generationConfig": {
                            "maxOutputTokens": 256
                        }
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = String::from_utf8(body.to_vec()).unwrap();
    assert!(body.contains("\"candidates\""));
    assert!(!body.contains("\"finishReason\""));

    let holds = store.list_account_holds().await.unwrap();
    assert_eq!(holds.len(), 1);
    assert_eq!(holds[0].status, AccountHoldStatus::Released);
    assert_approx_eq(holds[0].captured_quantity, 0.0);
    assert_approx_eq(holds[0].released_quantity, holds[0].estimated_quantity);

    let settlements = store.list_request_settlement_records().await.unwrap();
    assert_eq!(settlements.len(), 1);
    assert_eq!(settlements[0].status, RequestSettlementStatus::Released);

    let request_facts = store.list_request_meter_facts().await.unwrap();
    assert_eq!(request_facts.len(), 1);
    assert_eq!(request_facts[0].request_status, RequestStatus::Failed);
    assert_eq!(
        request_facts[0].usage_capture_status,
        UsageCaptureStatus::Failed
    );

    let lots = store.list_account_benefit_lots().await.unwrap();
    assert_eq!(lots.len(), 1);
    assert_approx_eq(lots[0].remaining_quantity, 1.0);
    assert_approx_eq(lots[0].held_quantity, 0.0);
}

#[tokio::test]
async fn stateful_images_route_captures_canonical_account_hold_from_request_price() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new().route("/v1/images/generations", post(upstream_images_success));
    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let seeded = seed_dual_scoped_gateway_account(&pool, 7201, 8201, 1.0).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    configure_openai_provider_with_price(
        admin_app,
        &admin_token,
        &format!("http://{address}"),
        0.05,
        0.0,
        0.0,
    )
    .await;

    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let store = SqliteAdminStore::new(pool);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/images/generations")
                .header("authorization", format!("Bearer {}", seeded.plaintext))
                .header("x-request-id", "req-canonical-images-capture")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"prompt\":\"draw a skyline\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let holds = store.list_account_holds().await.unwrap();
    assert_eq!(holds.len(), 1);
    assert_eq!(holds[0].status, AccountHoldStatus::Captured);
    assert_approx_eq(holds[0].estimated_quantity, 0.05);
    assert_approx_eq(holds[0].captured_quantity, 0.05);
    assert_approx_eq(holds[0].released_quantity, 0.0);

    let settlements = store.list_request_settlement_records().await.unwrap();
    assert_eq!(settlements.len(), 1);
    assert_eq!(settlements[0].status, RequestSettlementStatus::Captured);
    assert_approx_eq(settlements[0].captured_credit_amount, 0.05);
    assert_approx_eq(settlements[0].retail_charge_amount, 0.05);

    let request_facts = store.list_request_meter_facts().await.unwrap();
    assert_eq!(request_facts.len(), 1);
    assert_eq!(request_facts[0].request_status, RequestStatus::Succeeded);
    assert_eq!(
        request_facts[0].usage_capture_status,
        UsageCaptureStatus::Captured
    );
    assert_eq!(
        request_facts[0].gateway_request_ref.as_deref(),
        Some("req-canonical-images-capture")
    );
    assert_approx_eq(
        request_facts[0].actual_credit_charge.unwrap_or_default(),
        0.05,
    );
}

#[tokio::test]
async fn stateful_audio_transcriptions_route_captures_canonical_account_hold_from_request_price() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new().route(
        "/v1/audio/transcriptions",
        post(upstream_transcriptions_success),
    );
    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let seeded = seed_dual_scoped_gateway_account(&pool, 7202, 8202, 1.0).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    configure_openai_provider_with_price(
        admin_app,
        &admin_token,
        &format!("http://{address}"),
        0.025,
        0.0,
        0.0,
    )
    .await;

    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let store = SqliteAdminStore::new(pool);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/audio/transcriptions")
                .header("authorization", format!("Bearer {}", seeded.plaintext))
                .header("x-request-id", "req-canonical-transcriptions-capture")
                .header("content-type", "application/json")
                .body(Body::from("{\"model\":\"gpt-4.1\",\"file_id\":\"file_1\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let holds = store.list_account_holds().await.unwrap();
    assert_eq!(holds.len(), 1);
    assert_eq!(holds[0].status, AccountHoldStatus::Captured);
    assert_approx_eq(holds[0].estimated_quantity, 0.025);
    assert_approx_eq(holds[0].captured_quantity, 0.025);
    assert_approx_eq(holds[0].released_quantity, 0.0);

    let settlements = store.list_request_settlement_records().await.unwrap();
    assert_eq!(settlements.len(), 1);
    assert_eq!(settlements[0].status, RequestSettlementStatus::Captured);
    assert_approx_eq(settlements[0].captured_credit_amount, 0.025);
    assert_approx_eq(settlements[0].retail_charge_amount, 0.025);

    let request_facts = store.list_request_meter_facts().await.unwrap();
    assert_eq!(request_facts.len(), 1);
    assert_eq!(request_facts[0].request_status, RequestStatus::Succeeded);
    assert_eq!(
        request_facts[0].usage_capture_status,
        UsageCaptureStatus::Captured
    );
    assert_eq!(
        request_facts[0].gateway_request_ref.as_deref(),
        Some("req-canonical-transcriptions-capture")
    );
    assert_approx_eq(
        request_facts[0].actual_credit_charge.unwrap_or_default(),
        0.025,
    );
}

#[tokio::test]
async fn stateful_audio_translations_route_captures_canonical_account_hold_from_request_price() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new().route(
        "/v1/audio/translations",
        post(upstream_translations_success),
    );
    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let seeded = seed_dual_scoped_gateway_account(&pool, 7205, 8205, 1.0).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    configure_openai_provider_with_price(
        admin_app,
        &admin_token,
        &format!("http://{address}"),
        0.025,
        0.0,
        0.0,
    )
    .await;

    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let store = SqliteAdminStore::new(pool);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/audio/translations")
                .header("authorization", format!("Bearer {}", seeded.plaintext))
                .header("x-request-id", "req-canonical-translations-capture")
                .header("content-type", "application/json")
                .body(Body::from("{\"model\":\"gpt-4.1\",\"file_id\":\"file_1\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let holds = store.list_account_holds().await.unwrap();
    assert_eq!(holds.len(), 1);
    assert_eq!(holds[0].status, AccountHoldStatus::Captured);
    assert_approx_eq(holds[0].estimated_quantity, 0.025);
    assert_approx_eq(holds[0].captured_quantity, 0.025);
    assert_approx_eq(holds[0].released_quantity, 0.0);

    let settlements = store.list_request_settlement_records().await.unwrap();
    assert_eq!(settlements.len(), 1);
    assert_eq!(settlements[0].status, RequestSettlementStatus::Captured);
    assert_approx_eq(settlements[0].captured_credit_amount, 0.025);
    assert_approx_eq(settlements[0].retail_charge_amount, 0.025);

    let request_facts = store.list_request_meter_facts().await.unwrap();
    assert_eq!(request_facts.len(), 1);
    assert_eq!(request_facts[0].request_status, RequestStatus::Succeeded);
    assert_eq!(
        request_facts[0].usage_capture_status,
        UsageCaptureStatus::Captured
    );
    assert_eq!(
        request_facts[0].gateway_request_ref.as_deref(),
        Some("req-canonical-translations-capture")
    );
}

#[tokio::test]
async fn stateful_images_route_releases_canonical_account_hold_when_upstream_relay_fails() {
    let pool = memory_pool().await;
    let seeded = seed_dual_scoped_gateway_account(&pool, 7203, 8203, 1.0).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    configure_openai_provider_with_price(
        admin_app,
        &admin_token,
        "http://127.0.0.1:1",
        0.05,
        0.0,
        0.0,
    )
    .await;

    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let store = SqliteAdminStore::new(pool);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/images/generations")
                .header("authorization", format!("Bearer {}", seeded.plaintext))
                .header("x-request-id", "req-canonical-images-release")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"prompt\":\"draw a skyline\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);

    let holds = store.list_account_holds().await.unwrap();
    assert_eq!(holds.len(), 1);
    assert_eq!(holds[0].status, AccountHoldStatus::Released);
    assert_approx_eq(holds[0].captured_quantity, 0.0);
    assert_approx_eq(holds[0].released_quantity, holds[0].estimated_quantity);

    let settlements = store.list_request_settlement_records().await.unwrap();
    assert_eq!(settlements.len(), 1);
    assert_eq!(settlements[0].status, RequestSettlementStatus::Released);

    let request_facts = store.list_request_meter_facts().await.unwrap();
    assert_eq!(request_facts.len(), 1);
    assert_eq!(request_facts[0].request_status, RequestStatus::Failed);
    assert_eq!(
        request_facts[0].usage_capture_status,
        UsageCaptureStatus::Failed
    );
    assert_eq!(
        request_facts[0].gateway_request_ref.as_deref(),
        Some("req-canonical-images-release")
    );
}

#[tokio::test]
async fn stateful_audio_speech_route_captures_canonical_account_hold_from_request_price() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new().route("/v1/audio/speech", post(upstream_speech_success));
    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let seeded = seed_dual_scoped_gateway_account(&pool, 7204, 8204, 1.0).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    configure_openai_provider_with_price(
        admin_app,
        &admin_token,
        &format!("http://{address}"),
        0.025,
        0.0,
        0.0,
    )
    .await;

    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let store = SqliteAdminStore::new(pool);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/audio/speech")
                .header("authorization", format!("Bearer {}", seeded.plaintext))
                .header("x-request-id", "req-canonical-audio-speech-capture")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"input\":\"hello\",\"voice\":\"alloy\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let holds = store.list_account_holds().await.unwrap();
    assert_eq!(holds.len(), 1);
    assert_eq!(holds[0].status, AccountHoldStatus::Captured);
    assert_approx_eq(holds[0].estimated_quantity, 0.025);
    assert_approx_eq(holds[0].captured_quantity, 0.025);
    assert_approx_eq(holds[0].released_quantity, 0.0);

    let settlements = store.list_request_settlement_records().await.unwrap();
    assert_eq!(settlements.len(), 1);
    assert_eq!(settlements[0].status, RequestSettlementStatus::Captured);
    assert_approx_eq(settlements[0].captured_credit_amount, 0.025);
    assert_approx_eq(settlements[0].retail_charge_amount, 0.025);

    let request_facts = store.list_request_meter_facts().await.unwrap();
    assert_eq!(request_facts.len(), 1);
    assert_eq!(request_facts[0].request_status, RequestStatus::Succeeded);
    assert_eq!(
        request_facts[0].usage_capture_status,
        UsageCaptureStatus::Captured
    );
    assert_eq!(
        request_facts[0].gateway_request_ref.as_deref(),
        Some("req-canonical-audio-speech-capture")
    );
}

#[tokio::test]
async fn stateful_music_route_defers_pending_canonical_hold_until_retrieve_reconciles_final_seconds(
) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/music", post(upstream_music_create_pending))
        .route(
            "/v1/music/music_pending",
            axum::routing::get(upstream_music_retrieve_completed),
        );
    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let seeded = seed_dual_scoped_gateway_account(&pool, 7301, 8301, 1.0).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    configure_openai_model_provider_with_custom_price(
        admin_app.clone(),
        &admin_token,
        &format!("http://{address}"),
        "suno-v4",
        "per_second_music",
        0.01,
        0.002,
        0.0,
    )
    .await;

    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let store = SqliteAdminStore::new(pool);
    let estimated_charge = 0.01 + (125.0 * 0.002);
    let reconciled_charge = 0.01 + (123.0 * 0.002);

    let create_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/music")
                .header("authorization", format!("Bearer {}", seeded.plaintext))
                .header("x-request-id", "req-canonical-music-pending")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"suno-v4\",\"prompt\":\"compose a synthwave track\",\"duration_seconds\":30.0}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_response.status(), StatusCode::OK);
    let create_json = read_json(create_response).await;
    assert_eq!(create_json["data"][0]["id"], "music_pending");
    assert_eq!(create_json["data"][0]["status"], "processing");

    let holds = store.list_account_holds().await.unwrap();
    assert_eq!(holds.len(), 1);
    assert_eq!(holds[0].status, AccountHoldStatus::Held);
    assert_approx_eq(holds[0].estimated_quantity, estimated_charge);
    assert_approx_eq(holds[0].captured_quantity, 0.0);
    assert_approx_eq(holds[0].released_quantity, 0.0);

    assert!(store
        .list_request_settlement_records()
        .await
        .unwrap()
        .is_empty());
    assert!(store.list_usage_records().await.unwrap().is_empty());
    assert!(store.list_billing_events().await.unwrap().is_empty());

    let request_facts = store.list_request_meter_facts().await.unwrap();
    assert_eq!(request_facts.len(), 1);
    assert_eq!(request_facts[0].request_status, RequestStatus::Running);
    assert_eq!(
        request_facts[0].usage_capture_status,
        UsageCaptureStatus::Pending
    );
    assert_eq!(
        request_facts[0].gateway_request_ref.as_deref(),
        Some("req-canonical-music-pending")
    );
    assert_eq!(
        request_facts[0].upstream_request_ref.as_deref(),
        Some("music_pending")
    );

    let retrieve_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/music/music_pending")
                .header("authorization", format!("Bearer {}", seeded.plaintext))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    if retrieve_response.status() != StatusCode::OK {
        let status = retrieve_response.status();
        let body = to_bytes(retrieve_response.into_body(), usize::MAX)
            .await
            .unwrap();
        panic!(
            "expected retrieve to succeed, got {} with body {}",
            status,
            String::from_utf8_lossy(&body)
        );
    }
    let retrieve_json = read_json(retrieve_response).await;
    assert_eq!(retrieve_json["id"], "music_pending");
    assert_eq!(retrieve_json["status"], "completed");
    assert_eq!(retrieve_json["duration_seconds"], 123.0);

    let holds = store.list_account_holds().await.unwrap();
    assert_eq!(holds.len(), 1);
    assert_eq!(holds[0].status, AccountHoldStatus::PartiallyReleased);
    assert_approx_eq(holds[0].captured_quantity, reconciled_charge);
    assert_approx_eq(
        holds[0].released_quantity,
        estimated_charge - reconciled_charge,
    );

    let settlements = store.list_request_settlement_records().await.unwrap();
    assert_eq!(settlements.len(), 1);
    assert_eq!(
        settlements[0].status,
        RequestSettlementStatus::PartiallyReleased
    );
    assert_approx_eq(settlements[0].captured_credit_amount, reconciled_charge);
    assert_approx_eq(settlements[0].retail_charge_amount, reconciled_charge);
    assert_approx_eq(
        settlements[0].released_credit_amount,
        estimated_charge - reconciled_charge,
    );

    let request_facts = store.list_request_meter_facts().await.unwrap();
    assert_eq!(request_facts.len(), 1);
    assert_eq!(request_facts[0].request_status, RequestStatus::Succeeded);
    assert_eq!(
        request_facts[0].usage_capture_status,
        UsageCaptureStatus::Reconciled
    );
    assert_approx_eq(
        request_facts[0].actual_credit_charge.unwrap_or_default(),
        reconciled_charge,
    );

    let usage_records = store.list_usage_records().await.unwrap();
    assert_eq!(usage_records.len(), 1);

    let billing_events = store.list_billing_events().await.unwrap();
    assert_eq!(billing_events.len(), 1);
    assert_eq!(
        billing_events[0].reference_id.as_deref(),
        Some("music_pending")
    );
    assert_eq!(billing_events[0].music_seconds, 123.0);

    let retrieve_response = gateway_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/music/music_pending")
                .header("authorization", format!("Bearer {}", seeded.plaintext))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(retrieve_response.status(), StatusCode::OK);

    assert_eq!(
        store.list_request_settlement_records().await.unwrap().len(),
        1
    );
    assert_eq!(store.list_usage_records().await.unwrap().len(), 1);
    assert_eq!(store.list_billing_events().await.unwrap().len(), 1);
}

#[tokio::test]
async fn stateful_video_route_defers_pending_canonical_hold_until_retrieve_reconciles_final_seconds(
) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/videos", post(upstream_video_create_pending))
        .route(
            "/v1/videos/video_pending",
            axum::routing::get(upstream_video_retrieve_completed),
        );
    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let seeded = seed_dual_scoped_gateway_account(&pool, 7302, 8302, 1.0).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    configure_openai_model_provider_with_custom_price(
        admin_app.clone(),
        &admin_token,
        &format!("http://{address}"),
        "veo-3",
        "per_minute_video",
        0.05,
        0.30,
        0.0,
    )
    .await;

    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let store = SqliteAdminStore::new(pool);
    let estimated_charge = 0.05 + (60.0 / 60.0 * 0.30);
    let reconciled_charge = 0.05 + (24.0 / 60.0 * 0.30);

    let create_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/videos")
                .header("authorization", format!("Bearer {}", seeded.plaintext))
                .header("x-request-id", "req-canonical-video-pending")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"veo-3\",\"prompt\":\"Generate a cinematic city flyover\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_response.status(), StatusCode::OK);
    let create_json = read_json(create_response).await;
    assert_eq!(create_json["data"][0]["id"], "video_1");
    assert_eq!(create_json["data"][0]["status"], "processing");

    let holds = store.list_account_holds().await.unwrap();
    assert_eq!(holds.len(), 1);
    assert_eq!(holds[0].status, AccountHoldStatus::Held);
    assert_approx_eq(holds[0].estimated_quantity, estimated_charge);
    assert_approx_eq(holds[0].captured_quantity, 0.0);
    assert_approx_eq(holds[0].released_quantity, 0.0);

    assert!(store
        .list_request_settlement_records()
        .await
        .unwrap()
        .is_empty());
    assert!(store.list_usage_records().await.unwrap().is_empty());
    assert!(store.list_billing_events().await.unwrap().is_empty());

    let request_facts = store.list_request_meter_facts().await.unwrap();
    assert_eq!(request_facts.len(), 1);
    assert_eq!(request_facts[0].request_status, RequestStatus::Running);
    assert_eq!(
        request_facts[0].usage_capture_status,
        UsageCaptureStatus::Pending
    );
    assert_eq!(
        request_facts[0].gateway_request_ref.as_deref(),
        Some("req-canonical-video-pending")
    );
    assert_eq!(
        request_facts[0].upstream_request_ref.as_deref(),
        Some("video_1")
    );

    let retrieve_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/videos/video_1")
                .header("authorization", format!("Bearer {}", seeded.plaintext))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    if retrieve_response.status() != StatusCode::OK {
        let status = retrieve_response.status();
        let body = to_bytes(retrieve_response.into_body(), usize::MAX)
            .await
            .unwrap();
        panic!(
            "expected retrieve to succeed, got {} with body {}",
            status,
            String::from_utf8_lossy(&body)
        );
    }
    let retrieve_json = read_json(retrieve_response).await;
    assert_eq!(retrieve_json["id"], "video_1");
    assert_eq!(retrieve_json["status"], "completed");
    assert_eq!(retrieve_json["duration_seconds"], 24.0);

    let holds = store.list_account_holds().await.unwrap();
    assert_eq!(holds.len(), 1);
    assert_eq!(holds[0].status, AccountHoldStatus::PartiallyReleased);
    assert_approx_eq(holds[0].captured_quantity, reconciled_charge);
    assert_approx_eq(
        holds[0].released_quantity,
        estimated_charge - reconciled_charge,
    );

    let settlements = store.list_request_settlement_records().await.unwrap();
    assert_eq!(settlements.len(), 1);
    assert_eq!(
        settlements[0].status,
        RequestSettlementStatus::PartiallyReleased
    );
    assert_approx_eq(settlements[0].captured_credit_amount, reconciled_charge);
    assert_approx_eq(settlements[0].retail_charge_amount, reconciled_charge);
    assert_approx_eq(
        settlements[0].released_credit_amount,
        estimated_charge - reconciled_charge,
    );

    let request_facts = store.list_request_meter_facts().await.unwrap();
    assert_eq!(request_facts.len(), 1);
    assert_eq!(request_facts[0].request_status, RequestStatus::Succeeded);
    assert_eq!(
        request_facts[0].usage_capture_status,
        UsageCaptureStatus::Reconciled
    );
    assert_approx_eq(
        request_facts[0].actual_credit_charge.unwrap_or_default(),
        reconciled_charge,
    );

    let usage_records = store.list_usage_records().await.unwrap();
    assert_eq!(usage_records.len(), 1);

    let billing_events = store.list_billing_events().await.unwrap();
    assert_eq!(billing_events.len(), 1);
    assert_eq!(billing_events[0].reference_id.as_deref(), Some("video_1"));
    assert_eq!(billing_events[0].video_seconds, 24.0);

    let retrieve_response = gateway_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/videos/video_1")
                .header("authorization", format!("Bearer {}", seeded.plaintext))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(retrieve_response.status(), StatusCode::OK);

    assert_eq!(
        store.list_request_settlement_records().await.unwrap().len(),
        1
    );
    assert_eq!(store.list_usage_records().await.unwrap().len(), 1);
    assert_eq!(store.list_billing_events().await.unwrap().len(), 1);
}

#[tokio::test]
async fn stateful_video_list_reconciles_pending_canonical_hold_from_terminal_items_once() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new().route(
        "/v1/videos",
        post(upstream_video_create_pending).get(upstream_video_list_completed),
    );
    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let seeded = seed_dual_scoped_gateway_account(&pool, 7303, 8303, 1.0).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    configure_openai_model_provider_with_custom_price(
        admin_app.clone(),
        &admin_token,
        &format!("http://{address}"),
        "veo-3",
        "per_minute_video",
        0.05,
        0.30,
        0.0,
    )
    .await;

    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let store = SqliteAdminStore::new(pool);
    let estimated_charge = 0.05 + (60.0 / 60.0 * 0.30);
    let reconciled_charge = 0.05 + (24.0 / 60.0 * 0.30);

    let create_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/videos")
                .header("authorization", format!("Bearer {}", seeded.plaintext))
                .header("x-request-id", "req-canonical-video-list-pending")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"veo-3\",\"prompt\":\"Generate a dramatic nature montage\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_response.status(), StatusCode::OK);
    let create_json = read_json(create_response).await;
    assert_eq!(create_json["data"][0]["id"], "video_1");

    let holds = store.list_account_holds().await.unwrap();
    assert_eq!(holds.len(), 1);
    assert_eq!(holds[0].status, AccountHoldStatus::Held);
    assert_approx_eq(holds[0].estimated_quantity, estimated_charge);

    let list_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/videos")
                .header("authorization", format!("Bearer {}", seeded.plaintext))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(list_response.status(), StatusCode::OK);
    let list_json = read_json(list_response).await;
    assert_eq!(list_json["data"][0]["id"], "video_1");
    assert_eq!(list_json["data"][0]["status"], "completed");
    assert_eq!(list_json["data"][0]["duration_seconds"], 24.0);

    let holds = store.list_account_holds().await.unwrap();
    assert_eq!(holds.len(), 1);
    assert_eq!(holds[0].status, AccountHoldStatus::PartiallyReleased);
    assert_approx_eq(holds[0].captured_quantity, reconciled_charge);
    assert_approx_eq(
        holds[0].released_quantity,
        estimated_charge - reconciled_charge,
    );

    let settlements = store.list_request_settlement_records().await.unwrap();
    assert_eq!(settlements.len(), 1);
    assert_eq!(
        settlements[0].status,
        RequestSettlementStatus::PartiallyReleased
    );
    assert_approx_eq(settlements[0].captured_credit_amount, reconciled_charge);

    let billing_events = store.list_billing_events().await.unwrap();
    assert_eq!(billing_events.len(), 1);
    assert_eq!(billing_events[0].reference_id.as_deref(), Some("video_1"));
    assert_eq!(billing_events[0].video_seconds, 24.0);

    let list_response = gateway_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/videos")
                .header("authorization", format!("Bearer {}", seeded.plaintext))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_response.status(), StatusCode::OK);

    assert_eq!(
        store.list_request_settlement_records().await.unwrap().len(),
        1
    );
    assert_eq!(store.list_usage_records().await.unwrap().len(), 1);
    assert_eq!(store.list_billing_events().await.unwrap().len(), 1);
}

async fn upstream_images_success() -> Json<Value> {
    Json(serde_json::json!({
        "created": 1_712_345_678_u64,
        "data": [
            {
                "b64_json": "aW1hZ2U="
            }
        ]
    }))
}

async fn upstream_transcriptions_success() -> Json<Value> {
    Json(serde_json::json!({
        "text": "hello from upstream"
    }))
}

async fn upstream_translations_success() -> Json<Value> {
    Json(serde_json::json!({
        "text": "hello from translated upstream"
    }))
}

async fn upstream_speech_success() -> axum::response::Response {
    (
        [(axum::http::header::CONTENT_TYPE, "audio/mpeg")],
        "speech-bytes",
    )
        .into_response()
}

async fn upstream_music_create_pending() -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "object":"list",
            "data":[{
                "id":"music_pending",
                "object":"music",
                "status":"processing",
                "model":"suno-v4"
            }]
        })),
    )
}

async fn upstream_music_retrieve_completed() -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"music_pending",
            "object":"music",
            "status":"completed",
            "model":"suno-v4",
            "duration_seconds":12.0,
            "audio_url":"https://example.com/music.mp3"
        })),
    )
}

async fn upstream_video_create_pending() -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "object":"list",
            "data":[{
                "id":"video_1",
                "object":"video",
                "status":"processing",
                "model":"veo-3"
            }]
        })),
    )
}

async fn upstream_video_retrieve_completed() -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"video_1",
            "object":"video",
            "status":"completed",
            "model":"veo-3",
            "duration_seconds":24.0,
            "url":"https://example.com/video.mp4"
        })),
    )
}

async fn upstream_video_list_completed() -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "object":"list",
            "data":[{
                "id":"video_1",
                "object":"video",
                "status":"completed",
                "model":"veo-3",
                "duration_seconds":24.0,
                "url":"https://example.com/video.mp4"
            }]
        })),
    )
}

fn assert_approx_eq(left: f64, right: f64) {
    assert!(
        (left - right).abs() < 1e-9,
        "expected {left} to be approximately {right}"
    );
}

struct SeededGatewayAccount {
    plaintext: String,
    hashed: String,
}

async fn seed_dual_scoped_gateway_account(
    pool: &SqlitePool,
    account_id: u64,
    lot_id: u64,
    lot_quantity: f64,
) -> SeededGatewayAccount {
    let store = SqliteAdminStore::new(pool.clone());
    let created = persist_gateway_api_key_with_metadata(
        &store,
        "tenant-1",
        "project-1",
        "live",
        "Dual scoped canonical key",
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();

    store
        .insert_identity_user_record(
            &IdentityUserRecord::new(9001, 1001, 2002)
                .with_display_name(Some("Canonical User".to_owned()))
                .with_created_at_ms(10)
                .with_updated_at_ms(10),
        )
        .await
        .unwrap();
    store
        .insert_canonical_api_key_record(
            &CanonicalApiKeyRecord::new(778899, 1001, 2002, 9001, &created.hashed)
                .with_key_prefix("skw_live")
                .with_display_name("Dual scoped key")
                .with_created_at_ms(20)
                .with_updated_at_ms(20),
        )
        .await
        .unwrap();

    let account = AccountRecord::new(account_id, 1001, 2002, 9001, AccountType::Primary)
        .with_created_at_ms(30)
        .with_updated_at_ms(30);
    store.insert_account_record(&account).await.unwrap();

    let lot = AccountBenefitLotRecord::new(
        lot_id,
        1001,
        2002,
        account_id,
        9001,
        AccountBenefitType::CashCredit,
    )
    .with_source_type(AccountBenefitSourceType::Recharge)
    .with_original_quantity(lot_quantity)
    .with_remaining_quantity(lot_quantity)
    .with_created_at_ms(40)
    .with_updated_at_ms(40);
    store.insert_account_benefit_lot(&lot).await.unwrap();

    SeededGatewayAccount {
        plaintext: created.plaintext,
        hashed: created.hashed,
    }
}

async fn configure_broken_openai_provider(admin_app: axum::Router, admin_token: &str) {
    configure_openai_provider_with_price(
        admin_app,
        admin_token,
        "http://127.0.0.1:1",
        0.01,
        2.5,
        10.0,
    )
    .await;
}

async fn configure_openai_provider_with_price(
    admin_app: axum::Router,
    admin_token: &str,
    base_url: &str,
    request_price: f64,
    input_price: f64,
    output_price: f64,
) {
    configure_openai_model_provider_with_price(
        admin_app,
        admin_token,
        base_url,
        "gpt-4.1",
        request_price,
        input_price,
        output_price,
    )
    .await;
}

async fn configure_openai_model_provider_with_custom_price(
    admin_app: axum::Router,
    admin_token: &str,
    base_url: &str,
    model_id: &str,
    price_unit: &str,
    request_price: f64,
    input_price: f64,
    output_price: f64,
) {
    let channel = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/channels")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"id\":\"openai\",\"name\":\"OpenAI\"}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(channel.status(), StatusCode::CREATED);

    let provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-broken-chat\",\"channel_id\":\"openai\",\"extension_id\":\"sdkwork.provider.openai.official\",\"adapter_kind\":\"custom-openai\",\"base_url\":\"{base_url}\",\"display_name\":\"Broken Chat Provider\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider.status(), StatusCode::CREATED);

    let credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"tenant_id\":\"tenant-1\",\"provider_id\":\"provider-broken-chat\",\"key_reference\":\"cred-broken\",\"secret_value\":\"sk-broken\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(credential.status(), StatusCode::CREATED);

    let model = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"external_name\":\"{model_id}\",\"provider_id\":\"provider-broken-chat\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(model.status(), StatusCode::CREATED);

    let price = admin_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/model-prices")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"channel_id\":\"openai\",\"model_id\":\"{model_id}\",\"proxy_provider_id\":\"provider-broken-chat\",\"currency_code\":\"USD\",\"price_unit\":\"{price_unit}\",\"input_price\":{input_price},\"output_price\":{output_price},\"cache_read_price\":0.0,\"cache_write_price\":0.0,\"request_price\":{request_price},\"is_active\":true}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(price.status(), StatusCode::CREATED);
}

async fn configure_openai_model_provider_with_price(
    admin_app: axum::Router,
    admin_token: &str,
    base_url: &str,
    model_id: &str,
    request_price: f64,
    input_price: f64,
    output_price: f64,
) {
    let channel = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/channels")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"id\":\"openai\",\"name\":\"OpenAI\"}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(channel.status(), StatusCode::CREATED);

    let provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-broken-chat\",\"channel_id\":\"openai\",\"extension_id\":\"sdkwork.provider.openai.official\",\"adapter_kind\":\"custom-openai\",\"base_url\":\"{base_url}\",\"display_name\":\"Broken Chat Provider\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider.status(), StatusCode::CREATED);

    let credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"tenant_id\":\"tenant-1\",\"provider_id\":\"provider-broken-chat\",\"key_reference\":\"cred-broken\",\"secret_value\":\"sk-broken\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(credential.status(), StatusCode::CREATED);

    let model = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"external_name\":\"{model_id}\",\"provider_id\":\"provider-broken-chat\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(model.status(), StatusCode::CREATED);

    let price = admin_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/model-prices")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"channel_id\":\"openai\",\"model_id\":\"{model_id}\",\"proxy_provider_id\":\"provider-broken-chat\",\"currency_code\":\"USD\",\"price_unit\":\"per_1m_tokens\",\"input_price\":{input_price},\"output_price\":{output_price},\"cache_read_price\":0.0,\"cache_write_price\":0.0,\"request_price\":{request_price},\"is_active\":true}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(price.status(), StatusCode::CREATED);
}

async fn upstream_chat_with_usage() -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"chatcmpl_upstream",
            "object":"chat.completion",
            "model":"gpt-4.1",
            "choices":[],
            "usage":{
                "prompt_tokens":120,
                "completion_tokens":80,
                "total_tokens":200
            }
        })),
    )
}

async fn upstream_chat_stream_with_usage() -> impl IntoResponse {
    (
        [(axum::http::header::CONTENT_TYPE, "text/event-stream")],
        concat!(
            "data: {\"id\":\"chatcmpl_stream_upstream\",\"object\":\"chat.completion.chunk\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"hello\"}}]}\n\n",
            "data: {\"id\":\"chatcmpl_stream_upstream\",\"object\":\"chat.completion.chunk\",\"choices\":[],\"usage\":{\"prompt_tokens\":120,\"completion_tokens\":80,\"total_tokens\":200}}\n\n",
            "data: [DONE]\n\n"
        ),
    )
}

async fn upstream_chat_stream_without_done() -> impl IntoResponse {
    (
        [(axum::http::header::CONTENT_TYPE, "text/event-stream")],
        "data: {\"id\":\"chatcmpl_stream_upstream\",\"object\":\"chat.completion.chunk\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"hello\"}}]}\n\n",
    )
}

async fn upstream_responses_with_usage() -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"resp_upstream",
            "object":"response",
            "model":"gpt-4.1",
            "output":[],
            "usage":{
                "input_tokens":160,
                "output_tokens":40,
                "total_tokens":200
            }
        })),
    )
}

async fn upstream_responses_stream_with_completion_usage() -> impl IntoResponse {
    (
        [(axum::http::header::CONTENT_TYPE, "text/event-stream")],
        concat!(
            "data: {\"type\":\"response.created\",\"response\":{\"id\":\"resp_upstream_stream\",\"object\":\"response\",\"model\":\"gpt-4.1\"}}\n\n",
            "data: {\"type\":\"response.output_text.delta\",\"delta\":\"hello\"}\n\n",
            "data: {\"type\":\"response.completed\",\"response\":{\"id\":\"resp_upstream_stream\",\"usage\":{\"input_tokens\":160,\"output_tokens\":40,\"total_tokens\":200}}}\n\n",
            "data: [DONE]\n\n"
        ),
    )
}

async fn upstream_responses_stream_without_completion() -> impl IntoResponse {
    (
        [(axum::http::header::CONTENT_TYPE, "text/event-stream")],
        concat!(
            "data: {\"type\":\"response.output_text.delta\",\"delta\":\"hello\"}\n\n",
            "data: [DONE]\n\n"
        ),
    )
}

async fn upstream_completions_with_usage() -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"cmpl_upstream",
            "object":"text_completion",
            "choices":[{"index":0,"text":"relay completion","finish_reason":"stop"}],
            "usage":{
                "prompt_tokens":90,
                "completion_tokens":30,
                "total_tokens":120
            }
        })),
    )
}

async fn upstream_embeddings_with_usage() -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "object":"list",
            "data":[{"object":"embedding","embedding":[0.42, 0.11],"index":0}],
            "model":"text-embedding-3-large",
            "usage":{
                "prompt_tokens":75,
                "total_tokens":75
            }
        })),
    )
}

async fn upstream_moderations() -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"modr_upstream",
            "model":"omni-moderation-latest",
            "results":[{"flagged":false,"category_scores":{"violence":0.0}}]
        })),
    )
}
