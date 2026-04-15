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
#[path = "gateway_openapi_paths_assistants_threads.rs"]
mod paths_assistants_threads;
#[allow(dead_code)]
#[path = "gateway_openapi_paths_files_batches.rs"]
mod paths_files_batches;
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
#[path = "gateway_openapi_paths_vector_compat.rs"]
mod paths_vector_compat;

mod openapi_paths {
    pub(crate) use super::paths_assistants_threads::*;
    pub(crate) use super::paths_files_batches::*;
    pub(crate) use super::paths_market_commercial::*;
    pub(crate) use super::paths_media::*;
    pub(crate) use super::paths_models_chat::*;
    pub(crate) use super::paths_vector_compat::*;
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
        openapi_paths::chat_completions,
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
        openapi_paths::transcriptions,
        openapi_paths::translations,
        openapi_paths::audio_speech,
        openapi_paths::audio_voices,
        openapi_paths::audio_voice_consents,
        openapi_paths::assistants_list,
        openapi_paths::assistants_create,
        openapi_paths::assistants_get,
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
        openapi_paths::uploads_create,
        openapi_paths::upload_parts_create,
        openapi_paths::upload_complete,
        openapi_paths::upload_cancel,
        openapi_paths::batches_list,
        openapi_paths::batches_create,
        openapi_paths::batch_get,
        openapi_paths::batch_cancel,
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
    tags(
        (name = "system", description = "Gateway health and system-facing routes."),
        (name = "models", description = "Model listing and model metadata routes."),
        (name = "chat", description = "OpenAI-compatible chat completion routes."),
        (name = "completions", description = "OpenAI-compatible text completion routes."),
        (name = "responses", description = "OpenAI-compatible response generation routes."),
        (name = "conversations", description = "OpenAI-compatible conversation and conversation item routes."),
        (name = "embeddings", description = "Embedding generation routes."),
        (name = "moderations", description = "Moderation and safety evaluation routes."),
        (name = "images", description = "Image generation, edit, and variation routes."),
        (name = "audio", description = "Audio transcription, translation, speech, and voice routes."),
        (name = "files", description = "File upload, listing, and retrieval routes."),
        (name = "uploads", description = "Multi-part upload lifecycle routes."),
        (name = "batches", description = "Batch execution submission and management routes."),
        (name = "vector-stores", description = "Vector store search and file management routes."),
        (name = "assistants", description = "Assistant creation and retrieval routes."),
        (name = "threads", description = "Assistant thread and message management routes."),
        (name = "runs", description = "Assistant run orchestration and run step routes."),
        (name = "realtime", description = "Realtime session bootstrap routes."),
        (name = "market", description = "Public API product catalog and quote routes."),
        (name = "marketing", description = "Coupon-first marketing validation and redemption routes."),
        (name = "commercial", description = "Commercial account and benefit-lot visibility routes."),
        (name = "compatibility", description = "Anthropic and Gemini compatibility routes.")
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
