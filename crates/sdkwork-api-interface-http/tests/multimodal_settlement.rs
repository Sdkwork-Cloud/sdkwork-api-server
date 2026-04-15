#![allow(clippy::too_many_arguments)]

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use axum::routing::{get, post};
use axum::{extract::State, Json, Router};
use sdkwork_api_app_identity::{persist_gateway_api_key_with_metadata, PersistGatewayApiKeyInput};
use sdkwork_api_domain_billing::{
    AccountBenefitLotRecord, AccountBenefitSourceType, AccountBenefitType, AccountHoldStatus,
    AccountRecord, AccountType, RequestSettlementStatus,
};
use sdkwork_api_domain_identity::{CanonicalApiKeyRecord, IdentityUserRecord};
use sdkwork_api_domain_usage::{RequestStatus, UsageCaptureStatus};
use sdkwork_api_storage_core::{AccountKernelStore, IdentityKernelStore};
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use serde_json::Value;
use sqlx::SqlitePool;
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

mod support;

#[derive(Clone, Default)]
struct UpstreamCaptureState {
    authorization: Arc<Mutex<Option<String>>>,
}

struct SeededGatewayAccount {
    plaintext: String,
    hashed: String,
}

struct VideoMutationCase {
    route_uri: &'static str,
    request_body: &'static str,
    upstream_post_path: &'static str,
    route_key: &'static str,
    child_video_id: &'static str,
}

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
async fn video_remix_pending_route_holds_and_reconciles_canonical_video_pricing() {
    run_pending_video_mutation_reconcile_case(VideoMutationCase {
        route_uri: "/v1/videos/video_1/remix",
        request_body: "{\"prompt\":\"Make it sunset\"}",
        upstream_post_path: "/v1/videos/video_1/remix",
        route_key: "video_1",
        child_video_id: "video_1_remix",
    })
    .await;
}

#[tokio::test]
async fn video_extend_pending_route_holds_and_reconciles_canonical_video_pricing() {
    run_pending_video_mutation_reconcile_case(VideoMutationCase {
        route_uri: "/v1/videos/video_1/extend",
        request_body: "{\"prompt\":\"Extend the ending\"}",
        upstream_post_path: "/v1/videos/video_1/extend",
        route_key: "video_1",
        child_video_id: "video_1_extended",
    })
    .await;
}

#[tokio::test]
async fn video_edits_pending_route_holds_and_reconciles_canonical_video_pricing() {
    run_pending_video_mutation_reconcile_case(VideoMutationCase {
        route_uri: "/v1/videos/edits",
        request_body: "{\"prompt\":\"Add dramatic lighting\",\"video_id\":\"video_1\"}",
        upstream_post_path: "/v1/videos/edits",
        route_key: "video_1",
        child_video_id: "video_1_edited",
    })
    .await;
}

#[tokio::test]
async fn video_extensions_pending_route_holds_and_reconciles_canonical_video_pricing() {
    run_pending_video_mutation_reconcile_case(VideoMutationCase {
        route_uri: "/v1/videos/extensions",
        request_body: "{\"prompt\":\"Extend the ending\",\"video_id\":\"video_1\"}",
        upstream_post_path: "/v1/videos/extensions",
        route_key: "video_1",
        child_video_id: "video_1_extended",
    })
    .await;
}

