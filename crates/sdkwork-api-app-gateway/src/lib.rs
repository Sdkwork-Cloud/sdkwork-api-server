use anyhow::Result;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use sdkwork_api_app_credential::{resolve_provider_secret_with_manager, CredentialSecretManager};
use sdkwork_api_app_routing::select_route_with_store;
use sdkwork_api_contract_openai::assistants::{
    AssistantObject, CreateAssistantRequest, DeleteAssistantResponse, ListAssistantsResponse,
    UpdateAssistantRequest,
};
use sdkwork_api_contract_openai::audio::{
    CreateSpeechRequest, CreateTranscriptionRequest, CreateTranslationRequest, SpeechResponse,
    TranscriptionObject, TranslationObject,
};
use sdkwork_api_contract_openai::batches::{BatchObject, CreateBatchRequest, ListBatchesResponse};
use sdkwork_api_contract_openai::chat_completions::{
    ChatCompletionMessageObject, ChatCompletionResponse, CreateChatCompletionRequest,
    DeleteChatCompletionResponse, ListChatCompletionMessagesResponse, ListChatCompletionsResponse,
    UpdateChatCompletionRequest,
};
use sdkwork_api_contract_openai::completions::{CompletionObject, CreateCompletionRequest};
use sdkwork_api_contract_openai::conversations::{
    ConversationItemObject, ConversationObject, CreateConversationItemsRequest,
    CreateConversationRequest, DeleteConversationItemResponse, DeleteConversationResponse,
    ListConversationItemsResponse, ListConversationsResponse, UpdateConversationRequest,
};
use sdkwork_api_contract_openai::embeddings::CreateEmbeddingRequest;
use sdkwork_api_contract_openai::embeddings::CreateEmbeddingResponse;
use sdkwork_api_contract_openai::evals::{CreateEvalRequest, EvalObject};
use sdkwork_api_contract_openai::files::{
    CreateFileRequest, DeleteFileResponse, FileObject, ListFilesResponse,
};
use sdkwork_api_contract_openai::fine_tuning::{
    CreateFineTuningJobRequest, FineTuningJobObject, ListFineTuningJobsResponse,
};
use sdkwork_api_contract_openai::images::{
    CreateImageEditRequest, CreateImageRequest, CreateImageVariationRequest, ImageObject,
    ImagesResponse,
};
use sdkwork_api_contract_openai::models::{DeleteModelResponse, ListModelsResponse, ModelObject};
use sdkwork_api_contract_openai::moderations::{
    CreateModerationRequest, ModerationCategoryScores, ModerationResponse, ModerationResult,
};
use sdkwork_api_contract_openai::realtime::{CreateRealtimeSessionRequest, RealtimeSessionObject};
use sdkwork_api_contract_openai::responses::{
    CompactResponseRequest, CountResponseInputTokensRequest, CreateResponseRequest,
    DeleteResponseResponse, ListResponseInputItemsResponse, ResponseCompactionObject,
    ResponseInputItemObject, ResponseInputTokensObject, ResponseObject,
};
use sdkwork_api_contract_openai::runs::{
    CreateRunRequest, CreateThreadAndRunRequest, ListRunStepsResponse, ListRunsResponse, RunObject,
    RunStepObject, SubmitToolOutputsRunRequest, UpdateRunRequest,
};
use sdkwork_api_contract_openai::threads::{
    CreateThreadMessageRequest, CreateThreadRequest, DeleteThreadMessageResponse,
    DeleteThreadResponse, ListThreadMessagesResponse, ThreadMessageObject, ThreadObject,
    UpdateThreadMessageRequest, UpdateThreadRequest,
};
use sdkwork_api_contract_openai::uploads::{
    AddUploadPartRequest, CompleteUploadRequest, CreateUploadRequest, UploadObject,
    UploadPartObject,
};
use sdkwork_api_contract_openai::vector_stores::{
    CreateVectorStoreFileBatchRequest, CreateVectorStoreFileRequest, CreateVectorStoreRequest,
    DeleteVectorStoreFileResponse, DeleteVectorStoreResponse, ListVectorStoreFilesResponse,
    ListVectorStoresResponse, SearchVectorStoreRequest, SearchVectorStoreResponse,
    UpdateVectorStoreRequest, VectorStoreFileBatchObject, VectorStoreFileObject, VectorStoreObject,
};
use sdkwork_api_contract_openai::videos::{
    CreateVideoRequest, DeleteVideoResponse, RemixVideoRequest, VideoObject, VideosResponse,
};
use sdkwork_api_contract_openai::webhooks::{
    CreateWebhookRequest, DeleteWebhookResponse, ListWebhooksResponse, UpdateWebhookRequest,
    WebhookObject,
};
use sdkwork_api_domain_catalog::ProxyProvider;
use sdkwork_api_domain_routing::{RoutingDecision, RoutingDecisionSource};
use sdkwork_api_extension_core::{
    ExtensionKind, ExtensionManifest, ExtensionProtocol, ExtensionRuntime,
};
use sdkwork_api_extension_host::{
    discover_extension_packages, ensure_connector_runtime_started,
    verify_discovered_extension_package_trust, BuiltinProviderExtensionFactory,
    DiscoveredExtensionPackage, ExtensionDiscoveryPolicy, ExtensionHost,
};
use sdkwork_api_provider_core::{ProviderRequest, ProviderStreamOutput};
use sdkwork_api_provider_ollama::OllamaProviderAdapter;
use sdkwork_api_provider_openai::OpenAiProviderAdapter;
use sdkwork_api_provider_openrouter::OpenRouterProviderAdapter;
use sdkwork_api_storage_core::AdminStore;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

pub const LOCAL_PROVIDER_ID: &str = "sdkwork.local";

pub fn service_name() -> &'static str {
    "gateway-service"
}

pub fn list_models(_tenant_id: &str, _project_id: &str) -> Result<ListModelsResponse> {
    Ok(ListModelsResponse::new(vec![ModelObject::new(
        "gpt-4.1", "sdkwork",
    )]))
}

pub fn get_model(_tenant_id: &str, _project_id: &str, model_id: &str) -> Result<ModelObject> {
    Ok(ModelObject::new(model_id, "sdkwork"))
}

pub fn delete_model(
    _tenant_id: &str,
    _project_id: &str,
    model_id: &str,
) -> Result<DeleteModelResponse> {
    Ok(DeleteModelResponse::deleted(model_id))
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

pub async fn get_model_from_store(
    store: &dyn AdminStore,
    _tenant_id: &str,
    _project_id: &str,
    model_id: &str,
) -> Result<Option<ModelObject>> {
    Ok(store
        .find_model(model_id)
        .await?
        .map(|entry| ModelObject::new(entry.external_name, entry.provider_id)))
}

pub async fn delete_model_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    model_id: &str,
) -> Result<Option<Value>> {
    let Some(model_entry) = store.find_model(model_id).await? else {
        return Ok(None);
    };

    if let Some(provider) = store.find_provider(&model_entry.provider_id).await? {
        if let Some(api_key) =
            resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
                .await?
        {
            let response = execute_json_provider_request_for_provider(
                store,
                &provider,
                &api_key,
                ProviderRequest::ModelsDelete(model_id),
            )
            .await?;

            if let Some(response) = response {
                let _ = store.delete_model(model_id).await?;
                return Ok(Some(response));
            }
        }
    }

    if store.delete_model(model_id).await? {
        return Ok(Some(serde_json::to_value(DeleteModelResponse::deleted(
            model_id,
        ))?));
    }

    Ok(None)
}

