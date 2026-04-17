#![allow(dead_code)]

use super::*;
use crate::gateway_market::{
    GatewayApiErrorResponse, GatewayCommercialAccountResponse,
    GatewayCommercialBenefitLotsResponse, GatewayCouponRedemptionConfirmRequest,
    GatewayCouponRedemptionConfirmResponse, GatewayCouponRedemptionRollbackRequest,
    GatewayCouponRedemptionRollbackResponse, GatewayCouponReservationRequest,
    GatewayCouponReservationResponse, GatewayCouponValidationRequest,
    GatewayCouponValidationResponse, GatewayMarketOffersResponse, GatewayMarketProductsResponse,
};
use sdkwork_api_app_commerce::{PortalCommerceQuote, PortalCommerceQuoteRequest};
use sdkwork_api_contract_openai::assistants::{AssistantObject, ListAssistantsResponse};
use sdkwork_api_contract_openai::audio::{
    ListVoicesResponse, SpeechResponse, TranscriptionObject, TranslationObject, VoiceConsentObject,
};
use sdkwork_api_contract_openai::batches::{BatchObject, ListBatchesResponse};
use sdkwork_api_contract_openai::chat_completions::ChatCompletionResponse;
use sdkwork_api_contract_openai::completions::CompletionObject;
use sdkwork_api_contract_openai::conversations::{
    ConversationItemObject, ConversationObject, DeleteConversationItemResponse,
    DeleteConversationResponse, ListConversationItemsResponse, ListConversationsResponse,
};
use sdkwork_api_contract_openai::embeddings::CreateEmbeddingResponse;
use sdkwork_api_contract_openai::errors::OpenAiErrorResponse;
use sdkwork_api_contract_openai::files::{DeleteFileResponse, FileObject, ListFilesResponse};
use sdkwork_api_contract_openai::images::ImagesResponse;
use sdkwork_api_contract_openai::models::ListModelsResponse;
use sdkwork_api_contract_openai::moderations::ModerationResponse;
use sdkwork_api_contract_openai::realtime::RealtimeSessionObject;
use sdkwork_api_contract_openai::responses::{
    DeleteResponseResponse, ListResponseInputItemsResponse, ResponseCompactionObject,
    ResponseInputTokensObject, ResponseObject,
};
use sdkwork_api_contract_openai::runs::{
    ListRunStepsResponse, ListRunsResponse, RunObject, RunStepObject,
};
use sdkwork_api_contract_openai::threads::{
    DeleteThreadMessageResponse, DeleteThreadResponse, ListThreadMessagesResponse,
    ThreadMessageObject, ThreadObject,
};
use sdkwork_api_contract_openai::uploads::{UploadObject, UploadPartObject};
use sdkwork_api_contract_openai::vector_stores::{
    DeleteVectorStoreFileResponse, DeleteVectorStoreResponse, ListVectorStoreFilesResponse,
    ListVectorStoresResponse, SearchVectorStoreResponse, VectorStoreFileBatchObject,
    VectorStoreFileObject, VectorStoreObject,
};
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::openapi::Server;
use utoipa::{Modify, OpenApi};

