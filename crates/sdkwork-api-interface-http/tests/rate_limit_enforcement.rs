use axum::body::{to_bytes, Body};
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use axum::routing::post;
use axum::{Json, Router};
use serde_json::Value;
use sqlx::SqlitePool;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tower::ServiceExt;

mod support;

#[derive(Clone, Default)]
struct UpstreamCaptureState {
    request_count: Arc<AtomicUsize>,
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

fn header_text<'a>(response: &'a Response, name: &str) -> &'a str {
    response
        .headers()
        .get(name)
        .and_then(|value| value.to_str().ok())
        .unwrap()
}

async fn upstream_chat_handler(
    State(state): State<UpstreamCaptureState>,
) -> (StatusCode, Json<Value>) {
    state.request_count.fetch_add(1, Ordering::SeqCst);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"chatcmpl_rate_limit",
            "object":"chat.completion",
            "model":"gpt-4.1",
            "choices":[],
            "usage":{
                "prompt_tokens":12,
                "completion_tokens":8,
                "total_tokens":20
            }
        })),
    )
}

async fn configure_openai_chat_provider(
    admin_app: &Router,
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
    assert_eq!(channel.status(), StatusCode::CREATED);

    let provider = admin_app
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
                        "display_name": "Rate Limit Chat Provider"
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
                    serde_json::json!({
                        "tenant_id": tenant_id,
                        "provider_id": provider_id,
                        "key_reference": format!("cred-{provider_id}"),
                        "secret_value": format!("sk-{provider_id}")
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
                    serde_json::json!({
                        "external_name": "gpt-4.1",
                        "provider_id": provider_id
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(model.status(), StatusCode::CREATED);

    let policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": "route-chat-rate-limit",
                        "capability": "chat_completion",
                        "model_pattern": "gpt-4.1",
                        "enabled": true,
                        "priority": 300,
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

#[tokio::test]
async fn gateway_chat_completions_returns_429_when_rate_limit_is_exhausted() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/chat/completions", post(upstream_chat_handler))
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, "tenant-1", "project-1").await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(&pool, admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    configure_openai_chat_provider(
        &admin_app,
        &admin_token,
        "tenant-1",
        "provider-chat-rate-limit",
        &format!("http://{address}"),
    )
    .await;

    let create_policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/gateway/rate-limit-policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"policy_id\":\"rate-project-1-chat\",\"project_id\":\"project-1\",\"route_key\":\"/v1/chat/completions\",\"requests_per_window\":1,\"window_seconds\":60,\"enabled\":true}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_policy.status(), StatusCode::CREATED);

    let first = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"hi\"}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(
        header_text(&first, "x-ratelimit-policy"),
        "rate-project-1-chat"
    );
    assert_eq!(header_text(&first, "x-ratelimit-limit"), "1");
    assert_eq!(header_text(&first, "x-ratelimit-remaining"), "0");
    let success_reset = header_text(&first, "x-ratelimit-reset")
        .parse::<u64>()
        .unwrap();
    assert!(success_reset <= 60);
    assert!(success_reset > 0);
    let first_json = read_json(first).await;
    assert_eq!(first_json["id"], "chatcmpl_rate_limit");

    let second = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"hi again\"}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(second.status(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(
        header_text(&second, "x-ratelimit-policy"),
        "rate-project-1-chat"
    );
    assert_eq!(header_text(&second, "x-ratelimit-limit"), "1");
    assert_eq!(header_text(&second, "x-ratelimit-remaining"), "0");
    let retry_after = header_text(&second, "retry-after").parse::<u64>().unwrap();
    let rejected_reset = header_text(&second, "x-ratelimit-reset")
        .parse::<u64>()
        .unwrap();
    assert!(retry_after <= 60);
    assert!(retry_after > 0);
    assert_eq!(rejected_reset, retry_after);
    let json = read_json(second).await;
    assert_eq!(json["error"]["code"], "rate_limit_exceeded");
    assert_eq!(upstream_state.request_count.load(Ordering::SeqCst), 1);
}
