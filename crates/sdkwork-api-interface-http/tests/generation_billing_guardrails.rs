use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use sdkwork_api_app_identity::{persist_gateway_api_key_with_metadata, PersistGatewayApiKeyInput};
use serde_json::Value;
use sqlx::SqlitePool;
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

mod support;

#[tokio::test]
async fn stateful_completions_route_keeps_request_model_for_billing_despite_response_id() {
    let tenant_id = "tenant-completions-model-billing";
    let project_id = "project-completions-model-billing";
    let request_model = "gpt-3.5-turbo-instruct";
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/completions", post(upstream_completions_handler))
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    create_channel(&admin_app, &admin_token).await;
    create_provider(
        &admin_app,
        &admin_token,
        "provider-completions-model-route",
        &format!("http://{address}"),
        "Completions Model Route Provider",
    )
    .await;
    create_provider(
        &admin_app,
        &admin_token,
        "provider-completions-created-id",
        "http://127.0.0.1:1",
        "Completions Created Id Provider",
    )
    .await;
    create_credential(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-completions-model-route",
        "cred-completions-model-route",
        "sk-completions-model-route",
    )
    .await;
    create_credential(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-completions-created-id",
        "cred-completions-created-id",
        "sk-completions-created-id",
    )
    .await;
    create_model(
        &admin_app,
        &admin_token,
        request_model,
        "provider-completions-model-route",
    )
    .await;
    create_routing_policy(
        &admin_app,
        &admin_token,
        "route-completions-by-request-model",
        "completions",
        request_model,
        200,
        "provider-completions-model-route",
    )
    .await;
    create_routing_policy(
        &admin_app,
        &admin_token,
        "route-completions-by-created-id",
        "completions",
        "cmpl_upstream",
        100,
        "provider-completions-created-id",
    )
    .await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/completions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"model\":\"{request_model}\",\"prompt\":\"keep request model\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "cmpl_upstream");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-completions-model-route")
    );
    support::assert_single_usage_record_and_decision_log(
        admin_app.clone(),
        &admin_token,
        request_model,
        "provider-completions-model-route",
        request_model,
    )
    .await;

    let billing_events = admin_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/billing/events")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(billing_events.status(), StatusCode::OK);
    let billing_json = read_json(billing_events).await;
    assert_eq!(billing_json.as_array().unwrap().len(), 1);
    assert_eq!(billing_json[0]["capability"], "completions");
    assert_eq!(billing_json[0]["route_key"], request_model);
    assert_eq!(billing_json[0]["usage_model"], request_model);
    assert_eq!(
        billing_json[0]["provider_id"],
        "provider-completions-model-route"
    );
    assert_eq!(billing_json[0]["accounting_mode"], "platform_credit");
    assert_eq!(billing_json[0]["channel_id"], "openai");
    assert!(billing_json[0]["api_key_hash"].as_str().unwrap().len() > 10);
    assert_eq!(billing_json[0]["reference_id"], "cmpl_upstream");
    assert!(billing_json[0]["compiled_routing_snapshot_id"].is_string());
}

#[tokio::test]
async fn stateful_moderations_route_keeps_request_model_for_billing_despite_response_id() {
    let tenant_id = "tenant-moderations-model-billing";
    let project_id = "project-moderations-model-billing";
    let request_model = "omni-moderation-latest";
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/moderations", post(upstream_moderations_handler))
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    create_channel(&admin_app, &admin_token).await;
    create_provider(
        &admin_app,
        &admin_token,
        "provider-moderations-model-route",
        &format!("http://{address}"),
        "Moderations Model Route Provider",
    )
    .await;
    create_provider(
        &admin_app,
        &admin_token,
        "provider-moderations-created-id",
        "http://127.0.0.1:1",
        "Moderations Created Id Provider",
    )
    .await;
    create_credential(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-moderations-model-route",
        "cred-moderations-model-route",
        "sk-moderations-model-route",
    )
    .await;
    create_credential(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-moderations-created-id",
        "cred-moderations-created-id",
        "sk-moderations-created-id",
    )
    .await;
    create_model(
        &admin_app,
        &admin_token,
        request_model,
        "provider-moderations-model-route",
    )
    .await;
    create_routing_policy(
        &admin_app,
        &admin_token,
        "route-moderations-by-request-model",
        "moderations",
        request_model,
        200,
        "provider-moderations-model-route",
    )
    .await;
    create_routing_policy(
        &admin_app,
        &admin_token,
        "route-moderations-by-created-id",
        "moderations",
        "modr_upstream",
        100,
        "provider-moderations-created-id",
    )
    .await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/moderations")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"model\":\"{request_model}\",\"input\":\"keep request model\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "modr_upstream");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-moderations-model-route")
    );
    support::assert_single_usage_record_and_decision_log(
        admin_app,
        &admin_token,
        request_model,
        "provider-moderations-model-route",
        request_model,
    )
    .await;
}

