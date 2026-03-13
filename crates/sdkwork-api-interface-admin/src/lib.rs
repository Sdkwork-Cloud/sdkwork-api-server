use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use sdkwork_api_app_billing::list_ledger_entries;
use sdkwork_api_app_catalog::{
    list_channels, list_model_entries, list_providers, persist_channel, persist_model,
    persist_provider,
};
use sdkwork_api_app_credential::CredentialSecretManager;
use sdkwork_api_app_credential::{list_credentials, persist_credential_with_secret_and_manager};
use sdkwork_api_app_identity::{
    issue_jwt, list_gateway_api_keys, persist_gateway_api_key, verify_jwt, Claims,
    CreatedGatewayApiKey,
};
use sdkwork_api_app_routing::simulate_route_with_store;
use sdkwork_api_app_tenant::{list_projects, list_tenants, persist_project, persist_tenant};
use sdkwork_api_app_usage::list_usage_records;
use sdkwork_api_domain_billing::LedgerEntry;
use sdkwork_api_domain_catalog::{Channel, ModelCatalogEntry, ProxyProvider};
use sdkwork_api_domain_credential::UpstreamCredential;
use sdkwork_api_domain_identity::GatewayApiKeyRecord;
use sdkwork_api_domain_tenant::{Project, Tenant};
use sdkwork_api_domain_usage::UsageRecord;
use sdkwork_api_storage_core::AdminStore;
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct AdminApiState {
    store: Arc<dyn AdminStore>,
    secret_manager: CredentialSecretManager,
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
        Self {
            store: Arc::new(SqliteAdminStore::new(pool)),
            secret_manager,
        }
    }

    pub fn with_store_and_secret_manager(
        store: Arc<dyn AdminStore>,
        secret_manager: CredentialSecretManager,
    ) -> Self {
        Self {
            store,
            secret_manager,
        }
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
    adapter_kind: String,
    base_url: String,
    display_name: String,
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
struct RoutingSimulationRequest {
    capability: String,
    model: String,
}

#[derive(Debug, Serialize)]
struct RoutingSimulationResponse {
    selected_provider_id: String,
    candidate_ids: Vec<String>,
}

pub fn admin_router() -> Router {
    Router::new()
        .route("/admin/health", get(|| async { "ok" }))
        .route("/admin/auth/login", post(login_handler))
        .route("/admin/auth/me", get(|| async { "me" }))
        .route("/admin/tenants", get(|| async { "tenants" }))
        .route("/admin/projects", get(|| async { "projects" }))
        .route("/admin/api-keys", get(|| async { "api-keys" }))
        .route("/admin/channels", get(|| async { "channels" }))
        .route("/admin/providers", get(|| async { "providers" }))
        .route("/admin/credentials", get(|| async { "credentials" }))
        .route("/admin/models", get(|| async { "models" }))
        .route("/admin/usage/records", get(|| async { "usage-records" }))
        .route("/admin/billing/ledger", get(|| async { "billing-ledger" }))
        .route("/admin/routing/policies", get(|| async { "policies" }))
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
    Router::new()
        .route("/admin/health", get(|| async { "ok" }))
        .route("/admin/auth/login", post(login_handler))
        .route("/admin/auth/me", get(|| async { "me" }))
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
        .route("/admin/usage/records", get(list_usage_records_handler))
        .route("/admin/billing/ledger", get(list_ledger_entries_handler))
        .route("/admin/routing/policies", get(|| async { "policies" }))
        .route("/admin/routing/simulations", post(simulate_routing_handler))
        .with_state(AdminApiState::with_store_and_secret_manager(
            store,
            secret_manager,
        ))
}

async fn login_handler(
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let token = issue_jwt(&request.subject).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let claims = verify_jwt(&token).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(LoginResponse { token, claims }))
}

async fn list_channels_handler(
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<Channel>>, StatusCode> {
    list_channels(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_channel_handler(
    State(state): State<AdminApiState>,
    Json(request): Json<CreateChannelRequest>,
) -> Result<(StatusCode, Json<Channel>), StatusCode> {
    let channel = persist_channel(state.store.as_ref(), &request.id, &request.name)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(channel)))
}

async fn list_providers_handler(
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<ProxyProvider>>, StatusCode> {
    list_providers(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_provider_handler(
    State(state): State<AdminApiState>,
    Json(request): Json<CreateProviderRequest>,
) -> Result<(StatusCode, Json<ProxyProvider>), StatusCode> {
    let provider = persist_provider(
        state.store.as_ref(),
        &request.id,
        &request.channel_id,
        &request.adapter_kind,
        &request.base_url,
        &request.display_name,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(provider)))
}

async fn list_credentials_handler(
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<UpstreamCredential>>, StatusCode> {
    list_credentials(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_credential_handler(
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
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<ModelCatalogEntry>>, StatusCode> {
    list_model_entries(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_model_handler(
    State(state): State<AdminApiState>,
    Json(request): Json<CreateModelRequest>,
) -> Result<(StatusCode, Json<ModelCatalogEntry>), StatusCode> {
    let model = persist_model(
        state.store.as_ref(),
        &request.external_name,
        &request.provider_id,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(model)))
}

async fn list_tenants_handler(
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<Tenant>>, StatusCode> {
    list_tenants(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_tenant_handler(
    State(state): State<AdminApiState>,
    Json(request): Json<CreateTenantRequest>,
) -> Result<(StatusCode, Json<Tenant>), StatusCode> {
    let tenant = persist_tenant(state.store.as_ref(), &request.id, &request.name)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(tenant)))
}

async fn list_projects_handler(
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<Project>>, StatusCode> {
    list_projects(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_project_handler(
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
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<GatewayApiKeyRecord>>, StatusCode> {
    list_gateway_api_keys(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_api_key_handler(
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

async fn simulate_routing_handler(
    State(state): State<AdminApiState>,
    Json(request): Json<RoutingSimulationRequest>,
) -> Result<Json<RoutingSimulationResponse>, StatusCode> {
    let decision =
        simulate_route_with_store(state.store.as_ref(), &request.capability, &request.model)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(RoutingSimulationResponse {
        selected_provider_id: decision.selected_provider_id,
        candidate_ids: decision.candidate_ids,
    }))
}

async fn list_usage_records_handler(
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<UsageRecord>>, StatusCode> {
    list_usage_records(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn list_ledger_entries_handler(
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<LedgerEntry>>, StatusCode> {
    list_ledger_entries(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
