use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
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
use serde_json::Value;

pub fn map_model_object(model: &str) -> ModelCatalogEntry {
    ModelCatalogEntry::new(model, "provider-openai-official")
}

#[derive(Debug, Clone)]
pub struct OpenAiProviderAdapter {
    base_url: String,
    client: Client,
}

impl OpenAiProviderAdapter {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            client: Client::new(),
        }
    }

    pub async fn chat_completions(
        &self,
        api_key: &str,
        request: &CreateChatCompletionRequest,
    ) -> Result<Value> {
        self.post_json("/v1/chat/completions", api_key, request)
            .await
    }

    pub async fn chat_completions_stream(
        &self,
        api_key: &str,
        request: &CreateChatCompletionRequest,
    ) -> Result<reqwest::Response> {
        self.post_stream("/v1/chat/completions", api_key, request)
            .await
    }

    pub async fn responses(&self, api_key: &str, request: &CreateResponseRequest) -> Result<Value> {
        self.post_json("/v1/responses", api_key, request).await
    }

    pub async fn completions(
        &self,
        api_key: &str,
        request: &CreateCompletionRequest,
    ) -> Result<Value> {
        self.post_json("/v1/completions", api_key, request).await
    }

    pub async fn embeddings(
        &self,
        api_key: &str,
        request: &CreateEmbeddingRequest,
    ) -> Result<Value> {
        self.post_json("/v1/embeddings", api_key, request).await
    }

    pub async fn moderations(
        &self,
        api_key: &str,
        request: &CreateModerationRequest,
    ) -> Result<Value> {
        self.post_json("/v1/moderations", api_key, request).await
    }

    pub async fn images_generations(
        &self,
        api_key: &str,
        request: &CreateImageRequest,
    ) -> Result<Value> {
        self.post_json("/v1/images/generations", api_key, request)
            .await
    }

    pub async fn audio_transcriptions(
        &self,
        api_key: &str,
        request: &CreateTranscriptionRequest,
    ) -> Result<Value> {
        self.post_json("/v1/audio/transcriptions", api_key, request)
            .await
    }

    pub async fn audio_translations(
        &self,
        api_key: &str,
        request: &CreateTranslationRequest,
    ) -> Result<Value> {
        self.post_json("/v1/audio/translations", api_key, request)
            .await
    }

    pub async fn audio_speech(
        &self,
        api_key: &str,
        request: &CreateSpeechRequest,
    ) -> Result<reqwest::Response> {
        self.post_stream("/v1/audio/speech", api_key, request).await
    }

    pub async fn fine_tuning_jobs(
        &self,
        api_key: &str,
        request: &CreateFineTuningJobRequest,
    ) -> Result<Value> {
        self.post_json("/v1/fine_tuning/jobs", api_key, request)
            .await
    }

    pub async fn assistants(
        &self,
        api_key: &str,
        request: &CreateAssistantRequest,
    ) -> Result<Value> {
        self.post_json("/v1/assistants", api_key, request).await
    }

    pub async fn realtime_sessions(
        &self,
        api_key: &str,
        request: &CreateRealtimeSessionRequest,
    ) -> Result<Value> {
        self.post_json("/v1/realtime/sessions", api_key, request)
            .await
    }

    pub async fn evals(&self, api_key: &str, request: &CreateEvalRequest) -> Result<Value> {
        self.post_json("/v1/evals", api_key, request).await
    }

    pub async fn batches(&self, api_key: &str, request: &CreateBatchRequest) -> Result<Value> {
        self.post_json("/v1/batches", api_key, request).await
    }

    pub async fn vector_stores(
        &self,
        api_key: &str,
        request: &CreateVectorStoreRequest,
    ) -> Result<Value> {
        self.post_json("/v1/vector_stores", api_key, request).await
    }

    async fn post_json<T: serde::Serialize>(
        &self,
        path: &str,
        api_key: &str,
        request: &T,
    ) -> Result<Value> {
        let response = self
            .client
            .post(format!("{}{}", self.base_url, path))
            .bearer_auth(api_key)
            .json(request)
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json::<Value>().await?)
    }

    async fn post_stream<T: serde::Serialize>(
        &self,
        path: &str,
        api_key: &str,
        request: &T,
    ) -> Result<reqwest::Response> {
        let response = self
            .client
            .post(format!("{}{}", self.base_url, path))
            .bearer_auth(api_key)
            .json(request)
            .send()
            .await?
            .error_for_status()?;

        Ok(response)
    }
}

impl ProviderAdapter for OpenAiProviderAdapter {
    fn id(&self) -> &'static str {
        "openai"
    }
}

#[async_trait]
impl ProviderExecutionAdapter for OpenAiProviderAdapter {
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
