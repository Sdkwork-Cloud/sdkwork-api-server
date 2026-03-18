use std::sync::Arc;

use axum::{
    extract::{FromRequestParts, Path, State},
    http::{header, request::Parts, StatusCode},
    routing::{delete, get, post},
    Json, Router,
};
use sdkwork_api_app_billing::{
    list_ledger_entries, list_quota_policies, summarize_billing_snapshot,
};
use sdkwork_api_app_identity::{
    change_portal_password, create_portal_api_key_with_metadata, delete_portal_api_key,
    list_portal_api_keys, load_portal_user_profile, load_portal_workspace_summary,
    login_portal_user, register_portal_user, set_portal_api_key_active, verify_portal_jwt,
    CreatedGatewayApiKey, PortalAuthSession, PortalClaims, PortalIdentityError,
    PortalWorkspaceSummary,
};
use sdkwork_api_app_routing::{
    list_routing_decision_logs, select_route_with_store_context,
    simulate_route_with_store_selection_context, RouteSelectionContext,
};
use sdkwork_api_app_usage::{list_usage_records, summarize_usage_records};
use sdkwork_api_domain_billing::{LedgerEntry, ProjectBillingSummary};
use sdkwork_api_domain_catalog::ProxyProvider;
use sdkwork_api_domain_identity::{GatewayApiKeyRecord, PortalUserProfile};
use sdkwork_api_domain_routing::{
    ProjectRoutingPreferences, RoutingDecision, RoutingDecisionLog, RoutingDecisionSource,
    RoutingStrategy,
};
use sdkwork_api_domain_usage::{UsageRecord, UsageSummary};
use sdkwork_api_observability::{observe_http_metrics, observe_http_tracing, HttpMetricsRegistry};
use sdkwork_api_storage_core::{AdminStore, Reloadable};
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

const DEFAULT_PORTAL_JWT_SIGNING_SECRET: &str = "local-dev-portal-jwt-secret";

pub struct PortalApiState {
    live_store: Reloadable<Arc<dyn AdminStore>>,
    store: Arc<dyn AdminStore>,
    live_jwt_signing_secret: Reloadable<String>,
    jwt_signing_secret: String,
}

impl Clone for PortalApiState {
    fn clone(&self) -> Self {
        Self {
            live_store: self.live_store.clone(),
            store: self.live_store.snapshot(),
            live_jwt_signing_secret: self.live_jwt_signing_secret.clone(),
            jwt_signing_secret: self.live_jwt_signing_secret.snapshot(),
        }
    }
}

impl PortalApiState {
    pub fn new(pool: SqlitePool) -> Self {
        Self::with_jwt_secret(pool, DEFAULT_PORTAL_JWT_SIGNING_SECRET)
    }

    pub fn with_jwt_secret(pool: SqlitePool, jwt_signing_secret: impl Into<String>) -> Self {
        Self::with_store_and_jwt_secret(Arc::new(SqliteAdminStore::new(pool)), jwt_signing_secret)
    }

    pub fn with_store_and_jwt_secret(
        store: Arc<dyn AdminStore>,
        jwt_signing_secret: impl Into<String>,
    ) -> Self {
        Self::with_live_store_and_jwt_secret_handle(
            Reloadable::new(store),
            Reloadable::new(jwt_signing_secret.into()),
        )
    }

    pub fn with_live_store_and_jwt_secret_handle(
        live_store: Reloadable<Arc<dyn AdminStore>>,
        live_jwt_signing_secret: Reloadable<String>,
    ) -> Self {
        Self {
            store: live_store.snapshot(),
            live_store,
            jwt_signing_secret: live_jwt_signing_secret.snapshot(),
            live_jwt_signing_secret,
        }
    }
}

#[derive(Clone, Debug)]
struct AuthenticatedPortalClaims(PortalClaims);

impl AuthenticatedPortalClaims {
    fn claims(&self) -> &PortalClaims {
        &self.0
    }
}

impl FromRequestParts<PortalApiState> for AuthenticatedPortalClaims {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &PortalApiState,
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