#[derive(Clone)]
struct ProviderExecutionTarget {
    provider_id: String,
    runtime_key: String,
    base_url: String,
    local_fallback: bool,
}

impl ProviderExecutionTarget {
    fn local() -> Self {
        Self {
            provider_id: LOCAL_PROVIDER_ID.to_owned(),
            runtime_key: String::new(),
            base_url: String::new(),
            local_fallback: true,
        }
    }

    fn upstream(provider_id: String, runtime_key: String, base_url: String) -> Self {
        Self {
            provider_id,
            runtime_key,
            base_url,
            local_fallback: false,
        }
    }
}

#[derive(Clone)]
struct ProviderExecutionDescriptor {
    provider_id: String,
    runtime_key: String,
    base_url: String,
    api_key: String,
    local_fallback: bool,
}

async fn build_extension_host_from_store(store: &dyn AdminStore) -> Result<ExtensionHost> {
    let mut host = configured_extension_host()?;

    let mut installations = store.list_extension_installations().await?;
    installations.sort_by(|left, right| left.installation_id.cmp(&right.installation_id));
    for installation in installations {
        match host.install(installation) {
            Ok(()) => {}
            Err(sdkwork_api_extension_host::ExtensionHostError::ManifestNotFound { .. }) => {}
            Err(error) => return Err(error.into()),
        }
    }

    let mut instances = store.list_extension_instances().await?;
    instances.sort_by(|left, right| left.instance_id.cmp(&right.instance_id));
    for instance in instances {
        match host.mount_instance(instance) {
            Ok(()) => {}
            Err(sdkwork_api_extension_host::ExtensionHostError::InstallationNotFound {
                ..
            }) => {}
            Err(error) => return Err(error.into()),
        }
    }

    Ok(host)
}

async fn provider_execution_target_for_provider(
    store: &dyn AdminStore,
    provider: &ProxyProvider,
) -> Result<ProviderExecutionTarget> {
    let host = build_extension_host_from_store(store).await?;

    match host.load_plan(&provider.id) {
        Ok(load_plan) => {
            if !load_plan.enabled {
                return Ok(ProviderExecutionTarget::local());
            }

            let resolved_base_url = load_plan
                .base_url
                .clone()
                .unwrap_or_else(|| provider.base_url.clone());
            if load_plan.runtime == ExtensionRuntime::Connector {
                ensure_connector_runtime_started(&load_plan, &resolved_base_url)
                    .map_err(anyhow::Error::new)?;
            }

            Ok(ProviderExecutionTarget::upstream(
                provider.id.clone(),
                load_plan.extension_id,
                resolved_base_url,
            ))
        }
        Err(sdkwork_api_extension_host::ExtensionHostError::InstanceNotFound { .. }) => {
            Ok(ProviderExecutionTarget::upstream(
                provider.id.clone(),
                provider_runtime_key(provider).to_owned(),
                provider.base_url.clone(),
            ))
        }
        Err(error) => Err(error.into()),
    }
}

async fn provider_execution_descriptor_for_provider(
    store: &dyn AdminStore,
    provider: &ProxyProvider,
    api_key: String,
) -> Result<ProviderExecutionDescriptor> {
    let target = provider_execution_target_for_provider(store, provider).await?;
    Ok(ProviderExecutionDescriptor {
        provider_id: target.provider_id,
        runtime_key: target.runtime_key,
        base_url: target.base_url,
        api_key,
        local_fallback: target.local_fallback,
    })
}

static ROUTING_DECISION_CACHE: OnceLock<Mutex<HashMap<String, RoutingDecision>>> = OnceLock::new();

fn routing_decision_cache() -> &'static Mutex<HashMap<String, RoutingDecision>> {
    ROUTING_DECISION_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn routing_decision_cache_key(
    tenant_id: &str,
    project_id: Option<&str>,
    capability: &str,
    route_key: &str,
) -> String {
    format!(
        "{tenant_id}|{}|{capability}|{route_key}",
        project_id.unwrap_or_default()
    )
}

fn cache_routing_decision(
    tenant_id: &str,
    project_id: Option<&str>,
    capability: &str,
    route_key: &str,
    decision: &RoutingDecision,
) {
    let key = routing_decision_cache_key(tenant_id, project_id, capability, route_key);
    let mut cache = match routing_decision_cache().lock() {
        Ok(cache) => cache,
        Err(poisoned) => poisoned.into_inner(),
    };
    cache.insert(key, decision.clone());
}

fn take_cached_routing_decision(
    tenant_id: &str,
    project_id: Option<&str>,
    capability: &str,
    route_key: &str,
) -> Option<RoutingDecision> {
    let key = routing_decision_cache_key(tenant_id, project_id, capability, route_key);
    let mut cache = match routing_decision_cache().lock() {
        Ok(cache) => cache,
        Err(poisoned) => poisoned.into_inner(),
    };
    cache.remove(&key)
}

async fn select_gateway_route(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: Option<&str>,
    capability: &str,
    route_key: &str,
) -> Result<RoutingDecision> {
    let decision = select_route_with_store(
        store,
        capability,
        route_key,
        RoutingDecisionSource::Gateway,
        Some(tenant_id),
        project_id,
        None,
    )
    .await?;
    cache_routing_decision(tenant_id, project_id, capability, route_key, &decision);
    Ok(decision)
}

pub async fn planned_execution_provider_id_for_route(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    route_key: &str,
) -> Result<String> {
    let decision =
        match take_cached_routing_decision(tenant_id, Some(project_id), capability, route_key) {
            Some(decision) => decision,
            None => {
                select_route_with_store(
                    store,
                    capability,
                    route_key,
                    RoutingDecisionSource::Gateway,
                    Some(tenant_id),
                    Some(project_id),
                    None,
                )
                .await?
            }
        };
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(LOCAL_PROVIDER_ID.to_owned());
    };

    let target = provider_execution_target_for_provider(store, &provider).await?;
    if target.local_fallback {
        return Ok(target.provider_id);
    }

    let has_credential = store
        .find_provider_credential(tenant_id, &provider.id)
        .await?
        .is_some();

    if has_credential {
        Ok(target.provider_id)
    } else {
        Ok(LOCAL_PROVIDER_ID.to_owned())
    }
}

async fn resolve_non_model_provider(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    route_key: &str,
) -> Result<Option<(String, String, String)>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(project_id), capability, route_key).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    let descriptor = provider_execution_descriptor_for_provider(store, &provider, api_key).await?;
    if descriptor.local_fallback {
        return Ok(None);
    }

    Ok(Some((
        descriptor.runtime_key,
        descriptor.base_url,
        descriptor.api_key,
    )))
}

