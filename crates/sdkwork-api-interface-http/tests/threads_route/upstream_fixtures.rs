use super::*;

pub(super) async fn upstream_threads_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);

    (StatusCode::OK, Json(thread_json("default")))
}

pub(super) async fn upstream_thread_retrieve_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);

    (StatusCode::OK, Json(thread_json("default")))
}

pub(super) async fn upstream_thread_update_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);

    (StatusCode::OK, Json(thread_json("next")))
}

pub(super) async fn upstream_thread_delete_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"thread_1",
            "object":"thread.deleted",
            "deleted":true
        })),
    )
}

pub(super) async fn upstream_thread_messages_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);

    (StatusCode::OK, Json(thread_message_json("msg_1")))
}

pub(super) async fn upstream_thread_messages_list_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "object":"list",
            "data":[thread_message_json("msg_1")],
            "first_id":"msg_1",
            "last_id":"msg_1",
            "has_more":false
        })),
    )
}

pub(super) async fn upstream_thread_message_retrieve_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);

    (StatusCode::OK, Json(thread_message_json("msg_1")))
}

pub(super) async fn upstream_thread_message_update_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);

    (StatusCode::OK, Json(thread_message_json("msg_1")))
}

pub(super) async fn upstream_thread_message_delete_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"msg_1",
            "object":"thread.message.deleted",
            "deleted":true
        })),
    )
}

fn thread_json(workspace: &str) -> Value {
    serde_json::json!({
        "id":"thread_1",
        "object":"thread",
        "metadata":{"workspace":workspace}
    })
}

fn thread_message_json(id: &str) -> Value {
    serde_json::json!({
        "id":id,
        "object":"thread.message",
        "thread_id":"thread_1",
        "assistant_id":null,
        "run_id":null,
        "role":"assistant",
        "status":"completed",
        "metadata":{"pinned":"true"},
        "content":[{
            "type":"text",
            "text":{
                "value":"hello",
                "annotations":[]
            }
        }]
    })
}
