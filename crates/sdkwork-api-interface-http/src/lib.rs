use std::sync::Arc;

use axum::{
    body::Body,
    extract::Json as ExtractJson,
    extract::Multipart,
    extract::Path,
    extract::State,
    http::header,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use sdkwork_api_app_billing::persist_ledger_entry;
use sdkwork_api_app_credential::CredentialSecretManager;
use sdkwork_api_app_gateway::cancel_batch;
use sdkwork_api_app_gateway::cancel_fine_tuning_job;
use sdkwork_api_app_gateway::cancel_upload;
use sdkwork_api_app_gateway::cancel_vector_store_file_batch;
use sdkwork_api_app_gateway::complete_upload;
use sdkwork_api_app_gateway::create_assistant;
use sdkwork_api_app_gateway::create_batch;
use sdkwork_api_app_gateway::create_chat_completion;
use sdkwork_api_app_gateway::create_completion;
use sdkwork_api_app_gateway::create_eval;
use sdkwork_api_app_gateway::create_file;
use sdkwork_api_app_gateway::create_fine_tuning_job;
use sdkwork_api_app_gateway::create_image_generation;
use sdkwork_api_app_gateway::create_moderation;
use sdkwork_api_app_gateway::create_realtime_session;
use sdkwork_api_app_gateway::create_speech_response;
use sdkwork_api_app_gateway::create_transcription;
use sdkwork_api_app_gateway::create_translation;
use sdkwork_api_app_gateway::create_upload;
use sdkwork_api_app_gateway::create_upload_part;
use sdkwork_api_app_gateway::create_vector_store;
use sdkwork_api_app_gateway::create_vector_store_file;
use sdkwork_api_app_gateway::create_vector_store_file_batch;
use sdkwork_api_app_gateway::create_video;
use sdkwork_api_app_gateway::create_webhook;
use sdkwork_api_app_gateway::delete_assistant;
use sdkwork_api_app_gateway::delete_file;
use sdkwork_api_app_gateway::delete_response;
use sdkwork_api_app_gateway::delete_vector_store;
use sdkwork_api_app_gateway::delete_vector_store_file;
use sdkwork_api_app_gateway::delete_video;
use sdkwork_api_app_gateway::delete_webhook;
use sdkwork_api_app_gateway::file_content;
use sdkwork_api_app_gateway::get_assistant;
use sdkwork_api_app_gateway::get_batch;
use sdkwork_api_app_gateway::get_file;
use sdkwork_api_app_gateway::get_fine_tuning_job;
use sdkwork_api_app_gateway::get_model;
use sdkwork_api_app_gateway::get_model_from_store;
use sdkwork_api_app_gateway::get_response;
use sdkwork_api_app_gateway::get_vector_store;
use sdkwork_api_app_gateway::get_vector_store_file;
use sdkwork_api_app_gateway::get_vector_store_file_batch;
use sdkwork_api_app_gateway::get_video;
use sdkwork_api_app_gateway::get_webhook;
use sdkwork_api_app_gateway::list_assistants;
use sdkwork_api_app_gateway::list_batches;
use sdkwork_api_app_gateway::list_files;
use sdkwork_api_app_gateway::list_fine_tuning_jobs;
use sdkwork_api_app_gateway::list_models;
use sdkwork_api_app_gateway::list_response_input_items;
use sdkwork_api_app_gateway::list_vector_store_file_batch_files;
use sdkwork_api_app_gateway::list_vector_store_files;
use sdkwork_api_app_gateway::list_vector_stores;
use sdkwork_api_app_gateway::list_videos;
use sdkwork_api_app_gateway::list_webhooks;
use sdkwork_api_app_gateway::remix_video;
use sdkwork_api_app_gateway::search_vector_store;
use sdkwork_api_app_gateway::update_assistant;
use sdkwork_api_app_gateway::update_vector_store;
use sdkwork_api_app_gateway::update_webhook;
use sdkwork_api_app_gateway::video_content;
use sdkwork_api_app_gateway::{
    create_embedding, create_response, list_models_from_store, relay_assistant_from_store,
    relay_batch_from_store, relay_cancel_batch_from_store, relay_cancel_fine_tuning_job_from_store,
    relay_cancel_upload_from_store, relay_cancel_vector_store_file_batch_from_store,
    relay_chat_completion_from_store, relay_chat_completion_stream_from_store,
    relay_complete_upload_from_store, relay_completion_from_store,
    relay_delete_assistant_from_store, relay_delete_file_from_store,
    relay_delete_response_from_store, relay_delete_vector_store_file_from_store,
    relay_delete_vector_store_from_store, relay_delete_video_from_store,
    relay_delete_webhook_from_store, relay_embedding_from_store, relay_eval_from_store,
    relay_file_content_from_store, relay_file_from_store, relay_fine_tuning_job_from_store,
    relay_get_assistant_from_store, relay_get_batch_from_store, relay_get_file_from_store,
    relay_get_fine_tuning_job_from_store, relay_get_response_from_store,
    relay_get_vector_store_file_batch_from_store, relay_get_vector_store_file_from_store,
    relay_get_vector_store_from_store, relay_get_video_from_store, relay_get_webhook_from_store,
    relay_image_generation_from_store, relay_list_assistants_from_store,
    relay_list_batches_from_store, relay_list_files_from_store,
    relay_list_fine_tuning_jobs_from_store, relay_list_response_input_items_from_store,
    relay_list_vector_store_file_batch_files_from_store, relay_list_vector_store_files_from_store,
    relay_list_vector_stores_from_store, relay_list_videos_from_store,
    relay_list_webhooks_from_store, relay_moderation_from_store, relay_realtime_session_from_store,
    relay_remix_video_from_store, relay_response_from_store, relay_search_vector_store_from_store,
    relay_speech_from_store, relay_transcription_from_store, relay_translation_from_store,
    relay_update_assistant_from_store, relay_update_vector_store_from_store,
    relay_update_webhook_from_store, relay_upload_from_store, relay_upload_part_from_store,
    relay_vector_store_file_batch_from_store, relay_vector_store_file_from_store,
    relay_vector_store_from_store, relay_video_content_from_store, relay_video_from_store,
    relay_webhook_from_store,
};
use sdkwork_api_app_routing::simulate_route_with_store;
use sdkwork_api_app_usage::persist_usage_record;
use sdkwork_api_contract_openai::assistants::{CreateAssistantRequest, UpdateAssistantRequest};
use sdkwork_api_contract_openai::audio::{
    CreateSpeechRequest, CreateTranscriptionRequest, CreateTranslationRequest,
};
use sdkwork_api_contract_openai::batches::CreateBatchRequest;
use sdkwork_api_contract_openai::chat_completions::CreateChatCompletionRequest;
use sdkwork_api_contract_openai::completions::CreateCompletionRequest;
use sdkwork_api_contract_openai::embeddings::CreateEmbeddingRequest;
use sdkwork_api_contract_openai::evals::CreateEvalRequest;
use sdkwork_api_contract_openai::files::CreateFileRequest;
use sdkwork_api_contract_openai::fine_tuning::CreateFineTuningJobRequest;
use sdkwork_api_contract_openai::images::CreateImageRequest;
use sdkwork_api_contract_openai::moderations::CreateModerationRequest;
use sdkwork_api_contract_openai::realtime::CreateRealtimeSessionRequest;
use sdkwork_api_contract_openai::responses::{
    CreateResponseRequest, DeleteResponseResponse, ListResponseInputItemsResponse, ResponseObject,
};
use sdkwork_api_contract_openai::streaming::SseFrame;
use sdkwork_api_contract_openai::uploads::{
    AddUploadPartRequest, CompleteUploadRequest, CreateUploadRequest,
};
use sdkwork_api_contract_openai::vector_stores::{
    CreateVectorStoreFileBatchRequest, CreateVectorStoreFileRequest, CreateVectorStoreRequest,
    SearchVectorStoreRequest, UpdateVectorStoreRequest,
};
use sdkwork_api_contract_openai::videos::{CreateVideoRequest, RemixVideoRequest};
use sdkwork_api_contract_openai::webhooks::{CreateWebhookRequest, UpdateWebhookRequest};
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
        .route("/v1/models/{model_id}", get(model_retrieve_handler))
        .route("/v1/chat/completions", post(chat_completions_handler))
        .route("/v1/completions", post(completions_handler))
        .route("/v1/responses", post(responses_handler))
        .route(
            "/v1/responses/{response_id}",
            get(response_retrieve_handler).delete(response_delete_handler),
        )
        .route(
            "/v1/responses/{response_id}/input_items",
            get(response_input_items_list_handler),
        )
        .route("/v1/embeddings", post(embeddings_handler))
        .route("/v1/moderations", post(moderations_handler))
        .route("/v1/images/generations", post(image_generations_handler))
        .route("/v1/audio/transcriptions", post(transcriptions_handler))
        .route("/v1/audio/translations", post(translations_handler))
        .route("/v1/audio/speech", post(audio_speech_handler))
        .route("/v1/files", get(files_list_handler).post(files_handler))
        .route(
            "/v1/files/{file_id}",
            get(file_retrieve_handler).delete(file_delete_handler),
        )
        .route("/v1/files/{file_id}/content", get(file_content_handler))
        .route("/v1/videos", get(videos_list_handler).post(videos_handler))
        .route(
            "/v1/videos/{video_id}",
            get(video_retrieve_handler).delete(video_delete_handler),
        )
        .route("/v1/videos/{video_id}/content", get(video_content_handler))
        .route("/v1/videos/{video_id}/remix", post(video_remix_handler))
        .route("/v1/uploads", post(uploads_handler))
        .route("/v1/uploads/{upload_id}/parts", post(upload_parts_handler))
        .route(
            "/v1/uploads/{upload_id}/complete",
            post(upload_complete_handler),
        )
        .route(
            "/v1/uploads/{upload_id}/cancel",
            post(upload_cancel_handler),
        )
        .route(
            "/v1/fine_tuning/jobs",
            get(fine_tuning_jobs_list_handler).post(fine_tuning_jobs_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}",
            get(fine_tuning_job_retrieve_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}/cancel",
            post(fine_tuning_job_cancel_handler),
        )
        .route(
            "/v1/assistants",
            get(assistants_list_handler).post(assistants_handler),
        )
        .route(
            "/v1/assistants/{assistant_id}",
            get(assistant_retrieve_handler)
                .post(assistant_update_handler)
                .delete(assistant_delete_handler),
        )
        .route(
            "/v1/webhooks",
            get(webhooks_list_handler).post(webhooks_handler),
        )
        .route(
            "/v1/webhooks/{webhook_id}",
            get(webhook_retrieve_handler)
                .post(webhook_update_handler)
                .delete(webhook_delete_handler),
        )
        .route("/v1/realtime/sessions", post(realtime_sessions_handler))
        .route("/v1/evals", post(evals_handler))
        .route(
            "/v1/batches",
            get(batches_list_handler).post(batches_handler),
        )
        .route("/v1/batches/{batch_id}", get(batch_retrieve_handler))
        .route("/v1/batches/{batch_id}/cancel", post(batch_cancel_handler))
        .route(
            "/v1/vector_stores",
            get(vector_stores_list_handler).post(vector_stores_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}",
            get(vector_store_retrieve_handler)
                .post(vector_store_update_handler)
                .delete(vector_store_delete_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/search",
            post(vector_store_search_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/files",
            get(vector_store_files_list_handler).post(vector_store_files_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/files/{file_id}",
            get(vector_store_file_retrieve_handler).delete(vector_store_file_delete_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/file_batches",
            post(vector_store_file_batches_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/file_batches/{batch_id}",
            get(vector_store_file_batch_retrieve_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/file_batches/{batch_id}/cancel",
            post(vector_store_file_batch_cancel_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/file_batches/{batch_id}/files",
            get(vector_store_file_batch_files_handler),
        )
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
            "/v1/models/{model_id}",
            get(model_retrieve_from_store_handler),
        )
        .route(
            "/v1/chat/completions",
            post(chat_completions_with_state_handler),
        )
        .route("/v1/completions", post(completions_with_state_handler))
        .route("/v1/responses", post(responses_with_state_handler))
        .route(
            "/v1/responses/{response_id}",
            get(response_retrieve_with_state_handler).delete(response_delete_with_state_handler),
        )
        .route(
            "/v1/responses/{response_id}/input_items",
            get(response_input_items_list_with_state_handler),
        )
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
        .route("/v1/audio/speech", post(audio_speech_with_state_handler))
        .route(
            "/v1/files",
            get(files_list_with_state_handler).post(files_with_state_handler),
        )
        .route(
            "/v1/files/{file_id}",
            get(file_retrieve_with_state_handler).delete(file_delete_with_state_handler),
        )
        .route(
            "/v1/files/{file_id}/content",
            get(file_content_with_state_handler),
        )
        .route(
            "/v1/videos",
            get(videos_list_with_state_handler).post(videos_with_state_handler),
        )
        .route(
            "/v1/videos/{video_id}",
            get(video_retrieve_with_state_handler).delete(video_delete_with_state_handler),
        )
        .route(
            "/v1/videos/{video_id}/content",
            get(video_content_with_state_handler),
        )
        .route(
            "/v1/videos/{video_id}/remix",
            post(video_remix_with_state_handler),
        )
        .route("/v1/uploads", post(uploads_with_state_handler))
        .route(
            "/v1/uploads/{upload_id}/parts",
            post(upload_parts_with_state_handler),
        )
        .route(
            "/v1/uploads/{upload_id}/complete",
            post(upload_complete_with_state_handler),
        )
        .route(
            "/v1/uploads/{upload_id}/cancel",
            post(upload_cancel_with_state_handler),
        )
        .route(
            "/v1/fine_tuning/jobs",
            get(fine_tuning_jobs_list_with_state_handler).post(fine_tuning_jobs_with_state_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}",
            get(fine_tuning_job_retrieve_with_state_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}/cancel",
            post(fine_tuning_job_cancel_with_state_handler),
        )
        .route(
            "/v1/assistants",
            get(assistants_list_with_state_handler).post(assistants_with_state_handler),
        )
        .route(
            "/v1/assistants/{assistant_id}",
            get(assistant_retrieve_with_state_handler)
                .post(assistant_update_with_state_handler)
                .delete(assistant_delete_with_state_handler),
        )
        .route(
            "/v1/webhooks",
            get(webhooks_list_with_state_handler).post(webhooks_with_state_handler),
        )
        .route(
            "/v1/webhooks/{webhook_id}",
            get(webhook_retrieve_with_state_handler)
                .post(webhook_update_with_state_handler)
                .delete(webhook_delete_with_state_handler),
        )
        .route(
            "/v1/realtime/sessions",
            post(realtime_sessions_with_state_handler),
        )
        .route("/v1/evals", post(evals_with_state_handler))
        .route(
            "/v1/batches",
            get(batches_list_with_state_handler).post(batches_with_state_handler),
        )
        .route(
            "/v1/batches/{batch_id}",
            get(batch_retrieve_with_state_handler),
        )
        .route(
            "/v1/batches/{batch_id}/cancel",
            post(batch_cancel_with_state_handler),
        )
        .route(
            "/v1/vector_stores",
            get(vector_stores_list_with_state_handler).post(vector_stores_with_state_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}",
            get(vector_store_retrieve_with_state_handler)
                .post(vector_store_update_with_state_handler)
                .delete(vector_store_delete_with_state_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/search",
            post(vector_store_search_with_state_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/files",
            get(vector_store_files_list_with_state_handler)
                .post(vector_store_files_with_state_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/files/{file_id}",
            get(vector_store_file_retrieve_with_state_handler)
                .delete(vector_store_file_delete_with_state_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/file_batches",
            post(vector_store_file_batches_with_state_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/file_batches/{batch_id}",
            get(vector_store_file_batch_retrieve_with_state_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/file_batches/{batch_id}/cancel",
            post(vector_store_file_batch_cancel_with_state_handler),
        )
        .route(
            "/v1/vector_stores/{vector_store_id}/file_batches/{batch_id}/files",
            get(vector_store_file_batch_files_with_state_handler),
        )
        .with_state(GatewayApiState::with_store_and_secret_manager(
            store,
            secret_manager,
        ))
}

async fn list_models_handler() -> Json<sdkwork_api_contract_openai::models::ListModelsResponse> {
    Json(list_models("tenant-1", "project-1").expect("models response"))
}

async fn model_retrieve_handler(
    Path(model_id): Path<String>,
) -> Json<sdkwork_api_contract_openai::models::ModelObject> {
    Json(get_model("tenant-1", "project-1", &model_id).expect("model response"))
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

async fn model_retrieve_from_store_handler(
    State(state): State<GatewayApiState>,
    Path(model_id): Path<String>,
) -> Result<Json<sdkwork_api_contract_openai::models::ModelObject>, Response> {
    get_model_from_store(state.store.as_ref(), "tenant-1", "project-1", &model_id)
        .await
        .map_err(|_| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to load model",
            )
                .into_response()
        })?
        .map(Json)
        .ok_or_else(|| (axum::http::StatusCode::NOT_FOUND, "model not found").into_response())
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
) -> Json<ResponseObject> {
    Json(create_response("tenant-1", "project-1", &request.model).expect("response"))
}

async fn response_retrieve_handler(Path(response_id): Path<String>) -> Json<ResponseObject> {
    Json(get_response("tenant-1", "project-1", &response_id).expect("response retrieve"))
}

async fn response_input_items_list_handler(
    Path(response_id): Path<String>,
) -> Json<ListResponseInputItemsResponse> {
    Json(
        list_response_input_items("tenant-1", "project-1", &response_id)
            .expect("response input items"),
    )
}

async fn response_delete_handler(Path(response_id): Path<String>) -> Json<DeleteResponseResponse> {
    Json(delete_response("tenant-1", "project-1", &response_id).expect("response delete"))
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

async fn audio_speech_handler(ExtractJson(request): ExtractJson<CreateSpeechRequest>) -> Response {
    local_speech_response(&request)
}

async fn files_handler(multipart: Multipart) -> Response {
    match parse_file_request(multipart).await {
        Ok(request) => {
            Json(create_file("tenant-1", "project-1", &request).expect("file")).into_response()
        }
        Err(response) => response,
    }
}

async fn files_list_handler() -> Json<sdkwork_api_contract_openai::files::ListFilesResponse> {
    Json(list_files("tenant-1", "project-1").expect("files list"))
}

async fn file_retrieve_handler(
    Path(file_id): Path<String>,
) -> Json<sdkwork_api_contract_openai::files::FileObject> {
    Json(get_file("tenant-1", "project-1", &file_id).expect("file retrieve"))
}

async fn file_delete_handler(
    Path(file_id): Path<String>,
) -> Json<sdkwork_api_contract_openai::files::DeleteFileResponse> {
    Json(delete_file("tenant-1", "project-1", &file_id).expect("file delete"))
}

async fn file_content_handler(Path(file_id): Path<String>) -> Response {
    local_file_content_response(&file_id)
}

async fn videos_handler(
    ExtractJson(request): ExtractJson<CreateVideoRequest>,
) -> Json<sdkwork_api_contract_openai::videos::VideosResponse> {
    Json(create_video("tenant-1", "project-1", &request.model, &request.prompt).expect("video"))
}

async fn videos_list_handler() -> Json<sdkwork_api_contract_openai::videos::VideosResponse> {
    Json(list_videos("tenant-1", "project-1").expect("videos list"))
}

async fn video_retrieve_handler(
    Path(video_id): Path<String>,
) -> Json<sdkwork_api_contract_openai::videos::VideoObject> {
    Json(get_video("tenant-1", "project-1", &video_id).expect("video retrieve"))
}

async fn video_delete_handler(
    Path(video_id): Path<String>,
) -> Json<sdkwork_api_contract_openai::videos::DeleteVideoResponse> {
    Json(delete_video("tenant-1", "project-1", &video_id).expect("video delete"))
}

async fn video_content_handler(Path(video_id): Path<String>) -> Response {
    local_video_content_response(&video_id)
}

async fn video_remix_handler(
    Path(video_id): Path<String>,
    ExtractJson(request): ExtractJson<RemixVideoRequest>,
) -> Json<sdkwork_api_contract_openai::videos::VideosResponse> {
    Json(remix_video("tenant-1", "project-1", &video_id, &request.prompt).expect("video remix"))
}

async fn uploads_handler(
    ExtractJson(request): ExtractJson<CreateUploadRequest>,
) -> Json<sdkwork_api_contract_openai::uploads::UploadObject> {
    Json(create_upload("tenant-1", "project-1", &request).expect("upload"))
}

async fn upload_parts_handler(Path(upload_id): Path<String>, multipart: Multipart) -> Response {
    match parse_upload_part_request(upload_id, multipart).await {
        Ok(request) => {
            Json(create_upload_part("tenant-1", "project-1", &request).expect("upload part"))
                .into_response()
        }
        Err(response) => response,
    }
}

async fn upload_complete_handler(
    Path(upload_id): Path<String>,
    ExtractJson(mut request): ExtractJson<CompleteUploadRequest>,
) -> Json<sdkwork_api_contract_openai::uploads::UploadObject> {
    request.upload_id = upload_id;
    Json(complete_upload("tenant-1", "project-1", &request).expect("upload complete"))
}

async fn upload_cancel_handler(
    Path(upload_id): Path<String>,
) -> Json<sdkwork_api_contract_openai::uploads::UploadObject> {
    Json(cancel_upload("tenant-1", "project-1", &upload_id).expect("upload cancel"))
}

async fn fine_tuning_jobs_handler(
    ExtractJson(request): ExtractJson<CreateFineTuningJobRequest>,
) -> Json<sdkwork_api_contract_openai::fine_tuning::FineTuningJobObject> {
    Json(create_fine_tuning_job("tenant-1", "project-1", &request.model).expect("fine tuning"))
}

async fn fine_tuning_jobs_list_handler(
) -> Json<sdkwork_api_contract_openai::fine_tuning::ListFineTuningJobsResponse> {
    Json(list_fine_tuning_jobs("tenant-1", "project-1").expect("fine tuning list"))
}

async fn fine_tuning_job_retrieve_handler(
    Path(fine_tuning_job_id): Path<String>,
) -> Json<sdkwork_api_contract_openai::fine_tuning::FineTuningJobObject> {
    Json(
        get_fine_tuning_job("tenant-1", "project-1", &fine_tuning_job_id)
            .expect("fine tuning retrieve"),
    )
}

async fn fine_tuning_job_cancel_handler(
    Path(fine_tuning_job_id): Path<String>,
) -> Json<sdkwork_api_contract_openai::fine_tuning::FineTuningJobObject> {
    Json(
        cancel_fine_tuning_job("tenant-1", "project-1", &fine_tuning_job_id)
            .expect("fine tuning cancel"),
    )
}

async fn assistants_handler(
    ExtractJson(request): ExtractJson<CreateAssistantRequest>,
) -> Json<sdkwork_api_contract_openai::assistants::AssistantObject> {
    Json(
        create_assistant("tenant-1", "project-1", &request.name, &request.model)
            .expect("assistant"),
    )
}

async fn assistants_list_handler(
) -> Json<sdkwork_api_contract_openai::assistants::ListAssistantsResponse> {
    Json(list_assistants("tenant-1", "project-1").expect("assistants list"))
}

async fn assistant_retrieve_handler(
    Path(assistant_id): Path<String>,
) -> Json<sdkwork_api_contract_openai::assistants::AssistantObject> {
    Json(get_assistant("tenant-1", "project-1", &assistant_id).expect("assistant retrieve"))
}

async fn assistant_update_handler(
    Path(assistant_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateAssistantRequest>,
) -> Json<sdkwork_api_contract_openai::assistants::AssistantObject> {
    Json(
        update_assistant(
            "tenant-1",
            "project-1",
            &assistant_id,
            request.name.as_deref().unwrap_or("assistant"),
        )
        .expect("assistant update"),
    )
}

async fn assistant_delete_handler(
    Path(assistant_id): Path<String>,
) -> Json<sdkwork_api_contract_openai::assistants::DeleteAssistantResponse> {
    Json(delete_assistant("tenant-1", "project-1", &assistant_id).expect("assistant delete"))
}

async fn webhooks_handler(
    ExtractJson(request): ExtractJson<CreateWebhookRequest>,
) -> Json<sdkwork_api_contract_openai::webhooks::WebhookObject> {
    Json(create_webhook("tenant-1", "project-1", &request.url, &request.events).expect("webhook"))
}

async fn webhooks_list_handler() -> Json<sdkwork_api_contract_openai::webhooks::ListWebhooksResponse>
{
    Json(list_webhooks("tenant-1", "project-1").expect("webhooks list"))
}

async fn webhook_retrieve_handler(
    Path(webhook_id): Path<String>,
) -> Json<sdkwork_api_contract_openai::webhooks::WebhookObject> {
    Json(get_webhook("tenant-1", "project-1", &webhook_id).expect("webhook retrieve"))
}

async fn webhook_update_handler(
    Path(webhook_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateWebhookRequest>,
) -> Json<sdkwork_api_contract_openai::webhooks::WebhookObject> {
    Json(
        update_webhook(
            "tenant-1",
            "project-1",
            &webhook_id,
            request
                .url
                .as_deref()
                .unwrap_or("https://example.com/webhook"),
        )
        .expect("webhook update"),
    )
}

async fn webhook_delete_handler(
    Path(webhook_id): Path<String>,
) -> Json<sdkwork_api_contract_openai::webhooks::DeleteWebhookResponse> {
    Json(delete_webhook("tenant-1", "project-1", &webhook_id).expect("webhook delete"))
}

async fn realtime_sessions_handler(
    ExtractJson(request): ExtractJson<CreateRealtimeSessionRequest>,
) -> Json<sdkwork_api_contract_openai::realtime::RealtimeSessionObject> {
    Json(
        create_realtime_session("tenant-1", "project-1", &request.model).expect("realtime session"),
    )
}

async fn evals_handler(
    ExtractJson(request): ExtractJson<CreateEvalRequest>,
) -> Json<sdkwork_api_contract_openai::evals::EvalObject> {
    Json(create_eval("tenant-1", "project-1", &request.name).expect("eval"))
}

async fn batches_handler(
    ExtractJson(request): ExtractJson<CreateBatchRequest>,
) -> Json<sdkwork_api_contract_openai::batches::BatchObject> {
    Json(
        create_batch(
            "tenant-1",
            "project-1",
            &request.endpoint,
            &request.input_file_id,
        )
        .expect("batch"),
    )
}

async fn batches_list_handler() -> Json<sdkwork_api_contract_openai::batches::ListBatchesResponse> {
    Json(list_batches("tenant-1", "project-1").expect("batches list"))
}

async fn batch_retrieve_handler(
    Path(batch_id): Path<String>,
) -> Json<sdkwork_api_contract_openai::batches::BatchObject> {
    Json(get_batch("tenant-1", "project-1", &batch_id).expect("batch retrieve"))
}

async fn batch_cancel_handler(
    Path(batch_id): Path<String>,
) -> Json<sdkwork_api_contract_openai::batches::BatchObject> {
    Json(cancel_batch("tenant-1", "project-1", &batch_id).expect("batch cancel"))
}

async fn vector_stores_list_handler(
) -> Json<sdkwork_api_contract_openai::vector_stores::ListVectorStoresResponse> {
    Json(list_vector_stores("tenant-1", "project-1").expect("vector stores list"))
}

async fn vector_stores_handler(
    ExtractJson(request): ExtractJson<CreateVectorStoreRequest>,
) -> Json<sdkwork_api_contract_openai::vector_stores::VectorStoreObject> {
    Json(create_vector_store("tenant-1", "project-1", &request.name).expect("vector store"))
}

async fn vector_store_retrieve_handler(
    Path(vector_store_id): Path<String>,
) -> Json<sdkwork_api_contract_openai::vector_stores::VectorStoreObject> {
    Json(
        get_vector_store("tenant-1", "project-1", &vector_store_id).expect("vector store retrieve"),
    )
}

async fn vector_store_update_handler(
    Path(vector_store_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateVectorStoreRequest>,
) -> Json<sdkwork_api_contract_openai::vector_stores::VectorStoreObject> {
    Json(
        update_vector_store(
            "tenant-1",
            "project-1",
            &vector_store_id,
            request.name.as_deref().unwrap_or("vector-store"),
        )
        .expect("vector store update"),
    )
}

async fn vector_store_delete_handler(
    Path(vector_store_id): Path<String>,
) -> Json<sdkwork_api_contract_openai::vector_stores::DeleteVectorStoreResponse> {
    Json(
        delete_vector_store("tenant-1", "project-1", &vector_store_id)
            .expect("vector store delete"),
    )
}

async fn vector_store_search_handler(
    Path(vector_store_id): Path<String>,
    ExtractJson(request): ExtractJson<SearchVectorStoreRequest>,
) -> Json<sdkwork_api_contract_openai::vector_stores::SearchVectorStoreResponse> {
    Json(
        search_vector_store("tenant-1", "project-1", &vector_store_id, &request.query)
            .expect("vector store search"),
    )
}

async fn vector_store_files_handler(
    Path(vector_store_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateVectorStoreFileRequest>,
) -> Json<sdkwork_api_contract_openai::vector_stores::VectorStoreFileObject> {
    Json(
        create_vector_store_file("tenant-1", "project-1", &vector_store_id, &request.file_id)
            .expect("vector store file"),
    )
}

async fn vector_store_files_list_handler(
    Path(vector_store_id): Path<String>,
) -> Json<sdkwork_api_contract_openai::vector_stores::ListVectorStoreFilesResponse> {
    Json(
        list_vector_store_files("tenant-1", "project-1", &vector_store_id)
            .expect("vector store files list"),
    )
}

async fn vector_store_file_retrieve_handler(
    Path((vector_store_id, file_id)): Path<(String, String)>,
) -> Json<sdkwork_api_contract_openai::vector_stores::VectorStoreFileObject> {
    Json(
        get_vector_store_file("tenant-1", "project-1", &vector_store_id, &file_id)
            .expect("vector store file retrieve"),
    )
}

async fn vector_store_file_delete_handler(
    Path((vector_store_id, file_id)): Path<(String, String)>,
) -> Json<sdkwork_api_contract_openai::vector_stores::DeleteVectorStoreFileResponse> {
    Json(
        delete_vector_store_file("tenant-1", "project-1", &vector_store_id, &file_id)
            .expect("vector store file delete"),
    )
}

async fn vector_store_file_batches_handler(
    Path(vector_store_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateVectorStoreFileBatchRequest>,
) -> Json<sdkwork_api_contract_openai::vector_stores::VectorStoreFileBatchObject> {
    Json(
        create_vector_store_file_batch(
            "tenant-1",
            "project-1",
            &vector_store_id,
            &request.file_ids,
        )
        .expect("vector store file batch"),
    )
}

async fn vector_store_file_batch_retrieve_handler(
    Path((vector_store_id, batch_id)): Path<(String, String)>,
) -> Json<sdkwork_api_contract_openai::vector_stores::VectorStoreFileBatchObject> {
    Json(
        get_vector_store_file_batch("tenant-1", "project-1", &vector_store_id, &batch_id)
            .expect("vector store file batch retrieve"),
    )
}

async fn vector_store_file_batch_cancel_handler(
    Path((vector_store_id, batch_id)): Path<(String, String)>,
) -> Json<sdkwork_api_contract_openai::vector_stores::VectorStoreFileBatchObject> {
    Json(
        cancel_vector_store_file_batch("tenant-1", "project-1", &vector_store_id, &batch_id)
            .expect("vector store file batch cancel"),
    )
}

async fn vector_store_file_batch_files_handler(
    Path((vector_store_id, batch_id)): Path<(String, String)>,
) -> Json<sdkwork_api_contract_openai::vector_stores::ListVectorStoreFilesResponse> {
    Json(
        list_vector_store_file_batch_files("tenant-1", "project-1", &vector_store_id, &batch_id)
            .expect("vector store file batch files"),
    )
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

                return upstream_passthrough_response(response);
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

fn upstream_passthrough_response(response: reqwest::Response) -> Response {
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

fn local_file_content_response(file_id: &str) -> Response {
    let bytes = file_content("tenant-1", "project-1", file_id).expect("file content");
    Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/jsonl")
        .body(Body::from(bytes))
        .expect("valid local file content response")
}

fn local_video_content_response(video_id: &str) -> Response {
    let bytes = video_content("tenant-1", "project-1", video_id).expect("video content");
    Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(header::CONTENT_TYPE, "video/mp4")
        .body(Body::from(bytes))
        .expect("valid local video content response")
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

async fn response_retrieve_with_state_handler(
    State(state): State<GatewayApiState>,
    Path(response_id): Path<String>,
) -> Response {
    match relay_get_response_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &response_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(state.store.as_ref(), "responses", &response_id, 20, 0.02)
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
                "failed to relay upstream response retrieve",
            )
                .into_response();
        }
    }

    if record_gateway_usage(state.store.as_ref(), "responses", &response_id, 20, 0.02)
        .await
        .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(get_response("tenant-1", "project-1", &response_id).expect("response retrieve"))
        .into_response()
}

async fn response_input_items_list_with_state_handler(
    State(state): State<GatewayApiState>,
    Path(response_id): Path<String>,
) -> Response {
    match relay_list_response_input_items_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &response_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(state.store.as_ref(), "responses", &response_id, 20, 0.02)
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
                "failed to relay upstream response input items",
            )
                .into_response();
        }
    }

    if record_gateway_usage(state.store.as_ref(), "responses", &response_id, 20, 0.02)
        .await
        .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(
        list_response_input_items("tenant-1", "project-1", &response_id)
            .expect("response input items"),
    )
    .into_response()
}

async fn response_delete_with_state_handler(
    State(state): State<GatewayApiState>,
    Path(response_id): Path<String>,
) -> Response {
    match relay_delete_response_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &response_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(state.store.as_ref(), "responses", &response_id, 20, 0.02)
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
                "failed to relay upstream response delete",
            )
                .into_response();
        }
    }

    if record_gateway_usage(state.store.as_ref(), "responses", &response_id, 20, 0.02)
        .await
        .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(delete_response("tenant-1", "project-1", &response_id).expect("response delete"))
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

async fn audio_speech_with_state_handler(
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateSpeechRequest>,
) -> Response {
    match relay_speech_from_store(
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
                "audio_speech",
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

            return upstream_passthrough_response(response);
        }
        Ok(None) => {}
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_GATEWAY,
                "failed to relay upstream speech",
            )
                .into_response();
        }
    }

    if record_gateway_usage(
        state.store.as_ref(),
        "audio_speech",
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

    local_speech_response(&request)
}

async fn files_with_state_handler(
    State(state): State<GatewayApiState>,
    multipart: Multipart,
) -> Response {
    let request = match parse_file_request(multipart).await {
        Ok(request) => request,
        Err(response) => return response,
    };

    match relay_file_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(state.store.as_ref(), "files", &request.purpose, 5, 0.005)
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
                "failed to relay upstream file",
            )
                .into_response();
        }
    }

    if record_gateway_usage(state.store.as_ref(), "files", &request.purpose, 5, 0.005)
        .await
        .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(create_file("tenant-1", "project-1", &request).expect("file")).into_response()
}

async fn files_list_with_state_handler(State(state): State<GatewayApiState>) -> Response {
    match relay_list_files_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(state.store.as_ref(), "files", "list", 1, 0.001)
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
                "failed to relay upstream files list",
            )
                .into_response();
        }
    }

    if record_gateway_usage(state.store.as_ref(), "files", "list", 1, 0.001)
        .await
        .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(list_files("tenant-1", "project-1").expect("files list")).into_response()
}

async fn file_retrieve_with_state_handler(
    State(state): State<GatewayApiState>,
    Path(file_id): Path<String>,
) -> Response {
    match relay_get_file_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &file_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(state.store.as_ref(), "files", &file_id, 1, 0.001)
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
                "failed to relay upstream file retrieve",
            )
                .into_response();
        }
    }

    if record_gateway_usage(state.store.as_ref(), "files", &file_id, 1, 0.001)
        .await
        .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(get_file("tenant-1", "project-1", &file_id).expect("file retrieve")).into_response()
}

async fn file_delete_with_state_handler(
    State(state): State<GatewayApiState>,
    Path(file_id): Path<String>,
) -> Response {
    match relay_delete_file_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &file_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(state.store.as_ref(), "files", &file_id, 1, 0.001)
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
                "failed to relay upstream file delete",
            )
                .into_response();
        }
    }

    if record_gateway_usage(state.store.as_ref(), "files", &file_id, 1, 0.001)
        .await
        .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(delete_file("tenant-1", "project-1", &file_id).expect("file delete")).into_response()
}