pub async fn relay_chat_completion_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateChatCompletionRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "chat_completion",
        &request.model,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ChatCompletions(request),
    )
    .await
}

pub async fn relay_list_chat_completions_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "chat_completion",
        "chat_completions",
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ChatCompletionsList,
    )
    .await
}

pub async fn relay_get_chat_completion_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    completion_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "chat_completion",
        completion_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ChatCompletionsRetrieve(completion_id),
    )
    .await
}

pub async fn relay_update_chat_completion_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    completion_id: &str,
    request: &UpdateChatCompletionRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "chat_completion",
        completion_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ChatCompletionsUpdate(completion_id, request),
    )
    .await
}

pub async fn relay_delete_chat_completion_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    completion_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "chat_completion",
        completion_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ChatCompletionsDelete(completion_id),
    )
    .await
}

pub async fn relay_list_chat_completion_messages_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    completion_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "chat_completion",
        completion_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ChatCompletionsMessagesList(completion_id),
    )
    .await
}

pub async fn relay_chat_completion_stream_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateChatCompletionRequest,
) -> Result<Option<ProviderStreamOutput>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "chat_completion",
        &request.model,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_stream_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ChatCompletionsStream(request),
    )
    .await
}

pub async fn relay_conversation_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateConversationRequest,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "responses",
        "conversations",
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::Conversations(request),
    )
    .await
}

pub async fn relay_list_conversations_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "responses",
        "conversations",
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ConversationsList,
    )
    .await
}

pub async fn relay_get_conversation_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "responses",
        conversation_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ConversationsRetrieve(conversation_id),
    )
    .await
}

pub async fn relay_update_conversation_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
    request: &UpdateConversationRequest,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "responses",
        conversation_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ConversationsUpdate(conversation_id, request),
    )
    .await
}

pub async fn relay_delete_conversation_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "responses",
        conversation_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ConversationsDelete(conversation_id),
    )
    .await
}

pub async fn relay_conversation_items_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
    request: &CreateConversationItemsRequest,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "responses",
        conversation_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ConversationItems(conversation_id, request),
    )
    .await
}

pub async fn relay_thread_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateThreadRequest,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        "threads",
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::Threads(request),
    )
    .await
}

pub async fn relay_get_thread_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadsRetrieve(thread_id),
    )
    .await
}

pub async fn relay_update_thread_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    request: &UpdateThreadRequest,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadsUpdate(thread_id, request),
    )
    .await
}

pub async fn relay_delete_thread_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadsDelete(thread_id),
    )
    .await
}

pub async fn relay_thread_messages_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    request: &CreateThreadMessageRequest,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadMessages(thread_id, request),
    )
    .await
}

pub async fn relay_list_thread_messages_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadMessagesList(thread_id),
    )
    .await
}

pub async fn relay_get_thread_message_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    message_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadMessagesRetrieve(thread_id, message_id),
    )
    .await
}

pub async fn relay_update_thread_message_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    message_id: &str,
    request: &UpdateThreadMessageRequest,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadMessagesUpdate(thread_id, message_id, request),
    )
    .await
}

pub async fn relay_delete_thread_message_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    message_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadMessagesDelete(thread_id, message_id),
    )
    .await
}

pub async fn relay_thread_run_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    request: &CreateRunRequest,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadRuns(thread_id, request),
    )
    .await
}

pub async fn relay_list_thread_runs_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadRunsList(thread_id),
    )
    .await
}

pub async fn relay_get_thread_run_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadRunsRetrieve(thread_id, run_id),
    )
    .await
}

pub async fn relay_update_thread_run_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
    request: &UpdateRunRequest,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadRunsUpdate(thread_id, run_id, request),
    )
    .await
}

pub async fn relay_cancel_thread_run_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadRunsCancel(thread_id, run_id),
    )
    .await
}

pub async fn relay_submit_thread_run_tool_outputs_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
    request: &SubmitToolOutputsRunRequest,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadRunsSubmitToolOutputs(thread_id, run_id, request),
    )
    .await
}

pub async fn relay_list_thread_run_steps_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadRunStepsList(thread_id, run_id),
    )
    .await
}

pub async fn relay_get_thread_run_step_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
    step_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadRunStepsRetrieve(thread_id, run_id, step_id),
    )
    .await
}

pub async fn relay_thread_and_run_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateThreadAndRunRequest,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        "threads/runs",
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadsRuns(request),
    )
    .await
}

pub async fn relay_list_conversation_items_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "responses",
        conversation_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ConversationItemsList(conversation_id),
    )
    .await
}

pub async fn relay_get_conversation_item_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
    item_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "responses",
        conversation_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ConversationItemsRetrieve(conversation_id, item_id),
    )
    .await
}

pub async fn relay_delete_conversation_item_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
    item_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "responses",
        conversation_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ConversationItemsDelete(conversation_id, item_id),
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
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "responses",
        &request.model,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::Responses(request),
    )
    .await
}

pub async fn relay_response_stream_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateResponseRequest,
) -> Result<Option<ProviderStreamOutput>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "responses",
        &request.model,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_stream_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ResponsesStream(request),
    )
    .await
}

pub async fn relay_count_response_input_tokens_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CountResponseInputTokensRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "responses",
        &request.model,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ResponsesInputTokens(request),
    )
    .await
}

pub async fn relay_get_response_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    response_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "responses",
        response_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ResponsesRetrieve(response_id),
    )
    .await
}

pub async fn relay_cancel_response_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    response_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "responses",
        response_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ResponsesCancel(response_id),
    )
    .await
}

pub async fn relay_delete_response_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    response_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "responses",
        response_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ResponsesDelete(response_id),
    )
    .await
}

pub async fn relay_compact_response_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CompactResponseRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "responses",
        &request.model,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ResponsesCompact(request),
    )
    .await
}

pub async fn relay_list_response_input_items_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    response_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "responses",
        response_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ResponsesInputItemsList(response_id),
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
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "completions",
        &request.model,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
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
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "embeddings",
        &request.model,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
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
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "moderations",
        &request.model,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
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
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "images",
        &request.model,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ImagesGenerations(request),
    )
    .await
}

pub async fn relay_image_edit_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateImageEditRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "images",
        request.model_or_default(),
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ImagesEdits(request),
    )
    .await
}

pub async fn relay_image_variation_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateImageVariationRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "images",
        request.model_or_default(),
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ImagesVariations(request),
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
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "audio_transcriptions",
        &request.model,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
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
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "audio_translations",
        &request.model,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
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
) -> Result<Option<ProviderStreamOutput>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "audio_speech",
        &request.model,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_stream_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::AudioSpeech(request),
    )
    .await
}

pub async fn relay_file_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateFileRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "files",
        &request.purpose,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::Files(request),
    )
    .await
}

pub async fn relay_list_files_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "files", "files").await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::FilesList,
    )
    .await
}

