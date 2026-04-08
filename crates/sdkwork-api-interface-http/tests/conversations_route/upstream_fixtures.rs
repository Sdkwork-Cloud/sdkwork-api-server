use super::*;

pub(super) async fn upstream_conversations_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"conv_upstream",
            "object":"conversation",
            "metadata":{"workspace":"default"}
        })),
    )
}

pub(super) async fn upstream_conversations_list_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "object":"list",
            "data":[{"id":"conv_1","object":"conversation"}]
        })),
    )
}

pub(super) async fn upstream_conversation_retrieve_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"conv_1",
            "object":"conversation",
            "metadata":{"workspace":"default"}
        })),
    )
}

pub(super) async fn upstream_conversation_update_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"conv_1",
            "object":"conversation",
            "metadata":{"workspace":"next"}
        })),
    )
}

pub(super) async fn upstream_conversation_delete_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"conv_1",
            "object":"conversation.deleted",
            "deleted":true
        })),
    )
}

pub(super) async fn upstream_conversation_items_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "object":"list",
            "data":[conversation_item_json("item_1")]
        })),
    )
}

pub(super) async fn upstream_conversation_items_list_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    upstream_conversation_items_handler(State(state), headers).await
}

pub(super) async fn upstream_conversation_item_retrieve_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);

    (StatusCode::OK, Json(conversation_item_json("item_1")))
}

pub(super) async fn upstream_conversation_item_delete_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"item_1",
            "object":"conversation.item.deleted",
            "deleted":true
        })),
    )
}

fn conversation_item_json(id: &str) -> Value {
    serde_json::json!({
        "id":id,
        "object":"conversation.item",
        "type":"message",
        "role":"assistant",
        "content":[{"type":"output_text","text":"hello"}]
    })
}
