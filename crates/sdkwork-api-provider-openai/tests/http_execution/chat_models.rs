use super::support::*;

#[tokio::test]
async fn adapter_posts_authorized_json_to_openai_compatible_upstream() {
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
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
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

    let response = adapter
        .chat_completions("sk-upstream-openai", &request)
        .await
        .unwrap();

    assert_eq!(response["object"], "chat.completion");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert_eq!(
        state.body.lock().unwrap().as_ref().unwrap()["model"],
        "gpt-4.1"
    );
}

#[tokio::test]
async fn adapter_surfaces_retry_after_seconds_on_rate_limited_error() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new().route(
        "/v1/chat/completions",
        post(capture_chat_request_rate_limited_with_retry_after),
    );

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
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
        .expect_err("429 response should surface as error");
    let http_error = error
        .downcast_ref::<sdkwork_api_provider_core::ProviderHttpError>()
        .expect("provider http error");

    assert_eq!(http_error.status(), Some(reqwest::StatusCode::TOO_MANY_REQUESTS));
    assert_eq!(http_error.retry_after_secs(), Some(1));
}

#[tokio::test]
async fn adapter_surfaces_retry_after_http_date_on_rate_limited_error() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new().route(
        "/v1/chat/completions",
        post(capture_chat_request_rate_limited_with_http_date_retry_after),
    );

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
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
        .expect_err("429 response should surface as error");
    let http_error = error
        .downcast_ref::<sdkwork_api_provider_core::ProviderHttpError>()
        .expect("provider http error");

    assert_eq!(http_error.status(), Some(reqwest::StatusCode::TOO_MANY_REQUESTS));
    assert!(
        http_error.retry_after_secs().unwrap_or_default() > 60 * 60 * 24,
        "expected http-date retry-after to parse into a large positive delay, got {:?}",
        http_error.retry_after_secs()
    );
}

#[tokio::test]
async fn adapter_lists_chat_completions_from_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/chat/completions", get(capture_chat_list_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let response = adapter
        .list_chat_completions("sk-upstream-openai")
        .await
        .unwrap();

    assert_eq!(response["object"], "list");
    assert_eq!(response["data"][0]["id"], "chatcmpl_1");
}

#[tokio::test]
async fn adapter_retrieves_chat_completion_from_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/chat/completions/chatcmpl_1",
            get(capture_chat_retrieve_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let response = adapter
        .retrieve_chat_completion("sk-upstream-openai", "chatcmpl_1")
        .await
        .unwrap();

    assert_eq!(response["id"], "chatcmpl_1");
}

#[tokio::test]
async fn adapter_deletes_model_on_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/models/ft:gpt-4.1:sdkwork",
            delete(capture_model_delete_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let response = adapter
        .delete_model("sk-upstream-openai", "ft:gpt-4.1:sdkwork")
        .await
        .unwrap();

    assert_eq!(response["id"], "ft:gpt-4.1:sdkwork");
    assert_eq!(response["deleted"], true);
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
}

#[tokio::test]
async fn adapter_lists_models_from_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/models", get(capture_models_list_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let response = adapter.list_models("sk-upstream-openai").await.unwrap();

    assert_eq!(response["object"], "list");
    assert_eq!(response["data"][0]["id"], "gpt-4.1");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
}

#[tokio::test]
async fn adapter_retrieves_model_from_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/models/gpt-4.1", get(capture_model_retrieve_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let response = adapter
        .retrieve_model("sk-upstream-openai", "gpt-4.1")
        .await
        .unwrap();

    assert_eq!(response["id"], "gpt-4.1");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
}

#[tokio::test]
async fn adapter_updates_chat_completion_on_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/chat/completions/chatcmpl_1",
            post(capture_chat_update_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request = sdkwork_api_contract_openai::chat_completions::UpdateChatCompletionRequest::new(
        json!({"tier":"gold"}),
    );
    let response = adapter
        .update_chat_completion("sk-upstream-openai", "chatcmpl_1", &request)
        .await
        .unwrap();

    assert_eq!(response["metadata"]["tier"], "gold");
}

