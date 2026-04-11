use super::*;

fn capture_upstream_request(
    state: &UpstreamCaptureState,
    headers: &axum::http::HeaderMap,
) -> usize {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    state.request_count.fetch_add(1, Ordering::SeqCst) + 1
}


pub(super) async fn upstream_chat_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    capture_upstream_request(&state, &headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"chatcmpl_upstream",
            "object":"chat.completion",
            "model":"gpt-4.1",
            "choices":[]
        })),
    )
}

pub(super) async fn upstream_chat_handler_with_usage(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    capture_upstream_request(&state, &headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"chatcmpl_upstream",
            "object":"chat.completion",
            "model":"gpt-4.1",
            "choices":[],
            "usage":{
                "prompt_tokens":120,
                "completion_tokens":80,
                "total_tokens":200
            }
        })),
    )
}

pub(super) async fn upstream_chat_handler_failure(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    capture_upstream_request(&state, &headers);

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({
            "error":{
                "message":"primary upstream failed",
                "type":"server_error",
                "code":"upstream_failed"
            }
        })),
    )
}

pub(super) async fn upstream_chat_handler_retryable_once_then_success(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    let attempt = capture_upstream_request(&state, &headers);

    if attempt == 1 {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({
                "error":{
                    "message":"upstream rate limited",
                    "type":"rate_limit_error",
                    "code":"retry_later"
                }
            })),
        );
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"chatcmpl_retry_recovered",
            "object":"chat.completion",
            "model":"gpt-4.1",
            "choices":[],
            "usage":{
                "prompt_tokens":24,
                "completion_tokens":12,
                "total_tokens":36
            }
        })),
    )
}

pub(super) async fn upstream_chat_handler_retry_after_once_then_success(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    let attempt = capture_upstream_request(&state, &headers);

    if attempt == 1 {
        return axum::response::Response::builder()
            .status(StatusCode::TOO_MANY_REQUESTS)
            .header("content-type", "application/json")
            .header("retry-after", "1")
            .body(Body::from(
                serde_json::json!({
                    "error":{
                        "message":"upstream rate limited with retry-after",
                        "type":"rate_limit_error",
                        "code":"retry_later"
                    }
                })
                .to_string(),
            ))
            .unwrap();
    }

    axum::response::Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::json!({
                "id":"chatcmpl_retry_after_recovered",
                "object":"chat.completion",
                "model":"gpt-4.1",
                "choices":[],
                "usage":{
                    "prompt_tokens":21,
                    "completion_tokens":9,
                    "total_tokens":30
                }
            })
            .to_string(),
        ))
        .unwrap()
}

pub(super) async fn upstream_chat_handler_non_retryable_once_then_success(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    let attempt = capture_upstream_request(&state, &headers);

    if attempt == 1 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error":{
                    "message":"invalid upstream payload",
                    "type":"invalid_request_error",
                    "code":"invalid_request"
                }
            })),
        );
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"chatcmpl_non_retryable_unexpected_retry",
            "object":"chat.completion",
            "model":"gpt-4.1",
            "choices":[]
        })),
    )
}

pub(super) async fn upstream_chat_handler_backup_with_usage(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    capture_upstream_request(&state, &headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"chatcmpl_backup",
            "object":"chat.completion",
            "model":"gpt-4.1",
            "choices":[],
            "usage":{
                "prompt_tokens":42,
                "completion_tokens":18,
                "total_tokens":60
            }
        })),
    )
}

pub(super) async fn upstream_chat_stream_handler_success(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    capture_upstream_request(&state, &headers);

    axum::response::Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/event-stream")
        .body(Body::from(
            "data: {\"id\":\"chatcmpl_stream_backup\",\"object\":\"chat.completion.chunk\"}\n\ndata: [DONE]\n\n",
        ))
        .unwrap()
}

pub(super) async fn upstream_chat_stream_handler_retryable_once_then_success(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    let attempt = capture_upstream_request(&state, &headers);

    if attempt == 1 {
        return axum::response::Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "error":{
                        "message":"stream temporarily unavailable",
                        "type":"server_error",
                        "code":"retry_later"
                    }
                })
                .to_string(),
            ))
            .unwrap();
    }

    axum::response::Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/event-stream")
        .body(Body::from(
            "data: {\"id\":\"chatcmpl_stream_retry_recovered\",\"object\":\"chat.completion.chunk\"}\n\ndata: [DONE]\n\n",
        ))
        .unwrap()
}

pub(super) async fn upstream_chat_list_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    capture_upstream_request(&state, &headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
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

pub(super) async fn upstream_chat_retrieve_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    capture_upstream_request(&state, &headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"chatcmpl_1",
            "object":"chat.completion",
            "model":"gpt-4.1",
            "choices":[]
        })),
    )
}

pub(super) async fn upstream_chat_update_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    capture_upstream_request(&state, &headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"chatcmpl_1",
            "object":"chat.completion",
            "model":"gpt-4.1",
            "metadata":{"tier":"gold"},
            "choices":[]
        })),
    )
}

pub(super) async fn upstream_chat_delete_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    capture_upstream_request(&state, &headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"chatcmpl_1",
            "object":"chat.completion.deleted",
            "deleted":true
        })),
    )
}

pub(super) async fn upstream_chat_messages_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    capture_upstream_request(&state, &headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
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