async fn file_content_with_state_handler(
    State(state): State<GatewayApiState>,
    Path(file_id): Path<String>,
) -> Response {
    match relay_file_content_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &file_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(state.store.as_ref(), "files", &file_id, 1, 0.001)
                .await
                .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return upstream_passthrough_response(response);
        }
        Ok(None) => {}
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_GATEWAY,
                "failed to relay upstream file content",
            )
                .into_response();
        }
    }

    if record_gateway_usage(state.store.as_ref(), "files", &file_id, 1, 0.001)
        .await
        .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    local_file_content_response(&file_id)
}

async fn videos_with_state_handler(
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateVideoRequest>,
) -> Response {
    match relay_video_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(state.store.as_ref(), "videos", &request.model, 90, 0.09)
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
                "failed to relay upstream video create",
            )
                .into_response();
        }
    }

    if record_gateway_usage(state.store.as_ref(), "videos", &request.model, 90, 0.09)
        .await
        .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(create_video("tenant-1", "project-1", &request.model, &request.prompt).expect("video"))
        .into_response()
}

async fn videos_list_with_state_handler(State(state): State<GatewayApiState>) -> Response {
    match relay_list_videos_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(state.store.as_ref(), "videos", "videos", 20, 0.02)
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
                "failed to relay upstream videos list",
            )
                .into_response();
        }
    }

    if record_gateway_usage(state.store.as_ref(), "videos", "videos", 20, 0.02)
        .await
        .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(list_videos("tenant-1", "project-1").expect("videos list")).into_response()
}

