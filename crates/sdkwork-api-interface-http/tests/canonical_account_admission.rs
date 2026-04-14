use axum::body::Body;
use axum::http::{Request, StatusCode};
use sdkwork_api_app_identity::{
    gateway_auth_subject_from_request_context, hash_gateway_api_key, GatewayRequestContext,
};
use sdkwork_api_domain_billing::{
    AccountBenefitLotRecord, AccountBenefitSourceType, AccountBenefitType, AccountHoldStatus,
    AccountRecord, AccountStatus, AccountType, RequestSettlementStatus,
};
use sdkwork_api_storage_core::AccountKernelStore;
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use sqlx::SqlitePool;
use tower::ServiceExt;

mod support;

async fn memory_pool() -> SqlitePool {
    sdkwork_api_storage_sqlite::run_migrations("sqlite::memory:")
        .await
        .unwrap()
}

async fn seed_platform_credit_account(
    store: &SqliteAdminStore,
    tenant_id: &str,
    project_id: &str,
    api_key: &str,
    account_id: u64,
    lot_id: u64,
) -> AccountRecord {
    let request_context = GatewayRequestContext {
        tenant_id: tenant_id.to_owned(),
        project_id: project_id.to_owned(),
        environment: "live".to_owned(),
        api_key_hash: hash_gateway_api_key(&api_key),
        api_key_group_id: None,
        canonical_tenant_id: None,
        canonical_organization_id: None,
        canonical_user_id: None,
        canonical_api_key_id: None,
    };
    let subject = gateway_auth_subject_from_request_context(&request_context);

    let account = AccountRecord::new(
        account_id,
        subject.tenant_id,
        subject.organization_id,
        subject.user_id,
        AccountType::Primary,
    )
    .with_status(AccountStatus::Active)
    .with_created_at_ms(10)
    .with_updated_at_ms(10);
    let balance_lot = AccountBenefitLotRecord::new(
        lot_id,
        subject.tenant_id,
        subject.organization_id,
        account.account_id,
        subject.user_id,
        AccountBenefitType::CashCredit,
    )
    .with_source_type(AccountBenefitSourceType::Recharge)
    .with_original_quantity(10.0)
    .with_remaining_quantity(10.0)
    .with_created_at_ms(11)
    .with_updated_at_ms(11);

    store.insert_account_record(&account).await.unwrap();
    store
        .insert_account_benefit_lot(&balance_lot)
        .await
        .unwrap();
    account
}

async fn assert_platform_credit_settlement(store: &SqliteAdminStore, account_id: u64, amount: f64) {
    let holds = store.list_account_holds().await.unwrap();
    assert_eq!(holds.len(), 1);
    assert_eq!(holds[0].account_id, account_id);
    assert_eq!(holds[0].status, AccountHoldStatus::Captured);
    assert!((holds[0].estimated_quantity - amount).abs() < f64::EPSILON);
    assert!((holds[0].captured_quantity - amount).abs() < f64::EPSILON);
    assert!((holds[0].released_quantity - 0.0).abs() < f64::EPSILON);

    let settlements = store.list_request_settlement_records().await.unwrap();
    assert_eq!(settlements.len(), 1);
    assert_eq!(settlements[0].account_id, account_id);
    assert_eq!(settlements[0].status, RequestSettlementStatus::Captured);
    assert!((settlements[0].estimated_credit_hold - amount).abs() < f64::EPSILON);
    assert!((settlements[0].captured_credit_amount - amount).abs() < f64::EPSILON);
    assert!((settlements[0].released_credit_amount - 0.0).abs() < f64::EPSILON);
    assert!((settlements[0].retail_charge_amount - amount).abs() < f64::EPSILON);

    let lots = store.list_account_benefit_lots().await.unwrap();
    assert_eq!(lots.len(), 1);
    assert!((lots[0].remaining_quantity - (10.0 - amount)).abs() < f64::EPSILON);
    assert!((lots[0].held_quantity - 0.0).abs() < f64::EPSILON);
}

async fn assert_platform_credit_release(store: &SqliteAdminStore, account_id: u64, amount: f64) {
    let holds = store.list_account_holds().await.unwrap();
    assert_eq!(holds.len(), 1);
    assert_eq!(holds[0].account_id, account_id);
    assert_eq!(holds[0].status, AccountHoldStatus::Released);
    assert!((holds[0].estimated_quantity - amount).abs() < f64::EPSILON);
    assert!((holds[0].captured_quantity - 0.0).abs() < f64::EPSILON);
    assert!((holds[0].released_quantity - amount).abs() < f64::EPSILON);

    let settlements = store.list_request_settlement_records().await.unwrap();
    assert_eq!(settlements.len(), 0);

    let lots = store.list_account_benefit_lots().await.unwrap();
    assert_eq!(lots.len(), 1);
    assert!((lots[0].remaining_quantity - 10.0).abs() < f64::EPSILON);
    assert!((lots[0].held_quantity - 0.0).abs() < f64::EPSILON);
}