        verify_portal_jwt(token, &state.jwt_signing_secret)
            .map(Self)
            .map_err(|_| StatusCode::UNAUTHORIZED)
    }
}

#[derive(Debug, Deserialize)]
struct RegisterRequest {
    email: String,
    password: String,
    display_name: String,
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct CreateApiKeyRequest {
    environment: String,
    label: String,
    #[serde(default)]
    expires_at_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct UpdateApiKeyStatusRequest {
    active: bool,
}

#[derive(Debug, Deserialize)]
struct ChangePasswordRequest {
    current_password: String,
    new_password: String,
}

#[derive(Debug, Deserialize)]
struct SaveRoutingPreferencesRequest {
    preset_id: String,
    strategy: RoutingStrategy,
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
    #[serde(default)]
    preferred_region: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PortalRoutingPreviewRequest {
    capability: String,
    model: String,
    #[serde(default)]
    requested_region: Option<String>,
    #[serde(default)]
    selection_seed: Option<u64>,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: ErrorBody,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    message: String,
}

#[derive(Debug, Serialize)]
struct PortalDashboardSummary {
    workspace: PortalWorkspaceSummary,
    usage_summary: UsageSummary,
    billing_summary: ProjectBillingSummary,
    recent_requests: Vec<UsageRecord>,
    api_key_count: usize,
}

#[derive(Debug, Serialize)]
struct PortalRoutingProviderOption {
    provider_id: String,
    display_name: String,
    channel_id: String,
    #[serde(default)]
    preferred: bool,
    #[serde(default)]
    default_provider: bool,
}

#[derive(Debug, Serialize)]
struct PortalRoutingSummary {
    project_id: String,
    preferences: ProjectRoutingPreferences,
    latest_model_hint: String,
    preview: RoutingDecision,
    provider_options: Vec<PortalRoutingProviderOption>,
}

pub fn portal_router() -> Router {
    let service_name: Arc<str> = Arc::from("portal");
    let metrics = Arc::new(HttpMetricsRegistry::new("portal"));
    Router::new()
        .route(
            "/metrics",
            get({
                let metrics = metrics.clone();
                move || {
                    let metrics = metrics.clone();
                    async move {
                        (
                            [(
                                header::CONTENT_TYPE,
                                "text/plain; version=0.0.4; charset=utf-8",
                            )],
                            metrics.render_prometheus(),
                        )
                    }
                }
            }),
        )
        .route("/portal/health", get(|| async { "ok" }))
        .route("/portal/auth/register", post(|| async { "register" }))
        .route("/portal/auth/login", post(|| async { "login" }))
        .route("/portal/auth/me", get(|| async { "me" }))
        .route(
            "/portal/auth/change-password",
            post(|| async { "change-password" }),
        )
        .route("/portal/dashboard", get(|| async { "dashboard" }))
        .route("/portal/workspace", get(|| async { "workspace" }))
        .route("/portal/api-keys", get(|| async { "api-keys" }))
        .route("/portal/usage/records", get(|| async { "usage-records" }))
        .route("/portal/usage/summary", get(|| async { "usage-summary" }))
        .route(
            "/portal/billing/summary",
            get(|| async { "billing-summary" }),
        )
        .route("/portal/billing/ledger", get(|| async { "billing-ledger" }))
        .route(
            "/portal/routing/summary",
            get(|| async { "routing-summary" }),
        )
        .route(
            "/portal/routing/preferences",
            get(|| async { "routing-preferences" }).post(|| async { "routing-preferences" }),
        )
        .route(
            "/portal/routing/preview",
            post(|| async { "routing-preview" }),
        )
        .route(
            "/portal/routing/decision-logs",
            get(|| async { "routing-decision-logs" }),
        )
        .layer(axum::middleware::from_fn_with_state(
            metrics,
            observe_http_metrics,
        ))
        .layer(axum::middleware::from_fn_with_state(
            service_name,
            observe_http_tracing,
        ))
}

pub fn portal_router_with_pool(pool: SqlitePool) -> Router {
    portal_router_with_pool_and_jwt_secret(pool, DEFAULT_PORTAL_JWT_SIGNING_SECRET)
}

pub fn portal_router_with_pool_and_jwt_secret(
    pool: SqlitePool,
    jwt_signing_secret: impl Into<String>,
) -> Router {
    portal_router_with_store_and_jwt_secret(
        Arc::new(SqliteAdminStore::new(pool)),
        jwt_signing_secret,
    )
}

pub fn portal_router_with_store(store: Arc<dyn AdminStore>) -> Router {
    portal_router_with_store_and_jwt_secret(store, DEFAULT_PORTAL_JWT_SIGNING_SECRET)
}

pub fn portal_router_with_store_and_jwt_secret(
    store: Arc<dyn AdminStore>,
    jwt_signing_secret: impl Into<String>,
) -> Router {
    portal_router_with_state(PortalApiState::with_store_and_jwt_secret(
        store,
        jwt_signing_secret,
    ))
}

pub fn portal_router_with_state(state: PortalApiState) -> Router {
    let service_name: Arc<str> = Arc::from("portal");
    let metrics = Arc::new(HttpMetricsRegistry::new("portal"));
    Router::new()
        .route(
            "/metrics",
            get({
                let metrics = metrics.clone();
                move || {
                    let metrics = metrics.clone();
                    async move {
                        (
                            [(
                                header::CONTENT_TYPE,
                                "text/plain; version=0.0.4; charset=utf-8",
                            )],
                            metrics.render_prometheus(),
                        )
                    }
                }
            }),
        )
        .route("/portal/health", get(|| async { "ok" }))
        .route("/portal/auth/register", post(register_handler))
        .route("/portal/auth/login", post(login_handler))
        .route("/portal/auth/me", get(me_handler))
        .route(
            "/portal/auth/change-password",
            post(change_password_handler),
        )
        .route("/portal/dashboard", get(dashboard_handler))
        .route("/portal/workspace", get(workspace_handler))
        .route(
            "/portal/api-keys",
            get(list_api_keys_handler).post(create_api_key_handler),
        )
        .route(
            "/portal/api-keys/{hashed_key}/status",
            post(update_api_key_status_handler),
        )
        .route(
            "/portal/api-keys/{hashed_key}",
            delete(delete_api_key_handler),
        )
        .route("/portal/usage/records", get(list_usage_records_handler))
        .route("/portal/usage/summary", get(usage_summary_handler))
        .route("/portal/billing/summary", get(billing_summary_handler))
        .route("/portal/billing/ledger", get(list_billing_ledger_handler))
        .route("/portal/routing/summary", get(routing_summary_handler))
        .route(
            "/portal/routing/preferences",
            get(get_routing_preferences_handler).post(save_routing_preferences_handler),
        )
        .route("/portal/routing/preview", post(preview_routing_handler))
        .route(
            "/portal/routing/decision-logs",
            get(list_routing_decision_logs_handler),
        )
        .layer(axum::middleware::from_fn_with_state(
            metrics,
            observe_http_metrics,
        ))
        .layer(axum::middleware::from_fn_with_state(
            service_name,
            observe_http_tracing,
        ))
        .with_state(state)
}

async fn register_handler(
    State(state): State<PortalApiState>,
    Json(request): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<PortalAuthSession>), (StatusCode, Json<ErrorResponse>)> {
    register_portal_user(
        state.store.as_ref(),
        &request.email,
        &request.password,
        &request.display_name,
        &state.jwt_signing_secret,
    )
    .await
    .map(|session| (StatusCode::CREATED, Json(session)))
    .map_err(portal_error_response)
}

async fn login_handler(
    State(state): State<PortalApiState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<PortalAuthSession>, (StatusCode, Json<ErrorResponse>)> {
    login_portal_user(
        state.store.as_ref(),
        &request.email,
        &request.password,
        &state.jwt_signing_secret,
    )
    .await
    .map(Json)
    .map_err(portal_error_response)
}

async fn me_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalUserProfile>, StatusCode> {
    load_portal_user_profile(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(Json)
        .ok_or(StatusCode::UNAUTHORIZED)
}

async fn change_password_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    Json(request): Json<ChangePasswordRequest>,
) -> Result<Json<PortalUserProfile>, (StatusCode, Json<ErrorResponse>)> {
    change_portal_password(
        state.store.as_ref(),
        &claims.claims().sub,
        &request.current_password,
        &request.new_password,
    )
    .await
    .map(Json)
    .map_err(portal_error_response)
}

async fn workspace_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalWorkspaceSummary>, StatusCode> {
    load_workspace_for_user(state.store.as_ref(), &claims.claims().sub)
        .await
        .map(Json)
}

async fn list_api_keys_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<GatewayApiKeyRecord>>, (StatusCode, Json<ErrorResponse>)> {
    list_portal_api_keys(state.store.as_ref(), &claims.claims().sub)
        .await
        .map(Json)
        .map_err(portal_error_response)
}

async fn create_api_key_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    Json(request): Json<CreateApiKeyRequest>,
) -> Result<(StatusCode, Json<CreatedGatewayApiKey>), (StatusCode, Json<ErrorResponse>)> {
    create_portal_api_key_with_metadata(
        state.store.as_ref(),
        &claims.claims().sub,
        &request.environment,
        &request.label,
        request.expires_at_ms,
    )
    .await
    .map(|created| (StatusCode::CREATED, Json(created)))
    .map_err(portal_error_response)
}

async fn update_api_key_status_handler(
    claims: AuthenticatedPortalClaims,
    Path(hashed_key): Path<String>,
    State(state): State<PortalApiState>,
    Json(request): Json<UpdateApiKeyStatusRequest>,
) -> Result<Json<GatewayApiKeyRecord>, (StatusCode, Json<ErrorResponse>)> {
    match set_portal_api_key_active(
        state.store.as_ref(),
        &claims.claims().sub,
        &hashed_key,
        request.active,
    )
    .await
    .map_err(portal_error_response)?
    {
        Some(record) => Ok(Json(record)),
        None => Err(portal_error_response(PortalIdentityError::NotFound(
            "api key not found".to_owned(),
        ))),
    }
}

async fn delete_api_key_handler(
    claims: AuthenticatedPortalClaims,
    Path(hashed_key): Path<String>,
    State(state): State<PortalApiState>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let deleted = delete_portal_api_key(state.store.as_ref(), &claims.claims().sub, &hashed_key)
        .await
        .map_err(portal_error_response)?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(portal_error_response(PortalIdentityError::NotFound(
            "api key not found".to_owned(),
        )))
    }
}

async fn dashboard_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalDashboardSummary>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let usage_records =
        load_project_usage_records(state.store.as_ref(), &workspace.project.id).await?;
    let usage_summary = summarize_usage_records(&usage_records);
    let billing_summary =
        load_project_billing_summary(state.store.as_ref(), &workspace.project.id).await?;
    let api_key_count = list_portal_api_keys(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .len();

    let recent_requests = usage_records.iter().take(10).cloned().collect();

    Ok(Json(PortalDashboardSummary {
        workspace,
        usage_summary,
        billing_summary,
        recent_requests,
        api_key_count,
    }))
}

async fn list_usage_records_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<UsageRecord>>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    load_project_usage_records(state.store.as_ref(), &workspace.project.id)
        .await
        .map(Json)
}

async fn usage_summary_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<UsageSummary>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let usage_records =
        load_project_usage_records(state.store.as_ref(), &workspace.project.id).await?;
    Ok(Json(summarize_usage_records(&usage_records)))
}