async fn video_retrieve_with_state_handler(
    State(state): State<GatewayApiState>,
    Path(video_id): Path<String>,
) -> Response {
    match relay_get_video_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &video_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(state.store.as_ref(), "videos", &video_id, 20, 0.02)
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
                "failed to relay upstream video retrieve",
            )
                .into_response();
        }
    }

    if record_gateway_usage(state.store.as_ref(), "videos", &video_id, 20, 0.02)
        .await
        .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(get_video("tenant-1", "project-1", &video_id).expect("video retrieve")).into_response()
}

async fn video_delete_with_state_handler(
    State(state): State<GatewayApiState>,
    Path(video_id): Path<String>,
) -> Response {
    match relay_delete_video_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &video_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(state.store.as_ref(), "videos", &video_id, 20, 0.02)
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
                "failed to relay upstream video delete",
            )
                .into_response();
        }
    }

    if record_gateway_usage(state.store.as_ref(), "videos", &video_id, 20, 0.02)
        .await
        .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(delete_video("tenant-1", "project-1", &video_id).expect("video delete")).into_response()
}

async fn video_content_with_state_handler(
    State(state): State<GatewayApiState>,
    Path(video_id): Path<String>,
) -> Response {
    match relay_video_content_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &video_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(state.store.as_ref(), "videos", &video_id, 20, 0.02)
                .await
                .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return upstream_passthrough_response(response);
        }
        Ok(None) => {}
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_GATEWAY,
                "failed to relay upstream video content",
            )
                .into_response();
        }
    }

    if record_gateway_usage(state.store.as_ref(), "videos", &video_id, 20, 0.02)
        .await
        .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    local_video_content_response(&video_id)
}

