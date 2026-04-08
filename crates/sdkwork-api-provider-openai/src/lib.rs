use std::collections::HashMap;
use std::time::SystemTime;

use anyhow::Result;
use async_trait::async_trait;
use reqwest::{Client, RequestBuilder};
use sdkwork_api_contract_openai::assistants::{CreateAssistantRequest, UpdateAssistantRequest};
use sdkwork_api_contract_openai::audio::{
    CreateSpeechRequest, CreateTranscriptionRequest, CreateTranslationRequest,
    CreateVoiceConsentRequest,
};
use sdkwork_api_contract_openai::batches::CreateBatchRequest;
use sdkwork_api_contract_openai::chat_completions::{
    CreateChatCompletionRequest, UpdateChatCompletionRequest,
};
use sdkwork_api_contract_openai::completions::CreateCompletionRequest;
use sdkwork_api_contract_openai::containers::{CreateContainerFileRequest, CreateContainerRequest};
use sdkwork_api_contract_openai::conversations::{
    CreateConversationItemsRequest, CreateConversationRequest, UpdateConversationRequest,
};
use sdkwork_api_contract_openai::embeddings::CreateEmbeddingRequest;
use sdkwork_api_contract_openai::evals::{
    CreateEvalRequest, CreateEvalRunRequest, UpdateEvalRequest,
};
use sdkwork_api_contract_openai::files::CreateFileRequest;
use sdkwork_api_contract_openai::fine_tuning::{
    CreateFineTuningCheckpointPermissionsRequest, CreateFineTuningJobRequest,
};
use sdkwork_api_contract_openai::images::{
    CreateImageEditRequest, CreateImageRequest, CreateImageVariationRequest,
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
use sdkwork_api_domain_catalog::ModelCatalogEntry;
use sdkwork_api_provider_core::{
    ProviderAdapter, ProviderExecutionAdapter, ProviderHttpError, ProviderOutput, ProviderRequest,
    ProviderRequestOptions, ProviderRetryAfterSource, ProviderStreamOutput,
};
use serde_json::Value;

mod adapter_core;
mod dialog_resources;
mod media_resources;
mod control_resources;
mod openai_transport;
mod trait_impl;

pub(crate) use openai_transport::*;
pub use adapter_core::{map_model_object, OpenAiProviderAdapter};
