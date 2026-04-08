use super::support::*;

#[tokio::test]
async fn adapter_posts_responses_to_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/responses", post(capture_responses_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request = sdkwork_api_contract_openai::responses::CreateResponseRequest {
        model: "gpt-4.1".to_owned(),
        input: Value::String("hello".to_owned()),
        stream: Some(false),
    };

    let response = adapter
        .responses("sk-upstream-openai", &request)
        .await
        .unwrap();

    assert_eq!(response["id"], "resp_upstream");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
}

#[tokio::test]
async fn adapter_posts_streaming_responses_to_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/responses", post(capture_responses_stream_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request = sdkwork_api_contract_openai::responses::CreateResponseRequest {
        model: "gpt-4.1".to_owned(),
        input: Value::String("hello".to_owned()),
        stream: Some(true),
    };

    let response = adapter
        .responses_stream("sk-upstream-openai", &request)
        .await
        .unwrap();

    assert_eq!(response.content_type(), "text/event-stream");
    let mut stream = response.into_body_stream();
    let mut bytes = Vec::new();
    while let Some(chunk) = stream.next().await {
        bytes.extend_from_slice(&chunk.unwrap());
    }

    let body = String::from_utf8(bytes).unwrap();
    assert!(body.contains("resp_upstream_stream"));
    assert!(body.contains("[DONE]"));
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
}

#[tokio::test]
async fn adapter_manages_conversations_on_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/conversations",
            get(capture_conversations_list_request).post(capture_conversations_request),
        )
        .route(
            "/v1/conversations/conv_1",
            get(capture_conversation_retrieve_request)
                .post(capture_conversation_update_request)
                .delete(capture_conversation_delete_request),
        )
        .route(
            "/v1/conversations/conv_1/items",
            get(capture_conversation_items_list_request).post(capture_conversation_items_request),
        )
        .route(
            "/v1/conversations/conv_1/items/item_1",
            get(capture_conversation_item_retrieve_request)
                .delete(capture_conversation_item_delete_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));

    let create_request =
        sdkwork_api_contract_openai::conversations::CreateConversationRequest::with_metadata(
            json!({"workspace":"default"}),
        );
    let create_response = adapter
        .conversations("sk-upstream-openai", &create_request)
        .await
        .unwrap();
    assert_eq!(create_response["id"], "conv_upstream");

    let list_response = adapter
        .list_conversations("sk-upstream-openai")
        .await
        .unwrap();
    assert_eq!(list_response["object"], "list");

    let retrieve_response = adapter
        .retrieve_conversation("sk-upstream-openai", "conv_1")
        .await
        .unwrap();
    assert_eq!(retrieve_response["id"], "conv_1");

    let update_request =
        sdkwork_api_contract_openai::conversations::UpdateConversationRequest::with_metadata(
            json!({"workspace":"next"}),
        );
    let update_response = adapter
        .update_conversation("sk-upstream-openai", "conv_1", &update_request)
        .await
        .unwrap();
    assert_eq!(update_response["metadata"]["workspace"], "next");

    let delete_response = adapter
        .delete_conversation("sk-upstream-openai", "conv_1")
        .await
        .unwrap();
    assert_eq!(delete_response["deleted"], true);

    let items_request =
        sdkwork_api_contract_openai::conversations::CreateConversationItemsRequest::new(vec![
            json!({"id":"item_1","type":"message","role":"user","content":[{"type":"input_text","text":"hello"}]}),
        ]);
    let create_items_response = adapter
        .create_conversation_items("sk-upstream-openai", "conv_1", &items_request)
        .await
        .unwrap();
    assert_eq!(create_items_response["data"][0]["id"], "item_1");

    let list_items_response = adapter
        .list_conversation_items("sk-upstream-openai", "conv_1")
        .await
        .unwrap();
    assert_eq!(list_items_response["data"][0]["id"], "item_1");

    let retrieve_item_response = adapter
        .retrieve_conversation_item("sk-upstream-openai", "conv_1", "item_1")
        .await
        .unwrap();
    assert_eq!(retrieve_item_response["id"], "item_1");

    let delete_item_response = adapter
        .delete_conversation_item("sk-upstream-openai", "conv_1", "item_1")
        .await
        .unwrap();
    assert_eq!(delete_item_response["deleted"], true);
}

#[tokio::test]
async fn adapter_retrieves_response_from_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/responses/resp_1",
            get(capture_response_retrieve_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let response = adapter
        .retrieve_response("sk-upstream-openai", "resp_1")
        .await
        .unwrap();

    assert_eq!(response["id"], "resp_1");
}

#[tokio::test]
async fn adapter_lists_response_input_items_from_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/responses/resp_1/input_items",
            get(capture_response_input_items_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let response = adapter
        .list_response_input_items("sk-upstream-openai", "resp_1")
        .await
        .unwrap();

    assert_eq!(response["data"][0]["id"], "item_1");
}

