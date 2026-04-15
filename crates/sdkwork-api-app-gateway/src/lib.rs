use anyhow::{bail, Context, Result};
use sdkwork_api_app_credential::{
    official_provider_secret_configured, resolve_credential_secret_with_manager,
    resolve_official_provider_secret_with_manager,
    resolve_provider_secret_with_fallback_and_manager,
    resolve_provider_secret_with_fallback_and_manager as resolve_provider_secret_with_manager,
    CredentialSecretManager,
};
use sdkwork_api_app_routing::{
    select_route_with_store_context, simulate_route_with_store_selection_context,
    RouteSelectionContext,
};
use sdkwork_api_cache_core::{
    cache_get_or_insert_with, CacheStore, CacheTag, DistributedLockStore,
};
use sdkwork_api_cache_memory::MemoryCacheStore;
use sdkwork_api_contract_openai::assistants::{
    AssistantObject, CreateAssistantRequest, DeleteAssistantResponse, ListAssistantsResponse,
    UpdateAssistantRequest,
};
use sdkwork_api_contract_openai::audio::{
    CreateSpeechRequest, CreateTranscriptionRequest, CreateTranslationRequest,
    CreateVoiceConsentRequest, ListVoicesResponse, SpeechResponse, TranscriptionObject,
    TranslationObject, VoiceConsentObject,
};
use sdkwork_api_contract_openai::batches::{BatchObject, CreateBatchRequest, ListBatchesResponse};
use sdkwork_api_contract_openai::chat_completions::{
    ChatCompletionResponse, CreateChatCompletionRequest, DeleteChatCompletionResponse,
    ListChatCompletionMessagesResponse, ListChatCompletionsResponse, UpdateChatCompletionRequest,
};
use sdkwork_api_contract_openai::completions::{CompletionObject, CreateCompletionRequest};
use sdkwork_api_contract_openai::containers::{
    ContainerFileObject, ContainerObject, CreateContainerFileRequest, CreateContainerRequest,
    DeleteContainerFileResponse, DeleteContainerResponse, ListContainerFilesResponse,
    ListContainersResponse,
};
use sdkwork_api_contract_openai::conversations::{
    ConversationItemObject, ConversationObject, CreateConversationItemsRequest,
    CreateConversationRequest, DeleteConversationItemResponse, DeleteConversationResponse,
    ListConversationItemsResponse, ListConversationsResponse, UpdateConversationRequest,
};
use sdkwork_api_contract_openai::embeddings::CreateEmbeddingRequest;
use sdkwork_api_contract_openai::embeddings::CreateEmbeddingResponse;
use sdkwork_api_contract_openai::evals::{
    CreateEvalRequest, CreateEvalRunRequest, DeleteEvalResponse, DeleteEvalRunResponse, EvalObject,
    EvalRunObject, EvalRunOutputItemObject, ListEvalRunOutputItemsResponse, ListEvalRunsResponse,
    ListEvalsResponse, UpdateEvalRequest,
};
use sdkwork_api_contract_openai::files::{
    CreateFileRequest, DeleteFileResponse, FileObject, ListFilesResponse,
};
use sdkwork_api_contract_openai::fine_tuning::{
    CreateFineTuningCheckpointPermissionsRequest, CreateFineTuningJobRequest,
    DeleteFineTuningCheckpointPermissionResponse, FineTuningJobObject,
    ListFineTuningCheckpointPermissionsResponse, ListFineTuningJobCheckpointsResponse,
    ListFineTuningJobEventsResponse, ListFineTuningJobsResponse,
};
use sdkwork_api_contract_openai::images::{
    CreateImageEditRequest, CreateImageRequest, CreateImageVariationRequest, ImagesResponse,
};
use sdkwork_api_contract_openai::models::{DeleteModelResponse, ListModelsResponse, ModelObject};
use sdkwork_api_contract_openai::moderations::{CreateModerationRequest, ModerationResponse};
use sdkwork_api_contract_openai::music::{
    CreateMusicLyricsRequest, CreateMusicRequest, DeleteMusicResponse, MusicLyricsObject,
    MusicObject, MusicTracksResponse,
};
use sdkwork_api_contract_openai::realtime::{CreateRealtimeSessionRequest, RealtimeSessionObject};
use sdkwork_api_contract_openai::responses::{
    CompactResponseRequest, CountResponseInputTokensRequest, CreateResponseRequest,
    DeleteResponseResponse, ListResponseInputItemsResponse, ResponseCompactionObject,
    ResponseInputTokensObject, ResponseObject,
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
    CreateVideoCharacterRequest, CreateVideoRequest, DeleteVideoResponse, EditVideoRequest,
    ExtendVideoRequest, RemixVideoRequest, UpdateVideoCharacterRequest, VideoCharacterObject,
    VideoCharactersResponse, VideoObject, VideosResponse,
};
use sdkwork_api_contract_openai::webhooks::{
    CreateWebhookRequest, DeleteWebhookResponse, ListWebhooksResponse, UpdateWebhookRequest,
    WebhookObject,
};
use sdkwork_api_domain_catalog::ProxyProvider;
use sdkwork_api_domain_routing::{
    ProviderHealthSnapshot, RoutingDecision, RoutingDecisionLog, RoutingDecisionSource,
    RoutingPolicy,
};
use sdkwork_api_extension_core::{
    ExtensionKind, ExtensionManifest, ExtensionModality, ExtensionProtocol, ExtensionRuntime,
};
use sdkwork_api_extension_host::{
    discover_extension_packages, ensure_connector_runtime_started, shutdown_all_connector_runtimes,
    shutdown_all_native_dynamic_runtimes, shutdown_connector_runtime,
    shutdown_connector_runtimes_for_extension, shutdown_native_dynamic_runtimes_for_extension,
    verify_discovered_extension_package_trust, BuiltinProviderExtensionFactory,
    DiscoveredExtensionPackage, ExtensionDiscoveryPolicy, ExtensionHost, ExtensionHostError,
};
use sdkwork_api_observability::HttpMetricsRegistry;
use sdkwork_api_provider_core::{
    ProviderExecutionAdapter, ProviderHttpError, ProviderOutput, ProviderRequest,
    ProviderRequestOptions, ProviderRetryAfterSource, ProviderStreamOutput,
};
use sdkwork_api_provider_ollama::OllamaProviderAdapter;
use sdkwork_api_provider_openai::OpenAiProviderAdapter;
use sdkwork_api_provider_openrouter::OpenRouterProviderAdapter;
use sdkwork_api_storage_core::{AdminStore, Reloadable};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::future::Future;
use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, UNIX_EPOCH};
use tokio::task::JoinHandle;
use tokio::time::MissedTickBehavior;

