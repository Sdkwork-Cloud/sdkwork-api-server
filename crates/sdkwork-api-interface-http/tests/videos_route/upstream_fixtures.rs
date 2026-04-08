use super::*;

async fn upstream_videos_handler(
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
                "id":"video_upstream",
                "object":"video",
                "url":"https://example.com/video.mp4"
            }]
        })),
    )
}

async fn upstream_videos_list_handler(
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
                "id":"video_1",
                "object":"video",
                "url":"https://example.com/video.mp4"
            }]
        })),
    )
}

async fn upstream_video_retrieve_handler(
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
            "id":"video_1",
            "object":"video",
            "url":"https://example.com/video.mp4"
        })),
    )
}

async fn upstream_video_delete_handler(
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
            "id":"video_1",
            "object":"video.deleted",
            "deleted":true
        })),
    )
}

async fn upstream_video_content_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (
    [(axum::http::header::HeaderName, &'static str); 1],
    &'static [u8],
) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    (
        [(axum::http::header::CONTENT_TYPE, "video/mp4")],
        b"UPSTREAM-VIDEO",
    )
}

async fn upstream_video_remix_handler(
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
                "id":"video_1_remix",
                "object":"video",
                "url":"https://example.com/video-remix.mp4"
            }]
        })),
    )
}

async fn upstream_video_characters_list_handler(
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
                "id":"char_1",
                "object":"video.character",
                "name":"Hero"
            }]
        })),
    )
}

async fn upstream_video_character_retrieve_handler(
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
            "id":"char_1",
            "object":"video.character",
            "name":"Hero"
        })),
    )
}

async fn upstream_video_character_update_handler(
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
            "id":"char_1",
            "object":"video.character",
            "name":"Hero"
        })),
    )
}

async fn upstream_video_extend_handler(
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
                "id":"video_1_extended",
                "object":"video",
                "url":"https://example.com/video-extended.mp4"
            }]
        })),
    )
}

async fn upstream_video_character_create_handler(
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
            "id":"char_upstream",
            "object":"video.character",
            "name":"Hero"
        })),
    )
}

async fn upstream_video_character_canonical_retrieve_handler(
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
            "id":"char_1",
            "object":"video.character",
            "name":"Hero"
        })),
    )
}

async fn upstream_video_edit_handler(
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
            "data":[
                {
                    "id":"video_1_edited",
                    "object":"video",
                    "url":"https://example.com/video-edited.mp4"
                }
            ]
        })),
    )
}

async fn upstream_video_extensions_handler(
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
            "data":[
                {
                    "id":"video_1_extended",
                    "object":"video",
                    "url":"https://example.com/video-extended.mp4"
                }
            ]
        })),
    )
}