async fn billing_summary_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<ProjectBillingSummary>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    load_project_billing_summary(state.store.as_ref(), &workspace.project.id)
        .await
        .map(Json)
}

async fn list_billing_ledger_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<LedgerEntry>>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let ledger = list_ledger_entries(state.store.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .filter(|entry| entry.project_id == workspace.project.id)
        .collect();
    Ok(Json(ledger))
}

async fn get_routing_preferences_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<ProjectRoutingPreferences>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    load_project_routing_preferences_or_default(state.store.as_ref(), &workspace.project.id)
        .await
        .map(Json)
}

async fn save_routing_preferences_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    Json(request): Json<SaveRoutingPreferencesRequest>,
) -> Result<Json<ProjectRoutingPreferences>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let preferences = ProjectRoutingPreferences::new(workspace.project.id.clone())
        .with_preset_id(request.preset_id)
        .with_strategy(request.strategy)
        .with_ordered_provider_ids(request.ordered_provider_ids)
        .with_default_provider_id_option(request.default_provider_id)
        .with_max_cost_option(request.max_cost)
        .with_max_latency_ms_option(request.max_latency_ms)
        .with_require_healthy(request.require_healthy)
        .with_preferred_region_option(request.preferred_region)
        .with_updated_at_ms(current_time_millis());

    state
        .store
        .insert_project_routing_preferences(&preferences)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn preview_routing_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    Json(request): Json<PortalRoutingPreviewRequest>,
) -> Result<Json<RoutingDecision>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    select_route_with_store_context(
        state.store.as_ref(),
        &request.capability,
        &request.model,
        portal_route_selection_context(
            &workspace,
            RoutingDecisionSource::PortalSimulation,
            request.requested_region.as_deref(),
            request.selection_seed,
        ),
    )
    .await
    .map(Json)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn list_routing_decision_logs_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<RoutingDecisionLog>>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    load_project_routing_decision_logs(state.store.as_ref(), &workspace.project.id)
        .await
        .map(Json)
}

