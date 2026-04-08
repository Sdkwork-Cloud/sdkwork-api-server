use super::*;

pub(super) async fn upstream_containers_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"container_upstream",
            "object":"container",
            "name":"ci-container",
            "status":"running"
        })),
    )
}

pub(super) async fn upstream_containers_list_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "object":"list",
            "data":[
                {
                    "id":"container_1",
                    "object":"container",
                    "name":"ci-container",
                    "status":"running"
                }
            ]
        })),
    )
}

pub(super) async fn upstream_container_retrieve_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"container_1",
            "object":"container",
            "name":"ci-container",
            "status":"running"
        })),
    )
}

pub(super) async fn upstream_container_delete_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"container_1",
            "object":"container.deleted",
            "deleted":true
        })),
    )
}

pub(super) async fn upstream_container_files_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"container_file_upstream",
            "object":"container.file",
            "container_id":"container_1"
        })),
    )
}

pub(super) async fn upstream_container_files_list_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "object":"list",
            "data":[
                {
                    "id":"file_1",
                    "object":"container.file",
                    "container_id":"container_1"
                }
            ]
        })),
    )
}

pub(super) async fn upstream_container_file_retrieve_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"file_1",
            "object":"container.file",
            "container_id":"container_1"
        })),
    )
}

pub(super) async fn upstream_container_file_delete_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    state.capture_headers(&headers);
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"file_1",
            "object":"container.file.deleted",
            "deleted":true
        })),
    )
}

pub(super) async fn upstream_container_file_content_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (
    [(axum::http::header::HeaderName, &'static str); 1],
    &'static [u8],
) {
    state.capture_headers(&headers);
    (
        [(axum::http::header::CONTENT_TYPE, "application/octet-stream")],
        b"CONTAINER-FILE",
    )
}