async fn run_pending_video_mutation_reconcile_case(case: VideoMutationCase) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream_state = UpstreamCaptureState::default();
    let request_trace_id = format!("req-{}", case.child_video_id);
    let pending_response = serde_json::json!({
        "object":"list",
        "data":[{
            "id": case.child_video_id,
            "object":"video",
            "status":"processing",
            "model":"veo-3"
        }]
    });
    let retrieve_response = serde_json::json!({
        "id": case.child_video_id,
        "object":"video",
        "status":"completed",
        "model":"veo-3",
        "duration_seconds":24.0,
        "url":"https://example.com/video.mp4"
    });
    let retrieve_path = format!("/v1/videos/{}", case.child_video_id);
    let upstream = Router::new()
        .route(
            case.upstream_post_path,
            post({
                let pending_response = pending_response.clone();
                move |State(state): State<UpstreamCaptureState>, headers: axum::http::HeaderMap| {
                    let pending_response = pending_response.clone();
                    async move {
                        *state.authorization.lock().unwrap() = headers
                            .get("authorization")
                            .and_then(|value| value.to_str().ok())
                            .map(ToOwned::to_owned);
                        (StatusCode::OK, Json(pending_response))
                    }
                }
            }),
        )
        .route(
            &retrieve_path,
            get({
                let retrieve_response = retrieve_response.clone();
                move || {
                    let retrieve_response = retrieve_response.clone();
                    async move { (StatusCode::OK, Json(retrieve_response)) }
                }
            }),
        )
        .with_state(upstream_state.clone());
    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let seeded = seed_dual_scoped_gateway_account(&pool, 7801, 8801, 1.0).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(&pool, admin_app.clone()).await;
    configure_video_provider_with_custom_price(
        admin_app.clone(),
        &admin_token,
        &format!("http://{address}"),
        "provider-video-mutation",
        case.route_key,
        "veo-3",
        "per_minute_video",
        0.05,
        0.30,
    )
    .await;

    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let store = SqliteAdminStore::new(pool);
    let estimated_charge = 0.05 + 0.30;
    let reconciled_charge = 0.05 + (24.0 / 60.0 * 0.30);

    let create_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(case.route_uri)
                .header("authorization", format!("Bearer {}", seeded.plaintext))
                .header("x-request-id", &request_trace_id)
                .header("content-type", "application/json")
                .body(Body::from(case.request_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_response.status(), StatusCode::OK);
    let create_json = read_json(create_response).await;
    assert_eq!(create_json["data"][0]["id"], case.child_video_id);
    assert_eq!(create_json["data"][0]["status"], "processing");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-broken")
    );

    let holds = store.list_account_holds().await.unwrap();
    assert_eq!(holds.len(), 1);
    assert_eq!(holds[0].status, AccountHoldStatus::Held);
    assert_approx_eq(holds[0].estimated_quantity, estimated_charge);
    assert_approx_eq(holds[0].captured_quantity, 0.0);
    assert_approx_eq(holds[0].released_quantity, 0.0);

    let request_facts = store.list_request_meter_facts().await.unwrap();
    assert_eq!(request_facts.len(), 1);
    assert_eq!(request_facts[0].request_status, RequestStatus::Running);
    assert_eq!(
        request_facts[0].usage_capture_status,
        UsageCaptureStatus::Pending
    );
    assert_eq!(request_facts[0].model_code, "veo-3");
    assert_eq!(
        request_facts[0].upstream_request_ref.as_deref(),
        Some(case.child_video_id)
    );
    assert_eq!(
        request_facts[0].gateway_request_ref.as_deref(),
        Some(request_trace_id.as_str())
    );
    assert_eq!(
        request_facts[0].api_key_hash.as_deref(),
        Some(seeded.hashed.as_str())
    );

    assert!(store.list_usage_records().await.unwrap().is_empty());
    assert!(store.list_billing_events().await.unwrap().is_empty());
    assert!(store
        .list_request_settlement_records()
        .await
        .unwrap()
        .is_empty());

    let retrieve_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/v1/videos/{}", case.child_video_id))
                .header("authorization", format!("Bearer {}", seeded.plaintext))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(retrieve_response.status(), StatusCode::OK);
    let retrieve_json = read_json(retrieve_response).await;
    assert_eq!(retrieve_json["id"], case.child_video_id);
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
    assert_eq!(billing_events[0].capability, "videos");
    assert_eq!(billing_events[0].route_key, "veo-3");
    assert_eq!(billing_events[0].usage_model, "veo-3");
    assert_eq!(
        billing_events[0].reference_id.as_deref(),
        Some(case.child_video_id)
    );
    assert_eq!(billing_events[0].video_seconds, 24.0);

    let retrieve_response = gateway_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/v1/videos/{}", case.child_video_id))
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

fn assert_approx_eq(left: f64, right: f64) {
    assert!(
        (left - right).abs() < 1e-9,
        "expected {left} to be approximately {right}"
    );
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
        PersistGatewayApiKeyInput {
            tenant_id: "tenant-1",
            project_id: "project-1",
            environment: "live",
            label: "Dual scoped canonical key",
            expires_at_ms: None,
            plaintext_key: None,
            notes: None,
            api_key_group_id: None,
        },
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

async fn configure_video_provider_with_custom_price(
    admin_app: axum::Router,
    admin_token: &str,
    base_url: &str,
    provider_id: &str,
    route_key: &str,
    model_id: &str,
    price_unit: &str,
    request_price: f64,
    input_price: f64,
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
                    "{{\"id\":\"{provider_id}\",\"channel_id\":\"openai\",\"extension_id\":\"sdkwork.provider.openai.official\",\"adapter_kind\":\"custom-openai\",\"base_url\":\"{base_url}\",\"display_name\":\"Video Mutation Provider\"}}"
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
                    "{{\"tenant_id\":\"tenant-1\",\"provider_id\":\"{provider_id}\",\"key_reference\":\"cred-video-mutation\",\"secret_value\":\"sk-broken\"}}"
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
                .body(Body::from(format!(
                    "{{\"external_name\":\"{model_id}\",\"provider_id\":\"{provider_id}\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(model.status(), StatusCode::CREATED);

    let price = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/model-prices")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"channel_id\":\"openai\",\"model_id\":\"{model_id}\",\"proxy_provider_id\":\"{provider_id}\",\"currency_code\":\"USD\",\"price_unit\":\"{price_unit}\",\"input_price\":{input_price},\"output_price\":0.0,\"cache_read_price\":0.0,\"cache_write_price\":0.0,\"request_price\":{request_price},\"is_active\":true}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(price.status(), StatusCode::CREATED);

    let policy = admin_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": format!("route-video-mutation-{route_key}"),
                        "capability": "videos",
                        "model_pattern": route_key,
                        "enabled": true,
                        "priority": 200,
                        "ordered_provider_ids": [provider_id]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(policy.status(), StatusCode::CREATED);
}