async fn routing_summary_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalRoutingSummary>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let preferences =
        load_project_routing_preferences_or_default(state.store.as_ref(), &workspace.project.id)
            .await?;
    let (latest_capability_hint, latest_model_hint) =
        load_latest_route_hint(state.store.as_ref(), &workspace.project.id).await?;
    let preview = simulate_route_with_store_selection_context(
        state.store.as_ref(),
        &latest_capability_hint,
        &latest_model_hint,
        portal_route_selection_context(
            &workspace,
            RoutingDecisionSource::PortalSimulation,
            None,
            None,
        ),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let provider_options =
        load_routing_provider_options(state.store.as_ref(), &latest_model_hint, &preferences)
            .await?;

    Ok(Json(PortalRoutingSummary {
        project_id: workspace.project.id,
        preferences,
        latest_model_hint,
        preview,
        provider_options,
    }))
}

fn portal_error_response(error: PortalIdentityError) -> (StatusCode, Json<ErrorResponse>) {
    let status = match error {
        PortalIdentityError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        PortalIdentityError::DuplicateEmail => StatusCode::CONFLICT,
        PortalIdentityError::Protected(_) => StatusCode::CONFLICT,
        PortalIdentityError::InvalidCredentials | PortalIdentityError::InactiveUser => {
            StatusCode::UNAUTHORIZED
        }
        PortalIdentityError::NotFound(_) => StatusCode::NOT_FOUND,
        PortalIdentityError::Storage(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };
    let body = ErrorResponse {
        error: ErrorBody {
            message: error.to_string(),
        },
    };
    (status, Json(body))
}

async fn load_workspace_for_user(
    store: &dyn AdminStore,
    user_id: &str,
) -> Result<PortalWorkspaceSummary, StatusCode> {
    load_portal_workspace_summary(store, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)
}

async fn load_project_usage_records(
    store: &dyn AdminStore,
    project_id: &str,
) -> Result<Vec<UsageRecord>, StatusCode> {
    let mut usage_records = list_usage_records(store)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .filter(|record| record.project_id == project_id)
        .collect::<Vec<_>>();
    usage_records.sort_by(|left, right| right.created_at_ms.cmp(&left.created_at_ms));
    Ok(usage_records)
}

async fn load_project_billing_summary(
    store: &dyn AdminStore,
    project_id: &str,
) -> Result<ProjectBillingSummary, StatusCode> {
    let ledger = list_ledger_entries(store)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .filter(|entry| entry.project_id == project_id)
        .collect::<Vec<_>>();
    let policies = list_quota_policies(store)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .filter(|policy| policy.project_id == project_id)
        .collect::<Vec<_>>();
    let billing = summarize_billing_snapshot(&ledger, &policies);

    Ok(billing
        .projects
        .into_iter()
        .next()
        .unwrap_or_else(|| ProjectBillingSummary::new(project_id.to_owned())))
}

fn current_time_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn portal_route_selection_context<'a>(
    workspace: &'a PortalWorkspaceSummary,
    decision_source: RoutingDecisionSource,
    requested_region: Option<&'a str>,
    selection_seed: Option<u64>,
) -> RouteSelectionContext<'a> {
    RouteSelectionContext::new(decision_source)
        .with_tenant_id_option(Some(workspace.tenant.id.as_str()))
        .with_project_id_option(Some(workspace.project.id.as_str()))
        .with_requested_region_option(requested_region)
        .with_selection_seed_option(selection_seed)
}

