use std::sync::Arc;

use axum::{
    body::Body,
    extract::Json as ExtractJson,
    extract::State,
    http::header,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use sdkwork_api_app_billing::persist_ledger_entry;
use sdkwork_api_app_credential::CredentialSecretManager;
use sdkwork_api_app_gateway::create_chat_completion;
use sdkwork_api_app_gateway::create_completion;
use sdkwork_api_app_gateway::create_fine_tuning_job;
use sdkwork_api_app_gateway::create_image_generation;
use sdkwork_api_app_gateway::create_moderation;
use sdkwork_api_app_gateway::create_transcription;
use sdkwork_api_app_gateway::create_translation;
use sdkwork_api_app_gateway::list_models;
use sdkwork_api_app_gateway::{
    create_embedding, create_response, list_models_from_store, relay_chat_completion_from_store,
    relay_chat_completion_stream_from_store, relay_completion_from_store,
    relay_embedding_from_store, relay_fine_tuning_job_from_store,
    relay_image_generation_from_store, relay_moderation_from_store, relay_response_from_store,
    relay_transcription_from_store, relay_translation_from_store,
};
use sdkwork_api_app_routing::simulate_route_with_store;
use sdkwork_api_app_usage::persist_usage_record;
use sdkwork_api_contract_openai::audio::{CreateTranscriptionRequest, CreateTranslationRequest};
use sdkwork_api_contract_openai::chat_completions::CreateChatCompletionRequest;
use sdkwork_api_contract_openai::completions::CreateCompletionRequest;
use sdkwork_api_contract_openai::embeddings::CreateEmbeddingRequest;
use sdkwork_api_contract_openai::fine_tuning::CreateFineTuningJobRequest;
use sdkwork_api_contract_openai::images::CreateImageRequest;
use sdkwork_api_contract_openai::moderations::CreateModerationRequest;
use sdkwork_api_contract_openai::responses::CreateResponseRequest;
use sdkwork_api_contract_openai::streaming::SseFrame;
use sdkwork_api_storage_core::AdminStore;
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct GatewayApiState {
    store: Arc<dyn AdminStore>,
    secret_manager: CredentialSecretManager,
}

impl GatewayApiState {
    pub fn new(pool: SqlitePool) -> Self {
        Self::with_master_key(pool, "local-dev-master-key")
    }

    pub fn with_master_key(pool: SqlitePool, credential_master_key: impl Into<String>) -> Self {
        Self::with_store_and_secret_manager(
            Arc::new(SqliteAdminStore::new(pool)),
            CredentialSecretManager::database_encrypted(credential_master_key),
        )
    }

    pub fn with_secret_manager(pool: SqlitePool, secret_manager: CredentialSecretManager) -> Self {
        Self {
            store: Arc::new(SqliteAdminStore::new(pool)),
            secret_manager,
        }
    }

    pub fn with_store_and_secret_manager(
        store: Arc<dyn AdminStore>,
        secret_manager: CredentialSecretManager,
    ) -> Self {
        Self {
            store,
            secret_manager,
        }
    }
}

pub fn gateway_router() -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/v1/models", get(list_models_handler))
        .route("/v1/chat/completions", post(chat_completions_handler))
        .route("/v1/completions", post(completions_handler))
        .route("/v1/responses", post(responses_handler))
        .route("/v1/embeddings", post(embeddings_handler))
        .route("/v1/moderations", post(moderations_handler))
        .route("/v1/images/generations", post(image_generations_handler))
        .route("/v1/audio/transcriptions", post(transcriptions_handler))
        .route("/v1/audio/translations", post(translations_handler))
        .route("/v1/fine_tuning/jobs", post(fine_tuning_jobs_handler))
}

pub fn gateway_router_with_pool(pool: SqlitePool) -> Router {
    gateway_router_with_pool_and_master_key(pool, "local-dev-master-key")
}

pub fn gateway_router_with_store(store: Arc<dyn AdminStore>) -> Router {
    gateway_router_with_store_and_secret_manager(
        store,
        CredentialSecretManager::database_encrypted("local-dev-master-key"),
    )
}

