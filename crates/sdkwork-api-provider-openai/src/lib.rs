use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use sdkwork_api_contract_openai::chat_completions::CreateChatCompletionRequest;
use sdkwork_api_contract_openai::embeddings::CreateEmbeddingRequest;
use sdkwork_api_contract_openai::responses::CreateResponseRequest;
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

    pub async fn embeddings(
        &self,
        api_key: &str,
        request: &CreateEmbeddingRequest,
    ) -> Result<Value> {
        self.post_json("/v1/embeddings", api_key, request).await
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
            ProviderRequest::Responses(request) => Ok(ProviderOutput::Json(
                self.responses(api_key, request).await?,
            )),
            ProviderRequest::Embeddings(request) => Ok(ProviderOutput::Json(
                self.embeddings(api_key, request).await?,
            )),
        }
    }
}