#[allow(dead_code)]
#[path = "gateway_openapi_paths_agents.rs"]
mod paths_agents;
#[allow(dead_code)]
#[path = "gateway_openapi_paths_assistants_threads.rs"]
mod paths_assistants_threads;
#[allow(dead_code)]
#[path = "gateway_openapi_paths_code_claude.rs"]
mod paths_code_claude;
#[allow(dead_code)]
#[path = "gateway_openapi_paths_code_gemini.rs"]
mod paths_code_gemini;
#[allow(dead_code)]
#[path = "gateway_openapi_paths_code_openai.rs"]
mod paths_code_openai;
#[allow(dead_code)]
#[path = "gateway_openapi_paths_files_batches.rs"]
mod paths_files_batches;
#[allow(dead_code)]
#[path = "gateway_openapi_paths_images_dashscope.rs"]
mod paths_images_dashscope;
#[allow(dead_code)]
#[path = "gateway_openapi_paths_images_volcengine.rs"]
mod paths_images_volcengine;
#[allow(dead_code)]
#[path = "gateway_openapi_paths_jobs.rs"]
mod paths_jobs;
#[allow(dead_code)]
#[path = "gateway_openapi_paths_market_commercial.rs"]
mod paths_market_commercial;
#[allow(dead_code)]
#[path = "gateway_openapi_paths_media.rs"]
mod paths_media;
#[allow(dead_code)]
#[path = "gateway_openapi_paths_models_chat.rs"]
mod paths_models_chat;
#[allow(dead_code)]
#[path = "gateway_openapi_paths_music.rs"]
mod paths_music;
#[allow(dead_code)]
#[path = "gateway_openapi_paths_music_google.rs"]
mod paths_music_google;
#[allow(dead_code)]
#[path = "gateway_openapi_paths_music_minimax.rs"]
mod paths_music_minimax;
#[allow(dead_code)]
#[path = "gateway_openapi_paths_music_suno.rs"]
mod paths_music_suno;
#[allow(dead_code)]
#[path = "gateway_openapi_paths_storage.rs"]
mod paths_storage;
#[allow(dead_code)]
#[path = "gateway_openapi_paths_vector_compat.rs"]
mod paths_vector_compat;
#[allow(dead_code)]
#[path = "gateway_openapi_paths_video.rs"]
mod paths_video;
#[allow(dead_code)]
#[path = "gateway_openapi_paths_video_dashscope.rs"]
mod paths_video_dashscope;
#[allow(dead_code)]
#[path = "gateway_openapi_paths_video_google_veo.rs"]
mod paths_video_google_veo;
#[allow(dead_code)]
#[path = "gateway_openapi_paths_video_minimax.rs"]
mod paths_video_minimax;
#[allow(dead_code)]
#[path = "gateway_openapi_paths_video_vidu.rs"]
mod paths_video_vidu;
#[allow(dead_code)]
#[path = "gateway_openapi_paths_video_volcengine.rs"]
mod paths_video_volcengine;