pub async fn relay_get_file_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    file_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "files", file_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::FilesRetrieve(file_id),
    )
    .await
}

pub async fn relay_delete_file_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    file_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "files", file_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::FilesDelete(file_id),
    )
    .await
}

pub async fn relay_file_content_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    file_id: &str,
) -> Result<Option<ProviderStreamOutput>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "files", file_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_stream_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::FilesContent(file_id),
    )
    .await
}

pub async fn relay_upload_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateUploadRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "uploads",
        &request.purpose,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::Uploads(request),
    )
    .await
}

pub async fn relay_upload_part_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &AddUploadPartRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "uploads",
        &request.upload_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::UploadParts(request),
    )
    .await
}

pub async fn relay_complete_upload_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CompleteUploadRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "uploads",
        &request.upload_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::UploadComplete(request),
    )
    .await
}

pub async fn relay_cancel_upload_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    upload_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "uploads", upload_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::UploadCancel(upload_id),
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
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "fine_tuning",
        &request.model,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::FineTuningJobs(request),
    )
    .await
}

pub async fn relay_list_fine_tuning_jobs_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "fine_tuning", "jobs").await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::FineTuningJobsList,
    )
    .await
}

pub async fn relay_get_fine_tuning_job_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    job_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "fine_tuning", job_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::FineTuningJobsRetrieve(job_id),
    )
    .await
}

pub async fn relay_cancel_fine_tuning_job_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    job_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "fine_tuning", job_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::FineTuningJobsCancel(job_id),
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
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "assistants",
        &request.model,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::Assistants(request),
    )
    .await
}

pub async fn relay_list_assistants_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "assistants",
        "assistants",
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::AssistantsList,
    )
    .await
}

pub async fn relay_get_assistant_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    assistant_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "assistants",
        assistant_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::AssistantsRetrieve(assistant_id),
    )
    .await
}

pub async fn relay_update_assistant_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    assistant_id: &str,
    request: &UpdateAssistantRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "assistants",
        assistant_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::AssistantsUpdate(assistant_id, request),
    )
    .await
}

pub async fn relay_delete_assistant_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    assistant_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "assistants",
        assistant_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::AssistantsDelete(assistant_id),
    )
    .await
}

pub async fn relay_webhook_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateWebhookRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "webhooks",
        &request.url,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::Webhooks(request),
    )
    .await
}

pub async fn relay_list_webhooks_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "webhooks", "webhooks").await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::WebhooksList,
    )
    .await
}

pub async fn relay_get_webhook_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    webhook_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "webhooks", webhook_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::WebhooksRetrieve(webhook_id),
    )
    .await
}

pub async fn relay_update_webhook_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    webhook_id: &str,
    request: &UpdateWebhookRequest,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "webhooks", webhook_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::WebhooksUpdate(webhook_id, request),
    )
    .await
}

pub async fn relay_delete_webhook_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    webhook_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "webhooks", webhook_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::WebhooksDelete(webhook_id),
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
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "realtime_sessions",
        &request.model,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
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
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "evals", &request.name).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
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
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "batches",
        &request.endpoint,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::Batches(request),
    )
    .await
}

pub async fn relay_list_batches_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "batches", "batches").await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::BatchesList,
    )
    .await
}

pub async fn relay_get_batch_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    batch_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "batches", batch_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::BatchesRetrieve(batch_id),
    )
    .await
}

pub async fn relay_cancel_batch_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    batch_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "batches", batch_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::BatchesCancel(batch_id),
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
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_stores",
        &request.name,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VectorStores(request),
    )
    .await
}

pub async fn relay_list_vector_stores_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_stores",
        "vector_stores",
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VectorStoresList,
    )
    .await
}

pub async fn relay_get_vector_store_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_stores",
        vector_store_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VectorStoresRetrieve(vector_store_id),
    )
    .await
}

pub async fn relay_update_vector_store_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    request: &UpdateVectorStoreRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_stores",
        vector_store_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VectorStoresUpdate(vector_store_id, request),
    )
    .await
}

pub async fn relay_delete_vector_store_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_stores",
        vector_store_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VectorStoresDelete(vector_store_id),
    )
    .await
}

pub async fn relay_search_vector_store_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    request: &SearchVectorStoreRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_store_search",
        vector_store_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VectorStoresSearch(vector_store_id, request),
    )
    .await
}

pub async fn relay_vector_store_file_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    request: &CreateVectorStoreFileRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_store_files",
        vector_store_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VectorStoreFiles(vector_store_id, request),
    )
    .await
}

pub async fn relay_list_vector_store_files_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_store_files",
        vector_store_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VectorStoreFilesList(vector_store_id),
    )
    .await
}

pub async fn relay_get_vector_store_file_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    file_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_store_files",
        vector_store_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VectorStoreFilesRetrieve(vector_store_id, file_id),
    )
    .await
}

pub async fn relay_delete_vector_store_file_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    file_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_store_files",
        vector_store_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VectorStoreFilesDelete(vector_store_id, file_id),
    )
    .await
}

pub async fn relay_vector_store_file_batch_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    request: &CreateVectorStoreFileBatchRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_store_file_batches",
        vector_store_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VectorStoreFileBatches(vector_store_id, request),
    )
    .await
}

pub async fn relay_get_vector_store_file_batch_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    batch_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_store_file_batches",
        vector_store_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VectorStoreFileBatchesRetrieve(vector_store_id, batch_id),
    )
    .await
}

pub async fn relay_cancel_vector_store_file_batch_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    batch_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_store_file_batches",
        vector_store_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VectorStoreFileBatchesCancel(vector_store_id, batch_id),
    )
    .await
}

pub async fn relay_list_vector_store_file_batch_files_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    batch_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_store_file_batches",
        vector_store_id,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VectorStoreFileBatchesListFiles(vector_store_id, batch_id),
    )
    .await
}

pub async fn relay_video_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateVideoRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "videos",
        &request.model,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::Videos(request),
    )
    .await
}

pub async fn relay_list_videos_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "videos", "videos").await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VideosList,
    )
    .await
}

pub async fn relay_get_video_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    video_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "videos", video_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VideosRetrieve(video_id),
    )
    .await
}

pub async fn relay_delete_video_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    video_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "videos", video_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VideosDelete(video_id),
    )
    .await
}

pub async fn relay_video_content_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    video_id: &str,
) -> Result<Option<ProviderStreamOutput>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "videos", video_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_stream_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VideosContent(video_id),
    )
    .await
}

pub async fn relay_remix_video_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    video_id: &str,
    request: &RemixVideoRequest,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "videos", video_id).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::VideosRemix(video_id, request),
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

pub fn create_conversation(_tenant_id: &str, _project_id: &str) -> Result<ConversationObject> {
    Ok(ConversationObject::new("conv_1"))
}

pub fn list_conversations(
    _tenant_id: &str,
    _project_id: &str,
) -> Result<ListConversationsResponse> {
    Ok(ListConversationsResponse::new(vec![
        ConversationObject::new("conv_1"),
    ]))
}