pub fn gateway_router_with_pool_and_master_key(
    pool: SqlitePool,
    credential_master_key: impl Into<String>,
) -> Router {
    gateway_router_with_store_and_secret_manager(
        Arc::new(SqliteAdminStore::new(pool)),
        CredentialSecretManager::database_encrypted(credential_master_key),
    )
}

pub fn gateway_router_with_pool_and_secret_manager(
    pool: SqlitePool,
    secret_manager: CredentialSecretManager,
) -> Router {
    gateway_router_with_store_and_secret_manager(
        Arc::new(SqliteAdminStore::new(pool)),
        secret_manager,
    )
}

pub fn gateway_router_with_store_and_secret_manager(
    store: Arc<dyn AdminStore>,
    secret_manager: CredentialSecretManager,
) -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/v1/models", get(list_models_from_store_handler))
        .route(
            "/v1/chat/completions",
            post(chat_completions_with_state_handler),
        )
        .route("/v1/completions", post(completions_with_state_handler))
        .route("/v1/responses", post(responses_with_state_handler))
        .route("/v1/embeddings", post(embeddings_with_state_handler))
        .route("/v1/moderations", post(moderations_with_state_handler))
        .route(
            "/v1/images/generations",
            post(image_generations_with_state_handler),
        )
        .route(
            "/v1/audio/transcriptions",
            post(transcriptions_with_state_handler),
        )
        .route(
            "/v1/audio/translations",
            post(translations_with_state_handler),
        )
        .route(
            "/v1/fine_tuning/jobs",
            post(fine_tuning_jobs_with_state_handler),
        )
        .with_state(GatewayApiState::with_store_and_secret_manager(
            store,
            secret_manager,
        ))
}

async fn list_models_handler() -> Json<sdkwork_api_contract_openai::models::ListModelsResponse> {
    Json(list_models("tenant-1", "project-1").expect("models response"))
}

async fn list_models_from_store_handler(
    State(state): State<GatewayApiState>,
) -> Result<Json<sdkwork_api_contract_openai::models::ListModelsResponse>, Response> {
    list_models_from_store(state.store.as_ref(), "tenant-1", "project-1")
        .await
        .map(Json)
        .map_err(|_| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to load models",
            )
                .into_response()
        })
}

async fn chat_completions_handler(
    ExtractJson(request): ExtractJson<CreateChatCompletionRequest>,
) -> Response {
    if request.stream.unwrap_or(false) {
        let body = format!(
            "{}{}",
            SseFrame::data("{\"id\":\"chatcmpl_1\",\"object\":\"chat.completion.chunk\"}"),
            SseFrame::data("[DONE]")
        );
        ([(header::CONTENT_TYPE, "text/event-stream")], body).into_response()
    } else {
        Json(
            create_chat_completion("tenant-1", "project-1", &request.model)
                .expect("chat completion"),
        )
        .into_response()
    }
}

async fn responses_handler(
    ExtractJson(request): ExtractJson<CreateResponseRequest>,
) -> Json<sdkwork_api_contract_openai::responses::ResponseObject> {
    Json(create_response("tenant-1", "project-1", &request.model).expect("response"))
}

async fn completions_handler(
    ExtractJson(request): ExtractJson<CreateCompletionRequest>,
) -> Json<sdkwork_api_contract_openai::completions::CompletionObject> {
    Json(create_completion("tenant-1", "project-1", &request.model).expect("completion"))
}

async fn embeddings_handler(
    ExtractJson(request): ExtractJson<CreateEmbeddingRequest>,
) -> Json<sdkwork_api_contract_openai::embeddings::CreateEmbeddingResponse> {
    Json(create_embedding("tenant-1", "project-1", &request.model).expect("embedding"))
}

async fn moderations_handler(
    ExtractJson(request): ExtractJson<CreateModerationRequest>,
) -> Json<sdkwork_api_contract_openai::moderations::ModerationResponse> {
    Json(create_moderation("tenant-1", "project-1", &request.model).expect("moderation"))
}

async fn image_generations_handler(
    ExtractJson(request): ExtractJson<CreateImageRequest>,
) -> Json<sdkwork_api_contract_openai::images::ImagesResponse> {
    Json(
        create_image_generation("tenant-1", "project-1", &request.model).expect("image generation"),
    )
}