async fn response_body_text(response: axum::response::Response) -> String {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    String::from_utf8_lossy(&bytes).into_owned()
}

#[tokio::test]
async fn stateful_chat_route_captures_platform_credit_hold_into_request_settlement() {
    let tenant_id = "tenant-commercial-admission";
    let project_id = "project-commercial-admission";
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let account =
        seed_platform_credit_account(&store, tenant_id, project_id, &api_key, 8801, 9901).await;

    let app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"bill canonically\"}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = response_body_text(response).await;
    assert_eq!(status, StatusCode::OK, "unexpected body: {body}");
    assert_platform_credit_settlement(&store, account.account_id, 0.10).await;
}

#[tokio::test]
async fn stateful_responses_route_captures_platform_credit_hold_into_request_settlement() {
    let tenant_id = "tenant-commercial-admission-responses";
    let project_id = "project-commercial-admission-responses";
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let account =
        seed_platform_credit_account(&store, tenant_id, project_id, &api_key, 8802, 9902).await;

    let app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"input\":\"bill responses canonically\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_platform_credit_settlement(&store, account.account_id, 0.12).await;
}

#[tokio::test]
async fn stateful_completions_route_captures_platform_credit_hold_into_request_settlement() {
    let tenant_id = "tenant-commercial-admission-completions";
    let project_id = "project-commercial-admission-completions";
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let account =
        seed_platform_credit_account(&store, tenant_id, project_id, &api_key, 8803, 9903).await;

    let app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/completions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-3.5-turbo-instruct\",\"prompt\":\"bill completions canonically\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_platform_credit_settlement(&store, account.account_id, 0.08).await;
}

#[tokio::test]
async fn stateful_embeddings_route_captures_platform_credit_hold_into_request_settlement() {
    let tenant_id = "tenant-commercial-admission-embeddings";
    let project_id = "project-commercial-admission-embeddings";
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let account =
        seed_platform_credit_account(&store, tenant_id, project_id, &api_key, 8804, 9904).await;

    let app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/embeddings")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"text-embedding-3-large\",\"input\":\"bill embeddings canonically\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_platform_credit_settlement(&store, account.account_id, 0.01).await;
}

#[tokio::test]
async fn stateful_invalid_embeddings_route_releases_platform_credit_hold() {
    let tenant_id = "tenant-commercial-admission-embeddings-invalid";
    let project_id = "project-commercial-admission-embeddings-invalid";
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let account =
        seed_platform_credit_account(&store, tenant_id, project_id, &api_key, 8807, 9907).await;

    let app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/embeddings")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"\",\"input\":\"invalid embeddings\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = response_body_text(response).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "unexpected body: {body}");
    assert!(
        body.contains("Embedding model is required."),
        "unexpected body: {body}"
    );
    assert_platform_credit_release(&store, account.account_id, 0.01).await;
}

#[tokio::test]
async fn stateful_anthropic_messages_route_captures_platform_credit_hold_into_request_settlement() {
    let tenant_id = "tenant-commercial-admission-anthropic";
    let project_id = "project-commercial-admission-anthropic";
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let account =
        seed_platform_credit_account(&store, tenant_id, project_id, &api_key, 8805, 9905).await;

    let app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("x-api-key", api_key.clone())
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "model": "gpt-4.1",
                        "max_tokens": 128,
                        "messages": [
                            {
                                "role": "user",
                                "content": "bill anthropic canonically"
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
    assert_platform_credit_settlement(&store, account.account_id, 0.10).await;
}

#[tokio::test]
async fn stateful_gemini_generate_content_route_captures_platform_credit_hold_into_request_settlement(
) {
    let tenant_id = "tenant-commercial-admission-gemini";
    let project_id = "project-commercial-admission-gemini";
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let account =
        seed_platform_credit_account(&store, tenant_id, project_id, &api_key, 8806, 9906).await;

    let app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/v1beta/models/gemini-2.5-pro:generateContent?key={api_key}"
                ))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "contents": [
                            {
                                "role": "user",
                                "parts": [
                                    { "text": "bill gemini canonically" }
                                ]
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
    assert_platform_credit_settlement(&store, account.account_id, 0.10).await;
}