pub fn get_conversation(
    _tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
) -> Result<ConversationObject> {
    Ok(ConversationObject::new(conversation_id))
}

pub fn update_conversation(
    _tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
    metadata: Value,
) -> Result<ConversationObject> {
    Ok(ConversationObject::with_metadata(conversation_id, metadata))
}

pub fn delete_conversation(
    _tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
) -> Result<DeleteConversationResponse> {
    Ok(DeleteConversationResponse::deleted(conversation_id))
}

pub fn create_conversation_items(
    _tenant_id: &str,
    _project_id: &str,
    _conversation_id: &str,
) -> Result<ListConversationItemsResponse> {
    Ok(ListConversationItemsResponse::new(vec![
        ConversationItemObject::message("item_1", "assistant", "hello"),
    ]))
}

pub fn list_conversation_items(
    _tenant_id: &str,
    _project_id: &str,
    _conversation_id: &str,
) -> Result<ListConversationItemsResponse> {
    Ok(ListConversationItemsResponse::new(vec![
        ConversationItemObject::message("item_1", "assistant", "hello"),
    ]))
}

pub fn get_conversation_item(
    _tenant_id: &str,
    _project_id: &str,
    _conversation_id: &str,
    item_id: &str,
) -> Result<ConversationItemObject> {
    Ok(ConversationItemObject::message(
        item_id,
        "assistant",
        "hello",
    ))
}

pub fn delete_conversation_item(
    _tenant_id: &str,
    _project_id: &str,
    _conversation_id: &str,
    item_id: &str,
) -> Result<DeleteConversationItemResponse> {
    Ok(DeleteConversationItemResponse::deleted(item_id))
}

pub fn list_chat_completions(
    _tenant_id: &str,
    _project_id: &str,
) -> Result<ListChatCompletionsResponse> {
    Ok(ListChatCompletionsResponse::new(vec![
        ChatCompletionResponse::empty("chatcmpl_1", "gpt-4.1"),
    ]))
}

pub fn get_chat_completion(
    _tenant_id: &str,
    _project_id: &str,
    completion_id: &str,
) -> Result<ChatCompletionResponse> {
    Ok(ChatCompletionResponse::empty(completion_id, "gpt-4.1"))
}

pub fn update_chat_completion(
    _tenant_id: &str,
    _project_id: &str,
    completion_id: &str,
    metadata: Value,
) -> Result<ChatCompletionResponse> {
    Ok(ChatCompletionResponse::with_metadata(
        completion_id,
        "gpt-4.1",
        metadata,
    ))
}

pub fn delete_chat_completion(
    _tenant_id: &str,
    _project_id: &str,
    completion_id: &str,
) -> Result<DeleteChatCompletionResponse> {
    Ok(DeleteChatCompletionResponse::deleted(completion_id))
}

pub fn list_chat_completion_messages(
    _tenant_id: &str,
    _project_id: &str,
    _completion_id: &str,
) -> Result<ListChatCompletionMessagesResponse> {
    Ok(ListChatCompletionMessagesResponse::new(vec![
        ChatCompletionMessageObject::assistant("msg_1", "hello"),
    ]))
}

pub fn create_response(_tenant_id: &str, _project_id: &str, model: &str) -> Result<ResponseObject> {
    Ok(ResponseObject::empty("resp_1", model))
}

pub fn count_response_input_tokens(
    _tenant_id: &str,
    _project_id: &str,
    _model: &str,
) -> Result<ResponseInputTokensObject> {
    Ok(ResponseInputTokensObject::new(42))
}

pub fn get_response(
    _tenant_id: &str,
    _project_id: &str,
    response_id: &str,
) -> Result<ResponseObject> {
    Ok(ResponseObject::empty(response_id, "gpt-4.1"))
}

pub fn list_response_input_items(
    _tenant_id: &str,
    _project_id: &str,
    _response_id: &str,
) -> Result<ListResponseInputItemsResponse> {
    Ok(ListResponseInputItemsResponse::new(vec![
        ResponseInputItemObject::message("item_1"),
    ]))
}

pub fn delete_response(
    _tenant_id: &str,
    _project_id: &str,
    response_id: &str,
) -> Result<DeleteResponseResponse> {
    Ok(DeleteResponseResponse::deleted(response_id))
}

pub fn cancel_response(
    _tenant_id: &str,
    _project_id: &str,
    response_id: &str,
) -> Result<ResponseObject> {
    Ok(ResponseObject::cancelled(response_id, "gpt-4.1"))
}

