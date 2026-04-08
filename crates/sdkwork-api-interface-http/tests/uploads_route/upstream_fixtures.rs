use super::*;

pub(super) async fn upstream_uploads_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_authorization(&headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"upload_upstream",
            "object":"upload",
            "purpose":"batch",
            "filename":"input.jsonl",
            "mime_type":"application/jsonl",
            "bytes":1024,
            "part_ids":[],
            "status":"pending"
        })),
    )
}

pub(super) async fn upstream_upload_parts_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"part_upstream",
            "object":"upload.part",
            "upload_id":"upload_1",
            "status":"completed"
        })),
    )
}

pub(super) async fn upstream_upload_complete_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_authorization(&headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"upload_upstream",
            "object":"upload",
            "purpose":"batch",
            "filename":"input.jsonl",
            "mime_type":"application/jsonl",
            "bytes":1024,
            "part_ids":["part_1","part_2"],
            "status":"completed"
        })),
    )
}

pub(super) async fn upstream_upload_cancel_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_authorization(&headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"upload_upstream",
            "object":"upload",
            "purpose":"batch",
            "filename":"input.jsonl",
            "mime_type":"application/jsonl",
            "bytes":1024,
            "part_ids":["part_1","part_2"],
            "status":"cancelled"
        })),
    )
}