#[tokio::test]
async fn adapter_deletes_response_on_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/responses/resp_1",
            delete(capture_response_delete_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let response = adapter
        .delete_response("sk-upstream-openai", "resp_1")
        .await
        .unwrap();

    assert_eq!(response["deleted"], true);
}

#[tokio::test]
async fn adapter_counts_response_input_tokens_on_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/responses/input_tokens",
            post(capture_response_input_tokens_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request = sdkwork_api_contract_openai::responses::CountResponseInputTokensRequest::new(
        "gpt-4.1",
        json!("hello"),
    );

    let response = adapter
        .count_response_input_tokens("sk-upstream-openai", &request)
        .await
        .unwrap();

    assert_eq!(response["object"], "response.input_tokens");
    assert_eq!(response["input_tokens"], 21);
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
}

#[tokio::test]
async fn adapter_cancels_response_on_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/responses/resp_1/cancel",
            post(capture_response_cancel_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let response = adapter
        .cancel_response("sk-upstream-openai", "resp_1")
        .await
        .unwrap();

    assert_eq!(response["status"], "cancelled");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
}

#[tokio::test]
async fn adapter_compacts_response_on_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/responses/compact",
            post(capture_response_compact_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request = sdkwork_api_contract_openai::responses::CompactResponseRequest::new(
        "gpt-4.1",
        json!("hello"),
    );

    let response = adapter
        .compact_response("sk-upstream-openai", &request)
        .await
        .unwrap();

    assert_eq!(response["object"], "response.compaction");
    assert_eq!(response["model"], "gpt-4.1");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
}


async fn capture_conversations_request(
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
            "id":"conv_upstream",
            "object":"conversation",
            "metadata":{"workspace":"default"}
        })),
    )
}

async fn capture_conversations_list_request(
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
            "data":[{"id":"conv_1","object":"conversation"}]
        })),
    )
}

async fn capture_conversation_retrieve_request(
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
            "id":"conv_1",
            "object":"conversation",
            "metadata":{"workspace":"default"}
        })),
    )
}

async fn capture_conversation_update_request(
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
            "id":"conv_1",
            "object":"conversation",
            "metadata":{"workspace":"next"}
        })),
    )
}

async fn capture_conversation_delete_request(
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
            "id":"conv_1",
            "object":"conversation.deleted",
            "deleted":true
        })),
    )
}

async fn capture_conversation_items_request(
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
            "object":"list",
            "data":[{
                "id":"item_1",
                "object":"conversation.item",
                "type":"message",
                "role":"assistant",
                "content":[{"type":"output_text","text":"hello"}]
            }]
        })),
    )
}

async fn capture_conversation_items_list_request(
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
                "id":"item_1",
                "object":"conversation.item",
                "type":"message",
                "role":"assistant",
                "content":[{"type":"output_text","text":"hello"}]
            }]
        })),
    )
}

async fn capture_conversation_item_retrieve_request(
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
            "id":"item_1",
            "object":"conversation.item",
            "type":"message",
            "role":"assistant",
            "content":[{"type":"output_text","text":"hello"}]
        })),
    )
}

async fn capture_conversation_item_delete_request(
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
            "id":"item_1",
            "object":"conversation.item.deleted",
            "deleted":true
        })),
    )
}


async fn capture_responses_request(
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
            "id":"resp_upstream",
            "object":"response",
            "model":"gpt-4.1",
            "output":[]
        })),
    )
}

async fn capture_responses_stream_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> axum::response::Response {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    *state.body.lock().unwrap() = Some(body);

    (
        [(axum::http::header::CONTENT_TYPE, "text/event-stream")],
        "data: {\"id\":\"resp_upstream_stream\",\"type\":\"response.output_text.delta\"}\n\ndata: [DONE]\n\n",
    )
        .into_response()
}

async fn capture_response_retrieve_request(
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
            "id":"resp_1",
            "object":"response",
            "model":"gpt-4.1",
            "output":[]
        })),
    )
}

async fn capture_response_input_items_request(
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
                "id":"item_1",
                "object":"response.input_item",
                "type":"message"
            }]
        })),
    )
}

async fn capture_response_delete_request(
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
            "id":"resp_1",
            "object":"response.deleted",
            "deleted":true
        })),
    )
}

async fn capture_response_input_tokens_request(
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
            "object":"response.input_tokens",
            "input_tokens":21
        })),
    )
}

async fn capture_response_cancel_request(
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
            "id":"resp_1",
            "object":"response",
            "model":"gpt-4.1",
            "status":"cancelled",
            "output":[]
        })),
    )
}

async fn capture_response_compact_request(
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
            "id":"resp_cmp_1",
            "object":"response.compaction",
            "model":"gpt-4.1"
        })),
    )
}

