use anyhow::Result;
use async_trait::async_trait;
use sdkwork_api_contract_openai::assistants::CreateAssistantRequest;
use sdkwork_api_contract_openai::audio::{
    CreateSpeechRequest, CreateTranscriptionRequest, CreateTranslationRequest,
};
use sdkwork_api_contract_openai::batches::CreateBatchRequest;
use sdkwork_api_contract_openai::chat_completions::CreateChatCompletionRequest;
use sdkwork_api_contract_openai::completions::CreateCompletionRequest;
use sdkwork_api_contract_openai::embeddings::CreateEmbeddingRequest;
use sdkwork_api_contract_openai::evals::CreateEvalRequest;
use sdkwork_api_contract_openai::fine_tuning::CreateFineTuningJobRequest;
use sdkwork_api_contract_openai::images::CreateImageRequest;
use sdkwork_api_contract_openai::moderations::CreateModerationRequest;
use sdkwork_api_contract_openai::realtime::CreateRealtimeSessionRequest;
use sdkwork_api_contract_openai::responses::CreateResponseRequest;
use sdkwork_api_contract_openai::vector_stores::CreateVectorStoreRequest;
use sdkwork_api_domain_catalog::ModelCatalogEntry;
use sdkwork_api_provider_core::{
    ProviderAdapter, ProviderExecutionAdapter, ProviderOutput, ProviderRequest,
};
use sdkwork_api_provider_openai::OpenAiProviderAdapter;
use serde_json::Value;

pub fn adapter_id() -> &'static str {
    "ollama"
}

pub fn map_model_object(model: &str) -> ModelCatalogEntry {
    ModelCatalogEntry::new(model, "provider-ollama-local")
}

#[derive(Debug, Clone)]
pub struct OllamaProviderAdapter {
    delegate: OpenAiProviderAdapter,
}

impl OllamaProviderAdapter {
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

    pub async fn audio_speech(
        &self,
        api_key: &str,
        request: &CreateSpeechRequest,
    ) -> Result<reqwest::Response> {
        self.delegate.audio_speech(api_key, request).await
    }

    pub async fn fine_tuning_jobs(
        &self,
        api_key: &str,
        request: &CreateFineTuningJobRequest,
    ) -> Result<Value> {
        self.delegate.fine_tuning_jobs(api_key, request).await
    }

    pub async fn assistants(
        &self,
        api_key: &str,
        request: &CreateAssistantRequest,
    ) -> Result<Value> {
        self.delegate.assistants(api_key, request).await
    }

    pub async fn realtime_sessions(
        &self,
        api_key: &str,
        request: &CreateRealtimeSessionRequest,
    ) -> Result<Value> {
        self.delegate.realtime_sessions(api_key, request).await
    }

    pub async fn evals(&self, api_key: &str, request: &CreateEvalRequest) -> Result<Value> {
        self.delegate.evals(api_key, request).await
    }

    pub async fn batches(&self, api_key: &str, request: &CreateBatchRequest) -> Result<Value> {
        self.delegate.batches(api_key, request).await
    }

    pub async fn vector_stores(
        &self,
        api_key: &str,
        request: &CreateVectorStoreRequest,
    ) -> Result<Value> {
        self.delegate.vector_stores(api_key, request).await
    }
}

impl ProviderAdapter for OllamaProviderAdapter {
    fn id(&self) -> &'static str {
        "ollama"
    }
}

#[async_trait]
impl ProviderExecutionAdapter for OllamaProviderAdapter {
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
            ProviderRequest::AudioSpeech(request) => Ok(ProviderOutput::Stream(
                self.audio_speech(api_key, request).await?,
            )),
            ProviderRequest::FineTuningJobs(request) => Ok(ProviderOutput::Json(
                self.fine_tuning_jobs(api_key, request).await?,
            )),
            ProviderRequest::Assistants(request) => Ok(ProviderOutput::Json(
                self.assistants(api_key, request).await?,
            )),
            ProviderRequest::RealtimeSessions(request) => Ok(ProviderOutput::Json(
                self.realtime_sessions(api_key, request).await?,
            )),
            ProviderRequest::Evals(request) => {
                Ok(ProviderOutput::Json(self.evals(api_key, request).await?))
            }
            ProviderRequest::Batches(request) => {
                Ok(ProviderOutput::Json(self.batches(api_key, request).await?))
            }
            ProviderRequest::VectorStores(request) => Ok(ProviderOutput::Json(
                self.vector_stores(api_key, request).await?,
            )),
        }
    }
}