async fn video_remix_with_state_handler(
    State(state): State<GatewayApiState>,
    Path(video_id): Path<String>,
    ExtractJson(request): ExtractJson<RemixVideoRequest>,
) -> Response {
    match relay_remix_video_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &video_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(state.store.as_ref(), "videos", &video_id, 60, 0.06)
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
                "failed to relay upstream video remix",
            )
                .into_response();
        }
    }

    if record_gateway_usage(state.store.as_ref(), "videos", &video_id, 60, 0.06)
        .await
        .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(remix_video("tenant-1", "project-1", &video_id, &request.prompt).expect("video remix"))
        .into_response()
}

async fn uploads_with_state_handler(
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateUploadRequest>,
) -> Response {
    match relay_upload_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(state.store.as_ref(), "uploads", &request.purpose, 8, 0.008)
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
                "failed to relay upstream upload",
            )
                .into_response();
        }
    }

    if record_gateway_usage(state.store.as_ref(), "uploads", &request.purpose, 8, 0.008)
        .await
        .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(create_upload("tenant-1", "project-1", &request).expect("upload")).into_response()
}

async fn upload_parts_with_state_handler(
    State(state): State<GatewayApiState>,
    Path(upload_id): Path<String>,
    multipart: Multipart,
) -> Response {
    let request = match parse_upload_part_request(upload_id, multipart).await {
        Ok(request) => request,
        Err(response) => return response,
    };

    match relay_upload_part_from_store(
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
                "uploads",
                &request.upload_id,
                4,
                0.004,
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
                "failed to relay upstream upload part",
            )
                .into_response();
        }
    }

    if record_gateway_usage(
        state.store.as_ref(),
        "uploads",
        &request.upload_id,
        4,
        0.004,
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

    Json(create_upload_part("tenant-1", "project-1", &request).expect("upload part"))
        .into_response()
}