#[tokio::test]
async fn stateful_completions_route_inherits_api_key_group_accounting_mode_for_billing() {
    let tenant_id = "tenant-completions-group-byok";
    let project_id = "project-completions-group-byok";
    let request_model = "gpt-4.1-mini";
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/completions", post(upstream_completions_handler))
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());

    create_channel(&admin_app, &admin_token).await;
    create_provider(
        &admin_app,
        &admin_token,
        "provider-completions-byok-route",
        &format!("http://{address}"),
        "Completions BYOK Provider",
    )
    .await;
    create_credential(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-completions-byok-route",
        "cred-completions-byok-route",
        "sk-completions-byok-route",
    )
    .await;
    create_model(
        &admin_app,
        &admin_token,
        request_model,
        "provider-completions-byok-route",
    )
    .await;
    create_routing_policy(
        &admin_app,
        &admin_token,
        "route-completions-byok-group",
        "completions",
        request_model,
        200,
        "provider-completions-byok-route",
    )
    .await;

    let create_group = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/api-key-groups")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"project_id\":\"{project_id}\",\"environment\":\"live\",\"name\":\"BYOK Keys\",\"slug\":\"byok-keys\",\"default_accounting_mode\":\"byok\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_group.status(), StatusCode::CREATED);
    let create_group_json = read_json(create_group).await;
    let group_id = create_group_json["group_id"].as_str().unwrap().to_owned();

    let store = sdkwork_api_storage_sqlite::SqliteAdminStore::new(pool.clone());
    let api_key = persist_gateway_api_key_with_metadata(
        &store,
        PersistGatewayApiKeyInput {
            tenant_id,
            project_id,
            environment: "live",
            label: "BYOK test key",
            expires_at_ms: None,
            plaintext_key: None,
            notes: None,
            api_key_group_id: Some(&group_id),
        },
    )
    .await
    .unwrap()
    .plaintext;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/completions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"model\":\"{request_model}\",\"prompt\":\"bill this as byok\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let billing_events = admin_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/billing/events")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(billing_events.status(), StatusCode::OK);
    let billing_json = read_json(billing_events).await;
    assert_eq!(billing_json.as_array().unwrap().len(), 1);
    assert_eq!(billing_json[0]["api_key_group_id"], group_id);
    assert_eq!(billing_json[0]["accounting_mode"], "byok");
}

#[tokio::test]
async fn stateful_image_generation_records_image_count_in_billing_event() {
    let tenant_id = "tenant-images-billing-count";
    let project_id = "project-images-billing-count";
    let request_model = "gpt-image-1";
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/images/generations", post(upstream_images_handler))
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    create_channel(&admin_app, &admin_token).await;
    create_provider(
        &admin_app,
        &admin_token,
        "provider-images-billing-count",
        &format!("http://{address}"),
        "Image Billing Count Provider",
    )
    .await;
    create_credential(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-images-billing-count",
        "cred-images-billing-count",
        "sk-images-billing-count",
    )
    .await;
    create_model(
        &admin_app,
        &admin_token,
        request_model,
        "provider-images-billing-count",
    )
    .await;
    create_routing_policy(
        &admin_app,
        &admin_token,
        "route-images-billing-count",
        "images",
        request_model,
        200,
        "provider-images-billing-count",
    )
    .await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/images/generations")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"model\":\"{request_model}\",\"prompt\":\"generate two variants\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["data"].as_array().unwrap().len(), 2);
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-images-billing-count")
    );

    let billing_events = admin_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/billing/events")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(billing_events.status(), StatusCode::OK);
    let billing_json = read_json(billing_events).await;
    assert_eq!(billing_json.as_array().unwrap().len(), 1);
    assert_eq!(billing_json[0]["capability"], "images");
    assert_eq!(billing_json[0]["route_key"], request_model);
    assert_eq!(
        billing_json[0]["provider_id"],
        "provider-images-billing-count"
    );
    assert_eq!(billing_json[0]["reference_id"], "image_upstream_1");
    assert_eq!(billing_json[0]["image_count"], 2);
}

async fn create_channel(admin_app: &Router, admin_token: &str) {
    let response = admin_app
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
    assert_eq!(response.status(), StatusCode::CREATED);
}

async fn create_provider(
    admin_app: &Router,
    admin_token: &str,
    provider_id: &str,
    base_url: &str,
    display_name: &str,
) {
    let response = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "id": provider_id,
                        "channel_id": "openai",
                        "adapter_kind": "openai",
                        "base_url": base_url,
                        "display_name": display_name,
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}

async fn create_credential(
    admin_app: &Router,
    admin_token: &str,
    tenant_id: &str,
    provider_id: &str,
    key_reference: &str,
    secret_value: &str,
) {
    let response = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "tenant_id": tenant_id,
                        "provider_id": provider_id,
                        "key_reference": key_reference,
                        "secret_value": secret_value,
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}

async fn create_model(
    admin_app: &Router,
    admin_token: &str,
    external_name: &str,
    provider_id: &str,
) {
    let response = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "external_name": external_name,
                        "provider_id": provider_id,
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}

async fn create_routing_policy(
    admin_app: &Router,
    admin_token: &str,
    policy_id: &str,
    capability: &str,
    model_pattern: &str,
    priority: i64,
    provider_id: &str,
) {
    let response = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": policy_id,
                        "capability": capability,
                        "model_pattern": model_pattern,
                        "enabled": true,
                        "priority": priority,
                        "ordered_provider_ids": [provider_id],
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}

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

#[derive(Clone, Default)]
struct UpstreamCaptureState {
    authorization: Arc<Mutex<Option<String>>>,
}

async fn upstream_completions_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id": "cmpl_upstream",
            "object": "text_completion",
            "choices": [{"index": 0, "text": "relay completion", "finish_reason": "stop"}],
        })),
    )
}

async fn upstream_moderations_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id": "modr_upstream",
            "model": "omni-moderation-latest",
            "results": [{"flagged": false, "category_scores": {"violence": 0.0}}],
        })),
    )
}

async fn upstream_images_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "created": 1_744_000_000,
            "data": [
                { "b64_json": "image-a" },
                { "b64_json": "image-b" }
            ],
            "id": "image_upstream_1"
        })),
    )
}