pub fn compact_response(
    _tenant_id: &str,
    _project_id: &str,
    model: &str,
) -> Result<ResponseCompactionObject> {
    Ok(ResponseCompactionObject::new("resp_cmp_1", model))
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

pub fn create_image_edit(
    _tenant_id: &str,
    _project_id: &str,
    _request: &CreateImageEditRequest,
) -> Result<ImagesResponse> {
    Ok(ImagesResponse::new(vec![ImageObject::base64(
        "sdkwork-image",
    )]))
}

pub fn create_image_variation(
    _tenant_id: &str,
    _project_id: &str,
    _request: &CreateImageVariationRequest,
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

pub fn create_file(
    _tenant_id: &str,
    _project_id: &str,
    request: &CreateFileRequest,
) -> Result<FileObject> {
    Ok(FileObject::with_bytes(
        "file_1",
        &request.filename,
        &request.purpose,
        request.bytes.len() as u64,
    ))
}

pub fn list_files(_tenant_id: &str, _project_id: &str) -> Result<ListFilesResponse> {
    Ok(ListFilesResponse::new(vec![FileObject::with_bytes(
        "file_1",
        "train.jsonl",
        "fine-tune",
        2,
    )]))
}

pub fn get_file(_tenant_id: &str, _project_id: &str, file_id: &str) -> Result<FileObject> {
    Ok(FileObject::with_bytes(
        file_id,
        "train.jsonl",
        "fine-tune",
        2,
    ))
}

pub fn delete_file(
    _tenant_id: &str,
    _project_id: &str,
    file_id: &str,
) -> Result<DeleteFileResponse> {
    Ok(DeleteFileResponse::deleted(file_id))
}

pub fn file_content(_tenant_id: &str, _project_id: &str, _file_id: &str) -> Result<Vec<u8>> {
    Ok(b"{}".to_vec())
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

pub fn create_upload(
    _tenant_id: &str,
    _project_id: &str,
    request: &CreateUploadRequest,
) -> Result<UploadObject> {
    Ok(UploadObject::with_details(
        "upload_1",
        &request.filename,
        &request.purpose,
        &request.mime_type,
        request.bytes,
        vec![],
    ))
}

pub fn create_video(
    _tenant_id: &str,
    _project_id: &str,
    _model: &str,
    _prompt: &str,
) -> Result<VideosResponse> {
    Ok(VideosResponse::new(vec![VideoObject::new(
        "video_1",
        "https://example.com/video.mp4",
    )]))
}

pub fn list_videos(_tenant_id: &str, _project_id: &str) -> Result<VideosResponse> {
    Ok(VideosResponse::new(vec![VideoObject::new(
        "video_1",
        "https://example.com/video.mp4",
    )]))
}

pub fn get_video(_tenant_id: &str, _project_id: &str, video_id: &str) -> Result<VideoObject> {
    Ok(VideoObject::new(video_id, "https://example.com/video.mp4"))
}

pub fn delete_video(
    _tenant_id: &str,
    _project_id: &str,
    video_id: &str,
) -> Result<DeleteVideoResponse> {
    Ok(DeleteVideoResponse::deleted(video_id))
}

pub fn video_content(_tenant_id: &str, _project_id: &str, _video_id: &str) -> Result<Vec<u8>> {
    Ok(b"VIDEO".to_vec())
}

pub fn remix_video(
    _tenant_id: &str,
    _project_id: &str,
    _video_id: &str,
    _prompt: &str,
) -> Result<VideosResponse> {
    Ok(VideosResponse::new(vec![VideoObject::new(
        "video_1_remix",
        "https://example.com/video-remix.mp4",
    )]))
}

pub fn create_upload_part(
    _tenant_id: &str,
    _project_id: &str,
    request: &AddUploadPartRequest,
) -> Result<UploadPartObject> {
    Ok(UploadPartObject::new("part_1", &request.upload_id))
}

pub fn complete_upload(
    _tenant_id: &str,
    _project_id: &str,
    request: &CompleteUploadRequest,
) -> Result<UploadObject> {
    Ok(UploadObject::completed(
        &request.upload_id,
        "input.jsonl",
        "batch",
        "application/jsonl",
        0,
        request.part_ids.clone(),
    ))
}

pub fn cancel_upload(_tenant_id: &str, _project_id: &str, upload_id: &str) -> Result<UploadObject> {
    Ok(UploadObject::cancelled(
        upload_id,
        "input.jsonl",
        "batch",
        "application/jsonl",
        0,
        vec![],
    ))
}

pub fn create_fine_tuning_job(
    _tenant_id: &str,
    _project_id: &str,
    model: &str,
) -> Result<FineTuningJobObject> {
    Ok(FineTuningJobObject::new("ftjob_1", model))
}

pub fn list_fine_tuning_jobs(
    _tenant_id: &str,
    _project_id: &str,
) -> Result<ListFineTuningJobsResponse> {
    Ok(ListFineTuningJobsResponse::new(vec![
        FineTuningJobObject::new("ftjob_1", "gpt-4.1-mini"),
    ]))
}

pub fn get_fine_tuning_job(
    _tenant_id: &str,
    _project_id: &str,
    job_id: &str,
) -> Result<FineTuningJobObject> {
    Ok(FineTuningJobObject::new(job_id, "gpt-4.1-mini"))
}

pub fn cancel_fine_tuning_job(
    _tenant_id: &str,
    _project_id: &str,
    job_id: &str,
) -> Result<FineTuningJobObject> {
    Ok(FineTuningJobObject::cancelled(job_id, "gpt-4.1-mini"))
}

pub fn create_assistant(
    _tenant_id: &str,
    _project_id: &str,
    name: &str,
    model: &str,
) -> Result<AssistantObject> {
    Ok(AssistantObject::new("asst_1", name, model))
}

pub fn list_assistants(_tenant_id: &str, _project_id: &str) -> Result<ListAssistantsResponse> {
    Ok(ListAssistantsResponse::new(vec![AssistantObject::new(
        "asst_1", "Support", "gpt-4.1",
    )]))
}

pub fn get_assistant(
    _tenant_id: &str,
    _project_id: &str,
    assistant_id: &str,
) -> Result<AssistantObject> {
    Ok(AssistantObject::new(assistant_id, "Support", "gpt-4.1"))
}

pub fn update_assistant(
    _tenant_id: &str,
    _project_id: &str,
    assistant_id: &str,
    name: &str,
) -> Result<AssistantObject> {
    Ok(AssistantObject::new(assistant_id, name, "gpt-4.1"))
}

pub fn delete_assistant(
    _tenant_id: &str,
    _project_id: &str,
    assistant_id: &str,
) -> Result<DeleteAssistantResponse> {
    Ok(DeleteAssistantResponse::deleted(assistant_id))
}

pub fn create_thread(_tenant_id: &str, _project_id: &str) -> Result<ThreadObject> {
    Ok(ThreadObject::new("thread_1"))
}

pub fn get_thread(_tenant_id: &str, _project_id: &str, thread_id: &str) -> Result<ThreadObject> {
    Ok(ThreadObject::new(thread_id))
}

pub fn update_thread(_tenant_id: &str, _project_id: &str, thread_id: &str) -> Result<ThreadObject> {
    Ok(ThreadObject::new(thread_id))
}

pub fn delete_thread(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
) -> Result<DeleteThreadResponse> {
    Ok(DeleteThreadResponse::deleted(thread_id))
}

pub fn create_thread_message(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    role: &str,
    text: &str,
) -> Result<ThreadMessageObject> {
    Ok(ThreadMessageObject::text("msg_1", thread_id, role, text))
}

pub fn list_thread_messages(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
) -> Result<ListThreadMessagesResponse> {
    Ok(ListThreadMessagesResponse::new(vec![
        ThreadMessageObject::text("msg_1", thread_id, "assistant", "hello"),
    ]))
}

pub fn get_thread_message(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    message_id: &str,
) -> Result<ThreadMessageObject> {
    Ok(ThreadMessageObject::text(
        message_id,
        thread_id,
        "assistant",
        "hello",
    ))
}

pub fn update_thread_message(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    message_id: &str,
) -> Result<ThreadMessageObject> {
    Ok(ThreadMessageObject::text(
        message_id,
        thread_id,
        "assistant",
        "hello",
    ))
}

pub fn delete_thread_message(
    _tenant_id: &str,
    _project_id: &str,
    _thread_id: &str,
    message_id: &str,
) -> Result<DeleteThreadMessageResponse> {
    Ok(DeleteThreadMessageResponse::deleted(message_id))
}

pub fn create_thread_run(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    assistant_id: &str,
    model: Option<&str>,
) -> Result<RunObject> {
    Ok(RunObject::queued(
        "run_1",
        thread_id,
        assistant_id,
        model.unwrap_or("gpt-4.1"),
    ))
}

pub fn create_thread_and_run(
    _tenant_id: &str,
    _project_id: &str,
    assistant_id: &str,
) -> Result<RunObject> {
    Ok(RunObject::queued(
        "run_1",
        "thread_1",
        assistant_id,
        "gpt-4.1",
    ))
}

pub fn list_thread_runs(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
) -> Result<ListRunsResponse> {
    Ok(ListRunsResponse::new(vec![RunObject::queued(
        "run_1", thread_id, "asst_1", "gpt-4.1",
    )]))
}

pub fn get_thread_run(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> Result<RunObject> {
    Ok(RunObject::in_progress(
        run_id, thread_id, "asst_1", "gpt-4.1",
    ))
}

pub fn update_thread_run(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> Result<RunObject> {
    Ok(RunObject::with_metadata(
        run_id,
        thread_id,
        "asst_1",
        "gpt-4.1",
        "in_progress",
        serde_json::json!({"priority":"high"}),
    ))
}

pub fn cancel_thread_run(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> Result<RunObject> {
    Ok(RunObject::cancelled(run_id, thread_id, "asst_1", "gpt-4.1"))
}

pub fn submit_thread_run_tool_outputs(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
    _tool_outputs: Vec<(&str, &str)>,
) -> Result<RunObject> {
    Ok(RunObject::queued(run_id, thread_id, "asst_1", "gpt-4.1"))
}

pub fn list_thread_run_steps(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> Result<ListRunStepsResponse> {
    Ok(ListRunStepsResponse::new(vec![
        RunStepObject::message_creation("step_1", thread_id, run_id, "asst_1", "msg_1"),
    ]))
}

pub fn get_thread_run_step(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
    step_id: &str,
) -> Result<RunStepObject> {
    Ok(RunStepObject::message_creation(
        step_id, thread_id, run_id, "asst_1", "msg_1",
    ))
}

pub fn create_webhook(
    _tenant_id: &str,
    _project_id: &str,
    url: &str,
    _events: &[String],
) -> Result<WebhookObject> {
    Ok(WebhookObject::new("wh_1", url))
}

pub fn list_webhooks(_tenant_id: &str, _project_id: &str) -> Result<ListWebhooksResponse> {
    Ok(ListWebhooksResponse::new(vec![WebhookObject::new(
        "wh_1",
        "https://example.com/webhook",
    )]))
}

pub fn get_webhook(_tenant_id: &str, _project_id: &str, webhook_id: &str) -> Result<WebhookObject> {
    Ok(WebhookObject::new(
        webhook_id,
        "https://example.com/webhook",
    ))
}

pub fn update_webhook(
    _tenant_id: &str,
    _project_id: &str,
    webhook_id: &str,
    url: &str,
) -> Result<WebhookObject> {
    Ok(WebhookObject::new(webhook_id, url))
}

pub fn delete_webhook(
    _tenant_id: &str,
    _project_id: &str,
    webhook_id: &str,
) -> Result<DeleteWebhookResponse> {
    Ok(DeleteWebhookResponse::deleted(webhook_id))
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

pub fn list_batches(_tenant_id: &str, _project_id: &str) -> Result<ListBatchesResponse> {
    Ok(ListBatchesResponse::new(vec![BatchObject::new(
        "batch_1",
        "/v1/responses",
        "file_1",
    )]))
}

pub fn get_batch(_tenant_id: &str, _project_id: &str, batch_id: &str) -> Result<BatchObject> {
    Ok(BatchObject::new(batch_id, "/v1/responses", "file_1"))
}

pub fn cancel_batch(_tenant_id: &str, _project_id: &str, batch_id: &str) -> Result<BatchObject> {
    Ok(BatchObject::cancelled(batch_id, "/v1/responses", "file_1"))
}

pub fn create_vector_store(
    _tenant_id: &str,
    _project_id: &str,
    name: &str,
) -> Result<VectorStoreObject> {
    Ok(VectorStoreObject::new("vs_1", name))
}

pub fn list_vector_stores(_tenant_id: &str, _project_id: &str) -> Result<ListVectorStoresResponse> {
    Ok(ListVectorStoresResponse::new(vec![VectorStoreObject::new(
        "vs_1", "kb-main",
    )]))
}

pub fn get_vector_store(
    _tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
) -> Result<VectorStoreObject> {
    Ok(VectorStoreObject::new(vector_store_id, "kb-main"))
}

pub fn update_vector_store(
    _tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    name: &str,
) -> Result<VectorStoreObject> {
    Ok(VectorStoreObject::new(vector_store_id, name))
}

pub fn delete_vector_store(
    _tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
) -> Result<DeleteVectorStoreResponse> {
    Ok(DeleteVectorStoreResponse::deleted(vector_store_id))
}

pub fn search_vector_store(
    _tenant_id: &str,
    _project_id: &str,
    _vector_store_id: &str,
    query: &str,
) -> Result<SearchVectorStoreResponse> {
    Ok(SearchVectorStoreResponse::sample(query))
}

pub fn create_vector_store_file(
    _tenant_id: &str,
    _project_id: &str,
    _vector_store_id: &str,
    file_id: &str,
) -> Result<VectorStoreFileObject> {
    Ok(VectorStoreFileObject::new(file_id))
}

pub fn list_vector_store_files(
    _tenant_id: &str,
    _project_id: &str,
    _vector_store_id: &str,
) -> Result<ListVectorStoreFilesResponse> {
    Ok(ListVectorStoreFilesResponse::new(vec![
        VectorStoreFileObject::new("file_1"),
    ]))
}

pub fn get_vector_store_file(
    _tenant_id: &str,
    _project_id: &str,
    _vector_store_id: &str,
    file_id: &str,
) -> Result<VectorStoreFileObject> {
    Ok(VectorStoreFileObject::new(file_id))
}

pub fn delete_vector_store_file(
    _tenant_id: &str,
    _project_id: &str,
    _vector_store_id: &str,
    file_id: &str,
) -> Result<DeleteVectorStoreFileResponse> {
    Ok(DeleteVectorStoreFileResponse::deleted(file_id))
}

pub fn create_vector_store_file_batch<T: AsRef<str>>(
    _tenant_id: &str,
    _project_id: &str,
    _vector_store_id: &str,
    file_ids: &[T],
) -> Result<VectorStoreFileBatchObject> {
    let _ = file_ids.first().map(AsRef::as_ref);
    Ok(VectorStoreFileBatchObject::new("vsfb_1"))
}

pub fn get_vector_store_file_batch(
    _tenant_id: &str,
    _project_id: &str,
    _vector_store_id: &str,
    batch_id: &str,
) -> Result<VectorStoreFileBatchObject> {
    Ok(VectorStoreFileBatchObject::new(batch_id))
}

pub fn cancel_vector_store_file_batch(
    _tenant_id: &str,
    _project_id: &str,
    _vector_store_id: &str,
    batch_id: &str,
) -> Result<VectorStoreFileBatchObject> {
    Ok(VectorStoreFileBatchObject::cancelled(batch_id))
}

pub fn list_vector_store_file_batch_files(
    _tenant_id: &str,
    _project_id: &str,
    _vector_store_id: &str,
    _batch_id: &str,
) -> Result<ListVectorStoreFilesResponse> {
    Ok(ListVectorStoreFilesResponse::new(vec![
        VectorStoreFileObject::new("file_1"),
    ]))
}

pub fn builtin_extension_host() -> ExtensionHost {
    let mut host = ExtensionHost::new();
    host.register_builtin_provider(BuiltinProviderExtensionFactory::new(
        ExtensionManifest::new(
            "sdkwork.provider.openai.official",
            ExtensionKind::Provider,
            "0.1.0",
            ExtensionRuntime::Builtin,
        ),
        "openai",
        |base_url| Box::new(OpenAiProviderAdapter::new(base_url)),
    ));
    host.register_builtin_provider(BuiltinProviderExtensionFactory::new(
        ExtensionManifest::new(
            "sdkwork.provider.openrouter",
            ExtensionKind::Provider,
            "0.1.0",
            ExtensionRuntime::Builtin,
        ),
        "openrouter",
        |base_url| Box::new(OpenRouterProviderAdapter::new(base_url)),
    ));
    host.register_builtin_provider(BuiltinProviderExtensionFactory::new(
        ExtensionManifest::new(
            "sdkwork.provider.ollama",
            ExtensionKind::Provider,
            "0.1.0",
            ExtensionRuntime::Builtin,
        ),
        "ollama",
        |base_url| Box::new(OllamaProviderAdapter::new(base_url)),
    ));
    host
}

fn provider_runtime_key(provider: &ProxyProvider) -> &str {
    &provider.extension_id
}

fn configured_extension_host() -> Result<ExtensionHost> {
    let mut host = builtin_extension_host();
    let policy = configured_extension_discovery_policy();

    for package in discover_extension_packages(&policy)? {
        let trust = verify_discovered_extension_package_trust(&package, &policy);
        if !trust.load_allowed {
            continue;
        }
        register_discovered_extension(&mut host, package);
    }

    Ok(host)
}

fn configured_extension_discovery_policy() -> ExtensionDiscoveryPolicy {
    let search_paths = std::env::var_os("SDKWORK_EXTENSION_PATHS")
        .map(|value| std::env::split_paths(&value).collect::<Vec<_>>())
        .unwrap_or_default();

    let mut policy = ExtensionDiscoveryPolicy::new(search_paths)
        .with_connector_extensions(env_flag(
            "SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS",
            true,
        ))
        .with_native_dynamic_extensions(env_flag(
            "SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS",
            false,
        ))
        .with_required_signatures_for_connector_extensions(env_flag(
            "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS",
            false,
        ))
        .with_required_signatures_for_native_dynamic_extensions(env_flag(
            "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS",
            true,
        ));
    for (publisher, public_key) in env_trusted_signers("SDKWORK_EXTENSION_TRUSTED_SIGNERS") {
        policy = policy.with_trusted_signer(publisher, public_key);
    }
    policy
}

fn env_flag(key: &str, default: bool) -> bool {
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse::<bool>().ok())
        .unwrap_or(default)
}

fn env_trusted_signers(key: &str) -> Vec<(String, String)> {
    std::env::var(key)
        .ok()
        .map(|value| parse_trusted_signers(&value))
        .unwrap_or_default()
}

fn parse_trusted_signers(value: &str) -> Vec<(String, String)> {
    value
        .split(';')
        .filter_map(|entry| {
            let entry = entry.trim();
            if entry.is_empty() {
                return None;
            }
            let (publisher, public_key) = entry.split_once('=')?;
            let publisher = publisher.trim();
            let public_key = public_key.trim();
            if publisher.is_empty() || public_key.is_empty() {
                return None;
            }
            Some((publisher.to_owned(), public_key.to_owned()))
        })
        .collect()
}

fn register_discovered_extension(host: &mut ExtensionHost, package: DiscoveredExtensionPackage) {
    if host.manifest(&package.manifest.id).is_some() {
        return;
    }

    if package.manifest.runtime == ExtensionRuntime::NativeDynamic {
        let _ = host.register_discovered_native_dynamic_provider(package);
        return;
    }

    match (
        package.manifest.kind.clone(),
        package.manifest.protocol.clone(),
    ) {
        (ExtensionKind::Provider, Some(ExtensionProtocol::OpenAi)) => {
            host.register_discovered_provider(package, "openai", |base_url| {
                Box::new(OpenAiProviderAdapter::new(base_url))
            });
        }
        (ExtensionKind::Provider, Some(ExtensionProtocol::OpenRouter)) => {
            host.register_discovered_provider(package, "openrouter", |base_url| {
                Box::new(OpenRouterProviderAdapter::new(base_url))
            });
        }
        (ExtensionKind::Provider, Some(ExtensionProtocol::Ollama)) => {
            host.register_discovered_provider(package, "ollama", |base_url| {
                Box::new(OllamaProviderAdapter::new(base_url))
            });
        }
        _ => host.register_discovered_manifest(package),
    }
}

async fn execute_json_provider_request_for_provider(
    store: &dyn AdminStore,
    provider: &ProxyProvider,
    api_key: &str,
    request: ProviderRequest<'_>,
) -> Result<Option<Value>> {
    let descriptor =
        provider_execution_descriptor_for_provider(store, provider, api_key.to_owned()).await?;
    debug_assert!(descriptor.local_fallback || descriptor.provider_id == provider.id);
    if descriptor.local_fallback {
        return Ok(None);
    }

    execute_json_provider_request(
        &descriptor.runtime_key,
        descriptor.base_url,
        &descriptor.api_key,
        request,
    )
    .await
}

async fn execute_stream_provider_request_for_provider(
    store: &dyn AdminStore,
    provider: &ProxyProvider,
    api_key: &str,
    request: ProviderRequest<'_>,
) -> Result<Option<ProviderStreamOutput>> {
    let descriptor =
        provider_execution_descriptor_for_provider(store, provider, api_key.to_owned()).await?;
    debug_assert!(descriptor.local_fallback || descriptor.provider_id == provider.id);
    if descriptor.local_fallback {
        return Ok(None);
    }

    execute_stream_provider_request(
        &descriptor.runtime_key,
        descriptor.base_url,
        &descriptor.api_key,
        request,
    )
    .await
}

async fn execute_json_provider_request(
    runtime_key: &str,
    base_url: String,
    api_key: &str,
    request: ProviderRequest<'_>,
) -> Result<Option<Value>> {
    let host = configured_extension_host()?;
    let Some(adapter) = host.resolve_provider(runtime_key, base_url) else {
        return Ok(None);
    };

    let response = adapter.execute(api_key, request).await?;
    Ok(response.into_json())
}

async fn execute_stream_provider_request(
    runtime_key: &str,
    base_url: String,
    api_key: &str,
    request: ProviderRequest<'_>,
) -> Result<Option<ProviderStreamOutput>> {
    let host = configured_extension_host()?;
    let Some(adapter) = host.resolve_provider(runtime_key, base_url) else {
        return Ok(None);
    };

    let response = adapter.execute(api_key, request).await?;
    Ok(response.into_stream())
}

pub async fn execute_json_provider_request_with_runtime(
    runtime_key: &str,
    base_url: impl Into<String>,
    api_key: &str,
    request: ProviderRequest<'_>,
) -> Result<Option<Value>> {
    execute_json_provider_request(runtime_key, base_url.into(), api_key, request).await
}

pub async fn execute_stream_provider_request_with_runtime(
    runtime_key: &str,
    base_url: impl Into<String>,
    api_key: &str,
    request: ProviderRequest<'_>,
) -> Result<Option<ProviderStreamOutput>> {
    execute_stream_provider_request(runtime_key, base_url.into(), api_key, request).await
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