async fn upload_complete_with_state_handler(
    State(state): State<GatewayApiState>,
    Path(upload_id): Path<String>,
    ExtractJson(mut request): ExtractJson<CompleteUploadRequest>,
) -> Response {
    request.upload_id = upload_id;

    match relay_complete_upload_from_store(
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
                "uploads",
                &request.upload_id,
                4,
                0.004,
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
                "failed to relay upstream upload completion",
            )
                .into_response();
        }
    }

    if record_gateway_usage(
        state.store.as_ref(),
        "uploads",
        &request.upload_id,
        4,
        0.004,
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

    Json(complete_upload("tenant-1", "project-1", &request).expect("upload complete"))
        .into_response()
}

async fn upload_cancel_with_state_handler(
    State(state): State<GatewayApiState>,
    Path(upload_id): Path<String>,
) -> Response {
    match relay_cancel_upload_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &upload_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(state.store.as_ref(), "uploads", &upload_id, 4, 0.004)
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
                "failed to relay upstream upload cancellation",
            )
                .into_response();
        }
    }

    if record_gateway_usage(state.store.as_ref(), "uploads", &upload_id, 4, 0.004)
        .await
        .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(cancel_upload("tenant-1", "project-1", &upload_id).expect("upload cancel")).into_response()
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

async fn fine_tuning_jobs_list_with_state_handler(
    State(state): State<GatewayApiState>,
) -> Response {
    match relay_list_fine_tuning_jobs_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(state.store.as_ref(), "fine_tuning", "jobs", 20, 0.02)
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
                "failed to relay upstream fine tuning jobs list",
            )
                .into_response();
        }
    }

    if record_gateway_usage(state.store.as_ref(), "fine_tuning", "jobs", 20, 0.02)
        .await
        .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(list_fine_tuning_jobs("tenant-1", "project-1").expect("fine tuning list")).into_response()
}

