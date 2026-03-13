use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use sdkwork_api_contract_openai::chat_completions::CreateChatCompletionRequest;
use sdkwork_api_contract_openai::embeddings::CreateEmbeddingRequest;
use sdkwork_api_contract_openai::responses::CreateResponseRequest;
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
    Responses(&'a CreateResponseRequest),
    Embeddings(&'a CreateEmbeddingRequest),
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
