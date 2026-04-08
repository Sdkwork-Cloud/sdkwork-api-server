use super::*;

pub(super) async fn upstream_files_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"file_upstream",
            "object":"file",
            "purpose":"fine-tune",
            "filename":"train.jsonl",
            "bytes":2,
            "status":"processed"
        })),
    )
}

pub(super) async fn upstream_files_list_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "object":"list",
            "data":[{
                "id":"file_1",
                "object":"file",
                "purpose":"fine-tune",
                "filename":"train.jsonl",
                "bytes":2,
                "status":"processed"
            }]
        })),
    )
}

pub(super) async fn upstream_file_retrieve_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"file_1",
            "object":"file",
            "purpose":"fine-tune",
            "filename":"train.jsonl",
            "bytes":2,
            "status":"processed"
        })),
    )
}

pub(super) async fn upstream_file_delete_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"file_1",
            "object":"file",
            "deleted":true
        })),
    )
}

pub(super) async fn upstream_file_content_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (
    [(axum::http::header::HeaderName, &'static str); 1],
    &'static [u8],
) {
    state.capture_headers(&headers);

    (
        [(axum::http::header::CONTENT_TYPE, "application/jsonl")],
        b"{}",
    )
}