pub const LOCAL_PROVIDER_ID: &str = "sdkwork.local";

pub(crate) fn local_object_id_matches(id: &str, prefix: &str) -> bool {
    id.starts_with(&format!("{prefix}_local_"))
}

tokio::task_local! {
    static REQUEST_ROUTING_REGION: Option<String>;
}

tokio::task_local! {
    static REQUEST_API_KEY_GROUP_ID: Option<String>;
}

mod gateway_cache;
mod gateway_execution_context;
mod gateway_extension_host;
mod gateway_provider_resolution;
mod gateway_routing;
mod gateway_runtime_execution;
mod gateway_types;
mod model_catalog;
mod relay_assistants_realtime_webhooks;
mod relay_chat;
mod relay_compute;
mod relay_containers;
mod relay_conversations;
mod relay_evals_batches;
mod relay_files_uploads;
mod relay_fine_tuning;
mod relay_music_video;
mod relay_responses;
mod relay_threads;
mod relay_vector_stores;
mod request_context;

#[cfg(test)]
mod tests;

pub(crate) use gateway_cache::{
    cache_routing_decision, capability_catalog_cache_store, capability_catalog_list_cache_key,
    capability_catalog_model_cache_key, gateway_provider_in_flight_counter,
    gateway_provider_max_in_flight_limit, routing_recovery_probe_lock_store,
    take_cached_routing_decision, CachedCapabilityCatalogList, CachedCapabilityCatalogModel,
    CAPABILITY_CATALOG_CACHE_TAG_ALL_MODELS, CAPABILITY_CATALOG_CACHE_TTL_MS,
};
pub(crate) use gateway_execution_context::*;
pub(crate) use gateway_extension_host::{
    configured_extension_host, preferred_provider_runtime_key,
};
pub(crate) use gateway_provider_resolution::{
    build_extension_host_from_store, provider_execution_descriptor_for_provider,
    provider_execution_descriptor_for_provider_account_context,
    provider_execution_target_for_provider, resolve_non_model_provider,
    ProviderExecutionDescriptor, ProviderExecutionTarget,
};
pub(crate) use gateway_routing::{
    gateway_execution_failover_fallback_reason, gateway_execution_observed_at_ms,
    gateway_usage_context_for_decision_provider, persist_gateway_execution_failover_decision_log,
    persist_gateway_execution_health_snapshot, resolve_store_relay_provider_for_decision,
    select_gateway_route,
};
pub(crate) use gateway_runtime_execution::{
    execute_json_provider_request, execute_json_provider_request_for_descriptor_with_options,
    execute_json_provider_request_for_provider,
    execute_stream_provider_request_for_descriptor_with_options,
    execute_stream_provider_request_for_provider, normalize_local_speech_format,
    provider_execution_descriptor_from_planned_context, record_gateway_execution_failover,
    record_gateway_provider_health, record_gateway_provider_health_persist_failure,
    record_gateway_recovery_probe_from_decision,
};

