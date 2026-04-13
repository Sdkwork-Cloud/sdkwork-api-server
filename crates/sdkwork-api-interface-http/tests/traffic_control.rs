use axum::body::{to_bytes, Body};
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use sdkwork_api_app_credential::CredentialSecretManager;
use sdkwork_api_app_rate_limit::{CommercialAdmissionPolicy, InMemoryGatewayTrafficController};
use sdkwork_api_interface_http::GatewayApiState;
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use serde_json::{json, Value};
use sqlx::SqlitePool;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio::sync::Notify;
use tokio::time::{sleep, timeout, Duration};
use tower::ServiceExt;

mod support;

#[derive(Clone)]
struct SlowUpstreamState {
    hit_count: Arc<AtomicUsize>,
    started: Arc<Notify>,
    release: Arc<Notify>,
}

impl SlowUpstreamState {
    fn new() -> Self {
        Self {
            hit_count: Arc::new(AtomicUsize::new(0)),
            started: Arc::new(Notify::new()),
            release: Arc::new(Notify::new()),
        }
    }
}

async fn slow_chat_completion_handler(State(state): State<SlowUpstreamState>) -> Json<Value> {
    state.hit_count.fetch_add(1, Ordering::SeqCst);
    state.started.notify_waiters();
    state.release.notified().await;
    Json(json!({
        "id": "chatcmpl_traffic_control",
        "object": "chat.completion",
        "created": 1_700_000_000u64,
        "model": "gpt-4.1",
        "choices": [
            {
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "traffic-control-ok"
                },
                "finish_reason": "stop"
            }
        ],
        "usage": {
            "prompt_tokens": 24,
            "completion_tokens": 12,
            "total_tokens": 36
        }
    }))
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

async fn spawn_slow_openai_upstream() -> (String, SlowUpstreamState) {
    let state = SlowUpstreamState::new();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/chat/completions", post(slow_chat_completion_handler))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    (format!("http://{address}"), state)
}

async fn wait_for_upstream_hit_or_request_completion(
    state: &SlowUpstreamState,
    request_handle: &tokio::task::JoinHandle<axum::response::Response>,
) {
    timeout(Duration::from_secs(10), async {
        loop {
            if state.hit_count.load(Ordering::SeqCst) > 0 || request_handle.is_finished() {
                break;
            }
            sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("first request should either hit upstream or fail fast");
}

async fn configure_chat_provider(
    admin_app: Router,
    admin_token: &str,
    tenant_id: &str,
    provider_id: &str,
    base_url: &str,
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
    assert!(matches!(
        channel.status(),
        StatusCode::CREATED | StatusCode::CONFLICT
    ));

    let provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "id": provider_id,
                        "channel_id": "openai",
                        "adapter_kind": "openai",
                        "base_url": base_url,
                        "display_name": "Traffic Control Provider"
                    })
                    .to_string(),
                ))
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
                    json!({
                        "tenant_id": tenant_id,
                        "provider_id": provider_id,
                        "key_reference": format!("cred-{provider_id}"),
                        "secret_value": "sk-upstream-traffic"
                    })
                    .to_string(),
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
                .body(Body::from(
                    json!({
                        "external_name": "gpt-4.1",
                        "provider_id": provider_id,
                        "capabilities": ["chat_completions"],
                        "streaming": false
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(model.status(), StatusCode::CREATED);

    let routing_policy = admin_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "policy_id": format!("route-{provider_id}"),
                        "capability": "chat_completion",
                        "model_pattern": "gpt-4.1",
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
    assert_eq!(routing_policy.status(), StatusCode::CREATED);
}

fn gateway_router_with_controller(
    pool: SqlitePool,
    controller: Arc<InMemoryGatewayTrafficController>,
) -> Router {
    let store = Arc::new(SqliteAdminStore::new(pool));
    sdkwork_api_interface_http::gateway_router_with_state(
        GatewayApiState::with_store_secret_manager_and_traffic_controller(
            store,
            CredentialSecretManager::database_encrypted("local-dev-master-key"),
            controller,
        ),
    )
}

fn chat_request(api_key: &str, content: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/v1/chat/completions")
        .header("authorization", format!("Bearer {api_key}"))
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "model": "gpt-4.1",
                "messages": [
                    {
                        "role": "user",
                        "content": content
                    }
                ]
            })
            .to_string(),
        ))
        .unwrap()
}

