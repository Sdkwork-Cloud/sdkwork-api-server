use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use sdkwork_api_app_catalog::{
    list_channels, list_model_entries, list_providers, persist_channel, persist_model,
    persist_provider,
};
use sdkwork_api_app_credential::{list_credentials, persist_credential};
use sdkwork_api_app_identity::{issue_jwt, verify_jwt, Claims};
use sdkwork_api_domain_catalog::{Channel, ModelCatalogEntry, ProxyProvider};
use sdkwork_api_domain_credential::UpstreamCredential;
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct AdminApiState {
    store: SqliteAdminStore,
}

impl AdminApiState {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            store: SqliteAdminStore::new(pool),
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
    display_name: String,
}

#[derive(Debug, Deserialize)]
struct CreateCredentialRequest {
    tenant_id: String,
    provider_id: String,
    key_reference: String,
}

#[derive(Debug, Deserialize)]
struct CreateModelRequest {
    external_name: String,
    provider_id: String,
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
        .route("/admin/routing/policies", get(|| async { "policies" }))
        .route(
            "/admin/routing/simulations",
            post(|| async { "simulations" }),
        )
}

pub fn admin_router_with_pool(pool: SqlitePool) -> Router {
    Router::new()
        .route("/admin/health", get(|| async { "ok" }))
        .route("/admin/auth/login", post(login_handler))
        .route("/admin/auth/me", get(|| async { "me" }))
        .route("/admin/tenants", get(|| async { "tenants" }))
        .route("/admin/projects", get(|| async { "projects" }))
        .route("/admin/api-keys", get(|| async { "api-keys" }))
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
        .route("/admin/routing/policies", get(|| async { "policies" }))
        .route(
            "/admin/routing/simulations",
            post(|| async { "simulations" }),
        )
        .with_state(AdminApiState::new(pool))
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
    list_channels(&state.store)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_channel_handler(
    State(state): State<AdminApiState>,
    Json(request): Json<CreateChannelRequest>,
) -> Result<(StatusCode, Json<Channel>), StatusCode> {
    let channel = persist_channel(&state.store, &request.id, &request.name)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(channel)))
}

async fn list_providers_handler(
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<ProxyProvider>>, StatusCode> {
    list_providers(&state.store)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_provider_handler(
    State(state): State<AdminApiState>,
    Json(request): Json<CreateProviderRequest>,
) -> Result<(StatusCode, Json<ProxyProvider>), StatusCode> {
    let provider = persist_provider(
        &state.store,
        &request.id,
        &request.channel_id,
        &request.display_name,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(provider)))
}

async fn list_credentials_handler(
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<UpstreamCredential>>, StatusCode> {
    list_credentials(&state.store)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_credential_handler(
    State(state): State<AdminApiState>,
    Json(request): Json<CreateCredentialRequest>,
) -> Result<(StatusCode, Json<UpstreamCredential>), StatusCode> {
    let credential = persist_credential(
        &state.store,
        &request.tenant_id,
        &request.provider_id,
        &request.key_reference,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(credential)))
}

async fn list_models_handler(
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<ModelCatalogEntry>>, StatusCode> {
    list_model_entries(&state.store)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_model_handler(
    State(state): State<AdminApiState>,
    Json(request): Json<CreateModelRequest>,
) -> Result<(StatusCode, Json<ModelCatalogEntry>), StatusCode> {
    let model = persist_model(&state.store, &request.external_name, &request.provider_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(model)))
}
