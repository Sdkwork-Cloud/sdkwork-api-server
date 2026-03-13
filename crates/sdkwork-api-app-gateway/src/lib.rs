use anyhow::Result;
use sdkwork_api_app_credential::{resolve_provider_secret_with_manager, CredentialSecretManager};
use sdkwork_api_app_routing::simulate_route_with_store;
use sdkwork_api_contract_openai::audio::{
    CreateTranscriptionRequest, CreateTranslationRequest, TranscriptionObject, TranslationObject,
};
use sdkwork_api_contract_openai::chat_completions::ChatCompletionResponse;
use sdkwork_api_contract_openai::chat_completions::CreateChatCompletionRequest;
use sdkwork_api_contract_openai::completions::{CompletionObject, CreateCompletionRequest};
use sdkwork_api_contract_openai::embeddings::CreateEmbeddingRequest;
use sdkwork_api_contract_openai::embeddings::CreateEmbeddingResponse;
use sdkwork_api_contract_openai::fine_tuning::{CreateFineTuningJobRequest, FineTuningJobObject};
use sdkwork_api_contract_openai::images::{CreateImageRequest, ImageObject, ImagesResponse};
use sdkwork_api_contract_openai::models::{ListModelsResponse, ModelObject};
use sdkwork_api_contract_openai::moderations::{
    CreateModerationRequest, ModerationCategoryScores, ModerationResponse, ModerationResult,
};
use sdkwork_api_contract_openai::responses::CreateResponseRequest;
use sdkwork_api_contract_openai::responses::ResponseObject;
use sdkwork_api_provider_core::{ProviderRegistry, ProviderRequest};
use sdkwork_api_provider_ollama::OllamaProviderAdapter;
use sdkwork_api_provider_openai::OpenAiProviderAdapter;
use sdkwork_api_provider_openrouter::OpenRouterProviderAdapter;
use sdkwork_api_storage_core::AdminStore;
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
    store: &dyn AdminStore,
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
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateChatCompletionRequest,
) -> Result<Option<Value>> {
    let decision = simulate_route_with_store(store, "chat_completion", &request.model).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
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
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateChatCompletionRequest,
) -> Result<Option<reqwest::Response>> {
    let decision = simulate_route_with_store(store, "chat_completion", &request.model).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
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
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateResponseRequest,
) -> Result<Option<Value>> {
    let decision = simulate_route_with_store(store, "responses", &request.model).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
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

pub async fn relay_completion_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateCompletionRequest,
) -> Result<Option<Value>> {
    let decision = simulate_route_with_store(store, "completions", &request.model).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &provider.adapter_kind,
        provider.base_url,
        &api_key,
        ProviderRequest::Completions(request),
    )
    .await
}

pub async fn relay_embedding_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateEmbeddingRequest,
) -> Result<Option<Value>> {
    let decision = simulate_route_with_store(store, "embeddings", &request.model).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
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

pub async fn relay_moderation_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateModerationRequest,
) -> Result<Option<Value>> {
    let decision = simulate_route_with_store(store, "moderations", &request.model).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &provider.adapter_kind,
        provider.base_url,
        &api_key,
        ProviderRequest::Moderations(request),
    )
    .await
}

pub async fn relay_image_generation_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateImageRequest,
) -> Result<Option<Value>> {
    let decision = simulate_route_with_store(store, "images", &request.model).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &provider.adapter_kind,
        provider.base_url,
        &api_key,
        ProviderRequest::ImagesGenerations(request),
    )
    .await
}

pub async fn relay_transcription_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateTranscriptionRequest,
) -> Result<Option<Value>> {
    let decision = simulate_route_with_store(store, "audio_transcriptions", &request.model).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &provider.adapter_kind,
        provider.base_url,
        &api_key,
        ProviderRequest::AudioTranscriptions(request),
    )
    .await
}

pub async fn relay_translation_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateTranslationRequest,
) -> Result<Option<Value>> {
    let decision = simulate_route_with_store(store, "audio_translations", &request.model).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &provider.adapter_kind,
        provider.base_url,
        &api_key,
        ProviderRequest::AudioTranslations(request),
    )
    .await
}

pub async fn relay_fine_tuning_job_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateFineTuningJobRequest,
) -> Result<Option<Value>> {
    let decision = simulate_route_with_store(store, "fine_tuning", &request.model).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &provider.adapter_kind,
        provider.base_url,
        &api_key,
        ProviderRequest::FineTuningJobs(request),
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

pub fn create_completion(
    _tenant_id: &str,
    _project_id: &str,
    _model: &str,
) -> Result<CompletionObject> {
    Ok(CompletionObject::new("cmpl_1", "SDKWork completion"))
}

pub fn create_embedding(
    _tenant_id: &str,
    _project_id: &str,
    model: &str,
) -> Result<CreateEmbeddingResponse> {
    Ok(CreateEmbeddingResponse::empty(model))
}

pub fn create_moderation(
    _tenant_id: &str,
    _project_id: &str,
    model: &str,
) -> Result<ModerationResponse> {
    Ok(ModerationResponse {
        id: "modr_1".to_owned(),
        model: model.to_owned(),
        results: vec![ModerationResult {
            flagged: false,
            category_scores: ModerationCategoryScores { violence: 0.0 },
        }],
    })
}

pub fn create_image_generation(
    _tenant_id: &str,
    _project_id: &str,
    _model: &str,
) -> Result<ImagesResponse> {
    Ok(ImagesResponse::new(vec![ImageObject::base64(
        "sdkwork-image",
    )]))
}

pub fn create_transcription(
    _tenant_id: &str,
    _project_id: &str,
    _model: &str,
) -> Result<TranscriptionObject> {
    Ok(TranscriptionObject::new("sdkwork transcription"))
}

pub fn create_translation(
    _tenant_id: &str,
    _project_id: &str,
    _model: &str,
) -> Result<TranslationObject> {
    Ok(TranslationObject::new("sdkwork translation"))
}

pub fn create_fine_tuning_job(
    _tenant_id: &str,
    _project_id: &str,
    model: &str,
) -> Result<FineTuningJobObject> {
    Ok(FineTuningJobObject::new("ftjob_1", model))
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
