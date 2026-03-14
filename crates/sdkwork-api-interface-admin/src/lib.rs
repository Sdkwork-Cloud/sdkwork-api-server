use std::sync::Arc;

use axum::{
    extract::FromRequestParts,
    extract::State,
    http::header,
    http::request::Parts,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use sdkwork_api_app_billing::{
    create_quota_policy, list_ledger_entries, list_quota_policies, persist_quota_policy,
    summarize_billing_from_store,
};
use sdkwork_api_app_catalog::{
    list_channels, list_model_entries, list_providers, persist_channel,
    persist_model_with_metadata, persist_provider_with_bindings_and_extension_id,
};
use sdkwork_api_app_credential::CredentialSecretManager;
use sdkwork_api_app_credential::{list_credentials, persist_credential_with_secret_and_manager};
use sdkwork_api_app_extension::{
    configured_extension_discovery_policy_from_env, list_discovered_extension_packages,
    list_extension_installations, list_extension_instances, list_extension_runtime_statuses,
    list_provider_health_snapshots, persist_extension_installation, persist_extension_instance,
    ExtensionDiscoveryPolicy, PersistExtensionInstanceInput,
};
use sdkwork_api_app_identity::{
    issue_jwt, list_gateway_api_keys, persist_gateway_api_key, verify_jwt, Claims,
    CreatedGatewayApiKey,
};
use sdkwork_api_app_routing::{
    create_routing_policy, list_routing_decision_logs, list_routing_policies,
    persist_routing_policy, select_route_with_store, CreateRoutingPolicyInput,
};
use sdkwork_api_app_tenant::{list_projects, list_tenants, persist_project, persist_tenant};
use sdkwork_api_app_usage::list_usage_records;
use sdkwork_api_app_usage::summarize_usage_records_from_store;
use sdkwork_api_domain_billing::{BillingSummary, LedgerEntry, QuotaPolicy};
use sdkwork_api_domain_catalog::{
    Channel, ModelCapability, ModelCatalogEntry, ProviderChannelBinding, ProxyProvider,
};
use sdkwork_api_domain_credential::UpstreamCredential;
use sdkwork_api_domain_identity::GatewayApiKeyRecord;
use sdkwork_api_domain_routing::{
    ProviderHealthSnapshot, RoutingDecisionLog, RoutingDecisionSource, RoutingPolicy,
    RoutingStrategy,
};
use sdkwork_api_domain_tenant::{Project, Tenant};
use sdkwork_api_domain_usage::{UsageRecord, UsageSummary};
use sdkwork_api_extension_core::{ExtensionInstallation, ExtensionInstance, ExtensionRuntime};
use sdkwork_api_storage_core::AdminStore;
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

const DEFAULT_ADMIN_JWT_SIGNING_SECRET: &str = "local-dev-admin-jwt-secret";

#[derive(Clone)]
pub struct AdminApiState {
    store: Arc<dyn AdminStore>,
    secret_manager: CredentialSecretManager,
    jwt_signing_secret: String,
    extension_discovery_policy: ExtensionDiscoveryPolicy,
}

impl AdminApiState {
    pub fn new(pool: SqlitePool) -> Self {
        Self::with_master_key(pool, "local-dev-master-key")
    }

    pub fn with_master_key(pool: SqlitePool, credential_master_key: impl Into<String>) -> Self {
        Self::with_store_and_secret_manager(
            Arc::new(SqliteAdminStore::new(pool)),
            CredentialSecretManager::database_encrypted(credential_master_key),
        )
    }

    pub fn with_secret_manager(pool: SqlitePool, secret_manager: CredentialSecretManager) -> Self {
        Self::with_secret_manager_and_jwt_secret(
            pool,
            secret_manager,
            DEFAULT_ADMIN_JWT_SIGNING_SECRET,
        )
    }

    pub fn with_secret_manager_and_jwt_secret(
        pool: SqlitePool,
        secret_manager: CredentialSecretManager,
        jwt_signing_secret: impl Into<String>,
    ) -> Self {
        Self {
            store: Arc::new(SqliteAdminStore::new(pool)),
            secret_manager,
            jwt_signing_secret: jwt_signing_secret.into(),
            extension_discovery_policy: configured_extension_discovery_policy_from_env(),
        }
    }

    pub fn with_store_and_secret_manager(
        store: Arc<dyn AdminStore>,
        secret_manager: CredentialSecretManager,
    ) -> Self {
        Self::with_store_and_secret_manager_and_jwt_secret(
            store,
            secret_manager,
            DEFAULT_ADMIN_JWT_SIGNING_SECRET,
        )
    }

    pub fn with_store_and_secret_manager_and_jwt_secret(
        store: Arc<dyn AdminStore>,
        secret_manager: CredentialSecretManager,
        jwt_signing_secret: impl Into<String>,
    ) -> Self {
        Self {
            store,
            secret_manager,
            jwt_signing_secret: jwt_signing_secret.into(),
            extension_discovery_policy: configured_extension_discovery_policy_from_env(),
        }
    }
}

#[derive(Clone, Debug)]
struct AuthenticatedAdminClaims(Claims);

impl AuthenticatedAdminClaims {
    fn claims(&self) -> &Claims {
        &self.0
    }
}

impl FromRequestParts<AdminApiState> for AuthenticatedAdminClaims {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AdminApiState,
    ) -> Result<Self, Self::Rejection> {
        let Some(header_value) = parts.headers.get(header::AUTHORIZATION) else {
            return Err(StatusCode::UNAUTHORIZED);
        };
        let Ok(header_value) = header_value.to_str() else {
            return Err(StatusCode::UNAUTHORIZED);
        };
        let Some(token) = header_value
            .strip_prefix("Bearer ")
            .or_else(|| header_value.strip_prefix("bearer "))
        else {
            return Err(StatusCode::UNAUTHORIZED);
        };

        verify_jwt(token, &state.jwt_signing_secret)
            .map(Self)
            .map_err(|_| StatusCode::UNAUTHORIZED)
    }
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    subject: String,
}

