use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{json, Value};
use tokio::net::TcpListener;
use tokio::time::sleep;

#[derive(Clone, Default)]
struct TimeoutState {
    request_count: Arc<Mutex<usize>>,
}

#[tokio::test]
async fn adapter_classifies_request_timeouts_as_retryable_failures() {
    let state = TimeoutState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = Router::new()
        .route("/v1/chat/completions", post(delayed_chat_handler))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter = sdkwork_api_provider_openai::OpenAiProviderAdapter::with_http_config(
        format!("http://{address}"),
        sdkwork_api_provider_openai::OpenAiProviderHttpConfig::default()
            .with_connect_timeout(Duration::from_millis(50))
            .with_request_timeout(Duration::from_millis(50)),
    );
    let request = sdkwork_api_contract_openai::chat_completions::CreateChatCompletionRequest {
        model: "gpt-4.1".to_owned(),
        messages: vec![
            sdkwork_api_contract_openai::chat_completions::ChatMessageInput {
                role: "user".to_owned(),
                content: Value::String("hello".to_owned()),
                extra: serde_json::Map::new(),
            },
        ],
        stream: Some(false),
        extra: serde_json::Map::new(),
    };

    let error = adapter
        .chat_completions("sk-upstream-openai", &request)
        .await
        .expect_err("delayed upstream should time out");
    let failure = sdkwork_api_provider_openai::classify_upstream_error(&error);

    assert_eq!(*state.request_count.lock().unwrap(), 1);
    assert!(failure.retryable);
    assert_eq!(
        failure.category,
        sdkwork_api_provider_openai::UpstreamFailureCategory::Timeout
    );
    assert!(failure.message.contains("timeout"));
}

#[tokio::test]
async fn adapter_classifies_connect_failures_as_retryable_failures() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    drop(listener);

    let adapter = sdkwork_api_provider_openai::OpenAiProviderAdapter::with_http_config(
        format!("http://{address}"),
        sdkwork_api_provider_openai::OpenAiProviderHttpConfig::default()
            .with_connect_timeout(Duration::from_millis(50))
            .with_request_timeout(Duration::from_millis(100)),
    );

    let error = adapter
        .list_models("sk-upstream-openai")
        .await
        .expect_err("closed port should fail to connect");
    let failure = sdkwork_api_provider_openai::classify_upstream_error(&error);

    assert!(failure.retryable);
    assert_eq!(
        failure.category,
        sdkwork_api_provider_openai::UpstreamFailureCategory::Connect
    );
    assert!(failure.message.contains("connect"));
}

#[tokio::test]
async fn adapter_classifies_retryable_http_status_failures() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = Router::new().route("/v1/models", get(unavailable_models_handler));

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let error = adapter
        .list_models("sk-upstream-openai")
        .await
        .expect_err("502 should surface as retryable upstream failure");
    let failure = sdkwork_api_provider_openai::classify_upstream_error(&error);

    assert!(failure.retryable);
    assert_eq!(
        failure.category,
        sdkwork_api_provider_openai::UpstreamFailureCategory::HttpStatus
    );
    assert_eq!(failure.status_code, Some(502));
}

async fn delayed_chat_handler(
    State(state): State<TimeoutState>,
) -> (axum::http::StatusCode, Json<Value>) {
    *state.request_count.lock().unwrap() += 1;
    sleep(Duration::from_millis(250)).await;

    (
        axum::http::StatusCode::OK,
        Json(json!({
            "id": "chatcmpl_timeout",
            "object": "chat.completion",
            "model": "gpt-4.1",
            "choices": []
        })),
    )
}

async fn unavailable_models_handler() -> (axum::http::StatusCode, Json<Value>) {
    (
        axum::http::StatusCode::BAD_GATEWAY,
        Json(json!({
            "error": {
                "message": "temporary upstream outage"
            }
        })),
    )
}