async fn fine_tuning_job_retrieve_with_state_handler(
    State(state): State<GatewayApiState>,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match relay_get_fine_tuning_job_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &fine_tuning_job_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(
                state.store.as_ref(),
                "fine_tuning",
                &fine_tuning_job_id,
                20,
                0.02,
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
                "failed to relay upstream fine tuning job retrieve",
            )
                .into_response();
        }
    }

    if record_gateway_usage(
        state.store.as_ref(),
        "fine_tuning",
        &fine_tuning_job_id,
        20,
        0.02,
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

    Json(
        get_fine_tuning_job("tenant-1", "project-1", &fine_tuning_job_id)
            .expect("fine tuning retrieve"),
    )
    .into_response()
}

async fn fine_tuning_job_cancel_with_state_handler(
    State(state): State<GatewayApiState>,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match relay_cancel_fine_tuning_job_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &fine_tuning_job_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(
                state.store.as_ref(),
                "fine_tuning",
                &fine_tuning_job_id,
                20,
                0.02,
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
                "failed to relay upstream fine tuning job cancel",
            )
                .into_response();
        }
    }

    if record_gateway_usage(
        state.store.as_ref(),
        "fine_tuning",
        &fine_tuning_job_id,
        20,
        0.02,
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

    Json(
        cancel_fine_tuning_job("tenant-1", "project-1", &fine_tuning_job_id)
            .expect("fine tuning cancel"),
    )
    .into_response()
}

async fn assistants_with_state_handler(
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateAssistantRequest>,
) -> Response {
    match relay_assistant_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(state.store.as_ref(), "assistants", &request.model, 20, 0.02)
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
                "failed to relay upstream assistant",
            )
                .into_response();
        }
    }

    if record_gateway_usage(state.store.as_ref(), "assistants", &request.model, 20, 0.02)
        .await
        .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(
        create_assistant("tenant-1", "project-1", &request.name, &request.model)
            .expect("assistant"),
    )
    .into_response()
}

async fn assistants_list_with_state_handler(State(state): State<GatewayApiState>) -> Response {
    match relay_list_assistants_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(state.store.as_ref(), "assistants", "assistants", 20, 0.02)
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
                "failed to relay upstream assistants list",
            )
                .into_response();
        }
    }

    if record_gateway_usage(state.store.as_ref(), "assistants", "assistants", 20, 0.02)
        .await
        .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(list_assistants("tenant-1", "project-1").expect("assistants list")).into_response()
}

async fn assistant_retrieve_with_state_handler(
    State(state): State<GatewayApiState>,
    Path(assistant_id): Path<String>,
) -> Response {
    match relay_get_assistant_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &assistant_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(state.store.as_ref(), "assistants", &assistant_id, 20, 0.02)
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
                "failed to relay upstream assistant retrieve",
            )
                .into_response();
        }
    }

    if record_gateway_usage(state.store.as_ref(), "assistants", &assistant_id, 20, 0.02)
        .await
        .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(get_assistant("tenant-1", "project-1", &assistant_id).expect("assistant retrieve"))
        .into_response()
}

async fn assistant_update_with_state_handler(
    State(state): State<GatewayApiState>,
    Path(assistant_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateAssistantRequest>,
) -> Response {
    match relay_update_assistant_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &assistant_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_target = request.model.as_deref().unwrap_or(assistant_id.as_str());
            if record_gateway_usage(state.store.as_ref(), "assistants", usage_target, 20, 0.02)
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
                "failed to relay upstream assistant update",
            )
                .into_response();
        }
    }

    let usage_target = request.model.as_deref().unwrap_or(assistant_id.as_str());
    if record_gateway_usage(state.store.as_ref(), "assistants", usage_target, 20, 0.02)
        .await
        .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(
        update_assistant(
            "tenant-1",
            "project-1",
            &assistant_id,
            request.name.as_deref().unwrap_or("assistant"),
        )
        .expect("assistant update"),
    )
    .into_response()
}

async fn assistant_delete_with_state_handler(
    State(state): State<GatewayApiState>,
    Path(assistant_id): Path<String>,
) -> Response {
    match relay_delete_assistant_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &assistant_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(state.store.as_ref(), "assistants", &assistant_id, 20, 0.02)
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
                "failed to relay upstream assistant delete",
            )
                .into_response();
        }
    }

    if record_gateway_usage(state.store.as_ref(), "assistants", &assistant_id, 20, 0.02)
        .await
        .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(delete_assistant("tenant-1", "project-1", &assistant_id).expect("assistant delete"))
        .into_response()
}

async fn webhooks_with_state_handler(
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateWebhookRequest>,
) -> Response {
    match relay_webhook_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(state.store.as_ref(), "webhooks", &request.url, 20, 0.02)
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
                "failed to relay upstream webhook",
            )
                .into_response();
        }
    }

    if record_gateway_usage(state.store.as_ref(), "webhooks", &request.url, 20, 0.02)
        .await
        .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(create_webhook("tenant-1", "project-1", &request.url, &request.events).expect("webhook"))
        .into_response()
}

async fn webhooks_list_with_state_handler(State(state): State<GatewayApiState>) -> Response {
    match relay_list_webhooks_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(state.store.as_ref(), "webhooks", "webhooks", 20, 0.02)
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
                "failed to relay upstream webhooks list",
            )
                .into_response();
        }
    }

    if record_gateway_usage(state.store.as_ref(), "webhooks", "webhooks", 20, 0.02)
        .await
        .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(list_webhooks("tenant-1", "project-1").expect("webhooks list")).into_response()
}

async fn webhook_retrieve_with_state_handler(
    State(state): State<GatewayApiState>,
    Path(webhook_id): Path<String>,
) -> Response {
    match relay_get_webhook_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &webhook_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(state.store.as_ref(), "webhooks", &webhook_id, 20, 0.02)
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
                "failed to relay upstream webhook retrieve",
            )
                .into_response();
        }
    }

    if record_gateway_usage(state.store.as_ref(), "webhooks", &webhook_id, 20, 0.02)
        .await
        .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(get_webhook("tenant-1", "project-1", &webhook_id).expect("webhook retrieve"))
        .into_response()
}

async fn webhook_update_with_state_handler(
    State(state): State<GatewayApiState>,
    Path(webhook_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateWebhookRequest>,
) -> Response {
    match relay_update_webhook_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &webhook_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_target = request.url.as_deref().unwrap_or(webhook_id.as_str());
            if record_gateway_usage(state.store.as_ref(), "webhooks", usage_target, 20, 0.02)
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
                "failed to relay upstream webhook update",
            )
                .into_response();
        }
    }

    let usage_target = request.url.as_deref().unwrap_or(webhook_id.as_str());
    if record_gateway_usage(state.store.as_ref(), "webhooks", usage_target, 20, 0.02)
        .await
        .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(
        update_webhook(
            "tenant-1",
            "project-1",
            &webhook_id,
            request
                .url
                .as_deref()
                .unwrap_or("https://example.com/webhook"),
        )
        .expect("webhook update"),
    )
    .into_response()
}

