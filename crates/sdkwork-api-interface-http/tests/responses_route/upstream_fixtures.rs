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

pub(super) async fn upstream_responses_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    capture_upstream_request(&state, &headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"resp_upstream",
            "object":"response",
            "model":"gpt-4.1",
            "output":[]
        })),
    )
}

pub(super) async fn upstream_responses_handler_with_usage(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    capture_upstream_request(&state, &headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"resp_upstream",
            "object":"response",
            "model":"gpt-4.1",
            "output":[],
            "usage":{
                "input_tokens":160,
                "output_tokens":40,
                "total_tokens":200
            }
        })),
    )
}

pub(super) async fn upstream_responses_handler_failure(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    capture_upstream_request(&state, &headers);

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({
            "error":{
                "message":"primary responses upstream failed",
                "type":"server_error",
                "code":"upstream_failed"
            }
        })),
    )
}

pub(super) async fn upstream_responses_stream_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    capture_upstream_request(&state, &headers);

    (
        [(axum::http::header::CONTENT_TYPE, "text/event-stream")],
        "data: {\"id\":\"resp_upstream_stream\",\"type\":\"response.output_text.delta\"}\n\ndata: [DONE]\n\n",
    )
        .into_response()
}

pub(super) async fn upstream_responses_handler_retryable_once_then_success(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    let attempt = capture_upstream_request(&state, &headers);

    if attempt == 1 {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({
                "error":{
                    "message":"responses upstream rate limited",
                    "type":"rate_limit_error",
                    "code":"retry_later"
                }
            })),
        );
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"resp_retry_recovered",
            "object":"response",
            "model":"gpt-4.1",
            "output":[]
        })),
    )
}

pub(super) async fn upstream_responses_handler_retry_after_once_then_success(
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
                        "message":"responses upstream rate limited with retry-after",
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
                "id":"resp_retry_after_recovered",
                "object":"response",
                "model":"gpt-4.1",
                "output":[]
            })
            .to_string(),
        ))
        .unwrap()
}

pub(super) async fn upstream_responses_handler_http_date_retry_after_once_then_success(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    let attempt = capture_upstream_request(&state, &headers);

    if attempt == 1 {
        return axum::response::Response::builder()
            .status(StatusCode::TOO_MANY_REQUESTS)
            .header("content-type", "application/json")
            .header("retry-after", "Thu, 01 Jan 2099 00:00:00 GMT")
            .body(Body::from(
                serde_json::json!({
                    "error":{
                        "message":"responses upstream rate limited with http-date retry-after",
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
                "id":"resp_http_date_retry_after_recovered",
                "object":"response",
                "model":"gpt-4.1",
                "output":[]
            })
            .to_string(),
        ))
        .unwrap()
}

pub(super) async fn upstream_responses_handler_non_retryable_once_then_success(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    let attempt = capture_upstream_request(&state, &headers);

    if attempt == 1 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error":{
                    "message":"invalid responses upstream payload",
                    "type":"invalid_request_error",
                    "code":"invalid_request"
                }
            })),
        );
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"resp_non_retryable_unexpected_retry",
            "object":"response",
            "model":"gpt-4.1",
            "output":[]
        })),
    )
}

pub(super) async fn upstream_responses_stream_handler_retryable_once_then_success(
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
                        "message":"responses stream temporarily unavailable",
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
            "data: {\"id\":\"resp_stream_retry_recovered\",\"type\":\"response.output_text.delta\"}\n\ndata: [DONE]\n\n",
        ))
        .unwrap()
}

pub(super) async fn upstream_response_retrieve_handler(
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
            "id":"resp_1",
            "object":"response",
            "model":"gpt-4.1",
            "output":[]
        })),
    )
}

pub(super) async fn upstream_response_input_items_handler(
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
            "object":"list",
            "data":[{
                "id":"item_1",
                "object":"response.input_item",
                "type":"message"
            }]
        })),
    )
}

pub(super) async fn upstream_response_delete_handler(
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
            "id":"resp_1",
            "object":"response.deleted",
            "deleted":true
        })),
    )
}

pub(super) async fn upstream_response_input_tokens_handler(
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
            "object":"response.input_tokens",
            "input_tokens":21
        })),
    )
}

pub(super) async fn upstream_response_cancel_handler(
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
            "id":"resp_1",
            "object":"response",
            "model":"gpt-4.1",
            "status":"cancelled",
            "output":[]
        })),
    )
}

pub(super) async fn upstream_response_compact_handler(
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
            "id":"resp_cmp_1",
            "object":"response.compaction",
            "model":"gpt-4.1"
        })),
    )
}
