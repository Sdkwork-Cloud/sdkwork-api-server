use anyhow::Result;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use sdkwork_api_app_credential::{resolve_provider_secret_with_manager, CredentialSecretManager};
use sdkwork_api_app_routing::simulate_route_with_store;
use sdkwork_api_contract_openai::assistants::{AssistantObject, CreateAssistantRequest};
use sdkwork_api_contract_openai::audio::{
    CreateSpeechRequest, CreateTranscriptionRequest, CreateTranslationRequest, SpeechResponse,
    TranscriptionObject, TranslationObject,
};
use sdkwork_api_contract_openai::batches::{BatchObject, CreateBatchRequest, ListBatchesResponse};
use sdkwork_api_contract_openai::chat_completions::ChatCompletionResponse;
use sdkwork_api_contract_openai::chat_completions::CreateChatCompletionRequest;
use sdkwork_api_contract_openai::completions::{CompletionObject, CreateCompletionRequest};
use sdkwork_api_contract_openai::embeddings::CreateEmbeddingRequest;
use sdkwork_api_contract_openai::embeddings::CreateEmbeddingResponse;
use sdkwork_api_contract_openai::evals::{CreateEvalRequest, EvalObject};
use sdkwork_api_contract_openai::files::{
    CreateFileRequest, DeleteFileResponse, FileObject, ListFilesResponse,
};
use sdkwork_api_contract_openai::fine_tuning::{
    CreateFineTuningJobRequest, FineTuningJobObject, ListFineTuningJobsResponse,
};
use sdkwork_api_contract_openai::images::{CreateImageRequest, ImageObject, ImagesResponse};
use sdkwork_api_contract_openai::models::{ListModelsResponse, ModelObject};
use sdkwork_api_contract_openai::moderations::{
    CreateModerationRequest, ModerationCategoryScores, ModerationResponse, ModerationResult,
};
use sdkwork_api_contract_openai::realtime::{CreateRealtimeSessionRequest, RealtimeSessionObject};
use sdkwork_api_contract_openai::responses::CreateResponseRequest;
use sdkwork_api_contract_openai::responses::ResponseObject;
use sdkwork_api_contract_openai::uploads::{
    AddUploadPartRequest, CompleteUploadRequest, CreateUploadRequest, UploadObject,
    UploadPartObject,
};
use sdkwork_api_contract_openai::vector_stores::{
    CreateVectorStoreRequest, DeleteVectorStoreResponse, ListVectorStoresResponse,
    UpdateVectorStoreRequest, VectorStoreObject,
};
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

pub async fn relay_file_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateFileRequest,
) -> Result<Option<Value>> {
    let decision = simulate_route_with_store(store, "files", &request.purpose).await?;
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
    let decision = simulate_route_with_store(store, "files", "files").await?;
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
    let decision = simulate_route_with_store(store, "files", file_id).await?;
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
    let decision = simulate_route_with_store(store, "files", file_id).await?;
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
) -> Result<Option<reqwest::Response>> {
    let decision = simulate_route_with_store(store, "files", file_id).await?;
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
    let decision = simulate_route_with_store(store, "uploads", &request.purpose).await?;
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
    let decision = simulate_route_with_store(store, "uploads", &request.upload_id).await?;
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
    let decision = simulate_route_with_store(store, "uploads", &request.upload_id).await?;
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
    let decision = simulate_route_with_store(store, "uploads", upload_id).await?;
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

pub async fn relay_list_fine_tuning_jobs_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
) -> Result<Option<Value>> {
    let decision = simulate_route_with_store(store, "fine_tuning", "jobs").await?;
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
    let decision = simulate_route_with_store(store, "fine_tuning", job_id).await?;
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
    let decision = simulate_route_with_store(store, "fine_tuning", job_id).await?;
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

pub async fn relay_list_batches_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
) -> Result<Option<Value>> {
    let decision = simulate_route_with_store(store, "batches", "batches").await?;
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
    let decision = simulate_route_with_store(store, "batches", batch_id).await?;
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
    let decision = simulate_route_with_store(store, "batches", batch_id).await?;
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

pub async fn relay_list_vector_stores_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
) -> Result<Option<Value>> {
    let decision = simulate_route_with_store(store, "vector_stores", "vector_stores").await?;
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
    let decision = simulate_route_with_store(store, "vector_stores", vector_store_id).await?;
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
    let decision = simulate_route_with_store(store, "vector_stores", vector_store_id).await?;
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
    let decision = simulate_route_with_store(store, "vector_stores", vector_store_id).await?;
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
        ProviderRequest::VectorStoresDelete(vector_store_id),
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
