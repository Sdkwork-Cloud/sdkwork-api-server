use super::support::*;

#[tokio::test]
async fn adapter_posts_moderations_to_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/moderations", post(capture_moderation_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request = sdkwork_api_contract_openai::moderations::CreateModerationRequest::new(
        "omni-moderation-latest",
        "hello",
    );

    let response = adapter
        .moderations("sk-upstream-openai", &request)
        .await
        .unwrap();

    assert_eq!(response["id"], "modr_upstream");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert_eq!(
        state.body.lock().unwrap().as_ref().unwrap()["input"],
        "hello"
    );
}

#[tokio::test]
async fn adapter_posts_image_generations_to_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/images/generations", post(capture_image_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request =
        sdkwork_api_contract_openai::images::CreateImageRequest::new("gpt-image-1", "hello");

    let response = adapter
        .images_generations("sk-upstream-openai", &request)
        .await
        .unwrap();

    assert_eq!(response["data"][0]["b64_json"], "upstream-image");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert_eq!(
        state.body.lock().unwrap().as_ref().unwrap()["prompt"],
        "hello"
    );
}

#[tokio::test]
async fn adapter_posts_image_edits_to_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/images/edits", post(capture_image_multipart_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request = sdkwork_api_contract_openai::images::CreateImageEditRequest::new(
        "make it sunset",
        sdkwork_api_contract_openai::images::ImageUpload::new("source.png", b"PNGDATA".to_vec())
            .with_content_type("image/png"),
    )
    .with_model("gpt-image-1")
    .with_mask(
        sdkwork_api_contract_openai::images::ImageUpload::new("mask.png", b"MASKDATA".to_vec())
            .with_content_type("image/png"),
    );

    let response = adapter
        .images_edits("sk-upstream-openai", &request)
        .await
        .unwrap();

    assert_eq!(response["data"][0]["b64_json"], "upstream-image");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert!(state
        .content_type
        .lock()
        .unwrap()
        .as_deref()
        .unwrap_or_default()
        .starts_with("multipart/form-data; boundary="));
    let raw_body = state.raw_body.lock().unwrap().clone().unwrap();
    let body = String::from_utf8_lossy(&raw_body);
    assert!(body.contains("make it sunset"));
    assert!(body.contains("source.png"));
    assert!(body.contains("mask.png"));
}

#[tokio::test]
async fn adapter_posts_image_variations_to_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/images/variations",
            post(capture_image_multipart_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request = sdkwork_api_contract_openai::images::CreateImageVariationRequest::new(
        sdkwork_api_contract_openai::images::ImageUpload::new("source.png", b"PNGDATA".to_vec())
            .with_content_type("image/png"),
    )
    .with_model("gpt-image-1");

    let response = adapter
        .images_variations("sk-upstream-openai", &request)
        .await
        .unwrap();

    assert_eq!(response["data"][0]["b64_json"], "upstream-image");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert!(state
        .content_type
        .lock()
        .unwrap()
        .as_deref()
        .unwrap_or_default()
        .starts_with("multipart/form-data; boundary="));
    let raw_body = state.raw_body.lock().unwrap().clone().unwrap();
    let body = String::from_utf8_lossy(&raw_body);
    assert!(body.contains("source.png"));
    assert!(body.contains("gpt-image-1"));
}

#[tokio::test]
async fn adapter_posts_videos_to_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/videos", post(capture_video_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request = sdkwork_api_contract_openai::videos::CreateVideoRequest::new(
        "sora-1",
        "A short cinematic flyover",
    );

    let response = adapter
        .videos("sk-upstream-openai", &request)
        .await
        .unwrap();

    assert_eq!(response["data"][0]["id"], "video_upstream");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
}

#[tokio::test]
async fn adapter_lists_videos_from_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/videos", get(capture_videos_list_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let response = adapter.list_videos("sk-upstream-openai").await.unwrap();

    assert_eq!(response["data"][0]["id"], "video_1");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
}

#[tokio::test]
async fn adapter_retrieves_video_from_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/videos/video_1", get(capture_video_retrieve_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let response = adapter
        .retrieve_video("sk-upstream-openai", "video_1")
        .await
        .unwrap();

    assert_eq!(response["id"], "video_1");
}

#[tokio::test]
async fn adapter_deletes_video_on_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/videos/video_1", delete(capture_video_delete_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let response = adapter
        .delete_video("sk-upstream-openai", "video_1")
        .await
        .unwrap();

    assert_eq!(response["deleted"], true);
}