async fn load_project_routing_preferences_or_default(
    store: &dyn AdminStore,
    project_id: &str,
) -> Result<ProjectRoutingPreferences, StatusCode> {
    store
        .find_project_routing_preferences(project_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(Ok)
        .unwrap_or_else(|| {
            Ok(ProjectRoutingPreferences::new(project_id.to_owned())
                .with_preset_id("platform_default"))
        })
}

async fn load_project_routing_decision_logs(
    store: &dyn AdminStore,
    project_id: &str,
) -> Result<Vec<RoutingDecisionLog>, StatusCode> {
    let mut logs = list_routing_decision_logs(store)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .filter(|log| log.project_id.as_deref() == Some(project_id))
        .collect::<Vec<_>>();
    logs.sort_by(|left, right| {
        right
            .created_at_ms
            .cmp(&left.created_at_ms)
            .then_with(|| right.decision_id.cmp(&left.decision_id))
    });
    Ok(logs)
}

async fn load_latest_route_hint(
    store: &dyn AdminStore,
    project_id: &str,
) -> Result<(String, String), StatusCode> {
    let routing_logs = load_project_routing_decision_logs(store, project_id).await?;
    if let Some(log) = routing_logs.first() {
        return Ok((log.capability.clone(), log.route_key.clone()));
    }

    let usage_records = load_project_usage_records(store, project_id).await?;
    if let Some(record) = usage_records.first() {
        return Ok(("chat_completion".to_owned(), record.model.clone()));
    }

    let models = store
        .list_models()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if let Some(model) = models.first() {
        return Ok(("chat_completion".to_owned(), model.external_name.clone()));
    }

    Ok(("chat_completion".to_owned(), "gpt-4.1".to_owned()))
}

async fn load_routing_provider_options(
    store: &dyn AdminStore,
    model: &str,
    preferences: &ProjectRoutingPreferences,
) -> Result<Vec<PortalRoutingProviderOption>, StatusCode> {
    let model_provider_ids = store
        .list_models()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .filter(|entry| entry.external_name == model)
        .map(|entry| entry.provider_id)
        .collect::<Vec<_>>();

    let mut providers = store
        .list_providers()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .filter(|provider| {
            model_provider_ids.is_empty()
                || model_provider_ids
                    .iter()
                    .any(|provider_id| provider_id == &provider.id)
        })
        .collect::<Vec<_>>();
    sort_routing_provider_options(&mut providers, preferences);

    Ok(providers
        .into_iter()
        .map(|provider| PortalRoutingProviderOption {
            preferred: preferences
                .ordered_provider_ids
                .iter()
                .any(|provider_id| provider_id == &provider.id),
            default_provider: preferences.default_provider_id.as_deref() == Some(&provider.id),
            provider_id: provider.id,
            display_name: provider.display_name,
            channel_id: provider.channel_id,
        })
        .collect())
}

fn sort_routing_provider_options(
    providers: &mut [ProxyProvider],
    preferences: &ProjectRoutingPreferences,
) {
    providers.sort_by(|left, right| {
        provider_preference_rank(preferences, &left.id)
            .cmp(&provider_preference_rank(preferences, &right.id))
            .then_with(|| left.display_name.cmp(&right.display_name))
            .then_with(|| left.id.cmp(&right.id))
    });
}

fn provider_preference_rank(preferences: &ProjectRoutingPreferences, provider_id: &str) -> usize {
    preferences
        .ordered_provider_ids
        .iter()
        .position(|candidate| candidate == provider_id)
        .unwrap_or(usize::MAX)
}
