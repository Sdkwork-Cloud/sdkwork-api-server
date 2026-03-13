use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use sdkwork_api_contract_openai::assistants::{CreateAssistantRequest, UpdateAssistantRequest};
use sdkwork_api_contract_openai::audio::{
    CreateSpeechRequest, CreateTranscriptionRequest, CreateTranslationRequest,
};
use sdkwork_api_contract_openai::batches::CreateBatchRequest;
use sdkwork_api_contract_openai::chat_completions::{
    CreateChatCompletionRequest, UpdateChatCompletionRequest,
};
use sdkwork_api_contract_openai::completions::CreateCompletionRequest;
use sdkwork_api_contract_openai::embeddings::CreateEmbeddingRequest;
use sdkwork_api_contract_openai::evals::CreateEvalRequest;
use sdkwork_api_contract_openai::files::CreateFileRequest;
use sdkwork_api_contract_openai::fine_tuning::CreateFineTuningJobRequest;
use sdkwork_api_contract_openai::images::CreateImageRequest;
use sdkwork_api_contract_openai::moderations::CreateModerationRequest;
use sdkwork_api_contract_openai::realtime::CreateRealtimeSessionRequest;
use sdkwork_api_contract_openai::responses::{
    CompactResponseRequest, CountResponseInputTokensRequest, CreateResponseRequest,
};
use sdkwork_api_contract_openai::uploads::{
    AddUploadPartRequest, CompleteUploadRequest, CreateUploadRequest,
};
use sdkwork_api_contract_openai::vector_stores::{
    CreateVectorStoreFileBatchRequest, CreateVectorStoreFileRequest, CreateVectorStoreRequest,
    SearchVectorStoreRequest, UpdateVectorStoreRequest,
};
use sdkwork_api_contract_openai::videos::{CreateVideoRequest, RemixVideoRequest};
use sdkwork_api_contract_openai::webhooks::{CreateWebhookRequest, UpdateWebhookRequest};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilitySupport {
    Supported,
    Unsupported,
}

pub trait ProviderAdapter {
    fn id(&self) -> &'static str;
}

pub enum ProviderRequest<'a> {
    ChatCompletions(&'a CreateChatCompletionRequest),
    ChatCompletionsStream(&'a CreateChatCompletionRequest),
    ChatCompletionsList,
    ChatCompletionsRetrieve(&'a str),
    ChatCompletionsUpdate(&'a str, &'a UpdateChatCompletionRequest),
    ChatCompletionsDelete(&'a str),
    ChatCompletionsMessagesList(&'a str),
    Completions(&'a CreateCompletionRequest),
    Responses(&'a CreateResponseRequest),
    ResponsesInputTokens(&'a CountResponseInputTokensRequest),
    ResponsesRetrieve(&'a str),
    ResponsesDelete(&'a str),
    ResponsesInputItemsList(&'a str),
    ResponsesCancel(&'a str),
    ResponsesCompact(&'a CompactResponseRequest),
    Embeddings(&'a CreateEmbeddingRequest),
    Moderations(&'a CreateModerationRequest),
    ImagesGenerations(&'a CreateImageRequest),
    AudioTranscriptions(&'a CreateTranscriptionRequest),
    AudioTranslations(&'a CreateTranslationRequest),
    AudioSpeech(&'a CreateSpeechRequest),
    Files(&'a CreateFileRequest),
    FilesList,
    FilesRetrieve(&'a str),
    FilesDelete(&'a str),
    FilesContent(&'a str),
    Uploads(&'a CreateUploadRequest),
    UploadParts(&'a AddUploadPartRequest),
    UploadComplete(&'a CompleteUploadRequest),
    UploadCancel(&'a str),
    FineTuningJobs(&'a CreateFineTuningJobRequest),
    FineTuningJobsList,
    FineTuningJobsRetrieve(&'a str),
    FineTuningJobsCancel(&'a str),
    Assistants(&'a CreateAssistantRequest),
    AssistantsList,
    AssistantsRetrieve(&'a str),
    AssistantsUpdate(&'a str, &'a UpdateAssistantRequest),
    AssistantsDelete(&'a str),
    RealtimeSessions(&'a CreateRealtimeSessionRequest),
    Evals(&'a CreateEvalRequest),
    Batches(&'a CreateBatchRequest),
    BatchesList,
    BatchesRetrieve(&'a str),
    BatchesCancel(&'a str),
    VectorStores(&'a CreateVectorStoreRequest),
    VectorStoresList,
    VectorStoresRetrieve(&'a str),
    VectorStoresUpdate(&'a str, &'a UpdateVectorStoreRequest),
    VectorStoresDelete(&'a str),
    VectorStoresSearch(&'a str, &'a SearchVectorStoreRequest),
    VectorStoreFiles(&'a str, &'a CreateVectorStoreFileRequest),
    VectorStoreFilesList(&'a str),
    VectorStoreFilesRetrieve(&'a str, &'a str),
    VectorStoreFilesDelete(&'a str, &'a str),
    VectorStoreFileBatches(&'a str, &'a CreateVectorStoreFileBatchRequest),
    VectorStoreFileBatchesRetrieve(&'a str, &'a str),
    VectorStoreFileBatchesCancel(&'a str, &'a str),
    VectorStoreFileBatchesListFiles(&'a str, &'a str),
    Videos(&'a CreateVideoRequest),
    VideosList,
    VideosRetrieve(&'a str),
    VideosDelete(&'a str),
    VideosContent(&'a str),
    VideosRemix(&'a str, &'a RemixVideoRequest),
    Webhooks(&'a CreateWebhookRequest),
    WebhooksList,
    WebhooksRetrieve(&'a str),
    WebhooksUpdate(&'a str, &'a UpdateWebhookRequest),
    WebhooksDelete(&'a str),
}

pub enum ProviderOutput {
    Json(Value),
    Stream(reqwest::Response),
}

impl ProviderOutput {
    pub fn into_json(self) -> Option<Value> {
        match self {
            Self::Json(value) => Some(value),
            Self::Stream(_) => None,
        }
    }

    pub fn into_stream(self) -> Option<reqwest::Response> {
        match self {
            Self::Json(_) => None,
            Self::Stream(response) => Some(response),
        }
    }
}

#[async_trait]
pub trait ProviderExecutionAdapter: ProviderAdapter + Send + Sync {
    async fn execute(&self, api_key: &str, request: ProviderRequest<'_>) -> Result<ProviderOutput>;
}

type AdapterFactory =
    Arc<dyn Fn(String) -> Box<dyn ProviderExecutionAdapter> + Send + Sync + 'static>;

#[derive(Default, Clone)]
pub struct ProviderRegistry {
    factories: HashMap<String, AdapterFactory>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_factory<F>(&mut self, adapter_kind: impl Into<String>, factory: F)
    where
        F: Fn(String) -> Box<dyn ProviderExecutionAdapter> + Send + Sync + 'static,
    {
        self.factories
            .insert(adapter_kind.into(), Arc::new(factory));
    }

    pub fn resolve(
        &self,
        adapter_kind: &str,
        base_url: impl Into<String>,
    ) -> Option<Box<dyn ProviderExecutionAdapter>> {
        self.factories
            .get(adapter_kind)
            .map(|factory| factory(base_url.into()))
    }
}
