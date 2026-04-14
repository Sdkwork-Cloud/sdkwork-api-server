use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use sdkwork_api_app_identity::{
    hash_gateway_api_key, persist_gateway_api_key_with_metadata, PersistGatewayApiKeyInput,
};
use sdkwork_api_domain_billing::{
    AccountBenefitLotRecord, AccountBenefitSourceType, AccountBenefitType, AccountHoldStatus,
    AccountRecord, AccountType,
};
use sdkwork_api_domain_identity::{ApiKeyGroupRecord, CanonicalApiKeyRecord, IdentityUserRecord};
use sdkwork_api_storage_core::{AccountKernelStore, IdentityKernelStore};
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use serde_json::Value;
use sqlx::SqlitePool;
use tower::ServiceExt;

mod support;

async fn read_json(response: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn memory_pool() -> SqlitePool {
    sdkwork_api_storage_sqlite::run_migrations("sqlite::memory:")
        .await
        .unwrap()
}

#[tokio::test]
async fn stateful_gateway_requires_api_key_and_uses_request_context() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    let group = ApiKeyGroupRecord::new(
        "group-live",
        "tenant-live",
        "project-live",
        "live",
        "Production keys",
        "production-keys",
    );
    store.insert_api_key_group(&group).await.unwrap();
    let created = persist_gateway_api_key_with_metadata(
        &store,
        PersistGatewayApiKeyInput {
            tenant_id: "tenant-live",
            project_id: "project-live",
            environment: "live",
            label: "Production request key",
            expires_at_ms: None,
            plaintext_key: None,
            notes: None,
            api_key_group_id: Some(&group.group_id),
        },
    )
    .await
    .unwrap();

    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let admin_token = support::issue_admin_token(admin_app.clone()).await;

    let unauthorized = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"hi\"}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);

    let authorized = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {}", created.plaintext))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"hi\"}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(authorized.status(), StatusCode::OK);

    let ledger = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/billing/ledger")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(ledger.status(), StatusCode::OK);
    let ledger_json = read_json(ledger).await;
    assert_eq!(ledger_json[0]["project_id"], "project-live");

    let routing_logs = admin_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/routing/decision-logs")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(routing_logs.status(), StatusCode::OK);
    let routing_logs_json = read_json(routing_logs).await;
    assert_eq!(routing_logs_json[0]["project_id"], "project-live");
    assert_eq!(routing_logs_json[0]["api_key_group_id"], "group-live");
}

#[tokio::test]
async fn stateful_moderations_route_requires_gateway_api_key() {
    let pool = memory_pool().await;
    let _api_key = support::issue_gateway_api_key(&pool, "tenant-live", "project-live").await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let unauthorized = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/moderations")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"omni-moderation-latest\",\"input\":\"hi\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn stateful_images_generation_route_requires_gateway_api_key() {
    let pool = memory_pool().await;
    let _api_key = support::issue_gateway_api_key(&pool, "tenant-live", "project-live").await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let unauthorized = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/images/generations")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-image-1\",\"prompt\":\"draw a lighthouse\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn stateful_gateway_accepts_canonical_only_api_key_and_resolves_payable_account() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new().route("/v1/chat/completions", post(upstream_chat_with_usage));
    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;

    create_tenant_and_project(&admin_app, &admin_token, "1001", "project-canonical-1001").await;
    configure_openai_provider_with_price(
        admin_app.clone(),
        &admin_token,
        &format!("http://{address}"),
        "1001",
        0.01,
        2.5,
        10.0,
    )
    .await;

    let plaintext = "sk-canonical-only-live";
    let hashed = hash_gateway_api_key(plaintext);
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
            &CanonicalApiKeyRecord::new(778899, 1001, 2002, 9001, &hashed)
                .with_key_prefix("skw_live")
                .with_display_name("Canonical-only key")
                .with_created_at_ms(20)
                .with_updated_at_ms(20),
        )
        .await
        .unwrap();
    store
        .insert_account_record(
            &AccountRecord::new(7001, 1001, 2002, 9001, AccountType::Primary)
                .with_created_at_ms(30)
                .with_updated_at_ms(30),
        )
        .await
        .unwrap();
    store
        .insert_account_benefit_lot(
            &AccountBenefitLotRecord::new(
                8001,
                1001,
                2002,
                7001,
                9001,
                AccountBenefitType::CashCredit,
            )
            .with_source_type(AccountBenefitSourceType::Recharge)
            .with_original_quantity(1.0)
            .with_remaining_quantity(1.0)
            .with_created_at_ms(40)
            .with_updated_at_ms(40),
        )
        .await
        .unwrap();

    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {plaintext}"))
                .header("x-request-id", "req-canonical-only-auth")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"route with canonical only key\"}],\"max_completion_tokens\":100}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let holds = store.list_account_holds().await.unwrap();
    assert_eq!(holds.len(), 1);
    assert_eq!(holds[0].account_id, 7001);
    assert_eq!(holds[0].status, AccountHoldStatus::Captured);

    let usage_records = store.list_usage_records().await.unwrap();
    assert_eq!(usage_records.len(), 1);
    assert_eq!(usage_records[0].project_id, "project-canonical-1001");
    assert_eq!(
        usage_records[0].api_key_hash.as_deref(),
        Some(hashed.as_str())
    );

    let billing_events = store.list_billing_events().await.unwrap();
    assert_eq!(billing_events.len(), 1);
    assert_eq!(billing_events[0].project_id, "project-canonical-1001");
    assert_eq!(
        billing_events[0].api_key_hash.as_deref(),
        Some(hashed.as_str())
    );

    let canonical_key = store
        .find_canonical_api_key_record_by_hash(&hashed)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(canonical_key.api_key_id, 778899);
    assert!(canonical_key.last_used_at_ms.is_some());

    let routing_logs = admin_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/routing/decision-logs")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(routing_logs.status(), StatusCode::OK);
    let routing_logs_json = read_json(routing_logs).await;
    assert_eq!(routing_logs_json[0]["project_id"], "project-canonical-1001");
}

async fn create_tenant_and_project(
    admin_app: &axum::Router,
    admin_token: &str,
    tenant_id: &str,
    project_id: &str,
) {
    let tenant = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/tenants")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"{tenant_id}\",\"name\":\"Canonical Tenant\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(tenant.status(), StatusCode::CREATED);

    let project = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/projects")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"id\":\"{project_id}\",\"name\":\"default\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(project.status(), StatusCode::CREATED);
}

async fn configure_openai_provider_with_price(
    admin_app: axum::Router,
    admin_token: &str,
    base_url: &str,
    tenant_id: &str,
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
                    "{{\"id\":\"provider-openai-canonical-only\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"{base_url}\",\"display_name\":\"OpenAI Canonical Only\"}}"
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
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"provider-openai-canonical-only\",\"key_reference\":\"cred-openai-canonical-only\",\"secret_value\":\"sk-upstream-openai\"}}"
                )))
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
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-openai-canonical-only\"}",
                ))
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
                    "{{\"channel_id\":\"openai\",\"model_id\":\"gpt-4.1\",\"proxy_provider_id\":\"provider-openai-canonical-only\",\"currency_code\":\"USD\",\"price_unit\":\"per_1m_tokens\",\"input_price\":{input_price},\"output_price\":{output_price},\"cache_read_price\":0.0,\"cache_write_price\":0.0,\"request_price\":{request_price},\"is_active\":true}}"
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
            "id":"chatcmpl_canonical_only",
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
