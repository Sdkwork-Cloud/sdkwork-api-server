use anyhow::Result;
use sdkwork_api_app_credential::resolve_provider_secret;
use sdkwork_api_app_routing::simulate_route_with_store;
use sdkwork_api_contract_openai::chat_completions::ChatCompletionResponse;
use sdkwork_api_contract_openai::chat_completions::CreateChatCompletionRequest;
use sdkwork_api_contract_openai::embeddings::CreateEmbeddingRequest;
use sdkwork_api_contract_openai::embeddings::CreateEmbeddingResponse;
use sdkwork_api_contract_openai::models::{ListModelsResponse, ModelObject};
use sdkwork_api_contract_openai::responses::CreateResponseRequest;
use sdkwork_api_contract_openai::responses::ResponseObject;
use sdkwork_api_provider_core::{ProviderRegistry, ProviderRequest};
use sdkwork_api_provider_ollama::OllamaProviderAdapter;
use sdkwork_api_provider_openai::OpenAiProviderAdapter;
use sdkwork_api_provider_openrouter::OpenRouterProviderAdapter;
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use serde_json::Value;

pub fn service_name() -> &'static str {
    "gateway-service"
}

pub fn list_models(_tenant_id: &str, _project_id: &str) -> Result<ListModelsResponse> {
    Ok(ListModelsResponse::new(vec![ModelObject::new(
        "gpt-4.1", "sdkwork",
    )]))
}

pub async fn list_models_from_store(
    store: &SqliteAdminStore,
    _tenant_id: &str,
    _project_id: &str,
) -> Result<ListModelsResponse> {
    let models = store.list_models().await?;
    Ok(ListModelsResponse::new(
        models
            .into_iter()
            .map(|entry| ModelObject::new(entry.external_name, entry.provider_id))
            .collect(),
    ))
}

pub async fn relay_chat_completion_from_store(
    store: &SqliteAdminStore,
    master_key: &str,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateChatCompletionRequest,
) -> Result<Option<Value>> {
    let decision = simulate_route_with_store(store, "chat_completion", &request.model).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) = resolve_provider_secret(store, master_key, tenant_id, &provider.id).await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &provider.adapter_kind,
        provider.base_url,
        &api_key,
        ProviderRequest::ChatCompletions(request),
    )
    .await
}

pub async fn relay_chat_completion_stream_from_store(
    store: &SqliteAdminStore,
    master_key: &str,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateChatCompletionRequest,
) -> Result<Option<reqwest::Response>> {
    let decision = simulate_route_with_store(store, "chat_completion", &request.model).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) = resolve_provider_secret(store, master_key, tenant_id, &provider.id).await?
    else {
        return Ok(None);
    };

    execute_stream_provider_request(
        &provider.adapter_kind,
        provider.base_url,
        &api_key,
        ProviderRequest::ChatCompletionsStream(request),
    )
    .await
}

pub async fn relay_response_from_store(
    store: &SqliteAdminStore,
    master_key: &str,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateResponseRequest,
) -> Result<Option<Value>> {
    let decision = simulate_route_with_store(store, "responses", &request.model).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) = resolve_provider_secret(store, master_key, tenant_id, &provider.id).await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &provider.adapter_kind,
        provider.base_url,
        &api_key,
        ProviderRequest::Responses(request),
    )
    .await
}

pub async fn relay_embedding_from_store(
    store: &SqliteAdminStore,
    master_key: &str,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateEmbeddingRequest,
) -> Result<Option<Value>> {
    let decision = simulate_route_with_store(store, "embeddings", &request.model).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) = resolve_provider_secret(store, master_key, tenant_id, &provider.id).await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &provider.adapter_kind,
        provider.base_url,
        &api_key,
        ProviderRequest::Embeddings(request),
    )
    .await
}

pub fn create_chat_completion(
    _tenant_id: &str,
    _project_id: &str,
    model: &str,
) -> Result<ChatCompletionResponse> {
    Ok(ChatCompletionResponse::empty("chatcmpl_1", model))
}

pub fn create_response(_tenant_id: &str, _project_id: &str, model: &str) -> Result<ResponseObject> {
    Ok(ResponseObject::empty("resp_1", model))
}

pub fn create_embedding(
    _tenant_id: &str,
    _project_id: &str,
    model: &str,
) -> Result<CreateEmbeddingResponse> {
    Ok(CreateEmbeddingResponse::empty(model))
}

fn default_provider_registry() -> ProviderRegistry {
    let mut registry = ProviderRegistry::new();
    registry.register_factory("openai", |base_url| {
        Box::new(OpenAiProviderAdapter::new(base_url))
    });
    registry.register_factory("openrouter", |base_url| {
        Box::new(OpenRouterProviderAdapter::new(base_url))
    });
    registry.register_factory("ollama", |base_url| {
        Box::new(OllamaProviderAdapter::new(base_url))
    });
    registry
}

async fn execute_json_provider_request(
    adapter_kind: &str,
    base_url: String,
    api_key: &str,
    request: ProviderRequest<'_>,
) -> Result<Option<Value>> {
    let registry = default_provider_registry();
    let Some(adapter) = registry.resolve(adapter_kind, base_url) else {
        return Ok(None);
    };

    let response = adapter.execute(api_key, request).await?;
    Ok(response.into_json())
}

async fn execute_stream_provider_request(
    adapter_kind: &str,
    base_url: String,
    api_key: &str,
    request: ProviderRequest<'_>,
) -> Result<Option<reqwest::Response>> {
    let registry = default_provider_registry();
    let Some(adapter) = registry.resolve(adapter_kind, base_url) else {
        return Ok(None);
    };

    let response = adapter.execute(api_key, request).await?;
    Ok(response.into_stream())
}