pub use gateway_provider_resolution::{
    inspect_provider_execution_views, ProviderExecutionView, ProviderRouteExecutionMode,
    ProviderRouteExecutionView, ProviderRouteReadinessView,
};
pub use gateway_types::*;
pub use model_catalog::*;
pub use relay_assistants_realtime_webhooks::*;
pub use relay_chat::*;
pub use relay_compute::*;
pub use relay_containers::*;
pub use relay_conversations::*;
pub use relay_evals_batches::*;
pub use relay_files_uploads::{
    cancel_upload, complete_upload, create_file, create_upload, create_upload_part, delete_file,
    file_content, get_file, list_files, relay_cancel_upload_from_store,
    relay_complete_upload_from_store, relay_delete_file_from_store, relay_file_content_from_store,
    relay_file_from_store, relay_get_file_from_store, relay_list_files_from_store,
    relay_upload_from_store, relay_upload_part_from_store,
};
pub use relay_fine_tuning::*;
pub use relay_music_video::*;
pub use relay_responses::*;
pub use relay_threads::*;
pub use relay_vector_stores::*;

pub type ConfiguredExtensionHostReloadReport =
    gateway_extension_host::ConfiguredExtensionHostReloadReport;
pub type ConfiguredExtensionHostReloadScope =
    gateway_extension_host::ConfiguredExtensionHostReloadScope;

pub const ROUTING_DECISION_CACHE_NAMESPACE: &str = gateway_cache::ROUTING_DECISION_CACHE_NAMESPACE;
pub const CAPABILITY_CATALOG_CACHE_NAMESPACE: &str =
    gateway_cache::CAPABILITY_CATALOG_CACHE_NAMESPACE;

pub fn configure_route_decision_cache_store(cache_store: Arc<dyn CacheStore>) {
    gateway_cache::configure_route_decision_cache_store(cache_store);
}