#[tokio::test]
async fn adapter_deletes_chat_completion_on_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/chat/completions/chatcmpl_1",
            delete(capture_chat_delete_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let response = adapter
        .delete_chat_completion("sk-upstream-openai", "chatcmpl_1")
        .await
        .unwrap();

    assert_eq!(response["deleted"], true);
}

#[tokio::test]
async fn adapter_lists_chat_completion_messages_from_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/chat/completions/chatcmpl_1/messages",
            get(capture_chat_messages_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let response = adapter
        .list_chat_completion_messages("sk-upstream-openai", "chatcmpl_1")
        .await
        .unwrap();

    assert_eq!(response["data"][0]["id"], "msg_1");
}

#[tokio::test]
async fn adapter_posts_legacy_completions_to_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/completions", post(capture_completion_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request = sdkwork_api_contract_openai::completions::CreateCompletionRequest::new(
        "gpt-3.5-turbo-instruct",
        "hello",
    );

    let response = adapter
        .completions("sk-upstream-openai", &request)
        .await
        .unwrap();

    assert_eq!(response["object"], "text_completion");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert_eq!(
        state.body.lock().unwrap().as_ref().unwrap()["prompt"],
        "hello"
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
            "model":"gpt-4.1",
            "choices":[]
        })),
    )
}

async fn capture_chat_request_rate_limited_with_retry_after() -> impl IntoResponse {
    (
        StatusCode::TOO_MANY_REQUESTS,
        [("retry-after", "1")],
        Json(json!({
            "error":{
                "message":"rate limited by upstream",
                "type":"rate_limit_error",
                "code":"too_many_requests"
            }
        })),
    )
}

async fn capture_chat_request_rate_limited_with_http_date_retry_after() -> impl IntoResponse {
    (
        StatusCode::TOO_MANY_REQUESTS,
        [("retry-after", "Thu, 01 Jan 2099 00:00:00 GMT")],
        Json(json!({
            "error":{
                "message":"rate limited by upstream with http-date retry-after",
                "type":"rate_limit_error",
                "code":"too_many_requests"
            }
        })),
    )
}

async fn capture_chat_list_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    (
        StatusCode::OK,
        Json(json!({
            "object":"list",
            "data":[{
                "id":"chatcmpl_1",
                "object":"chat.completion",
                "model":"gpt-4.1",
                "choices":[]
            }]
        })),
    )
}

async fn capture_chat_retrieve_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    (
        StatusCode::OK,
        Json(json!({
            "id":"chatcmpl_1",
            "object":"chat.completion",
            "model":"gpt-4.1",
            "choices":[]
        })),
    )
}

async fn capture_chat_update_request(
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
            "id":"chatcmpl_1",
            "object":"chat.completion",
            "model":"gpt-4.1",
            "metadata":{"tier":"gold"},
            "choices":[]
        })),
    )
}

async fn capture_chat_delete_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    *state.raw_body.lock().unwrap() = Some(body.to_vec());

    (
        StatusCode::OK,
        Json(json!({
            "id":"chatcmpl_1",
            "object":"chat.completion.deleted",
            "deleted":true
        })),
    )
}

async fn capture_model_delete_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    *state.raw_body.lock().unwrap() = Some(body.to_vec());

    (
        StatusCode::OK,
        Json(json!({
            "id":"ft:gpt-4.1:sdkwork",
            "object":"model",
            "deleted":true
        })),
    )
}

async fn capture_models_list_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    (
        StatusCode::OK,
        Json(json!({
            "object":"list",
            "data":[{
                "id":"gpt-4.1",
                "object":"model",
                "created":1710000000,
                "owned_by":"openai"
            }]
        })),
    )
}

async fn capture_model_retrieve_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    (
        StatusCode::OK,
        Json(json!({
            "id":"gpt-4.1",
            "object":"model",
            "created":1710000000,
            "owned_by":"openai"
        })),
    )
}

async fn capture_chat_messages_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    (
        StatusCode::OK,
        Json(json!({
            "object":"list",
            "data":[{
                "id":"msg_1",
                "object":"chat.completion.message",
                "role":"assistant",
                "content":"hello"
            }]
        })),
    )
}


async fn capture_completion_request(
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
            "id":"cmpl_upstream",
            "object":"text_completion",
            "choices":[{"index":0,"text":"hello from upstream","finish_reason":"stop"}]
        })),
    )
}