async fn transcriptions_handler(
    ExtractJson(request): ExtractJson<CreateTranscriptionRequest>,
) -> Json<sdkwork_api_contract_openai::audio::TranscriptionObject> {
    Json(create_transcription("tenant-1", "project-1", &request.model).expect("transcription"))
}

async fn translations_handler(
    ExtractJson(request): ExtractJson<CreateTranslationRequest>,
) -> Json<sdkwork_api_contract_openai::audio::TranslationObject> {
    Json(create_translation("tenant-1", "project-1", &request.model).expect("translation"))
}

async fn fine_tuning_jobs_handler(
    ExtractJson(request): ExtractJson<CreateFineTuningJobRequest>,
) -> Json<sdkwork_api_contract_openai::fine_tuning::FineTuningJobObject> {
    Json(create_fine_tuning_job("tenant-1", "project-1", &request.model).expect("fine tuning"))
}

async fn chat_completions_with_state_handler(
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateChatCompletionRequest>,
) -> Response {
    if request.stream.unwrap_or(false) {
        match relay_chat_completion_stream_from_store(
            state.store.as_ref(),
            &state.secret_manager,
            "tenant-1",
            "project-1",
            &request,
        )
        .await
        {
            Ok(Some(response)) => {
                let usage_result = record_gateway_usage(
                    state.store.as_ref(),
                    "chat_completion",
                    &request.model,
                    100,
                    0.10,
                )
                .await;
                if usage_result.is_err() {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to record usage",
                    )
                        .into_response();
                }

                return upstream_stream_response(response);
            }
            Ok(None) => {}
            Err(_) => {
                return (
                    axum::http::StatusCode::BAD_GATEWAY,
                    "failed to relay upstream chat completion stream",
                )
                    .into_response();
            }
        }
    } else {
        match relay_chat_completion_from_store(
            state.store.as_ref(),
            &state.secret_manager,
            "tenant-1",
            "project-1",
            &request,
        )
        .await
        {
            Ok(Some(response)) => {
                let usage_result = record_gateway_usage(
                    state.store.as_ref(),
                    "chat_completion",
                    &request.model,
                    100,
                    0.10,
                )
                .await;
                if usage_result.is_err() {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to record usage",
                    )
                        .into_response();
                }

                return Json(response).into_response();
            }
            Ok(None) => {}
            Err(_) => {
                return (
                    axum::http::StatusCode::BAD_GATEWAY,
                    "failed to relay upstream chat completion",
                )
                    .into_response();
            }
        }
    }

    let usage_result = record_gateway_usage(
        state.store.as_ref(),
        "chat_completion",
        &request.model,
        100,
        0.10,
    )
    .await;
    if usage_result.is_err() {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    if request.stream.unwrap_or(false) {
        let body = format!(
            "{}{}",
            SseFrame::data("{\"id\":\"chatcmpl_1\",\"object\":\"chat.completion.chunk\"}"),
            SseFrame::data("[DONE]")
        );
        ([(header::CONTENT_TYPE, "text/event-stream")], body).into_response()
    } else {
        Json(
            create_chat_completion("tenant-1", "project-1", &request.model)
                .expect("chat completion"),
        )
        .into_response()
    }
}

fn upstream_stream_response(response: reqwest::Response) -> Response {
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("text/event-stream")
        .to_owned();

    Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .body(Body::from_stream(response.bytes_stream()))
        .expect("valid upstream stream response")
}

async fn responses_with_state_handler(
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateResponseRequest>,
) -> Response {
    match relay_response_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(state.store.as_ref(), "responses", &request.model, 120, 0.12)
                .await
                .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_GATEWAY,
                "failed to relay upstream response",
            )
                .into_response();
        }
    }

    if record_gateway_usage(state.store.as_ref(), "responses", &request.model, 120, 0.12)
        .await
        .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(create_response("tenant-1", "project-1", &request.model).expect("response"))
        .into_response()
}

