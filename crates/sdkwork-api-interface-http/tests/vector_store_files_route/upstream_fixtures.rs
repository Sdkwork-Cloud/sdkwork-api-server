use super::*;

pub(super) async fn upstream_vector_store_files_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"file_1",
            "object":"vector_store.file",
            "status":"completed"
        })),
    )
}

pub(super) async fn upstream_vector_store_files_distinct_id_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"vsfile_upstream",
            "object":"vector_store.file",
            "status":"completed"
        })),
    )
}

pub(super) async fn upstream_vector_store_files_list_handler(
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
                "object":"vector_store.file",
                "status":"completed"
            }]
        })),
    )
}

pub(super) async fn upstream_vector_store_file_retrieve_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"file_1",
            "object":"vector_store.file",
            "status":"completed"
        })),
    )
}

pub(super) async fn upstream_vector_store_file_delete_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"file_1",
            "object":"vector_store.file.deleted",
            "deleted":true
        })),
    )
}