#[derive(Debug, Serialize)]
struct LoginResponse {
    token: String,
    claims: Claims,
}

#[derive(Debug, Deserialize)]
struct CreateChannelRequest {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct CreateProviderRequest {
    id: String,
    channel_id: String,
    #[serde(default)]
    extension_id: Option<String>,
    #[serde(default)]
    channel_bindings: Vec<CreateProviderChannelBindingRequest>,
    adapter_kind: String,
    base_url: String,
    display_name: String,
}

#[derive(Debug, Deserialize)]
struct CreateProviderChannelBindingRequest {
    channel_id: String,
    #[serde(default)]
    is_primary: bool,
}

#[derive(Debug, Deserialize)]
struct CreateCredentialRequest {
    tenant_id: String,
    provider_id: String,
    key_reference: String,
    secret_value: String,
}

#[derive(Debug, Deserialize)]
struct CreateModelRequest {
    external_name: String,
    provider_id: String,
    #[serde(default)]
    capabilities: Vec<ModelCapability>,
    #[serde(default)]
    streaming: bool,
    context_window: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct CreateTenantRequest {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct CreateProjectRequest {
    tenant_id: String,
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct CreateApiKeyRequest {
    tenant_id: String,
    project_id: String,
    environment: String,
}

#[derive(Debug, Deserialize)]
struct CreateExtensionInstallationRequest {
    installation_id: String,
    extension_id: String,
    runtime: ExtensionRuntime,
    enabled: bool,
    entrypoint: Option<String>,
    #[serde(default)]
    config: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct CreateExtensionInstanceRequest {
    instance_id: String,
    installation_id: String,
    extension_id: String,
    enabled: bool,
    base_url: Option<String>,
    credential_ref: Option<String>,
    #[serde(default)]
    config: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct CreateRoutingPolicyRequest {
    policy_id: String,
    capability: String,
    model_pattern: String,
    #[serde(default = "default_true")]
    enabled: bool,
    #[serde(default)]
    priority: i32,
    #[serde(default)]
    strategy: Option<RoutingStrategy>,
    #[serde(default)]
    ordered_provider_ids: Vec<String>,
    #[serde(default)]
    default_provider_id: Option<String>,
    #[serde(default)]
    max_cost: Option<f64>,
    #[serde(default)]
    max_latency_ms: Option<u64>,
    #[serde(default)]
    require_healthy: bool,
}

#[derive(Debug, Deserialize)]
struct CreateQuotaPolicyRequest {
    policy_id: String,
    project_id: String,
    max_units: u64,
    #[serde(default = "default_true")]
    enabled: bool,
}

#[derive(Debug, Deserialize)]
struct RoutingSimulationRequest {
    capability: String,
    model: String,
    #[serde(default)]
    selection_seed: Option<u64>,
}

#[derive(Debug, Serialize)]
struct RoutingSimulationResponse {
    selected_provider_id: String,
    candidate_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    matched_policy_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    strategy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    selection_seed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    selection_reason: Option<String>,
    #[serde(default)]
    slo_applied: bool,
    #[serde(default)]
    slo_degraded: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    assessments: Vec<sdkwork_api_domain_routing::RoutingCandidateAssessment>,
}

pub fn admin_router() -> Router {
    Router::new()
        .route("/admin/health", get(|| async { "ok" }))
        .route("/admin/auth/login", post(|| async { "login" }))
        .route("/admin/auth/me", get(|| async { "me" }))
        .route("/admin/tenants", get(|| async { "tenants" }))
        .route("/admin/projects", get(|| async { "projects" }))
        .route("/admin/api-keys", get(|| async { "api-keys" }))
        .route("/admin/channels", get(|| async { "channels" }))
        .route("/admin/providers", get(|| async { "providers" }))
        .route("/admin/credentials", get(|| async { "credentials" }))
        .route("/admin/models", get(|| async { "models" }))
        .route(
            "/admin/extensions/installations",
            get(|| async { "extension-installations" }),
        )
        .route(
            "/admin/extensions/packages",
            get(|| async { "extension-packages" }),
        )
        .route(
            "/admin/extensions/instances",
            get(|| async { "extension-instances" }),
        )
        .route(
            "/admin/extensions/runtime-statuses",
            get(|| async { "extension-runtime-statuses" }),
        )
        .route("/admin/usage/records", get(|| async { "usage-records" }))
        .route("/admin/usage/summary", get(|| async { "usage-summary" }))
        .route("/admin/billing/ledger", get(|| async { "billing-ledger" }))
        .route(
            "/admin/billing/summary",
            get(|| async { "billing-summary" }),
        )
        .route(
            "/admin/billing/quota-policies",
            get(|| async { "billing-quota-policies" }),
        )
        .route("/admin/routing/policies", get(|| async { "policies" }))
        .route(
            "/admin/routing/health-snapshots",
            get(|| async { "health-snapshots" }),
        )
        .route(
            "/admin/routing/decision-logs",
            get(|| async { "decision-logs" }),
        )
        .route(
            "/admin/routing/simulations",
            post(|| async { "simulations" }),
        )
}

pub fn admin_router_with_pool(pool: SqlitePool) -> Router {
    admin_router_with_pool_and_master_key(pool, "local-dev-master-key")
}

pub fn admin_router_with_store(store: Arc<dyn AdminStore>) -> Router {
    admin_router_with_store_and_secret_manager(
        store,
        CredentialSecretManager::database_encrypted("local-dev-master-key"),
    )
}

pub fn admin_router_with_pool_and_master_key(
    pool: SqlitePool,
    credential_master_key: impl Into<String>,
) -> Router {
    admin_router_with_store_and_secret_manager(
        Arc::new(SqliteAdminStore::new(pool)),
        CredentialSecretManager::database_encrypted(credential_master_key),
    )
}

pub fn admin_router_with_pool_and_secret_manager(
    pool: SqlitePool,
    secret_manager: CredentialSecretManager,
) -> Router {
    admin_router_with_store_and_secret_manager(
        Arc::new(SqliteAdminStore::new(pool)),
        secret_manager,
    )
}

pub fn admin_router_with_store_and_secret_manager(
    store: Arc<dyn AdminStore>,
    secret_manager: CredentialSecretManager,
) -> Router {
    admin_router_with_store_and_secret_manager_and_jwt_secret(
        store,
        secret_manager,
        DEFAULT_ADMIN_JWT_SIGNING_SECRET,
    )
}

pub fn admin_router_with_store_and_secret_manager_and_jwt_secret(
    store: Arc<dyn AdminStore>,
    secret_manager: CredentialSecretManager,
    jwt_signing_secret: impl Into<String>,
) -> Router {
    let state = AdminApiState::with_store_and_secret_manager_and_jwt_secret(
        store,
        secret_manager,
        jwt_signing_secret,
    );
    Router::new()
        .route("/admin/health", get(|| async { "ok" }))
        .route("/admin/auth/login", post(login_handler))
        .route("/admin/auth/me", get(me_handler))
        .route(
            "/admin/tenants",
            get(list_tenants_handler).post(create_tenant_handler),
        )
        .route(
            "/admin/projects",
            get(list_projects_handler).post(create_project_handler),
        )
        .route(
            "/admin/api-keys",
            get(list_api_keys_handler).post(create_api_key_handler),
        )
        .route(
            "/admin/channels",
            get(list_channels_handler).post(create_channel_handler),
        )
        .route(
            "/admin/providers",
            get(list_providers_handler).post(create_provider_handler),
        )
        .route(
            "/admin/credentials",
            get(list_credentials_handler).post(create_credential_handler),
        )
        .route(
            "/admin/models",
            get(list_models_handler).post(create_model_handler),
        )
        .route(
            "/admin/extensions/installations",
            get(list_extension_installations_handler).post(create_extension_installation_handler),
        )
        .route(
            "/admin/extensions/packages",
            get(list_extension_packages_handler),
        )
        .route(
            "/admin/extensions/instances",
            get(list_extension_instances_handler).post(create_extension_instance_handler),
        )
        .route(
            "/admin/extensions/runtime-statuses",
            get(list_extension_runtime_statuses_handler),
        )
        .route("/admin/usage/records", get(list_usage_records_handler))
        .route("/admin/usage/summary", get(usage_summary_handler))
        .route("/admin/billing/ledger", get(list_ledger_entries_handler))
        .route("/admin/billing/summary", get(billing_summary_handler))
        .route(
            "/admin/billing/quota-policies",
            get(list_quota_policies_handler).post(create_quota_policy_handler),
        )
        .route(
            "/admin/routing/policies",
            get(list_routing_policies_handler).post(create_routing_policy_handler),
        )
        .route(
            "/admin/routing/health-snapshots",
            get(list_provider_health_snapshots_handler),
        )
        .route(
            "/admin/routing/decision-logs",
            get(list_routing_decision_logs_handler),
        )
        .route("/admin/routing/simulations", post(simulate_routing_handler))
        .with_state(state)
}

async fn login_handler(
    State(state): State<AdminApiState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let token = issue_jwt(&request.subject, &state.jwt_signing_secret)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let claims = verify_jwt(&token, &state.jwt_signing_secret)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(LoginResponse { token, claims }))
}

async fn me_handler(claims: AuthenticatedAdminClaims) -> Json<Claims> {
    Json(claims.claims().clone())
}

async fn list_channels_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<Channel>>, StatusCode> {
    list_channels(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_channel_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateChannelRequest>,
) -> Result<(StatusCode, Json<Channel>), StatusCode> {
    let channel = persist_channel(state.store.as_ref(), &request.id, &request.name)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(channel)))
}

async fn list_providers_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<ProxyProvider>>, StatusCode> {
    list_providers(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_provider_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateProviderRequest>,
) -> Result<(StatusCode, Json<ProxyProvider>), StatusCode> {
    let primary_channel_id = request
        .channel_bindings
        .iter()
        .find(|binding| binding.is_primary)
        .map(|binding| binding.channel_id.as_str())
        .unwrap_or(&request.channel_id);
    let bindings = provider_bindings_from_request(&request);
    let provider = persist_provider_with_bindings_and_extension_id(
        state.store.as_ref(),
        &request.id,
        primary_channel_id,
        &request.adapter_kind,
        request.extension_id.as_deref(),
        &request.base_url,
        &request.display_name,
        &bindings,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(provider)))
}

async fn list_credentials_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<UpstreamCredential>>, StatusCode> {
    list_credentials(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_credential_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateCredentialRequest>,
) -> Result<(StatusCode, Json<UpstreamCredential>), StatusCode> {
    let credential = persist_credential_with_secret_and_manager(
        state.store.as_ref(),
        &state.secret_manager,
        &request.tenant_id,
        &request.provider_id,
        &request.key_reference,
        &request.secret_value,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(credential)))
}

async fn list_models_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<ModelCatalogEntry>>, StatusCode> {
    list_model_entries(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_model_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateModelRequest>,
) -> Result<(StatusCode, Json<ModelCatalogEntry>), StatusCode> {
    let model = persist_model_with_metadata(
        state.store.as_ref(),
        &request.external_name,
        &request.provider_id,
        &request.capabilities,
        request.streaming,
        request.context_window,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(model)))
}

async fn list_tenants_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<Tenant>>, StatusCode> {
    list_tenants(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_tenant_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateTenantRequest>,
) -> Result<(StatusCode, Json<Tenant>), StatusCode> {
    let tenant = persist_tenant(state.store.as_ref(), &request.id, &request.name)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(tenant)))
}

async fn list_projects_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<Project>>, StatusCode> {
    list_projects(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_project_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<Project>), StatusCode> {
    let project = persist_project(
        state.store.as_ref(),
        &request.tenant_id,
        &request.id,
        &request.name,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(project)))
}

async fn list_api_keys_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<GatewayApiKeyRecord>>, StatusCode> {
    list_gateway_api_keys(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_api_key_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateApiKeyRequest>,
) -> Result<(StatusCode, Json<CreatedGatewayApiKey>), StatusCode> {
    let created = persist_gateway_api_key(
        state.store.as_ref(),
        &request.tenant_id,
        &request.project_id,
        &request.environment,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(created)))
}

async fn list_extension_installations_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<ExtensionInstallation>>, StatusCode> {
    list_extension_installations(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_extension_installation_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateExtensionInstallationRequest>,
) -> Result<(StatusCode, Json<ExtensionInstallation>), StatusCode> {
    let installation = persist_extension_installation(
        state.store.as_ref(),
        &request.installation_id,
        &request.extension_id,
        request.runtime,
        request.enabled,
        request.entrypoint.as_deref(),
        request.config,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(installation)))
}

async fn list_extension_instances_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<ExtensionInstance>>, StatusCode> {
    list_extension_instances(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn list_extension_packages_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<sdkwork_api_app_extension::DiscoveredExtensionPackageRecord>>, StatusCode> {
    list_discovered_extension_packages(&state.extension_discovery_policy)
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_extension_instance_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateExtensionInstanceRequest>,
) -> Result<(StatusCode, Json<ExtensionInstance>), StatusCode> {
    let instance = persist_extension_instance(
        state.store.as_ref(),
        PersistExtensionInstanceInput {
            instance_id: &request.instance_id,
            installation_id: &request.installation_id,
            extension_id: &request.extension_id,
            enabled: request.enabled,
            base_url: request.base_url.as_deref(),
            credential_ref: request.credential_ref.as_deref(),
            config: request.config,
        },
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(instance)))
}

async fn list_extension_runtime_statuses_handler(
    _claims: AuthenticatedAdminClaims,
    _state: State<AdminApiState>,
) -> Result<Json<Vec<sdkwork_api_app_extension::ExtensionRuntimeStatusRecord>>, StatusCode> {
    list_extension_runtime_statuses()
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn list_provider_health_snapshots_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<ProviderHealthSnapshot>>, StatusCode> {
    list_provider_health_snapshots(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn simulate_routing_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<RoutingSimulationRequest>,
) -> Result<Json<RoutingSimulationResponse>, StatusCode> {
    let decision = select_route_with_store(
        state.store.as_ref(),
        &request.capability,
        &request.model,
        RoutingDecisionSource::AdminSimulation,
        None,
        None,
        request.selection_seed,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(RoutingSimulationResponse {
        selected_provider_id: decision.selected_provider_id,
        candidate_ids: decision.candidate_ids,
        matched_policy_id: decision.matched_policy_id,
        strategy: decision.strategy,
        selection_seed: decision.selection_seed,
        selection_reason: decision.selection_reason,
        slo_applied: decision.slo_applied,
        slo_degraded: decision.slo_degraded,
        assessments: decision.assessments,
    }))
}

async fn list_routing_decision_logs_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<RoutingDecisionLog>>, StatusCode> {
    list_routing_decision_logs(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn list_routing_policies_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<RoutingPolicy>>, StatusCode> {
    list_routing_policies(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_routing_policy_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateRoutingPolicyRequest>,
) -> Result<(StatusCode, Json<RoutingPolicy>), StatusCode> {
    let policy = create_routing_policy(CreateRoutingPolicyInput {
        policy_id: &request.policy_id,
        capability: &request.capability,
        model_pattern: &request.model_pattern,
        enabled: request.enabled,
        priority: request.priority,
        strategy: request.strategy,
        ordered_provider_ids: &request.ordered_provider_ids,
        default_provider_id: request.default_provider_id.as_deref(),
        max_cost: request.max_cost,
        max_latency_ms: request.max_latency_ms,
        require_healthy: request.require_healthy,
    })
    .map_err(|_| StatusCode::BAD_REQUEST)?;
    let policy = persist_routing_policy(state.store.as_ref(), &policy)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(policy)))
}

async fn list_usage_records_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<UsageRecord>>, StatusCode> {
    list_usage_records(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn usage_summary_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<UsageSummary>, StatusCode> {
    summarize_usage_records_from_store(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn list_ledger_entries_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<LedgerEntry>>, StatusCode> {
    list_ledger_entries(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn billing_summary_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<BillingSummary>, StatusCode> {
    summarize_billing_from_store(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn list_quota_policies_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<QuotaPolicy>>, StatusCode> {
    list_quota_policies(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_quota_policy_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateQuotaPolicyRequest>,
) -> Result<(StatusCode, Json<QuotaPolicy>), StatusCode> {
    let policy = create_quota_policy(
        &request.policy_id,
        &request.project_id,
        request.max_units,
        request.enabled,
    )
    .map_err(|_| StatusCode::BAD_REQUEST)?;
    let policy = persist_quota_policy(state.store.as_ref(), &policy)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(policy)))
}

fn provider_bindings_from_request(request: &CreateProviderRequest) -> Vec<ProviderChannelBinding> {
    let mut bindings = if request.channel_bindings.is_empty() {
        vec![ProviderChannelBinding::primary(
            &request.id,
            &request.channel_id,
        )]
    } else {
        request
            .channel_bindings
            .iter()
            .map(|binding| {
                let base = ProviderChannelBinding::new(&request.id, &binding.channel_id);
                if binding.is_primary {
                    ProviderChannelBinding::primary(&request.id, &binding.channel_id)
                } else {
                    base
                }
            })
            .collect::<Vec<_>>()
    };

    if !bindings
        .iter()
        .any(|binding| binding.channel_id == request.channel_id)
    {
        bindings.push(ProviderChannelBinding::primary(
            &request.id,
            &request.channel_id,
        ));
    }

    bindings
}

fn default_true() -> bool {
    true
}