pub fn configure_route_recovery_probe_lock_store(lock_store: Arc<dyn DistributedLockStore>) {
    gateway_cache::configure_route_recovery_probe_lock_store(lock_store);
}

pub fn configure_gateway_provider_max_in_flight_limit(limit: Option<usize>) {
    gateway_cache::configure_gateway_provider_max_in_flight_limit(limit);
}

pub fn configure_capability_catalog_cache_store(cache_store: Arc<dyn CacheStore>) {
    gateway_cache::configure_capability_catalog_cache_store(cache_store);
}

pub fn clear_capability_catalog_cache_store() {
    gateway_cache::clear_capability_catalog_cache_store();
}

pub async fn invalidate_capability_catalog_cache() {
    gateway_cache::invalidate_capability_catalog_cache().await;
}

pub async fn planned_execution_provider_id_for_route(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    route_key: &str,
) -> Result<String> {
    gateway_routing::planned_execution_provider_id_for_route(
        store, tenant_id, project_id, capability, route_key,
    )
    .await
}

pub async fn planned_execution_usage_context_for_route(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    route_key: &str,
) -> Result<PlannedExecutionUsageContext> {
    gateway_routing::planned_execution_usage_context_for_route(
        store, tenant_id, project_id, capability, route_key,
    )
    .await
}

pub async fn planned_execution_provider_context_for_route_without_log(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    route_key: &str,
) -> Result<Option<PlannedExecutionProviderContext>> {
    gateway_routing::planned_execution_provider_context_for_route_without_log(
        store,
        secret_manager,
        tenant_id,
        project_id,
        capability,
        route_key,
    )
    .await
}

pub async fn planned_execution_provider_context_for_route_without_log_with_selection_seed(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    route_key: &str,
    selection_seed: Option<u64>,
) -> Result<Option<PlannedExecutionProviderContext>> {
    gateway_routing::planned_execution_provider_context_for_route_without_log_with_selection_seed(
        store,
        secret_manager,
        tenant_id,
        project_id,
        capability,
        route_key,
        selection_seed,
    )
    .await
}

pub async fn persist_planned_execution_decision_log(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    route_key: &str,
    decision: &RoutingDecision,
) -> Result<()> {
    gateway_routing::persist_planned_execution_decision_log(
        store, tenant_id, project_id, capability, route_key, decision,
    )
    .await
}

pub async fn with_request_routing_region<T, F>(requested_region: Option<String>, future: F) -> T
where
    F: Future<Output = T>,
{
    request_context::with_request_routing_region(requested_region, future).await
}

pub async fn with_request_api_key_group_id<T, F>(api_key_group_id: Option<String>, future: F) -> T
where
    F: Future<Output = T>,
{
    request_context::with_request_api_key_group_id(api_key_group_id, future).await
}

pub fn current_request_routing_region() -> Option<String> {
    request_context::current_request_routing_region()
}

pub fn current_request_api_key_group_id() -> Option<String> {
    request_context::current_request_api_key_group_id()
}

pub fn builtin_extension_host() -> ExtensionHost {
    gateway_extension_host::builtin_extension_host()
}

pub fn reload_configured_extension_host() -> Result<ConfiguredExtensionHostReloadReport> {
    gateway_extension_host::reload_configured_extension_host()
}

pub fn reload_extension_host_with_scope(
    scope: &ConfiguredExtensionHostReloadScope,
) -> Result<ConfiguredExtensionHostReloadReport> {
    gateway_extension_host::reload_extension_host_with_scope(scope)
}

pub fn reload_extension_host_with_policy(
    policy: &ExtensionDiscoveryPolicy,
) -> Result<ConfiguredExtensionHostReloadReport> {
    gateway_extension_host::reload_extension_host_with_policy(policy)
}

pub fn start_configured_extension_hot_reload_supervision(
    interval_secs: u64,
) -> Option<JoinHandle<()>> {
    gateway_extension_host::start_configured_extension_hot_reload_supervision(interval_secs)
}