mod openapi_paths {
    pub(crate) use super::paths_agents::*;
    pub(crate) use super::paths_assistants_threads::*;
    pub(crate) use super::paths_code_claude::*;
    pub(crate) use super::paths_code_gemini::*;
    pub(crate) use super::paths_code_openai::*;
    pub(crate) use super::paths_files_batches::*;
    pub(crate) use super::paths_images_dashscope::*;
    pub(crate) use super::paths_images_volcengine::*;
    pub(crate) use super::paths_jobs::*;
    pub(crate) use super::paths_market_commercial::*;
    pub(crate) use super::paths_media::*;
    pub(crate) use super::paths_models_chat::*;
    pub(crate) use super::paths_music::*;
    pub(crate) use super::paths_music_google::*;
    pub(crate) use super::paths_music_minimax::*;
    pub(crate) use super::paths_music_suno::*;
    pub(crate) use super::paths_storage::*;
    pub(crate) use super::paths_vector_compat::*;
    pub(crate) use super::paths_video::*;
    pub(crate) use super::paths_video_dashscope::*;
    pub(crate) use super::paths_video_google_veo::*;
    pub(crate) use super::paths_video_minimax::*;
    pub(crate) use super::paths_video_vidu::*;
    pub(crate) use super::paths_video_volcengine::*;
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "SDKWORK Gateway API",
        version = env!("CARGO_PKG_VERSION"),
        description = "OpenAPI 3.1 schema generated directly from the current gateway router implementation."
    ),
    modifiers(&GatewayApiDocModifier),
    paths(
        openapi_paths::health,
        openapi_paths::market_products,
        openapi_paths::market_offers,
        openapi_paths::market_quotes,
        openapi_paths::marketing_coupon_validate,
        openapi_paths::marketing_coupon_reserve,
        openapi_paths::marketing_coupon_confirm,
        openapi_paths::marketing_coupon_rollback,
        openapi_paths::commercial_account,
        openapi_paths::commercial_account_benefit_lots,
        openapi_paths::list_models,
        openapi_paths::get_model,
        openapi_paths::model_delete,
        openapi_paths::chat_completions_list,
        openapi_paths::chat_completions,
        openapi_paths::chat_completion_get,
        openapi_paths::chat_completion_update,
        openapi_paths::chat_completion_delete,
        openapi_paths::chat_completion_messages_list,
        openapi_paths::completions,
        openapi_paths::responses,
        openapi_paths::responses_input_tokens,
        openapi_paths::responses_compact,
        openapi_paths::response_get,
        openapi_paths::response_delete,
        openapi_paths::response_input_items,
        openapi_paths::response_cancel,
        openapi_paths::embeddings,
        openapi_paths::moderations,
        openapi_paths::image_generations,
        openapi_paths::image_edits,
        openapi_paths::image_variations,
        openapi_paths::images_dashscope_generation_create,
        openapi_paths::images_volcengine_generate_create,
        openapi_paths::images_dashscope_task_get,
        openapi_paths::video_dashscope_synthesis_create,
        openapi_paths::video_google_veo_models_action_create,
        openapi_paths::video_volcengine_tasks_create,
        openapi_paths::video_volcengine_task_get,
        openapi_paths::transcriptions,
        openapi_paths::translations,
        openapi_paths::audio_speech,
        openapi_paths::audio_voices,
        openapi_paths::audio_voice_consents,
        openapi_paths::video_minimax_generation_create,
        openapi_paths::video_minimax_generation_query,
        openapi_paths::video_minimax_file_retrieve,
        openapi_paths::video_vidu_text2video_create,
        openapi_paths::video_vidu_img2video_create,
        openapi_paths::video_vidu_reference2video_create,
        openapi_paths::video_vidu_task_creations_get,
        openapi_paths::video_vidu_task_cancel_create,
        openapi_paths::containers_list,
        openapi_paths::containers_create,
        openapi_paths::container_get,
        openapi_paths::container_delete,
        openapi_paths::container_files_list,
        openapi_paths::container_files_create,
        openapi_paths::container_file_get,
        openapi_paths::container_file_delete,
        openapi_paths::container_file_content,
        openapi_paths::assistants_list,
        openapi_paths::assistants_create,
        openapi_paths::assistants_get,
        openapi_paths::assistants_update,
        openapi_paths::assistants_delete,
        openapi_paths::conversations_list,
        openapi_paths::conversations_create,
        openapi_paths::conversation_get,
        openapi_paths::conversation_update,
        openapi_paths::conversation_delete,
        openapi_paths::conversation_items_list,
        openapi_paths::conversation_items_create,
        openapi_paths::conversation_item_get,
        openapi_paths::conversation_item_delete,
        openapi_paths::threads_create,
        openapi_paths::thread_get,
        openapi_paths::thread_update,
        openapi_paths::thread_delete,
        openapi_paths::thread_messages_list,
        openapi_paths::thread_messages_create,
        openapi_paths::thread_message_get,
        openapi_paths::thread_message_update,
        openapi_paths::thread_message_delete,
        openapi_paths::thread_and_run_create,
        openapi_paths::thread_runs_list,
        openapi_paths::thread_runs_create,
        openapi_paths::thread_run_get,
        openapi_paths::thread_run_update,
        openapi_paths::thread_run_cancel,
        openapi_paths::thread_run_submit_tool_outputs,
        openapi_paths::thread_run_steps_list,
        openapi_paths::thread_run_step_get,
        openapi_paths::realtime_sessions,
        openapi_paths::files_list,
        openapi_paths::files_create,
        openapi_paths::file_get,
        openapi_paths::file_delete,
        openapi_paths::file_content,
        openapi_paths::videos_list,
        openapi_paths::videos_create,
        openapi_paths::video_get,
        openapi_paths::video_delete,
        openapi_paths::video_content,
        openapi_paths::video_remix,
        openapi_paths::video_characters_create,
        openapi_paths::video_character_canonical_get,
        openapi_paths::video_edits,
        openapi_paths::video_extensions,
        openapi_paths::video_characters_list,
        openapi_paths::video_character_get,
        openapi_paths::video_character_update,
        openapi_paths::video_extend,
        openapi_paths::music_list,
        openapi_paths::music_create,
        openapi_paths::music_get,
        openapi_paths::music_delete,
        openapi_paths::music_content,
        openapi_paths::music_lyrics,
        openapi_paths::music_google_predict_create,
        openapi_paths::music_minimax_generation_create,
        openapi_paths::music_minimax_lyrics_create,
        openapi_paths::music_suno_generate_create,
        openapi_paths::music_suno_generate_record_info_get,
        openapi_paths::music_suno_lyrics_create,
        openapi_paths::music_suno_lyrics_record_info_get,
        openapi_paths::uploads_create,
        openapi_paths::upload_parts_create,
        openapi_paths::upload_complete,
        openapi_paths::upload_cancel,
        openapi_paths::batches_list,
        openapi_paths::batches_create,
        openapi_paths::batch_get,
        openapi_paths::batch_cancel,
        openapi_paths::fine_tuning_jobs_list,
        openapi_paths::fine_tuning_jobs_create,
        openapi_paths::fine_tuning_job_get,
        openapi_paths::fine_tuning_job_cancel,
        openapi_paths::fine_tuning_job_events,
        openapi_paths::fine_tuning_job_checkpoints,
        openapi_paths::fine_tuning_job_pause,
        openapi_paths::fine_tuning_job_resume,
        openapi_paths::fine_tuning_checkpoint_permissions_list,
        openapi_paths::fine_tuning_checkpoint_permissions_create,
        openapi_paths::fine_tuning_checkpoint_permission_delete,
        openapi_paths::webhooks_list,
        openapi_paths::webhooks_create,
        openapi_paths::webhook_get,
        openapi_paths::webhook_update,
        openapi_paths::webhook_delete,
        openapi_paths::evals_list,
        openapi_paths::evals_create,
        openapi_paths::eval_get,
        openapi_paths::eval_update,
        openapi_paths::eval_delete,
        openapi_paths::eval_runs_list,
        openapi_paths::eval_runs_create,
        openapi_paths::eval_run_get,
        openapi_paths::eval_run_delete,
        openapi_paths::eval_run_cancel,
        openapi_paths::eval_run_output_items_list,
        openapi_paths::eval_run_output_item_get,
        openapi_paths::vector_stores_list,
        openapi_paths::vector_stores_create,
        openapi_paths::vector_store_get,
        openapi_paths::vector_store_update,
        openapi_paths::vector_store_delete,
        openapi_paths::vector_store_search,
        openapi_paths::vector_store_files_list,
        openapi_paths::vector_store_files_create,
        openapi_paths::vector_store_file_get,
        openapi_paths::vector_store_file_delete,
        openapi_paths::vector_store_file_batches_create,
        openapi_paths::vector_store_file_batch_get,
        openapi_paths::vector_store_file_batch_cancel,
        openapi_paths::vector_store_file_batch_files_list,
        openapi_paths::anthropic_messages,
        openapi_paths::anthropic_count_tokens,
        openapi_paths::gemini_models_compat
    ),
    components(
        schemas(
            sdkwork_api_contract_openai::containers::CreateContainerRequest,
            sdkwork_api_contract_openai::containers::ContainerObject,
            sdkwork_api_contract_openai::containers::ListContainersResponse,
            sdkwork_api_contract_openai::containers::DeleteContainerResponse,
            sdkwork_api_contract_openai::containers::CreateContainerFileRequest,
            sdkwork_api_contract_openai::containers::ContainerFileObject,
            sdkwork_api_contract_openai::containers::ListContainerFilesResponse,
            sdkwork_api_contract_openai::containers::DeleteContainerFileResponse,
            sdkwork_api_contract_openai::videos::CreateVideoRequest,
            sdkwork_api_contract_openai::videos::RemixVideoRequest,
            sdkwork_api_contract_openai::videos::ExtendVideoRequest,
            sdkwork_api_contract_openai::videos::CreateVideoCharacterRequest,
            sdkwork_api_contract_openai::videos::EditVideoRequest,
            sdkwork_api_contract_openai::videos::UpdateVideoCharacterRequest,
            sdkwork_api_contract_openai::videos::VideoObject,
            sdkwork_api_contract_openai::videos::VideosResponse,
            sdkwork_api_contract_openai::videos::VideoCharacterObject,
            sdkwork_api_contract_openai::videos::VideoCharactersResponse,
            sdkwork_api_contract_openai::videos::DeleteVideoResponse,
            sdkwork_api_contract_openai::music::CreateMusicRequest,
            sdkwork_api_contract_openai::music::CreateMusicLyricsRequest,
            sdkwork_api_contract_openai::music::MusicObject,
            sdkwork_api_contract_openai::music::MusicTracksResponse,
            sdkwork_api_contract_openai::music::MusicLyricsObject,
            sdkwork_api_contract_openai::music::DeleteMusicResponse,
            sdkwork_api_contract_openai::webhooks::CreateWebhookRequest,
            sdkwork_api_contract_openai::webhooks::UpdateWebhookRequest,
            sdkwork_api_contract_openai::webhooks::WebhookObject,
            sdkwork_api_contract_openai::webhooks::ListWebhooksResponse,
            sdkwork_api_contract_openai::webhooks::DeleteWebhookResponse,
            sdkwork_api_contract_openai::evals::CreateEvalRequest,
            sdkwork_api_contract_openai::evals::EvalDataSourceConfig,
            sdkwork_api_contract_openai::evals::EvalObject,
            sdkwork_api_contract_openai::evals::ListEvalsResponse,
            sdkwork_api_contract_openai::evals::UpdateEvalRequest,
            sdkwork_api_contract_openai::evals::DeleteEvalResponse,
            sdkwork_api_contract_openai::evals::CreateEvalRunRequest,
            sdkwork_api_contract_openai::evals::EvalRunObject,
            sdkwork_api_contract_openai::evals::ListEvalRunsResponse,
            sdkwork_api_contract_openai::evals::DeleteEvalRunResponse,
            sdkwork_api_contract_openai::evals::EvalRunOutputItemObject,
            sdkwork_api_contract_openai::evals::ListEvalRunOutputItemsResponse,
            sdkwork_api_contract_openai::fine_tuning::CreateFineTuningJobRequest,
            sdkwork_api_contract_openai::fine_tuning::FineTuningJobObject,
            sdkwork_api_contract_openai::fine_tuning::ListFineTuningJobsResponse,
            sdkwork_api_contract_openai::fine_tuning::FineTuningJobEventObject,
            sdkwork_api_contract_openai::fine_tuning::ListFineTuningJobEventsResponse,
            sdkwork_api_contract_openai::fine_tuning::FineTuningJobCheckpointObject,
            sdkwork_api_contract_openai::fine_tuning::ListFineTuningJobCheckpointsResponse,
            sdkwork_api_contract_openai::fine_tuning::CreateFineTuningCheckpointPermissionsRequest,
            sdkwork_api_contract_openai::fine_tuning::FineTuningCheckpointPermissionObject,
            sdkwork_api_contract_openai::fine_tuning::ListFineTuningCheckpointPermissionsResponse,
            sdkwork_api_contract_openai::fine_tuning::DeleteFineTuningCheckpointPermissionResponse
        )
    ),
    tags(
        (name = "system.sdkwork", description = "Gateway health and system-facing routes."),
        (name = "code.openai", description = "Official OpenAI and Codex mirror routes."),
        (name = "code.claude", description = "Official Claude mirror routes."),
        (name = "code.gemini", description = "Official Gemini mirror routes, including image-capable Gemini generateContent models such as Nano Banana."),
        (name = "conversations", description = "OpenAI-compatible conversation and conversation item routes."),
        (name = "containers", description = "Container lifecycle and container file routes."),
        (name = "images.openai", description = "Official OpenAI image mirror routes."),
        (name = "images.kling", description = "Official shared DashScope image transport published for Kling-compatible clients."),
        (name = "images.aliyun", description = "Official shared DashScope image transport published for Aliyun-compatible clients."),
        (name = "images.volcengine", description = "Official Volcengine image mirror routes."),
        (name = "audio.openai", description = "Official shared audio mirror routes."),
        (name = "video.openai", description = "Official shared video mirror routes, including Sora 2 and Sora 2 Pro."),
        (name = "video.kling", description = "Official shared DashScope video transport published for Kling-compatible clients."),
        (name = "video.aliyun", description = "Official shared DashScope video transport published for Aliyun-compatible clients."),
        (name = "video.google-veo", description = "Official Google Veo video mirror routes, including Veo 3-class models selected through the official Vertex AI model path."),
        (name = "video.minimax", description = "Official MiniMax video mirror routes."),
        (name = "video.vidu", description = "Official Vidu video mirror routes."),
        (name = "video.volcengine", description = "Official Volcengine video mirror routes."),
        (name = "music.openai", description = "Official shared music mirror routes."),
        (name = "music.google", description = "Official Google music mirror routes."),
        (name = "music.minimax", description = "Official MiniMax music mirror routes."),
        (name = "music.suno", description = "Official Suno music mirror routes."),
        (name = "files", description = "File upload, listing, and retrieval routes."),
        (name = "uploads", description = "Multi-part upload lifecycle routes."),
        (name = "batches", description = "Batch execution submission and management routes."),
        (name = "fine-tuning", description = "Fine-tuning job, checkpoint, and permission routes."),
        (name = "webhooks", description = "Webhook management routes."),
        (name = "evals", description = "Evaluation, evaluation run, and output item routes."),
        (name = "vector-stores", description = "Vector store search and file management routes."),
        (name = "assistants", description = "Assistant creation and retrieval routes."),
        (name = "threads", description = "Assistant thread and message management routes."),
        (name = "runs", description = "Assistant run orchestration and run step routes."),
        (name = "realtime", description = "Realtime session bootstrap routes."),
        (name = "market", description = "Public API product catalog and quote routes."),
        (name = "marketing", description = "Coupon-first marketing validation and redemption routes."),
        (name = "commercial", description = "Commercial account and benefit-lot visibility routes.")
    )
)]
struct GatewayApiDoc;

struct GatewayApiDocModifier;

impl Modify for GatewayApiDocModifier {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        openapi.servers = Some(vec![Server::new("/")]);
        openapi
            .components
            .get_or_insert_with(utoipa::openapi::Components::new)
            .add_security_scheme(
                "bearerAuth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("API Key")
                        .build(),
                ),
            );
    }
}

pub(crate) fn gateway_openapi_document() -> utoipa::openapi::OpenApi {
    GatewayApiDoc::openapi()
}
