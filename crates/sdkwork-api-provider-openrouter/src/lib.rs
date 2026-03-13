use anyhow::Result;
use async_trait::async_trait;
use sdkwork_api_contract_openai::audio::{CreateTranscriptionRequest, CreateTranslationRequest};
use sdkwork_api_contract_openai::chat_completions::CreateChatCompletionRequest;
use sdkwork_api_contract_openai::completions::CreateCompletionRequest;
use sdkwork_api_contract_openai::embeddings::CreateEmbeddingRequest;
use sdkwork_api_contract_openai::fine_tuning::CreateFineTuningJobRequest;
use sdkwork_api_contract_openai::images::CreateImageRequest;
use sdkwork_api_contract_openai::moderations::CreateModerationRequest;
use sdkwork_api_contract_openai::responses::CreateResponseRequest;
use sdkwork_api_domain_catalog::ModelCatalogEntry;
use sdkwork_api_provider_core::{
    ProviderAdapter, ProviderExecutionAdapter, ProviderOutput, ProviderRequest,
};
use sdkwork_api_provider_openai::OpenAiProviderAdapter;
use serde_json::Value;

pub fn adapter_id() -> &'static str {
    "openrouter"
}

pub fn map_model_object(model: &str) -> ModelCatalogEntry {
    ModelCatalogEntry::new(model, "provider-openrouter-main")
}

#[derive(Debug, Clone)]
pub struct OpenRouterProviderAdapter {
    delegate: OpenAiProviderAdapter,
}

impl OpenRouterProviderAdapter {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            delegate: OpenAiProviderAdapter::new(base_url),
        }
    }

    pub async fn chat_completions(
        &self,
        api_key: &str,
        request: &CreateChatCompletionRequest,
    ) -> Result<Value> {
        self.delegate.chat_completions(api_key, request).await
    }

    pub async fn chat_completions_stream(
        &self,
        api_key: &str,
        request: &CreateChatCompletionRequest,
    ) -> Result<reqwest::Response> {
        self.delegate
            .chat_completions_stream(api_key, request)
            .await
    }

    pub async fn responses(&self, api_key: &str, request: &CreateResponseRequest) -> Result<Value> {
        self.delegate.responses(api_key, request).await
    }

    pub async fn completions(
        &self,
        api_key: &str,
        request: &CreateCompletionRequest,
    ) -> Result<Value> {
        self.delegate.completions(api_key, request).await
    }

    pub async fn embeddings(
        &self,
        api_key: &str,
        request: &CreateEmbeddingRequest,
    ) -> Result<Value> {
        self.delegate.embeddings(api_key, request).await
    }

    pub async fn moderations(
        &self,
        api_key: &str,
        request: &CreateModerationRequest,
    ) -> Result<Value> {
        self.delegate.moderations(api_key, request).await
    }

    pub async fn images_generations(
        &self,
        api_key: &str,
        request: &CreateImageRequest,
    ) -> Result<Value> {
        self.delegate.images_generations(api_key, request).await
    }

    pub async fn audio_transcriptions(
        &self,
        api_key: &str,
        request: &CreateTranscriptionRequest,
    ) -> Result<Value> {
        self.delegate.audio_transcriptions(api_key, request).await
    }

    pub async fn audio_translations(
        &self,
        api_key: &str,
        request: &CreateTranslationRequest,
    ) -> Result<Value> {
        self.delegate.audio_translations(api_key, request).await
    }

    pub async fn fine_tuning_jobs(
        &self,
        api_key: &str,
        request: &CreateFineTuningJobRequest,
    ) -> Result<Value> {
        self.delegate.fine_tuning_jobs(api_key, request).await
    }
}

impl ProviderAdapter for OpenRouterProviderAdapter {
    fn id(&self) -> &'static str {
        "openrouter"
    }
}

#[async_trait]
impl ProviderExecutionAdapter for OpenRouterProviderAdapter {
    async fn execute(&self, api_key: &str, request: ProviderRequest<'_>) -> Result<ProviderOutput> {
        match request {
            ProviderRequest::ChatCompletions(request) => Ok(ProviderOutput::Json(
                self.chat_completions(api_key, request).await?,
            )),
            ProviderRequest::ChatCompletionsStream(request) => Ok(ProviderOutput::Stream(
                self.chat_completions_stream(api_key, request).await?,
            )),
            ProviderRequest::Completions(request) => Ok(ProviderOutput::Json(
                self.completions(api_key, request).await?,
            )),
            ProviderRequest::Responses(request) => Ok(ProviderOutput::Json(
                self.responses(api_key, request).await?,
            )),
            ProviderRequest::Embeddings(request) => Ok(ProviderOutput::Json(
                self.embeddings(api_key, request).await?,
            )),
            ProviderRequest::Moderations(request) => Ok(ProviderOutput::Json(
                self.moderations(api_key, request).await?,
            )),
            ProviderRequest::ImagesGenerations(request) => Ok(ProviderOutput::Json(
                self.images_generations(api_key, request).await?,
            )),
            ProviderRequest::AudioTranscriptions(request) => Ok(ProviderOutput::Json(
                self.audio_transcriptions(api_key, request).await?,
            )),
            ProviderRequest::AudioTranslations(request) => Ok(ProviderOutput::Json(
                self.audio_translations(api_key, request).await?,
            )),
            ProviderRequest::FineTuningJobs(request) => Ok(ProviderOutput::Json(
                self.fine_tuning_jobs(api_key, request).await?,
            )),
        }
    }
}