async fn webhook_delete_with_state_handler(
    State(state): State<GatewayApiState>,
    Path(webhook_id): Path<String>,
) -> Response {
    match relay_delete_webhook_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &webhook_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(state.store.as_ref(), "webhooks", &webhook_id, 20, 0.02)
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
                "failed to relay upstream webhook delete",
            )
                .into_response();
        }
    }

    if record_gateway_usage(state.store.as_ref(), "webhooks", &webhook_id, 20, 0.02)
        .await
        .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(delete_webhook("tenant-1", "project-1", &webhook_id).expect("webhook delete"))
        .into_response()
}

async fn realtime_sessions_with_state_handler(
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateRealtimeSessionRequest>,
) -> Response {
    match relay_realtime_session_from_store(
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
                "realtime_sessions",
                &request.model,
                30,
                0.03,
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
                "failed to relay upstream realtime session",
            )
                .into_response();
        }
    }

    if record_gateway_usage(
        state.store.as_ref(),
        "realtime_sessions",
        &request.model,
        30,
        0.03,
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

    Json(create_realtime_session("tenant-1", "project-1", &request.model).expect("realtime"))
        .into_response()
}

async fn evals_with_state_handler(
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateEvalRequest>,
) -> Response {
    match relay_eval_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(state.store.as_ref(), "evals", &request.name, 40, 0.04)
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
                "failed to relay upstream eval",
            )
                .into_response();
        }
    }

    if record_gateway_usage(state.store.as_ref(), "evals", &request.name, 40, 0.04)
        .await
        .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(create_eval("tenant-1", "project-1", &request.name).expect("eval")).into_response()
}

async fn batches_with_state_handler(
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateBatchRequest>,
) -> Response {
    match relay_batch_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(state.store.as_ref(), "batches", &request.endpoint, 60, 0.06)
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
                "failed to relay upstream batch",
            )
                .into_response();
        }
    }

    if record_gateway_usage(state.store.as_ref(), "batches", &request.endpoint, 60, 0.06)
        .await
        .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(
        create_batch(
            "tenant-1",
            "project-1",
            &request.endpoint,
            &request.input_file_id,
        )
        .expect("batch"),
    )
    .into_response()
}

async fn batches_list_with_state_handler(State(state): State<GatewayApiState>) -> Response {
    match relay_list_batches_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(state.store.as_ref(), "batches", "batches", 20, 0.02)
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
                "failed to relay upstream batches list",
            )
                .into_response();
        }
    }

    if record_gateway_usage(state.store.as_ref(), "batches", "batches", 20, 0.02)
        .await
        .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(list_batches("tenant-1", "project-1").expect("batches list")).into_response()
}

async fn batch_retrieve_with_state_handler(
    State(state): State<GatewayApiState>,
    Path(batch_id): Path<String>,
) -> Response {
    match relay_get_batch_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &batch_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(state.store.as_ref(), "batches", &batch_id, 20, 0.02)
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
                "failed to relay upstream batch retrieve",
            )
                .into_response();
        }
    }

    if record_gateway_usage(state.store.as_ref(), "batches", &batch_id, 20, 0.02)
        .await
        .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(get_batch("tenant-1", "project-1", &batch_id).expect("batch retrieve")).into_response()
}

async fn batch_cancel_with_state_handler(
    State(state): State<GatewayApiState>,
    Path(batch_id): Path<String>,
) -> Response {
    match relay_cancel_batch_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &batch_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(state.store.as_ref(), "batches", &batch_id, 20, 0.02)
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
                "failed to relay upstream batch cancel",
            )
                .into_response();
        }
    }

    if record_gateway_usage(state.store.as_ref(), "batches", &batch_id, 20, 0.02)
        .await
        .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(cancel_batch("tenant-1", "project-1", &batch_id).expect("batch cancel")).into_response()
}

async fn vector_stores_with_state_handler(
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateVectorStoreRequest>,
) -> Response {
    match relay_vector_store_from_store(
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
                "vector_stores",
                &request.name,
                35,
                0.035,
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
                "failed to relay upstream vector store",
            )
                .into_response();
        }
    }

    if record_gateway_usage(
        state.store.as_ref(),
        "vector_stores",
        &request.name,
        35,
        0.035,
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

    Json(create_vector_store("tenant-1", "project-1", &request.name).expect("vector store"))
        .into_response()
}

async fn vector_stores_list_with_state_handler(State(state): State<GatewayApiState>) -> Response {
    match relay_list_vector_stores_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(
                state.store.as_ref(),
                "vector_stores",
                "vector_stores",
                20,
                0.02,
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
                "failed to relay upstream vector stores list",
            )
                .into_response();
        }
    }

    if record_gateway_usage(
        state.store.as_ref(),
        "vector_stores",
        "vector_stores",
        20,
        0.02,
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

    Json(list_vector_stores("tenant-1", "project-1").expect("vector stores list")).into_response()
}

async fn vector_store_retrieve_with_state_handler(
    State(state): State<GatewayApiState>,
    Path(vector_store_id): Path<String>,
) -> Response {
    match relay_get_vector_store_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &vector_store_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(
                state.store.as_ref(),
                "vector_stores",
                &vector_store_id,
                20,
                0.02,
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
                "failed to relay upstream vector store retrieve",
            )
                .into_response();
        }
    }

    if record_gateway_usage(
        state.store.as_ref(),
        "vector_stores",
        &vector_store_id,
        20,
        0.02,
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

    Json(
        get_vector_store("tenant-1", "project-1", &vector_store_id).expect("vector store retrieve"),
    )
    .into_response()
}

async fn vector_store_update_with_state_handler(
    State(state): State<GatewayApiState>,
    Path(vector_store_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateVectorStoreRequest>,
) -> Response {
    match relay_update_vector_store_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &vector_store_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(
                state.store.as_ref(),
                "vector_stores",
                &vector_store_id,
                35,
                0.035,
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
                "failed to relay upstream vector store update",
            )
                .into_response();
        }
    }

    if record_gateway_usage(
        state.store.as_ref(),
        "vector_stores",
        &vector_store_id,
        35,
        0.035,
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

    Json(
        update_vector_store(
            "tenant-1",
            "project-1",
            &vector_store_id,
            request.name.as_deref().unwrap_or("vector-store"),
        )
        .expect("vector store update"),
    )
    .into_response()
}

async fn vector_store_delete_with_state_handler(
    State(state): State<GatewayApiState>,
    Path(vector_store_id): Path<String>,
) -> Response {
    match relay_delete_vector_store_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &vector_store_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(
                state.store.as_ref(),
                "vector_stores",
                &vector_store_id,
                20,
                0.02,
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
                "failed to relay upstream vector store delete",
            )
                .into_response();
        }
    }

    if record_gateway_usage(
        state.store.as_ref(),
        "vector_stores",
        &vector_store_id,
        20,
        0.02,
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

    Json(
        delete_vector_store("tenant-1", "project-1", &vector_store_id)
            .expect("vector store delete"),
    )
    .into_response()
}

async fn vector_store_search_with_state_handler(
    State(state): State<GatewayApiState>,
    Path(vector_store_id): Path<String>,
    ExtractJson(request): ExtractJson<SearchVectorStoreRequest>,
) -> Response {
    match relay_search_vector_store_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &vector_store_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(
                state.store.as_ref(),
                "vector_store_search",
                &vector_store_id,
                20,
                0.02,
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
                "failed to relay upstream vector store search",
            )
                .into_response();
        }
    }

    if record_gateway_usage(
        state.store.as_ref(),
        "vector_store_search",
        &vector_store_id,
        20,
        0.02,
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

    Json(
        search_vector_store("tenant-1", "project-1", &vector_store_id, &request.query)
            .expect("vector store search"),
    )
    .into_response()
}

async fn vector_store_files_with_state_handler(
    State(state): State<GatewayApiState>,
    Path(vector_store_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateVectorStoreFileRequest>,
) -> Response {
    match relay_vector_store_file_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &vector_store_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(
                state.store.as_ref(),
                "vector_store_files",
                &vector_store_id,
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
                "failed to relay upstream vector store file",
            )
                .into_response();
        }
    }

    if record_gateway_usage(
        state.store.as_ref(),
        "vector_store_files",
        &vector_store_id,
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

    Json(
        create_vector_store_file("tenant-1", "project-1", &vector_store_id, &request.file_id)
            .expect("vector store file"),
    )
    .into_response()
}