#[tokio::test]
async fn gateway_rejects_second_inflight_request_when_project_concurrency_cap_is_exhausted() {
    let tenant_id = "tenant-traffic-project";
    let project_id = "project-traffic-project";
    let (base_url, upstream_state) = spawn_slow_openai_upstream().await;
    let pool = memory_pool().await;
    let api_key_a = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let api_key_b = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    configure_chat_provider(
        admin_app,
        &admin_token,
        tenant_id,
        "provider-traffic-project",
        &base_url,
    )
    .await;

    let controller = Arc::new(InMemoryGatewayTrafficController::new());
    controller.replace_policies(vec![CommercialAdmissionPolicy::new(
        "project-concurrency-cap",
        project_id,
    )
    .with_project_concurrency_limit(1)]);

    let gateway_app = gateway_router_with_controller(pool, controller);

    let first = tokio::spawn({
        let gateway_app = gateway_app.clone();
        let request = chat_request(&api_key_a, "first-project-request");
        async move { gateway_app.oneshot(request).await.unwrap() }
    });

    wait_for_upstream_hit_or_request_completion(&upstream_state, &first).await;
    if upstream_state.hit_count.load(Ordering::SeqCst) == 0 {
        let first = first.await.unwrap();
        panic!(
            "first project-scoped request finished before upstream dispatch with status {}",
            first.status()
        );
    }

    let second = timeout(
        Duration::from_secs(10),
        gateway_app.oneshot(chat_request(&api_key_b, "second-project-request")),
    )
    .await
    .expect("second project-scoped request should not hang")
    .unwrap();

    assert_eq!(second.status(), StatusCode::TOO_MANY_REQUESTS);
    let json = read_json(second).await;
    assert_eq!(json["error"]["code"], "gateway_concurrency_exceeded");
    assert_eq!(
        upstream_state.hit_count.load(Ordering::SeqCst),
        1,
        "the second request should be rejected before reaching upstream"
    );

    upstream_state.release.notify_waiters();
    let first = first.await.unwrap();
    assert_eq!(first.status(), StatusCode::OK);
}

#[tokio::test]
async fn gateway_rejects_second_inflight_request_when_api_key_concurrency_cap_is_exhausted() {
    let tenant_id = "tenant-traffic-api-key";
    let project_id = "project-traffic-api-key";
    let (base_url, upstream_state) = spawn_slow_openai_upstream().await;
    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let hashed_key = sdkwork_api_app_identity::hash_gateway_api_key(&api_key);
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    configure_chat_provider(
        admin_app,
        &admin_token,
        tenant_id,
        "provider-traffic-api-key",
        &base_url,
    )
    .await;

    let controller = Arc::new(InMemoryGatewayTrafficController::new());
    controller.replace_policies(vec![CommercialAdmissionPolicy::new(
        "api-key-concurrency-cap",
        project_id,
    )
    .with_api_key_hash(&hashed_key)
    .with_api_key_concurrency_limit(1)]);

    let gateway_app = gateway_router_with_controller(pool, controller);

    let first = tokio::spawn({
        let gateway_app = gateway_app.clone();
        let request = chat_request(&api_key, "first-key-request");
        async move { gateway_app.oneshot(request).await.unwrap() }
    });

    wait_for_upstream_hit_or_request_completion(&upstream_state, &first).await;
    if upstream_state.hit_count.load(Ordering::SeqCst) == 0 {
        let first = first.await.unwrap();
        panic!(
            "first api-key-scoped request finished before upstream dispatch with status {}",
            first.status()
        );
    }

    let second = timeout(
        Duration::from_secs(10),
        gateway_app.oneshot(chat_request(&api_key, "second-key-request")),
    )
    .await
    .expect("second api-key-scoped request should not hang")
    .unwrap();

    assert_eq!(second.status(), StatusCode::TOO_MANY_REQUESTS);
    let json = read_json(second).await;
    assert_eq!(json["error"]["code"], "gateway_concurrency_exceeded");
    assert_eq!(
        upstream_state.hit_count.load(Ordering::SeqCst),
        1,
        "the second request should be rejected before reaching upstream"
    );

    upstream_state.release.notify_waiters();
    let first = first.await.unwrap();
    assert_eq!(first.status(), StatusCode::OK);
}

#[tokio::test]
async fn gateway_rejects_provider_execution_when_provider_backpressure_is_exhausted() {
    let tenant_id = "tenant-traffic-provider";
    let project_id = "project-traffic-provider";
    let provider_id = "provider-traffic-backpressure";
    let (base_url, upstream_state) = spawn_slow_openai_upstream().await;
    let pool = memory_pool().await;
    let api_key_a = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let api_key_b = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    configure_chat_provider(admin_app, &admin_token, tenant_id, provider_id, &base_url).await;

    let controller = Arc::new(InMemoryGatewayTrafficController::new());
    controller.replace_policies(vec![CommercialAdmissionPolicy::new(
        "provider-backpressure-cap",
        project_id,
    )
    .with_provider_id(provider_id)
    .with_provider_concurrency_limit(1)]);

    let gateway_app = gateway_router_with_controller(pool, controller);

    let first = tokio::spawn({
        let gateway_app = gateway_app.clone();
        let request = chat_request(&api_key_a, "first-provider-request");
        async move { gateway_app.oneshot(request).await.unwrap() }
    });

    wait_for_upstream_hit_or_request_completion(&upstream_state, &first).await;
    if upstream_state.hit_count.load(Ordering::SeqCst) == 0 {
        let first = first.await.unwrap();
        panic!(
            "first provider-scoped request finished before upstream dispatch with status {}",
            first.status()
        );
    }

    let second = timeout(
        Duration::from_secs(10),
        gateway_app.oneshot(chat_request(&api_key_b, "second-provider-request")),
    )
    .await
    .expect("second provider-scoped request should not hang")
    .unwrap();

    assert_eq!(second.status(), StatusCode::SERVICE_UNAVAILABLE);
    let json = read_json(second).await;
    assert_eq!(json["error"]["code"], "provider_backpressure_exceeded");
    assert_eq!(
        upstream_state.hit_count.load(Ordering::SeqCst),
        1,
        "the saturated provider must not receive a second concurrent request"
    );

    upstream_state.release.notify_waiters();
    let first = first.await.unwrap();
    assert_eq!(first.status(), StatusCode::OK);
}