#[tokio::test]
async fn adapter_reads_video_content_from_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/videos/video_1/content",
            get(capture_video_content_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let response = adapter
        .video_content("sk-upstream-openai", "video_1")
        .await
        .unwrap();
    let mut stream = response.into_body_stream();
    let mut bytes = Vec::new();
    while let Some(chunk) = stream.next().await {
        bytes.extend_from_slice(&chunk.unwrap());
    }

    assert_eq!(bytes.as_slice(), b"UPSTREAM-VIDEO");
}

#[tokio::test]
async fn adapter_remixes_video_on_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/videos/video_1/remix",
            post(capture_video_remix_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request = sdkwork_api_contract_openai::videos::RemixVideoRequest::new("Make it sunset");
    let response = adapter
        .remix_video("sk-upstream-openai", "video_1", &request)
        .await
        .unwrap();

    assert_eq!(response["data"][0]["id"], "video_1_remix");
}

#[tokio::test]
async fn adapter_posts_audio_transcriptions_to_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/audio/transcriptions",
            post(capture_transcription_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request = sdkwork_api_contract_openai::audio::CreateTranscriptionRequest::new(
        "gpt-4o-mini-transcribe",
        "file_1",
    );

    let response = adapter
        .audio_transcriptions("sk-upstream-openai", &request)
        .await
        .unwrap();

    assert_eq!(response["text"], "upstream transcription");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert_eq!(
        state.body.lock().unwrap().as_ref().unwrap()["file_id"],
        "file_1"
    );
}

#[tokio::test]
async fn adapter_posts_audio_translations_to_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/audio/translations", post(capture_translation_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request = sdkwork_api_contract_openai::audio::CreateTranslationRequest::new(
        "gpt-4o-mini-transcribe",
        "file_1",
    );

    let response = adapter
        .audio_translations("sk-upstream-openai", &request)
        .await
        .unwrap();

    assert_eq!(response["text"], "upstream translation");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert_eq!(
        state.body.lock().unwrap().as_ref().unwrap()["file_id"],
        "file_1"
    );
}

#[tokio::test]
async fn adapter_posts_audio_speech_to_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/audio/speech", post(capture_speech_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request = sdkwork_api_contract_openai::audio::CreateSpeechRequest::new(
        "gpt-4o-mini-tts",
        "nova",
        "Hello",
    );

    let response = adapter
        .audio_speech("sk-upstream-openai", &request)
        .await
        .unwrap();

    assert_eq!(response.content_type(), "audio/mpeg");
    let mut stream = response.into_body_stream();
    let mut bytes = Vec::new();
    while let Some(chunk) = stream.next().await {
        bytes.extend_from_slice(&chunk.unwrap());
    }
    assert_eq!(bytes.as_slice(), b"AUDIO");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert_eq!(
        state.body.lock().unwrap().as_ref().unwrap()["voice"],
        "nova"
    );
}


async fn capture_moderation_request(
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
            "id":"modr_upstream",
            "model":"omni-moderation-latest",
            "results":[{"flagged":false,"category_scores":{"violence":0.0}}]
        })),
    )
}

async fn capture_image_request(
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
            "data":[{"b64_json":"upstream-image"}]
        })),
    )
}

async fn capture_image_multipart_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    *state.content_type.lock().unwrap() = headers
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    *state.raw_body.lock().unwrap() = Some(body.to_vec());

    (
        StatusCode::OK,
        Json(json!({
            "data":[{"b64_json":"upstream-image"}]
        })),
    )
}

async fn capture_video_request(
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
                "id":"video_upstream",
                "object":"video",
                "url":"https://example.com/video.mp4"
            }]
        })),
    )
}

async fn capture_videos_list_request(
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
                "id":"video_1",
                "object":"video",
                "url":"https://example.com/video.mp4"
            }]
        })),
    )
}

async fn capture_video_retrieve_request(
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
            "id":"video_1",
            "object":"video",
            "url":"https://example.com/video.mp4"
        })),
    )
}

async fn capture_video_delete_request(
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
            "id":"video_1",
            "object":"video.deleted",
            "deleted":true
        })),
    )
}

async fn capture_video_content_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
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

async fn capture_video_remix_request(
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
                "id":"video_1_remix",
                "object":"video",
                "url":"https://example.com/video-remix.mp4"
            }]
        })),
    )
}

async fn capture_transcription_request(
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
            "text":"upstream transcription"
        })),
    )
}

async fn capture_translation_request(
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
            "text":"upstream translation"
        })),
    )
}


async fn capture_speech_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> (
    [(axum::http::header::HeaderName, &'static str); 1],
    &'static [u8],
) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    *state.body.lock().unwrap() = Some(body);

    ([(axum::http::header::CONTENT_TYPE, "audio/mpeg")], b"AUDIO")
}

