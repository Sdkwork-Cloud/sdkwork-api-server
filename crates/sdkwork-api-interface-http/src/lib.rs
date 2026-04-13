mod compat_anthropic;
mod compat_gemini;
mod compat_streaming;

use std::io;
use std::sync::{Arc, OnceLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::Context as _;
use axum::{
    body::Body,
    extract::Extension,
    extract::FromRequestParts,
    extract::Json as ExtractJson,
    extract::Multipart,
    extract::Path,
    extract::State,
    http::header,
    http::request::Parts,
    http::HeaderMap,
    http::Request,
    http::StatusCode,
    middleware::Next,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use bytes::Bytes;
use compat_anthropic::{
    anthropic_bad_gateway_response, anthropic_count_tokens_request,
    anthropic_invalid_request_response, anthropic_request_to_chat_completion,
    anthropic_stream_from_openai, openai_chat_response_to_anthropic,
    openai_count_tokens_to_anthropic,
};
use compat_gemini::{
    gemini_bad_gateway_response, gemini_count_tokens_request, gemini_invalid_request_response,
    gemini_request_to_chat_completion, gemini_stream_from_openai, openai_chat_response_to_gemini,
    openai_count_tokens_to_gemini,
};
use futures_util::{stream, StreamExt};
use sdkwork_api_app_billing::{
    capture_account_hold, check_quota, create_account_hold, create_billing_event,
    persist_billing_event, persist_ledger_entry, plan_account_hold, reconcile_account_hold,
    release_account_hold, resolve_payable_account_for_gateway_subject, BillingAccountingMode,
    CaptureAccountHoldInput, CreateAccountHoldInput, CreateBillingEventInput, QuotaCheckResult,
    ReleaseAccountHoldInput,
};
use sdkwork_api_app_credential::CredentialSecretManager;
use sdkwork_api_app_gateway::cancel_batch;
use sdkwork_api_app_gateway::cancel_fine_tuning_job;
use sdkwork_api_app_gateway::cancel_response;
use sdkwork_api_app_gateway::cancel_thread_run;
use sdkwork_api_app_gateway::cancel_upload;
use sdkwork_api_app_gateway::cancel_vector_store_file_batch;
use sdkwork_api_app_gateway::compact_response;
use sdkwork_api_app_gateway::complete_upload;
use sdkwork_api_app_gateway::count_response_input_tokens;
use sdkwork_api_app_gateway::create_assistant;
use sdkwork_api_app_gateway::create_audio_voice_consent;
use sdkwork_api_app_gateway::create_batch;
use sdkwork_api_app_gateway::create_chat_completion;
use sdkwork_api_app_gateway::create_completion;
use sdkwork_api_app_gateway::create_conversation;
use sdkwork_api_app_gateway::create_conversation_items;
use sdkwork_api_app_gateway::create_eval;
use sdkwork_api_app_gateway::create_eval_run;
use sdkwork_api_app_gateway::create_file;
use sdkwork_api_app_gateway::create_fine_tuning_job;
use sdkwork_api_app_gateway::create_image_edit;
use sdkwork_api_app_gateway::create_image_generation;
use sdkwork_api_app_gateway::create_image_variation;
use sdkwork_api_app_gateway::create_moderation;
use sdkwork_api_app_gateway::create_music;
use sdkwork_api_app_gateway::create_music_lyrics;
use sdkwork_api_app_gateway::create_realtime_session;
use sdkwork_api_app_gateway::create_speech_response;
use sdkwork_api_app_gateway::create_thread;
use sdkwork_api_app_gateway::create_thread_and_run;
use sdkwork_api_app_gateway::create_thread_message;
use sdkwork_api_app_gateway::create_thread_run;
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
use sdkwork_api_app_gateway::delete_chat_completion;
use sdkwork_api_app_gateway::delete_conversation;
use sdkwork_api_app_gateway::delete_conversation_item;
use sdkwork_api_app_gateway::delete_eval;
use sdkwork_api_app_gateway::delete_file;
use sdkwork_api_app_gateway::delete_model;
use sdkwork_api_app_gateway::delete_music;
use sdkwork_api_app_gateway::delete_response;
use sdkwork_api_app_gateway::delete_thread;
use sdkwork_api_app_gateway::delete_thread_message;
use sdkwork_api_app_gateway::delete_vector_store;
use sdkwork_api_app_gateway::delete_vector_store_file;
use sdkwork_api_app_gateway::delete_video;
use sdkwork_api_app_gateway::delete_webhook;
use sdkwork_api_app_gateway::extend_video;
use sdkwork_api_app_gateway::file_content;
use sdkwork_api_app_gateway::get_assistant;
use sdkwork_api_app_gateway::get_batch;
use sdkwork_api_app_gateway::get_chat_completion;
use sdkwork_api_app_gateway::get_conversation;
use sdkwork_api_app_gateway::get_conversation_item;
use sdkwork_api_app_gateway::get_eval;
use sdkwork_api_app_gateway::get_eval_run;
use sdkwork_api_app_gateway::get_file;
use sdkwork_api_app_gateway::get_fine_tuning_job;
use sdkwork_api_app_gateway::get_model;
use sdkwork_api_app_gateway::get_model_from_store;
use sdkwork_api_app_gateway::get_music;
use sdkwork_api_app_gateway::get_response;
use sdkwork_api_app_gateway::get_thread;
use sdkwork_api_app_gateway::get_thread_message;
use sdkwork_api_app_gateway::get_thread_run;
use sdkwork_api_app_gateway::get_thread_run_step;
use sdkwork_api_app_gateway::get_vector_store;
use sdkwork_api_app_gateway::get_vector_store_file;
use sdkwork_api_app_gateway::get_vector_store_file_batch;
use sdkwork_api_app_gateway::get_video;
use sdkwork_api_app_gateway::get_video_character;
use sdkwork_api_app_gateway::get_webhook;
use sdkwork_api_app_gateway::list_assistants;
use sdkwork_api_app_gateway::list_audio_voices;
use sdkwork_api_app_gateway::list_batches;
use sdkwork_api_app_gateway::list_chat_completion_messages;
use sdkwork_api_app_gateway::list_chat_completions;
use sdkwork_api_app_gateway::list_conversation_items;
use sdkwork_api_app_gateway::list_conversations;
use sdkwork_api_app_gateway::list_eval_runs;
use sdkwork_api_app_gateway::list_evals;
use sdkwork_api_app_gateway::list_files;
use sdkwork_api_app_gateway::list_fine_tuning_job_checkpoints;
use sdkwork_api_app_gateway::list_fine_tuning_job_events;
use sdkwork_api_app_gateway::list_fine_tuning_jobs;
use sdkwork_api_app_gateway::list_models;
use sdkwork_api_app_gateway::list_music;
use sdkwork_api_app_gateway::list_response_input_items;
use sdkwork_api_app_gateway::list_thread_messages;
use sdkwork_api_app_gateway::list_thread_run_steps;
use sdkwork_api_app_gateway::list_thread_runs;
use sdkwork_api_app_gateway::list_vector_store_file_batch_files;
use sdkwork_api_app_gateway::list_vector_store_files;
use sdkwork_api_app_gateway::list_vector_stores;
use sdkwork_api_app_gateway::list_video_characters;
use sdkwork_api_app_gateway::list_videos;
use sdkwork_api_app_gateway::list_webhooks;
use sdkwork_api_app_gateway::music_content;
use sdkwork_api_app_gateway::remix_video;
use sdkwork_api_app_gateway::search_vector_store;
use sdkwork_api_app_gateway::submit_thread_run_tool_outputs;
use sdkwork_api_app_gateway::update_assistant;
use sdkwork_api_app_gateway::update_chat_completion;
use sdkwork_api_app_gateway::update_conversation;
use sdkwork_api_app_gateway::update_eval;
use sdkwork_api_app_gateway::update_thread;
use sdkwork_api_app_gateway::update_thread_message;
use sdkwork_api_app_gateway::update_thread_run;
use sdkwork_api_app_gateway::update_vector_store;
use sdkwork_api_app_gateway::update_video_character;
use sdkwork_api_app_gateway::update_webhook;
use sdkwork_api_app_gateway::video_content;
use sdkwork_api_app_gateway::{
    create_embedding, create_response, delete_model_from_store,
    execute_json_provider_request_with_runtime_and_options,
    execute_stream_provider_request_with_runtime_and_options, list_models_from_store,
    planned_execution_usage_context_for_route, relay_assistant_from_store,
    relay_audio_voice_consent_from_store, relay_audio_voices_from_store, relay_batch_from_store,
    relay_cancel_batch_from_store, relay_cancel_fine_tuning_job_from_store,
    relay_cancel_response_from_store, relay_cancel_thread_run_from_store,
    relay_cancel_upload_from_store, relay_cancel_vector_store_file_batch_from_store,
    relay_chat_completion_from_store, relay_chat_completion_from_store_with_options,
    relay_chat_completion_stream_from_store, relay_chat_completion_stream_from_store_with_options,
    relay_compact_response_from_store, relay_complete_upload_from_store,
    relay_completion_from_store, relay_conversation_from_store,
    relay_conversation_items_from_store, relay_count_response_input_tokens_from_store,
    relay_delete_assistant_from_store, relay_delete_chat_completion_from_store,
    relay_delete_conversation_from_store, relay_delete_conversation_item_from_store,
    relay_delete_eval_from_store, relay_delete_file_from_store, relay_delete_music_from_store,
    relay_delete_response_from_store, relay_delete_thread_from_store,
    relay_delete_thread_message_from_store, relay_delete_vector_store_file_from_store,
    relay_delete_vector_store_from_store, relay_delete_video_from_store,
    relay_delete_webhook_from_store, relay_embedding_from_store, relay_eval_from_store,
    relay_eval_run_from_store, relay_extend_video_from_store, relay_file_content_from_store,
    relay_file_from_store, relay_fine_tuning_job_from_store, relay_get_assistant_from_store,
    relay_get_batch_from_store, relay_get_chat_completion_from_store,
    relay_get_conversation_from_store, relay_get_conversation_item_from_store,
    relay_get_eval_from_store, relay_get_eval_run_from_store, relay_get_file_from_store,
    relay_get_fine_tuning_job_from_store, relay_get_music_from_store,
    relay_get_response_from_store, relay_get_thread_from_store,
    relay_get_thread_message_from_store, relay_get_thread_run_from_store,
    relay_get_thread_run_step_from_store, relay_get_vector_store_file_batch_from_store,
    relay_get_vector_store_file_from_store, relay_get_vector_store_from_store,
    relay_get_video_character_from_store, relay_get_video_from_store, relay_get_webhook_from_store,
    relay_image_edit_from_store, relay_image_generation_from_store,
    relay_image_variation_from_store, relay_list_assistants_from_store,
    relay_list_batches_from_store, relay_list_chat_completion_messages_from_store,
    relay_list_chat_completions_from_store, relay_list_conversation_items_from_store,
    relay_list_conversations_from_store, relay_list_eval_runs_from_store,
    relay_list_evals_from_store, relay_list_files_from_store,
    relay_list_fine_tuning_job_checkpoints_from_store,
    relay_list_fine_tuning_job_events_from_store, relay_list_fine_tuning_jobs_from_store,
    relay_list_music_from_store, relay_list_response_input_items_from_store,
    relay_list_thread_messages_from_store, relay_list_thread_run_steps_from_store,
    relay_list_thread_runs_from_store, relay_list_vector_store_file_batch_files_from_store,
    relay_list_vector_store_files_from_store, relay_list_vector_stores_from_store,
    relay_list_video_characters_from_store, relay_list_videos_from_store,
    relay_list_webhooks_from_store, relay_moderation_from_store, relay_music_content_from_store,
    relay_music_from_store, relay_music_lyrics_from_store, relay_realtime_session_from_store,
    relay_remix_video_from_store, relay_response_from_store, relay_response_stream_from_store,
    relay_search_vector_store_from_store, relay_speech_from_store,
    relay_submit_thread_run_tool_outputs_from_store, relay_thread_and_run_from_store,
    relay_thread_from_store, relay_thread_messages_from_store, relay_thread_run_from_store,
    relay_transcription_from_store, relay_translation_from_store,
    relay_update_assistant_from_store, relay_update_chat_completion_from_store,
    relay_update_conversation_from_store, relay_update_eval_from_store,
    relay_update_thread_from_store, relay_update_thread_message_from_store,
    relay_update_thread_run_from_store, relay_update_vector_store_from_store,
    relay_update_video_character_from_store, relay_update_webhook_from_store,
    relay_upload_from_store, relay_upload_part_from_store,
    relay_vector_store_file_batch_from_store, relay_vector_store_file_from_store,
    relay_vector_store_from_store, relay_video_content_from_store, relay_video_from_store,
    relay_webhook_from_store, with_request_api_key_group_id, with_request_routing_region,
};
use sdkwork_api_app_identity::{
    resolve_gateway_auth_subject_from_api_key, resolve_gateway_request_context,
    resolve_gateway_request_context_from_auth_subject,
    GatewayRequestContext as IdentityGatewayRequestContext,
};
use sdkwork_api_app_rate_limit::{
    check_rate_limit, with_gateway_traffic_context, GatewayTrafficAdmissionError,
    GatewayTrafficAdmissionPermit, GatewayTrafficController, GatewayTrafficRequestContext,
    InMemoryGatewayTrafficController,
};
use sdkwork_api_app_usage::persist_usage_record_with_tokens_and_facts;
use sdkwork_api_contract_openai::assistants::{CreateAssistantRequest, UpdateAssistantRequest};
use sdkwork_api_contract_openai::audio::{
    CreateSpeechRequest, CreateTranscriptionRequest, CreateTranslationRequest,
    CreateVoiceConsentRequest,
};
use sdkwork_api_contract_openai::batches::CreateBatchRequest;
use sdkwork_api_contract_openai::chat_completions::{
    ChatMessageInput, CreateChatCompletionRequest, UpdateChatCompletionRequest,
};
use sdkwork_api_contract_openai::completions::CreateCompletionRequest;
use sdkwork_api_contract_openai::containers::{CreateContainerFileRequest, CreateContainerRequest};
use sdkwork_api_contract_openai::conversations::{
    CreateConversationItemsRequest, CreateConversationRequest, UpdateConversationRequest,
};
use sdkwork_api_contract_openai::embeddings::CreateEmbeddingRequest;
use sdkwork_api_contract_openai::errors::OpenAiErrorResponse;
use sdkwork_api_contract_openai::evals::{
    CreateEvalRequest, CreateEvalRunRequest, UpdateEvalRequest,
};
use sdkwork_api_contract_openai::files::CreateFileRequest;
use sdkwork_api_contract_openai::fine_tuning::{
    CreateFineTuningCheckpointPermissionsRequest, CreateFineTuningJobRequest,
};
use sdkwork_api_contract_openai::images::{
    CreateImageEditRequest, CreateImageRequest, CreateImageVariationRequest, ImageUpload,
};
use sdkwork_api_contract_openai::moderations::CreateModerationRequest;
use sdkwork_api_contract_openai::music::{CreateMusicLyricsRequest, CreateMusicRequest};
use sdkwork_api_contract_openai::realtime::CreateRealtimeSessionRequest;
use sdkwork_api_contract_openai::responses::{
    CompactResponseRequest, CountResponseInputTokensRequest, CreateResponseRequest,
};
use sdkwork_api_contract_openai::runs::{
    CreateRunRequest, CreateThreadAndRunRequest, SubmitToolOutputsRunRequest, UpdateRunRequest,
};
use sdkwork_api_contract_openai::streaming::SseFrame;
use sdkwork_api_contract_openai::threads::{
    CreateThreadMessageRequest, CreateThreadRequest, UpdateThreadMessageRequest,
    UpdateThreadRequest,
};
use sdkwork_api_contract_openai::uploads::{
    AddUploadPartRequest, CompleteUploadRequest, CreateUploadRequest,
};
use sdkwork_api_contract_openai::vector_stores::{
    CreateVectorStoreFileBatchRequest, CreateVectorStoreFileRequest, CreateVectorStoreRequest,
    SearchVectorStoreRequest, UpdateVectorStoreRequest,
};
use sdkwork_api_contract_openai::videos::{
    CreateVideoCharacterRequest, CreateVideoRequest, EditVideoRequest, ExtendVideoRequest,
    RemixVideoRequest, UpdateVideoCharacterRequest,
};
use sdkwork_api_contract_openai::webhooks::{CreateWebhookRequest, UpdateWebhookRequest};
use sdkwork_api_domain_catalog::ModelPriceRecord;
use sdkwork_api_domain_identity::GatewayAuthSubject;
use sdkwork_api_domain_rate_limit::RateLimitCheckResult;
use sdkwork_api_domain_usage::{RequestMeterFactRecord, RequestStatus, UsageCaptureStatus};
use sdkwork_api_observability::{
    annotate_current_http_metrics, observe_http_metrics, observe_http_tracing,
    record_current_commercial_event, CommercialEventDimensions, CommercialEventKind,
    HttpMetricsRegistry, RequestId,
};
use sdkwork_api_openapi::{
    build_openapi_document, extract_routes_from_function, render_docs_html, HttpMethod,
    OpenApiServiceSpec, RouteEntry,
};
use sdkwork_api_policy_billing::{
    builtin_billing_policy_registry, BillingPolicyExecutionInput, BillingPolicyExecutionResult,
    GROUP_DEFAULT_BILLING_POLICY_ID,
};
use sdkwork_api_provider_core::{ProviderRequest, ProviderRequestOptions, ProviderStreamOutput};
use sdkwork_api_storage_core::{AdminStore, Reloadable};
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use serde_json::Value;
use sqlx::SqlitePool;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tower_http::cors::{Any, CorsLayer};

const DEFAULT_STATELESS_TENANT_ID: &str = "sdkwork-stateless";
const DEFAULT_STATELESS_PROJECT_ID: &str = "sdkwork-stateless-default";
const GATEWAY_OPENAPI_SPEC: OpenApiServiceSpec<'static> = OpenApiServiceSpec {
    title: "SDKWORK Gateway API",
    version: env!("CARGO_PKG_VERSION"),
    description: "OpenAPI 3.1 inventory generated from the current gateway router implementation.",
    openapi_path: "/openapi.json",
    docs_path: "/docs",
};

fn gateway_route_inventory() -> &'static [RouteEntry] {
    static ROUTES: OnceLock<Vec<RouteEntry>> = OnceLock::new();
    ROUTES
        .get_or_init(|| {
            extract_routes_from_function(include_str!("lib.rs"), "gateway_router_with_state")
                .expect("gateway route inventory")
        })
        .as_slice()
}

fn gateway_openapi_document() -> &'static Value {
    static DOCUMENT: OnceLock<Value> = OnceLock::new();
    DOCUMENT.get_or_init(|| {
        build_openapi_document(
            &GATEWAY_OPENAPI_SPEC,
            gateway_route_inventory(),
            gateway_tag_for_path,
            gateway_route_requires_bearer_auth,
            gateway_operation_summary,
        )
    })
}

fn gateway_docs_html() -> &'static str {
    static HTML: OnceLock<String> = OnceLock::new();
    HTML.get_or_init(|| render_docs_html(&GATEWAY_OPENAPI_SPEC))
        .as_str()
}

async fn gateway_openapi_handler() -> Json<Value> {
    Json(gateway_openapi_document().clone())
}

async fn gateway_docs_handler() -> Html<String> {
    Html(gateway_docs_html().to_owned())
}

fn gateway_tag_for_path(path: &str) -> String {
    match path {
        "/metrics" | "/health" => "system".to_owned(),
        "/docs" | "/openapi.json" => "docs".to_owned(),
        _ if path.starts_with("/v1/") || path.starts_with("/v1beta/") => path
            .trim_start_matches("/v1/")
            .trim_start_matches("/v1beta/")
            .split('/')
            .find(|segment| !segment.is_empty() && !segment.starts_with('{'))
            .unwrap_or("gateway")
            .to_owned(),
        _ => "gateway".to_owned(),
    }
}

fn gateway_route_requires_bearer_auth(path: &str, _method: HttpMethod) -> bool {
    path.starts_with("/v1/") || path.starts_with("/v1beta/")
}

fn gateway_operation_summary(path: &str, method: HttpMethod) -> String {
    match path {
        "/metrics" => "Prometheus metrics".to_owned(),
        "/health" => "Health check".to_owned(),
        "/openapi.json" => "OpenAPI document".to_owned(),
        "/docs" => "Interactive API inventory".to_owned(),
        _ => format!(
            "{} {}",
            method.display_name(),
            humanize_route_path(
                path,
                if path.starts_with("/v1beta/") {
                    Some("v1beta")
                } else if path.starts_with("/v1/") {
                    Some("v1")
                } else {
                    None
                },
            )
        ),
    }
}

fn browser_cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
}

fn humanize_route_path(path: &str, ignored_prefix: Option<&str>) -> String {
    let parts = path
        .trim_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .filter(|segment| Some(*segment) != ignored_prefix)
        .map(|segment| {
            if segment.starts_with('{') && segment.ends_with('}') {
                format!(
                    "by {}",
                    segment
                        .trim_matches(|ch| ch == '{' || ch == '}')
                        .replace(['_', '-'], " ")
                )
            } else {
                segment.replace(['_', '-'], " ")
            }
        })
        .collect::<Vec<_>>();

    if parts.is_empty() {
        "root".to_owned()
    } else {
        parts.join(" / ")
    }
}

pub struct GatewayApiState {
    live_store: Reloadable<Arc<dyn AdminStore>>,
    live_secret_manager: Reloadable<CredentialSecretManager>,
    store: Arc<dyn AdminStore>,
    secret_manager: CredentialSecretManager,
    traffic_controller: Arc<dyn GatewayTrafficController>,
}

impl Clone for GatewayApiState {
    fn clone(&self) -> Self {
        Self {
            live_store: self.live_store.clone(),
            live_secret_manager: self.live_secret_manager.clone(),
            store: self.live_store.snapshot(),
            secret_manager: self.live_secret_manager.snapshot(),
            traffic_controller: Arc::clone(&self.traffic_controller),
        }
    }
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
        Self::with_store_and_secret_manager(Arc::new(SqliteAdminStore::new(pool)), secret_manager)
    }

    pub fn with_store_and_secret_manager(
        store: Arc<dyn AdminStore>,
        secret_manager: CredentialSecretManager,
    ) -> Self {
        Self::with_store_secret_manager_and_traffic_controller(
            store,
            secret_manager,
            Arc::new(InMemoryGatewayTrafficController::new()),
        )
    }

    pub fn with_store_secret_manager_and_traffic_controller(
        store: Arc<dyn AdminStore>,
        secret_manager: CredentialSecretManager,
        traffic_controller: Arc<dyn GatewayTrafficController>,
    ) -> Self {
        Self::with_live_store_and_secret_manager_handle(
            Reloadable::new(store),
            Reloadable::new(secret_manager),
            traffic_controller,
        )
    }

    pub fn with_live_store_and_secret_manager(
        live_store: Reloadable<Arc<dyn AdminStore>>,
        secret_manager: CredentialSecretManager,
    ) -> Self {
        Self::with_live_store_secret_manager_and_traffic_controller(
            live_store,
            secret_manager,
            Arc::new(InMemoryGatewayTrafficController::new()),
        )
    }

    pub fn with_live_store_secret_manager_and_traffic_controller(
        live_store: Reloadable<Arc<dyn AdminStore>>,
        secret_manager: CredentialSecretManager,
        traffic_controller: Arc<dyn GatewayTrafficController>,
    ) -> Self {
        Self::with_live_store_and_secret_manager_handle(
            live_store,
            Reloadable::new(secret_manager),
            traffic_controller,
        )
    }

    pub fn with_live_store_and_secret_manager_handle(
        live_store: Reloadable<Arc<dyn AdminStore>>,
        live_secret_manager: Reloadable<CredentialSecretManager>,
        traffic_controller: Arc<dyn GatewayTrafficController>,
    ) -> Self {
        Self {
            store: live_store.snapshot(),
            secret_manager: live_secret_manager.snapshot(),
            live_store,
            live_secret_manager,
            traffic_controller,
        }
    }
}

tokio::task_local! {
    static CURRENT_GATEWAY_REQUEST_CONTEXT: IdentityGatewayRequestContext;
}

tokio::task_local! {
    static CURRENT_GATEWAY_REQUEST_STARTED_AT: Instant;
}

const CHAT_COMPLETION_HOLD_UNITS: u64 = 100;
const CHAT_COMPLETION_RETAIL_CHARGE: f64 = 0.10;
const DEFAULT_CHAT_COMPLETION_ESTIMATED_INPUT_TOKENS: u64 = 256;
const DEFAULT_CHAT_COMPLETION_ESTIMATED_OUTPUT_TOKENS: u64 = 1024;
const MODERATIONS_HOLD_UNITS: u64 = 1;
const MODERATIONS_RETAIL_CHARGE: f64 = 0.001;
const DEFAULT_MODERATIONS_ESTIMATED_INPUT_TOKENS: u64 = 256;
const RESPONSES_HOLD_UNITS: u64 = 120;
const RESPONSES_RETAIL_CHARGE: f64 = 0.12;
const DEFAULT_RESPONSES_ESTIMATED_INPUT_TOKENS: u64 = 256;
const DEFAULT_RESPONSES_ESTIMATED_OUTPUT_TOKENS: u64 = 1024;
const COMPLETIONS_HOLD_UNITS: u64 = 80;
const COMPLETIONS_RETAIL_CHARGE: f64 = 0.08;
const DEFAULT_COMPLETIONS_ESTIMATED_INPUT_TOKENS: u64 = 256;
const DEFAULT_COMPLETIONS_ESTIMATED_OUTPUT_TOKENS: u64 = 1024;
const EMBEDDINGS_HOLD_UNITS: u64 = 10;
const EMBEDDINGS_RETAIL_CHARGE: f64 = 0.01;
const DEFAULT_EMBEDDINGS_ESTIMATED_INPUT_TOKENS: u64 = 256;
const DEFAULT_VIDEO_ESTIMATED_SECONDS: f64 = 60.0;
const DEFAULT_MUSIC_ESTIMATED_SECONDS: f64 = 125.0;
const GATEWAY_HOLD_TTL_MS: u64 = 5 * 60 * 1000;
const GATEWAY_ACCOUNT_KERNEL_REQUEST_ID_BASE: u64 = 1_000_000_000_000_000;
const GATEWAY_ACCOUNT_KERNEL_REQUEST_ID_MODULUS: u64 = 8_000_000_000_000_000;

#[derive(Clone, Debug)]
struct CanonicalGatewayAuthSubject(GatewayAuthSubject);

#[derive(Clone, Debug)]
struct CanonicalHeldCharge {
    hold_id: u64,
    request_id: u64,
    estimated_quantity: f64,
    estimated_usage: TokenUsageMetrics,
    provider_cost_amount: f64,
    model_price: ModelPriceRecord,
}

#[derive(Clone, Debug)]
struct CanonicalChatSettlementAmounts {
    usage_units: u64,
    customer_charge: f64,
}

#[derive(Clone)]
struct UsagePricedStreamSettlementContext {
    store: Arc<dyn AdminStore>,
    tenant_id: String,
    project_id: String,
    model: String,
    canonical_hold: Option<CanonicalHeldCharge>,
}

#[derive(Clone, Debug, Default)]
struct ResponsesCompletedEvent {
    token_usage: Option<TokenUsageMetrics>,
    reference_id: Option<String>,
}

#[derive(Debug, Default)]
struct ResponsesSseSettlementTracker {
    buffer: Vec<u8>,
    completed_event: Option<ResponsesCompletedEvent>,
}

#[derive(Debug, Default)]
struct ChatCompletionsSseSettlementTracker {
    buffer: Vec<u8>,
    saw_done: bool,
    token_usage: Option<TokenUsageMetrics>,
    reference_id: Option<String>,
}

enum CanonicalChatAdmission {
    NotApplicable,
    InsufficientBalance {
        account_id: u64,
        shortfall_quantity: f64,
    },
    Held(CanonicalHeldCharge),
}

#[derive(Clone, Debug)]
struct AuthenticatedGatewayRequest {
    context: IdentityGatewayRequestContext,
    canonical_subject: Option<GatewayAuthSubject>,
    _request_traffic_permit: GatewayTrafficAdmissionPermit,
}

impl AuthenticatedGatewayRequest {
    fn tenant_id(&self) -> &str {
        self.context.tenant_id()
    }

    fn project_id(&self) -> &str {
        self.context.project_id()
    }

    fn canonical_subject(&self) -> Option<&GatewayAuthSubject> {
        self.canonical_subject.as_ref()
    }
}

impl FromRequestParts<GatewayApiState> for AuthenticatedGatewayRequest {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &GatewayApiState,
    ) -> Result<Self, Self::Rejection> {
        let mut canonical_subject = parts
            .extensions
            .get::<CanonicalGatewayAuthSubject>()
            .map(|subject| subject.0.clone());
        let context = if let Some(context) = parts
            .extensions
            .get::<IdentityGatewayRequestContext>()
            .cloned()
        {
            context
        } else {
            let Some(header_value) = parts.headers.get(header::AUTHORIZATION) else {
                return Err(StatusCode::UNAUTHORIZED.into_response());
            };
            let Ok(header_value) = header_value.to_str() else {
                return Err(StatusCode::UNAUTHORIZED.into_response());
            };
            let Some(token) = header_value
                .strip_prefix("Bearer ")
                .or_else(|| header_value.strip_prefix("bearer "))
            else {
                return Err(StatusCode::UNAUTHORIZED.into_response());
            };

            let resolved = resolve_gateway_authentication(state.store.as_ref(), token)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;
            canonical_subject = canonical_subject.or(resolved.canonical_subject);
            let Some(context) = resolved.context else {
                return Err(StatusCode::UNAUTHORIZED.into_response());
            };
            context
        };

        if let Err(response) =
            enforce_gateway_request_rate_limit(state.store.as_ref(), &context, parts.uri.path())
                .await
        {
            return Err(response);
        }

        let traffic_context = GatewayTrafficRequestContext::new(
            context.tenant_id(),
            context.project_id(),
            context.api_key_hash(),
            parts.uri.path(),
        )
        .with_api_key_group_id_option(context.api_key_group_id.clone());
        let request_traffic_permit = match state
            .traffic_controller
            .acquire_request_admission(&traffic_context)
            .await
        {
            Ok(permit) => permit,
            Err(error) => return Err(commercial_admission_error_response(&error)),
        };

        Ok(Self {
            context,
            canonical_subject,
            _request_traffic_permit: request_traffic_permit,
        })
    }
}

#[derive(Clone, Debug)]
struct CompatAuthenticatedGatewayRequest {
    context: IdentityGatewayRequestContext,
    _request_traffic_permit: GatewayTrafficAdmissionPermit,
}

impl CompatAuthenticatedGatewayRequest {
    fn tenant_id(&self) -> &str {
        self.context.tenant_id()
    }

    fn project_id(&self) -> &str {
        self.context.project_id()
    }
}

impl FromRequestParts<GatewayApiState> for CompatAuthenticatedGatewayRequest {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &GatewayApiState,
    ) -> Result<Self, Self::Rejection> {
        let context = if let Some(context) = parts
            .extensions
            .get::<IdentityGatewayRequestContext>()
            .cloned()
        {
            context
        } else {
            let Some(token) = extract_compat_gateway_token(parts) else {
                return Err(StatusCode::UNAUTHORIZED.into_response());
            };

            let resolved = resolve_gateway_authentication(state.store.as_ref(), &token)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;
            let Some(context) = resolved.context else {
                return Err(StatusCode::UNAUTHORIZED.into_response());
            };
            context
        };

        if let Err(response) =
            enforce_gateway_request_rate_limit(state.store.as_ref(), &context, parts.uri.path())
                .await
        {
            return Err(response);
        }

        let traffic_context = GatewayTrafficRequestContext::new(
            context.tenant_id(),
            context.project_id(),
            context.api_key_hash(),
            parts.uri.path(),
        )
        .with_api_key_group_id_option(context.api_key_group_id.clone());
        let request_traffic_permit = match state
            .traffic_controller
            .acquire_request_admission(&traffic_context)
            .await
        {
            Ok(permit) => permit,
            Err(error) => return Err(commercial_admission_error_response(&error)),
        };

        Ok(Self {
            context,
            _request_traffic_permit: request_traffic_permit,
        })
    }
}

fn extract_compat_gateway_token(parts: &Parts) -> Option<String> {
    extract_bearer_token(&parts.headers)
        .or_else(|| header_value(parts.headers.get("x-api-key")))
        .or_else(|| header_value(parts.headers.get("x-goog-api-key")))
        .or_else(|| query_parameter(parts.uri.query(), "key"))
}

async fn enforce_gateway_request_rate_limit(
    store: &dyn AdminStore,
    context: &IdentityGatewayRequestContext,
    route_key: &str,
) -> Result<(), Response> {
    match check_rate_limit(
        store,
        context.project_id(),
        Some(context.api_key_hash()),
        route_key,
        None,
        1,
    )
    .await
    {
        Ok(result) if result.allowed => Ok(()),
        Ok(result) => Err(rate_limit_exceeded_response(
            context.project_id(),
            route_key,
            &result,
        )),
        Err(_) => Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to evaluate rate limit",
        )
            .into_response()),
    }
}

fn rate_limit_exceeded_response(
    project_id: &str,
    route_key: &str,
    evaluation: &RateLimitCheckResult,
) -> Response {
    annotate_current_http_metrics(|dimensions| {
        dimensions.tenant = Some(project_id.to_owned());
    });
    record_current_commercial_event(
        CommercialEventKind::Throttling,
        CommercialEventDimensions::default()
            .with_route(route_key.to_owned())
            .with_tenant(project_id.to_owned())
            .with_result("rate_limit_exceeded"),
    );
    let mut error = OpenAiErrorResponse::new(
        rate_limit_exceeded_message(project_id, route_key, evaluation),
        "rate_limit_exceeded",
    );
    error.error.code = Some("rate_limit_exceeded".to_owned());
    (axum::http::StatusCode::TOO_MANY_REQUESTS, Json(error)).into_response()
}

fn commercial_admission_error_response(error: &GatewayTrafficAdmissionError) -> Response {
    let mut event_dimensions =
        CommercialEventDimensions::default().with_result(error.code().to_owned());
    match error {
        GatewayTrafficAdmissionError::ProjectConcurrencyExceeded { project_id, .. }
        | GatewayTrafficAdmissionError::ApiKeyConcurrencyExceeded { project_id, .. } => {
            event_dimensions = event_dimensions.with_tenant(project_id.clone());
            annotate_current_http_metrics(|dimensions| {
                dimensions.tenant = Some(project_id.clone());
            });
        }
        GatewayTrafficAdmissionError::ProviderBackpressureExceeded {
            provider_id,
            project_id,
            ..
        } => {
            event_dimensions = event_dimensions
                .with_provider(provider_id.clone())
                .with_tenant(project_id.clone());
            annotate_current_http_metrics(|dimensions| {
                dimensions.provider = Some(provider_id.clone());
                dimensions.tenant = Some(project_id.clone());
            });
        }
    }
    record_current_commercial_event(CommercialEventKind::Throttling, event_dimensions);
    let mut openai_error = OpenAiErrorResponse::new(error.message(), "rate_limit_error");
    openai_error.error.code = Some(error.code().to_owned());

    let status = match error {
        GatewayTrafficAdmissionError::ProjectConcurrencyExceeded { .. }
        | GatewayTrafficAdmissionError::ApiKeyConcurrencyExceeded { .. } => {
            StatusCode::TOO_MANY_REQUESTS
        }
        GatewayTrafficAdmissionError::ProviderBackpressureExceeded { .. } => {
            StatusCode::SERVICE_UNAVAILABLE
        }
    };

    (status, Json(openai_error)).into_response()
}

fn rate_limit_exceeded_message(
    project_id: &str,
    route_key: &str,
    evaluation: &RateLimitCheckResult,
) -> String {
    match (evaluation.policy_id.as_deref(), evaluation.limit_requests) {
        (Some(policy_id), Some(limit_requests)) => format!(
            "Rate limit exceeded for project {project_id} on route {route_key} under policy {policy_id}: requested {} requests with {} already used against a limit of {limit_requests}.",
            evaluation.requested_requests, evaluation.used_requests,
        ),
        (_, Some(limit_requests)) => format!(
            "Rate limit exceeded for project {project_id} on route {route_key}: requested {} requests with {} already used against a limit of {limit_requests}.",
            evaluation.requested_requests, evaluation.used_requests,
        ),
        _ => format!(
            "Rate limit exceeded for project {project_id} on route {route_key}: requested {} requests with {} already used.",
            evaluation.requested_requests, evaluation.used_requests,
        ),
    }
}

fn extract_bearer_token(headers: &axum::http::HeaderMap) -> Option<String> {
    let header_value = header_value(headers.get(header::AUTHORIZATION))?;
    header_value
        .strip_prefix("Bearer ")
        .or_else(|| header_value.strip_prefix("bearer "))
        .map(ToOwned::to_owned)
}

fn header_value(value: Option<&axum::http::HeaderValue>) -> Option<String> {
    value
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
}

fn request_options_from_header_names(
    headers: &HeaderMap,
    header_names: &[&str],
) -> ProviderRequestOptions {
    header_names.iter().fold(
        ProviderRequestOptions::default(),
        |options, name| match header_value(headers.get(*name)) {
            Some(value) => options.with_header(*name, value),
            None => options,
        },
    )
}

fn anthropic_request_options(headers: &HeaderMap) -> ProviderRequestOptions {
    request_options_from_header_names(headers, &["anthropic-version", "anthropic-beta"])
}

fn query_parameter(query: Option<&str>, key: &str) -> Option<String> {
    let query = query?;
    query.split('&').find_map(|pair| {
        let (name, value) = pair.split_once('=')?;
        if name == key {
            Some(value.to_owned())
        } else {
            None
        }
    })
}

fn current_gateway_request_context() -> Option<IdentityGatewayRequestContext> {
    CURRENT_GATEWAY_REQUEST_CONTEXT.try_with(Clone::clone).ok()
}

fn current_gateway_request_latency_ms() -> Option<u64> {
    CURRENT_GATEWAY_REQUEST_STARTED_AT
        .try_with(|started_at| started_at.elapsed().as_millis() as u64)
        .ok()
}

async fn resolve_optional_gateway_canonical_subject(
    store: &dyn AdminStore,
    token: &str,
) -> anyhow::Result<Option<GatewayAuthSubject>> {
    let Some(identity_store) = store.identity_kernel() else {
        return Ok(None);
    };

    resolve_gateway_auth_subject_from_api_key(identity_store, token).await
}

struct ResolvedGatewayAuthentication {
    context: Option<IdentityGatewayRequestContext>,
    canonical_subject: Option<GatewayAuthSubject>,
}

async fn resolve_gateway_authentication(
    store: &dyn AdminStore,
    token: &str,
) -> anyhow::Result<ResolvedGatewayAuthentication> {
    let canonical_subject = resolve_optional_gateway_canonical_subject(store, token).await?;
    let context = match resolve_gateway_request_context(store, token).await? {
        Some(context) => Some(context),
        None => match canonical_subject.as_ref() {
            Some(subject) => {
                resolve_gateway_request_context_from_auth_subject(store, subject).await?
            }
            None => None,
        },
    };

    Ok(ResolvedGatewayAuthentication {
        context,
        canonical_subject,
    })
}

async fn apply_gateway_request_context(
    State(state): State<GatewayApiState>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    let token = extract_bearer_token(request.headers())
        .or_else(|| header_value(request.headers().get("x-api-key")))
        .or_else(|| header_value(request.headers().get("x-goog-api-key")))
        .or_else(|| query_parameter(request.uri().query(), "key"));

    let Some(token) = token else {
        return next.run(request).await;
    };

    let resolved = match resolve_gateway_authentication(state.store.as_ref(), &token).await {
        Ok(resolved) => resolved,
        Err(_) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to resolve canonical gateway subject",
            )
                .into_response();
        }
    };
    if let Some(subject) = resolved.canonical_subject {
        request
            .extensions_mut()
            .insert(CanonicalGatewayAuthSubject(subject));
    }

    let Some(context) = resolved.context else {
        return next.run(request).await;
    };

    let traffic_context = GatewayTrafficRequestContext::new(
        context.tenant_id(),
        context.project_id(),
        context.api_key_hash(),
        request.uri().path(),
    )
    .with_api_key_group_id_option(context.api_key_group_id.clone());
    request.extensions_mut().insert(context.clone());
    annotate_current_http_metrics(|dimensions| {
        dimensions.tenant = Some(context.tenant_id().to_owned());
    });
    with_gateway_traffic_context(
        Arc::clone(&state.traffic_controller),
        traffic_context,
        CURRENT_GATEWAY_REQUEST_CONTEXT.scope(
            context,
            with_request_api_key_group_id(
                request
                    .extensions()
                    .get::<IdentityGatewayRequestContext>()
                    .and_then(|context| context.api_key_group_id.clone()),
                CURRENT_GATEWAY_REQUEST_STARTED_AT.scope(Instant::now(), next.run(request)),
            ),
        ),
    )
    .await
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatelessGatewayUpstream {
    runtime_key: String,
    base_url: String,
    api_key: String,
}

impl StatelessGatewayUpstream {
    pub fn new(
        runtime_key: impl Into<String>,
        base_url: impl Into<String>,
        api_key: impl Into<String>,
    ) -> Self {
        Self {
            runtime_key: runtime_key.into(),
            base_url: base_url.into(),
            api_key: api_key.into(),
        }
    }

    pub fn from_adapter_kind(
        adapter_kind: impl Into<String>,
        base_url: impl Into<String>,
        api_key: impl Into<String>,
    ) -> Self {
        Self::new(adapter_kind, base_url, api_key)
    }

    pub fn runtime_key(&self) -> &str {
        &self.runtime_key
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn api_key(&self) -> &str {
        &self.api_key
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatelessGatewayConfig {
    tenant_id: String,
    project_id: String,
    upstream: Option<StatelessGatewayUpstream>,
}

impl Default for StatelessGatewayConfig {
    fn default() -> Self {
        Self {
            tenant_id: DEFAULT_STATELESS_TENANT_ID.to_owned(),
            project_id: DEFAULT_STATELESS_PROJECT_ID.to_owned(),
            upstream: None,
        }
    }
}

impl StatelessGatewayConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_identity(
        mut self,
        tenant_id: impl Into<String>,
        project_id: impl Into<String>,
    ) -> Self {
        self.tenant_id = tenant_id.into();
        self.project_id = project_id.into();
        self
    }

    pub fn with_upstream(mut self, upstream: StatelessGatewayUpstream) -> Self {
        self.upstream = Some(upstream);
        self
    }

    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    pub fn project_id(&self) -> &str {
        &self.project_id
    }

    pub fn upstream(&self) -> Option<&StatelessGatewayUpstream> {
        self.upstream.as_ref()
    }

    fn into_context(self) -> StatelessGatewayContext {
        StatelessGatewayContext {
            tenant_id: Arc::from(self.tenant_id),
            project_id: Arc::from(self.project_id),
            upstream: self.upstream.map(Arc::new),
        }
    }
}

#[derive(Clone, Debug)]
struct StatelessGatewayContext {
    tenant_id: Arc<str>,
    project_id: Arc<str>,
    upstream: Option<Arc<StatelessGatewayUpstream>>,
}

#[derive(Clone, Debug)]
struct StatelessGatewayRequest(StatelessGatewayContext);

impl StatelessGatewayRequest {
    fn tenant_id(&self) -> &str {
        &self.0.tenant_id
    }

    fn project_id(&self) -> &str {
        &self.0.project_id
    }

    fn upstream(&self) -> Option<&StatelessGatewayUpstream> {
        self.0.upstream.as_deref()
    }
}

impl FromRequestParts<StatelessGatewayContext> for StatelessGatewayRequest {
    type Rejection = StatusCode;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &StatelessGatewayContext,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self(state.clone()))
    }
}

async fn apply_request_routing_region(request: Request<Body>, next: Next) -> Response {
    let requested_region = request
        .headers()
        .get("x-sdkwork-region")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    with_request_routing_region(requested_region, next.run(request)).await
}

pub fn gateway_router() -> Router {
    gateway_router_with_stateless_config(StatelessGatewayConfig::default())
}

pub fn gateway_router_with_stateless_config(config: StatelessGatewayConfig) -> Router {
    let service_name: Arc<str> = Arc::from("gateway");
    let metrics = Arc::new(HttpMetricsRegistry::new("gateway"));
    Router::new()
        .route("/openapi.json", get(gateway_openapi_handler))
        .route("/docs", get(gateway_docs_handler))
        .route(
            "/metrics",
            get({
                let metrics = metrics.clone();
                move || {
                    let metrics = metrics.clone();
                    async move {
                        (
                            [(
                                header::CONTENT_TYPE,
                                "text/plain; version=0.0.4; charset=utf-8",
                            )],
                            metrics.render_prometheus(),
                        )
                    }
                }
            }),
        )
        .route("/health", get(|| async { "ok" }))
        .route("/v1/messages", post(anthropic_messages_handler))
        .route(
            "/v1/messages/count_tokens",
            post(anthropic_count_tokens_handler),
        )
        .route("/v1beta/models/{*tail}", post(gemini_models_compat_handler))
        .route("/v1/models", get(list_models_handler))
        .route(
            "/v1/models/{model_id}",
            get(model_retrieve_handler).delete(model_delete_handler),
        )
        .route(
            "/v1/chat/completions",
            get(chat_completions_list_handler).post(chat_completions_handler),
        )
        .route(
            "/v1/chat/completions/{completion_id}",
            get(chat_completion_retrieve_handler)
                .post(chat_completion_update_handler)
                .delete(chat_completion_delete_handler),
        )
        .route(
            "/v1/chat/completions/{completion_id}/messages",
            get(chat_completion_messages_list_handler),
        )
        .route("/v1/completions", post(completions_handler))
        .route(
            "/v1/conversations",
            get(conversations_list_handler).post(conversations_handler),
        )
        .route(
            "/v1/conversations/{conversation_id}",
            get(conversation_retrieve_handler)
                .post(conversation_update_handler)
                .delete(conversation_delete_handler),
        )
        .route(
            "/v1/conversations/{conversation_id}/items",
            get(conversation_items_list_handler).post(conversation_items_handler),
        )
        .route(
            "/v1/conversations/{conversation_id}/items/{item_id}",
            get(conversation_item_retrieve_handler).delete(conversation_item_delete_handler),
        )
        .route("/v1/threads", post(threads_handler))
        .route(
            "/v1/threads/{thread_id}",
            get(thread_retrieve_handler)
                .post(thread_update_handler)
                .delete(thread_delete_handler),
        )
        .route(
            "/v1/threads/{thread_id}/messages",
            get(thread_messages_list_handler).post(thread_messages_handler),
        )
        .route(
            "/v1/threads/{thread_id}/messages/{message_id}",
            get(thread_message_retrieve_handler)
                .post(thread_message_update_handler)
                .delete(thread_message_delete_handler),
        )
        .route("/v1/threads/runs", post(thread_and_run_handler))
        .route(
            "/v1/threads/{thread_id}/runs",
            get(thread_runs_list_handler).post(thread_runs_handler),
        )
        .route(
            "/v1/threads/{thread_id}/runs/{run_id}",
            get(thread_run_retrieve_handler).post(thread_run_update_handler),
        )
        .route(
            "/v1/threads/{thread_id}/runs/{run_id}/cancel",
            post(thread_run_cancel_handler),
        )
        .route(
            "/v1/threads/{thread_id}/runs/{run_id}/submit_tool_outputs",
            post(thread_run_submit_tool_outputs_handler),
        )
        .route(
            "/v1/threads/{thread_id}/runs/{run_id}/steps",
            get(thread_run_steps_list_handler),
        )
        .route(
            "/v1/threads/{thread_id}/runs/{run_id}/steps/{step_id}",
            get(thread_run_step_retrieve_handler),
        )
        .route("/v1/responses", post(responses_handler))
        .route(
            "/v1/responses/input_tokens",
            post(response_input_tokens_handler),
        )
        .route("/v1/responses/compact", post(response_compact_handler))
        .route(
            "/v1/responses/{response_id}",
            get(response_retrieve_handler).delete(response_delete_handler),
        )
        .route(
            "/v1/responses/{response_id}/input_items",
            get(response_input_items_list_handler),
        )
        .route(
            "/v1/responses/{response_id}/cancel",
            post(response_cancel_handler),
        )
        .route("/v1/embeddings", post(embeddings_handler))
        .route("/v1/moderations", post(moderations_handler))
        .route("/v1/images/generations", post(image_generations_handler))
        .route("/v1/images/edits", post(image_edits_handler))
        .route("/v1/images/variations", post(image_variations_handler))
        .route("/v1/audio/transcriptions", post(transcriptions_handler))
        .route("/v1/audio/translations", post(translations_handler))
        .route("/v1/audio/speech", post(audio_speech_handler))
        .route("/v1/audio/voices", get(audio_voices_handler))
        .route(
            "/v1/audio/voice_consents",
            post(audio_voice_consents_handler),
        )
        .route(
            "/v1/containers",
            get(containers_list_handler).post(containers_handler),
        )
        .route(
            "/v1/containers/{container_id}",
            get(container_retrieve_handler).delete(container_delete_handler),
        )
        .route(
            "/v1/containers/{container_id}/files",
            get(container_files_list_handler).post(container_files_handler),
        )
        .route(
            "/v1/containers/{container_id}/files/{file_id}",
            get(container_file_retrieve_handler).delete(container_file_delete_handler),
        )
        .route(
            "/v1/containers/{container_id}/files/{file_id}/content",
            get(container_file_content_handler),
        )
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
        .route(
            "/v1/videos/characters",
            post(video_character_create_handler),
        )
        .route(
            "/v1/videos/characters/{character_id}",
            get(video_character_retrieve_canonical_handler),
        )
        .route("/v1/videos/edits", post(video_edits_handler))
        .route("/v1/videos/extensions", post(video_extensions_handler))
        .route(
            "/v1/videos/{video_id}/characters",
            get(video_characters_list_handler),
        )
        .route(
            "/v1/videos/{video_id}/characters/{character_id}",
            get(video_character_retrieve_handler).post(video_character_update_handler),
        )
        .route("/v1/videos/{video_id}/extend", post(video_extend_handler))
        .route("/v1/music", get(music_list_handler).post(music_handler))
        .route(
            "/v1/music/{music_id}",
            get(music_retrieve_handler).delete(music_delete_handler),
        )
        .route("/v1/music/{music_id}/content", get(music_content_handler))
        .route("/v1/music/lyrics", post(music_lyrics_handler))
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
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}/events",
            get(fine_tuning_job_events_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}/checkpoints",
            get(fine_tuning_job_checkpoints_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}/pause",
            post(fine_tuning_job_pause_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}/resume",
            post(fine_tuning_job_resume_handler),
        )
        .route(
            "/v1/fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions",
            get(fine_tuning_checkpoint_permissions_list_handler)
                .post(fine_tuning_checkpoint_permissions_handler),
        )
        .route(
            "/v1/fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions/{permission_id}",
            axum::routing::delete(fine_tuning_checkpoint_permission_delete_handler),
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
        .route("/v1/evals", get(evals_list_handler).post(evals_handler))
        .route(
            "/v1/evals/{eval_id}",
            get(eval_retrieve_handler)
                .post(eval_update_handler)
                .delete(eval_delete_handler),
        )
        .route(
            "/v1/evals/{eval_id}/runs",
            get(eval_runs_list_handler).post(eval_runs_handler),
        )
        .route(
            "/v1/evals/{eval_id}/runs/{run_id}",
            get(eval_run_retrieve_handler).delete(eval_run_delete_handler),
        )
        .route(
            "/v1/evals/{eval_id}/runs/{run_id}/cancel",
            post(eval_run_cancel_handler),
        )
        .route(
            "/v1/evals/{eval_id}/runs/{run_id}/output_items",
            get(eval_run_output_items_list_handler),
        )
        .route(
            "/v1/evals/{eval_id}/runs/{run_id}/output_items/{output_item_id}",
            get(eval_run_output_item_retrieve_handler),
        )
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
        .layer(axum::middleware::from_fn(apply_request_routing_region))
        .layer(axum::middleware::from_fn_with_state(
            metrics,
            observe_http_metrics,
        ))
        .layer(browser_cors_layer())
        .layer(axum::middleware::from_fn_with_state(
            service_name,
            observe_http_tracing,
        ))
        .with_state(config.into_context())
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
    gateway_router_with_state(GatewayApiState::with_store_and_secret_manager(
        store,
        secret_manager,
    ))
}

pub fn gateway_router_with_state(state: GatewayApiState) -> Router {
    let service_name: Arc<str> = Arc::from("gateway");
    let metrics = Arc::new(HttpMetricsRegistry::new("gateway"));
    Router::new()
        .route("/openapi.json", get(gateway_openapi_handler))
        .route("/docs", get(gateway_docs_handler))
        .route(
            "/metrics",
            get({
                let metrics = metrics.clone();
                move || {
                    let metrics = metrics.clone();
                    async move {
                        (
                            [(
                                header::CONTENT_TYPE,
                                "text/plain; version=0.0.4; charset=utf-8",
                            )],
                            metrics.render_prometheus(),
                        )
                    }
                }
            }),
        )
        .route("/health", get(|| async { "ok" }))
        .route("/v1/messages", post(anthropic_messages_with_state_handler))
        .route(
            "/v1/messages/count_tokens",
            post(anthropic_count_tokens_with_state_handler),
        )
        .route(
            "/v1beta/models/{*tail}",
            post(gemini_models_compat_with_state_handler),
        )
        .route("/v1/models", get(list_models_from_store_handler))
        .route(
            "/v1/models/{model_id}",
            get(model_retrieve_from_store_handler).delete(model_delete_from_store_handler),
        )
        .route(
            "/v1/chat/completions",
            get(chat_completions_list_with_state_handler).post(chat_completions_with_state_handler),
        )
        .route(
            "/v1/chat/completions/{completion_id}",
            get(chat_completion_retrieve_with_state_handler)
                .post(chat_completion_update_with_state_handler)
                .delete(chat_completion_delete_with_state_handler),
        )
        .route(
            "/v1/chat/completions/{completion_id}/messages",
            get(chat_completion_messages_list_with_state_handler),
        )
        .route("/v1/completions", post(completions_with_state_handler))
        .route(
            "/v1/conversations",
            get(conversations_list_with_state_handler).post(conversations_with_state_handler),
        )
        .route(
            "/v1/conversations/{conversation_id}",
            get(conversation_retrieve_with_state_handler)
                .post(conversation_update_with_state_handler)
                .delete(conversation_delete_with_state_handler),
        )
        .route(
            "/v1/conversations/{conversation_id}/items",
            get(conversation_items_list_with_state_handler)
                .post(conversation_items_with_state_handler),
        )
        .route(
            "/v1/conversations/{conversation_id}/items/{item_id}",
            get(conversation_item_retrieve_with_state_handler)
                .delete(conversation_item_delete_with_state_handler),
        )
        .route("/v1/threads", post(threads_with_state_handler))
        .route(
            "/v1/threads/{thread_id}",
            get(thread_retrieve_with_state_handler)
                .post(thread_update_with_state_handler)
                .delete(thread_delete_with_state_handler),
        )
        .route(
            "/v1/threads/{thread_id}/messages",
            get(thread_messages_list_with_state_handler).post(thread_messages_with_state_handler),
        )
        .route(
            "/v1/threads/{thread_id}/messages/{message_id}",
            get(thread_message_retrieve_with_state_handler)
                .post(thread_message_update_with_state_handler)
                .delete(thread_message_delete_with_state_handler),
        )
        .route("/v1/threads/runs", post(thread_and_run_with_state_handler))
        .route(
            "/v1/threads/{thread_id}/runs",
            get(thread_runs_list_with_state_handler).post(thread_runs_with_state_handler),
        )
        .route(
            "/v1/threads/{thread_id}/runs/{run_id}",
            get(thread_run_retrieve_with_state_handler).post(thread_run_update_with_state_handler),
        )
        .route(
            "/v1/threads/{thread_id}/runs/{run_id}/cancel",
            post(thread_run_cancel_with_state_handler),
        )
        .route(
            "/v1/threads/{thread_id}/runs/{run_id}/submit_tool_outputs",
            post(thread_run_submit_tool_outputs_with_state_handler),
        )
        .route(
            "/v1/threads/{thread_id}/runs/{run_id}/steps",
            get(thread_run_steps_list_with_state_handler),
        )
        .route(
            "/v1/threads/{thread_id}/runs/{run_id}/steps/{step_id}",
            get(thread_run_step_retrieve_with_state_handler),
        )
        .route("/v1/responses", post(responses_with_state_handler))
        .route(
            "/v1/responses/input_tokens",
            post(response_input_tokens_with_state_handler),
        )
        .route(
            "/v1/responses/compact",
            post(response_compact_with_state_handler),
        )
        .route(
            "/v1/responses/{response_id}",
            get(response_retrieve_with_state_handler).delete(response_delete_with_state_handler),
        )
        .route(
            "/v1/responses/{response_id}/input_items",
            get(response_input_items_list_with_state_handler),
        )
        .route(
            "/v1/responses/{response_id}/cancel",
            post(response_cancel_with_state_handler),
        )
        .route("/v1/embeddings", post(embeddings_with_state_handler))
        .route("/v1/moderations", post(moderations_with_state_handler))
        .route(
            "/v1/images/generations",
            post(image_generations_with_state_handler),
        )
        .route("/v1/images/edits", post(image_edits_with_state_handler))
        .route(
            "/v1/images/variations",
            post(image_variations_with_state_handler),
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
        .route("/v1/audio/voices", get(audio_voices_with_state_handler))
        .route(
            "/v1/audio/voice_consents",
            post(audio_voice_consents_with_state_handler),
        )
        .route(
            "/v1/containers",
            get(containers_list_with_state_handler).post(containers_with_state_handler),
        )
        .route(
            "/v1/containers/{container_id}",
            get(container_retrieve_with_state_handler).delete(container_delete_with_state_handler),
        )
        .route(
            "/v1/containers/{container_id}/files",
            get(container_files_list_with_state_handler).post(container_files_with_state_handler),
        )
        .route(
            "/v1/containers/{container_id}/files/{file_id}",
            get(container_file_retrieve_with_state_handler)
                .delete(container_file_delete_with_state_handler),
        )
        .route(
            "/v1/containers/{container_id}/files/{file_id}/content",
            get(container_file_content_with_state_handler),
        )
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
        .route(
            "/v1/videos/characters",
            post(video_character_create_with_state_handler),
        )
        .route(
            "/v1/videos/characters/{character_id}",
            get(video_character_retrieve_canonical_with_state_handler),
        )
        .route("/v1/videos/edits", post(video_edits_with_state_handler))
        .route(
            "/v1/videos/extensions",
            post(video_extensions_with_state_handler),
        )
        .route(
            "/v1/videos/{video_id}/characters",
            get(video_characters_list_with_state_handler),
        )
        .route(
            "/v1/videos/{video_id}/characters/{character_id}",
            get(video_character_retrieve_with_state_handler)
                .post(video_character_update_with_state_handler),
        )
        .route(
            "/v1/videos/{video_id}/extend",
            post(video_extend_with_state_handler),
        )
        .route(
            "/v1/music",
            get(music_list_with_state_handler).post(music_with_state_handler),
        )
        .route(
            "/v1/music/{music_id}",
            get(music_retrieve_with_state_handler).delete(music_delete_with_state_handler),
        )
        .route(
            "/v1/music/{music_id}/content",
            get(music_content_with_state_handler),
        )
        .route("/v1/music/lyrics", post(music_lyrics_with_state_handler))
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
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}/events",
            get(fine_tuning_job_events_with_state_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}/checkpoints",
            get(fine_tuning_job_checkpoints_with_state_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}/pause",
            post(fine_tuning_job_pause_with_state_handler),
        )
        .route(
            "/v1/fine_tuning/jobs/{fine_tuning_job_id}/resume",
            post(fine_tuning_job_resume_with_state_handler),
        )
        .route(
            "/v1/fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions",
            get(fine_tuning_checkpoint_permissions_list_with_state_handler)
                .post(fine_tuning_checkpoint_permissions_with_state_handler),
        )
        .route(
            "/v1/fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions/{permission_id}",
            axum::routing::delete(fine_tuning_checkpoint_permission_delete_with_state_handler),
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
        .route(
            "/v1/evals",
            get(evals_list_with_state_handler).post(evals_with_state_handler),
        )
        .route(
            "/v1/evals/{eval_id}",
            get(eval_retrieve_with_state_handler)
                .post(eval_update_with_state_handler)
                .delete(eval_delete_with_state_handler),
        )
        .route(
            "/v1/evals/{eval_id}/runs",
            get(eval_runs_list_with_state_handler).post(eval_runs_with_state_handler),
        )
        .route(
            "/v1/evals/{eval_id}/runs/{run_id}",
            get(eval_run_retrieve_with_state_handler).delete(eval_run_delete_with_state_handler),
        )
        .route(
            "/v1/evals/{eval_id}/runs/{run_id}/cancel",
            post(eval_run_cancel_with_state_handler),
        )
        .route(
            "/v1/evals/{eval_id}/runs/{run_id}/output_items",
            get(eval_run_output_items_list_with_state_handler),
        )
        .route(
            "/v1/evals/{eval_id}/runs/{run_id}/output_items/{output_item_id}",
            get(eval_run_output_item_retrieve_with_state_handler),
        )
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
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            apply_gateway_request_context,
        ))
        .layer(axum::middleware::from_fn(apply_request_routing_region))
        .layer(axum::middleware::from_fn_with_state(
            metrics,
            observe_http_metrics,
        ))
        .layer(axum::middleware::from_fn_with_state(
            service_name,
            observe_http_tracing,
        ))
        .with_state(state)
}

async fn list_models_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::ModelsList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream model list");
        }
    }

    Json(
        list_models(request_context.tenant_id(), request_context.project_id())
            .expect("models response"),
    )
    .into_response()
}

async fn model_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(model_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::ModelsRetrieve(&model_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream model");
        }
    }

    Json(
        get_model(
            request_context.tenant_id(),
            request_context.project_id(),
            &model_id,
        )
        .expect("model response"),
    )
    .into_response()
}

async fn model_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(model_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::ModelsDelete(&model_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream model delete");
        }
    }

    Json(
        delete_model(
            request_context.tenant_id(),
            request_context.project_id(),
            &model_id,
        )
        .expect("model delete response"),
    )
    .into_response()
}

async fn list_models_from_store_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Result<Json<sdkwork_api_contract_openai::models::ListModelsResponse>, Response> {
    list_models_from_store(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
    )
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
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(model_id): Path<String>,
) -> Result<Json<sdkwork_api_contract_openai::models::ModelObject>, Response> {
    get_model_from_store(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        &model_id,
    )
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

async fn model_delete_from_store_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(model_id): Path<String>,
) -> Result<Json<Value>, Response> {
    delete_model_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &model_id,
    )
    .await
    .map_err(|_| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to delete model",
        )
            .into_response()
    })?
    .map(Json)
    .ok_or_else(|| (axum::http::StatusCode::NOT_FOUND, "model not found").into_response())
}

async fn chat_completions_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateChatCompletionRequest>,
) -> Response {
    if request.stream.unwrap_or(false) {
        match relay_stateless_stream_request(
            &request_context,
            ProviderRequest::ChatCompletionsStream(&request),
        )
        .await
        {
            Ok(Some(response)) => return upstream_passthrough_response(response),
            Ok(None) => {}
            Err(_) => {
                return bad_gateway_openai_response(
                    "failed to relay upstream chat completion stream",
                );
            }
        }

        return local_chat_completion_stream_response(&request.model);
    }

    match relay_stateless_json_request(&request_context, ProviderRequest::ChatCompletions(&request))
        .await
    {
        Ok(Some(response)) => Json(response).into_response(),
        Ok(None) => Json(
            create_chat_completion(
                request_context.tenant_id(),
                request_context.project_id(),
                &request.model,
            )
            .expect("chat completion"),
        )
        .into_response(),
        Err(_) => bad_gateway_openai_response("failed to relay upstream chat completion"),
    }
}

async fn chat_completions_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::ChatCompletionsList).await
    {
        Ok(Some(response)) => Json(response).into_response(),
        Ok(None) => Json(
            list_chat_completions(request_context.tenant_id(), request_context.project_id())
                .expect("chat completions"),
        )
        .into_response(),
        Err(_) => bad_gateway_openai_response("failed to relay upstream chat completion list"),
    }
}

async fn chat_completion_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(completion_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ChatCompletionsRetrieve(&completion_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream chat completion retrieve",
            );
        }
    }

    Json(
        get_chat_completion(
            request_context.tenant_id(),
            request_context.project_id(),
            &completion_id,
        )
        .expect("chat completion"),
    )
    .into_response()
}

async fn chat_completion_update_handler(
    request_context: StatelessGatewayRequest,
    Path(completion_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateChatCompletionRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ChatCompletionsUpdate(&completion_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream chat completion update");
        }
    }

    Json(
        update_chat_completion(
            request_context.tenant_id(),
            request_context.project_id(),
            &completion_id,
            request.metadata.unwrap_or(serde_json::json!({})),
        )
        .expect("chat completion update"),
    )
    .into_response()
}

async fn chat_completion_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(completion_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ChatCompletionsDelete(&completion_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream chat completion delete");
        }
    }

    Json(
        delete_chat_completion(
            request_context.tenant_id(),
            request_context.project_id(),
            &completion_id,
        )
        .expect("chat completion delete"),
    )
    .into_response()
}

async fn chat_completion_messages_list_handler(
    request_context: StatelessGatewayRequest,
    Path(completion_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ChatCompletionsMessagesList(&completion_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream chat completion messages",
            );
        }
    }

    Json(
        list_chat_completion_messages(
            request_context.tenant_id(),
            request_context.project_id(),
            &completion_id,
        )
        .expect("chat completion messages"),
    )
    .into_response()
}

async fn conversations_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateConversationRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Conversations(&request))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream conversation");
        }
    }

    Json(
        create_conversation(request_context.tenant_id(), request_context.project_id())
            .expect("conversation"),
    )
    .into_response()
}

async fn conversations_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::ConversationsList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream conversation list");
        }
    }

    Json(
        list_conversations(request_context.tenant_id(), request_context.project_id())
            .expect("conversation list"),
    )
    .into_response()
}

async fn conversation_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(conversation_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ConversationsRetrieve(&conversation_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream conversation retrieve");
        }
    }

    Json(
        get_conversation(
            request_context.tenant_id(),
            request_context.project_id(),
            &conversation_id,
        )
        .expect("conversation"),
    )
    .into_response()
}

async fn conversation_update_handler(
    request_context: StatelessGatewayRequest,
    Path(conversation_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateConversationRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ConversationsUpdate(&conversation_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream conversation update");
        }
    }

    Json(
        update_conversation(
            request_context.tenant_id(),
            request_context.project_id(),
            &conversation_id,
            request.metadata.unwrap_or(serde_json::json!({})),
        )
        .expect("conversation update"),
    )
    .into_response()
}

async fn conversation_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(conversation_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ConversationsDelete(&conversation_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream conversation delete");
        }
    }

    Json(
        delete_conversation(
            request_context.tenant_id(),
            request_context.project_id(),
            &conversation_id,
        )
        .expect("conversation delete"),
    )
    .into_response()
}

async fn conversation_items_handler(
    request_context: StatelessGatewayRequest,
    Path(conversation_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateConversationItemsRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ConversationItems(&conversation_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream conversation items");
        }
    }

    Json(
        create_conversation_items(
            request_context.tenant_id(),
            request_context.project_id(),
            &conversation_id,
        )
        .expect("conversation items create"),
    )
    .into_response()
}

async fn conversation_items_list_handler(
    request_context: StatelessGatewayRequest,
    Path(conversation_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ConversationItemsList(&conversation_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream conversation items list");
        }
    }

    Json(
        list_conversation_items(
            request_context.tenant_id(),
            request_context.project_id(),
            &conversation_id,
        )
        .expect("conversation items list"),
    )
    .into_response()
}

async fn conversation_item_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((conversation_id, item_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ConversationItemsRetrieve(&conversation_id, &item_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream conversation item retrieve",
            );
        }
    }

    Json(
        get_conversation_item(
            request_context.tenant_id(),
            request_context.project_id(),
            &conversation_id,
            &item_id,
        )
        .expect("conversation item"),
    )
    .into_response()
}

async fn conversation_item_delete_handler(
    request_context: StatelessGatewayRequest,
    Path((conversation_id, item_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ConversationItemsDelete(&conversation_id, &item_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream conversation item delete",
            );
        }
    }

    Json(
        delete_conversation_item(
            request_context.tenant_id(),
            request_context.project_id(),
            &conversation_id,
            &item_id,
        )
        .expect("conversation item delete"),
    )
    .into_response()
}

async fn threads_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateThreadRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Threads(&request)).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread");
        }
    }

    Json(create_thread(request_context.tenant_id(), request_context.project_id()).expect("thread"))
        .into_response()
}

async fn thread_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(thread_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadsRetrieve(&thread_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread retrieve");
        }
    }

    Json(
        get_thread(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
        )
        .expect("thread retrieve"),
    )
    .into_response()
}

async fn thread_update_handler(
    request_context: StatelessGatewayRequest,
    Path(thread_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateThreadRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadsUpdate(&thread_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread update");
        }
    }

    Json(
        update_thread(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
        )
        .expect("thread update"),
    )
    .into_response()
}

async fn thread_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(thread_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::ThreadsDelete(&thread_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread delete");
        }
    }

    Json(
        delete_thread(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
        )
        .expect("thread delete"),
    )
    .into_response()
}

async fn thread_messages_handler(
    request_context: StatelessGatewayRequest,
    Path(thread_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateThreadMessageRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadMessages(&thread_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread message");
        }
    }

    let text = request.content.as_str().unwrap_or("hello");
    Json(
        create_thread_message(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
            &request.role,
            text,
        )
        .expect("thread message create"),
    )
    .into_response()
}

async fn thread_messages_list_handler(
    request_context: StatelessGatewayRequest,
    Path(thread_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadMessagesList(&thread_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread messages list");
        }
    }

    Json(
        list_thread_messages(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
        )
        .expect("thread messages list"),
    )
    .into_response()
}

async fn thread_message_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((thread_id, message_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadMessagesRetrieve(&thread_id, &message_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread message retrieve");
        }
    }

    Json(
        get_thread_message(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
            &message_id,
        )
        .expect("thread message retrieve"),
    )
    .into_response()
}

async fn thread_message_update_handler(
    request_context: StatelessGatewayRequest,
    Path((thread_id, message_id)): Path<(String, String)>,
    ExtractJson(request): ExtractJson<UpdateThreadMessageRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadMessagesUpdate(&thread_id, &message_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread message update");
        }
    }

    Json(
        update_thread_message(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
            &message_id,
        )
        .expect("thread message update"),
    )
    .into_response()
}

async fn thread_message_delete_handler(
    request_context: StatelessGatewayRequest,
    Path((thread_id, message_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadMessagesDelete(&thread_id, &message_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread message delete");
        }
    }

    Json(
        delete_thread_message(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
            &message_id,
        )
        .expect("thread message delete"),
    )
    .into_response()
}

async fn thread_and_run_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateThreadAndRunRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::ThreadsRuns(&request))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread and run");
        }
    }

    Json(
        create_thread_and_run(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.assistant_id,
        )
        .expect("thread and run create"),
    )
    .into_response()
}

async fn thread_runs_handler(
    request_context: StatelessGatewayRequest,
    Path(thread_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateRunRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadRuns(&thread_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread run");
        }
    }

    Json(
        create_thread_run(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
            &request.assistant_id,
            request.model.as_deref(),
        )
        .expect("thread run create"),
    )
    .into_response()
}

async fn thread_runs_list_handler(
    request_context: StatelessGatewayRequest,
    Path(thread_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadRunsList(&thread_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread runs list");
        }
    }

    Json(
        list_thread_runs(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
        )
        .expect("thread runs list"),
    )
    .into_response()
}

async fn thread_run_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((thread_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadRunsRetrieve(&thread_id, &run_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread run retrieve");
        }
    }

    Json(
        get_thread_run(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
            &run_id,
        )
        .expect("thread run"),
    )
    .into_response()
}

async fn thread_run_update_handler(
    request_context: StatelessGatewayRequest,
    Path((thread_id, run_id)): Path<(String, String)>,
    ExtractJson(request): ExtractJson<UpdateRunRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadRunsUpdate(&thread_id, &run_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread run update");
        }
    }

    Json(
        update_thread_run(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
            &run_id,
        )
        .expect("thread run update"),
    )
    .into_response()
}

async fn thread_run_cancel_handler(
    request_context: StatelessGatewayRequest,
    Path((thread_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadRunsCancel(&thread_id, &run_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread run cancel");
        }
    }

    Json(
        cancel_thread_run(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
            &run_id,
        )
        .expect("thread run cancel"),
    )
    .into_response()
}

async fn thread_run_submit_tool_outputs_handler(
    request_context: StatelessGatewayRequest,
    Path((thread_id, run_id)): Path<(String, String)>,
    ExtractJson(request): ExtractJson<SubmitToolOutputsRunRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadRunsSubmitToolOutputs(&thread_id, &run_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream thread run submit tool outputs",
            );
        }
    }

    let tool_outputs = request
        .tool_outputs
        .iter()
        .map(|output| (output.tool_call_id.as_str(), output.output.as_str()))
        .collect();
    Json(
        submit_thread_run_tool_outputs(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
            &run_id,
            tool_outputs,
        )
        .expect("thread run submit tool outputs"),
    )
    .into_response()
}

async fn thread_run_steps_list_handler(
    request_context: StatelessGatewayRequest,
    Path((thread_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadRunStepsList(&thread_id, &run_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread run steps list");
        }
    }

    Json(
        list_thread_run_steps(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
            &run_id,
        )
        .expect("thread run steps"),
    )
    .into_response()
}

async fn thread_run_step_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((thread_id, run_id, step_id)): Path<(String, String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadRunStepsRetrieve(&thread_id, &run_id, &step_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream thread run step retrieve",
            );
        }
    }

    Json(
        get_thread_run_step(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
            &run_id,
            &step_id,
        )
        .expect("thread run step"),
    )
    .into_response()
}

async fn responses_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateResponseRequest>,
) -> Response {
    if request.stream.unwrap_or(false) {
        match relay_stateless_stream_request(
            &request_context,
            ProviderRequest::ResponsesStream(&request),
        )
        .await
        {
            Ok(Some(response)) => return upstream_passthrough_response(response),
            Ok(None) => {}
            Err(_) => {
                return bad_gateway_openai_response("failed to relay upstream response stream");
            }
        }

        return local_response_stream_response("resp_1", &request.model);
    }

    match relay_stateless_json_request(&request_context, ProviderRequest::Responses(&request)).await
    {
        Ok(Some(response)) => Json(response).into_response(),
        Ok(None) => Json(
            create_response(
                request_context.tenant_id(),
                request_context.project_id(),
                &request.model,
            )
            .expect("response"),
        )
        .into_response(),
        Err(_) => bad_gateway_openai_response("failed to relay upstream response"),
    }
}

async fn response_input_tokens_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CountResponseInputTokensRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ResponsesInputTokens(&request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream response input tokens");
        }
    }

    Json(
        count_response_input_tokens(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        )
        .expect("response input tokens"),
    )
    .into_response()
}

async fn response_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(response_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ResponsesRetrieve(&response_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream response retrieve");
        }
    }

    Json(
        get_response(
            request_context.tenant_id(),
            request_context.project_id(),
            &response_id,
        )
        .expect("response retrieve"),
    )
    .into_response()
}

async fn response_input_items_list_handler(
    request_context: StatelessGatewayRequest,
    Path(response_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ResponsesInputItemsList(&response_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream response input items");
        }
    }

    Json(
        list_response_input_items(
            request_context.tenant_id(),
            request_context.project_id(),
            &response_id,
        )
        .expect("response input items"),
    )
    .into_response()
}

async fn response_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(response_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ResponsesDelete(&response_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream response delete");
        }
    }

    Json(
        delete_response(
            request_context.tenant_id(),
            request_context.project_id(),
            &response_id,
        )
        .expect("response delete"),
    )
    .into_response()
}

async fn response_cancel_handler(
    request_context: StatelessGatewayRequest,
    Path(response_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ResponsesCancel(&response_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream response cancel");
        }
    }

    Json(
        cancel_response(
            request_context.tenant_id(),
            request_context.project_id(),
            &response_id,
        )
        .expect("response cancel"),
    )
    .into_response()
}

async fn response_compact_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CompactResponseRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ResponsesCompact(&request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream response compact");
        }
    }

    Json(
        compact_response(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        )
        .expect("response compact"),
    )
    .into_response()
}

async fn completions_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateCompletionRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Completions(&request))
        .await
    {
        Ok(Some(response)) => Json(response).into_response(),
        Ok(None) => Json(
            create_completion(
                request_context.tenant_id(),
                request_context.project_id(),
                &request.model,
            )
            .expect("completion"),
        )
        .into_response(),
        Err(_) => bad_gateway_openai_response("failed to relay upstream completion"),
    }
}

async fn embeddings_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateEmbeddingRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Embeddings(&request))
        .await
    {
        Ok(Some(response)) => Json(response).into_response(),
        Ok(None) => Json(
            create_embedding(
                request_context.tenant_id(),
                request_context.project_id(),
                &request.model,
            )
            .expect("embedding"),
        )
        .into_response(),
        Err(_) => bad_gateway_openai_response("failed to relay upstream embedding"),
    }
}

async fn moderations_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateModerationRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Moderations(&request))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream moderation");
        }
    }

    Json(
        create_moderation(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        )
        .expect("moderation"),
    )
    .into_response()
}

async fn image_generations_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateImageRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ImagesGenerations(&request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream image generation");
        }
    }

    Json(
        create_image_generation(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        )
        .expect("image generation"),
    )
    .into_response()
}

async fn image_edits_handler(
    request_context: StatelessGatewayRequest,
    multipart: Multipart,
) -> Response {
    match parse_image_edit_request(multipart).await {
        Ok(request) => {
            match relay_stateless_json_request(
                &request_context,
                ProviderRequest::ImagesEdits(&request),
            )
            .await
            {
                Ok(Some(response)) => return Json(response).into_response(),
                Ok(None) => {}
                Err(_) => {
                    return bad_gateway_openai_response("failed to relay upstream image edit");
                }
            }

            Json(
                create_image_edit(
                    request_context.tenant_id(),
                    request_context.project_id(),
                    &request,
                )
                .expect("image edit"),
            )
            .into_response()
        }
        Err(response) => response,
    }
}

async fn image_variations_handler(
    request_context: StatelessGatewayRequest,
    multipart: Multipart,
) -> Response {
    match parse_image_variation_request(multipart).await {
        Ok(request) => {
            match relay_stateless_json_request(
                &request_context,
                ProviderRequest::ImagesVariations(&request),
            )
            .await
            {
                Ok(Some(response)) => return Json(response).into_response(),
                Ok(None) => {}
                Err(_) => {
                    return bad_gateway_openai_response("failed to relay upstream image variation");
                }
            }

            Json(
                create_image_variation(
                    request_context.tenant_id(),
                    request_context.project_id(),
                    &request,
                )
                .expect("image variation"),
            )
            .into_response()
        }
        Err(response) => response,
    }
}

async fn transcriptions_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateTranscriptionRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::AudioTranscriptions(&request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream transcription");
        }
    }

    Json(
        create_transcription(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        )
        .expect("transcription"),
    )
    .into_response()
}

async fn translations_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateTranslationRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::AudioTranslations(&request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream translation");
        }
    }

    Json(
        create_translation(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        )
        .expect("translation"),
    )
    .into_response()
}

async fn audio_speech_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateSpeechRequest>,
) -> Response {
    match relay_stateless_stream_request(&request_context, ProviderRequest::AudioSpeech(&request))
        .await
    {
        Ok(Some(response)) => return upstream_passthrough_response(response),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream audio speech");
        }
    }

    local_speech_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
}

async fn audio_voices_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::AudioVoicesList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream audio voices list");
        }
    }

    Json(
        list_audio_voices(request_context.tenant_id(), request_context.project_id())
            .expect("audio voices list"),
    )
    .into_response()
}

async fn audio_voice_consents_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateVoiceConsentRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::AudioVoiceConsents(&request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream audio voice consent");
        }
    }

    Json(
        create_audio_voice_consent(
            request_context.tenant_id(),
            request_context.project_id(),
            &request,
        )
        .expect("audio voice consent"),
    )
    .into_response()
}

async fn files_handler(request_context: StatelessGatewayRequest, multipart: Multipart) -> Response {
    match parse_file_request(multipart).await {
        Ok(request) => {
            match relay_stateless_json_request(&request_context, ProviderRequest::Files(&request))
                .await
            {
                Ok(Some(response)) => return Json(response).into_response(),
                Ok(None) => {}
                Err(_) => {
                    return bad_gateway_openai_response("failed to relay upstream file");
                }
            }

            Json(
                create_file(
                    request_context.tenant_id(),
                    request_context.project_id(),
                    &request,
                )
                .expect("file"),
            )
            .into_response()
        }
        Err(response) => response,
    }
}

async fn files_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::FilesList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream files list");
        }
    }

    Json(list_files(request_context.tenant_id(), request_context.project_id()).expect("files list"))
        .into_response()
}

async fn file_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(file_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::FilesRetrieve(&file_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream file retrieve");
        }
    }

    Json(
        get_file(
            request_context.tenant_id(),
            request_context.project_id(),
            &file_id,
        )
        .expect("file retrieve"),
    )
    .into_response()
}

async fn file_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(file_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::FilesDelete(&file_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream file delete");
        }
    }

    Json(
        delete_file(
            request_context.tenant_id(),
            request_context.project_id(),
            &file_id,
        )
        .expect("file delete"),
    )
    .into_response()
}

async fn file_content_handler(
    request_context: StatelessGatewayRequest,
    Path(file_id): Path<String>,
) -> Response {
    match relay_stateless_stream_request(&request_context, ProviderRequest::FilesContent(&file_id))
        .await
    {
        Ok(Some(response)) => return upstream_passthrough_response(response),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream file content");
        }
    }

    local_file_content_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &file_id,
    )
}

async fn containers_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateContainerRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Containers(&request))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream container");
        }
    }

    Json(
        sdkwork_api_app_gateway::create_container(
            request_context.tenant_id(),
            request_context.project_id(),
            &request,
        )
        .expect("container"),
    )
    .into_response()
}

async fn containers_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::ContainersList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream containers list");
        }
    }

    Json(
        sdkwork_api_app_gateway::list_containers(
            request_context.tenant_id(),
            request_context.project_id(),
        )
        .expect("containers list"),
    )
    .into_response()
}

async fn container_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(container_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ContainersRetrieve(&container_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream container retrieve");
        }
    }

    Json(
        sdkwork_api_app_gateway::get_container(
            request_context.tenant_id(),
            request_context.project_id(),
            &container_id,
        )
        .expect("container retrieve"),
    )
    .into_response()
}

async fn container_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(container_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ContainersDelete(&container_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream container delete");
        }
    }

    Json(
        sdkwork_api_app_gateway::delete_container(
            request_context.tenant_id(),
            request_context.project_id(),
            &container_id,
        )
        .expect("container delete"),
    )
    .into_response()
}

async fn container_files_handler(
    request_context: StatelessGatewayRequest,
    Path(container_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateContainerFileRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ContainerFiles(&container_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream container file");
        }
    }

    Json(
        sdkwork_api_app_gateway::create_container_file(
            request_context.tenant_id(),
            request_context.project_id(),
            &container_id,
            &request,
        )
        .expect("container file"),
    )
    .into_response()
}

async fn container_files_list_handler(
    request_context: StatelessGatewayRequest,
    Path(container_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ContainerFilesList(&container_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream container files list");
        }
    }

    Json(
        sdkwork_api_app_gateway::list_container_files(
            request_context.tenant_id(),
            request_context.project_id(),
            &container_id,
        )
        .expect("container files list"),
    )
    .into_response()
}

async fn container_file_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((container_id, file_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ContainerFilesRetrieve(&container_id, &file_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream container file retrieve");
        }
    }

    Json(
        sdkwork_api_app_gateway::get_container_file(
            request_context.tenant_id(),
            request_context.project_id(),
            &container_id,
            &file_id,
        )
        .expect("container file retrieve"),
    )
    .into_response()
}

async fn container_file_delete_handler(
    request_context: StatelessGatewayRequest,
    Path((container_id, file_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ContainerFilesDelete(&container_id, &file_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream container file delete");
        }
    }

    Json(
        sdkwork_api_app_gateway::delete_container_file(
            request_context.tenant_id(),
            request_context.project_id(),
            &container_id,
            &file_id,
        )
        .expect("container file delete"),
    )
    .into_response()
}

async fn container_file_content_handler(
    request_context: StatelessGatewayRequest,
    Path((container_id, file_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_stream_request(
        &request_context,
        ProviderRequest::ContainerFilesContent(&container_id, &file_id),
    )
    .await
    {
        Ok(Some(response)) => return upstream_passthrough_response(response),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream container file content");
        }
    }

    local_container_file_content_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
        &file_id,
    )
}

async fn music_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateMusicRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Music(&request)).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream music create");
        }
    }

    Json(
        create_music(
            request_context.tenant_id(),
            request_context.project_id(),
            &request,
        )
        .expect("music create"),
    )
    .into_response()
}

async fn music_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::MusicList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream music list");
        }
    }

    Json(list_music(request_context.tenant_id(), request_context.project_id()).expect("music list"))
        .into_response()
}

async fn music_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(music_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::MusicRetrieve(&music_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream music retrieve");
        }
    }

    Json(
        get_music(
            request_context.tenant_id(),
            request_context.project_id(),
            &music_id,
        )
        .expect("music retrieve"),
    )
    .into_response()
}

async fn music_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(music_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::MusicDelete(&music_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream music delete");
        }
    }

    Json(
        delete_music(
            request_context.tenant_id(),
            request_context.project_id(),
            &music_id,
        )
        .expect("music delete"),
    )
    .into_response()
}

async fn music_content_handler(
    request_context: StatelessGatewayRequest,
    Path(music_id): Path<String>,
) -> Response {
    match relay_stateless_stream_request(&request_context, ProviderRequest::MusicContent(&music_id))
        .await
    {
        Ok(Some(response)) => return upstream_passthrough_response(response),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream music content");
        }
    }

    local_music_content_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &music_id,
    )
}

async fn music_lyrics_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateMusicLyricsRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::MusicLyrics(&request))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream music lyrics");
        }
    }

    Json(
        create_music_lyrics(
            request_context.tenant_id(),
            request_context.project_id(),
            &request,
        )
        .expect("music lyrics"),
    )
    .into_response()
}

async fn videos_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateVideoRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Videos(&request)).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video");
        }
    }

    Json(
        create_video(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
            &request.prompt,
        )
        .expect("video"),
    )
    .into_response()
}

async fn videos_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::VideosList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream videos list");
        }
    }

    Json(
        list_videos(request_context.tenant_id(), request_context.project_id())
            .expect("videos list"),
    )
    .into_response()
}

async fn video_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(video_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::VideosRetrieve(&video_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video retrieve");
        }
    }

    Json(
        get_video(
            request_context.tenant_id(),
            request_context.project_id(),
            &video_id,
        )
        .expect("video retrieve"),
    )
    .into_response()
}

async fn video_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(video_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::VideosDelete(&video_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video delete");
        }
    }

    Json(
        delete_video(
            request_context.tenant_id(),
            request_context.project_id(),
            &video_id,
        )
        .expect("video delete"),
    )
    .into_response()
}

async fn video_content_handler(
    request_context: StatelessGatewayRequest,
    Path(video_id): Path<String>,
) -> Response {
    match relay_stateless_stream_request(
        &request_context,
        ProviderRequest::VideosContent(&video_id),
    )
    .await
    {
        Ok(Some(response)) => return upstream_passthrough_response(response),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video content");
        }
    }

    local_video_content_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
    )
}

async fn video_remix_handler(
    request_context: StatelessGatewayRequest,
    Path(video_id): Path<String>,
    ExtractJson(request): ExtractJson<RemixVideoRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VideosRemix(&video_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video remix");
        }
    }

    Json(
        remix_video(
            request_context.tenant_id(),
            request_context.project_id(),
            &video_id,
            &request.prompt,
        )
        .expect("video remix"),
    )
    .into_response()
}

async fn video_characters_list_handler(
    request_context: StatelessGatewayRequest,
    Path(video_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VideoCharactersList(&video_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video characters list");
        }
    }

    Json(
        list_video_characters(
            request_context.tenant_id(),
            request_context.project_id(),
            &video_id,
        )
        .expect("video characters list"),
    )
    .into_response()
}

async fn video_character_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((video_id, character_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VideoCharactersRetrieve(&video_id, &character_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream video character retrieve",
            );
        }
    }

    Json(
        get_video_character(
            request_context.tenant_id(),
            request_context.project_id(),
            &video_id,
            &character_id,
        )
        .expect("video character retrieve"),
    )
    .into_response()
}

async fn video_character_update_handler(
    request_context: StatelessGatewayRequest,
    Path((video_id, character_id)): Path<(String, String)>,
    ExtractJson(request): ExtractJson<UpdateVideoCharacterRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VideoCharactersUpdate(&video_id, &character_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video character update");
        }
    }

    Json(
        update_video_character(
            request_context.tenant_id(),
            request_context.project_id(),
            &video_id,
            &character_id,
            &request,
        )
        .expect("video character update"),
    )
    .into_response()
}

async fn video_extend_handler(
    request_context: StatelessGatewayRequest,
    Path(video_id): Path<String>,
    ExtractJson(request): ExtractJson<ExtendVideoRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VideosExtend(&video_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video extend");
        }
    }

    Json(
        extend_video(
            request_context.tenant_id(),
            request_context.project_id(),
            &video_id,
            &request.prompt,
        )
        .expect("video extend"),
    )
    .into_response()
}

async fn video_character_create_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateVideoCharacterRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VideoCharactersCreate(&request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video character create");
        }
    }

    Json(
        sdkwork_api_app_gateway::create_video_character(
            request_context.tenant_id(),
            request_context.project_id(),
            &request,
        )
        .expect("video character create"),
    )
    .into_response()
}

async fn video_character_retrieve_canonical_handler(
    request_context: StatelessGatewayRequest,
    Path(character_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VideoCharactersCanonicalRetrieve(&character_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream video character retrieve",
            );
        }
    }

    Json(
        sdkwork_api_app_gateway::get_video_character_canonical(
            request_context.tenant_id(),
            request_context.project_id(),
            &character_id,
        )
        .expect("video character canonical retrieve"),
    )
    .into_response()
}

async fn video_edits_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<EditVideoRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::VideosEdits(&request))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video edits");
        }
    }

    Json(
        sdkwork_api_app_gateway::edit_video(
            request_context.tenant_id(),
            request_context.project_id(),
            &request,
        )
        .expect("video edits"),
    )
    .into_response()
}

async fn video_extensions_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<ExtendVideoRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VideosExtensions(&request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video extensions");
        }
    }

    Json(
        sdkwork_api_app_gateway::extensions_video(
            request_context.tenant_id(),
            request_context.project_id(),
            &request,
        )
        .expect("video extensions"),
    )
    .into_response()
}

async fn uploads_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateUploadRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Uploads(&request)).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream upload");
        }
    }

    Json(
        create_upload(
            request_context.tenant_id(),
            request_context.project_id(),
            &request,
        )
        .expect("upload"),
    )
    .into_response()
}

async fn upload_parts_handler(
    request_context: StatelessGatewayRequest,
    Path(upload_id): Path<String>,
    multipart: Multipart,
) -> Response {
    match parse_upload_part_request(upload_id, multipart).await {
        Ok(request) => {
            match relay_stateless_json_request(
                &request_context,
                ProviderRequest::UploadParts(&request),
            )
            .await
            {
                Ok(Some(response)) => return Json(response).into_response(),
                Ok(None) => {}
                Err(_) => {
                    return bad_gateway_openai_response("failed to relay upstream upload part");
                }
            }

            Json(
                create_upload_part(
                    request_context.tenant_id(),
                    request_context.project_id(),
                    &request,
                )
                .expect("upload part"),
            )
            .into_response()
        }
        Err(response) => response,
    }
}

async fn upload_complete_handler(
    request_context: StatelessGatewayRequest,
    Path(upload_id): Path<String>,
    ExtractJson(mut request): ExtractJson<CompleteUploadRequest>,
) -> Response {
    request.upload_id = upload_id;
    match relay_stateless_json_request(&request_context, ProviderRequest::UploadComplete(&request))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream upload complete");
        }
    }

    Json(
        complete_upload(
            request_context.tenant_id(),
            request_context.project_id(),
            &request,
        )
        .expect("upload complete"),
    )
    .into_response()
}

async fn upload_cancel_handler(
    request_context: StatelessGatewayRequest,
    Path(upload_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::UploadCancel(&upload_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream upload cancel");
        }
    }

    Json(
        cancel_upload(
            request_context.tenant_id(),
            request_context.project_id(),
            &upload_id,
        )
        .expect("upload cancel"),
    )
    .into_response()
}

async fn fine_tuning_jobs_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateFineTuningJobRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::FineTuningJobs(&request))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream fine tuning job");
        }
    }

    Json(
        create_fine_tuning_job(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        )
        .expect("fine tuning"),
    )
    .into_response()
}

async fn fine_tuning_jobs_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::FineTuningJobsList).await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream fine tuning jobs list");
        }
    }

    Json(
        list_fine_tuning_jobs(request_context.tenant_id(), request_context.project_id())
            .expect("fine tuning list"),
    )
    .into_response()
}

async fn fine_tuning_job_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::FineTuningJobsRetrieve(&fine_tuning_job_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream fine tuning job retrieve",
            );
        }
    }

    Json(
        get_fine_tuning_job(
            request_context.tenant_id(),
            request_context.project_id(),
            &fine_tuning_job_id,
        )
        .expect("fine tuning retrieve"),
    )
    .into_response()
}

async fn fine_tuning_job_cancel_handler(
    request_context: StatelessGatewayRequest,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::FineTuningJobsCancel(&fine_tuning_job_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream fine tuning job cancel");
        }
    }

    Json(
        cancel_fine_tuning_job(
            request_context.tenant_id(),
            request_context.project_id(),
            &fine_tuning_job_id,
        )
        .expect("fine tuning cancel"),
    )
    .into_response()
}

async fn fine_tuning_job_events_handler(
    request_context: StatelessGatewayRequest,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::FineTuningJobsEvents(&fine_tuning_job_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream fine tuning job events");
        }
    }

    Json(
        list_fine_tuning_job_events(
            request_context.tenant_id(),
            request_context.project_id(),
            &fine_tuning_job_id,
        )
        .expect("fine tuning job events"),
    )
    .into_response()
}

async fn fine_tuning_job_checkpoints_handler(
    request_context: StatelessGatewayRequest,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::FineTuningJobsCheckpoints(&fine_tuning_job_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream fine tuning job checkpoints",
            );
        }
    }

    Json(
        list_fine_tuning_job_checkpoints(
            request_context.tenant_id(),
            request_context.project_id(),
            &fine_tuning_job_id,
        )
        .expect("fine tuning job checkpoints"),
    )
    .into_response()
}

async fn fine_tuning_job_pause_handler(
    request_context: StatelessGatewayRequest,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::FineTuningJobsPause(&fine_tuning_job_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream fine tuning job pause");
        }
    }

    Json(
        sdkwork_api_app_gateway::pause_fine_tuning_job(
            request_context.tenant_id(),
            request_context.project_id(),
            &fine_tuning_job_id,
        )
        .expect("fine tuning pause"),
    )
    .into_response()
}

async fn fine_tuning_job_resume_handler(
    request_context: StatelessGatewayRequest,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::FineTuningJobsResume(&fine_tuning_job_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream fine tuning job resume");
        }
    }

    Json(
        sdkwork_api_app_gateway::resume_fine_tuning_job(
            request_context.tenant_id(),
            request_context.project_id(),
            &fine_tuning_job_id,
        )
        .expect("fine tuning resume"),
    )
    .into_response()
}

async fn fine_tuning_checkpoint_permissions_handler(
    request_context: StatelessGatewayRequest,
    Path(fine_tuned_model_checkpoint): Path<String>,
    ExtractJson(request): ExtractJson<CreateFineTuningCheckpointPermissionsRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::FineTuningCheckpointPermissions(&fine_tuned_model_checkpoint, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream fine tuning checkpoint permissions create",
            );
        }
    }

    Json(
        sdkwork_api_app_gateway::create_fine_tuning_checkpoint_permissions(
            request_context.tenant_id(),
            request_context.project_id(),
            &request,
        )
        .expect("fine tuning checkpoint permissions create"),
    )
    .into_response()
}

async fn fine_tuning_checkpoint_permissions_list_handler(
    request_context: StatelessGatewayRequest,
    Path(fine_tuned_model_checkpoint): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::FineTuningCheckpointPermissionsList(&fine_tuned_model_checkpoint),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream fine tuning checkpoint permissions list",
            );
        }
    }

    Json(
        sdkwork_api_app_gateway::list_fine_tuning_checkpoint_permissions(
            request_context.tenant_id(),
            request_context.project_id(),
            &fine_tuned_model_checkpoint,
        )
        .expect("fine tuning checkpoint permissions list"),
    )
    .into_response()
}

async fn fine_tuning_checkpoint_permission_delete_handler(
    request_context: StatelessGatewayRequest,
    Path((fine_tuned_model_checkpoint, permission_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::FineTuningCheckpointPermissionsDelete(
            &fine_tuned_model_checkpoint,
            &permission_id,
        ),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream fine tuning checkpoint permission delete",
            );
        }
    }

    Json(
        sdkwork_api_app_gateway::delete_fine_tuning_checkpoint_permission(
            request_context.tenant_id(),
            request_context.project_id(),
            &permission_id,
        )
        .expect("fine tuning checkpoint permission delete"),
    )
    .into_response()
}

async fn assistants_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateAssistantRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Assistants(&request))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream assistant");
        }
    }

    Json(
        create_assistant(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.name,
            &request.model,
        )
        .expect("assistant"),
    )
    .into_response()
}

async fn assistants_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::AssistantsList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream assistants list");
        }
    }

    Json(
        list_assistants(request_context.tenant_id(), request_context.project_id())
            .expect("assistants list"),
    )
    .into_response()
}

async fn assistant_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(assistant_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::AssistantsRetrieve(&assistant_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream assistant retrieve");
        }
    }

    Json(
        get_assistant(
            request_context.tenant_id(),
            request_context.project_id(),
            &assistant_id,
        )
        .expect("assistant retrieve"),
    )
    .into_response()
}

async fn assistant_update_handler(
    request_context: StatelessGatewayRequest,
    Path(assistant_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateAssistantRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::AssistantsUpdate(&assistant_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream assistant update");
        }
    }

    Json(
        update_assistant(
            request_context.tenant_id(),
            request_context.project_id(),
            &assistant_id,
            request.name.as_deref().unwrap_or("assistant"),
        )
        .expect("assistant update"),
    )
    .into_response()
}

async fn assistant_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(assistant_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::AssistantsDelete(&assistant_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream assistant delete");
        }
    }

    Json(
        delete_assistant(
            request_context.tenant_id(),
            request_context.project_id(),
            &assistant_id,
        )
        .expect("assistant delete"),
    )
    .into_response()
}

async fn webhooks_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateWebhookRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Webhooks(&request)).await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream webhook");
        }
    }

    Json(
        create_webhook(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.url,
            &request.events,
        )
        .expect("webhook"),
    )
    .into_response()
}

async fn webhooks_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::WebhooksList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream webhooks list");
        }
    }

    Json(
        list_webhooks(request_context.tenant_id(), request_context.project_id())
            .expect("webhooks list"),
    )
    .into_response()
}

async fn webhook_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(webhook_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::WebhooksRetrieve(&webhook_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream webhook retrieve");
        }
    }

    Json(
        get_webhook(
            request_context.tenant_id(),
            request_context.project_id(),
            &webhook_id,
        )
        .expect("webhook retrieve"),
    )
    .into_response()
}

async fn webhook_update_handler(
    request_context: StatelessGatewayRequest,
    Path(webhook_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateWebhookRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::WebhooksUpdate(&webhook_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream webhook update");
        }
    }

    Json(
        update_webhook(
            request_context.tenant_id(),
            request_context.project_id(),
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

async fn webhook_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(webhook_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::WebhooksDelete(&webhook_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream webhook delete");
        }
    }

    Json(
        delete_webhook(
            request_context.tenant_id(),
            request_context.project_id(),
            &webhook_id,
        )
        .expect("webhook delete"),
    )
    .into_response()
}

async fn realtime_sessions_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateRealtimeSessionRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::RealtimeSessions(&request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream realtime session");
        }
    }

    Json(
        create_realtime_session(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        )
        .expect("realtime session"),
    )
    .into_response()
}

async fn evals_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateEvalRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Evals(&request)).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval");
        }
    }

    Json(
        create_eval(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.name,
        )
        .expect("eval"),
    )
    .into_response()
}

async fn evals_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::EvalsList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream evals list");
        }
    }

    Json(list_evals(request_context.tenant_id(), request_context.project_id()).expect("eval list"))
        .into_response()
}

async fn eval_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(eval_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::EvalsRetrieve(&eval_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval retrieve");
        }
    }

    Json(
        get_eval(
            request_context.tenant_id(),
            request_context.project_id(),
            &eval_id,
        )
        .expect("eval retrieve"),
    )
    .into_response()
}

async fn eval_update_handler(
    request_context: StatelessGatewayRequest,
    Path(eval_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateEvalRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::EvalsUpdate(&eval_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval update");
        }
    }

    Json(
        update_eval(
            request_context.tenant_id(),
            request_context.project_id(),
            &eval_id,
            &request,
        )
        .expect("eval update"),
    )
    .into_response()
}

async fn eval_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(eval_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::EvalsDelete(&eval_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval delete");
        }
    }

    Json(
        delete_eval(
            request_context.tenant_id(),
            request_context.project_id(),
            &eval_id,
        )
        .expect("eval delete"),
    )
    .into_response()
}

async fn eval_runs_list_handler(
    request_context: StatelessGatewayRequest,
    Path(eval_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::EvalRunsList(&eval_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval runs list");
        }
    }

    Json(
        list_eval_runs(
            request_context.tenant_id(),
            request_context.project_id(),
            &eval_id,
        )
        .expect("eval runs list"),
    )
    .into_response()
}

async fn eval_runs_handler(
    request_context: StatelessGatewayRequest,
    Path(eval_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateEvalRunRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::EvalRuns(&eval_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval run create");
        }
    }

    Json(
        create_eval_run(
            request_context.tenant_id(),
            request_context.project_id(),
            &eval_id,
            &request,
        )
        .expect("eval run create"),
    )
    .into_response()
}

async fn eval_run_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((eval_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::EvalRunsRetrieve(&eval_id, &run_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval run retrieve");
        }
    }

    Json(
        get_eval_run(
            request_context.tenant_id(),
            request_context.project_id(),
            &eval_id,
            &run_id,
        )
        .expect("eval run retrieve"),
    )
    .into_response()
}

async fn eval_run_delete_handler(
    request_context: StatelessGatewayRequest,
    Path((eval_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::EvalRunsDelete(&eval_id, &run_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval run delete");
        }
    }

    Json(
        sdkwork_api_app_gateway::delete_eval_run(
            request_context.tenant_id(),
            request_context.project_id(),
            &eval_id,
            &run_id,
        )
        .expect("eval run delete"),
    )
    .into_response()
}

async fn eval_run_cancel_handler(
    request_context: StatelessGatewayRequest,
    Path((eval_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::EvalRunsCancel(&eval_id, &run_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval run cancel");
        }
    }

    Json(
        sdkwork_api_app_gateway::cancel_eval_run(
            request_context.tenant_id(),
            request_context.project_id(),
            &eval_id,
            &run_id,
        )
        .expect("eval run cancel"),
    )
    .into_response()
}

async fn eval_run_output_items_list_handler(
    request_context: StatelessGatewayRequest,
    Path((eval_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::EvalRunOutputItemsList(&eval_id, &run_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream eval run output items list",
            );
        }
    }

    Json(
        sdkwork_api_app_gateway::list_eval_run_output_items(
            request_context.tenant_id(),
            request_context.project_id(),
            &eval_id,
            &run_id,
        )
        .expect("eval run output items list"),
    )
    .into_response()
}

async fn eval_run_output_item_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((eval_id, run_id, output_item_id)): Path<(String, String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::EvalRunOutputItemsRetrieve(&eval_id, &run_id, &output_item_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream eval run output item retrieve",
            );
        }
    }

    Json(
        sdkwork_api_app_gateway::get_eval_run_output_item(
            request_context.tenant_id(),
            request_context.project_id(),
            &eval_id,
            &run_id,
            &output_item_id,
        )
        .expect("eval run output item retrieve"),
    )
    .into_response()
}

async fn batches_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateBatchRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Batches(&request)).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream batch");
        }
    }

    Json(
        create_batch(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.endpoint,
            &request.input_file_id,
        )
        .expect("batch"),
    )
    .into_response()
}

async fn batches_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::BatchesList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream batches list");
        }
    }

    Json(
        list_batches(request_context.tenant_id(), request_context.project_id())
            .expect("batches list"),
    )
    .into_response()
}

async fn batch_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(batch_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::BatchesRetrieve(&batch_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream batch retrieve");
        }
    }

    Json(
        get_batch(
            request_context.tenant_id(),
            request_context.project_id(),
            &batch_id,
        )
        .expect("batch retrieve"),
    )
    .into_response()
}

async fn batch_cancel_handler(
    request_context: StatelessGatewayRequest,
    Path(batch_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::BatchesCancel(&batch_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream batch cancel");
        }
    }

    Json(
        cancel_batch(
            request_context.tenant_id(),
            request_context.project_id(),
            &batch_id,
        )
        .expect("batch cancel"),
    )
    .into_response()
}

async fn vector_stores_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::VectorStoresList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector stores list");
        }
    }

    Json(
        list_vector_stores(request_context.tenant_id(), request_context.project_id())
            .expect("vector stores list"),
    )
    .into_response()
}

async fn vector_stores_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateVectorStoreRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::VectorStores(&request))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store");
        }
    }

    Json(
        create_vector_store(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.name,
        )
        .expect("vector store"),
    )
    .into_response()
}

async fn vector_store_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(vector_store_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoresRetrieve(&vector_store_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store retrieve");
        }
    }

    Json(
        get_vector_store(
            request_context.tenant_id(),
            request_context.project_id(),
            &vector_store_id,
        )
        .expect("vector store retrieve"),
    )
    .into_response()
}

async fn vector_store_update_handler(
    request_context: StatelessGatewayRequest,
    Path(vector_store_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateVectorStoreRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoresUpdate(&vector_store_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store update");
        }
    }

    Json(
        update_vector_store(
            request_context.tenant_id(),
            request_context.project_id(),
            &vector_store_id,
            request.name.as_deref().unwrap_or("vector-store"),
        )
        .expect("vector store update"),
    )
    .into_response()
}

async fn vector_store_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(vector_store_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoresDelete(&vector_store_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store delete");
        }
    }

    Json(
        delete_vector_store(
            request_context.tenant_id(),
            request_context.project_id(),
            &vector_store_id,
        )
        .expect("vector store delete"),
    )
    .into_response()
}

async fn vector_store_search_handler(
    request_context: StatelessGatewayRequest,
    Path(vector_store_id): Path<String>,
    ExtractJson(request): ExtractJson<SearchVectorStoreRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoresSearch(&vector_store_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store search");
        }
    }

    Json(
        search_vector_store(
            request_context.tenant_id(),
            request_context.project_id(),
            &vector_store_id,
            &request.query,
        )
        .expect("vector store search"),
    )
    .into_response()
}

async fn vector_store_files_handler(
    request_context: StatelessGatewayRequest,
    Path(vector_store_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateVectorStoreFileRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoreFiles(&vector_store_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store file");
        }
    }

    Json(
        create_vector_store_file(
            request_context.tenant_id(),
            request_context.project_id(),
            &vector_store_id,
            &request.file_id,
        )
        .expect("vector store file"),
    )
    .into_response()
}

async fn vector_store_files_list_handler(
    request_context: StatelessGatewayRequest,
    Path(vector_store_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoreFilesList(&vector_store_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store files list");
        }
    }

    Json(
        list_vector_store_files(
            request_context.tenant_id(),
            request_context.project_id(),
            &vector_store_id,
        )
        .expect("vector store files list"),
    )
    .into_response()
}

async fn vector_store_file_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((vector_store_id, file_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoreFilesRetrieve(&vector_store_id, &file_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream vector store file retrieve",
            );
        }
    }

    Json(
        get_vector_store_file(
            request_context.tenant_id(),
            request_context.project_id(),
            &vector_store_id,
            &file_id,
        )
        .expect("vector store file retrieve"),
    )
    .into_response()
}

async fn vector_store_file_delete_handler(
    request_context: StatelessGatewayRequest,
    Path((vector_store_id, file_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoreFilesDelete(&vector_store_id, &file_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream vector store file delete",
            );
        }
    }

    Json(
        delete_vector_store_file(
            request_context.tenant_id(),
            request_context.project_id(),
            &vector_store_id,
            &file_id,
        )
        .expect("vector store file delete"),
    )
    .into_response()
}

async fn vector_store_file_batches_handler(
    request_context: StatelessGatewayRequest,
    Path(vector_store_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateVectorStoreFileBatchRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoreFileBatches(&vector_store_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store file batch");
        }
    }

    Json(
        create_vector_store_file_batch(
            request_context.tenant_id(),
            request_context.project_id(),
            &vector_store_id,
            &request.file_ids,
        )
        .expect("vector store file batch"),
    )
    .into_response()
}

async fn vector_store_file_batch_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((vector_store_id, batch_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoreFileBatchesRetrieve(&vector_store_id, &batch_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream vector store file batch retrieve",
            );
        }
    }

    Json(
        get_vector_store_file_batch(
            request_context.tenant_id(),
            request_context.project_id(),
            &vector_store_id,
            &batch_id,
        )
        .expect("vector store file batch retrieve"),
    )
    .into_response()
}

async fn vector_store_file_batch_cancel_handler(
    request_context: StatelessGatewayRequest,
    Path((vector_store_id, batch_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoreFileBatchesCancel(&vector_store_id, &batch_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream vector store file batch cancel",
            );
        }
    }

    Json(
        cancel_vector_store_file_batch(
            request_context.tenant_id(),
            request_context.project_id(),
            &vector_store_id,
            &batch_id,
        )
        .expect("vector store file batch cancel"),
    )
    .into_response()
}

async fn vector_store_file_batch_files_handler(
    request_context: StatelessGatewayRequest,
    Path((vector_store_id, batch_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoreFileBatchesListFiles(&vector_store_id, &batch_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream vector store file batch files",
            );
        }
    }

    Json(
        list_vector_store_file_batch_files(
            request_context.tenant_id(),
            request_context.project_id(),
            &vector_store_id,
            &batch_id,
        )
        .expect("vector store file batch files"),
    )
    .into_response()
}

async fn chat_completions_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Extension(request_id): Extension<RequestId>,
    ExtractJson(request): ExtractJson<CreateChatCompletionRequest>,
) -> Response {
    let canonical_hold = match admit_canonical_chat_request(
        state.store.as_ref(),
        &request_context,
        &request_id,
        &request,
    )
    .await
    {
        Ok(CanonicalChatAdmission::Held(hold)) => Some(hold),
        Ok(CanonicalChatAdmission::InsufficientBalance {
            account_id,
            shortfall_quantity,
        }) => {
            return account_balance_insufficient_response(account_id, shortfall_quantity);
        }
        Ok(CanonicalChatAdmission::NotApplicable) => {
            match enforce_project_quota(
                state.store.as_ref(),
                request_context.project_id(),
                CHAT_COMPLETION_HOLD_UNITS,
            )
            .await
            {
                Ok(Some(response)) => return response,
                Ok(None) => None,
                Err(_) => {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to evaluate quota",
                    )
                        .into_response();
                }
            }
        }
        Err(_) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to evaluate canonical account admission",
            )
                .into_response();
        }
    };

    if request.stream.unwrap_or(false) {
        let settlement_context = UsagePricedStreamSettlementContext {
            store: state.store.clone(),
            tenant_id: request_context.tenant_id().to_owned(),
            project_id: request_context.project_id().to_owned(),
            model: request.model.clone(),
            canonical_hold: canonical_hold.clone(),
        };
        match relay_chat_completion_stream_from_store(
            state.store.as_ref(),
            &state.secret_manager,
            request_context.tenant_id(),
            request_context.project_id(),
            &request,
        )
        .await
        {
            Ok(Some(response)) => {
                return chat_completions_stream_response_with_settlement(
                    response,
                    settlement_context,
                );
            }
            Ok(None) => {
                return chat_completions_stream_response_with_settlement(
                    local_chat_completion_stream_output(&request.model),
                    settlement_context,
                );
            }
            Err(error) => {
                if let Some(hold) = canonical_hold.as_ref() {
                    if settle_canonical_chat_request_release(state.store.as_ref(), hold)
                        .await
                        .is_err()
                    {
                        return (
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            "failed to release canonical account hold",
                        )
                            .into_response();
                    }
                }

                return openai_response_for_relay_error(
                    &error,
                    "failed to relay upstream chat completion stream",
                );
            }
        }
    } else {
        match relay_chat_completion_from_store(
            state.store.as_ref(),
            &state.secret_manager,
            request_context.tenant_id(),
            request_context.project_id(),
            &request,
        )
        .await
        {
            Ok(Some(response)) => {
                let token_usage = extract_token_usage_metrics(&response);
                let canonical_settlement =
                    canonical_usage_priced_settlement_amounts(canonical_hold.as_ref(), token_usage);
                let usage_result =
                    record_gateway_usage_for_project_with_route_key_and_tokens_and_reference(
                        state.store.as_ref(),
                        request_context.tenant_id(),
                        request_context.project_id(),
                        "chat_completion",
                        &request.model,
                        &request.model,
                        canonical_settlement
                            .as_ref()
                            .map(|settlement| settlement.usage_units)
                            .unwrap_or(CHAT_COMPLETION_HOLD_UNITS),
                        canonical_settlement
                            .as_ref()
                            .map(|settlement| settlement.customer_charge)
                            .unwrap_or(CHAT_COMPLETION_RETAIL_CHARGE),
                        token_usage,
                        response_usage_id_or_single_data_item_id(&response),
                    )
                    .await;
                if usage_result.is_err() {
                    if let Some(hold) = canonical_hold.as_ref() {
                        let _ =
                            settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
                    }
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to record usage",
                    )
                        .into_response();
                }
                if let (Some(hold), Some(settlement)) =
                    (canonical_hold.as_ref(), canonical_settlement.as_ref())
                {
                    if settle_canonical_chat_request_capture(state.store.as_ref(), hold, settlement)
                        .await
                        .is_err()
                    {
                        return (
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            "failed to capture canonical account hold",
                        )
                            .into_response();
                    }
                }

                return Json(response).into_response();
            }
            Ok(None) => {}
            Err(error) => {
                if let Some(hold) = canonical_hold.as_ref() {
                    if settle_canonical_chat_request_release(state.store.as_ref(), hold)
                        .await
                        .is_err()
                    {
                        return (
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            "failed to release canonical account hold",
                        )
                            .into_response();
                    }
                }

                return openai_response_for_relay_error(
                    &error,
                    "failed to relay upstream chat completion",
                );
            }
        }
    }

    let canonical_settlement =
        canonical_usage_priced_settlement_amounts(canonical_hold.as_ref(), None);

    let usage_result = record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "chat_completion",
        &request.model,
        canonical_settlement
            .as_ref()
            .map(|settlement| settlement.usage_units)
            .unwrap_or(CHAT_COMPLETION_HOLD_UNITS),
        canonical_settlement
            .as_ref()
            .map(|settlement| settlement.customer_charge)
            .unwrap_or(CHAT_COMPLETION_RETAIL_CHARGE),
    )
    .await;
    if usage_result.is_err() {
        if let Some(hold) = canonical_hold.as_ref() {
            let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
        }
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    if let (Some(hold), Some(settlement)) = (canonical_hold.as_ref(), canonical_settlement.as_ref())
    {
        if settle_canonical_chat_request_capture(state.store.as_ref(), hold, settlement)
            .await
            .is_err()
        {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to capture canonical account hold",
            )
                .into_response();
        }
    }

    if request.stream.unwrap_or(false) {
        local_chat_completion_stream_response(&request.model)
    } else {
        Json(
            create_chat_completion(
                request_context.tenant_id(),
                request_context.project_id(),
                &request.model,
            )
            .expect("chat completion"),
        )
        .into_response()
    }
}

async fn anthropic_messages_handler(
    request_context: StatelessGatewayRequest,
    headers: HeaderMap,
    ExtractJson(payload): ExtractJson<Value>,
) -> Response {
    let request = match anthropic_request_to_chat_completion(&payload) {
        Ok(request) => request,
        Err(error) => return anthropic_invalid_request_response(error.to_string()),
    };
    let options = anthropic_request_options(&headers);

    if request.stream.unwrap_or(false) {
        match relay_stateless_stream_request_with_options(
            &request_context,
            ProviderRequest::ChatCompletionsStream(&request),
            &options,
        )
        .await
        {
            Ok(Some(response)) => {
                return upstream_passthrough_response(anthropic_stream_from_openai(response));
            }
            Ok(None) => return local_anthropic_stream_response(&request.model),
            Err(_) => {
                return anthropic_bad_gateway_response(
                    "failed to relay upstream anthropic message stream",
                );
            }
        }
    }

    match relay_stateless_json_request_with_options(
        &request_context,
        ProviderRequest::ChatCompletions(&request),
        &options,
    )
    .await
    {
        Ok(Some(response)) => Json(openai_chat_response_to_anthropic(&response)).into_response(),
        Ok(None) => Json(openai_chat_response_to_anthropic(
            &serde_json::to_value(
                create_chat_completion(
                    request_context.tenant_id(),
                    request_context.project_id(),
                    &request.model,
                )
                .expect("chat completion"),
            )
            .expect("chat completion value"),
        ))
        .into_response(),
        Err(_) => anthropic_bad_gateway_response("failed to relay upstream anthropic message"),
    }
}

async fn anthropic_count_tokens_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(payload): ExtractJson<Value>,
) -> Response {
    let request = match anthropic_count_tokens_request(&payload) {
        Ok(request) => request,
        Err(error) => return anthropic_invalid_request_response(error.to_string()),
    };

    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ResponsesInputTokens(&request),
    )
    .await
    {
        Ok(Some(response)) => Json(openai_count_tokens_to_anthropic(&response)).into_response(),
        Ok(None) => Json(openai_count_tokens_to_anthropic(
            &serde_json::to_value(
                count_response_input_tokens(
                    request_context.tenant_id(),
                    request_context.project_id(),
                    &request.model,
                )
                .expect("response input tokens"),
            )
            .expect("response input token value"),
        ))
        .into_response(),
        Err(_) => anthropic_bad_gateway_response(
            "failed to relay upstream anthropic count tokens request",
        ),
    }
}

async fn anthropic_messages_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Extension(request_id): Extension<RequestId>,
    headers: HeaderMap,
    ExtractJson(payload): ExtractJson<Value>,
) -> Response {
    let request = match anthropic_request_to_chat_completion(&payload) {
        Ok(request) => request,
        Err(error) => return anthropic_invalid_request_response(error.to_string()),
    };
    let options = anthropic_request_options(&headers);

    let canonical_hold = match admit_canonical_chat_request(
        state.store.as_ref(),
        &request_context,
        &request_id,
        &request,
    )
    .await
    {
        Ok(CanonicalChatAdmission::Held(hold)) => Some(hold),
        Ok(CanonicalChatAdmission::InsufficientBalance {
            account_id,
            shortfall_quantity,
        }) => {
            return account_balance_insufficient_response(account_id, shortfall_quantity);
        }
        Ok(CanonicalChatAdmission::NotApplicable) => {
            match enforce_project_quota(
                state.store.as_ref(),
                request_context.project_id(),
                CHAT_COMPLETION_HOLD_UNITS,
            )
            .await
            {
                Ok(Some(response)) => return response,
                Ok(None) => None,
                Err(_) => {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to evaluate quota",
                    )
                        .into_response();
                }
            }
        }
        Err(_) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to evaluate canonical account admission",
            )
                .into_response();
        }
    };

    if request.stream.unwrap_or(false) {
        let settlement_context = UsagePricedStreamSettlementContext {
            store: state.store.clone(),
            tenant_id: request_context.tenant_id().to_owned(),
            project_id: request_context.project_id().to_owned(),
            model: request.model.clone(),
            canonical_hold: canonical_hold.clone(),
        };
        match relay_chat_completion_stream_from_store_with_options(
            state.store.as_ref(),
            &state.secret_manager,
            request_context.tenant_id(),
            request_context.project_id(),
            &request,
            &options,
        )
        .await
        {
            Ok(Some(response)) => {
                return upstream_passthrough_response(anthropic_stream_from_openai(
                    chat_completions_stream_output_with_settlement(response, settlement_context),
                ));
            }
            Ok(None) => {
                return upstream_passthrough_response(anthropic_stream_from_openai(
                    chat_completions_stream_output_with_settlement(
                        local_chat_completion_stream_output(&request.model),
                        settlement_context,
                    ),
                ));
            }
            Err(error) => {
                if let Some(hold) = canonical_hold.as_ref() {
                    if settle_canonical_chat_request_release(state.store.as_ref(), hold)
                        .await
                        .is_err()
                    {
                        return (
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            "failed to release canonical account hold",
                        )
                            .into_response();
                    }
                }
                if let Some(admission_error) = error.downcast_ref::<GatewayTrafficAdmissionError>()
                {
                    return commercial_admission_error_response(admission_error);
                }
                return anthropic_bad_gateway_response(
                    "failed to relay upstream anthropic message stream",
                );
            }
        }
    }

    match relay_chat_completion_from_store_with_options(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
        &options,
    )
    .await
    {
        Ok(Some(response)) => {
            let token_usage = extract_token_usage_metrics(&response);
            let canonical_settlement =
                canonical_usage_priced_settlement_amounts(canonical_hold.as_ref(), token_usage);
            if record_gateway_usage_for_project_with_route_key_and_tokens_and_reference(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "chat_completion",
                &request.model,
                &request.model,
                canonical_settlement
                    .as_ref()
                    .map(|settlement| settlement.usage_units)
                    .unwrap_or(CHAT_COMPLETION_HOLD_UNITS),
                canonical_settlement
                    .as_ref()
                    .map(|settlement| settlement.customer_charge)
                    .unwrap_or(CHAT_COMPLETION_RETAIL_CHARGE),
                token_usage,
                response_usage_id_or_single_data_item_id(&response),
            )
            .await
            .is_err()
            {
                if let Some(hold) = canonical_hold.as_ref() {
                    let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
                }
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            if let (Some(hold), Some(settlement)) =
                (canonical_hold.as_ref(), canonical_settlement.as_ref())
            {
                if settle_canonical_chat_request_capture(state.store.as_ref(), hold, settlement)
                    .await
                    .is_err()
                {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to capture canonical account hold",
                    )
                        .into_response();
                }
            }

            return Json(openai_chat_response_to_anthropic(&response)).into_response();
        }
        Ok(None) => {}
        Err(error) => {
            if let Some(hold) = canonical_hold.as_ref() {
                if settle_canonical_chat_request_release(state.store.as_ref(), hold)
                    .await
                    .is_err()
                {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to release canonical account hold",
                    )
                        .into_response();
                }
            }
            if let Some(admission_error) = error.downcast_ref::<GatewayTrafficAdmissionError>() {
                return commercial_admission_error_response(admission_error);
            }
            return anthropic_bad_gateway_response("failed to relay upstream anthropic message");
        }
    }

    let canonical_settlement =
        canonical_usage_priced_settlement_amounts(canonical_hold.as_ref(), None);

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "chat_completion",
        &request.model,
        canonical_settlement
            .as_ref()
            .map(|settlement| settlement.usage_units)
            .unwrap_or(CHAT_COMPLETION_HOLD_UNITS),
        canonical_settlement
            .as_ref()
            .map(|settlement| settlement.customer_charge)
            .unwrap_or(CHAT_COMPLETION_RETAIL_CHARGE),
    )
    .await
    .is_err()
    {
        if let Some(hold) = canonical_hold.as_ref() {
            let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
        }
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    if let (Some(hold), Some(settlement)) = (canonical_hold.as_ref(), canonical_settlement.as_ref())
    {
        if settle_canonical_chat_request_capture(state.store.as_ref(), hold, settlement)
            .await
            .is_err()
        {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to capture canonical account hold",
            )
                .into_response();
        }
    }

    Json(openai_chat_response_to_anthropic(
        &serde_json::to_value(
            create_chat_completion(
                request_context.tenant_id(),
                request_context.project_id(),
                &request.model,
            )
            .expect("chat completion"),
        )
        .expect("chat completion value"),
    ))
    .into_response()
}

async fn anthropic_count_tokens_with_state_handler(
    request_context: CompatAuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(payload): ExtractJson<Value>,
) -> Response {
    let request = match anthropic_count_tokens_request(&payload) {
        Ok(request) => request,
        Err(error) => return anthropic_invalid_request_response(error.to_string()),
    };

    match relay_count_response_input_tokens_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => Json(openai_count_tokens_to_anthropic(&response)).into_response(),
        Ok(None) => Json(openai_count_tokens_to_anthropic(
            &serde_json::to_value(
                count_response_input_tokens(
                    request_context.tenant_id(),
                    request_context.project_id(),
                    &request.model,
                )
                .expect("response input tokens"),
            )
            .expect("response input token value"),
        ))
        .into_response(),
        Err(_) => anthropic_bad_gateway_response(
            "failed to relay upstream anthropic count tokens request",
        ),
    }
}

async fn gemini_models_compat_handler(
    request_context: StatelessGatewayRequest,
    Path(tail): Path<String>,
    ExtractJson(payload): ExtractJson<Value>,
) -> Response {
    let Some((model, action)) = parse_gemini_compat_tail(&tail) else {
        return gemini_invalid_request_response("unsupported gemini compatibility route");
    };

    match action {
        GeminiCompatAction::GenerateContent => {
            let request = match gemini_request_to_chat_completion(&model, &payload) {
                Ok(request) => request,
                Err(error) => return gemini_invalid_request_response(error.to_string()),
            };

            match relay_stateless_json_request(
                &request_context,
                ProviderRequest::ChatCompletions(&request),
            )
            .await
            {
                Ok(Some(response)) => {
                    Json(openai_chat_response_to_gemini(&response)).into_response()
                }
                Ok(None) => Json(openai_chat_response_to_gemini(
                    &serde_json::to_value(
                        create_chat_completion(
                            request_context.tenant_id(),
                            request_context.project_id(),
                            &request.model,
                        )
                        .expect("chat completion"),
                    )
                    .expect("chat completion value"),
                ))
                .into_response(),
                Err(_) => gemini_bad_gateway_response(
                    "failed to relay upstream gemini generateContent request",
                ),
            }
        }
        GeminiCompatAction::StreamGenerateContent => {
            let mut request = match gemini_request_to_chat_completion(&model, &payload) {
                Ok(request) => request,
                Err(error) => return gemini_invalid_request_response(error.to_string()),
            };
            request.stream = Some(true);

            match relay_stateless_stream_request(
                &request_context,
                ProviderRequest::ChatCompletionsStream(&request),
            )
            .await
            {
                Ok(Some(response)) => {
                    upstream_passthrough_response(gemini_stream_from_openai(response))
                }
                Ok(None) => local_gemini_stream_response(),
                Err(_) => gemini_bad_gateway_response(
                    "failed to relay upstream gemini streamGenerateContent request",
                ),
            }
        }
        GeminiCompatAction::CountTokens => {
            let request = gemini_count_tokens_request(&model, &payload);
            match relay_stateless_json_request(
                &request_context,
                ProviderRequest::ResponsesInputTokens(&request),
            )
            .await
            {
                Ok(Some(response)) => {
                    Json(openai_count_tokens_to_gemini(&response)).into_response()
                }
                Ok(None) => Json(openai_count_tokens_to_gemini(
                    &serde_json::to_value(
                        count_response_input_tokens(
                            request_context.tenant_id(),
                            request_context.project_id(),
                            &request.model,
                        )
                        .expect("response input tokens"),
                    )
                    .expect("response input token value"),
                ))
                .into_response(),
                Err(_) => gemini_bad_gateway_response(
                    "failed to relay upstream gemini countTokens request",
                ),
            }
        }
    }
}

async fn gemini_models_compat_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Extension(request_id): Extension<RequestId>,
    Path(tail): Path<String>,
    ExtractJson(payload): ExtractJson<Value>,
) -> Response {
    let Some((model, action)) = parse_gemini_compat_tail(&tail) else {
        return gemini_invalid_request_response("unsupported gemini compatibility route");
    };

    match action {
        GeminiCompatAction::GenerateContent => {
            let request = match gemini_request_to_chat_completion(&model, &payload) {
                Ok(request) => request,
                Err(error) => return gemini_invalid_request_response(error.to_string()),
            };

            let canonical_hold = match admit_canonical_chat_request(
                state.store.as_ref(),
                &request_context,
                &request_id,
                &request,
            )
            .await
            {
                Ok(CanonicalChatAdmission::Held(hold)) => Some(hold),
                Ok(CanonicalChatAdmission::InsufficientBalance {
                    account_id,
                    shortfall_quantity,
                }) => {
                    return account_balance_insufficient_response(account_id, shortfall_quantity);
                }
                Ok(CanonicalChatAdmission::NotApplicable) => {
                    match enforce_project_quota(
                        state.store.as_ref(),
                        request_context.project_id(),
                        CHAT_COMPLETION_HOLD_UNITS,
                    )
                    .await
                    {
                        Ok(Some(response)) => return response,
                        Ok(None) => None,
                        Err(_) => {
                            return (
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                "failed to evaluate quota",
                            )
                                .into_response();
                        }
                    }
                }
                Err(_) => {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to evaluate canonical account admission",
                    )
                        .into_response();
                }
            };

            match relay_chat_completion_from_store(
                state.store.as_ref(),
                &state.secret_manager,
                request_context.tenant_id(),
                request_context.project_id(),
                &request,
            )
            .await
            {
                Ok(Some(response)) => {
                    let token_usage = extract_token_usage_metrics(&response);
                    let canonical_settlement = canonical_usage_priced_settlement_amounts(
                        canonical_hold.as_ref(),
                        token_usage,
                    );
                    if record_gateway_usage_for_project_with_route_key_and_tokens_and_reference(
                        state.store.as_ref(),
                        request_context.tenant_id(),
                        request_context.project_id(),
                        "chat_completion",
                        &request.model,
                        &request.model,
                        canonical_settlement
                            .as_ref()
                            .map(|settlement| settlement.usage_units)
                            .unwrap_or(CHAT_COMPLETION_HOLD_UNITS),
                        canonical_settlement
                            .as_ref()
                            .map(|settlement| settlement.customer_charge)
                            .unwrap_or(CHAT_COMPLETION_RETAIL_CHARGE),
                        token_usage,
                        response_usage_id_or_single_data_item_id(&response),
                    )
                    .await
                    .is_err()
                    {
                        if let Some(hold) = canonical_hold.as_ref() {
                            let _ =
                                settle_canonical_chat_request_release(state.store.as_ref(), hold)
                                    .await;
                        }
                        return (
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            "failed to record usage",
                        )
                            .into_response();
                    }

                    if let (Some(hold), Some(settlement)) =
                        (canonical_hold.as_ref(), canonical_settlement.as_ref())
                    {
                        if settle_canonical_chat_request_capture(
                            state.store.as_ref(),
                            hold,
                            settlement,
                        )
                        .await
                        .is_err()
                        {
                            return (
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                "failed to capture canonical account hold",
                            )
                                .into_response();
                        }
                    }

                    Json(openai_chat_response_to_gemini(&response)).into_response()
                }
                Ok(None) => {
                    let canonical_settlement =
                        canonical_usage_priced_settlement_amounts(canonical_hold.as_ref(), None);
                    if record_gateway_usage_for_project(
                        state.store.as_ref(),
                        request_context.tenant_id(),
                        request_context.project_id(),
                        "chat_completion",
                        &request.model,
                        canonical_settlement
                            .as_ref()
                            .map(|settlement| settlement.usage_units)
                            .unwrap_or(CHAT_COMPLETION_HOLD_UNITS),
                        canonical_settlement
                            .as_ref()
                            .map(|settlement| settlement.customer_charge)
                            .unwrap_or(CHAT_COMPLETION_RETAIL_CHARGE),
                    )
                    .await
                    .is_err()
                    {
                        if let Some(hold) = canonical_hold.as_ref() {
                            let _ =
                                settle_canonical_chat_request_release(state.store.as_ref(), hold)
                                    .await;
                        }
                        return (
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            "failed to record usage",
                        )
                            .into_response();
                    }

                    if let (Some(hold), Some(settlement)) =
                        (canonical_hold.as_ref(), canonical_settlement.as_ref())
                    {
                        if settle_canonical_chat_request_capture(
                            state.store.as_ref(),
                            hold,
                            settlement,
                        )
                        .await
                        .is_err()
                        {
                            return (
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                "failed to capture canonical account hold",
                            )
                                .into_response();
                        }
                    }

                    Json(openai_chat_response_to_gemini(
                        &serde_json::to_value(
                            create_chat_completion(
                                request_context.tenant_id(),
                                request_context.project_id(),
                                &request.model,
                            )
                            .expect("chat completion"),
                        )
                        .expect("chat completion value"),
                    ))
                    .into_response()
                }
                Err(_) => {
                    if let Some(hold) = canonical_hold.as_ref() {
                        if settle_canonical_chat_request_release(state.store.as_ref(), hold)
                            .await
                            .is_err()
                        {
                            return (
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                "failed to release canonical account hold",
                            )
                                .into_response();
                        }
                    }

                    gemini_bad_gateway_response(
                        "failed to relay upstream gemini generateContent request",
                    )
                }
            }
        }
        GeminiCompatAction::StreamGenerateContent => {
            let mut request = match gemini_request_to_chat_completion(&model, &payload) {
                Ok(request) => request,
                Err(error) => return gemini_invalid_request_response(error.to_string()),
            };
            request.stream = Some(true);

            let canonical_hold = match admit_canonical_chat_request(
                state.store.as_ref(),
                &request_context,
                &request_id,
                &request,
            )
            .await
            {
                Ok(CanonicalChatAdmission::Held(hold)) => Some(hold),
                Ok(CanonicalChatAdmission::InsufficientBalance {
                    account_id,
                    shortfall_quantity,
                }) => {
                    return account_balance_insufficient_response(account_id, shortfall_quantity);
                }
                Ok(CanonicalChatAdmission::NotApplicable) => {
                    match enforce_project_quota(
                        state.store.as_ref(),
                        request_context.project_id(),
                        CHAT_COMPLETION_HOLD_UNITS,
                    )
                    .await
                    {
                        Ok(Some(response)) => return response,
                        Ok(None) => None,
                        Err(_) => {
                            return (
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                "failed to evaluate quota",
                            )
                                .into_response();
                        }
                    }
                }
                Err(_) => {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to evaluate canonical account admission",
                    )
                        .into_response();
                }
            };

            let settlement_context = UsagePricedStreamSettlementContext {
                store: state.store.clone(),
                tenant_id: request_context.tenant_id().to_owned(),
                project_id: request_context.project_id().to_owned(),
                model: request.model.clone(),
                canonical_hold: canonical_hold.clone(),
            };

            match relay_chat_completion_stream_from_store(
                state.store.as_ref(),
                &state.secret_manager,
                request_context.tenant_id(),
                request_context.project_id(),
                &request,
            )
            .await
            {
                Ok(Some(response)) => upstream_passthrough_response(gemini_stream_from_openai(
                    chat_completions_stream_output_with_settlement(response, settlement_context),
                )),
                Ok(None) => upstream_passthrough_response(gemini_stream_from_openai(
                    chat_completions_stream_output_with_settlement(
                        local_chat_completion_stream_output(&request.model),
                        settlement_context,
                    ),
                )),
                Err(_) => {
                    if let Some(hold) = canonical_hold.as_ref() {
                        if settle_canonical_chat_request_release(state.store.as_ref(), hold)
                            .await
                            .is_err()
                        {
                            return (
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                "failed to release canonical account hold",
                            )
                                .into_response();
                        }
                    }

                    gemini_bad_gateway_response(
                        "failed to relay upstream gemini streamGenerateContent request",
                    )
                }
            }
        }
        GeminiCompatAction::CountTokens => {
            let request = gemini_count_tokens_request(&model, &payload);
            match relay_count_response_input_tokens_from_store(
                state.store.as_ref(),
                &state.secret_manager,
                request_context.tenant_id(),
                request_context.project_id(),
                &request,
            )
            .await
            {
                Ok(Some(response)) => {
                    Json(openai_count_tokens_to_gemini(&response)).into_response()
                }
                Ok(None) => Json(openai_count_tokens_to_gemini(
                    &serde_json::to_value(
                        count_response_input_tokens(
                            request_context.tenant_id(),
                            request_context.project_id(),
                            &request.model,
                        )
                        .expect("response input tokens"),
                    )
                    .expect("response input token value"),
                ))
                .into_response(),
                Err(_) => gemini_bad_gateway_response(
                    "failed to relay upstream gemini countTokens request",
                ),
            }
        }
    }
}

#[derive(Clone, Copy)]
enum GeminiCompatAction {
    GenerateContent,
    StreamGenerateContent,
    CountTokens,
}

fn parse_gemini_compat_tail(tail: &str) -> Option<(String, GeminiCompatAction)> {
    let tail = tail.trim_start_matches('/');
    let (model, action) = tail.split_once(':')?;
    let action = match action {
        "generateContent" => GeminiCompatAction::GenerateContent,
        "streamGenerateContent" => GeminiCompatAction::StreamGenerateContent,
        "countTokens" => GeminiCompatAction::CountTokens,
        _ => return None,
    };
    Some((model.to_owned(), action))
}

fn local_anthropic_stream_response(model: &str) -> Response {
    let body = format!(
        "event: message_start\ndata: {}\n\n\
event: message_delta\ndata: {}\n\n\
event: message_stop\ndata: {}\n\n",
        serde_json::json!({
            "type": "message_start",
            "message": {
                "id": "msg_1",
                "type": "message",
                "role": "assistant",
                "model": model,
                "content": [],
                "stop_reason": Value::Null,
                "stop_sequence": Value::Null,
                "usage": {
                    "input_tokens": 0,
                    "output_tokens": 0
                }
            }
        }),
        serde_json::json!({
            "type": "message_delta",
            "delta": {
                "stop_reason": "end_turn",
                "stop_sequence": Value::Null
            },
            "usage": {
                "output_tokens": 0
            }
        }),
        serde_json::json!({
            "type": "message_stop"
        })
    );
    ([(header::CONTENT_TYPE, "text/event-stream")], body).into_response()
}

fn local_gemini_stream_response() -> Response {
    let body = format!(
        "data: {}\n\n",
        serde_json::json!({
            "candidates": [{
                "content": {
                    "role": "model",
                    "parts": [
                        { "text": "" }
                    ]
                },
                "finishReason": "STOP"
            }]
        })
    );
    ([(header::CONTENT_TYPE, "text/event-stream")], body).into_response()
}

async fn chat_completions_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Response {
    match relay_list_chat_completions_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "chat_completion",
                "chat_completions",
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
            return bad_gateway_openai_response("failed to relay upstream chat completion list");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "chat_completion",
        "chat_completions",
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
        list_chat_completions(request_context.tenant_id(), request_context.project_id())
            .expect("chat completions"),
    )
    .into_response()
}

async fn chat_completion_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(completion_id): Path<String>,
) -> Response {
    match relay_get_chat_completion_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &completion_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "chat_completion",
                &completion_id,
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
            return bad_gateway_openai_response(
                "failed to relay upstream chat completion retrieve",
            );
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "chat_completion",
        &completion_id,
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
        get_chat_completion(
            request_context.tenant_id(),
            request_context.project_id(),
            &completion_id,
        )
        .expect("chat completion"),
    )
    .into_response()
}

async fn chat_completion_update_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(completion_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateChatCompletionRequest>,
) -> Response {
    match relay_update_chat_completion_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &completion_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "chat_completion",
                &completion_id,
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
            return bad_gateway_openai_response("failed to relay upstream chat completion update");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "chat_completion",
        &completion_id,
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
        update_chat_completion(
            request_context.tenant_id(),
            request_context.project_id(),
            &completion_id,
            request.metadata.unwrap_or(serde_json::json!({})),
        )
        .expect("chat completion update"),
    )
    .into_response()
}

async fn chat_completion_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(completion_id): Path<String>,
) -> Response {
    match relay_delete_chat_completion_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &completion_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "chat_completion",
                &completion_id,
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
            return bad_gateway_openai_response("failed to relay upstream chat completion delete");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "chat_completion",
        &completion_id,
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
        delete_chat_completion(
            request_context.tenant_id(),
            request_context.project_id(),
            &completion_id,
        )
        .expect("chat completion delete"),
    )
    .into_response()
}

async fn chat_completion_messages_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(completion_id): Path<String>,
) -> Response {
    match relay_list_chat_completion_messages_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &completion_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "chat_completion",
                &completion_id,
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
            return bad_gateway_openai_response(
                "failed to relay upstream chat completion messages",
            );
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "chat_completion",
        &completion_id,
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
        list_chat_completion_messages(
            request_context.tenant_id(),
            request_context.project_id(),
            &completion_id,
        )
        .expect("chat completion messages"),
    )
    .into_response()
}

async fn conversations_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateConversationRequest>,
) -> Response {
    match relay_conversation_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let conversation_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("conversations");
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "responses",
                "conversations",
                conversation_id,
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
            return bad_gateway_openai_response("failed to relay upstream conversation");
        }
    }

    let response = create_conversation(request_context.tenant_id(), request_context.project_id())
        .expect("conversation");

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        "conversations",
        response.id.as_str(),
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

    Json(response).into_response()
}

async fn conversations_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Response {
    match relay_list_conversations_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "responses",
                "conversations",
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
            return bad_gateway_openai_response("failed to relay upstream conversation list");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        "conversations",
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
        list_conversations(request_context.tenant_id(), request_context.project_id())
            .expect("conversation list"),
    )
    .into_response()
}

async fn conversation_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(conversation_id): Path<String>,
) -> Response {
    match relay_get_conversation_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_model = response_usage_id_or_single_data_item_id(&response)
                .unwrap_or(conversation_id.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "responses",
                &conversation_id,
                usage_model,
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
            return bad_gateway_openai_response("failed to relay upstream conversation retrieve");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        &conversation_id,
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
        get_conversation(
            request_context.tenant_id(),
            request_context.project_id(),
            &conversation_id,
        )
        .expect("conversation"),
    )
    .into_response()
}

async fn conversation_update_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(conversation_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateConversationRequest>,
) -> Response {
    match relay_update_conversation_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_model = response_usage_id_or_single_data_item_id(&response)
                .unwrap_or(conversation_id.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "responses",
                &conversation_id,
                usage_model,
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
            return bad_gateway_openai_response("failed to relay upstream conversation update");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        &conversation_id,
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
        update_conversation(
            request_context.tenant_id(),
            request_context.project_id(),
            &conversation_id,
            request.metadata.unwrap_or(serde_json::json!({})),
        )
        .expect("conversation update"),
    )
    .into_response()
}

async fn conversation_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(conversation_id): Path<String>,
) -> Response {
    match relay_delete_conversation_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_model = response_usage_id_or_single_data_item_id(&response)
                .unwrap_or(conversation_id.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "responses",
                &conversation_id,
                usage_model,
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
            return bad_gateway_openai_response("failed to relay upstream conversation delete");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        &conversation_id,
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
        delete_conversation(
            request_context.tenant_id(),
            request_context.project_id(),
            &conversation_id,
        )
        .expect("conversation delete"),
    )
    .into_response()
}

async fn conversation_items_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(conversation_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateConversationItemsRequest>,
) -> Response {
    match relay_conversation_items_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_model = response_usage_id_or_single_data_item_id(&response)
                .unwrap_or(conversation_id.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "responses",
                &conversation_id,
                usage_model,
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
            return bad_gateway_openai_response("failed to relay upstream conversation items");
        }
    }

    let response = create_conversation_items(
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
    )
    .expect("conversation items");
    let usage_model = match response.data.as_slice() {
        [item] => item.id.as_str(),
        _ => conversation_id.as_str(),
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        &conversation_id,
        usage_model,
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

    Json(response).into_response()
}

async fn conversation_items_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(conversation_id): Path<String>,
) -> Response {
    match relay_list_conversation_items_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "responses",
                &conversation_id,
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
            return bad_gateway_openai_response("failed to relay upstream conversation items list");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        &conversation_id,
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
        list_conversation_items(
            request_context.tenant_id(),
            request_context.project_id(),
            &conversation_id,
        )
        .expect("conversation items list"),
    )
    .into_response()
}

async fn conversation_item_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((conversation_id, item_id)): Path<(String, String)>,
) -> Response {
    match relay_get_conversation_item_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
        &item_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "responses",
                &conversation_id,
                &item_id,
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
            return bad_gateway_openai_response(
                "failed to relay upstream conversation item retrieve",
            );
        }
    }

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        &conversation_id,
        &item_id,
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
        get_conversation_item(
            request_context.tenant_id(),
            request_context.project_id(),
            &conversation_id,
            &item_id,
        )
        .expect("conversation item"),
    )
    .into_response()
}

async fn conversation_item_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((conversation_id, item_id)): Path<(String, String)>,
) -> Response {
    match relay_delete_conversation_item_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
        &item_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "responses",
                &conversation_id,
                &item_id,
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
            return bad_gateway_openai_response(
                "failed to relay upstream conversation item delete",
            );
        }
    }

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        &conversation_id,
        &item_id,
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
        delete_conversation_item(
            request_context.tenant_id(),
            request_context.project_id(),
            &conversation_id,
            &item_id,
        )
        .expect("conversation item delete"),
    )
    .into_response()
}

async fn threads_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateThreadRequest>,
) -> Response {
    match relay_thread_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let thread_usage_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("threads");
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                "threads",
                thread_usage_id,
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
            return bad_gateway_openai_response("failed to relay upstream thread");
        }
    }

    let response =
        create_thread(request_context.tenant_id(), request_context.project_id()).expect("thread");

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        "threads",
        response.id.as_str(),
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

    Json(response).into_response()
}

async fn thread_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(thread_id): Path<String>,
) -> Response {
    match relay_get_thread_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &thread_id,
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
            return bad_gateway_openai_response("failed to relay upstream thread retrieve");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &thread_id,
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
        get_thread(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
        )
        .expect("thread retrieve"),
    )
    .into_response()
}

async fn thread_update_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(thread_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateThreadRequest>,
) -> Response {
    match relay_update_thread_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &thread_id,
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
            return bad_gateway_openai_response("failed to relay upstream thread update");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &thread_id,
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
        update_thread(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
        )
        .expect("thread update"),
    )
    .into_response()
}

async fn thread_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(thread_id): Path<String>,
) -> Response {
    match relay_delete_thread_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &thread_id,
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
            return bad_gateway_openai_response("failed to relay upstream thread delete");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &thread_id,
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
        delete_thread(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
        )
        .expect("thread delete"),
    )
    .into_response()
}

async fn thread_messages_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(thread_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateThreadMessageRequest>,
) -> Response {
    match relay_thread_messages_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let message_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(thread_id.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &thread_id,
                message_id,
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
            return bad_gateway_openai_response("failed to relay upstream thread message");
        }
    }

    let text = request.content.as_str().unwrap_or("hello");
    let response = create_thread_message(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &request.role,
        text,
    )
    .expect("thread message create");

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &thread_id,
        response.id.as_str(),
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

    Json(response).into_response()
}

async fn thread_messages_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(thread_id): Path<String>,
) -> Response {
    match relay_list_thread_messages_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &thread_id,
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
            return bad_gateway_openai_response("failed to relay upstream thread message list");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &thread_id,
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
        list_thread_messages(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
        )
        .expect("thread messages list"),
    )
    .into_response()
}

async fn thread_message_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((thread_id, message_id)): Path<(String, String)>,
) -> Response {
    match relay_get_thread_message_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &message_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &thread_id,
                &message_id,
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
            return bad_gateway_openai_response("failed to relay upstream thread message retrieve");
        }
    }

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &thread_id,
        &message_id,
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
        get_thread_message(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
            &message_id,
        )
        .expect("thread message retrieve"),
    )
    .into_response()
}

async fn thread_message_update_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((thread_id, message_id)): Path<(String, String)>,
    ExtractJson(request): ExtractJson<UpdateThreadMessageRequest>,
) -> Response {
    match relay_update_thread_message_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &message_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &thread_id,
                &message_id,
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
            return bad_gateway_openai_response("failed to relay upstream thread message update");
        }
    }

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &thread_id,
        &message_id,
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
        update_thread_message(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
            &message_id,
        )
        .expect("thread message update"),
    )
    .into_response()
}

async fn thread_message_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((thread_id, message_id)): Path<(String, String)>,
) -> Response {
    match relay_delete_thread_message_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &message_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &thread_id,
                &message_id,
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
            return bad_gateway_openai_response("failed to relay upstream thread message delete");
        }
    }

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &thread_id,
        &message_id,
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
        delete_thread_message(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
            &message_id,
        )
        .expect("thread message delete"),
    )
    .into_response()
}

async fn thread_and_run_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateThreadAndRunRequest>,
) -> Response {
    match relay_thread_and_run_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_model =
                response_usage_id_or_single_data_item_id(&response).unwrap_or("threads/runs");
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                "threads/runs",
                usage_model,
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
            return bad_gateway_openai_response("failed to relay upstream thread and run");
        }
    }

    let response = create_thread_and_run(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.assistant_id,
    )
    .expect("thread and run create");

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        "threads/runs",
        response.id.as_str(),
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

    Json(response).into_response()
}

async fn thread_runs_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(thread_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateRunRequest>,
) -> Response {
    match relay_thread_run_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let run_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(thread_id.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &thread_id,
                run_id,
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
            return bad_gateway_openai_response("failed to relay upstream thread run");
        }
    }

    let response = create_thread_run(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &request.assistant_id,
        request.model.as_deref(),
    )
    .expect("thread run create");

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &thread_id,
        response.id.as_str(),
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

    Json(response).into_response()
}

async fn thread_runs_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(thread_id): Path<String>,
) -> Response {
    match relay_list_thread_runs_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &thread_id,
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
            return bad_gateway_openai_response("failed to relay upstream thread runs list");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &thread_id,
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
        list_thread_runs(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
        )
        .expect("thread runs list"),
    )
    .into_response()
}

async fn thread_run_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((thread_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_get_thread_run_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &thread_id,
                &run_id,
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
            return bad_gateway_openai_response("failed to relay upstream thread run retrieve");
        }
    }

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &thread_id,
        &run_id,
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
        get_thread_run(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
            &run_id,
        )
        .expect("thread run"),
    )
    .into_response()
}

async fn thread_run_update_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((thread_id, run_id)): Path<(String, String)>,
    ExtractJson(request): ExtractJson<UpdateRunRequest>,
) -> Response {
    match relay_update_thread_run_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &thread_id,
                &run_id,
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
            return bad_gateway_openai_response("failed to relay upstream thread run update");
        }
    }

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &thread_id,
        &run_id,
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
        update_thread_run(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
            &run_id,
        )
        .expect("thread run update"),
    )
    .into_response()
}

async fn thread_run_cancel_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((thread_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_cancel_thread_run_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &thread_id,
                &run_id,
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
            return bad_gateway_openai_response("failed to relay upstream thread run cancel");
        }
    }

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &thread_id,
        &run_id,
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
        cancel_thread_run(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
            &run_id,
        )
        .expect("thread run cancel"),
    )
    .into_response()
}

async fn thread_run_submit_tool_outputs_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((thread_id, run_id)): Path<(String, String)>,
    ExtractJson(request): ExtractJson<SubmitToolOutputsRunRequest>,
) -> Response {
    match relay_submit_thread_run_tool_outputs_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &thread_id,
                &run_id,
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
            return bad_gateway_openai_response("failed to relay upstream thread run tool outputs");
        }
    }

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &thread_id,
        &run_id,
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

    let tool_outputs = request
        .tool_outputs
        .iter()
        .map(|output| (output.tool_call_id.as_str(), output.output.as_str()))
        .collect();
    Json(
        submit_thread_run_tool_outputs(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
            &run_id,
            tool_outputs,
        )
        .expect("thread run tool outputs"),
    )
    .into_response()
}

async fn thread_run_steps_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((thread_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_list_thread_run_steps_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &thread_id,
                &run_id,
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
            return bad_gateway_openai_response("failed to relay upstream thread run steps");
        }
    }

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &thread_id,
        &run_id,
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
        list_thread_run_steps(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
            &run_id,
        )
        .expect("thread run steps"),
    )
    .into_response()
}

async fn thread_run_step_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((thread_id, run_id, step_id)): Path<(String, String, String)>,
) -> Response {
    match relay_get_thread_run_step_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
        &step_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &thread_id,
                &step_id,
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
            return bad_gateway_openai_response(
                "failed to relay upstream thread run step retrieve",
            );
        }
    }

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &thread_id,
        &step_id,
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
        get_thread_run_step(
            request_context.tenant_id(),
            request_context.project_id(),
            &thread_id,
            &run_id,
            &step_id,
        )
        .expect("thread run step"),
    )
    .into_response()
}

fn upstream_passthrough_response(response: ProviderStreamOutput) -> Response {
    let content_type = response.content_type().to_owned();
    Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .body(Body::from_stream(response.into_body_stream()))
        .expect("valid upstream stream response")
}

fn responses_stream_response_with_settlement(
    response: ProviderStreamOutput,
    settlement_context: UsagePricedStreamSettlementContext,
) -> Response {
    let content_type = response.content_type().to_owned();
    let body = Body::from_stream(spawn_responses_stream_with_settlement(
        response.into_body_stream(),
        settlement_context,
    ));
    Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .body(body)
        .expect("valid settled response stream response")
}

fn chat_completions_stream_response_with_settlement(
    response: ProviderStreamOutput,
    settlement_context: UsagePricedStreamSettlementContext,
) -> Response {
    upstream_passthrough_response(chat_completions_stream_output_with_settlement(
        response,
        settlement_context,
    ))
}

fn chat_completions_stream_output_with_settlement(
    response: ProviderStreamOutput,
    settlement_context: UsagePricedStreamSettlementContext,
) -> ProviderStreamOutput {
    let content_type = response.content_type().to_owned();
    let body = spawn_chat_completions_stream_with_settlement(
        response.into_body_stream(),
        settlement_context,
    );
    ProviderStreamOutput::new(content_type, body)
}

fn local_file_content_response(tenant_id: &str, project_id: &str, file_id: &str) -> Response {
    let bytes = file_content(tenant_id, project_id, file_id).expect("file content");
    Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/jsonl")
        .body(Body::from(bytes))
        .expect("valid local file content response")
}

fn local_container_file_content_response(
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
    file_id: &str,
) -> Response {
    let bytes = sdkwork_api_app_gateway::container_file_content(
        tenant_id,
        project_id,
        container_id,
        file_id,
    )
    .expect("container file content");
    Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .body(Body::from(bytes))
        .expect("valid local container file content response")
}

fn local_video_content_response(tenant_id: &str, project_id: &str, video_id: &str) -> Response {
    let bytes = video_content(tenant_id, project_id, video_id).expect("video content");
    Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(header::CONTENT_TYPE, "video/mp4")
        .body(Body::from(bytes))
        .expect("valid local video content response")
}

fn local_music_content_response(tenant_id: &str, project_id: &str, music_id: &str) -> Response {
    let bytes = music_content(tenant_id, project_id, music_id).expect("music content");
    Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(header::CONTENT_TYPE, "audio/mpeg")
        .body(Body::from(bytes))
        .expect("valid local music content response")
}

async fn responses_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Extension(request_id): Extension<RequestId>,
    ExtractJson(request): ExtractJson<CreateResponseRequest>,
) -> Response {
    let canonical_hold = match admit_canonical_response_request(
        state.store.as_ref(),
        &request_context,
        &request_id,
        &request,
    )
    .await
    {
        Ok(CanonicalChatAdmission::Held(hold)) => Some(hold),
        Ok(CanonicalChatAdmission::InsufficientBalance {
            account_id,
            shortfall_quantity,
        }) => {
            return account_balance_insufficient_response(account_id, shortfall_quantity);
        }
        Ok(CanonicalChatAdmission::NotApplicable) => {
            match enforce_project_quota(
                state.store.as_ref(),
                request_context.project_id(),
                RESPONSES_HOLD_UNITS,
            )
            .await
            {
                Ok(Some(response)) => return response,
                Ok(None) => None,
                Err(_) => {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to evaluate quota",
                    )
                        .into_response();
                }
            }
        }
        Err(_) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to evaluate canonical account admission",
            )
                .into_response();
        }
    };

    if request.stream.unwrap_or(false) {
        let settlement_context = UsagePricedStreamSettlementContext {
            store: state.store.clone(),
            tenant_id: request_context.tenant_id().to_owned(),
            project_id: request_context.project_id().to_owned(),
            model: request.model.clone(),
            canonical_hold: canonical_hold.clone(),
        };
        match relay_response_stream_from_store(
            state.store.as_ref(),
            &state.secret_manager,
            request_context.tenant_id(),
            request_context.project_id(),
            &request,
        )
        .await
        {
            Ok(Some(response)) => {
                return responses_stream_response_with_settlement(response, settlement_context);
            }
            Ok(None) => {
                return responses_stream_response_with_settlement(
                    local_response_stream_output("resp_1", &request.model),
                    settlement_context,
                );
            }
            Err(_) => {
                if let Some(hold) = canonical_hold.as_ref() {
                    if settle_canonical_chat_request_release(state.store.as_ref(), hold)
                        .await
                        .is_err()
                    {
                        return (
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            "failed to release canonical account hold",
                        )
                            .into_response();
                    }
                }
                return bad_gateway_openai_response("failed to relay upstream response stream");
            }
        }
    }

    match relay_response_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let token_usage = extract_token_usage_metrics(&response);
            let canonical_settlement =
                canonical_usage_priced_settlement_amounts(canonical_hold.as_ref(), token_usage);
            if record_gateway_usage_for_project_with_route_key_and_tokens_and_reference(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "responses",
                &request.model,
                &request.model,
                canonical_settlement
                    .as_ref()
                    .map(|settlement| settlement.usage_units)
                    .unwrap_or(RESPONSES_HOLD_UNITS),
                canonical_settlement
                    .as_ref()
                    .map(|settlement| settlement.customer_charge)
                    .unwrap_or(RESPONSES_RETAIL_CHARGE),
                token_usage,
                response_usage_id_or_single_data_item_id(&response),
            )
            .await
            .is_err()
            {
                if let Some(hold) = canonical_hold.as_ref() {
                    let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
                }
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }
            if let (Some(hold), Some(settlement)) =
                (canonical_hold.as_ref(), canonical_settlement.as_ref())
            {
                if settle_canonical_chat_request_capture(state.store.as_ref(), hold, settlement)
                    .await
                    .is_err()
                {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to capture canonical account hold",
                    )
                        .into_response();
                }
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            if let Some(hold) = canonical_hold.as_ref() {
                if settle_canonical_chat_request_release(state.store.as_ref(), hold)
                    .await
                    .is_err()
                {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to release canonical account hold",
                    )
                        .into_response();
                }
            }
            return bad_gateway_openai_response("failed to relay upstream response");
        }
    }

    let canonical_settlement =
        canonical_usage_priced_settlement_amounts(canonical_hold.as_ref(), None);

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        &request.model,
        canonical_settlement
            .as_ref()
            .map(|settlement| settlement.usage_units)
            .unwrap_or(RESPONSES_HOLD_UNITS),
        canonical_settlement
            .as_ref()
            .map(|settlement| settlement.customer_charge)
            .unwrap_or(RESPONSES_RETAIL_CHARGE),
    )
    .await
    .is_err()
    {
        if let Some(hold) = canonical_hold.as_ref() {
            let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
        }
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    if let (Some(hold), Some(settlement)) = (canonical_hold.as_ref(), canonical_settlement.as_ref())
    {
        if settle_canonical_chat_request_capture(state.store.as_ref(), hold, settlement)
            .await
            .is_err()
        {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to capture canonical account hold",
            )
                .into_response();
        }
    }

    Json(
        create_response(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        )
        .expect("response"),
    )
    .into_response()
}

async fn response_input_tokens_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CountResponseInputTokensRequest>,
) -> Response {
    match relay_count_response_input_tokens_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "responses",
                &request.model,
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
            return bad_gateway_openai_response("failed to relay upstream response input tokens");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        &request.model,
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
        count_response_input_tokens(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        )
        .expect("response input tokens"),
    )
    .into_response()
}

async fn response_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(response_id): Path<String>,
) -> Response {
    match relay_get_response_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &response_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "responses",
                &response_id,
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
            return bad_gateway_openai_response("failed to relay upstream response retrieve");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        &response_id,
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
        get_response(
            request_context.tenant_id(),
            request_context.project_id(),
            &response_id,
        )
        .expect("response retrieve"),
    )
    .into_response()
}

async fn response_input_items_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(response_id): Path<String>,
) -> Response {
    match relay_list_response_input_items_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &response_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "responses",
                &response_id,
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
            return bad_gateway_openai_response("failed to relay upstream response input items");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        &response_id,
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
        list_response_input_items(
            request_context.tenant_id(),
            request_context.project_id(),
            &response_id,
        )
        .expect("response input items"),
    )
    .into_response()
}

async fn response_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(response_id): Path<String>,
) -> Response {
    match relay_delete_response_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &response_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "responses",
                &response_id,
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
            return bad_gateway_openai_response("failed to relay upstream response delete");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        &response_id,
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
        delete_response(
            request_context.tenant_id(),
            request_context.project_id(),
            &response_id,
        )
        .expect("response delete"),
    )
    .into_response()
}

async fn response_cancel_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(response_id): Path<String>,
) -> Response {
    match relay_cancel_response_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &response_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "responses",
                &response_id,
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
            return bad_gateway_openai_response("failed to relay upstream response cancel");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        &response_id,
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
        cancel_response(
            request_context.tenant_id(),
            request_context.project_id(),
            &response_id,
        )
        .expect("response cancel"),
    )
    .into_response()
}

async fn response_compact_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CompactResponseRequest>,
) -> Response {
    match relay_compact_response_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "responses",
                &request.model,
                60,
                0.06,
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
            return bad_gateway_openai_response("failed to relay upstream response compact");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        &request.model,
        60,
        0.06,
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
        compact_response(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        )
        .expect("response compact"),
    )
    .into_response()
}

async fn completions_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Extension(request_id): Extension<RequestId>,
    ExtractJson(request): ExtractJson<CreateCompletionRequest>,
) -> Response {
    let canonical_hold = match admit_canonical_completion_request(
        state.store.as_ref(),
        &request_context,
        &request_id,
        &request,
    )
    .await
    {
        Ok(CanonicalChatAdmission::Held(hold)) => Some(hold),
        Ok(CanonicalChatAdmission::InsufficientBalance {
            account_id,
            shortfall_quantity,
        }) => {
            return account_balance_insufficient_response(account_id, shortfall_quantity);
        }
        Ok(CanonicalChatAdmission::NotApplicable) => {
            match enforce_project_quota(
                state.store.as_ref(),
                request_context.project_id(),
                COMPLETIONS_HOLD_UNITS,
            )
            .await
            {
                Ok(Some(response)) => return response,
                Ok(None) => None,
                Err(_) => {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to evaluate quota",
                    )
                        .into_response();
                }
            }
        }
        Err(_) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to evaluate canonical account admission",
            )
                .into_response();
        }
    };

    match relay_completion_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let token_usage = extract_token_usage_metrics(&response);
            let canonical_settlement =
                canonical_usage_priced_settlement_amounts(canonical_hold.as_ref(), token_usage);
            if record_gateway_usage_for_project_with_route_key_and_reference_id(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "completions",
                &request.model,
                &request.model,
                canonical_settlement
                    .as_ref()
                    .map(|settlement| settlement.usage_units)
                    .unwrap_or(COMPLETIONS_HOLD_UNITS),
                canonical_settlement
                    .as_ref()
                    .map(|settlement| settlement.customer_charge)
                    .unwrap_or(COMPLETIONS_RETAIL_CHARGE),
                response_usage_id_or_single_data_item_id(&response),
            )
            .await
            .is_err()
            {
                if let Some(hold) = canonical_hold.as_ref() {
                    let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
                }
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }
            if let (Some(hold), Some(settlement)) =
                (canonical_hold.as_ref(), canonical_settlement.as_ref())
            {
                if settle_canonical_chat_request_capture(state.store.as_ref(), hold, settlement)
                    .await
                    .is_err()
                {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to capture canonical account hold",
                    )
                        .into_response();
                }
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            if let Some(hold) = canonical_hold.as_ref() {
                if settle_canonical_chat_request_release(state.store.as_ref(), hold)
                    .await
                    .is_err()
                {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to release canonical account hold",
                    )
                        .into_response();
                }
            }
            return bad_gateway_openai_response("failed to relay upstream completion");
        }
    }

    let canonical_settlement =
        canonical_usage_priced_settlement_amounts(canonical_hold.as_ref(), None);

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "completions",
        &request.model,
        canonical_settlement
            .as_ref()
            .map(|settlement| settlement.usage_units)
            .unwrap_or(COMPLETIONS_HOLD_UNITS),
        canonical_settlement
            .as_ref()
            .map(|settlement| settlement.customer_charge)
            .unwrap_or(COMPLETIONS_RETAIL_CHARGE),
    )
    .await
    .is_err()
    {
        if let Some(hold) = canonical_hold.as_ref() {
            let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
        }
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    if let (Some(hold), Some(settlement)) = (canonical_hold.as_ref(), canonical_settlement.as_ref())
    {
        if settle_canonical_chat_request_capture(state.store.as_ref(), hold, settlement)
            .await
            .is_err()
        {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to capture canonical account hold",
            )
                .into_response();
        }
    }

    Json(
        create_completion(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        )
        .expect("completion"),
    )
    .into_response()
}

async fn embeddings_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Extension(request_id): Extension<RequestId>,
    ExtractJson(request): ExtractJson<CreateEmbeddingRequest>,
) -> Response {
    let canonical_hold = match admit_canonical_embedding_request(
        state.store.as_ref(),
        &request_context,
        &request_id,
        &request,
    )
    .await
    {
        Ok(CanonicalChatAdmission::Held(hold)) => Some(hold),
        Ok(CanonicalChatAdmission::InsufficientBalance {
            account_id,
            shortfall_quantity,
        }) => {
            return account_balance_insufficient_response(account_id, shortfall_quantity);
        }
        Ok(CanonicalChatAdmission::NotApplicable) => {
            match enforce_project_quota(
                state.store.as_ref(),
                request_context.project_id(),
                EMBEDDINGS_HOLD_UNITS,
            )
            .await
            {
                Ok(Some(response)) => return response,
                Ok(None) => None,
                Err(_) => {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to evaluate quota",
                    )
                        .into_response();
                }
            }
        }
        Err(_) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to evaluate canonical account admission",
            )
                .into_response();
        }
    };

    match relay_embedding_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let token_usage = extract_token_usage_metrics(&response);
            let canonical_settlement =
                canonical_usage_priced_settlement_amounts(canonical_hold.as_ref(), token_usage);
            if record_gateway_usage_for_project_with_route_key_and_tokens_and_reference(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "embeddings",
                &request.model,
                &request.model,
                canonical_settlement
                    .as_ref()
                    .map(|settlement| settlement.usage_units)
                    .unwrap_or(EMBEDDINGS_HOLD_UNITS),
                canonical_settlement
                    .as_ref()
                    .map(|settlement| settlement.customer_charge)
                    .unwrap_or(EMBEDDINGS_RETAIL_CHARGE),
                token_usage,
                response_usage_id_or_single_data_item_id(&response),
            )
            .await
            .is_err()
            {
                if let Some(hold) = canonical_hold.as_ref() {
                    let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
                }
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }
            if let (Some(hold), Some(settlement)) =
                (canonical_hold.as_ref(), canonical_settlement.as_ref())
            {
                if settle_canonical_chat_request_capture(state.store.as_ref(), hold, settlement)
                    .await
                    .is_err()
                {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to capture canonical account hold",
                    )
                        .into_response();
                }
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            if let Some(hold) = canonical_hold.as_ref() {
                if settle_canonical_chat_request_release(state.store.as_ref(), hold)
                    .await
                    .is_err()
                {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to release canonical account hold",
                    )
                        .into_response();
                }
            }
            return bad_gateway_openai_response("failed to relay upstream embedding");
        }
    }

    let canonical_settlement =
        canonical_usage_priced_settlement_amounts(canonical_hold.as_ref(), None);

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "embeddings",
        &request.model,
        canonical_settlement
            .as_ref()
            .map(|settlement| settlement.usage_units)
            .unwrap_or(EMBEDDINGS_HOLD_UNITS),
        canonical_settlement
            .as_ref()
            .map(|settlement| settlement.customer_charge)
            .unwrap_or(EMBEDDINGS_RETAIL_CHARGE),
    )
    .await
    .is_err()
    {
        if let Some(hold) = canonical_hold.as_ref() {
            let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
        }
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    if let (Some(hold), Some(settlement)) = (canonical_hold.as_ref(), canonical_settlement.as_ref())
    {
        if settle_canonical_chat_request_capture(state.store.as_ref(), hold, settlement)
            .await
            .is_err()
        {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to capture canonical account hold",
            )
                .into_response();
        }
    }

    Json(
        create_embedding(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        )
        .expect("embedding"),
    )
    .into_response()
}

async fn moderations_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Extension(request_id): Extension<RequestId>,
    ExtractJson(request): ExtractJson<CreateModerationRequest>,
) -> Response {
    let canonical_hold = match admit_canonical_moderation_request(
        state.store.as_ref(),
        &request_context,
        &request_id,
        &request,
    )
    .await
    {
        Ok(CanonicalChatAdmission::Held(hold)) => Some(hold),
        Ok(CanonicalChatAdmission::InsufficientBalance {
            account_id,
            shortfall_quantity,
        }) => {
            return account_balance_insufficient_response(account_id, shortfall_quantity);
        }
        Ok(CanonicalChatAdmission::NotApplicable) => None,
        Err(_) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to evaluate canonical account admission",
            )
                .into_response();
        }
    };

    match relay_moderation_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let canonical_settlement =
                canonical_usage_priced_settlement_amounts(canonical_hold.as_ref(), None);
            if record_gateway_usage_for_project_with_route_key_and_reference_id(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "moderations",
                &request.model,
                &request.model,
                canonical_settlement
                    .as_ref()
                    .map(|settlement| settlement.usage_units)
                    .unwrap_or(MODERATIONS_HOLD_UNITS),
                canonical_settlement
                    .as_ref()
                    .map(|settlement| settlement.customer_charge)
                    .unwrap_or(MODERATIONS_RETAIL_CHARGE),
                response_usage_id_or_single_data_item_id(&response),
            )
            .await
            .is_err()
            {
                if let Some(hold) = canonical_hold.as_ref() {
                    let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
                }
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            if let (Some(hold), Some(settlement)) =
                (canonical_hold.as_ref(), canonical_settlement.as_ref())
            {
                if settle_canonical_chat_request_capture(state.store.as_ref(), hold, settlement)
                    .await
                    .is_err()
                {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to capture canonical account hold",
                    )
                        .into_response();
                }
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            if let Some(hold) = canonical_hold.as_ref() {
                if settle_canonical_chat_request_release(state.store.as_ref(), hold)
                    .await
                    .is_err()
                {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to release canonical account hold",
                    )
                        .into_response();
                }
            }
            return bad_gateway_openai_response("failed to relay upstream moderation");
        }
    }

    let canonical_settlement =
        canonical_usage_priced_settlement_amounts(canonical_hold.as_ref(), None);

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "moderations",
        &request.model,
        canonical_settlement
            .as_ref()
            .map(|settlement| settlement.usage_units)
            .unwrap_or(MODERATIONS_HOLD_UNITS),
        canonical_settlement
            .as_ref()
            .map(|settlement| settlement.customer_charge)
            .unwrap_or(MODERATIONS_RETAIL_CHARGE),
    )
    .await
    .is_err()
    {
        if let Some(hold) = canonical_hold.as_ref() {
            let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
        }
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    if let (Some(hold), Some(settlement)) = (canonical_hold.as_ref(), canonical_settlement.as_ref())
    {
        if settle_canonical_chat_request_capture(state.store.as_ref(), hold, settlement)
            .await
            .is_err()
        {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to capture canonical account hold",
            )
                .into_response();
        }
    }

    Json(
        create_moderation(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        )
        .expect("moderation"),
    )
    .into_response()
}

async fn image_generations_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Extension(request_id): Extension<RequestId>,
    ExtractJson(request): ExtractJson<CreateImageRequest>,
) -> Response {
    let canonical_hold = match admit_canonical_request_priced_request(
        state.store.as_ref(),
        &request_context,
        &request_id,
        "images",
        &request.model,
        50,
    )
    .await
    {
        Ok(CanonicalChatAdmission::Held(hold)) => Some(hold),
        Ok(CanonicalChatAdmission::InsufficientBalance {
            account_id,
            shortfall_quantity,
        }) => {
            return account_balance_insufficient_response(account_id, shortfall_quantity);
        }
        Ok(CanonicalChatAdmission::NotApplicable) => None,
        Err(_) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to evaluate canonical account admission",
            )
                .into_response();
        }
    };
    match relay_image_generation_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_media_and_reference_id(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "images",
                &request.model,
                50,
                0.05,
                BillingMediaMetrics {
                    image_count: image_count_from_response(&response),
                    ..BillingMediaMetrics::default()
                },
                response_usage_id_or_single_data_item_id(&response),
            )
            .await
            .is_err()
            {
                if let Some(hold) = canonical_hold.as_ref() {
                    let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
                }
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            let canonical_settlement =
                canonical_usage_priced_settlement_amounts(canonical_hold.as_ref(), None);
            if let (Some(hold), Some(settlement)) =
                (canonical_hold.as_ref(), canonical_settlement.as_ref())
            {
                if settle_canonical_chat_request_capture(state.store.as_ref(), hold, settlement)
                    .await
                    .is_err()
                {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to capture canonical account hold",
                    )
                        .into_response();
                }
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            if let Some(hold) = canonical_hold.as_ref() {
                let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
            }
            return bad_gateway_openai_response("failed to relay upstream image generation");
        }
    }

    let response = create_image_generation(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
    )
    .expect("image");

    if record_gateway_usage_for_project_with_media_and_reference_id(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "images",
        &request.model,
        50,
        0.05,
        BillingMediaMetrics {
            image_count: u64::try_from(response.data.len()).unwrap_or(u64::MAX),
            ..BillingMediaMetrics::default()
        },
        None,
    )
    .await
    .is_err()
    {
        if let Some(hold) = canonical_hold.as_ref() {
            let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
        }
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    let canonical_settlement =
        canonical_usage_priced_settlement_amounts(canonical_hold.as_ref(), None);
    if let (Some(hold), Some(settlement)) = (canonical_hold.as_ref(), canonical_settlement.as_ref())
    {
        if settle_canonical_chat_request_capture(state.store.as_ref(), hold, settlement)
            .await
            .is_err()
        {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to capture canonical account hold",
            )
                .into_response();
        }
    }

    Json(response).into_response()
}

async fn image_edits_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    multipart: Multipart,
) -> Response {
    let request = match parse_image_edit_request(multipart).await {
        Ok(request) => request,
        Err(response) => return response,
    };
    let route_model = request.model_or_default().to_owned();

    match relay_image_edit_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_media_and_reference_id(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "images",
                &route_model,
                50,
                0.05,
                BillingMediaMetrics {
                    image_count: image_count_from_response(&response),
                    ..BillingMediaMetrics::default()
                },
                response_usage_id_or_single_data_item_id(&response),
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
            return bad_gateway_openai_response("failed to relay upstream image edit");
        }
    }

    let response = create_image_edit(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .expect("image edit");

    if record_gateway_usage_for_project_with_media_and_reference_id(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "images",
        &route_model,
        50,
        0.05,
        BillingMediaMetrics {
            image_count: u64::try_from(response.data.len()).unwrap_or(u64::MAX),
            ..BillingMediaMetrics::default()
        },
        None,
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

    Json(response).into_response()
}

async fn image_variations_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    multipart: Multipart,
) -> Response {
    let request = match parse_image_variation_request(multipart).await {
        Ok(request) => request,
        Err(response) => return response,
    };
    let route_model = request.model_or_default().to_owned();

    match relay_image_variation_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_media_and_reference_id(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "images",
                &route_model,
                50,
                0.05,
                BillingMediaMetrics {
                    image_count: image_count_from_response(&response),
                    ..BillingMediaMetrics::default()
                },
                response_usage_id_or_single_data_item_id(&response),
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
            return bad_gateway_openai_response("failed to relay upstream image variation");
        }
    }

    let response = create_image_variation(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .expect("image variation");

    if record_gateway_usage_for_project_with_media_and_reference_id(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "images",
        &route_model,
        50,
        0.05,
        BillingMediaMetrics {
            image_count: u64::try_from(response.data.len()).unwrap_or(u64::MAX),
            ..BillingMediaMetrics::default()
        },
        None,
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

    Json(response).into_response()
}

async fn transcriptions_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Extension(request_id): Extension<RequestId>,
    ExtractJson(request): ExtractJson<CreateTranscriptionRequest>,
) -> Response {
    let canonical_hold = match admit_canonical_request_priced_request(
        state.store.as_ref(),
        &request_context,
        &request_id,
        "audio_transcriptions",
        &request.model,
        25,
    )
    .await
    {
        Ok(CanonicalChatAdmission::Held(hold)) => Some(hold),
        Ok(CanonicalChatAdmission::InsufficientBalance {
            account_id,
            shortfall_quantity,
        }) => {
            return account_balance_insufficient_response(account_id, shortfall_quantity);
        }
        Ok(CanonicalChatAdmission::NotApplicable) => None,
        Err(_) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to evaluate canonical account admission",
            )
                .into_response();
        }
    };
    match relay_transcription_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "audio_transcriptions",
                &request.model,
                25,
                0.025,
            )
            .await
            .is_err()
            {
                if let Some(hold) = canonical_hold.as_ref() {
                    let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
                }
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            let canonical_settlement =
                canonical_usage_priced_settlement_amounts(canonical_hold.as_ref(), None);
            if let (Some(hold), Some(settlement)) =
                (canonical_hold.as_ref(), canonical_settlement.as_ref())
            {
                if settle_canonical_chat_request_capture(state.store.as_ref(), hold, settlement)
                    .await
                    .is_err()
                {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to capture canonical account hold",
                    )
                        .into_response();
                }
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            if let Some(hold) = canonical_hold.as_ref() {
                let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
            }
            return bad_gateway_openai_response("failed to relay upstream transcription");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "audio_transcriptions",
        &request.model,
        25,
        0.025,
    )
    .await
    .is_err()
    {
        if let Some(hold) = canonical_hold.as_ref() {
            let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
        }
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    let canonical_settlement =
        canonical_usage_priced_settlement_amounts(canonical_hold.as_ref(), None);
    if let (Some(hold), Some(settlement)) = (canonical_hold.as_ref(), canonical_settlement.as_ref())
    {
        if settle_canonical_chat_request_capture(state.store.as_ref(), hold, settlement)
            .await
            .is_err()
        {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to capture canonical account hold",
            )
                .into_response();
        }
    }

    Json(
        create_transcription(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        )
        .expect("transcription"),
    )
    .into_response()
}

async fn translations_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Extension(request_id): Extension<RequestId>,
    ExtractJson(request): ExtractJson<CreateTranslationRequest>,
) -> Response {
    let canonical_hold = match admit_canonical_request_priced_request(
        state.store.as_ref(),
        &request_context,
        &request_id,
        "audio_translations",
        &request.model,
        25,
    )
    .await
    {
        Ok(CanonicalChatAdmission::Held(hold)) => Some(hold),
        Ok(CanonicalChatAdmission::InsufficientBalance {
            account_id,
            shortfall_quantity,
        }) => {
            return account_balance_insufficient_response(account_id, shortfall_quantity);
        }
        Ok(CanonicalChatAdmission::NotApplicable) => None,
        Err(_) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to evaluate canonical account admission",
            )
                .into_response();
        }
    };
    match relay_translation_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "audio_translations",
                &request.model,
                25,
                0.025,
            )
            .await
            .is_err()
            {
                if let Some(hold) = canonical_hold.as_ref() {
                    let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
                }
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            let canonical_settlement =
                canonical_usage_priced_settlement_amounts(canonical_hold.as_ref(), None);
            if let (Some(hold), Some(settlement)) =
                (canonical_hold.as_ref(), canonical_settlement.as_ref())
            {
                if settle_canonical_chat_request_capture(state.store.as_ref(), hold, settlement)
                    .await
                    .is_err()
                {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to capture canonical account hold",
                    )
                        .into_response();
                }
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            if let Some(hold) = canonical_hold.as_ref() {
                let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
            }
            return bad_gateway_openai_response("failed to relay upstream translation");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "audio_translations",
        &request.model,
        25,
        0.025,
    )
    .await
    .is_err()
    {
        if let Some(hold) = canonical_hold.as_ref() {
            let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
        }
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    let canonical_settlement =
        canonical_usage_priced_settlement_amounts(canonical_hold.as_ref(), None);
    if let (Some(hold), Some(settlement)) = (canonical_hold.as_ref(), canonical_settlement.as_ref())
    {
        if settle_canonical_chat_request_capture(state.store.as_ref(), hold, settlement)
            .await
            .is_err()
        {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to capture canonical account hold",
            )
                .into_response();
        }
    }

    Json(
        create_translation(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        )
        .expect("translation"),
    )
    .into_response()
}

async fn audio_speech_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Extension(request_id): Extension<RequestId>,
    ExtractJson(request): ExtractJson<CreateSpeechRequest>,
) -> Response {
    let canonical_hold = match admit_canonical_request_priced_request(
        state.store.as_ref(),
        &request_context,
        &request_id,
        "audio_speech",
        &request.model,
        25,
    )
    .await
    {
        Ok(CanonicalChatAdmission::Held(hold)) => Some(hold),
        Ok(CanonicalChatAdmission::InsufficientBalance {
            account_id,
            shortfall_quantity,
        }) => {
            return account_balance_insufficient_response(account_id, shortfall_quantity);
        }
        Ok(CanonicalChatAdmission::NotApplicable) => None,
        Err(_) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to evaluate canonical account admission",
            )
                .into_response();
        }
    };
    match relay_speech_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "audio_speech",
                &request.model,
                25,
                0.025,
            )
            .await
            .is_err()
            {
                if let Some(hold) = canonical_hold.as_ref() {
                    let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
                }
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            let canonical_settlement =
                canonical_usage_priced_settlement_amounts(canonical_hold.as_ref(), None);
            if let (Some(hold), Some(settlement)) =
                (canonical_hold.as_ref(), canonical_settlement.as_ref())
            {
                if settle_canonical_chat_request_capture(state.store.as_ref(), hold, settlement)
                    .await
                    .is_err()
                {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to capture canonical account hold",
                    )
                        .into_response();
                }
            }

            return upstream_passthrough_response(response);
        }
        Ok(None) => {}
        Err(_) => {
            if let Some(hold) = canonical_hold.as_ref() {
                let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
            }
            return bad_gateway_openai_response("failed to relay upstream speech");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "audio_speech",
        &request.model,
        25,
        0.025,
    )
    .await
    .is_err()
    {
        if let Some(hold) = canonical_hold.as_ref() {
            let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
        }
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    let canonical_settlement =
        canonical_usage_priced_settlement_amounts(canonical_hold.as_ref(), None);
    if let (Some(hold), Some(settlement)) = (canonical_hold.as_ref(), canonical_settlement.as_ref())
    {
        if settle_canonical_chat_request_capture(state.store.as_ref(), hold, settlement)
            .await
            .is_err()
        {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to capture canonical account hold",
            )
                .into_response();
        }
    }

    local_speech_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
}

async fn audio_voices_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Response {
    match relay_audio_voices_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "audio",
                "voices",
                5,
                0.005,
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
            return bad_gateway_openai_response("failed to relay upstream audio voices list");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "audio",
        "voices",
        5,
        0.005,
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
        list_audio_voices(request_context.tenant_id(), request_context.project_id())
            .expect("audio voices list"),
    )
    .into_response()
}

async fn audio_voice_consents_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateVoiceConsentRequest>,
) -> Response {
    match relay_audio_voice_consent_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let consent_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(request.voice.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "audio",
                &request.voice,
                consent_id,
                5,
                0.005,
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
            return bad_gateway_openai_response("failed to relay upstream audio voice consent");
        }
    }

    let response = create_audio_voice_consent(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .expect("audio voice consent");

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "audio",
        &request.voice,
        response.id.as_str(),
        5,
        0.005,
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

    Json(response).into_response()
}

async fn files_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
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
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let file_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(request.purpose.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "files",
                &request.purpose,
                file_id,
                5,
                0.005,
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
            return bad_gateway_openai_response("failed to relay upstream file");
        }
    }

    let response = create_file(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .expect("file");

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "files",
        &request.purpose,
        response.id.as_str(),
        5,
        0.005,
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

    Json(response).into_response()
}

async fn files_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Response {
    match relay_list_files_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "files",
                "list",
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
            return bad_gateway_openai_response("failed to relay upstream files list");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "files",
        "list",
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

    Json(list_files(request_context.tenant_id(), request_context.project_id()).expect("files list"))
        .into_response()
}

async fn file_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(file_id): Path<String>,
) -> Response {
    match relay_get_file_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &file_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "files",
                &file_id,
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
            return bad_gateway_openai_response("failed to relay upstream file retrieve");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "files",
        &file_id,
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

    Json(
        get_file(
            request_context.tenant_id(),
            request_context.project_id(),
            &file_id,
        )
        .expect("file retrieve"),
    )
    .into_response()
}

async fn file_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(file_id): Path<String>,
) -> Response {
    match relay_delete_file_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &file_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "files",
                &file_id,
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
            return bad_gateway_openai_response("failed to relay upstream file delete");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "files",
        &file_id,
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

    Json(
        delete_file(
            request_context.tenant_id(),
            request_context.project_id(),
            &file_id,
        )
        .expect("file delete"),
    )
    .into_response()
}

async fn file_content_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(file_id): Path<String>,
) -> Response {
    match relay_file_content_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &file_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "files",
                &file_id,
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

            return upstream_passthrough_response(response);
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream file content");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "files",
        &file_id,
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

    local_file_content_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &file_id,
    )
}

async fn containers_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateContainerRequest>,
) -> Response {
    match sdkwork_api_app_gateway::relay_container_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let container_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(request.name.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "containers",
                &request.name,
                container_id,
                10,
                0.01,
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
            return bad_gateway_openai_response("failed to relay upstream container create");
        }
    }

    let response = sdkwork_api_app_gateway::create_container(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .expect("container");

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "containers",
        &request.name,
        response.id.as_str(),
        10,
        0.01,
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

    Json(response).into_response()
}

async fn containers_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Response {
    match sdkwork_api_app_gateway::relay_list_containers_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "containers",
                "containers",
                5,
                0.005,
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
            return bad_gateway_openai_response("failed to relay upstream containers list");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "containers",
        "containers",
        5,
        0.005,
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
        sdkwork_api_app_gateway::list_containers(
            request_context.tenant_id(),
            request_context.project_id(),
        )
        .expect("containers list"),
    )
    .into_response()
}

async fn container_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(container_id): Path<String>,
) -> Response {
    match sdkwork_api_app_gateway::relay_get_container_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "containers",
                &container_id,
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
            return bad_gateway_openai_response("failed to relay upstream container retrieve");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "containers",
        &container_id,
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

    Json(
        sdkwork_api_app_gateway::get_container(
            request_context.tenant_id(),
            request_context.project_id(),
            &container_id,
        )
        .expect("container retrieve"),
    )
    .into_response()
}

async fn container_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(container_id): Path<String>,
) -> Response {
    match sdkwork_api_app_gateway::relay_delete_container_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "containers",
                &container_id,
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
            return bad_gateway_openai_response("failed to relay upstream container delete");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "containers",
        &container_id,
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

    Json(
        sdkwork_api_app_gateway::delete_container(
            request_context.tenant_id(),
            request_context.project_id(),
            &container_id,
        )
        .expect("container delete"),
    )
    .into_response()
}

async fn container_files_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(container_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateContainerFileRequest>,
) -> Response {
    match sdkwork_api_app_gateway::relay_container_file_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_model = response_usage_id_or_single_data_item_id(&response)
                .unwrap_or(request.file_id.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "containers",
                &container_id,
                usage_model,
                8,
                0.008,
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
            return bad_gateway_openai_response("failed to relay upstream container file create");
        }
    }

    let response = sdkwork_api_app_gateway::create_container_file(
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
        &request,
    )
    .expect("container file");

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "containers",
        &container_id,
        response.id.as_str(),
        8,
        0.008,
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

    Json(response).into_response()
}

async fn container_files_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(container_id): Path<String>,
) -> Response {
    match sdkwork_api_app_gateway::relay_list_container_files_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "containers",
                &container_id,
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
            return bad_gateway_openai_response("failed to relay upstream container files list");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "containers",
        &container_id,
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

    Json(
        sdkwork_api_app_gateway::list_container_files(
            request_context.tenant_id(),
            request_context.project_id(),
            &container_id,
        )
        .expect("container files list"),
    )
    .into_response()
}

async fn container_file_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((container_id, file_id)): Path<(String, String)>,
) -> Response {
    match sdkwork_api_app_gateway::relay_get_container_file_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
        &file_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "containers",
                &container_id,
                &file_id,
                3,
                0.003,
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
            return bad_gateway_openai_response("failed to relay upstream container file retrieve");
        }
    }

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "containers",
        &container_id,
        &file_id,
        3,
        0.003,
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
        sdkwork_api_app_gateway::get_container_file(
            request_context.tenant_id(),
            request_context.project_id(),
            &container_id,
            &file_id,
        )
        .expect("container file retrieve"),
    )
    .into_response()
}

async fn container_file_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((container_id, file_id)): Path<(String, String)>,
) -> Response {
    match sdkwork_api_app_gateway::relay_delete_container_file_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
        &file_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "containers",
                &container_id,
                &file_id,
                3,
                0.003,
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
            return bad_gateway_openai_response("failed to relay upstream container file delete");
        }
    }

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "containers",
        &container_id,
        &file_id,
        3,
        0.003,
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
        sdkwork_api_app_gateway::delete_container_file(
            request_context.tenant_id(),
            request_context.project_id(),
            &container_id,
            &file_id,
        )
        .expect("container file delete"),
    )
    .into_response()
}

async fn container_file_content_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((container_id, file_id)): Path<(String, String)>,
) -> Response {
    match sdkwork_api_app_gateway::relay_container_file_content_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
        &file_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "containers",
                &container_id,
                &file_id,
                3,
                0.003,
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
            return bad_gateway_openai_response("failed to relay upstream container file content");
        }
    }

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "containers",
        &container_id,
        &file_id,
        3,
        0.003,
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

    local_container_file_content_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
        &file_id,
    )
}

async fn music_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Extension(request_id): Extension<RequestId>,
    ExtractJson(request): ExtractJson<CreateMusicRequest>,
) -> Response {
    let canonical_hold = match admit_canonical_music_request(
        state.store.as_ref(),
        &request_context,
        &request_id,
        &request,
    )
    .await
    {
        Ok(CanonicalChatAdmission::Held(hold)) => Some(hold),
        Ok(CanonicalChatAdmission::InsufficientBalance {
            account_id,
            shortfall_quantity,
        }) => {
            return account_balance_insufficient_response(account_id, shortfall_quantity);
        }
        Ok(CanonicalChatAdmission::NotApplicable) => None,
        Err(_) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to evaluate canonical account admission",
            )
                .into_response();
        }
    };

    let response = match relay_music_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => response,
        Ok(None) => serde_json::to_value(
            create_music(
                request_context.tenant_id(),
                request_context.project_id(),
                &request,
            )
            .expect("music create"),
        )
        .unwrap_or(Value::Null),
        Err(_) => {
            if let Some(hold) = canonical_hold.as_ref() {
                let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
            }
            return bad_gateway_openai_response("failed to relay upstream music create");
        }
    };

    let music_reference_id = response_usage_id_or_single_data_item_id(&response).map(str::to_owned);
    let completed_music_seconds = completed_music_seconds_from_response(&response);

    if let Some(hold) = canonical_hold.as_ref() {
        match music_reference_id.as_deref() {
            Some(music_reference_id) => {
                if annotate_canonical_request_upstream_reference(
                    state.store.as_ref(),
                    hold,
                    music_reference_id,
                    RequestStatus::Running,
                )
                .await
                .is_err()
                {
                    let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to annotate canonical music request",
                    )
                        .into_response();
                }
            }
            None if completed_music_seconds.is_none() => {
                let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
                return bad_gateway_openai_response(
                    "failed to relay upstream music create without reference id",
                );
            }
            None => {}
        }
    }

    if completed_music_seconds.is_some() || canonical_hold.is_none() {
        let music_seconds = completed_music_seconds.unwrap_or_else(|| {
            request
                .duration_seconds
                .unwrap_or_else(|| music_seconds_from_response(&response))
        });
        let settlement = canonical_hold
            .as_ref()
            .and_then(|hold| {
                canonical_music_settlement_amounts_from_model_price(
                    &hold.model_price,
                    music_seconds,
                )
            })
            .unwrap_or(CanonicalChatSettlementAmounts {
                usage_units: music_billing_units(music_seconds),
                customer_charge: music_billing_amount(music_seconds),
            });
        if record_gateway_usage_for_project_with_media_and_reference_id(
            state.store.as_ref(),
            request_context.tenant_id(),
            request_context.project_id(),
            "music",
            &request.model,
            settlement.usage_units,
            settlement.customer_charge,
            BillingMediaMetrics {
                music_seconds,
                ..BillingMediaMetrics::default()
            },
            music_reference_id.as_deref(),
        )
        .await
        .is_err()
        {
            if let Some(hold) = canonical_hold.as_ref() {
                let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
            }
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to record usage",
            )
                .into_response();
        }

        if let Some(hold) = canonical_hold.as_ref() {
            let canonical_settlement = canonical_music_settlement_amounts_from_model_price(
                &hold.model_price,
                music_seconds,
            )
            .unwrap_or(CanonicalChatSettlementAmounts {
                usage_units: hold.estimated_usage.total_tokens.max(1),
                customer_charge: hold.estimated_quantity,
            });
            if settle_canonical_chat_request_capture(
                state.store.as_ref(),
                hold,
                &canonical_settlement,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to capture canonical account hold",
                )
                    .into_response();
            }
        }
    }

    Json(response).into_response()
}

async fn music_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Response {
    let response = match relay_list_music_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
    )
    .await
    {
        Ok(Some(response)) => response,
        Ok(None) => serde_json::to_value(
            list_music(request_context.tenant_id(), request_context.project_id())
                .expect("music list"),
        )
        .unwrap_or(Value::Null),
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream music list");
        }
    };

    let response_value = serde_json::to_value(&response).unwrap_or(Value::Null);
    if let Err(_error) = reconcile_pending_music_response(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        &response_value,
    )
    .await
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to reconcile pending music settlement",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn music_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(music_id): Path<String>,
) -> Response {
    let response = match relay_get_music_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &music_id,
    )
    .await
    {
        Ok(Some(response)) => response,
        Ok(None) => serde_json::to_value(
            get_music(
                request_context.tenant_id(),
                request_context.project_id(),
                &music_id,
            )
            .expect("music retrieve"),
        )
        .unwrap_or(Value::Null),
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream music retrieve");
        }
    };

    let response_value = serde_json::to_value(&response).unwrap_or(Value::Null);
    if let Err(_error) = reconcile_pending_music_response(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        &response_value,
    )
    .await
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to reconcile pending music settlement",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn music_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(music_id): Path<String>,
) -> Response {
    match relay_delete_music_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &music_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "music",
                &music_id,
                10,
                0.01,
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
            return bad_gateway_openai_response("failed to relay upstream music delete");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "music",
        &music_id,
        10,
        0.01,
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
        delete_music(
            request_context.tenant_id(),
            request_context.project_id(),
            &music_id,
        )
        .expect("music delete"),
    )
    .into_response()
}

async fn music_content_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(music_id): Path<String>,
) -> Response {
    match relay_music_content_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &music_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "music",
                &music_id,
                10,
                0.01,
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
            return bad_gateway_openai_response("failed to relay upstream music content");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "music",
        &music_id,
        10,
        0.01,
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

    local_music_content_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &music_id,
    )
}

async fn music_lyrics_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateMusicLyricsRequest>,
) -> Response {
    match relay_music_lyrics_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_model = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("lyrics");
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "music",
                "lyrics",
                usage_model,
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
            return bad_gateway_openai_response("failed to relay upstream music lyrics");
        }
    }

    let response = create_music_lyrics(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .expect("music lyrics");

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "music",
        "lyrics",
        &response.id,
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

    Json(response).into_response()
}

async fn videos_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Extension(request_id): Extension<RequestId>,
    ExtractJson(request): ExtractJson<CreateVideoRequest>,
) -> Response {
    let canonical_hold = match admit_canonical_video_request(
        state.store.as_ref(),
        &request_context,
        &request_id,
        &request.model,
    )
    .await
    {
        Ok(CanonicalChatAdmission::Held(hold)) => Some(hold),
        Ok(CanonicalChatAdmission::InsufficientBalance {
            account_id,
            shortfall_quantity,
        }) => {
            return account_balance_insufficient_response(account_id, shortfall_quantity);
        }
        Ok(CanonicalChatAdmission::NotApplicable) => None,
        Err(_) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to evaluate canonical account admission",
            )
                .into_response();
        }
    };

    let response = match relay_video_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => response,
        Ok(None) => serde_json::to_value(
            create_video(
                request_context.tenant_id(),
                request_context.project_id(),
                &request.model,
                &request.prompt,
            )
            .expect("video"),
        )
        .unwrap_or(Value::Null),
        Err(_) => {
            if let Some(hold) = canonical_hold.as_ref() {
                let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
            }
            return bad_gateway_openai_response("failed to relay upstream video create");
        }
    };

    let usage_model =
        response_usage_id_or_single_data_item_id(&response).unwrap_or(request.model.as_str());
    let video_reference_id = response_usage_id_or_single_data_item_id(&response).map(str::to_owned);
    let completed_video_seconds = completed_video_seconds_from_response(&response);

    if let Some(hold) = canonical_hold.as_ref() {
        match video_reference_id.as_deref() {
            Some(video_reference_id) => {
                if annotate_canonical_request_upstream_reference(
                    state.store.as_ref(),
                    hold,
                    video_reference_id,
                    RequestStatus::Running,
                )
                .await
                .is_err()
                {
                    let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to annotate canonical video request",
                    )
                        .into_response();
                }
            }
            None if completed_video_seconds.is_none() => {
                let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
                return bad_gateway_openai_response(
                    "failed to relay upstream video create without reference id",
                );
            }
            None => {}
        }
    }

    if completed_video_seconds.is_some() || canonical_hold.is_none() {
        if let Some(video_seconds) = completed_video_seconds {
            let settlement = canonical_hold
                .as_ref()
                .and_then(|hold| {
                    canonical_video_settlement_amounts_from_model_price(
                        &hold.model_price,
                        video_seconds,
                    )
                })
                .unwrap_or(CanonicalChatSettlementAmounts {
                    usage_units: (video_seconds / 60.0).ceil().max(1.0) as u64,
                    customer_charge: 0.09,
                });
            if record_gateway_usage_for_project_with_media_and_reference_id(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "videos",
                &request.model,
                settlement.usage_units,
                settlement.customer_charge,
                BillingMediaMetrics {
                    video_seconds,
                    ..BillingMediaMetrics::default()
                },
                video_reference_id.as_deref(),
            )
            .await
            .is_err()
            {
                if let Some(hold) = canonical_hold.as_ref() {
                    let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
                }
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            if let Some(hold) = canonical_hold.as_ref() {
                let canonical_settlement = canonical_video_settlement_amounts_from_model_price(
                    &hold.model_price,
                    video_seconds,
                )
                .unwrap_or(CanonicalChatSettlementAmounts {
                    usage_units: hold.estimated_usage.total_tokens.max(1),
                    customer_charge: hold.estimated_quantity,
                });
                if settle_canonical_chat_request_capture(
                    state.store.as_ref(),
                    hold,
                    &canonical_settlement,
                )
                .await
                .is_err()
                {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to capture canonical account hold",
                    )
                        .into_response();
                }
            }
        } else if record_gateway_usage_for_project_with_route_key(
            state.store.as_ref(),
            request_context.tenant_id(),
            request_context.project_id(),
            "videos",
            &request.model,
            usage_model,
            90,
            0.09,
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
    }

    Json(response).into_response()
}

async fn videos_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Response {
    let response = match relay_list_videos_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
    )
    .await
    {
        Ok(Some(response)) => response,
        Ok(None) => serde_json::to_value(
            list_videos(request_context.tenant_id(), request_context.project_id())
                .expect("videos list"),
        )
        .unwrap_or(Value::Null),
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream videos list");
        }
    };

    let response_value = serde_json::to_value(&response).unwrap_or(Value::Null);
    if let Err(_error) = reconcile_pending_video_response(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        &response_value,
    )
    .await
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to reconcile pending video settlement",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn video_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(video_id): Path<String>,
) -> Response {
    let response = match relay_get_video_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
    )
    .await
    {
        Ok(Some(response)) => response,
        Ok(None) => serde_json::to_value(
            get_video(
                request_context.tenant_id(),
                request_context.project_id(),
                &video_id,
            )
            .expect("video retrieve"),
        )
        .unwrap_or(Value::Null),
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video retrieve");
        }
    };

    let response_value = serde_json::to_value(&response).unwrap_or(Value::Null);
    if let Err(_error) = reconcile_pending_video_response(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        &response_value,
    )
    .await
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to reconcile pending video settlement",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn video_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(video_id): Path<String>,
) -> Response {
    match relay_delete_video_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "videos",
                &video_id,
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
            return bad_gateway_openai_response("failed to relay upstream video delete");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        &video_id,
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
        delete_video(
            request_context.tenant_id(),
            request_context.project_id(),
            &video_id,
        )
        .expect("video delete"),
    )
    .into_response()
}

async fn video_content_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(video_id): Path<String>,
) -> Response {
    match relay_video_content_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "videos",
                &video_id,
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

            return upstream_passthrough_response(response);
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video content");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        &video_id,
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

    local_video_content_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
    )
}

async fn video_remix_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Extension(request_id): Extension<RequestId>,
    Path(video_id): Path<String>,
    ExtractJson(request): ExtractJson<RemixVideoRequest>,
) -> Response {
    let canonical_hold = match admit_canonical_video_request_for_route(
        state.store.as_ref(),
        &request_context,
        &request_id,
        &video_id,
        None,
    )
    .await
    {
        Ok(CanonicalChatAdmission::Held(hold)) => Some(hold),
        Ok(CanonicalChatAdmission::InsufficientBalance {
            account_id,
            shortfall_quantity,
        }) => {
            return account_balance_insufficient_response(account_id, shortfall_quantity);
        }
        Ok(CanonicalChatAdmission::NotApplicable) => None,
        Err(_) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to evaluate canonical account admission",
            )
                .into_response();
        }
    };

    match relay_remix_video_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_model =
                response_usage_id_or_single_data_item_id(&response).unwrap_or(video_id.as_str());
            let video_reference_id =
                response_usage_id_or_single_data_item_id(&response).map(str::to_owned);
            let completed_video_seconds = completed_video_seconds_from_response(&response);

            if let Some(hold) = canonical_hold.as_ref() {
                match video_reference_id.as_deref() {
                    Some(video_reference_id) => {
                        if annotate_canonical_request_upstream_reference(
                            state.store.as_ref(),
                            hold,
                            video_reference_id,
                            RequestStatus::Running,
                        )
                        .await
                        .is_err()
                        {
                            let _ =
                                settle_canonical_chat_request_release(state.store.as_ref(), hold)
                                    .await;
                            return (
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                "failed to annotate canonical video request",
                            )
                                .into_response();
                        }
                    }
                    None if completed_video_seconds.is_none() => {
                        let _ =
                            settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
                        return bad_gateway_openai_response(
                            "failed to relay upstream video remix without reference id",
                        );
                    }
                    None => {}
                }
            }

            if completed_video_seconds.is_some() || canonical_hold.is_none() {
                if let Some(video_seconds) = completed_video_seconds {
                    if let Some(hold) = canonical_hold.as_ref() {
                        let canonical_settlement =
                            canonical_video_settlement_amounts_from_model_price(
                                &hold.model_price,
                                video_seconds,
                            )
                            .unwrap_or(
                                CanonicalChatSettlementAmounts {
                                    usage_units: hold.estimated_usage.total_tokens.max(1),
                                    customer_charge: hold.estimated_quantity,
                                },
                            );
                        if record_gateway_usage_for_project_with_media_and_reference_id(
                            state.store.as_ref(),
                            request_context.tenant_id(),
                            request_context.project_id(),
                            "videos",
                            &hold.model_price.model_id,
                            canonical_settlement.usage_units,
                            canonical_settlement.customer_charge,
                            BillingMediaMetrics {
                                video_seconds,
                                ..BillingMediaMetrics::default()
                            },
                            video_reference_id.as_deref(),
                        )
                        .await
                        .is_err()
                        {
                            let _ =
                                settle_canonical_chat_request_release(state.store.as_ref(), hold)
                                    .await;
                            return (
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                "failed to record usage",
                            )
                                .into_response();
                        }
                        if settle_canonical_chat_request_capture(
                            state.store.as_ref(),
                            hold,
                            &canonical_settlement,
                        )
                        .await
                        .is_err()
                        {
                            return (
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                "failed to capture canonical account hold",
                            )
                                .into_response();
                        }
                    } else if record_gateway_usage_for_project_with_route_key(
                        state.store.as_ref(),
                        request_context.tenant_id(),
                        request_context.project_id(),
                        "videos",
                        &video_id,
                        usage_model,
                        60,
                        0.06,
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
                } else if record_gateway_usage_for_project_with_route_key(
                    state.store.as_ref(),
                    request_context.tenant_id(),
                    request_context.project_id(),
                    "videos",
                    &video_id,
                    usage_model,
                    60,
                    0.06,
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
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            if let Some(hold) = canonical_hold.as_ref() {
                let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
            }
            return bad_gateway_openai_response("failed to relay upstream video remix");
        }
    }

    let response = remix_video(
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
        &request.prompt,
    )
    .expect("video remix");
    let usage_model = match response.data.as_slice() {
        [item] => item.id.as_str(),
        _ => video_id.as_str(),
    };
    let response_value = serde_json::to_value(&response).unwrap_or(Value::Null);
    let video_reference_id =
        response_usage_id_or_single_data_item_id(&response_value).map(str::to_owned);
    let completed_video_seconds = completed_video_seconds_from_response(&response_value);

    if let Some(hold) = canonical_hold.as_ref() {
        match video_reference_id.as_deref() {
            Some(video_reference_id) => {
                if annotate_canonical_request_upstream_reference(
                    state.store.as_ref(),
                    hold,
                    video_reference_id,
                    RequestStatus::Running,
                )
                .await
                .is_err()
                {
                    let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to annotate canonical video request",
                    )
                        .into_response();
                }
            }
            None if completed_video_seconds.is_none() => {
                let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
                return bad_gateway_openai_response(
                    "failed to relay upstream video remix without reference id",
                );
            }
            None => {}
        }
    }

    if completed_video_seconds.is_some() || canonical_hold.is_none() {
        if let Some(video_seconds) = completed_video_seconds {
            if let Some(hold) = canonical_hold.as_ref() {
                let canonical_settlement = canonical_video_settlement_amounts_from_model_price(
                    &hold.model_price,
                    video_seconds,
                )
                .unwrap_or(CanonicalChatSettlementAmounts {
                    usage_units: hold.estimated_usage.total_tokens.max(1),
                    customer_charge: hold.estimated_quantity,
                });
                if record_gateway_usage_for_project_with_media_and_reference_id(
                    state.store.as_ref(),
                    request_context.tenant_id(),
                    request_context.project_id(),
                    "videos",
                    &hold.model_price.model_id,
                    canonical_settlement.usage_units,
                    canonical_settlement.customer_charge,
                    BillingMediaMetrics {
                        video_seconds,
                        ..BillingMediaMetrics::default()
                    },
                    video_reference_id.as_deref(),
                )
                .await
                .is_err()
                {
                    let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to record usage",
                    )
                        .into_response();
                }
                if settle_canonical_chat_request_capture(
                    state.store.as_ref(),
                    hold,
                    &canonical_settlement,
                )
                .await
                .is_err()
                {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to capture canonical account hold",
                    )
                        .into_response();
                }
            } else if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "videos",
                &video_id,
                usage_model,
                60,
                0.06,
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
        } else if record_gateway_usage_for_project_with_route_key(
            state.store.as_ref(),
            request_context.tenant_id(),
            request_context.project_id(),
            "videos",
            &video_id,
            usage_model,
            60,
            0.06,
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
    }

    Json(response).into_response()
}

async fn video_characters_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(video_id): Path<String>,
) -> Response {
    match relay_list_video_characters_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "videos",
                &video_id,
                60,
                0.06,
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
            return bad_gateway_openai_response("failed to relay upstream video characters list");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        &video_id,
        60,
        0.06,
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
        list_video_characters(
            request_context.tenant_id(),
            request_context.project_id(),
            &video_id,
        )
        .expect("video characters list"),
    )
    .into_response()
}

async fn video_character_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((video_id, character_id)): Path<(String, String)>,
) -> Response {
    match relay_get_video_character_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
        &character_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "videos",
                &video_id,
                &character_id,
                60,
                0.06,
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
            return bad_gateway_openai_response(
                "failed to relay upstream video character retrieve",
            );
        }
    }

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        &video_id,
        &character_id,
        60,
        0.06,
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
        get_video_character(
            request_context.tenant_id(),
            request_context.project_id(),
            &video_id,
            &character_id,
        )
        .expect("video character retrieve"),
    )
    .into_response()
}

async fn video_character_update_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((video_id, character_id)): Path<(String, String)>,
    ExtractJson(request): ExtractJson<UpdateVideoCharacterRequest>,
) -> Response {
    match relay_update_video_character_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
        &character_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "videos",
                &video_id,
                &character_id,
                60,
                0.06,
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
            return bad_gateway_openai_response("failed to relay upstream video character update");
        }
    }

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        &video_id,
        &character_id,
        60,
        0.06,
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
        update_video_character(
            request_context.tenant_id(),
            request_context.project_id(),
            &video_id,
            &character_id,
            &request,
        )
        .expect("video character update"),
    )
    .into_response()
}

async fn video_extend_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Extension(request_id): Extension<RequestId>,
    Path(video_id): Path<String>,
    ExtractJson(request): ExtractJson<ExtendVideoRequest>,
) -> Response {
    let canonical_hold = match admit_canonical_video_request_for_route(
        state.store.as_ref(),
        &request_context,
        &request_id,
        &video_id,
        None,
    )
    .await
    {
        Ok(CanonicalChatAdmission::Held(hold)) => Some(hold),
        Ok(CanonicalChatAdmission::InsufficientBalance {
            account_id,
            shortfall_quantity,
        }) => {
            return account_balance_insufficient_response(account_id, shortfall_quantity);
        }
        Ok(CanonicalChatAdmission::NotApplicable) => None,
        Err(_) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to evaluate canonical account admission",
            )
                .into_response();
        }
    };

    match relay_extend_video_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_model =
                response_usage_id_or_single_data_item_id(&response).unwrap_or(video_id.as_str());
            let video_reference_id =
                response_usage_id_or_single_data_item_id(&response).map(str::to_owned);
            let completed_video_seconds = completed_video_seconds_from_response(&response);

            if let Some(hold) = canonical_hold.as_ref() {
                match video_reference_id.as_deref() {
                    Some(video_reference_id) => {
                        if annotate_canonical_request_upstream_reference(
                            state.store.as_ref(),
                            hold,
                            video_reference_id,
                            RequestStatus::Running,
                        )
                        .await
                        .is_err()
                        {
                            let _ =
                                settle_canonical_chat_request_release(state.store.as_ref(), hold)
                                    .await;
                            return (
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                "failed to annotate canonical video request",
                            )
                                .into_response();
                        }
                    }
                    None if completed_video_seconds.is_none() => {
                        let _ =
                            settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
                        return bad_gateway_openai_response(
                            "failed to relay upstream video extend without reference id",
                        );
                    }
                    None => {}
                }
            }

            if completed_video_seconds.is_some() || canonical_hold.is_none() {
                if let Some(video_seconds) = completed_video_seconds {
                    if let Some(hold) = canonical_hold.as_ref() {
                        let canonical_settlement =
                            canonical_video_settlement_amounts_from_model_price(
                                &hold.model_price,
                                video_seconds,
                            )
                            .unwrap_or(
                                CanonicalChatSettlementAmounts {
                                    usage_units: hold.estimated_usage.total_tokens.max(1),
                                    customer_charge: hold.estimated_quantity,
                                },
                            );
                        if record_gateway_usage_for_project_with_media_and_reference_id(
                            state.store.as_ref(),
                            request_context.tenant_id(),
                            request_context.project_id(),
                            "videos",
                            &hold.model_price.model_id,
                            canonical_settlement.usage_units,
                            canonical_settlement.customer_charge,
                            BillingMediaMetrics {
                                video_seconds,
                                ..BillingMediaMetrics::default()
                            },
                            video_reference_id.as_deref(),
                        )
                        .await
                        .is_err()
                        {
                            let _ =
                                settle_canonical_chat_request_release(state.store.as_ref(), hold)
                                    .await;
                            return (
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                "failed to record usage",
                            )
                                .into_response();
                        }
                        if settle_canonical_chat_request_capture(
                            state.store.as_ref(),
                            hold,
                            &canonical_settlement,
                        )
                        .await
                        .is_err()
                        {
                            return (
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                "failed to capture canonical account hold",
                            )
                                .into_response();
                        }
                    } else if record_gateway_usage_for_project_with_route_key(
                        state.store.as_ref(),
                        request_context.tenant_id(),
                        request_context.project_id(),
                        "videos",
                        &video_id,
                        usage_model,
                        60,
                        0.06,
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
                } else if record_gateway_usage_for_project_with_route_key(
                    state.store.as_ref(),
                    request_context.tenant_id(),
                    request_context.project_id(),
                    "videos",
                    &video_id,
                    usage_model,
                    60,
                    0.06,
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
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            if let Some(hold) = canonical_hold.as_ref() {
                let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
            }
            return bad_gateway_openai_response("failed to relay upstream video extend");
        }
    }

    let response = extend_video(
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
        &request.prompt,
    )
    .expect("video extend");
    let usage_model = match response.data.as_slice() {
        [item] => item.id.as_str(),
        _ => video_id.as_str(),
    };
    let response_value = serde_json::to_value(&response).unwrap_or(Value::Null);
    let video_reference_id =
        response_usage_id_or_single_data_item_id(&response_value).map(str::to_owned);
    let completed_video_seconds = completed_video_seconds_from_response(&response_value);

    if let Some(hold) = canonical_hold.as_ref() {
        match video_reference_id.as_deref() {
            Some(video_reference_id) => {
                if annotate_canonical_request_upstream_reference(
                    state.store.as_ref(),
                    hold,
                    video_reference_id,
                    RequestStatus::Running,
                )
                .await
                .is_err()
                {
                    let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to annotate canonical video request",
                    )
                        .into_response();
                }
            }
            None if completed_video_seconds.is_none() => {
                let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
                return bad_gateway_openai_response(
                    "failed to relay upstream video extend without reference id",
                );
            }
            None => {}
        }
    }

    if completed_video_seconds.is_some() || canonical_hold.is_none() {
        if let Some(video_seconds) = completed_video_seconds {
            if let Some(hold) = canonical_hold.as_ref() {
                let canonical_settlement = canonical_video_settlement_amounts_from_model_price(
                    &hold.model_price,
                    video_seconds,
                )
                .unwrap_or(CanonicalChatSettlementAmounts {
                    usage_units: hold.estimated_usage.total_tokens.max(1),
                    customer_charge: hold.estimated_quantity,
                });
                if record_gateway_usage_for_project_with_media_and_reference_id(
                    state.store.as_ref(),
                    request_context.tenant_id(),
                    request_context.project_id(),
                    "videos",
                    &hold.model_price.model_id,
                    canonical_settlement.usage_units,
                    canonical_settlement.customer_charge,
                    BillingMediaMetrics {
                        video_seconds,
                        ..BillingMediaMetrics::default()
                    },
                    video_reference_id.as_deref(),
                )
                .await
                .is_err()
                {
                    let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to record usage",
                    )
                        .into_response();
                }
                if settle_canonical_chat_request_capture(
                    state.store.as_ref(),
                    hold,
                    &canonical_settlement,
                )
                .await
                .is_err()
                {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to capture canonical account hold",
                    )
                        .into_response();
                }
            } else if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "videos",
                &video_id,
                usage_model,
                60,
                0.06,
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
        } else if record_gateway_usage_for_project_with_route_key(
            state.store.as_ref(),
            request_context.tenant_id(),
            request_context.project_id(),
            "videos",
            &video_id,
            usage_model,
            60,
            0.06,
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
    }

    Json(response).into_response()
}

async fn video_character_create_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateVideoCharacterRequest>,
) -> Response {
    match sdkwork_api_app_gateway::relay_create_video_character_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let character_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(request.video_id.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "videos",
                &request.video_id,
                character_id,
                40,
                0.04,
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
            return bad_gateway_openai_response("failed to relay upstream video character create");
        }
    }

    let response = sdkwork_api_app_gateway::create_video_character(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .expect("video character create");

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        &request.video_id,
        response.id.as_str(),
        40,
        0.04,
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

    Json(response).into_response()
}

async fn video_character_retrieve_canonical_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(character_id): Path<String>,
) -> Response {
    match sdkwork_api_app_gateway::relay_get_video_character_canonical_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &character_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "videos",
                &character_id,
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
            return bad_gateway_openai_response(
                "failed to relay upstream video character retrieve",
            );
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        &character_id,
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
        sdkwork_api_app_gateway::get_video_character_canonical(
            request_context.tenant_id(),
            request_context.project_id(),
            &character_id,
        )
        .expect("video character canonical retrieve"),
    )
    .into_response()
}

async fn video_edits_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Extension(request_id): Extension<RequestId>,
    ExtractJson(request): ExtractJson<EditVideoRequest>,
) -> Response {
    let canonical_hold = match admit_canonical_video_request_for_route(
        state.store.as_ref(),
        &request_context,
        &request_id,
        &request.video_id,
        None,
    )
    .await
    {
        Ok(CanonicalChatAdmission::Held(hold)) => Some(hold),
        Ok(CanonicalChatAdmission::InsufficientBalance {
            account_id,
            shortfall_quantity,
        }) => {
            return account_balance_insufficient_response(account_id, shortfall_quantity);
        }
        Ok(CanonicalChatAdmission::NotApplicable) => None,
        Err(_) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to evaluate canonical account admission",
            )
                .into_response();
        }
    };

    match sdkwork_api_app_gateway::relay_edit_video_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_model = response_usage_id_or_single_data_item_id(&response)
                .unwrap_or(request.video_id.as_str());
            let video_reference_id =
                response_usage_id_or_single_data_item_id(&response).map(str::to_owned);
            let completed_video_seconds = completed_video_seconds_from_response(&response);

            if let Some(hold) = canonical_hold.as_ref() {
                match video_reference_id.as_deref() {
                    Some(video_reference_id) => {
                        if annotate_canonical_request_upstream_reference(
                            state.store.as_ref(),
                            hold,
                            video_reference_id,
                            RequestStatus::Running,
                        )
                        .await
                        .is_err()
                        {
                            let _ =
                                settle_canonical_chat_request_release(state.store.as_ref(), hold)
                                    .await;
                            return (
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                "failed to annotate canonical video request",
                            )
                                .into_response();
                        }
                    }
                    None if completed_video_seconds.is_none() => {
                        let _ =
                            settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
                        return bad_gateway_openai_response(
                            "failed to relay upstream video edits without reference id",
                        );
                    }
                    None => {}
                }
            }

            if completed_video_seconds.is_some() || canonical_hold.is_none() {
                if let Some(video_seconds) = completed_video_seconds {
                    if let Some(hold) = canonical_hold.as_ref() {
                        let canonical_settlement =
                            canonical_video_settlement_amounts_from_model_price(
                                &hold.model_price,
                                video_seconds,
                            )
                            .unwrap_or(
                                CanonicalChatSettlementAmounts {
                                    usage_units: hold.estimated_usage.total_tokens.max(1),
                                    customer_charge: hold.estimated_quantity,
                                },
                            );
                        if record_gateway_usage_for_project_with_media_and_reference_id(
                            state.store.as_ref(),
                            request_context.tenant_id(),
                            request_context.project_id(),
                            "videos",
                            &hold.model_price.model_id,
                            canonical_settlement.usage_units,
                            canonical_settlement.customer_charge,
                            BillingMediaMetrics {
                                video_seconds,
                                ..BillingMediaMetrics::default()
                            },
                            video_reference_id.as_deref(),
                        )
                        .await
                        .is_err()
                        {
                            let _ =
                                settle_canonical_chat_request_release(state.store.as_ref(), hold)
                                    .await;
                            return (
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                "failed to record usage",
                            )
                                .into_response();
                        }
                        if settle_canonical_chat_request_capture(
                            state.store.as_ref(),
                            hold,
                            &canonical_settlement,
                        )
                        .await
                        .is_err()
                        {
                            return (
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                "failed to capture canonical account hold",
                            )
                                .into_response();
                        }
                    } else if record_gateway_usage_for_project_with_route_key(
                        state.store.as_ref(),
                        request_context.tenant_id(),
                        request_context.project_id(),
                        "videos",
                        &request.video_id,
                        usage_model,
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
                } else if record_gateway_usage_for_project_with_route_key(
                    state.store.as_ref(),
                    request_context.tenant_id(),
                    request_context.project_id(),
                    "videos",
                    &request.video_id,
                    usage_model,
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
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            if let Some(hold) = canonical_hold.as_ref() {
                let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
            }
            return bad_gateway_openai_response("failed to relay upstream video edits");
        }
    }

    let response = sdkwork_api_app_gateway::edit_video(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .expect("video edits");
    let usage_model = match response.data.as_slice() {
        [item] => item.id.as_str(),
        _ => request.video_id.as_str(),
    };
    let response_value = serde_json::to_value(&response).unwrap_or(Value::Null);
    let video_reference_id =
        response_usage_id_or_single_data_item_id(&response_value).map(str::to_owned);
    let completed_video_seconds = completed_video_seconds_from_response(&response_value);

    if let Some(hold) = canonical_hold.as_ref() {
        match video_reference_id.as_deref() {
            Some(video_reference_id) => {
                if annotate_canonical_request_upstream_reference(
                    state.store.as_ref(),
                    hold,
                    video_reference_id,
                    RequestStatus::Running,
                )
                .await
                .is_err()
                {
                    let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to annotate canonical video request",
                    )
                        .into_response();
                }
            }
            None if completed_video_seconds.is_none() => {
                let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
                return bad_gateway_openai_response(
                    "failed to relay upstream video edits without reference id",
                );
            }
            None => {}
        }
    }

    if completed_video_seconds.is_some() || canonical_hold.is_none() {
        if let Some(video_seconds) = completed_video_seconds {
            if let Some(hold) = canonical_hold.as_ref() {
                let canonical_settlement = canonical_video_settlement_amounts_from_model_price(
                    &hold.model_price,
                    video_seconds,
                )
                .unwrap_or(CanonicalChatSettlementAmounts {
                    usage_units: hold.estimated_usage.total_tokens.max(1),
                    customer_charge: hold.estimated_quantity,
                });
                if record_gateway_usage_for_project_with_media_and_reference_id(
                    state.store.as_ref(),
                    request_context.tenant_id(),
                    request_context.project_id(),
                    "videos",
                    &hold.model_price.model_id,
                    canonical_settlement.usage_units,
                    canonical_settlement.customer_charge,
                    BillingMediaMetrics {
                        video_seconds,
                        ..BillingMediaMetrics::default()
                    },
                    video_reference_id.as_deref(),
                )
                .await
                .is_err()
                {
                    let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to record usage",
                    )
                        .into_response();
                }
                if settle_canonical_chat_request_capture(
                    state.store.as_ref(),
                    hold,
                    &canonical_settlement,
                )
                .await
                .is_err()
                {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to capture canonical account hold",
                    )
                        .into_response();
                }
            } else if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "videos",
                &request.video_id,
                usage_model,
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
        } else if record_gateway_usage_for_project_with_route_key(
            state.store.as_ref(),
            request_context.tenant_id(),
            request_context.project_id(),
            "videos",
            &request.video_id,
            usage_model,
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
    }

    Json(response).into_response()
}

async fn video_extensions_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Extension(request_id): Extension<RequestId>,
    ExtractJson(request): ExtractJson<ExtendVideoRequest>,
) -> Response {
    let route_key = request.video_id.as_deref().unwrap_or("videos");
    let canonical_hold = match admit_canonical_video_request_for_route(
        state.store.as_ref(),
        &request_context,
        &request_id,
        route_key,
        None,
    )
    .await
    {
        Ok(CanonicalChatAdmission::Held(hold)) => Some(hold),
        Ok(CanonicalChatAdmission::InsufficientBalance {
            account_id,
            shortfall_quantity,
        }) => {
            return account_balance_insufficient_response(account_id, shortfall_quantity);
        }
        Ok(CanonicalChatAdmission::NotApplicable) => None,
        Err(_) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to evaluate canonical account admission",
            )
                .into_response();
        }
    };

    match sdkwork_api_app_gateway::relay_extensions_video_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_model =
                response_usage_id_or_single_data_item_id(&response).unwrap_or(route_key);
            let video_reference_id =
                response_usage_id_or_single_data_item_id(&response).map(str::to_owned);
            let completed_video_seconds = completed_video_seconds_from_response(&response);

            if let Some(hold) = canonical_hold.as_ref() {
                match video_reference_id.as_deref() {
                    Some(video_reference_id) => {
                        if annotate_canonical_request_upstream_reference(
                            state.store.as_ref(),
                            hold,
                            video_reference_id,
                            RequestStatus::Running,
                        )
                        .await
                        .is_err()
                        {
                            let _ =
                                settle_canonical_chat_request_release(state.store.as_ref(), hold)
                                    .await;
                            return (
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                "failed to annotate canonical video request",
                            )
                                .into_response();
                        }
                    }
                    None if completed_video_seconds.is_none() => {
                        let _ =
                            settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
                        return bad_gateway_openai_response(
                            "failed to relay upstream video extensions without reference id",
                        );
                    }
                    None => {}
                }
            }

            if completed_video_seconds.is_some() || canonical_hold.is_none() {
                if let Some(video_seconds) = completed_video_seconds {
                    if let Some(hold) = canonical_hold.as_ref() {
                        let canonical_settlement =
                            canonical_video_settlement_amounts_from_model_price(
                                &hold.model_price,
                                video_seconds,
                            )
                            .unwrap_or(
                                CanonicalChatSettlementAmounts {
                                    usage_units: hold.estimated_usage.total_tokens.max(1),
                                    customer_charge: hold.estimated_quantity,
                                },
                            );
                        if record_gateway_usage_for_project_with_media_and_reference_id(
                            state.store.as_ref(),
                            request_context.tenant_id(),
                            request_context.project_id(),
                            "videos",
                            &hold.model_price.model_id,
                            canonical_settlement.usage_units,
                            canonical_settlement.customer_charge,
                            BillingMediaMetrics {
                                video_seconds,
                                ..BillingMediaMetrics::default()
                            },
                            video_reference_id.as_deref(),
                        )
                        .await
                        .is_err()
                        {
                            let _ =
                                settle_canonical_chat_request_release(state.store.as_ref(), hold)
                                    .await;
                            return (
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                "failed to record usage",
                            )
                                .into_response();
                        }
                        if settle_canonical_chat_request_capture(
                            state.store.as_ref(),
                            hold,
                            &canonical_settlement,
                        )
                        .await
                        .is_err()
                        {
                            return (
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                "failed to capture canonical account hold",
                            )
                                .into_response();
                        }
                    } else if record_gateway_usage_for_project_with_route_key(
                        state.store.as_ref(),
                        request_context.tenant_id(),
                        request_context.project_id(),
                        "videos",
                        route_key,
                        usage_model,
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
                } else if record_gateway_usage_for_project_with_route_key(
                    state.store.as_ref(),
                    request_context.tenant_id(),
                    request_context.project_id(),
                    "videos",
                    route_key,
                    usage_model,
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
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            if let Some(hold) = canonical_hold.as_ref() {
                let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
            }
            return bad_gateway_openai_response("failed to relay upstream video extensions");
        }
    }

    let response = sdkwork_api_app_gateway::extensions_video(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .expect("video extensions");
    let usage_model = match response.data.as_slice() {
        [item] => item.id.as_str(),
        _ => route_key,
    };
    let response_value = serde_json::to_value(&response).unwrap_or(Value::Null);
    let video_reference_id =
        response_usage_id_or_single_data_item_id(&response_value).map(str::to_owned);
    let completed_video_seconds = completed_video_seconds_from_response(&response_value);

    if let Some(hold) = canonical_hold.as_ref() {
        match video_reference_id.as_deref() {
            Some(video_reference_id) => {
                if annotate_canonical_request_upstream_reference(
                    state.store.as_ref(),
                    hold,
                    video_reference_id,
                    RequestStatus::Running,
                )
                .await
                .is_err()
                {
                    let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to annotate canonical video request",
                    )
                        .into_response();
                }
            }
            None if completed_video_seconds.is_none() => {
                let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
                return bad_gateway_openai_response(
                    "failed to relay upstream video extensions without reference id",
                );
            }
            None => {}
        }
    }

    if completed_video_seconds.is_some() || canonical_hold.is_none() {
        if let Some(video_seconds) = completed_video_seconds {
            if let Some(hold) = canonical_hold.as_ref() {
                let canonical_settlement = canonical_video_settlement_amounts_from_model_price(
                    &hold.model_price,
                    video_seconds,
                )
                .unwrap_or(CanonicalChatSettlementAmounts {
                    usage_units: hold.estimated_usage.total_tokens.max(1),
                    customer_charge: hold.estimated_quantity,
                });
                if record_gateway_usage_for_project_with_media_and_reference_id(
                    state.store.as_ref(),
                    request_context.tenant_id(),
                    request_context.project_id(),
                    "videos",
                    &hold.model_price.model_id,
                    canonical_settlement.usage_units,
                    canonical_settlement.customer_charge,
                    BillingMediaMetrics {
                        video_seconds,
                        ..BillingMediaMetrics::default()
                    },
                    video_reference_id.as_deref(),
                )
                .await
                .is_err()
                {
                    let _ = settle_canonical_chat_request_release(state.store.as_ref(), hold).await;
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to record usage",
                    )
                        .into_response();
                }
                if settle_canonical_chat_request_capture(
                    state.store.as_ref(),
                    hold,
                    &canonical_settlement,
                )
                .await
                .is_err()
                {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to capture canonical account hold",
                    )
                        .into_response();
                }
            } else if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "videos",
                route_key,
                usage_model,
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
        } else if record_gateway_usage_for_project_with_route_key(
            state.store.as_ref(),
            request_context.tenant_id(),
            request_context.project_id(),
            "videos",
            route_key,
            usage_model,
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
    }

    Json(response).into_response()
}

async fn uploads_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateUploadRequest>,
) -> Response {
    match relay_upload_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let upload_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(request.purpose.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "uploads",
                &request.purpose,
                upload_id,
                8,
                0.008,
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
            return bad_gateway_openai_response("failed to relay upstream upload");
        }
    }

    let response = create_upload(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .expect("upload");

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "uploads",
        &request.purpose,
        response.id.as_str(),
        8,
        0.008,
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

    Json(response).into_response()
}

async fn upload_parts_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
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
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let part_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(request.upload_id.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "uploads",
                &request.upload_id,
                part_id,
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
            return bad_gateway_openai_response("failed to relay upstream upload part");
        }
    }

    let response = create_upload_part(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .expect("upload part");

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "uploads",
        &request.upload_id,
        response.id.as_str(),
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

    Json(response).into_response()
}

async fn upload_complete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(upload_id): Path<String>,
    ExtractJson(mut request): ExtractJson<CompleteUploadRequest>,
) -> Response {
    request.upload_id = upload_id;

    match relay_complete_upload_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
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
            return bad_gateway_openai_response("failed to relay upstream upload completion");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
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

    Json(
        complete_upload(
            request_context.tenant_id(),
            request_context.project_id(),
            &request,
        )
        .expect("upload complete"),
    )
    .into_response()
}

async fn upload_cancel_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(upload_id): Path<String>,
) -> Response {
    match relay_cancel_upload_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &upload_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "uploads",
                &upload_id,
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
            return bad_gateway_openai_response("failed to relay upstream upload cancellation");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "uploads",
        &upload_id,
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

    Json(
        cancel_upload(
            request_context.tenant_id(),
            request_context.project_id(),
            &upload_id,
        )
        .expect("upload cancel"),
    )
    .into_response()
}

async fn fine_tuning_jobs_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateFineTuningJobRequest>,
) -> Response {
    match relay_fine_tuning_job_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let fine_tuning_job_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(request.model.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "fine_tuning",
                &request.model,
                fine_tuning_job_id,
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
            return bad_gateway_openai_response("failed to relay upstream fine tuning job");
        }
    }

    let response = create_fine_tuning_job(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
    )
    .expect("fine tuning");

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "fine_tuning",
        &request.model,
        response.id.as_str(),
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

    Json(response).into_response()
}

async fn fine_tuning_jobs_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Response {
    match relay_list_fine_tuning_jobs_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "fine_tuning",
                "jobs",
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
            return bad_gateway_openai_response("failed to relay upstream fine tuning jobs list");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "fine_tuning",
        "jobs",
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
        list_fine_tuning_jobs(request_context.tenant_id(), request_context.project_id())
            .expect("fine tuning list"),
    )
    .into_response()
}

async fn fine_tuning_job_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match relay_get_fine_tuning_job_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
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
            return bad_gateway_openai_response(
                "failed to relay upstream fine tuning job retrieve",
            );
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
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
        get_fine_tuning_job(
            request_context.tenant_id(),
            request_context.project_id(),
            &fine_tuning_job_id,
        )
        .expect("fine tuning retrieve"),
    )
    .into_response()
}

async fn fine_tuning_job_cancel_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match relay_cancel_fine_tuning_job_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
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
            return bad_gateway_openai_response("failed to relay upstream fine tuning job cancel");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
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
        cancel_fine_tuning_job(
            request_context.tenant_id(),
            request_context.project_id(),
            &fine_tuning_job_id,
        )
        .expect("fine tuning cancel"),
    )
    .into_response()
}

async fn fine_tuning_job_events_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match relay_list_fine_tuning_job_events_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
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
            return bad_gateway_openai_response("failed to relay upstream fine tuning job events");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
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
        list_fine_tuning_job_events(
            request_context.tenant_id(),
            request_context.project_id(),
            &fine_tuning_job_id,
        )
        .expect("fine tuning job events"),
    )
    .into_response()
}

async fn fine_tuning_job_checkpoints_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match relay_list_fine_tuning_job_checkpoints_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
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
            return bad_gateway_openai_response(
                "failed to relay upstream fine tuning job checkpoints",
            );
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
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
        list_fine_tuning_job_checkpoints(
            request_context.tenant_id(),
            request_context.project_id(),
            &fine_tuning_job_id,
        )
        .expect("fine tuning job checkpoints"),
    )
    .into_response()
}

async fn fine_tuning_job_pause_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match sdkwork_api_app_gateway::relay_pause_fine_tuning_job_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "fine_tuning",
                &fine_tuning_job_id,
                8,
                0.008,
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
            return bad_gateway_openai_response("failed to relay upstream fine tuning job pause");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "fine_tuning",
        &fine_tuning_job_id,
        8,
        0.008,
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
        sdkwork_api_app_gateway::pause_fine_tuning_job(
            request_context.tenant_id(),
            request_context.project_id(),
            &fine_tuning_job_id,
        )
        .expect("fine tuning pause"),
    )
    .into_response()
}

async fn fine_tuning_job_resume_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match sdkwork_api_app_gateway::relay_resume_fine_tuning_job_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "fine_tuning",
                &fine_tuning_job_id,
                8,
                0.008,
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
            return bad_gateway_openai_response("failed to relay upstream fine tuning job resume");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "fine_tuning",
        &fine_tuning_job_id,
        8,
        0.008,
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
        sdkwork_api_app_gateway::resume_fine_tuning_job(
            request_context.tenant_id(),
            request_context.project_id(),
            &fine_tuning_job_id,
        )
        .expect("fine tuning resume"),
    )
    .into_response()
}

async fn fine_tuning_checkpoint_permissions_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(fine_tuned_model_checkpoint): Path<String>,
    ExtractJson(request): ExtractJson<CreateFineTuningCheckpointPermissionsRequest>,
) -> Response {
    match sdkwork_api_app_gateway::relay_fine_tuning_checkpoint_permissions_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuned_model_checkpoint,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_model = response_usage_id_or_single_data_item_id(&response)
                .unwrap_or(fine_tuned_model_checkpoint.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "fine_tuning",
                &fine_tuned_model_checkpoint,
                usage_model,
                5,
                0.005,
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
            return bad_gateway_openai_response(
                "failed to relay upstream fine tuning checkpoint permissions create",
            );
        }
    }

    let response = sdkwork_api_app_gateway::create_fine_tuning_checkpoint_permissions(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .expect("fine tuning checkpoint permissions create");
    let usage_model = match response.data.as_slice() {
        [permission] => permission.id.as_str(),
        _ => fine_tuned_model_checkpoint.as_str(),
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "fine_tuning",
        &fine_tuned_model_checkpoint,
        usage_model,
        5,
        0.005,
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

    Json(response).into_response()
}

async fn fine_tuning_checkpoint_permissions_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(fine_tuned_model_checkpoint): Path<String>,
) -> Response {
    match sdkwork_api_app_gateway::relay_list_fine_tuning_checkpoint_permissions_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuned_model_checkpoint,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "fine_tuning",
                &fine_tuned_model_checkpoint,
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
            return bad_gateway_openai_response(
                "failed to relay upstream fine tuning checkpoint permissions list",
            );
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "fine_tuning",
        &fine_tuned_model_checkpoint,
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

    Json(
        sdkwork_api_app_gateway::list_fine_tuning_checkpoint_permissions(
            request_context.tenant_id(),
            request_context.project_id(),
            &fine_tuned_model_checkpoint,
        )
        .expect("fine tuning checkpoint permissions list"),
    )
    .into_response()
}

async fn fine_tuning_checkpoint_permission_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((fine_tuned_model_checkpoint, permission_id)): Path<(String, String)>,
) -> Response {
    match sdkwork_api_app_gateway::relay_delete_fine_tuning_checkpoint_permission_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuned_model_checkpoint,
        &permission_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "fine_tuning",
                &fine_tuned_model_checkpoint,
                &permission_id,
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
            return bad_gateway_openai_response(
                "failed to relay upstream fine tuning checkpoint permission delete",
            );
        }
    }

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "fine_tuning",
        &fine_tuned_model_checkpoint,
        &permission_id,
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

    Json(
        sdkwork_api_app_gateway::delete_fine_tuning_checkpoint_permission(
            request_context.tenant_id(),
            request_context.project_id(),
            &permission_id,
        )
        .expect("fine tuning checkpoint permission delete"),
    )
    .into_response()
}

async fn assistants_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateAssistantRequest>,
) -> Response {
    match relay_assistant_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let assistant_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(request.model.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &request.model,
                assistant_id,
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
            return bad_gateway_openai_response("failed to relay upstream assistant");
        }
    }

    let response = create_assistant(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.name,
        &request.model,
    )
    .expect("assistant");

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &request.model,
        response.id.as_str(),
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

    Json(response).into_response()
}

async fn assistants_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Response {
    match relay_list_assistants_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                "assistants",
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
            return bad_gateway_openai_response("failed to relay upstream assistants list");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        "assistants",
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
        list_assistants(request_context.tenant_id(), request_context.project_id())
            .expect("assistants list"),
    )
    .into_response()
}

async fn assistant_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(assistant_id): Path<String>,
) -> Response {
    match relay_get_assistant_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &assistant_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &assistant_id,
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
            return bad_gateway_openai_response("failed to relay upstream assistant retrieve");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &assistant_id,
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
        get_assistant(
            request_context.tenant_id(),
            request_context.project_id(),
            &assistant_id,
        )
        .expect("assistant retrieve"),
    )
    .into_response()
}

async fn assistant_update_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(assistant_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateAssistantRequest>,
) -> Response {
    match relay_update_assistant_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &assistant_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_target = request.model.as_deref().unwrap_or(assistant_id.as_str());
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                usage_target,
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
            return bad_gateway_openai_response("failed to relay upstream assistant update");
        }
    }

    let usage_target = request.model.as_deref().unwrap_or(assistant_id.as_str());
    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        usage_target,
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
        update_assistant(
            request_context.tenant_id(),
            request_context.project_id(),
            &assistant_id,
            request.name.as_deref().unwrap_or("assistant"),
        )
        .expect("assistant update"),
    )
    .into_response()
}

async fn assistant_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(assistant_id): Path<String>,
) -> Response {
    match relay_delete_assistant_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &assistant_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &assistant_id,
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
            return bad_gateway_openai_response("failed to relay upstream assistant delete");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &assistant_id,
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
        delete_assistant(
            request_context.tenant_id(),
            request_context.project_id(),
            &assistant_id,
        )
        .expect("assistant delete"),
    )
    .into_response()
}

async fn webhooks_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateWebhookRequest>,
) -> Response {
    match relay_webhook_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let webhook_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(request.url.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "webhooks",
                &request.url,
                webhook_id,
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
            return bad_gateway_openai_response("failed to relay upstream webhook");
        }
    }

    let response = create_webhook(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.url,
        &request.events,
    )
    .expect("webhook");

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "webhooks",
        &request.url,
        response.id.as_str(),
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

    Json(response).into_response()
}

async fn webhooks_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Response {
    match relay_list_webhooks_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "webhooks",
                "webhooks",
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
            return bad_gateway_openai_response("failed to relay upstream webhooks list");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "webhooks",
        "webhooks",
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
        list_webhooks(request_context.tenant_id(), request_context.project_id())
            .expect("webhooks list"),
    )
    .into_response()
}

async fn webhook_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(webhook_id): Path<String>,
) -> Response {
    match relay_get_webhook_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &webhook_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "webhooks",
                &webhook_id,
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
            return bad_gateway_openai_response("failed to relay upstream webhook retrieve");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "webhooks",
        &webhook_id,
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
        get_webhook(
            request_context.tenant_id(),
            request_context.project_id(),
            &webhook_id,
        )
        .expect("webhook retrieve"),
    )
    .into_response()
}

async fn webhook_update_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(webhook_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateWebhookRequest>,
) -> Response {
    match relay_update_webhook_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &webhook_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_target = request.url.as_deref().unwrap_or(webhook_id.as_str());
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "webhooks",
                usage_target,
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
            return bad_gateway_openai_response("failed to relay upstream webhook update");
        }
    }

    let usage_target = request.url.as_deref().unwrap_or(webhook_id.as_str());
    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "webhooks",
        usage_target,
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
        update_webhook(
            request_context.tenant_id(),
            request_context.project_id(),
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
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(webhook_id): Path<String>,
) -> Response {
    match relay_delete_webhook_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &webhook_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "webhooks",
                &webhook_id,
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
            return bad_gateway_openai_response("failed to relay upstream webhook delete");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "webhooks",
        &webhook_id,
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
        delete_webhook(
            request_context.tenant_id(),
            request_context.project_id(),
            &webhook_id,
        )
        .expect("webhook delete"),
    )
    .into_response()
}

async fn realtime_sessions_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateRealtimeSessionRequest>,
) -> Response {
    match relay_realtime_session_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let realtime_session_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(request.model.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "realtime_sessions",
                &request.model,
                realtime_session_id,
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
            return bad_gateway_openai_response("failed to relay upstream realtime session");
        }
    }

    let response = create_realtime_session(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
    )
    .expect("realtime");

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "realtime_sessions",
        &request.model,
        response.id.as_str(),
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

    Json(response).into_response()
}

async fn evals_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateEvalRequest>,
) -> Response {
    match relay_eval_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let eval_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(request.name.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "evals",
                &request.name,
                eval_id,
                40,
                0.04,
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
            return bad_gateway_openai_response("failed to relay upstream eval");
        }
    }

    let response = create_eval(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.name,
    )
    .expect("eval");

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "evals",
        &request.name,
        response.id.as_str(),
        40,
        0.04,
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

    Json(response).into_response()
}

async fn evals_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Response {
    match relay_list_evals_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "evals",
                "evals",
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
            return bad_gateway_openai_response("failed to relay upstream evals list");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "evals",
        "evals",
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

    Json(list_evals(request_context.tenant_id(), request_context.project_id()).expect("eval list"))
        .into_response()
}

async fn eval_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(eval_id): Path<String>,
) -> Response {
    match relay_get_eval_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "evals",
                &eval_id,
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
            return bad_gateway_openai_response("failed to relay upstream eval retrieve");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "evals",
        &eval_id,
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
        get_eval(
            request_context.tenant_id(),
            request_context.project_id(),
            &eval_id,
        )
        .expect("eval retrieve"),
    )
    .into_response()
}

async fn eval_update_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(eval_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateEvalRequest>,
) -> Response {
    match relay_update_eval_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "evals",
                &eval_id,
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
            return bad_gateway_openai_response("failed to relay upstream eval update");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "evals",
        &eval_id,
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
        update_eval(
            request_context.tenant_id(),
            request_context.project_id(),
            &eval_id,
            &request,
        )
        .expect("eval update"),
    )
    .into_response()
}

async fn eval_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(eval_id): Path<String>,
) -> Response {
    match relay_delete_eval_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "evals",
                &eval_id,
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
            return bad_gateway_openai_response("failed to relay upstream eval delete");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "evals",
        &eval_id,
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
        delete_eval(
            request_context.tenant_id(),
            request_context.project_id(),
            &eval_id,
        )
        .expect("eval delete"),
    )
    .into_response()
}

async fn eval_runs_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(eval_id): Path<String>,
) -> Response {
    match relay_list_eval_runs_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "evals",
                &eval_id,
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
            return bad_gateway_openai_response("failed to relay upstream eval runs list");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "evals",
        &eval_id,
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
        list_eval_runs(
            request_context.tenant_id(),
            request_context.project_id(),
            &eval_id,
        )
        .expect("eval runs list"),
    )
    .into_response()
}

async fn eval_runs_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(eval_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateEvalRunRequest>,
) -> Response {
    match relay_eval_run_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let run_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(eval_id.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "evals",
                &eval_id,
                run_id,
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
            return bad_gateway_openai_response("failed to relay upstream eval run create");
        }
    }

    let response = create_eval_run(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &request,
    )
    .expect("eval run create");

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "evals",
        &eval_id,
        response.id.as_str(),
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

    Json(response).into_response()
}

async fn eval_run_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((eval_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_get_eval_run_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &run_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "evals",
                &eval_id,
                &run_id,
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
            return bad_gateway_openai_response("failed to relay upstream eval run retrieve");
        }
    }

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "evals",
        &eval_id,
        &run_id,
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
        get_eval_run(
            request_context.tenant_id(),
            request_context.project_id(),
            &eval_id,
            &run_id,
        )
        .expect("eval run retrieve"),
    )
    .into_response()
}

async fn eval_run_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((eval_id, run_id)): Path<(String, String)>,
) -> Response {
    match sdkwork_api_app_gateway::relay_delete_eval_run_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &run_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "evals",
                &eval_id,
                &run_id,
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
            return bad_gateway_openai_response("failed to relay upstream eval run delete");
        }
    }

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "evals",
        &eval_id,
        &run_id,
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
        sdkwork_api_app_gateway::delete_eval_run(
            request_context.tenant_id(),
            request_context.project_id(),
            &eval_id,
            &run_id,
        )
        .expect("eval run delete"),
    )
    .into_response()
}

async fn eval_run_cancel_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((eval_id, run_id)): Path<(String, String)>,
) -> Response {
    match sdkwork_api_app_gateway::relay_cancel_eval_run_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &run_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "evals",
                &eval_id,
                &run_id,
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
            return bad_gateway_openai_response("failed to relay upstream eval run cancel");
        }
    }

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "evals",
        &eval_id,
        &run_id,
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
        sdkwork_api_app_gateway::cancel_eval_run(
            request_context.tenant_id(),
            request_context.project_id(),
            &eval_id,
            &run_id,
        )
        .expect("eval run cancel"),
    )
    .into_response()
}

async fn eval_run_output_items_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((eval_id, run_id)): Path<(String, String)>,
) -> Response {
    match sdkwork_api_app_gateway::relay_list_eval_run_output_items_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &run_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "evals",
                &eval_id,
                &run_id,
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
            return bad_gateway_openai_response(
                "failed to relay upstream eval run output items list",
            );
        }
    }

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "evals",
        &eval_id,
        &run_id,
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
        sdkwork_api_app_gateway::list_eval_run_output_items(
            request_context.tenant_id(),
            request_context.project_id(),
            &eval_id,
            &run_id,
        )
        .expect("eval run output items list"),
    )
    .into_response()
}

async fn eval_run_output_item_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((eval_id, run_id, output_item_id)): Path<(String, String, String)>,
) -> Response {
    match sdkwork_api_app_gateway::relay_get_eval_run_output_item_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &run_id,
        &output_item_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "evals",
                &eval_id,
                &output_item_id,
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
            return bad_gateway_openai_response(
                "failed to relay upstream eval run output item retrieve",
            );
        }
    }

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "evals",
        &eval_id,
        &output_item_id,
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
        sdkwork_api_app_gateway::get_eval_run_output_item(
            request_context.tenant_id(),
            request_context.project_id(),
            &eval_id,
            &run_id,
            &output_item_id,
        )
        .expect("eval run output item retrieve"),
    )
    .into_response()
}

async fn batches_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateBatchRequest>,
) -> Response {
    match relay_batch_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let batch_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(request.endpoint.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "batches",
                &request.endpoint,
                batch_id,
                60,
                0.06,
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
            return bad_gateway_openai_response("failed to relay upstream batch");
        }
    }

    let response = create_batch(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.endpoint,
        &request.input_file_id,
    )
    .expect("batch");

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "batches",
        &request.endpoint,
        response.id.as_str(),
        60,
        0.06,
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

    Json(response).into_response()
}

async fn batches_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Response {
    match relay_list_batches_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "batches",
                "batches",
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
            return bad_gateway_openai_response("failed to relay upstream batches list");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "batches",
        "batches",
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
        list_batches(request_context.tenant_id(), request_context.project_id())
            .expect("batches list"),
    )
    .into_response()
}

async fn batch_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(batch_id): Path<String>,
) -> Response {
    match relay_get_batch_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &batch_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "batches",
                &batch_id,
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
            return bad_gateway_openai_response("failed to relay upstream batch retrieve");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "batches",
        &batch_id,
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
        get_batch(
            request_context.tenant_id(),
            request_context.project_id(),
            &batch_id,
        )
        .expect("batch retrieve"),
    )
    .into_response()
}

async fn batch_cancel_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(batch_id): Path<String>,
) -> Response {
    match relay_cancel_batch_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &batch_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "batches",
                &batch_id,
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
            return bad_gateway_openai_response("failed to relay upstream batch cancel");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "batches",
        &batch_id,
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
        cancel_batch(
            request_context.tenant_id(),
            request_context.project_id(),
            &batch_id,
        )
        .expect("batch cancel"),
    )
    .into_response()
}

async fn vector_stores_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateVectorStoreRequest>,
) -> Response {
    match relay_vector_store_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let vector_store_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(request.name.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "vector_stores",
                &request.name,
                vector_store_id,
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
            return bad_gateway_openai_response("failed to relay upstream vector store");
        }
    }

    let response = create_vector_store(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.name,
    )
    .expect("vector store");

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_stores",
        &request.name,
        response.id.as_str(),
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

    Json(response).into_response()
}

async fn vector_stores_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Response {
    match relay_list_vector_stores_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
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
            return bad_gateway_openai_response("failed to relay upstream vector stores list");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
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

    Json(
        list_vector_stores(request_context.tenant_id(), request_context.project_id())
            .expect("vector stores list"),
    )
    .into_response()
}

async fn vector_store_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(vector_store_id): Path<String>,
) -> Response {
    match relay_get_vector_store_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
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
            return bad_gateway_openai_response("failed to relay upstream vector store retrieve");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
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
        get_vector_store(
            request_context.tenant_id(),
            request_context.project_id(),
            &vector_store_id,
        )
        .expect("vector store retrieve"),
    )
    .into_response()
}

async fn vector_store_update_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(vector_store_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateVectorStoreRequest>,
) -> Response {
    match relay_update_vector_store_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
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
            return bad_gateway_openai_response("failed to relay upstream vector store update");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
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
            request_context.tenant_id(),
            request_context.project_id(),
            &vector_store_id,
            request.name.as_deref().unwrap_or("vector-store"),
        )
        .expect("vector store update"),
    )
    .into_response()
}

async fn vector_store_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(vector_store_id): Path<String>,
) -> Response {
    match relay_delete_vector_store_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
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
            return bad_gateway_openai_response("failed to relay upstream vector store delete");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
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
        delete_vector_store(
            request_context.tenant_id(),
            request_context.project_id(),
            &vector_store_id,
        )
        .expect("vector store delete"),
    )
    .into_response()
}

async fn vector_store_search_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(vector_store_id): Path<String>,
    ExtractJson(request): ExtractJson<SearchVectorStoreRequest>,
) -> Response {
    match relay_search_vector_store_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
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
            return bad_gateway_openai_response("failed to relay upstream vector store search");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
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
        search_vector_store(
            request_context.tenant_id(),
            request_context.project_id(),
            &vector_store_id,
            &request.query,
        )
        .expect("vector store search"),
    )
    .into_response()
}

async fn vector_store_files_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(vector_store_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateVectorStoreFileRequest>,
) -> Response {
    match relay_vector_store_file_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_model = response_usage_id_or_single_data_item_id(&response)
                .unwrap_or(request.file_id.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "vector_store_files",
                &vector_store_id,
                usage_model,
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
            return bad_gateway_openai_response("failed to relay upstream vector store file");
        }
    }

    let response = create_vector_store_file(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &request.file_id,
    )
    .expect("vector store file");

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_store_files",
        &vector_store_id,
        response.id.as_str(),
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

    Json(response).into_response()
}

async fn vector_store_files_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(vector_store_id): Path<String>,
) -> Response {
    match relay_list_vector_store_files_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
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
            return bad_gateway_openai_response("failed to relay upstream vector store files list");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
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
        list_vector_store_files(
            request_context.tenant_id(),
            request_context.project_id(),
            &vector_store_id,
        )
        .expect("vector store files list"),
    )
    .into_response()
}

async fn vector_store_file_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((vector_store_id, file_id)): Path<(String, String)>,
) -> Response {
    match relay_get_vector_store_file_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &file_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "vector_store_files",
                &vector_store_id,
                &file_id,
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
            return bad_gateway_openai_response(
                "failed to relay upstream vector store file retrieve",
            );
        }
    }

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_store_files",
        &vector_store_id,
        &file_id,
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
        get_vector_store_file(
            request_context.tenant_id(),
            request_context.project_id(),
            &vector_store_id,
            &file_id,
        )
        .expect("vector store file retrieve"),
    )
    .into_response()
}

async fn vector_store_file_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((vector_store_id, file_id)): Path<(String, String)>,
) -> Response {
    match relay_delete_vector_store_file_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &file_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "vector_store_files",
                &vector_store_id,
                &file_id,
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
            return bad_gateway_openai_response(
                "failed to relay upstream vector store file delete",
            );
        }
    }

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_store_files",
        &vector_store_id,
        &file_id,
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
        delete_vector_store_file(
            request_context.tenant_id(),
            request_context.project_id(),
            &vector_store_id,
            &file_id,
        )
        .expect("vector store file delete"),
    )
    .into_response()
}

async fn vector_store_file_batches_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(vector_store_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateVectorStoreFileBatchRequest>,
) -> Response {
    match relay_vector_store_file_batch_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let batch_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(vector_store_id.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "vector_store_file_batches",
                &vector_store_id,
                batch_id,
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
            return bad_gateway_openai_response("failed to relay upstream vector store file batch");
        }
    }

    let response = create_vector_store_file_batch(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &request.file_ids,
    )
    .expect("vector store file batch");

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_store_file_batches",
        &vector_store_id,
        response.id.as_str(),
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

    Json(response).into_response()
}

async fn vector_store_file_batch_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((vector_store_id, batch_id)): Path<(String, String)>,
) -> Response {
    match relay_get_vector_store_file_batch_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &batch_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "vector_store_file_batches",
                &vector_store_id,
                &batch_id,
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
            return bad_gateway_openai_response(
                "failed to relay upstream vector store file batch retrieve",
            );
        }
    }

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_store_file_batches",
        &vector_store_id,
        &batch_id,
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
        get_vector_store_file_batch(
            request_context.tenant_id(),
            request_context.project_id(),
            &vector_store_id,
            &batch_id,
        )
        .expect("vector store file batch retrieve"),
    )
    .into_response()
}

async fn vector_store_file_batch_cancel_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((vector_store_id, batch_id)): Path<(String, String)>,
) -> Response {
    match relay_cancel_vector_store_file_batch_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &batch_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "vector_store_file_batches",
                &vector_store_id,
                &batch_id,
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
            return bad_gateway_openai_response(
                "failed to relay upstream vector store file batch cancel",
            );
        }
    }

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_store_file_batches",
        &vector_store_id,
        &batch_id,
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
        cancel_vector_store_file_batch(
            request_context.tenant_id(),
            request_context.project_id(),
            &vector_store_id,
            &batch_id,
        )
        .expect("vector store file batch cancel"),
    )
    .into_response()
}

async fn vector_store_file_batch_files_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((vector_store_id, batch_id)): Path<(String, String)>,
) -> Response {
    match relay_list_vector_store_file_batch_files_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &batch_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "vector_store_file_batches",
                &vector_store_id,
                &batch_id,
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
            return bad_gateway_openai_response(
                "failed to relay upstream vector store file batch files",
            );
        }
    }

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_store_file_batches",
        &vector_store_id,
        &batch_id,
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
        list_vector_store_file_batch_files(
            request_context.tenant_id(),
            request_context.project_id(),
            &vector_store_id,
            &batch_id,
        )
        .expect("vector store file batch files"),
    )
    .into_response()
}

async fn enforce_project_quota<S>(
    store: &S,
    project_id: &str,
    requested_units: u64,
) -> anyhow::Result<Option<Response>>
where
    S: sdkwork_api_app_billing::BillingQuotaStore + ?Sized,
{
    let evaluation = check_quota(store, project_id, requested_units).await?;
    if evaluation.allowed {
        Ok(None)
    } else {
        Ok(Some(quota_exceeded_response(project_id, &evaluation)))
    }
}

fn quota_exceeded_response(project_id: &str, evaluation: &QuotaCheckResult) -> Response {
    let mut error = OpenAiErrorResponse::new(
        quota_exceeded_message(project_id, evaluation),
        "insufficient_quota",
    );
    error.error.code = Some("quota_exceeded".to_owned());
    (StatusCode::TOO_MANY_REQUESTS, Json(error)).into_response()
}

fn account_balance_insufficient_response(account_id: u64, shortfall_quantity: f64) -> Response {
    let mut error = OpenAiErrorResponse::new(
        format!(
            "Canonical account {account_id} does not have enough available balance. Shortfall: {:.2} units.",
            shortfall_quantity.max(0.0)
        ),
        "insufficient_quota",
    );
    error.error.code = Some("account_balance_insufficient".to_owned());
    (StatusCode::TOO_MANY_REQUESTS, Json(error)).into_response()
}

async fn admit_canonical_chat_request(
    store: &dyn AdminStore,
    request_context: &AuthenticatedGatewayRequest,
    request_id: &RequestId,
    request: &CreateChatCompletionRequest,
) -> anyhow::Result<CanonicalChatAdmission> {
    let Some(subject) = request_context.canonical_subject() else {
        return Ok(CanonicalChatAdmission::NotApplicable);
    };
    let Some(account_store) = store.account_kernel() else {
        return Ok(CanonicalChatAdmission::NotApplicable);
    };
    let Some(account) = resolve_payable_account_for_gateway_subject(account_store, subject)
        .await
        .context("resolve payable account for canonical chat request")?
    else {
        return Ok(CanonicalChatAdmission::NotApplicable);
    };

    let now_ms =
        current_billing_timestamp_ms().context("resolve billing timestamp for chat request")?;
    let usage_context = planned_execution_usage_context_for_route(
        store,
        request_context.tenant_id(),
        request_context.project_id(),
        "chat_completion",
        &request.model,
    )
    .await
    .context("resolve planned execution usage context for canonical chat request")?;
    let Some(model_price) = resolve_active_model_price_for_route(
        store,
        &usage_context.provider_id,
        usage_context.channel_id.as_deref(),
        &request.model,
    )
    .await
    .context("resolve active model price for canonical chat request")?
    else {
        return Ok(CanonicalChatAdmission::NotApplicable);
    };
    let estimated_usage = estimate_chat_completion_usage_metrics(request);
    let Some(requested_quantity) =
        estimate_usage_priced_customer_charge(&model_price, estimated_usage)
    else {
        return Ok(CanonicalChatAdmission::NotApplicable);
    };
    if requested_quantity <= f64::EPSILON {
        return Ok(CanonicalChatAdmission::NotApplicable);
    }
    let hold_plan = plan_account_hold(
        account_store,
        account.account_id,
        requested_quantity,
        now_ms,
    )
    .await
    .context("plan account hold for canonical chat request")?;
    if !hold_plan.sufficient_balance {
        return Ok(CanonicalChatAdmission::InsufficientBalance {
            account_id: account.account_id,
            shortfall_quantity: hold_plan.shortfall_quantity,
        });
    }

    let kernel_request_id =
        gateway_account_kernel_request_id(request_id.as_str(), "chat_completion");
    let hold_id = gateway_account_kernel_hold_id(kernel_request_id)
        .context("derive hold identifier for canonical chat request")?;
    let request_meter_fact = RequestMeterFactRecord::new(
        kernel_request_id,
        subject.tenant_id,
        subject.organization_id,
        subject.user_id,
        account.account_id,
        subject.auth_type.as_str(),
        "chat_completion",
        usage_context.channel_id.as_deref().unwrap_or("gateway"),
        &request.model,
        &usage_context.provider_id,
    )
    .with_api_key_id(subject.api_key_id)
    .with_api_key_hash(subject.api_key_hash.clone())
    .with_jwt_subject(subject.jwt_subject.clone())
    .with_platform(subject.platform.clone())
    .with_owner(subject.owner.clone())
    .with_request_trace_id(Some(request_id.as_str().to_owned()))
    .with_gateway_request_ref(Some(request_id.as_str().to_owned()))
    .with_protocol_family("openai")
    .with_started_at_ms(now_ms)
    .with_created_at_ms(now_ms)
    .with_updated_at_ms(now_ms);

    create_account_hold(
        account_store,
        CreateAccountHoldInput {
            hold_id,
            account_id: account.account_id,
            request_id: kernel_request_id,
            estimated_quantity: requested_quantity,
            expires_at_ms: now_ms.saturating_add(GATEWAY_HOLD_TTL_MS),
            now_ms,
            request_meter_fact,
        },
    )
    .await
    .context("create canonical chat account hold")?;

    Ok(CanonicalChatAdmission::Held(CanonicalHeldCharge {
        hold_id,
        request_id: kernel_request_id,
        estimated_quantity: requested_quantity,
        estimated_usage,
        provider_cost_amount: usage_context.reference_amount.unwrap_or(0.0),
        model_price,
    }))
}

async fn admit_canonical_moderation_request(
    store: &dyn AdminStore,
    request_context: &AuthenticatedGatewayRequest,
    request_id: &RequestId,
    request: &CreateModerationRequest,
) -> anyhow::Result<CanonicalChatAdmission> {
    let Some(subject) = request_context.canonical_subject() else {
        return Ok(CanonicalChatAdmission::NotApplicable);
    };
    let Some(account_store) = store.account_kernel() else {
        return Ok(CanonicalChatAdmission::NotApplicable);
    };
    let Some(account) = resolve_payable_account_for_gateway_subject(account_store, subject).await?
    else {
        return Ok(CanonicalChatAdmission::NotApplicable);
    };

    let now_ms = current_billing_timestamp_ms()?;
    let usage_context = planned_execution_usage_context_for_route(
        store,
        request_context.tenant_id(),
        request_context.project_id(),
        "moderations",
        &request.model,
    )
    .await?;
    let Some(model_price) = resolve_active_model_price_for_route(
        store,
        &usage_context.provider_id,
        usage_context.channel_id.as_deref(),
        &request.model,
    )
    .await?
    else {
        return Ok(CanonicalChatAdmission::NotApplicable);
    };
    let estimated_usage = estimate_moderation_usage_metrics(request);
    let Some(requested_quantity) =
        estimate_usage_priced_customer_charge(&model_price, estimated_usage)
    else {
        return Ok(CanonicalChatAdmission::NotApplicable);
    };
    if requested_quantity <= f64::EPSILON {
        return Ok(CanonicalChatAdmission::NotApplicable);
    }
    let hold_plan = plan_account_hold(
        account_store,
        account.account_id,
        requested_quantity,
        now_ms,
    )
    .await?;
    if !hold_plan.sufficient_balance {
        return Ok(CanonicalChatAdmission::InsufficientBalance {
            account_id: account.account_id,
            shortfall_quantity: hold_plan.shortfall_quantity,
        });
    }

    let kernel_request_id = gateway_account_kernel_request_id(request_id.as_str(), "moderations");
    let hold_id = gateway_account_kernel_hold_id(kernel_request_id)?;
    let request_meter_fact = RequestMeterFactRecord::new(
        kernel_request_id,
        subject.tenant_id,
        subject.organization_id,
        subject.user_id,
        account.account_id,
        subject.auth_type.as_str(),
        "moderations",
        usage_context.channel_id.as_deref().unwrap_or("gateway"),
        &request.model,
        &usage_context.provider_id,
    )
    .with_api_key_id(subject.api_key_id)
    .with_api_key_hash(subject.api_key_hash.clone())
    .with_jwt_subject(subject.jwt_subject.clone())
    .with_platform(subject.platform.clone())
    .with_owner(subject.owner.clone())
    .with_request_trace_id(Some(request_id.as_str().to_owned()))
    .with_gateway_request_ref(Some(request_id.as_str().to_owned()))
    .with_protocol_family("openai")
    .with_started_at_ms(now_ms)
    .with_created_at_ms(now_ms)
    .with_updated_at_ms(now_ms);

    create_account_hold(
        account_store,
        CreateAccountHoldInput {
            hold_id,
            account_id: account.account_id,
            request_id: kernel_request_id,
            estimated_quantity: requested_quantity,
            expires_at_ms: now_ms.saturating_add(GATEWAY_HOLD_TTL_MS),
            now_ms,
            request_meter_fact,
        },
    )
    .await?;

    Ok(CanonicalChatAdmission::Held(CanonicalHeldCharge {
        hold_id,
        request_id: kernel_request_id,
        estimated_quantity: requested_quantity,
        estimated_usage,
        provider_cost_amount: usage_context.reference_amount.unwrap_or(0.0),
        model_price,
    }))
}

async fn admit_canonical_response_request(
    store: &dyn AdminStore,
    request_context: &AuthenticatedGatewayRequest,
    request_id: &RequestId,
    request: &CreateResponseRequest,
) -> anyhow::Result<CanonicalChatAdmission> {
    let Some(subject) = request_context.canonical_subject() else {
        return Ok(CanonicalChatAdmission::NotApplicable);
    };
    let Some(account_store) = store.account_kernel() else {
        return Ok(CanonicalChatAdmission::NotApplicable);
    };
    let Some(account) = resolve_payable_account_for_gateway_subject(account_store, subject).await?
    else {
        return Ok(CanonicalChatAdmission::NotApplicable);
    };

    let now_ms = current_billing_timestamp_ms()?;
    let usage_context = planned_execution_usage_context_for_route(
        store,
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        &request.model,
    )
    .await?;
    let Some(model_price) = resolve_active_model_price_for_route(
        store,
        &usage_context.provider_id,
        usage_context.channel_id.as_deref(),
        &request.model,
    )
    .await?
    else {
        return Ok(CanonicalChatAdmission::NotApplicable);
    };
    let estimated_usage = estimate_response_usage_metrics(request);
    let Some(requested_quantity) =
        estimate_usage_priced_customer_charge(&model_price, estimated_usage)
    else {
        return Ok(CanonicalChatAdmission::NotApplicable);
    };
    if requested_quantity <= f64::EPSILON {
        return Ok(CanonicalChatAdmission::NotApplicable);
    }
    let hold_plan = plan_account_hold(
        account_store,
        account.account_id,
        requested_quantity,
        now_ms,
    )
    .await?;
    if !hold_plan.sufficient_balance {
        return Ok(CanonicalChatAdmission::InsufficientBalance {
            account_id: account.account_id,
            shortfall_quantity: hold_plan.shortfall_quantity,
        });
    }

    let kernel_request_id = gateway_account_kernel_request_id(request_id.as_str(), "responses");
    let hold_id = gateway_account_kernel_hold_id(kernel_request_id)?;
    let request_meter_fact = RequestMeterFactRecord::new(
        kernel_request_id,
        subject.tenant_id,
        subject.organization_id,
        subject.user_id,
        account.account_id,
        subject.auth_type.as_str(),
        "responses",
        usage_context.channel_id.as_deref().unwrap_or("gateway"),
        &request.model,
        &usage_context.provider_id,
    )
    .with_api_key_id(subject.api_key_id)
    .with_api_key_hash(subject.api_key_hash.clone())
    .with_jwt_subject(subject.jwt_subject.clone())
    .with_platform(subject.platform.clone())
    .with_owner(subject.owner.clone())
    .with_request_trace_id(Some(request_id.as_str().to_owned()))
    .with_gateway_request_ref(Some(request_id.as_str().to_owned()))
    .with_protocol_family("openai")
    .with_started_at_ms(now_ms)
    .with_created_at_ms(now_ms)
    .with_updated_at_ms(now_ms);

    create_account_hold(
        account_store,
        CreateAccountHoldInput {
            hold_id,
            account_id: account.account_id,
            request_id: kernel_request_id,
            estimated_quantity: requested_quantity,
            expires_at_ms: now_ms.saturating_add(GATEWAY_HOLD_TTL_MS),
            now_ms,
            request_meter_fact,
        },
    )
    .await?;

    Ok(CanonicalChatAdmission::Held(CanonicalHeldCharge {
        hold_id,
        request_id: kernel_request_id,
        estimated_quantity: requested_quantity,
        estimated_usage,
        provider_cost_amount: usage_context.reference_amount.unwrap_or(0.0),
        model_price,
    }))
}

async fn admit_canonical_completion_request(
    store: &dyn AdminStore,
    request_context: &AuthenticatedGatewayRequest,
    request_id: &RequestId,
    request: &CreateCompletionRequest,
) -> anyhow::Result<CanonicalChatAdmission> {
    let Some(subject) = request_context.canonical_subject() else {
        return Ok(CanonicalChatAdmission::NotApplicable);
    };
    let Some(account_store) = store.account_kernel() else {
        return Ok(CanonicalChatAdmission::NotApplicable);
    };
    let Some(account) = resolve_payable_account_for_gateway_subject(account_store, subject).await?
    else {
        return Ok(CanonicalChatAdmission::NotApplicable);
    };

    let now_ms = current_billing_timestamp_ms()?;
    let usage_context = planned_execution_usage_context_for_route(
        store,
        request_context.tenant_id(),
        request_context.project_id(),
        "completions",
        &request.model,
    )
    .await?;
    let Some(model_price) = resolve_active_model_price_for_route(
        store,
        &usage_context.provider_id,
        usage_context.channel_id.as_deref(),
        &request.model,
    )
    .await?
    else {
        return Ok(CanonicalChatAdmission::NotApplicable);
    };
    let estimated_usage = estimate_completion_usage_metrics(request);
    let Some(requested_quantity) =
        estimate_usage_priced_customer_charge(&model_price, estimated_usage)
    else {
        return Ok(CanonicalChatAdmission::NotApplicable);
    };
    if requested_quantity <= f64::EPSILON {
        return Ok(CanonicalChatAdmission::NotApplicable);
    }
    let hold_plan = plan_account_hold(
        account_store,
        account.account_id,
        requested_quantity,
        now_ms,
    )
    .await?;
    if !hold_plan.sufficient_balance {
        return Ok(CanonicalChatAdmission::InsufficientBalance {
            account_id: account.account_id,
            shortfall_quantity: hold_plan.shortfall_quantity,
        });
    }

    let kernel_request_id = gateway_account_kernel_request_id(request_id.as_str(), "completions");
    let hold_id = gateway_account_kernel_hold_id(kernel_request_id)?;
    let request_meter_fact = RequestMeterFactRecord::new(
        kernel_request_id,
        subject.tenant_id,
        subject.organization_id,
        subject.user_id,
        account.account_id,
        subject.auth_type.as_str(),
        "completions",
        usage_context.channel_id.as_deref().unwrap_or("gateway"),
        &request.model,
        &usage_context.provider_id,
    )
    .with_api_key_id(subject.api_key_id)
    .with_api_key_hash(subject.api_key_hash.clone())
    .with_jwt_subject(subject.jwt_subject.clone())
    .with_platform(subject.platform.clone())
    .with_owner(subject.owner.clone())
    .with_request_trace_id(Some(request_id.as_str().to_owned()))
    .with_gateway_request_ref(Some(request_id.as_str().to_owned()))
    .with_protocol_family("openai")
    .with_started_at_ms(now_ms)
    .with_created_at_ms(now_ms)
    .with_updated_at_ms(now_ms);

    create_account_hold(
        account_store,
        CreateAccountHoldInput {
            hold_id,
            account_id: account.account_id,
            request_id: kernel_request_id,
            estimated_quantity: requested_quantity,
            expires_at_ms: now_ms.saturating_add(GATEWAY_HOLD_TTL_MS),
            now_ms,
            request_meter_fact,
        },
    )
    .await?;

    Ok(CanonicalChatAdmission::Held(CanonicalHeldCharge {
        hold_id,
        request_id: kernel_request_id,
        estimated_quantity: requested_quantity,
        estimated_usage,
        provider_cost_amount: usage_context.reference_amount.unwrap_or(0.0),
        model_price,
    }))
}

async fn admit_canonical_embedding_request(
    store: &dyn AdminStore,
    request_context: &AuthenticatedGatewayRequest,
    request_id: &RequestId,
    request: &CreateEmbeddingRequest,
) -> anyhow::Result<CanonicalChatAdmission> {
    let Some(subject) = request_context.canonical_subject() else {
        return Ok(CanonicalChatAdmission::NotApplicable);
    };
    let Some(account_store) = store.account_kernel() else {
        return Ok(CanonicalChatAdmission::NotApplicable);
    };
    let Some(account) = resolve_payable_account_for_gateway_subject(account_store, subject).await?
    else {
        return Ok(CanonicalChatAdmission::NotApplicable);
    };

    let now_ms = current_billing_timestamp_ms()?;
    let usage_context = planned_execution_usage_context_for_route(
        store,
        request_context.tenant_id(),
        request_context.project_id(),
        "embeddings",
        &request.model,
    )
    .await?;
    let Some(model_price) = resolve_active_model_price_for_route(
        store,
        &usage_context.provider_id,
        usage_context.channel_id.as_deref(),
        &request.model,
    )
    .await?
    else {
        return Ok(CanonicalChatAdmission::NotApplicable);
    };
    let estimated_usage = estimate_embedding_usage_metrics(request);
    let Some(requested_quantity) =
        estimate_usage_priced_customer_charge(&model_price, estimated_usage)
    else {
        return Ok(CanonicalChatAdmission::NotApplicable);
    };
    if requested_quantity <= f64::EPSILON {
        return Ok(CanonicalChatAdmission::NotApplicable);
    }
    let hold_plan = plan_account_hold(
        account_store,
        account.account_id,
        requested_quantity,
        now_ms,
    )
    .await?;
    if !hold_plan.sufficient_balance {
        return Ok(CanonicalChatAdmission::InsufficientBalance {
            account_id: account.account_id,
            shortfall_quantity: hold_plan.shortfall_quantity,
        });
    }

    let kernel_request_id = gateway_account_kernel_request_id(request_id.as_str(), "embeddings");
    let hold_id = gateway_account_kernel_hold_id(kernel_request_id)?;
    let request_meter_fact = RequestMeterFactRecord::new(
        kernel_request_id,
        subject.tenant_id,
        subject.organization_id,
        subject.user_id,
        account.account_id,
        subject.auth_type.as_str(),
        "embeddings",
        usage_context.channel_id.as_deref().unwrap_or("gateway"),
        &request.model,
        &usage_context.provider_id,
    )
    .with_api_key_id(subject.api_key_id)
    .with_api_key_hash(subject.api_key_hash.clone())
    .with_jwt_subject(subject.jwt_subject.clone())
    .with_platform(subject.platform.clone())
    .with_owner(subject.owner.clone())
    .with_request_trace_id(Some(request_id.as_str().to_owned()))
    .with_gateway_request_ref(Some(request_id.as_str().to_owned()))
    .with_protocol_family("openai")
    .with_started_at_ms(now_ms)
    .with_created_at_ms(now_ms)
    .with_updated_at_ms(now_ms);

    create_account_hold(
        account_store,
        CreateAccountHoldInput {
            hold_id,
            account_id: account.account_id,
            request_id: kernel_request_id,
            estimated_quantity: requested_quantity,
            expires_at_ms: now_ms.saturating_add(GATEWAY_HOLD_TTL_MS),
            now_ms,
            request_meter_fact,
        },
    )
    .await?;

    Ok(CanonicalChatAdmission::Held(CanonicalHeldCharge {
        hold_id,
        request_id: kernel_request_id,
        estimated_quantity: requested_quantity,
        estimated_usage,
        provider_cost_amount: usage_context.reference_amount.unwrap_or(0.0),
        model_price,
    }))
}

async fn admit_canonical_request_priced_request(
    store: &dyn AdminStore,
    request_context: &AuthenticatedGatewayRequest,
    request_id: &RequestId,
    capability: &str,
    model_id: &str,
    usage_units: u64,
) -> anyhow::Result<CanonicalChatAdmission> {
    let Some(subject) = request_context.canonical_subject() else {
        return Ok(CanonicalChatAdmission::NotApplicable);
    };
    let Some(account_store) = store.account_kernel() else {
        return Ok(CanonicalChatAdmission::NotApplicable);
    };
    let Some(account) = resolve_payable_account_for_gateway_subject(account_store, subject).await?
    else {
        return Ok(CanonicalChatAdmission::NotApplicable);
    };

    let now_ms = current_billing_timestamp_ms()?;
    let usage_context = planned_execution_usage_context_for_route(
        store,
        request_context.tenant_id(),
        request_context.project_id(),
        capability,
        model_id,
    )
    .await?;
    let Some(model_price) = resolve_active_model_price_for_route(
        store,
        &usage_context.provider_id,
        usage_context.channel_id.as_deref(),
        model_id,
    )
    .await?
    else {
        return Ok(CanonicalChatAdmission::NotApplicable);
    };
    let estimated_usage = TokenUsageMetrics {
        total_tokens: usage_units.max(1),
        ..TokenUsageMetrics::default()
    };
    let Some(requested_quantity) =
        estimate_usage_priced_customer_charge(&model_price, TokenUsageMetrics::default())
    else {
        return Ok(CanonicalChatAdmission::NotApplicable);
    };
    if requested_quantity <= f64::EPSILON {
        return Ok(CanonicalChatAdmission::NotApplicable);
    }

    let hold_plan = plan_account_hold(
        account_store,
        account.account_id,
        requested_quantity,
        now_ms,
    )
    .await
    .context("plan account hold for canonical fixed-price request")?;
    if !hold_plan.sufficient_balance {
        return Ok(CanonicalChatAdmission::InsufficientBalance {
            account_id: account.account_id,
            shortfall_quantity: hold_plan.shortfall_quantity,
        });
    }

    let kernel_request_id = gateway_account_kernel_request_id(request_id.as_str(), capability);
    let hold_id = gateway_account_kernel_hold_id(kernel_request_id)
        .context("derive hold identifier for canonical fixed-price request")?;
    let request_meter_fact = RequestMeterFactRecord::new(
        kernel_request_id,
        subject.tenant_id,
        subject.organization_id,
        subject.user_id,
        account.account_id,
        subject.auth_type.as_str(),
        capability,
        usage_context.channel_id.as_deref().unwrap_or("gateway"),
        model_id,
        &usage_context.provider_id,
    )
    .with_api_key_id(subject.api_key_id)
    .with_api_key_hash(subject.api_key_hash.clone())
    .with_jwt_subject(subject.jwt_subject.clone())
    .with_platform(subject.platform.clone())
    .with_owner(subject.owner.clone())
    .with_request_trace_id(Some(request_id.as_str().to_owned()))
    .with_gateway_request_ref(Some(request_id.as_str().to_owned()))
    .with_protocol_family("openai")
    .with_started_at_ms(now_ms)
    .with_created_at_ms(now_ms)
    .with_updated_at_ms(now_ms);

    create_account_hold(
        account_store,
        CreateAccountHoldInput {
            hold_id,
            account_id: account.account_id,
            request_id: kernel_request_id,
            estimated_quantity: requested_quantity,
            expires_at_ms: now_ms.saturating_add(GATEWAY_HOLD_TTL_MS),
            now_ms,
            request_meter_fact,
        },
    )
    .await
    .context("create canonical fixed-price account hold")?;

    Ok(CanonicalChatAdmission::Held(CanonicalHeldCharge {
        hold_id,
        request_id: kernel_request_id,
        estimated_quantity: requested_quantity,
        estimated_usage,
        provider_cost_amount: usage_context.reference_amount.unwrap_or(0.0),
        model_price,
    }))
}

async fn admit_canonical_music_request(
    store: &dyn AdminStore,
    request_context: &AuthenticatedGatewayRequest,
    request_id: &RequestId,
    request: &CreateMusicRequest,
) -> anyhow::Result<CanonicalChatAdmission> {
    let Some(subject) = request_context.canonical_subject() else {
        return Ok(CanonicalChatAdmission::NotApplicable);
    };
    let Some(account_store) = store.account_kernel() else {
        return Ok(CanonicalChatAdmission::NotApplicable);
    };
    let Some(account) = resolve_payable_account_for_gateway_subject(account_store, subject).await?
    else {
        return Ok(CanonicalChatAdmission::NotApplicable);
    };

    let now_ms = current_billing_timestamp_ms()?;
    let usage_context = planned_execution_usage_context_for_route(
        store,
        request_context.tenant_id(),
        request_context.project_id(),
        "music",
        &request.model,
    )
    .await?;
    let Some(model_price) = resolve_active_model_price_for_route(
        store,
        &usage_context.provider_id,
        usage_context.channel_id.as_deref(),
        &request.model,
    )
    .await?
    else {
        return Ok(CanonicalChatAdmission::NotApplicable);
    };
    let estimated_music_seconds = request
        .duration_seconds
        .unwrap_or(DEFAULT_MUSIC_ESTIMATED_SECONDS)
        .max(DEFAULT_MUSIC_ESTIMATED_SECONDS);
    let Some(estimated_settlement) =
        canonical_music_settlement_amounts_from_model_price(&model_price, estimated_music_seconds)
    else {
        return Ok(CanonicalChatAdmission::NotApplicable);
    };
    if estimated_settlement.customer_charge <= f64::EPSILON {
        return Ok(CanonicalChatAdmission::NotApplicable);
    }

    let hold_plan = plan_account_hold(
        account_store,
        account.account_id,
        estimated_settlement.customer_charge,
        now_ms,
    )
    .await?;
    if !hold_plan.sufficient_balance {
        return Ok(CanonicalChatAdmission::InsufficientBalance {
            account_id: account.account_id,
            shortfall_quantity: hold_plan.shortfall_quantity,
        });
    }

    let kernel_request_id = gateway_account_kernel_request_id(request_id.as_str(), "music");
    let hold_id = gateway_account_kernel_hold_id(kernel_request_id)?;
    let request_meter_fact = RequestMeterFactRecord::new(
        kernel_request_id,
        subject.tenant_id,
        subject.organization_id,
        subject.user_id,
        account.account_id,
        subject.auth_type.as_str(),
        "music",
        usage_context.channel_id.as_deref().unwrap_or("gateway"),
        &request.model,
        &usage_context.provider_id,
    )
    .with_api_key_id(subject.api_key_id)
    .with_api_key_hash(subject.api_key_hash.clone())
    .with_jwt_subject(subject.jwt_subject.clone())
    .with_platform(subject.platform.clone())
    .with_owner(subject.owner.clone())
    .with_request_trace_id(Some(request_id.as_str().to_owned()))
    .with_gateway_request_ref(Some(request_id.as_str().to_owned()))
    .with_protocol_family("openai")
    .with_started_at_ms(now_ms)
    .with_created_at_ms(now_ms)
    .with_updated_at_ms(now_ms);

    create_account_hold(
        account_store,
        CreateAccountHoldInput {
            hold_id,
            account_id: account.account_id,
            request_id: kernel_request_id,
            estimated_quantity: estimated_settlement.customer_charge,
            expires_at_ms: now_ms.saturating_add(GATEWAY_HOLD_TTL_MS),
            now_ms,
            request_meter_fact,
        },
    )
    .await?;

    Ok(CanonicalChatAdmission::Held(CanonicalHeldCharge {
        hold_id,
        request_id: kernel_request_id,
        estimated_quantity: estimated_settlement.customer_charge,
        estimated_usage: TokenUsageMetrics {
            total_tokens: estimated_settlement.usage_units,
            ..TokenUsageMetrics::default()
        },
        provider_cost_amount: usage_context.reference_amount.unwrap_or(0.0),
        model_price,
    }))
}

async fn admit_canonical_video_request(
    store: &dyn AdminStore,
    request_context: &AuthenticatedGatewayRequest,
    request_id: &RequestId,
    model_id: &str,
) -> anyhow::Result<CanonicalChatAdmission> {
    admit_canonical_video_request_for_route(
        store,
        request_context,
        request_id,
        model_id,
        Some(model_id),
    )
    .await
}

async fn admit_canonical_video_request_for_route(
    store: &dyn AdminStore,
    request_context: &AuthenticatedGatewayRequest,
    request_id: &RequestId,
    route_key: &str,
    model_hint: Option<&str>,
) -> anyhow::Result<CanonicalChatAdmission> {
    let Some(subject) = request_context.canonical_subject() else {
        return Ok(CanonicalChatAdmission::NotApplicable);
    };
    let Some(account_store) = store.account_kernel() else {
        return Ok(CanonicalChatAdmission::NotApplicable);
    };
    let Some(account) = resolve_payable_account_for_gateway_subject(account_store, subject).await?
    else {
        return Ok(CanonicalChatAdmission::NotApplicable);
    };

    let now_ms = current_billing_timestamp_ms()?;
    let usage_context = planned_execution_usage_context_for_route(
        store,
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        route_key,
    )
    .await?;
    let Some(model_price) = resolve_active_video_model_price_for_route(
        store,
        &usage_context.provider_id,
        usage_context.channel_id.as_deref(),
        route_key,
        model_hint,
    )
    .await?
    else {
        return Ok(CanonicalChatAdmission::NotApplicable);
    };
    let Some(estimated_settlement) = canonical_video_settlement_amounts_from_model_price(
        &model_price,
        DEFAULT_VIDEO_ESTIMATED_SECONDS,
    ) else {
        return Ok(CanonicalChatAdmission::NotApplicable);
    };
    if estimated_settlement.customer_charge <= f64::EPSILON {
        return Ok(CanonicalChatAdmission::NotApplicable);
    }

    let hold_plan = plan_account_hold(
        account_store,
        account.account_id,
        estimated_settlement.customer_charge,
        now_ms,
    )
    .await?;
    if !hold_plan.sufficient_balance {
        return Ok(CanonicalChatAdmission::InsufficientBalance {
            account_id: account.account_id,
            shortfall_quantity: hold_plan.shortfall_quantity,
        });
    }

    let kernel_request_id = gateway_account_kernel_request_id(request_id.as_str(), "videos");
    let hold_id = gateway_account_kernel_hold_id(kernel_request_id)?;
    let request_meter_fact = RequestMeterFactRecord::new(
        kernel_request_id,
        subject.tenant_id,
        subject.organization_id,
        subject.user_id,
        account.account_id,
        subject.auth_type.as_str(),
        "videos",
        usage_context.channel_id.as_deref().unwrap_or("gateway"),
        &model_price.model_id,
        &usage_context.provider_id,
    )
    .with_api_key_id(subject.api_key_id)
    .with_api_key_hash(subject.api_key_hash.clone())
    .with_jwt_subject(subject.jwt_subject.clone())
    .with_platform(subject.platform.clone())
    .with_owner(subject.owner.clone())
    .with_request_trace_id(Some(request_id.as_str().to_owned()))
    .with_gateway_request_ref(Some(request_id.as_str().to_owned()))
    .with_protocol_family("openai")
    .with_started_at_ms(now_ms)
    .with_created_at_ms(now_ms)
    .with_updated_at_ms(now_ms);

    create_account_hold(
        account_store,
        CreateAccountHoldInput {
            hold_id,
            account_id: account.account_id,
            request_id: kernel_request_id,
            estimated_quantity: estimated_settlement.customer_charge,
            expires_at_ms: now_ms.saturating_add(GATEWAY_HOLD_TTL_MS),
            now_ms,
            request_meter_fact,
        },
    )
    .await?;

    Ok(CanonicalChatAdmission::Held(CanonicalHeldCharge {
        hold_id,
        request_id: kernel_request_id,
        estimated_quantity: estimated_settlement.customer_charge,
        estimated_usage: TokenUsageMetrics {
            total_tokens: estimated_settlement.usage_units,
            ..TokenUsageMetrics::default()
        },
        provider_cost_amount: usage_context.reference_amount.unwrap_or(0.0),
        model_price,
    }))
}

async fn resolve_active_video_model_price_for_route(
    store: &dyn AdminStore,
    provider_id: &str,
    channel_id: Option<&str>,
    route_key: &str,
    model_hint: Option<&str>,
) -> anyhow::Result<Option<ModelPriceRecord>> {
    let mut candidate_model_ids = Vec::new();
    if let Some(model_hint) = model_hint.map(str::trim).filter(|value| !value.is_empty()) {
        candidate_model_ids.push(model_hint.to_owned());
    }
    let route_key = route_key.trim();
    if !route_key.is_empty() && !candidate_model_ids.iter().any(|value| value == route_key) {
        candidate_model_ids.push(route_key.to_owned());
    }

    for candidate_model_id in &candidate_model_ids {
        if let Some(model_price) =
            resolve_active_model_price_for_route(store, provider_id, channel_id, candidate_model_id)
                .await?
        {
            return Ok(Some(model_price));
        }
    }

    let mut candidate_prices = store
        .list_model_prices()
        .await?
        .into_iter()
        .filter(|price| {
            price.is_active
                && price.proxy_provider_id == provider_id
                && channel_id
                    .map(|channel_id| price.channel_id == channel_id)
                    .unwrap_or(true)
        })
        .filter(|price| {
            canonical_video_settlement_amounts_from_model_price(
                price,
                DEFAULT_VIDEO_ESTIMATED_SECONDS,
            )
            .is_some()
        })
        .collect::<Vec<_>>();
    candidate_prices.sort_by(|left, right| left.model_id.cmp(&right.model_id));

    if candidate_prices.len() == 1 {
        return Ok(candidate_prices.into_iter().next());
    }

    Ok(None)
}

async fn settle_canonical_chat_request_capture(
    store: &dyn AdminStore,
    hold: &CanonicalHeldCharge,
    settlement: &CanonicalChatSettlementAmounts,
) -> anyhow::Result<()> {
    let Some(account_store) = store.account_kernel() else {
        anyhow::bail!("account kernel store is not available for hold capture");
    };

    if let Err(error) = capture_account_hold(
        account_store,
        CaptureAccountHoldInput {
            hold_id: hold.hold_id,
            captured_quantity: settlement.customer_charge,
            provider_cost_amount: hold.provider_cost_amount,
            retail_charge_amount: settlement.customer_charge,
            settled_at_ms: current_billing_timestamp_ms()?,
        },
    )
    .await
    {
        record_current_commercial_event(
            CommercialEventKind::HoldFailure,
            CommercialEventDimensions::default().with_result("capture_failed"),
        );
        return Err(error);
    }
    Ok(())
}

async fn settle_canonical_chat_request_release(
    store: &dyn AdminStore,
    hold: &CanonicalHeldCharge,
) -> anyhow::Result<()> {
    let Some(account_store) = store.account_kernel() else {
        anyhow::bail!("account kernel store is not available for hold release");
    };

    if let Err(error) = release_account_hold(
        account_store,
        ReleaseAccountHoldInput {
            hold_id: hold.hold_id,
            settled_at_ms: current_billing_timestamp_ms()?,
        },
    )
    .await
    {
        record_current_commercial_event(
            CommercialEventKind::HoldFailure,
            CommercialEventDimensions::default().with_result("release_failed"),
        );
        return Err(error);
    }
    Ok(())
}

fn canonical_usage_priced_settlement_amounts(
    hold: Option<&CanonicalHeldCharge>,
    token_usage: Option<TokenUsageMetrics>,
) -> Option<CanonicalChatSettlementAmounts> {
    let Some(hold) = hold else {
        return None;
    };

    if let Some(token_usage) = token_usage {
        if let Some(customer_charge) =
            estimate_usage_priced_customer_charge(&hold.model_price, token_usage)
        {
            return Some(CanonicalChatSettlementAmounts {
                usage_units: token_usage.total_tokens.max(1),
                customer_charge,
            });
        }
    }

    Some(CanonicalChatSettlementAmounts {
        usage_units: hold.estimated_usage.total_tokens.max(1),
        customer_charge: hold.estimated_quantity,
    })
}

fn canonical_music_settlement_amounts_from_model_price(
    model_price: &ModelPriceRecord,
    music_seconds: f64,
) -> Option<CanonicalChatSettlementAmounts> {
    let normalized_music_seconds = music_seconds.max(1.0);
    match model_price.price_unit.as_str() {
        "per_second_music" | "per_music_second" | "per_second" => {
            Some(CanonicalChatSettlementAmounts {
                usage_units: normalized_music_seconds.ceil() as u64,
                customer_charge: (model_price.request_price
                    + normalized_music_seconds * model_price.input_price)
                    .max(0.0),
            })
        }
        "per_track" => Some(CanonicalChatSettlementAmounts {
            usage_units: 1,
            customer_charge: (model_price.request_price + model_price.input_price).max(0.0),
        }),
        "per_request" | "request" => Some(CanonicalChatSettlementAmounts {
            usage_units: 1,
            customer_charge: model_price.request_price.max(0.0),
        }),
        _ => None,
    }
}

fn canonical_video_settlement_amounts_from_model_price(
    model_price: &ModelPriceRecord,
    video_seconds: f64,
) -> Option<CanonicalChatSettlementAmounts> {
    let normalized_video_seconds = video_seconds.max(1.0);
    match model_price.price_unit.as_str() {
        "per_minute_video" | "per_video_minute" | "per_minute" => {
            Some(CanonicalChatSettlementAmounts {
                usage_units: ((normalized_video_seconds / 60.0).ceil()).max(1.0) as u64,
                customer_charge: (model_price.request_price
                    + (normalized_video_seconds / 60.0) * model_price.input_price)
                    .max(0.0),
            })
        }
        "per_second_video" | "per_video_second" | "per_second" => {
            Some(CanonicalChatSettlementAmounts {
                usage_units: normalized_video_seconds.ceil() as u64,
                customer_charge: (model_price.request_price
                    + normalized_video_seconds * model_price.input_price)
                    .max(0.0),
            })
        }
        "per_request" | "request" => Some(CanonicalChatSettlementAmounts {
            usage_units: 1,
            customer_charge: model_price.request_price.max(0.0),
        }),
        _ => None,
    }
}

async fn annotate_canonical_request_upstream_reference(
    store: &dyn AdminStore,
    hold: &CanonicalHeldCharge,
    upstream_request_ref: &str,
    request_status: RequestStatus,
) -> anyhow::Result<()> {
    let Some(account_store) = store.account_kernel() else {
        anyhow::bail!("account kernel store is not available for request annotation");
    };
    let Some(request_meter_fact) = account_store
        .list_request_meter_facts()
        .await?
        .into_iter()
        .find(|record| record.request_id == hold.request_id)
    else {
        anyhow::bail!(
            "request meter fact {} is missing for canonical hold {}",
            hold.request_id,
            hold.hold_id
        );
    };
    let updated_request_meter_fact = request_meter_fact
        .with_upstream_request_ref(Some(upstream_request_ref.to_owned()))
        .with_request_status(request_status)
        .with_updated_at_ms(current_billing_timestamp_ms()?);
    account_store
        .insert_request_meter_fact(&updated_request_meter_fact)
        .await?;
    Ok(())
}

async fn reconcile_pending_music_reference(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    music_reference_id: &str,
    music_seconds: f64,
) -> anyhow::Result<bool> {
    let Some(account_store) = store.account_kernel() else {
        return Ok(false);
    };
    let Some(request_meter_fact) = account_store
        .list_request_meter_facts()
        .await?
        .into_iter()
        .find(|record| {
            record.capability_code == "music"
                && record.upstream_request_ref.as_deref() == Some(music_reference_id)
                && record.usage_capture_status == UsageCaptureStatus::Pending
                && matches!(
                    record.request_status,
                    RequestStatus::Pending | RequestStatus::Running
                )
        })
    else {
        return Ok(false);
    };
    let Some(model_price) = resolve_active_model_price_for_route(
        store,
        &request_meter_fact.provider_code,
        Some(request_meter_fact.channel_code.as_str()),
        &request_meter_fact.model_code,
    )
    .await?
    else {
        return Ok(false);
    };
    let Some(settlement) =
        canonical_music_settlement_amounts_from_model_price(&model_price, music_seconds)
    else {
        return Ok(false);
    };

    record_gateway_usage_for_project_with_media_and_reference_id(
        store,
        tenant_id,
        project_id,
        "music",
        &request_meter_fact.model_code,
        settlement.usage_units,
        settlement.customer_charge,
        BillingMediaMetrics {
            music_seconds,
            ..BillingMediaMetrics::default()
        },
        Some(music_reference_id),
    )
    .await?;

    reconcile_account_hold(
        account_store,
        CaptureAccountHoldInput {
            hold_id: gateway_account_kernel_hold_id(request_meter_fact.request_id)?,
            captured_quantity: settlement.customer_charge,
            provider_cost_amount: 0.0,
            retail_charge_amount: settlement.customer_charge,
            settled_at_ms: current_billing_timestamp_ms()?,
        },
    )
    .await?;

    Ok(true)
}

async fn reconcile_pending_music_response(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    response: &Value,
) -> anyhow::Result<()> {
    if let Some(music_reference_id) = response.get("id").and_then(Value::as_str) {
        if let Some(music_seconds) = completed_music_seconds_from_value(response) {
            let _ = reconcile_pending_music_reference(
                store,
                tenant_id,
                project_id,
                music_reference_id,
                music_seconds,
            )
            .await?;
        }
    }

    if let Some(items) = response.get("data").and_then(Value::as_array) {
        for item in items {
            let Some(music_reference_id) = item.get("id").and_then(Value::as_str) else {
                continue;
            };
            let Some(music_seconds) = completed_music_seconds_from_value(item) else {
                continue;
            };
            let _ = reconcile_pending_music_reference(
                store,
                tenant_id,
                project_id,
                music_reference_id,
                music_seconds,
            )
            .await?;
        }
    }

    Ok(())
}

async fn reconcile_pending_video_reference(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    video_reference_id: &str,
    video_seconds: f64,
) -> anyhow::Result<bool> {
    let Some(account_store) = store.account_kernel() else {
        return Ok(false);
    };
    let Some(request_meter_fact) = account_store
        .list_request_meter_facts()
        .await?
        .into_iter()
        .find(|record| {
            record.capability_code == "videos"
                && record.upstream_request_ref.as_deref() == Some(video_reference_id)
                && record.usage_capture_status == UsageCaptureStatus::Pending
                && matches!(
                    record.request_status,
                    RequestStatus::Pending | RequestStatus::Running
                )
        })
    else {
        return Ok(false);
    };
    let Some(model_price) = resolve_active_model_price_for_route(
        store,
        &request_meter_fact.provider_code,
        Some(request_meter_fact.channel_code.as_str()),
        &request_meter_fact.model_code,
    )
    .await?
    else {
        return Ok(false);
    };
    let Some(settlement) =
        canonical_video_settlement_amounts_from_model_price(&model_price, video_seconds)
    else {
        return Ok(false);
    };

    record_gateway_usage_for_project_with_media_and_reference_id(
        store,
        tenant_id,
        project_id,
        "videos",
        &request_meter_fact.model_code,
        settlement.usage_units,
        settlement.customer_charge,
        BillingMediaMetrics {
            video_seconds,
            ..BillingMediaMetrics::default()
        },
        Some(video_reference_id),
    )
    .await?;

    reconcile_account_hold(
        account_store,
        CaptureAccountHoldInput {
            hold_id: gateway_account_kernel_hold_id(request_meter_fact.request_id)?,
            captured_quantity: settlement.customer_charge,
            provider_cost_amount: 0.0,
            retail_charge_amount: settlement.customer_charge,
            settled_at_ms: current_billing_timestamp_ms()?,
        },
    )
    .await?;

    Ok(true)
}

async fn reconcile_pending_video_response(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    response: &Value,
) -> anyhow::Result<()> {
    if let Some(video_reference_id) = response.get("id").and_then(Value::as_str) {
        if let Some(video_seconds) = completed_video_seconds_from_value(response) {
            let _ = reconcile_pending_video_reference(
                store,
                tenant_id,
                project_id,
                video_reference_id,
                video_seconds,
            )
            .await?;
        }
    }

    if let Some(items) = response.get("data").and_then(Value::as_array) {
        for item in items {
            let Some(video_reference_id) = item.get("id").and_then(Value::as_str) else {
                continue;
            };
            let Some(video_seconds) = completed_video_seconds_from_value(item) else {
                continue;
            };
            let _ = reconcile_pending_video_reference(
                store,
                tenant_id,
                project_id,
                video_reference_id,
                video_seconds,
            )
            .await?;
        }
    }

    Ok(())
}

async fn resolve_active_model_price_for_route(
    store: &dyn AdminStore,
    provider_id: &str,
    channel_id: Option<&str>,
    model_id: &str,
) -> anyhow::Result<Option<ModelPriceRecord>> {
    let prices = store
        .list_model_prices()
        .await?
        .into_iter()
        .filter(|price| {
            price.is_active && price.proxy_provider_id == provider_id && price.model_id == model_id
        })
        .collect::<Vec<_>>();

    if let Some(channel_id) = channel_id {
        if let Some(price) = prices
            .iter()
            .find(|price| price.channel_id == channel_id)
            .cloned()
        {
            return Ok(Some(price));
        }
    }

    Ok(prices.into_iter().next())
}

fn estimate_chat_completion_usage_metrics(
    request: &CreateChatCompletionRequest,
) -> TokenUsageMetrics {
    let input_tokens = request
        .messages
        .iter()
        .map(estimate_chat_message_tokens)
        .sum::<u64>()
        .max(DEFAULT_CHAT_COMPLETION_ESTIMATED_INPUT_TOKENS);
    let output_tokens = json_u64(request.extra.get("max_completion_tokens"))
        .or_else(|| json_u64(request.extra.get("max_tokens")))
        .unwrap_or(DEFAULT_CHAT_COMPLETION_ESTIMATED_OUTPUT_TOKENS)
        .max(DEFAULT_CHAT_COMPLETION_ESTIMATED_OUTPUT_TOKENS);

    TokenUsageMetrics {
        input_tokens,
        output_tokens,
        total_tokens: input_tokens.saturating_add(output_tokens),
    }
}

fn estimate_moderation_usage_metrics(request: &CreateModerationRequest) -> TokenUsageMetrics {
    let input_tokens = approximate_token_count_from_value(&request.input)
        .max(DEFAULT_MODERATIONS_ESTIMATED_INPUT_TOKENS);

    TokenUsageMetrics {
        input_tokens,
        output_tokens: 0,
        total_tokens: input_tokens,
    }
}

fn estimate_response_usage_metrics(request: &CreateResponseRequest) -> TokenUsageMetrics {
    let input_tokens = approximate_token_count_from_value(&request.input)
        .max(DEFAULT_RESPONSES_ESTIMATED_INPUT_TOKENS);
    let output_tokens = DEFAULT_RESPONSES_ESTIMATED_OUTPUT_TOKENS;

    TokenUsageMetrics {
        input_tokens,
        output_tokens,
        total_tokens: input_tokens.saturating_add(output_tokens),
    }
}

fn estimate_completion_usage_metrics(request: &CreateCompletionRequest) -> TokenUsageMetrics {
    let input_tokens = approximate_token_count_from_text(&request.prompt)
        .max(DEFAULT_COMPLETIONS_ESTIMATED_INPUT_TOKENS);
    let output_tokens = DEFAULT_COMPLETIONS_ESTIMATED_OUTPUT_TOKENS;

    TokenUsageMetrics {
        input_tokens,
        output_tokens,
        total_tokens: input_tokens.saturating_add(output_tokens),
    }
}

fn estimate_embedding_usage_metrics(request: &CreateEmbeddingRequest) -> TokenUsageMetrics {
    let input_tokens = approximate_token_count_from_value(&request.input)
        .max(DEFAULT_EMBEDDINGS_ESTIMATED_INPUT_TOKENS);

    TokenUsageMetrics {
        input_tokens,
        output_tokens: 0,
        total_tokens: input_tokens,
    }
}

fn estimate_chat_message_tokens(message: &ChatMessageInput) -> u64 {
    approximate_token_count_from_text(&message.role)
        .saturating_add(approximate_token_count_from_value(&message.content))
        .saturating_add(
            message
                .extra
                .values()
                .map(approximate_token_count_from_value)
                .sum::<u64>(),
        )
        .saturating_add(8)
}

fn approximate_token_count_from_value(value: &Value) -> u64 {
    match value {
        Value::Null => 0,
        Value::Bool(_) => 1,
        Value::Number(number) => approximate_token_count_from_text(&number.to_string()),
        Value::String(text) => approximate_token_count_from_text(text),
        Value::Array(values) => values.iter().map(approximate_token_count_from_value).sum(),
        Value::Object(map) => map.values().map(approximate_token_count_from_value).sum(),
    }
}

fn approximate_token_count_from_text(text: &str) -> u64 {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        0
    } else {
        ((trimmed.chars().count() as u64) + 3) / 4
    }
}

fn estimate_usage_priced_customer_charge(
    model_price: &ModelPriceRecord,
    usage: TokenUsageMetrics,
) -> Option<f64> {
    let token_scale = match model_price.price_unit.as_str() {
        "per_1m_tokens" => Some(1_000_000.0),
        "per_1k_tokens" => Some(1_000.0),
        "per_token" => Some(1.0),
        "per_request" | "request" => None,
        _ => return None,
    };

    let mut customer_charge = model_price.request_price;
    if let Some(token_scale) = token_scale {
        customer_charge += (usage.input_tokens as f64 * model_price.input_price) / token_scale;
        customer_charge += (usage.output_tokens as f64 * model_price.output_price) / token_scale;
    }

    Some(customer_charge.max(0.0))
}

fn gateway_account_kernel_request_id(request_id: &str, capability: &str) -> u64 {
    let mut hash = 14_695_981_039_346_656_037_u64;
    for byte in capability
        .bytes()
        .chain(std::iter::once(b':'))
        .chain(request_id.bytes())
    {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(1_099_511_628_211);
    }

    // Keep request, hold, ledger, and allocation identifiers inside the signed 64-bit
    // database range used by SQLite/PostgreSQL storage implementations.
    (hash % GATEWAY_ACCOUNT_KERNEL_REQUEST_ID_MODULUS)
        .saturating_add(GATEWAY_ACCOUNT_KERNEL_REQUEST_ID_BASE)
}

fn gateway_account_kernel_hold_id(request_id: u64) -> anyhow::Result<u64> {
    request_id
        .checked_mul(10)
        .and_then(|value| value.checked_add(9))
        .ok_or_else(|| anyhow::anyhow!("derived hold identifier overflow for {}", request_id))
}

fn bad_gateway_openai_response(message: impl Into<String>) -> Response {
    let mut error = OpenAiErrorResponse::new(message, "server_error");
    error.error.code = Some("bad_gateway".to_owned());
    (StatusCode::BAD_GATEWAY, Json(error)).into_response()
}

fn openai_response_for_relay_error(error: &anyhow::Error, message: impl Into<String>) -> Response {
    if let Some(admission_error) = error.downcast_ref::<GatewayTrafficAdmissionError>() {
        commercial_admission_error_response(admission_error)
    } else {
        bad_gateway_openai_response(message)
    }
}

fn invalid_request_openai_response(
    message: impl Into<String>,
    code: impl Into<String>,
) -> Response {
    let mut error = OpenAiErrorResponse::new(message, "invalid_request_error");
    error.error.code = Some(code.into());
    (StatusCode::BAD_REQUEST, Json(error)).into_response()
}

fn quota_exceeded_message(project_id: &str, evaluation: &QuotaCheckResult) -> String {
    match (evaluation.policy_id.as_deref(), evaluation.limit_units) {
        (Some(policy_id), Some(limit_units)) => format!(
            "Quota exceeded for project {project_id} under policy {policy_id}: requested {} units with {} already used against a limit of {limit_units}.",
            evaluation.requested_units, evaluation.used_units,
        ),
        (_, Some(limit_units)) => format!(
            "Quota exceeded for project {project_id}: requested {} units with {} already used against a limit of {limit_units}.",
            evaluation.requested_units, evaluation.used_units,
        ),
        _ => format!(
            "Quota exceeded for project {project_id}: requested {} units with {} already used.",
            evaluation.requested_units, evaluation.used_units,
        ),
    }
}

async fn record_gateway_usage_for_project(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    model: &str,
    units: u64,
    amount: f64,
) -> anyhow::Result<()> {
    record_gateway_usage_for_project_with_route_key_and_reference_id(
        store, tenant_id, project_id, capability, model, model, units, amount, None,
    )
    .await
}

#[derive(Debug, Clone, Copy, Default)]
struct TokenUsageMetrics {
    input_tokens: u64,
    output_tokens: u64,
    total_tokens: u64,
}

#[derive(Debug, Clone, Copy, Default)]
struct BillingMediaMetrics {
    image_count: u64,
    audio_seconds: f64,
    video_seconds: f64,
    music_seconds: f64,
}

fn json_u64(value: Option<&Value>) -> Option<u64> {
    value.and_then(|value| value.as_u64())
}

impl ResponsesSseSettlementTracker {
    fn ingest_chunk(&mut self, chunk: &Bytes) {
        self.buffer.extend_from_slice(chunk);
        while let Some(frame) = drain_next_sse_frame(&mut self.buffer) {
            self.observe_frame(&frame);
        }
    }

    fn finish(&mut self) {
        if !self.buffer.is_empty() {
            let frame = std::mem::take(&mut self.buffer);
            self.observe_frame(&frame);
        }
    }

    fn observe_frame(&mut self, frame: &[u8]) {
        let Some(payload) = sse_data_payload(frame) else {
            return;
        };
        if payload == "[DONE]" {
            return;
        }
        let Ok(value) = serde_json::from_str::<Value>(&payload) else {
            return;
        };
        if value.get("type").and_then(Value::as_str) != Some("response.completed") {
            return;
        }
        let Some(response) = value.get("response") else {
            return;
        };
        self.completed_event = Some(ResponsesCompletedEvent {
            token_usage: extract_token_usage_metrics(response),
            reference_id: response_usage_id_or_single_data_item_id(response).map(str::to_owned),
        });
    }
}

impl ChatCompletionsSseSettlementTracker {
    fn ingest_chunk(&mut self, chunk: &Bytes) {
        self.buffer.extend_from_slice(chunk);
        while let Some(frame) = drain_next_sse_frame(&mut self.buffer) {
            self.observe_frame(&frame);
        }
    }

    fn finish(&mut self) {
        if !self.buffer.is_empty() {
            let frame = std::mem::take(&mut self.buffer);
            self.observe_frame(&frame);
        }
    }

    fn observe_frame(&mut self, frame: &[u8]) {
        let Some(payload) = sse_data_payload(frame) else {
            return;
        };
        if payload == "[DONE]" {
            self.saw_done = true;
            return;
        }
        let Ok(value) = serde_json::from_str::<Value>(&payload) else {
            return;
        };
        if self.reference_id.is_none() {
            self.reference_id = response_usage_id_or_single_data_item_id(&value).map(str::to_owned);
        }
        if let Some(token_usage) = extract_token_usage_metrics(&value) {
            self.token_usage = Some(token_usage);
        }
    }
}

fn drain_next_sse_frame(buffer: &mut Vec<u8>) -> Option<Vec<u8>> {
    let mut frame_end = None;
    let mut delimiter_len = 0;
    let mut index = 0;
    while index + 1 < buffer.len() {
        if buffer[index] == b'\n' && buffer[index + 1] == b'\n' {
            frame_end = Some(index);
            delimiter_len = 2;
            break;
        }
        if index + 3 < buffer.len()
            && buffer[index] == b'\r'
            && buffer[index + 1] == b'\n'
            && buffer[index + 2] == b'\r'
            && buffer[index + 3] == b'\n'
        {
            frame_end = Some(index);
            delimiter_len = 4;
            break;
        }
        index += 1;
    }
    let frame_end = frame_end?;
    let frame = buffer[..frame_end].to_vec();
    buffer.drain(..frame_end + delimiter_len);
    Some(frame)
}

fn sse_data_payload(frame: &[u8]) -> Option<String> {
    let text = std::str::from_utf8(frame).ok()?;
    let payload = text
        .lines()
        .filter_map(|line| {
            let line = line.trim_end_matches('\r');
            line.strip_prefix("data:").map(|value| value.trim_start())
        })
        .collect::<Vec<_>>();
    if payload.is_empty() {
        None
    } else {
        Some(payload.join("\n"))
    }
}

fn extract_token_usage_metrics(response: &Value) -> Option<TokenUsageMetrics> {
    if let Some(usage) = response.get("usage") {
        let input_tokens = json_u64(usage.get("prompt_tokens"))
            .or_else(|| json_u64(usage.get("input_tokens")))
            .unwrap_or(0);
        let output_tokens = json_u64(usage.get("completion_tokens"))
            .or_else(|| json_u64(usage.get("output_tokens")))
            .unwrap_or(0);
        let total_tokens = json_u64(usage.get("total_tokens"))
            .unwrap_or_else(|| input_tokens.saturating_add(output_tokens));

        if input_tokens > 0 || output_tokens > 0 || total_tokens > 0 {
            return Some(TokenUsageMetrics {
                input_tokens,
                output_tokens,
                total_tokens,
            });
        }
    }

    let input_tokens = json_u64(response.get("input_tokens")).unwrap_or(0);
    let output_tokens = json_u64(response.get("output_tokens")).unwrap_or(0);
    let total_tokens = json_u64(response.get("total_tokens"))
        .unwrap_or_else(|| input_tokens.saturating_add(output_tokens));

    if input_tokens > 0 || output_tokens > 0 || total_tokens > 0 {
        return Some(TokenUsageMetrics {
            input_tokens,
            output_tokens,
            total_tokens,
        });
    }

    None
}

fn response_usage_id_or_single_data_item_id(response: &Value) -> Option<&str> {
    response.get("id").and_then(Value::as_str).or_else(|| {
        match response
            .get("data")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
        {
            Some([item]) => item.get("id").and_then(Value::as_str),
            _ => None,
        }
    })
}

fn image_count_from_response(response: &Value) -> u64 {
    response
        .get("data")
        .and_then(Value::as_array)
        .and_then(|data| u64::try_from(data.len()).ok())
        .unwrap_or(0)
}

fn json_f64(value: Option<&Value>) -> Option<f64> {
    value.and_then(|value| {
        value
            .as_f64()
            .or_else(|| value.as_str().and_then(|text| text.parse::<f64>().ok()))
    })
}

fn video_seconds_from_value(value: &Value) -> Option<f64> {
    json_f64(value.get("duration_seconds"))
        .or_else(|| json_f64(value.get("duration")))
        .or_else(|| {
            value
                .get("metadata")
                .and_then(|metadata| json_f64(metadata.get("duration_seconds")))
        })
        .filter(|seconds| *seconds > 0.0)
}

fn completed_video_seconds_from_value(value: &Value) -> Option<f64> {
    if matches!(
        value.get("status").and_then(Value::as_str),
        Some("pending" | "queued" | "processing" | "running" | "in_progress")
    ) {
        return None;
    }
    video_seconds_from_value(value)
}

fn completed_video_seconds_from_response(response: &Value) -> Option<f64> {
    completed_video_seconds_from_value(response).or_else(|| {
        response
            .get("data")
            .and_then(Value::as_array)
            .and_then(|data| match data.as_slice() {
                [item] => completed_video_seconds_from_value(item),
                _ => None,
            })
    })
}

fn music_seconds_from_response(response: &Value) -> f64 {
    response
        .get("duration_seconds")
        .and_then(Value::as_f64)
        .or_else(|| {
            response
                .get("data")
                .and_then(Value::as_array)
                .and_then(|data| match data.as_slice() {
                    [item] => item.get("duration_seconds").and_then(Value::as_f64),
                    _ => None,
                })
        })
        .unwrap_or(0.0)
}

fn completed_music_seconds_from_value(value: &Value) -> Option<f64> {
    if matches!(
        value.get("status").and_then(Value::as_str),
        Some("pending" | "queued" | "processing" | "running" | "in_progress")
    ) {
        return None;
    }
    value
        .get("duration_seconds")
        .and_then(Value::as_f64)
        .filter(|seconds| *seconds > 0.0)
}

fn completed_music_seconds_from_response(response: &Value) -> Option<f64> {
    completed_music_seconds_from_value(response).or_else(|| {
        response
            .get("data")
            .and_then(Value::as_array)
            .and_then(|data| match data.as_slice() {
                [item] => completed_music_seconds_from_value(item),
                _ => None,
            })
    })
}

fn music_billing_units(music_seconds: f64) -> u64 {
    music_seconds.max(1.0).ceil() as u64
}

fn music_billing_amount(music_seconds: f64) -> f64 {
    music_seconds.max(1.0) * 0.001
}

fn current_billing_timestamp_ms() -> anyhow::Result<u64> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64)
}

fn build_gateway_billing_event_id(
    project_id: &str,
    capability: &str,
    route_key: &str,
    provider_id: &str,
    reference_id: Option<&str>,
    created_at_ms: u64,
) -> String {
    format!(
        "bill_evt:{project_id}:{capability}:{route_key}:{provider_id}:{}:{created_at_ms}",
        reference_id.unwrap_or("none")
    )
}

fn billing_modality_for_capability(capability: &str) -> &'static str {
    match capability {
        "responses" => "multimodal",
        "images" | "image_edits" | "image_variations" => "image",
        "audio" | "speech" | "transcriptions" | "translations" => "audio",
        "videos" => "video",
        "music" => "music",
        _ => "text",
    }
}

async fn record_gateway_usage_for_project_with_route_key_and_reference_id(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    route_key: &str,
    usage_model: &str,
    units: u64,
    amount: f64,
    reference_id: Option<&str>,
) -> anyhow::Result<()> {
    record_gateway_usage_for_project_with_route_key_and_tokens_and_reference(
        store,
        tenant_id,
        project_id,
        capability,
        route_key,
        usage_model,
        units,
        amount,
        None,
        reference_id,
    )
    .await
}

async fn record_gateway_usage_for_project_with_media_and_reference_id(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    model: &str,
    units: u64,
    amount: f64,
    media_metrics: BillingMediaMetrics,
    reference_id: Option<&str>,
) -> anyhow::Result<()> {
    record_gateway_usage_for_project_with_route_key_and_tokens_reference_and_media(
        store,
        tenant_id,
        project_id,
        capability,
        model,
        model,
        units,
        amount,
        None,
        reference_id,
        media_metrics,
    )
    .await
}

async fn record_gateway_usage_for_project_with_route_key(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    route_key: &str,
    usage_model: &str,
    units: u64,
    amount: f64,
) -> anyhow::Result<()> {
    record_gateway_usage_for_project_with_route_key_and_tokens_and_reference(
        store,
        tenant_id,
        project_id,
        capability,
        route_key,
        usage_model,
        units,
        amount,
        None,
        None,
    )
    .await
}

async fn record_gateway_usage_for_project_with_route_key_and_tokens_and_reference(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    route_key: &str,
    usage_model: &str,
    units: u64,
    amount: f64,
    token_usage: Option<TokenUsageMetrics>,
    reference_id: Option<&str>,
) -> anyhow::Result<()> {
    record_gateway_usage_for_project_with_route_key_and_tokens_reference_and_media(
        store,
        tenant_id,
        project_id,
        capability,
        route_key,
        usage_model,
        units,
        amount,
        token_usage,
        reference_id,
        BillingMediaMetrics::default(),
    )
    .await
}

async fn record_gateway_usage_for_project_with_route_key_and_tokens_reference_and_media(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    route_key: &str,
    usage_model: &str,
    units: u64,
    amount: f64,
    token_usage: Option<TokenUsageMetrics>,
    reference_id: Option<&str>,
    media_metrics: BillingMediaMetrics,
) -> anyhow::Result<()> {
    let usage_context = planned_execution_usage_context_for_route(
        store, tenant_id, project_id, capability, route_key,
    )
    .await?;
    let token_usage = token_usage.unwrap_or_default();
    let request_context = current_gateway_request_context();
    let api_key_hash = request_context
        .as_ref()
        .map(|context| context.api_key_hash().to_owned());
    let billing_settlement = resolve_gateway_billing_settlement(
        store,
        request_context
            .as_ref()
            .and_then(|context| context.api_key_group_id()),
        usage_context.reference_amount,
        amount,
    )
    .await?;
    annotate_current_http_metrics(|dimensions| {
        dimensions.tenant = Some(tenant_id.to_owned());
        dimensions.model = Some(usage_model.to_owned());
        dimensions.provider = Some(usage_context.provider_id.clone());
        dimensions.billing_mode = Some(billing_settlement.accounting_mode.as_str().to_owned());
    });
    let latency_ms = current_gateway_request_latency_ms().or(usage_context.latency_ms);
    persist_usage_record_with_tokens_and_facts(
        store,
        project_id,
        usage_model,
        &usage_context.provider_id,
        units,
        amount,
        token_usage.input_tokens,
        token_usage.output_tokens,
        token_usage.total_tokens,
        api_key_hash.as_deref(),
        usage_context.channel_id.as_deref(),
        latency_ms,
        usage_context.reference_amount,
    )
    .await?;
    let created_at_ms = current_billing_timestamp_ms()?;
    let billing_event = create_billing_event(CreateBillingEventInput {
        event_id: &build_gateway_billing_event_id(
            project_id,
            capability,
            route_key,
            &usage_context.provider_id,
            reference_id,
            created_at_ms,
        ),
        tenant_id,
        project_id,
        api_key_group_id: usage_context.api_key_group_id.as_deref(),
        capability,
        route_key,
        usage_model,
        provider_id: &usage_context.provider_id,
        accounting_mode: billing_settlement.accounting_mode,
        operation_kind: capability,
        modality: billing_modality_for_capability(capability),
        api_key_hash: api_key_hash.as_deref(),
        channel_id: usage_context.channel_id.as_deref(),
        reference_id,
        latency_ms,
        units,
        request_count: 1,
        input_tokens: token_usage.input_tokens,
        output_tokens: token_usage.output_tokens,
        total_tokens: token_usage.total_tokens,
        cache_read_tokens: 0,
        cache_write_tokens: 0,
        image_count: media_metrics.image_count,
        audio_seconds: media_metrics.audio_seconds,
        video_seconds: media_metrics.video_seconds,
        music_seconds: media_metrics.music_seconds,
        upstream_cost: billing_settlement.upstream_cost,
        customer_charge: billing_settlement.customer_charge,
        applied_routing_profile_id: usage_context.applied_routing_profile_id.as_deref(),
        compiled_routing_snapshot_id: usage_context.compiled_routing_snapshot_id.as_deref(),
        fallback_reason: usage_context.fallback_reason.as_deref(),
        created_at_ms,
    })?;
    persist_billing_event(store, &billing_event).await?;
    persist_ledger_entry(store, project_id, units, amount).await?;
    Ok(())
}

async fn resolve_gateway_billing_settlement(
    store: &dyn AdminStore,
    api_key_group_id: Option<&str>,
    upstream_cost: Option<f64>,
    customer_charge: f64,
) -> anyhow::Result<BillingPolicyExecutionResult> {
    let group_default_accounting_mode =
        load_api_key_group_default_accounting_mode(store, api_key_group_id).await?;
    let registry = builtin_billing_policy_registry();
    let plugin = registry
        .resolve(GROUP_DEFAULT_BILLING_POLICY_ID)
        .expect("builtin group-default billing policy plugin must exist");

    plugin.execute(BillingPolicyExecutionInput {
        api_key_group_default_accounting_mode: group_default_accounting_mode.as_deref(),
        default_accounting_mode: BillingAccountingMode::PlatformCredit,
        upstream_cost,
        customer_charge,
    })
}

async fn load_api_key_group_default_accounting_mode(
    store: &dyn AdminStore,
    api_key_group_id: Option<&str>,
) -> anyhow::Result<Option<String>> {
    let Some(api_key_group_id) = api_key_group_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(None);
    };

    Ok(store
        .find_api_key_group(api_key_group_id)
        .await?
        .and_then(|group| group.default_accounting_mode))
}

async fn relay_stateless_json_request(
    request_context: &StatelessGatewayRequest,
    request: ProviderRequest<'_>,
) -> anyhow::Result<Option<Value>> {
    let options = ProviderRequestOptions::default();
    relay_stateless_json_request_with_options(request_context, request, &options).await
}

async fn relay_stateless_json_request_with_options(
    request_context: &StatelessGatewayRequest,
    request: ProviderRequest<'_>,
    options: &ProviderRequestOptions,
) -> anyhow::Result<Option<Value>> {
    let Some(upstream) = request_context.upstream() else {
        return Ok(None);
    };

    execute_json_provider_request_with_runtime_and_options(
        upstream.runtime_key(),
        upstream.base_url(),
        upstream.api_key(),
        request,
        options,
    )
    .await
}

async fn relay_stateless_stream_request(
    request_context: &StatelessGatewayRequest,
    request: ProviderRequest<'_>,
) -> anyhow::Result<Option<ProviderStreamOutput>> {
    let options = ProviderRequestOptions::default();
    relay_stateless_stream_request_with_options(request_context, request, &options).await
}

async fn relay_stateless_stream_request_with_options(
    request_context: &StatelessGatewayRequest,
    request: ProviderRequest<'_>,
    options: &ProviderRequestOptions,
) -> anyhow::Result<Option<ProviderStreamOutput>> {
    let Some(upstream) = request_context.upstream() else {
        return Ok(None);
    };

    execute_stream_provider_request_with_runtime_and_options(
        upstream.runtime_key(),
        upstream.base_url(),
        upstream.api_key(),
        request,
        options,
    )
    .await
}

fn local_speech_response(
    tenant_id: &str,
    project_id: &str,
    request: &CreateSpeechRequest,
) -> Response {
    let speech = match create_speech_response(tenant_id, project_id, request) {
        Ok(speech) => speech,
        Err(error) => {
            return invalid_request_openai_response(error.to_string(), "invalid_response_format");
        }
    };
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

fn local_chat_completion_stream_response(model: &str) -> Response {
    upstream_passthrough_response(local_chat_completion_stream_output(model))
}

fn local_chat_completion_stream_output(model: &str) -> ProviderStreamOutput {
    let chunk = serde_json::json!({
        "id":"chatcmpl_1",
        "object":"chat.completion.chunk",
        "model": model,
        "choices":[
            {
                "index": 0,
                "delta": {
                    "content": ""
                },
                "finish_reason": "stop"
            }
        ]
    })
    .to_string();
    let body = format!("{}{}", SseFrame::data(&chunk), SseFrame::data("[DONE]"));
    ProviderStreamOutput::new(
        "text/event-stream",
        stream::once(async move { Ok::<Bytes, io::Error>(Bytes::from(body)) }),
    )
}

fn local_response_stream_response(response_id: &str, model: &str) -> Response {
    upstream_passthrough_response(local_response_stream_output(response_id, model))
}

fn local_response_stream_output(response_id: &str, model: &str) -> ProviderStreamOutput {
    let created = serde_json::json!({
        "type":"response.created",
        "response": {
            "id": response_id,
            "object": "response",
            "model": model
        }
    })
    .to_string();
    let delta = serde_json::json!({
        "type":"response.output_text.delta",
        "delta":"hello"
    })
    .to_string();
    let completed = serde_json::json!({
        "type":"response.completed",
        "response": {
            "id": response_id
        }
    })
    .to_string();
    let body = format!(
        "{}{}{}",
        SseFrame::data(&created),
        SseFrame::data(&delta),
        SseFrame::data(&completed)
    );
    ProviderStreamOutput::new(
        "text/event-stream",
        stream::once(async move { Ok::<Bytes, io::Error>(Bytes::from(body)) }),
    )
}

fn spawn_responses_stream_with_settlement(
    mut upstream: sdkwork_api_provider_core::ProviderByteStream,
    settlement_context: UsagePricedStreamSettlementContext,
) -> ReceiverStream<Result<Bytes, io::Error>> {
    let (sender, receiver) = mpsc::channel(16);
    tokio::spawn(async move {
        let mut tracker = ResponsesSseSettlementTracker::default();
        let mut client_open = true;

        // Continue draining the upstream stream even after client disconnect so settlement can
        // still complete and held balance is not stranded.
        while let Some(item) = upstream.next().await {
            match item {
                Ok(chunk) => {
                    tracker.ingest_chunk(&chunk);
                    if client_open && sender.send(Ok(chunk)).await.is_err() {
                        client_open = false;
                    }
                }
                Err(error) => {
                    if client_open {
                        let forwarded = io::Error::new(error.kind(), error.to_string());
                        let _ = sender.send(Err(forwarded)).await;
                    }
                    let _ =
                        finalize_responses_stream_settlement(settlement_context, tracker, false)
                            .await;
                    return;
                }
            }
        }

        let _ = finalize_responses_stream_settlement(settlement_context, tracker, true).await;
    });
    ReceiverStream::new(receiver)
}

async fn finalize_responses_stream_settlement(
    settlement_context: UsagePricedStreamSettlementContext,
    mut tracker: ResponsesSseSettlementTracker,
    completed_normally: bool,
) -> anyhow::Result<()> {
    if !completed_normally {
        if let Some(hold) = settlement_context.canonical_hold.as_ref() {
            let _ = settle_canonical_chat_request_release(settlement_context.store.as_ref(), hold)
                .await;
        }
        return Ok(());
    }

    tracker.finish();
    let Some(completed_event) = tracker.completed_event else {
        if let Some(hold) = settlement_context.canonical_hold.as_ref() {
            let _ = settle_canonical_chat_request_release(settlement_context.store.as_ref(), hold)
                .await;
        }
        return Ok(());
    };

    let canonical_settlement = canonical_usage_priced_settlement_amounts(
        settlement_context.canonical_hold.as_ref(),
        completed_event.token_usage,
    );
    let usage_units = canonical_settlement
        .as_ref()
        .map(|settlement| settlement.usage_units)
        .unwrap_or(RESPONSES_HOLD_UNITS);
    let amount = canonical_settlement
        .as_ref()
        .map(|settlement| settlement.customer_charge)
        .unwrap_or(RESPONSES_RETAIL_CHARGE);

    let usage_result =
        if completed_event.token_usage.is_some() || completed_event.reference_id.is_some() {
            record_gateway_usage_for_project_with_route_key_and_tokens_and_reference(
                settlement_context.store.as_ref(),
                &settlement_context.tenant_id,
                &settlement_context.project_id,
                "responses",
                &settlement_context.model,
                &settlement_context.model,
                usage_units,
                amount,
                completed_event.token_usage,
                completed_event.reference_id.as_deref(),
            )
            .await
        } else {
            record_gateway_usage_for_project(
                settlement_context.store.as_ref(),
                &settlement_context.tenant_id,
                &settlement_context.project_id,
                "responses",
                &settlement_context.model,
                usage_units,
                amount,
            )
            .await
        };

    if usage_result.is_err() {
        if let Some(hold) = settlement_context.canonical_hold.as_ref() {
            let _ = settle_canonical_chat_request_release(settlement_context.store.as_ref(), hold)
                .await;
        }
        return usage_result;
    }

    if let (Some(hold), Some(settlement)) = (
        settlement_context.canonical_hold.as_ref(),
        canonical_settlement.as_ref(),
    ) {
        if let Err(error) = settle_canonical_chat_request_capture(
            settlement_context.store.as_ref(),
            hold,
            settlement,
        )
        .await
        {
            let _ = settle_canonical_chat_request_release(settlement_context.store.as_ref(), hold)
                .await;
            return Err(error);
        }
    }

    Ok(())
}

fn spawn_chat_completions_stream_with_settlement(
    mut upstream: sdkwork_api_provider_core::ProviderByteStream,
    settlement_context: UsagePricedStreamSettlementContext,
) -> ReceiverStream<Result<Bytes, io::Error>> {
    let (sender, receiver) = mpsc::channel(16);
    tokio::spawn(async move {
        let mut tracker = ChatCompletionsSseSettlementTracker::default();
        let mut client_open = true;

        // Continue draining the upstream stream even after client disconnect so settlement can
        // still complete and held balance is not stranded.
        while let Some(item) = upstream.next().await {
            match item {
                Ok(chunk) => {
                    tracker.ingest_chunk(&chunk);
                    if client_open && sender.send(Ok(chunk)).await.is_err() {
                        client_open = false;
                    }
                }
                Err(error) => {
                    if client_open {
                        let forwarded = io::Error::new(error.kind(), error.to_string());
                        let _ = sender.send(Err(forwarded)).await;
                    }
                    let _ = finalize_chat_completions_stream_settlement(
                        settlement_context,
                        tracker,
                        false,
                    )
                    .await;
                    return;
                }
            }
        }

        let _ =
            finalize_chat_completions_stream_settlement(settlement_context, tracker, true).await;
    });
    ReceiverStream::new(receiver)
}

async fn finalize_chat_completions_stream_settlement(
    settlement_context: UsagePricedStreamSettlementContext,
    mut tracker: ChatCompletionsSseSettlementTracker,
    completed_normally: bool,
) -> anyhow::Result<()> {
    if !completed_normally {
        if let Some(hold) = settlement_context.canonical_hold.as_ref() {
            let _ = settle_canonical_chat_request_release(settlement_context.store.as_ref(), hold)
                .await;
        }
        return Ok(());
    }

    tracker.finish();
    if !tracker.saw_done {
        if let Some(hold) = settlement_context.canonical_hold.as_ref() {
            let _ = settle_canonical_chat_request_release(settlement_context.store.as_ref(), hold)
                .await;
        }
        return Ok(());
    }

    let canonical_settlement = canonical_usage_priced_settlement_amounts(
        settlement_context.canonical_hold.as_ref(),
        tracker.token_usage,
    );
    let usage_units = canonical_settlement
        .as_ref()
        .map(|settlement| settlement.usage_units)
        .unwrap_or(CHAT_COMPLETION_HOLD_UNITS);
    let amount = canonical_settlement
        .as_ref()
        .map(|settlement| settlement.customer_charge)
        .unwrap_or(CHAT_COMPLETION_RETAIL_CHARGE);

    let usage_result = if tracker.token_usage.is_some() || tracker.reference_id.is_some() {
        record_gateway_usage_for_project_with_route_key_and_tokens_and_reference(
            settlement_context.store.as_ref(),
            &settlement_context.tenant_id,
            &settlement_context.project_id,
            "chat_completion",
            &settlement_context.model,
            &settlement_context.model,
            usage_units,
            amount,
            tracker.token_usage,
            tracker.reference_id.as_deref(),
        )
        .await
    } else {
        record_gateway_usage_for_project(
            settlement_context.store.as_ref(),
            &settlement_context.tenant_id,
            &settlement_context.project_id,
            "chat_completion",
            &settlement_context.model,
            usage_units,
            amount,
        )
        .await
    };

    if usage_result.is_err() {
        if let Some(hold) = settlement_context.canonical_hold.as_ref() {
            let _ = settle_canonical_chat_request_release(settlement_context.store.as_ref(), hold)
                .await;
        }
        return usage_result;
    }

    if let (Some(hold), Some(settlement)) = (
        settlement_context.canonical_hold.as_ref(),
        canonical_settlement.as_ref(),
    ) {
        if let Err(error) = settle_canonical_chat_request_capture(
            settlement_context.store.as_ref(),
            hold,
            settlement,
        )
        .await
        {
            let _ = settle_canonical_chat_request_release(settlement_context.store.as_ref(), hold)
                .await;
            return Err(error);
        }
    }

    Ok(())
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

async fn parse_image_edit_request(
    mut multipart: Multipart,
) -> Result<CreateImageEditRequest, Response> {
    let mut model = None;
    let mut prompt = None;
    let mut image = None;
    let mut mask = None;
    let mut n = None;
    let mut quality = None;
    let mut response_format = None;
    let mut size = None;
    let mut user = None;

    while let Some(field) = multipart.next_field().await.map_err(bad_multipart)? {
        match field.name() {
            Some("model") => model = Some(field.text().await.map_err(bad_multipart)?),
            Some("prompt") => prompt = Some(field.text().await.map_err(bad_multipart)?),
            Some("image") => image = Some(parse_image_upload_field(field).await?),
            Some("mask") => mask = Some(parse_image_upload_field(field).await?),
            Some("n") => {
                n = Some(
                    parse_u32_field(field.text().await.map_err(bad_multipart)?).map_err(
                        |message| (axum::http::StatusCode::BAD_REQUEST, message).into_response(),
                    )?,
                )
            }
            Some("quality") => quality = Some(field.text().await.map_err(bad_multipart)?),
            Some("response_format") => {
                response_format = Some(field.text().await.map_err(bad_multipart)?)
            }
            Some("size") => size = Some(field.text().await.map_err(bad_multipart)?),
            Some("user") => user = Some(field.text().await.map_err(bad_multipart)?),
            _ => {}
        }
    }

    let mut request = CreateImageEditRequest::new(
        prompt.ok_or_else(missing_multipart_field)?,
        image.ok_or_else(missing_multipart_field)?,
    );
    if let Some(model) = model {
        request = request.with_model(model);
    }
    if let Some(mask) = mask {
        request = request.with_mask(mask);
    }
    request.n = n;
    request.quality = quality;
    request.response_format = response_format;
    request.size = size;
    request.user = user;

    Ok(request)
}

async fn parse_image_variation_request(
    mut multipart: Multipart,
) -> Result<CreateImageVariationRequest, Response> {
    let mut model = None;
    let mut image = None;
    let mut n = None;
    let mut response_format = None;
    let mut size = None;
    let mut user = None;

    while let Some(field) = multipart.next_field().await.map_err(bad_multipart)? {
        match field.name() {
            Some("model") => model = Some(field.text().await.map_err(bad_multipart)?),
            Some("image") => image = Some(parse_image_upload_field(field).await?),
            Some("n") => {
                n = Some(
                    parse_u32_field(field.text().await.map_err(bad_multipart)?).map_err(
                        |message| (axum::http::StatusCode::BAD_REQUEST, message).into_response(),
                    )?,
                )
            }
            Some("response_format") => {
                response_format = Some(field.text().await.map_err(bad_multipart)?)
            }
            Some("size") => size = Some(field.text().await.map_err(bad_multipart)?),
            Some("user") => user = Some(field.text().await.map_err(bad_multipart)?),
            _ => {}
        }
    }

    let mut request = CreateImageVariationRequest::new(image.ok_or_else(missing_multipart_field)?);
    if let Some(model) = model {
        request = request.with_model(model);
    }
    request.n = n;
    request.response_format = response_format;
    request.size = size;
    request.user = user;

    Ok(request)
}

async fn parse_image_upload_field(
    field: axum::extract::multipart::Field<'_>,
) -> Result<ImageUpload, Response> {
    let filename = field
        .file_name()
        .map(ToOwned::to_owned)
        .ok_or_else(missing_multipart_field)?;
    let content_type = field.content_type().map(ToOwned::to_owned);
    let bytes = field.bytes().await.map_err(bad_multipart)?.to_vec();
    let mut upload = ImageUpload::new(filename, bytes);
    if let Some(content_type) = content_type {
        upload = upload.with_content_type(content_type);
    }
    Ok(upload)
}

fn parse_u32_field(value: String) -> Result<u32, &'static str> {
    value
        .parse::<u32>()
        .map_err(|_| "invalid numeric multipart field")
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
