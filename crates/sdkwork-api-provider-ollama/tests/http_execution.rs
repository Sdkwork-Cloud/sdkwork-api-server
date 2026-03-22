use std::sync::{Arc, Mutex};

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::post,
    Json, Router,
};
use serde_json::{json, Value};
use tokio::net::TcpListener;

#[derive(Clone, Default)]
struct CaptureState {
    authorization: Arc<Mutex<Option<String>>>,
    body: Arc<Mutex<Option<Value>>>,
}

#[tokio::test]
async fn adapter_posts_authorized_json_to_ollama_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/chat/completions", post(capture_chat_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_ollama::OllamaProviderAdapter::new(format!("http://{address}"));
    let request = sdkwork_api_contract_openai::chat_completions::CreateChatCompletionRequest {
        model: "llama3.2".to_owned(),
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

    let response = adapter
        .chat_completions("ollama-local-token", &request)
        .await
        .unwrap();

    assert_eq!(response["object"], "chat.completion");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer ollama-local-token")
    );
    assert_eq!(
        state.body.lock().unwrap().as_ref().unwrap()["model"],
        "llama3.2"
    );
}

async fn capture_chat_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    *state.body.lock().unwrap() = Some(body);

    (
        StatusCode::OK,
        Json(json!({
            "id":"chatcmpl_upstream",
            "object":"chat.completion",
            "model":"llama3.2",
            "choices":[]
        })),
    )
}