async fn completions_with_state_handler(
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateCompletionRequest>,
) -> Response {
    match relay_completion_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(
                state.store.as_ref(),
                "completions",
                &request.model,
                80,
                0.08,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_GATEWAY,
                "failed to relay upstream completion",
            )
                .into_response();
        }
    }

    if record_gateway_usage(
        state.store.as_ref(),
        "completions",
        &request.model,
        80,
        0.08,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(create_completion("tenant-1", "project-1", &request.model).expect("completion"))
        .into_response()
}

async fn embeddings_with_state_handler(
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateEmbeddingRequest>,
) -> Response {
    match relay_embedding_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(state.store.as_ref(), "embeddings", &request.model, 10, 0.01)
                .await
                .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_GATEWAY,
                "failed to relay upstream embedding",
            )
                .into_response();
        }
    }

    if record_gateway_usage(state.store.as_ref(), "embeddings", &request.model, 10, 0.01)
        .await
        .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(create_embedding("tenant-1", "project-1", &request.model).expect("embedding"))
        .into_response()
}

async fn moderations_with_state_handler(
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateModerationRequest>,
) -> Response {
    match relay_moderation_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(
                state.store.as_ref(),
                "moderations",
                &request.model,
                1,
                0.001,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_GATEWAY,
                "failed to relay upstream moderation",
            )
                .into_response();
        }
    }

    if record_gateway_usage(
        state.store.as_ref(),
        "moderations",
        &request.model,
        1,
        0.001,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(create_moderation("tenant-1", "project-1", &request.model).expect("moderation"))
        .into_response()
}

async fn image_generations_with_state_handler(
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateImageRequest>,
) -> Response {
    match relay_image_generation_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(state.store.as_ref(), "images", &request.model, 50, 0.05)
                .await
                .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_GATEWAY,
                "failed to relay upstream image generation",
            )
                .into_response();
        }
    }

    if record_gateway_usage(state.store.as_ref(), "images", &request.model, 50, 0.05)
        .await
        .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(create_image_generation("tenant-1", "project-1", &request.model).expect("image"))
        .into_response()
}

async fn transcriptions_with_state_handler(
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateTranscriptionRequest>,
) -> Response {
    match relay_transcription_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(
                state.store.as_ref(),
                "audio_transcriptions",
                &request.model,
                25,
                0.025,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_GATEWAY,
                "failed to relay upstream transcription",
            )
                .into_response();
        }
    }

    if record_gateway_usage(
        state.store.as_ref(),
        "audio_transcriptions",
        &request.model,
        25,
        0.025,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(create_transcription("tenant-1", "project-1", &request.model).expect("transcription"))
        .into_response()
}

async fn translations_with_state_handler(
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateTranslationRequest>,
) -> Response {
    match relay_translation_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(
                state.store.as_ref(),
                "audio_translations",
                &request.model,
                25,
                0.025,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_GATEWAY,
                "failed to relay upstream translation",
            )
                .into_response();
        }
    }

    if record_gateway_usage(
        state.store.as_ref(),
        "audio_translations",
        &request.model,
        25,
        0.025,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(create_translation("tenant-1", "project-1", &request.model).expect("translation"))
        .into_response()
}

async fn fine_tuning_jobs_with_state_handler(
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateFineTuningJobRequest>,
) -> Response {
    match relay_fine_tuning_job_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(
                state.store.as_ref(),
                "fine_tuning",
                &request.model,
                200,
                0.2,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_GATEWAY,
                "failed to relay upstream fine tuning job",
            )
                .into_response();
        }
    }

    if record_gateway_usage(
        state.store.as_ref(),
        "fine_tuning",
        &request.model,
        200,
        0.2,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(create_fine_tuning_job("tenant-1", "project-1", &request.model).expect("fine tuning"))
        .into_response()
}

async fn record_gateway_usage(
    store: &dyn AdminStore,
    capability: &str,
    model: &str,
    units: u64,
    amount: f64,
) -> anyhow::Result<()> {
    let decision = simulate_route_with_store(store, capability, model).await?;
    persist_usage_record(store, "project-1", model, &decision.selected_provider_id).await?;
    persist_ledger_entry(store, "project-1", units, amount).await?;
    Ok(())
}
