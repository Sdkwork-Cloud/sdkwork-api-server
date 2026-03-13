use std::sync::{Arc, Mutex};

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::post,
    Json, Router,
};
use serde_json::{json, Value};
use tokio::net::TcpListener;

#[derive(Clone, Default)]
struct CaptureState {
    authorization: Arc<Mutex<Option<String>>>,
    body: Arc<Mutex<Option<Value>>>,
}

#[tokio::test]
async fn adapter_posts_authorized_json_to_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/chat/completions", post(capture_chat_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request = sdkwork_api_contract_openai::chat_completions::CreateChatCompletionRequest {
        model: "gpt-4.1".to_owned(),
        messages: vec![
            sdkwork_api_contract_openai::chat_completions::ChatMessageInput {
                role: "user".to_owned(),
                content: Value::String("hello".to_owned()),
            },
        ],
        stream: Some(false),
    };

    let response = adapter
        .chat_completions("sk-upstream-openai", &request)
        .await
        .unwrap();

    assert_eq!(response["object"], "chat.completion");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert_eq!(
        state.body.lock().unwrap().as_ref().unwrap()["model"],
        "gpt-4.1"
    );
}

#[tokio::test]
async fn adapter_posts_legacy_completions_to_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/completions", post(capture_completion_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request = sdkwork_api_contract_openai::completions::CreateCompletionRequest::new(
        "gpt-3.5-turbo-instruct",
        "hello",
    );

    let response = adapter
        .completions("sk-upstream-openai", &request)
        .await
        .unwrap();

    assert_eq!(response["object"], "text_completion");
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
async fn adapter_posts_fine_tuning_jobs_to_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/fine_tuning/jobs", post(capture_fine_tuning_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request = sdkwork_api_contract_openai::fine_tuning::CreateFineTuningJobRequest::new(
        "file_1",
        "gpt-4.1-mini",
    );

    let response = adapter
        .fine_tuning_jobs("sk-upstream-openai", &request)
        .await
        .unwrap();

    assert_eq!(response["id"], "ftjob_upstream");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert_eq!(
        state.body.lock().unwrap().as_ref().unwrap()["training_file"],
        "file_1"
    );
}

#[tokio::test]
async fn adapter_posts_assistants_to_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/assistants", post(capture_assistant_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request =
        sdkwork_api_contract_openai::assistants::CreateAssistantRequest::new("Support", "gpt-4.1");

    let response = adapter
        .assistants("sk-upstream-openai", &request)
        .await
        .unwrap();

    assert_eq!(response["id"], "asst_upstream");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert_eq!(
        state.body.lock().unwrap().as_ref().unwrap()["name"],
        "Support"
    );
}

#[tokio::test]
async fn adapter_posts_realtime_sessions_to_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route(
            "/v1/realtime/sessions",
            post(capture_realtime_session_request),
        )
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request = sdkwork_api_contract_openai::realtime::CreateRealtimeSessionRequest::new(
        "gpt-4o-realtime-preview",
    );

    let response = adapter
        .realtime_sessions("sk-upstream-openai", &request)
        .await
        .unwrap();

    assert_eq!(response["id"], "sess_upstream");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert_eq!(
        state.body.lock().unwrap().as_ref().unwrap()["model"],
        "gpt-4o-realtime-preview"
    );
}

#[tokio::test]
async fn adapter_posts_evals_to_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/evals", post(capture_eval_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request =
        sdkwork_api_contract_openai::evals::CreateEvalRequest::new("qa-benchmark", "file_1");

    let response = adapter.evals("sk-upstream-openai", &request).await.unwrap();

    assert_eq!(response["id"], "eval_upstream");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert_eq!(
        state.body.lock().unwrap().as_ref().unwrap()["name"],
        "qa-benchmark"
    );
}

#[tokio::test]
async fn adapter_posts_batches_to_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/batches", post(capture_batch_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request = sdkwork_api_contract_openai::batches::CreateBatchRequest::new(
        "file_1",
        "/v1/responses",
        "24h",
    );

    let response = adapter
        .batches("sk-upstream-openai", &request)
        .await
        .unwrap();

    assert_eq!(response["id"], "batch_upstream");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert_eq!(
        state.body.lock().unwrap().as_ref().unwrap()["endpoint"],
        "/v1/responses"
    );
}

#[tokio::test]
async fn adapter_posts_vector_stores_to_openai_compatible_upstream() {
    let state = CaptureState::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let app = Router::new()
        .route("/v1/vector_stores", post(capture_vector_store_request))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let adapter =
        sdkwork_api_provider_openai::OpenAiProviderAdapter::new(format!("http://{address}"));
    let request =
        sdkwork_api_contract_openai::vector_stores::CreateVectorStoreRequest::new("kb-main");

    let response = adapter
        .vector_stores("sk-upstream-openai", &request)
        .await
        .unwrap();

    assert_eq!(response["id"], "vs_upstream");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert_eq!(
        state.body.lock().unwrap().as_ref().unwrap()["name"],
        "kb-main"
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

    assert_eq!(
        response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("audio/mpeg")
    );
    assert_eq!(response.bytes().await.unwrap().as_ref(), b"AUDIO");
    assert_eq!(
        state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert_eq!(
        state.body.lock().unwrap().as_ref().unwrap()["voice"],
        "nova"
    );
}

async fn capture_chat_request(
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
            "id":"chatcmpl_upstream",
            "object":"chat.completion",
            "model":"gpt-4.1",
            "choices":[]
        })),
    )
}

async fn capture_completion_request(
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
            "id":"cmpl_upstream",
            "object":"text_completion",
            "choices":[{"index":0,"text":"hello from upstream","finish_reason":"stop"}]
        })),
    )
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

async fn capture_fine_tuning_request(
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
            "id":"ftjob_upstream",
            "object":"fine_tuning.job",
            "model":"gpt-4.1-mini",
            "status":"queued"
        })),
    )
}

async fn capture_assistant_request(
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
            "id":"asst_upstream",
            "object":"assistant",
            "name":"Support",
            "model":"gpt-4.1"
        })),
    )
}

async fn capture_realtime_session_request(
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
            "id":"sess_upstream",
            "object":"realtime.session",
            "model":"gpt-4o-realtime-preview"
        })),
    )
}

async fn capture_eval_request(
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
            "id":"eval_upstream",
            "object":"eval",
            "name":"qa-benchmark",
            "status":"queued"
        })),
    )
}

async fn capture_batch_request(
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
            "id":"batch_upstream",
            "object":"batch",
            "endpoint":"/v1/responses",
            "input_file_id":"file_1",
            "status":"validating"
        })),
    )
}

async fn capture_vector_store_request(
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
            "id":"vs_upstream",
            "object":"vector_store",
            "name":"kb-main",
            "status":"completed"
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