async fn vector_store_files_list_with_state_handler(
    State(state): State<GatewayApiState>,
    Path(vector_store_id): Path<String>,
) -> Response {
    match relay_list_vector_store_files_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &vector_store_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(
                state.store.as_ref(),
                "vector_store_files",
                &vector_store_id,
                15,
                0.015,
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
                "failed to relay upstream vector store files list",
            )
                .into_response();
        }
    }

    if record_gateway_usage(
        state.store.as_ref(),
        "vector_store_files",
        &vector_store_id,
        15,
        0.015,
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

    Json(
        list_vector_store_files("tenant-1", "project-1", &vector_store_id)
            .expect("vector store files list"),
    )
    .into_response()
}

async fn vector_store_file_retrieve_with_state_handler(
    State(state): State<GatewayApiState>,
    Path((vector_store_id, file_id)): Path<(String, String)>,
) -> Response {
    match relay_get_vector_store_file_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &vector_store_id,
        &file_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(
                state.store.as_ref(),
                "vector_store_files",
                &vector_store_id,
                15,
                0.015,
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
                "failed to relay upstream vector store file retrieve",
            )
                .into_response();
        }
    }

    if record_gateway_usage(
        state.store.as_ref(),
        "vector_store_files",
        &vector_store_id,
        15,
        0.015,
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

    Json(
        get_vector_store_file("tenant-1", "project-1", &vector_store_id, &file_id)
            .expect("vector store file retrieve"),
    )
    .into_response()
}

async fn vector_store_file_delete_with_state_handler(
    State(state): State<GatewayApiState>,
    Path((vector_store_id, file_id)): Path<(String, String)>,
) -> Response {
    match relay_delete_vector_store_file_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &vector_store_id,
        &file_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(
                state.store.as_ref(),
                "vector_store_files",
                &vector_store_id,
                15,
                0.015,
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
                "failed to relay upstream vector store file delete",
            )
                .into_response();
        }
    }

    if record_gateway_usage(
        state.store.as_ref(),
        "vector_store_files",
        &vector_store_id,
        15,
        0.015,
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

    Json(
        delete_vector_store_file("tenant-1", "project-1", &vector_store_id, &file_id)
            .expect("vector store file delete"),
    )
    .into_response()
}

async fn vector_store_file_batches_with_state_handler(
    State(state): State<GatewayApiState>,
    Path(vector_store_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateVectorStoreFileBatchRequest>,
) -> Response {
    match relay_vector_store_file_batch_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &vector_store_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(
                state.store.as_ref(),
                "vector_store_file_batches",
                &vector_store_id,
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
                "failed to relay upstream vector store file batch",
            )
                .into_response();
        }
    }

    if record_gateway_usage(
        state.store.as_ref(),
        "vector_store_file_batches",
        &vector_store_id,
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

    Json(
        create_vector_store_file_batch(
            "tenant-1",
            "project-1",
            &vector_store_id,
            &request.file_ids,
        )
        .expect("vector store file batch"),
    )
    .into_response()
}

async fn vector_store_file_batch_retrieve_with_state_handler(
    State(state): State<GatewayApiState>,
    Path((vector_store_id, batch_id)): Path<(String, String)>,
) -> Response {
    match relay_get_vector_store_file_batch_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &vector_store_id,
        &batch_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(
                state.store.as_ref(),
                "vector_store_file_batches",
                &vector_store_id,
                15,
                0.015,
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
                "failed to relay upstream vector store file batch retrieve",
            )
                .into_response();
        }
    }

    if record_gateway_usage(
        state.store.as_ref(),
        "vector_store_file_batches",
        &vector_store_id,
        15,
        0.015,
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

    Json(
        get_vector_store_file_batch("tenant-1", "project-1", &vector_store_id, &batch_id)
            .expect("vector store file batch retrieve"),
    )
    .into_response()
}

async fn vector_store_file_batch_cancel_with_state_handler(
    State(state): State<GatewayApiState>,
    Path((vector_store_id, batch_id)): Path<(String, String)>,
) -> Response {
    match relay_cancel_vector_store_file_batch_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &vector_store_id,
        &batch_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(
                state.store.as_ref(),
                "vector_store_file_batches",
                &vector_store_id,
                15,
                0.015,
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
                "failed to relay upstream vector store file batch cancel",
            )
                .into_response();
        }
    }

    if record_gateway_usage(
        state.store.as_ref(),
        "vector_store_file_batches",
        &vector_store_id,
        15,
        0.015,
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

    Json(
        cancel_vector_store_file_batch("tenant-1", "project-1", &vector_store_id, &batch_id)
            .expect("vector store file batch cancel"),
    )
    .into_response()
}

async fn vector_store_file_batch_files_with_state_handler(
    State(state): State<GatewayApiState>,
    Path((vector_store_id, batch_id)): Path<(String, String)>,
) -> Response {
    match relay_list_vector_store_file_batch_files_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        "tenant-1",
        "project-1",
        &vector_store_id,
        &batch_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage(
                state.store.as_ref(),
                "vector_store_file_batches",
                &vector_store_id,
                15,
                0.015,
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
                "failed to relay upstream vector store file batch files",
            )
                .into_response();
        }
    }

    if record_gateway_usage(
        state.store.as_ref(),
        "vector_store_file_batches",
        &vector_store_id,
        15,
        0.015,
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

    Json(
        list_vector_store_file_batch_files("tenant-1", "project-1", &vector_store_id, &batch_id)
            .expect("vector store file batch files"),
    )
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

fn local_speech_response(request: &CreateSpeechRequest) -> Response {
    let speech = create_speech_response("tenant-1", "project-1", request).expect("speech");
    if request.stream_format.as_deref() == Some("sse") {
        let delta = serde_json::json!({
            "type":"response.output_audio.delta",
            "delta": speech.audio_base64,
            "format": speech.format,
        })
        .to_string();
        let done = serde_json::json!({
            "type":"response.completed"
        })
        .to_string();
        let body = format!("{}{}", SseFrame::data(&delta), SseFrame::data(&done));
        return ([(header::CONTENT_TYPE, "text/event-stream")], body).into_response();
    }

    let bytes = STANDARD
        .decode(speech.audio_base64.as_bytes())
        .unwrap_or_default();

    Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(header::CONTENT_TYPE, speech_content_type(&speech.format))
        .body(Body::from(bytes))
        .expect("valid speech response")
}

fn speech_content_type(format: &str) -> &'static str {
    match format {
        "mp3" => "audio/mpeg",
        "opus" => "audio/opus",
        "aac" => "audio/aac",
        "flac" => "audio/flac",
        "pcm" => "audio/pcm",
        _ => "audio/wav",
    }
}

async fn parse_file_request(mut multipart: Multipart) -> Result<CreateFileRequest, Response> {
    let mut purpose = None;
    let mut filename = None;
    let mut bytes = None;
    let mut content_type = None;

    while let Some(field) = multipart.next_field().await.map_err(bad_multipart)? {
        match field.name() {
            Some("purpose") => {
                purpose = Some(field.text().await.map_err(bad_multipart)?);
            }
            Some("file") => {
                filename = field.file_name().map(ToOwned::to_owned);
                content_type = field.content_type().map(ToOwned::to_owned);
                bytes = Some(field.bytes().await.map_err(bad_multipart)?.to_vec());
            }
            _ => {}
        }
    }

    let mut request = CreateFileRequest::new(
        purpose.ok_or_else(missing_multipart_field)?,
        filename.ok_or_else(missing_multipart_field)?,
        bytes.ok_or_else(missing_multipart_field)?,
    );
    if let Some(content_type) = content_type {
        request = request.with_content_type(content_type);
    }
    Ok(request)
}

async fn parse_upload_part_request(
    upload_id: String,
    mut multipart: Multipart,
) -> Result<AddUploadPartRequest, Response> {
    let mut data = None;
    let mut filename = None;
    let mut content_type = None;

    while let Some(field) = multipart.next_field().await.map_err(bad_multipart)? {
        if field.name() == Some("data") {
            filename = field.file_name().map(ToOwned::to_owned);
            content_type = field.content_type().map(ToOwned::to_owned);
            data = Some(field.bytes().await.map_err(bad_multipart)?.to_vec());
        }
    }

    let mut request =
        AddUploadPartRequest::new(upload_id, data.ok_or_else(missing_multipart_field)?);
    if let Some(filename) = filename {
        request = request.with_filename(filename);
    }
    if let Some(content_type) = content_type {
        request = request.with_content_type(content_type);
    }
    Ok(request)
}

fn bad_multipart(error: axum::extract::multipart::MultipartError) -> Response {
    (
        axum::http::StatusCode::BAD_REQUEST,
        format!("invalid multipart payload: {error}"),
    )
        .into_response()
}

fn missing_multipart_field() -> Response {
    (
        axum::http::StatusCode::BAD_REQUEST,
        "missing multipart field",
    )
        .into_response()
}
