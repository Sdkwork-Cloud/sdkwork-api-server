mod compat_anthropic;
mod compat_gemini;
mod compat_streaming;

use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use axum::{
    Json, Router,
    body::Body,
    extract::FromRequestParts,
    extract::Json as ExtractJson,
    extract::Multipart,
    extract::Path,
    extract::State,
    http::HeaderMap,
    http::HeaderValue,
    http::Request,
    http::StatusCode,
    http::header,
    http::request::Parts,
    middleware::Next,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use compat_anthropic::{
    anthropic_bad_gateway_response, anthropic_count_tokens_request,
    anthropic_invalid_request_response, anthropic_request_to_chat_completion,
    anthropic_stream_from_openai, openai_chat_response_to_anthropic,
    openai_count_tokens_to_anthropic,
};
use compat_gemini::{
    GeminiCompatAction, gemini_bad_gateway_response, gemini_count_tokens_request,
    gemini_invalid_request_response, gemini_request_to_chat_completion,
    gemini_stream_from_openai, openai_chat_response_to_gemini,
    openai_count_tokens_to_gemini, parse_gemini_compat_tail,
};
use sdkwork_api_app_billing::{
    BillingAccountingMode, CaptureAccountHoldInput, CreateAccountHoldInput,
    CreateBillingEventInput, GatewayCommercialBillingKernel, QuotaCheckResult,
    ReleaseAccountHoldInput, check_quota, create_billing_event, persist_billing_event,
    persist_ledger_entry,
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
    PlannedExecutionUsageContext, create_embedding, create_response, delete_model_from_store,
    execute_json_provider_request_with_runtime_and_options,
    execute_stream_provider_request_with_runtime_and_options, list_models_from_store,
    planned_execution_usage_context_for_route, relay_assistant_from_store,
    relay_audio_voice_consent_from_store, relay_audio_voices_from_store, relay_batch_from_store,
    relay_cancel_batch_from_store, relay_cancel_fine_tuning_job_from_store,
    relay_cancel_response_from_store, relay_cancel_thread_run_from_store,
    relay_cancel_upload_from_store, relay_cancel_vector_store_file_batch_from_store,
    relay_chat_completion_from_store_with_execution_context,
    relay_chat_completion_stream_from_store_with_execution_context,
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
    relay_remix_video_from_store, relay_response_from_store_with_execution_context,
    relay_response_stream_from_store_with_execution_context, relay_search_vector_store_from_store,
    relay_speech_from_store, relay_submit_thread_run_tool_outputs_from_store,
    relay_thread_and_run_from_store, relay_thread_from_store, relay_thread_messages_from_store,
    relay_thread_run_from_store, relay_transcription_from_store, relay_translation_from_store,
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
    GatewayRequestContext as IdentityGatewayRequestContext, resolve_gateway_request_context,
};
use sdkwork_api_app_rate_limit::check_rate_limit;
use sdkwork_api_app_usage::persist_usage_record_with_tokens_and_facts;
use sdkwork_api_config::HttpExposureConfig;
use sdkwork_api_contract_openai::assistants::{
    AssistantObject, DeleteAssistantResponse, ListAssistantsResponse,
};
use sdkwork_api_contract_openai::assistants::{CreateAssistantRequest, UpdateAssistantRequest};
use sdkwork_api_contract_openai::audio::{
    CreateSpeechRequest, CreateTranscriptionRequest, CreateTranslationRequest,
    CreateVoiceConsentRequest, ListVoicesResponse, SpeechResponse, TranscriptionObject,
    TranslationObject, VoiceConsentObject,
};
use sdkwork_api_contract_openai::batches::{BatchObject, CreateBatchRequest, ListBatchesResponse};
use sdkwork_api_contract_openai::chat_completions::{
    ChatCompletionResponse, CreateChatCompletionRequest, DeleteChatCompletionResponse,
    ListChatCompletionMessagesResponse, UpdateChatCompletionRequest,
};
use sdkwork_api_contract_openai::completions::{CompletionObject, CreateCompletionRequest};
use sdkwork_api_contract_openai::containers::{
    ContainerFileObject, ContainerObject, CreateContainerFileRequest, CreateContainerRequest,
    DeleteContainerFileResponse, DeleteContainerResponse, ListContainerFilesResponse,
};
use sdkwork_api_contract_openai::conversations::{
    ConversationItemObject, ConversationObject, CreateConversationItemsRequest,
    CreateConversationRequest, DeleteConversationItemResponse, DeleteConversationResponse,
    ListConversationItemsResponse, ListConversationsResponse, UpdateConversationRequest,
};
use sdkwork_api_contract_openai::embeddings::{CreateEmbeddingRequest, CreateEmbeddingResponse};
use sdkwork_api_contract_openai::errors::OpenAiErrorResponse;
use sdkwork_api_contract_openai::evals::{
    CreateEvalRequest, CreateEvalRunRequest, DeleteEvalResponse, DeleteEvalRunResponse, EvalObject,
    EvalRunObject, EvalRunOutputItemObject, ListEvalRunOutputItemsResponse, ListEvalRunsResponse,
    UpdateEvalRequest,
};
use sdkwork_api_contract_openai::files::{
    CreateFileRequest, DeleteFileResponse, FileObject, ListFilesResponse,
};
use sdkwork_api_contract_openai::fine_tuning::{
    CreateFineTuningCheckpointPermissionsRequest, CreateFineTuningJobRequest,
    DeleteFineTuningCheckpointPermissionResponse, FineTuningJobObject,
    ListFineTuningCheckpointPermissionsResponse, ListFineTuningJobCheckpointsResponse,
    ListFineTuningJobEventsResponse,
};
use sdkwork_api_contract_openai::images::{
    CreateImageEditRequest, CreateImageRequest, CreateImageVariationRequest, ImageUpload,
    ImagesResponse,
};
use sdkwork_api_contract_openai::models::ListModelsResponse;
use sdkwork_api_contract_openai::moderations::{CreateModerationRequest, ModerationResponse};
use sdkwork_api_contract_openai::music::{
    CreateMusicLyricsRequest, CreateMusicRequest, DeleteMusicResponse, MusicObject,
};
use sdkwork_api_contract_openai::realtime::{CreateRealtimeSessionRequest, RealtimeSessionObject};
use sdkwork_api_contract_openai::responses::{
    CompactResponseRequest, CountResponseInputTokensRequest, CreateResponseRequest,
    DeleteResponseResponse, ListResponseInputItemsResponse, ResponseCompactionObject,
    ResponseInputTokensObject, ResponseObject,
};
use sdkwork_api_contract_openai::runs::{
    CreateRunRequest, CreateThreadAndRunRequest, ListRunStepsResponse, ListRunsResponse, RunObject,
    RunStepObject, SubmitToolOutputsRunRequest, UpdateRunRequest,
};
use sdkwork_api_contract_openai::streaming::SseFrame;
use sdkwork_api_contract_openai::threads::{
    CreateThreadMessageRequest, CreateThreadRequest, DeleteThreadMessageResponse,
    DeleteThreadResponse, ListThreadMessagesResponse, ThreadMessageObject, ThreadObject,
    UpdateThreadMessageRequest, UpdateThreadRequest,
};
use sdkwork_api_contract_openai::uploads::{
    AddUploadPartRequest, CompleteUploadRequest, CreateUploadRequest, UploadObject,
    UploadPartObject,
};
use sdkwork_api_contract_openai::vector_stores::{
    CreateVectorStoreFileBatchRequest, CreateVectorStoreFileRequest, CreateVectorStoreRequest,
    DeleteVectorStoreFileResponse, DeleteVectorStoreResponse, ListVectorStoreFilesResponse,
    ListVectorStoresResponse, SearchVectorStoreRequest, SearchVectorStoreResponse,
    UpdateVectorStoreRequest, VectorStoreFileBatchObject, VectorStoreFileObject, VectorStoreObject,
};
use sdkwork_api_contract_openai::videos::{
    CreateVideoCharacterRequest, CreateVideoRequest, DeleteVideoResponse, EditVideoRequest,
    ExtendVideoRequest, RemixVideoRequest, UpdateVideoCharacterRequest, VideoObject,
};
use sdkwork_api_contract_openai::webhooks::{
    CreateWebhookRequest, DeleteWebhookResponse, UpdateWebhookRequest, WebhookObject,
};
use sdkwork_api_domain_rate_limit::RateLimitCheckResult;
use sdkwork_api_observability::{HttpMetricsRegistry, observe_http_metrics, observe_http_tracing};
use sdkwork_api_policy_billing::{
    BillingPolicyExecutionInput, BillingPolicyExecutionResult, GROUP_DEFAULT_BILLING_POLICY_ID,
    builtin_billing_policy_registry,
};
use sdkwork_api_provider_core::{ProviderRequest, ProviderRequestOptions, ProviderStreamOutput};
use sdkwork_api_storage_core::{AdminStore, Reloadable};
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use serde_json::Value;
use sqlx::SqlitePool;
use tower_http::cors::{Any, CorsLayer};
use utoipa::openapi::Server;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::{Config as SwaggerUiConfig, SwaggerUi, Url as SwaggerUiUrl};


include!("gateway_openapi.rs");
include!("gateway_http.rs");
include!("gateway_state.rs");
include!("gateway_auth.rs");
include!("gateway_routes.rs");
include!("gateway_models.rs");
include!("gateway_chat.rs");
include!("gateway_conversations.rs");
include!("gateway_threads.rs");
include!("gateway_thread_runs.rs");
include!("gateway_thread_run_steps.rs");
include!("gateway_responses.rs");
include!("gateway_generation.rs");
include!("gateway_images.rs");
include!("gateway_audio.rs");
include!("gateway_files.rs");
include!("gateway_containers.rs");
include!("gateway_music.rs");
include!("gateway_videos.rs");
include!("gateway_video_management.rs");
include!("gateway_uploads.rs");
include!("gateway_fine_tuning_jobs.rs");
include!("gateway_fine_tuning_permissions.rs");
include!("gateway_assistants.rs");
include!("gateway_webhooks.rs");
include!("gateway_realtime.rs");
include!("gateway_evals.rs");
include!("gateway_eval_runs.rs");
include!("gateway_batches.rs");
include!("gateway_vector_stores.rs");
include!("gateway_vector_store_files.rs");
include!("gateway_compat_handlers.rs");
include!("gateway_commercial.rs");
include!("gateway_stateless_relay.rs");
include!("gateway_streaming_support.rs");
include!("gateway_multipart_support.rs");
