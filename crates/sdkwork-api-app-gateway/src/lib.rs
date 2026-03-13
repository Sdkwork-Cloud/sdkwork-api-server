use anyhow::Result;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use sdkwork_api_app_credential::{resolve_provider_secret_with_manager, CredentialSecretManager};
use sdkwork_api_app_routing::simulate_route_with_store;
use sdkwork_api_contract_openai::assistants::{AssistantObject, CreateAssistantRequest};
use sdkwork_api_contract_openai::audio::{
    CreateSpeechRequest, CreateTranscriptionRequest, CreateTranslationRequest, SpeechResponse,
    TranscriptionObject, TranslationObject,
};
use sdkwork_api_contract_openai::batches::{BatchObject, CreateBatchRequest};
use sdkwork_api_contract_openai::chat_completions::ChatCompletionResponse;
use sdkwork_api_contract_openai::chat_completions::CreateChatCompletionRequest;
use sdkwork_api_contract_openai::completions::{CompletionObject, CreateCompletionRequest};
use sdkwork_api_contract_openai::embeddings::CreateEmbeddingRequest;
use sdkwork_api_contract_openai::embeddings::CreateEmbeddingResponse;
use sdkwork_api_contract_openai::evals::{CreateEvalRequest, EvalObject};
use sdkwork_api_contract_openai::fine_tuning::{CreateFineTuningJobRequest, FineTuningJobObject};
use sdkwork_api_contract_openai::images::{CreateImageRequest, ImageObject, ImagesResponse};
use sdkwork_api_contract_openai::models::{ListModelsResponse, ModelObject};
use sdkwork_api_contract_openai::moderations::{
    CreateModerationRequest, ModerationCategoryScores, ModerationResponse, ModerationResult,
};
use sdkwork_api_contract_openai::realtime::{CreateRealtimeSessionRequest, RealtimeSessionObject};
use sdkwork_api_contract_openai::responses::CreateResponseRequest;
use sdkwork_api_contract_openai::responses::ResponseObject;
use sdkwork_api_contract_openai::vector_stores::{CreateVectorStoreRequest, VectorStoreObject};
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

pub async fn relay_speech_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateSpeechRequest,
) -> Result<Option<reqwest::Response>> {
    let decision = simulate_route_with_store(store, "audio_speech", &request.model).await?;
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
        ProviderRequest::AudioSpeech(request),
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

pub async fn relay_assistant_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateAssistantRequest,
) -> Result<Option<Value>> {
    let decision = simulate_route_with_store(store, "assistants", &request.model).await?;
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
        ProviderRequest::Assistants(request),
    )
    .await
}

pub async fn relay_realtime_session_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateRealtimeSessionRequest,
) -> Result<Option<Value>> {
    let decision = simulate_route_with_store(store, "realtime_sessions", &request.model).await?;
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
        ProviderRequest::RealtimeSessions(request),
    )
    .await
}

pub async fn relay_eval_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateEvalRequest,
) -> Result<Option<Value>> {
    let decision = simulate_route_with_store(store, "evals", &request.name).await?;
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
        ProviderRequest::Evals(request),
    )
    .await
}

pub async fn relay_batch_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateBatchRequest,
) -> Result<Option<Value>> {
    let decision = simulate_route_with_store(store, "batches", &request.endpoint).await?;
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
        ProviderRequest::Batches(request),
    )
    .await
}

pub async fn relay_vector_store_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateVectorStoreRequest,
) -> Result<Option<Value>> {
    let decision = simulate_route_with_store(store, "vector_stores", &request.name).await?;
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
        ProviderRequest::VectorStores(request),
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

pub fn create_speech_response(
    _tenant_id: &str,
    _project_id: &str,
    request: &CreateSpeechRequest,
) -> Result<SpeechResponse> {
    let format = request
        .response_format
        .clone()
        .unwrap_or_else(|| "wav".to_owned());
    let bytes = fallback_speech_bytes(&format);
    Ok(SpeechResponse::new(format, STANDARD.encode(bytes)))
}

pub fn create_fine_tuning_job(
    _tenant_id: &str,
    _project_id: &str,
    model: &str,
) -> Result<FineTuningJobObject> {
    Ok(FineTuningJobObject::new("ftjob_1", model))
}

pub fn create_assistant(
    _tenant_id: &str,
    _project_id: &str,
    name: &str,
    model: &str,
) -> Result<AssistantObject> {
    Ok(AssistantObject::new("asst_1", name, model))
}

pub fn create_realtime_session(
    _tenant_id: &str,
    _project_id: &str,
    model: &str,
) -> Result<RealtimeSessionObject> {
    Ok(RealtimeSessionObject::new("sess_1", model))
}

pub fn create_eval(_tenant_id: &str, _project_id: &str, name: &str) -> Result<EvalObject> {
    Ok(EvalObject::new("eval_1", name))
}

pub fn create_batch(
    _tenant_id: &str,
    _project_id: &str,
    endpoint: &str,
    input_file_id: &str,
) -> Result<BatchObject> {
    Ok(BatchObject::new("batch_1", endpoint, input_file_id))
}

pub fn create_vector_store(
    _tenant_id: &str,
    _project_id: &str,
    name: &str,
) -> Result<VectorStoreObject> {
    Ok(VectorStoreObject::new("vs_1", name))
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

fn fallback_speech_bytes(format: &str) -> Vec<u8> {
    match format {
        "wav" => vec![
            0x52, 0x49, 0x46, 0x46, 0x24, 0x00, 0x00, 0x00, 0x57, 0x41, 0x56, 0x45, 0x66, 0x6d,
            0x74, 0x20, 0x10, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x40, 0x1f, 0x00, 0x00,
            0x80, 0x3e, 0x00, 0x00, 0x02, 0x00, 0x10, 0x00, 0x64, 0x61, 0x74, 0x61, 0x00, 0x00,
            0x00, 0x00,
        ],
        "pcm" => vec![0x00, 0x00],
        _ => Vec::new(),
    }
}