pub async fn execute_json_provider_request_with_runtime(
    runtime_key: &str,
    base_url: impl Into<String>,
    api_key: &str,
    request: ProviderRequest<'_>,
) -> Result<Option<Value>> {
    gateway_runtime_execution::execute_json_provider_request_with_runtime(
        runtime_key,
        base_url,
        api_key,
        request,
    )
    .await
}

pub async fn execute_json_provider_request_with_runtime_and_options(
    runtime_key: &str,
    base_url: impl Into<String>,
    api_key: &str,
    request: ProviderRequest<'_>,
    options: &ProviderRequestOptions,
) -> Result<Option<Value>> {
    gateway_runtime_execution::execute_json_provider_request_with_runtime_and_options(
        runtime_key,
        base_url,
        api_key,
        request,
        options,
    )
    .await
}

pub async fn execute_stream_provider_request_with_runtime(
    runtime_key: &str,
    base_url: impl Into<String>,
    api_key: &str,
    request: ProviderRequest<'_>,
) -> Result<Option<ProviderStreamOutput>> {
    gateway_runtime_execution::execute_stream_provider_request_with_runtime(
        runtime_key,
        base_url,
        api_key,
        request,
    )
    .await
}

pub async fn execute_stream_provider_request_with_runtime_and_options(
    runtime_key: &str,
    base_url: impl Into<String>,
    api_key: &str,
    request: ProviderRequest<'_>,
    options: &ProviderRequestOptions,
) -> Result<Option<ProviderStreamOutput>> {
    gateway_runtime_execution::execute_stream_provider_request_with_runtime_and_options(
        runtime_key,
        base_url,
        api_key,
        request,
        options,
    )
    .await
}

pub fn execute_raw_json_provider_operation_with_runtime(
    runtime_key: &str,
    base_url: impl Into<String>,
    api_key: &str,
    operation: &str,
    path_params: Vec<String>,
    body: Value,
    headers: HashMap<String, String>,
) -> Result<Option<Value>> {
    gateway_runtime_execution::execute_raw_json_provider_operation_with_runtime(
        runtime_key,
        base_url,
        api_key,
        operation,
        path_params,
        body,
        headers,
    )
}

pub async fn execute_raw_stream_provider_operation_with_runtime(
    runtime_key: &str,
    base_url: impl Into<String>,
    api_key: &str,
    operation: &str,
    path_params: Vec<String>,
    body: Value,
    headers: HashMap<String, String>,
) -> Result<Option<ProviderStreamOutput>> {
    gateway_runtime_execution::execute_raw_stream_provider_operation_with_runtime(
        runtime_key,
        base_url,
        api_key,
        operation,
        path_params,
        body,
        headers,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn execute_raw_json_provider_operation_from_planned_execution_context_with_options(
    store: &dyn AdminStore,
    planned: &PlannedExecutionProviderContext,
    capability: &str,
    operation: &str,
    path_params: Vec<String>,
    body: Value,
    headers: HashMap<String, String>,
    options: &ProviderRequestOptions,
) -> Result<Option<Value>> {
    gateway_runtime_execution::execute_raw_json_provider_operation_from_planned_execution_context_with_options(
        store,
        planned,
        capability,
        operation,
        path_params,
        body,
        headers,
        options,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn execute_raw_stream_provider_operation_from_planned_execution_context_with_options(
    store: &dyn AdminStore,
    planned: &PlannedExecutionProviderContext,
    capability: &str,
    operation: &str,
    path_params: Vec<String>,
    body: Value,
    headers: HashMap<String, String>,
    options: &ProviderRequestOptions,
) -> Result<Option<ProviderStreamOutput>> {
    gateway_runtime_execution::execute_raw_stream_provider_operation_from_planned_execution_context_with_options(
        store,
        planned,
        capability,
        operation,
        path_params,
        body,
        headers,
        options,
    )
    .await
}
