use std::sync::{Arc, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    body::Bytes,
    extract::FromRequestParts,
    extract::Path,
    extract::State,
    http::header,
    http::request::Parts,
    http::StatusCode,
    response::Html,
    routing::{delete, get, post, put},
    Json, Router,
};
use sdkwork_api_app_billing::{
    create_quota_policy, list_ledger_entries, list_quota_policies, persist_quota_policy,
    summarize_billing_from_store,
};
use sdkwork_api_app_catalog::{
    delete_channel as delete_catalog_channel, delete_channel_model as delete_catalog_channel_model,
    delete_model_price as delete_catalog_model_price, delete_model_variant,
    delete_provider as delete_catalog_provider, list_channel_models, list_channels,
    list_model_entries, list_model_prices, list_providers, persist_channel,
    persist_channel_model_with_metadata, persist_model_price_with_rates,
    persist_model_with_metadata, persist_provider_with_bindings_and_extension_id,
    PersistProviderWithBindingsRequest,
};
use sdkwork_api_app_coupon::{delete_coupon, list_coupons, persist_coupon};
use sdkwork_api_app_credential::CredentialSecretManager;
use sdkwork_api_app_credential::{
    delete_credential_with_manager, delete_provider_credentials_with_manager,
    delete_tenant_credentials_with_manager, list_credentials,
    persist_credential_with_secret_and_manager,
};
use sdkwork_api_app_extension::{
    configured_extension_discovery_policy_from_env, list_discovered_extension_packages,
    list_extension_installations, list_extension_instances, list_extension_runtime_statuses,
    list_provider_health_snapshots, persist_extension_installation, persist_extension_instance,
    PersistExtensionInstanceInput,
};
use sdkwork_api_app_gateway::{
    reload_extension_host_with_scope, ConfiguredExtensionHostReloadScope,
};
use sdkwork_api_app_identity::{
    change_admin_password, delete_admin_user, delete_gateway_api_key, delete_portal_user,
    list_admin_user_profiles, list_gateway_api_keys, list_portal_user_profiles,
    load_admin_user_profile, login_admin_user, persist_gateway_api_key, reset_admin_user_password,
    reset_portal_user_password, set_admin_user_active, set_gateway_api_key_active,
    set_portal_user_active, update_gateway_api_key_metadata, upsert_admin_user, upsert_portal_user,
    verify_jwt, AdminIdentityError, Claims, CreatedGatewayApiKey, PortalIdentityError,
};
use sdkwork_api_app_routing::{
    create_routing_policy, list_routing_decision_logs, list_routing_policies,
    persist_routing_policy, select_route_with_store_context, CreateRoutingPolicyInput,
    RouteSelectionContext,
};
use sdkwork_api_app_runtime::{
    create_extension_runtime_rollout_with_request, create_standalone_config_rollout,
    find_extension_runtime_rollout, find_standalone_config_rollout,
    list_extension_runtime_rollouts, list_standalone_config_rollouts,
    CreateExtensionRuntimeRolloutRequest, CreateStandaloneConfigRolloutRequest,
    ExtensionRuntimeRolloutDetails, StandaloneConfigRolloutDetails,
};
use sdkwork_api_app_tenant::{
    delete_project as delete_tenant_project, delete_tenant as delete_workspace_tenant,
    list_projects, list_tenants, persist_project, persist_tenant,
};
use sdkwork_api_app_usage::list_usage_records;
use sdkwork_api_app_usage::summarize_usage_records_from_store;
use sdkwork_api_domain_billing::{BillingSummary, LedgerEntry, QuotaPolicy};
use sdkwork_api_domain_catalog::{
    Channel, ChannelModelRecord, ModelCapability, ModelCatalogEntry, ModelPriceRecord,
    ProviderChannelBinding, ProxyProvider,
};
use sdkwork_api_domain_coupon::CouponCampaign;
use sdkwork_api_domain_credential::UpstreamCredential;
use sdkwork_api_domain_identity::{AdminUserProfile, GatewayApiKeyRecord, PortalUserProfile};
use sdkwork_api_domain_routing::{
    ProviderHealthSnapshot, RoutingDecisionLog, RoutingDecisionSource, RoutingPolicy,
    RoutingStrategy,
};
use sdkwork_api_domain_tenant::{Project, Tenant};
use sdkwork_api_domain_usage::{UsageRecord, UsageSummary};
use sdkwork_api_extension_core::{ExtensionInstallation, ExtensionInstance, ExtensionRuntime};
use sdkwork_api_observability::{observe_http_metrics, observe_http_tracing, HttpMetricsRegistry};
use sdkwork_api_openapi::{
    build_openapi_document, extract_routes_from_function, render_docs_html, HttpMethod,
    OpenApiServiceSpec, RouteEntry,
};
use sdkwork_api_storage_core::{AdminStore, Reloadable};
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::SqlitePool;

const DEFAULT_ADMIN_JWT_SIGNING_SECRET: &str = "local-dev-admin-jwt-secret";
const ADMIN_OPENAPI_SPEC: OpenApiServiceSpec<'static> = OpenApiServiceSpec {
    title: "SDKWORK Admin API",
    version: env!("CARGO_PKG_VERSION"),
    description: "OpenAPI 3.1 inventory generated from the current admin router implementation.",
    openapi_path: "/admin/openapi.json",
    docs_path: "/admin/docs",
};

fn admin_route_inventory() -> &'static [RouteEntry] {
    static ROUTES: OnceLock<Vec<RouteEntry>> = OnceLock::new();
    ROUTES
        .get_or_init(|| {
            extract_routes_from_function(include_str!("lib.rs"), "admin_router_with_state")
                .expect("admin route inventory")
        })
        .as_slice()
}

fn admin_openapi_document() -> &'static Value {
    static DOCUMENT: OnceLock<Value> = OnceLock::new();
    DOCUMENT.get_or_init(|| {
        build_openapi_document(
            &ADMIN_OPENAPI_SPEC,
            admin_route_inventory(),
            admin_tag_for_path,
            admin_route_requires_bearer_auth,
            admin_operation_summary,
        )
    })
}

fn admin_docs_html() -> &'static str {
    static HTML: OnceLock<String> = OnceLock::new();
    HTML.get_or_init(|| render_docs_html(&ADMIN_OPENAPI_SPEC))
        .as_str()
}

async fn admin_openapi_handler() -> Json<Value> {
    Json(admin_openapi_document().clone())
}

async fn admin_docs_handler() -> Html<String> {
    Html(admin_docs_html().to_owned())
}

fn admin_tag_for_path(path: &str) -> String {
    match path {
        "/metrics" | "/admin/health" => "system".to_owned(),
        "/admin/docs" | "/admin/openapi.json" => "docs".to_owned(),
        _ if path.starts_with("/admin/") => path
            .trim_start_matches("/admin/")
            .split('/')
            .find(|segment| !segment.is_empty() && !segment.starts_with('{'))
            .unwrap_or("admin")
            .to_owned(),
        _ => "admin".to_owned(),
    }
}

fn admin_route_requires_bearer_auth(path: &str, _method: HttpMethod) -> bool {
    !matches!(
        path,
        "/metrics" | "/admin/health" | "/admin/openapi.json" | "/admin/docs" | "/admin/auth/login"
    )
}

fn admin_operation_summary(path: &str, method: HttpMethod) -> String {
    match path {
        "/metrics" => "Prometheus metrics".to_owned(),
        "/admin/health" => "Health check".to_owned(),
        "/admin/openapi.json" => "OpenAPI document".to_owned(),
        "/admin/docs" => "Interactive API inventory".to_owned(),
        _ => format!(
            "{} {}",
            method.display_name(),
            humanize_admin_route_path(path)
        ),
    }
}

fn humanize_admin_route_path(path: &str) -> String {
    let parts = path
        .trim_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .filter(|segment| *segment != "admin")
        .map(|segment| {
            if segment.starts_with('{') && segment.ends_with('}') {
                format!(
                    "by {}",
                    segment
                        .trim_matches(|ch| ch == '{' || ch == '}')
                        .replace(['_', '-'], " ")
                )
            } else {
                segment.replace(['_', '-'], " ")
            }
        })
        .collect::<Vec<_>>();

    if parts.is_empty() {
        "root".to_owned()
    } else {
        parts.join(" / ")
    }
}

pub struct AdminApiState {
    live_store: Reloadable<Arc<dyn AdminStore>>,
    live_secret_manager: Reloadable<CredentialSecretManager>,
    store: Arc<dyn AdminStore>,
    secret_manager: CredentialSecretManager,
    live_jwt_signing_secret: Reloadable<String>,
    jwt_signing_secret: String,
}

impl Clone for AdminApiState {
    fn clone(&self) -> Self {
        Self {
            live_store: self.live_store.clone(),
            live_secret_manager: self.live_secret_manager.clone(),
            store: self.live_store.snapshot(),
            secret_manager: self.live_secret_manager.snapshot(),
            live_jwt_signing_secret: self.live_jwt_signing_secret.clone(),
            jwt_signing_secret: self.live_jwt_signing_secret.snapshot(),
        }
    }
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
        Self::with_store_and_secret_manager_and_jwt_secret(
            Arc::new(SqliteAdminStore::new(pool)),
            secret_manager,
            jwt_signing_secret,
        )
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
        Self::with_live_store_and_secret_manager_handle_and_jwt_secret_handle(
            Reloadable::new(store),
            Reloadable::new(secret_manager),
            Reloadable::new(jwt_signing_secret.into()),
        )
    }

    pub fn with_live_store_and_secret_manager_and_jwt_secret_handle(
        live_store: Reloadable<Arc<dyn AdminStore>>,
        secret_manager: CredentialSecretManager,
        live_jwt_signing_secret: Reloadable<String>,
    ) -> Self {
        Self::with_live_store_and_secret_manager_handle_and_jwt_secret_handle(
            live_store,
            Reloadable::new(secret_manager),
            live_jwt_signing_secret,
        )
    }

    pub fn with_live_store_and_secret_manager_handle_and_jwt_secret_handle(
        live_store: Reloadable<Arc<dyn AdminStore>>,
        live_secret_manager: Reloadable<CredentialSecretManager>,
        live_jwt_signing_secret: Reloadable<String>,
    ) -> Self {
        Self {
            store: live_store.snapshot(),
            secret_manager: live_secret_manager.snapshot(),
            live_store,
            live_secret_manager,
            jwt_signing_secret: live_jwt_signing_secret.snapshot(),
            live_jwt_signing_secret,
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
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct LoginResponse {
    token: String,
    claims: Claims,
    user: AdminUserProfile,
}

#[derive(Debug, Deserialize)]
struct ChangePasswordRequest {
    current_password: String,
    new_password: String,
}

fn default_user_active() -> bool {
    true
}

#[derive(Debug, Deserialize)]
struct UpsertOperatorUserRequest {
    #[serde(default)]
    id: Option<String>,
    email: String,
    display_name: String,
    #[serde(default)]
    password: Option<String>,
    #[serde(default = "default_user_active")]
    active: bool,
}

#[derive(Debug, Deserialize)]
struct UpsertPortalUserRequest {
    #[serde(default)]
    id: Option<String>,
    email: String,
    display_name: String,
    #[serde(default)]
    password: Option<String>,
    workspace_tenant_id: String,
    workspace_project_id: String,
    #[serde(default = "default_user_active")]
    active: bool,
}

#[derive(Debug, Deserialize)]
struct UpdateUserStatusRequest {
    active: bool,
}

#[derive(Debug, Deserialize)]
struct ResetUserPasswordRequest {
    new_password: String,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: ErrorBody,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    message: String,
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
struct CreateChannelModelRequest {
    channel_id: String,
    model_id: String,
    model_display_name: String,
    #[serde(default)]
    capabilities: Vec<ModelCapability>,
    #[serde(default)]
    streaming: bool,
    #[serde(default)]
    context_window: Option<u64>,
    #[serde(default)]
    description: Option<String>,
}

fn default_currency_code() -> String {
    "USD".to_owned()
}

fn default_price_unit() -> String {
    "per_1m_tokens".to_owned()
}

#[derive(Debug, Deserialize)]
struct CreateModelPriceRequest {
    channel_id: String,
    model_id: String,
    proxy_provider_id: String,
    #[serde(default = "default_currency_code")]
    currency_code: String,
    #[serde(default = "default_price_unit")]
    price_unit: String,
    #[serde(default)]
    input_price: f64,
    #[serde(default)]
    output_price: f64,
    #[serde(default)]
    cache_read_price: f64,
    #[serde(default)]
    cache_write_price: f64,
    #[serde(default)]
    request_price: f64,
    #[serde(default = "default_true")]
    is_active: bool,
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
struct CreateCouponRequest {
    id: String,
    code: String,
    discount_label: String,
    audience: String,
    remaining: u64,
    active: bool,
    note: String,
    expires_on: String,
}

#[derive(Debug, Deserialize)]
struct CreateApiKeyRequest {
    tenant_id: String,
    project_id: String,
    environment: String,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    notes: Option<String>,
    #[serde(default)]
    expires_at_ms: Option<u64>,
    #[serde(default)]
    plaintext_key: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UpdateApiKeyRequest {
    tenant_id: String,
    project_id: String,
    environment: String,
    label: String,
    #[serde(default)]
    notes: Option<String>,
    #[serde(default)]
    expires_at_ms: Option<u64>,
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
    requested_region: Option<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    requested_region: Option<String>,
    #[serde(default)]
    slo_applied: bool,
    #[serde(default)]
    slo_degraded: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    assessments: Vec<sdkwork_api_domain_routing::RoutingCandidateAssessment>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum ExtensionRuntimeReloadScope {
    All,
    Extension,
    Instance,
}

#[derive(Debug, Deserialize, Default)]
struct ExtensionRuntimeReloadRequest {
    #[serde(default)]
    extension_id: Option<String>,
    #[serde(default)]
    instance_id: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct ExtensionRuntimeRolloutCreateRequest {
    #[serde(default)]
    extension_id: Option<String>,
    #[serde(default)]
    instance_id: Option<String>,
    #[serde(default)]
    timeout_secs: Option<u64>,
}

#[derive(Debug, Serialize)]
struct ExtensionRuntimeReloadResponse {
    scope: ExtensionRuntimeReloadScope,
    requested_extension_id: Option<String>,
    requested_instance_id: Option<String>,
    resolved_extension_id: Option<String>,
    discovered_package_count: usize,
    loadable_package_count: usize,
    active_runtime_count: usize,
    reloaded_at_ms: u64,
    runtime_statuses: Vec<sdkwork_api_app_extension::ExtensionRuntimeStatusRecord>,
}

struct ResolvedExtensionRuntimeReloadRequest {
    scope: ExtensionRuntimeReloadScope,
    requested_extension_id: Option<String>,
    requested_instance_id: Option<String>,
    resolved_extension_id: Option<String>,
    gateway_scope: ConfiguredExtensionHostReloadScope,
}

#[derive(Debug, Serialize)]
struct ExtensionRuntimeRolloutParticipantResponse {
    node_id: String,
    service_kind: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    updated_at_ms: u64,
}

#[derive(Debug, Serialize)]
struct ExtensionRuntimeRolloutResponse {
    rollout_id: String,
    status: String,
    scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    requested_extension_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    requested_instance_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resolved_extension_id: Option<String>,
    created_by: String,
    created_at_ms: u64,
    deadline_at_ms: u64,
    participant_count: usize,
    participants: Vec<ExtensionRuntimeRolloutParticipantResponse>,
}

impl From<ExtensionRuntimeRolloutDetails> for ExtensionRuntimeRolloutResponse {
    fn from(value: ExtensionRuntimeRolloutDetails) -> Self {
        Self {
            rollout_id: value.rollout_id,
            status: value.status,
            scope: value.scope,
            requested_extension_id: value.requested_extension_id,
            requested_instance_id: value.requested_instance_id,
            resolved_extension_id: value.resolved_extension_id,
            created_by: value.created_by,
            created_at_ms: value.created_at_ms,
            deadline_at_ms: value.deadline_at_ms,
            participant_count: value.participant_count,
            participants: value
                .participants
                .into_iter()
                .map(|participant| ExtensionRuntimeRolloutParticipantResponse {
                    node_id: participant.node_id,
                    service_kind: participant.service_kind,
                    status: participant.status,
                    message: participant.message,
                    updated_at_ms: participant.updated_at_ms,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Deserialize, Default)]
struct StandaloneConfigRolloutCreateRequest {
    #[serde(default)]
    service_kind: Option<String>,
    #[serde(default)]
    timeout_secs: Option<u64>,
}

#[derive(Debug, Serialize)]
struct StandaloneConfigRolloutParticipantResponse {
    node_id: String,
    service_kind: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    updated_at_ms: u64,
}

#[derive(Debug, Serialize)]
struct StandaloneConfigRolloutResponse {
    rollout_id: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    requested_service_kind: Option<String>,
    created_by: String,
    created_at_ms: u64,
    deadline_at_ms: u64,
    participant_count: usize,
    participants: Vec<StandaloneConfigRolloutParticipantResponse>,
}

impl From<StandaloneConfigRolloutDetails> for StandaloneConfigRolloutResponse {
    fn from(value: StandaloneConfigRolloutDetails) -> Self {
        Self {
            rollout_id: value.rollout_id,
            status: value.status,
            requested_service_kind: value.requested_service_kind,
            created_by: value.created_by,
            created_at_ms: value.created_at_ms,
            deadline_at_ms: value.deadline_at_ms,
            participant_count: value.participant_count,
            participants: value
                .participants
                .into_iter()
                .map(|participant| StandaloneConfigRolloutParticipantResponse {
                    node_id: participant.node_id,
                    service_kind: participant.service_kind,
                    status: participant.status,
                    message: participant.message,
                    updated_at_ms: participant.updated_at_ms,
                })
                .collect(),
        }
    }
}

pub fn admin_router() -> Router {
    let service_name: Arc<str> = Arc::from("admin");
    let metrics = Arc::new(HttpMetricsRegistry::new("admin"));
    Router::new()
        .route("/admin/openapi.json", get(admin_openapi_handler))
        .route("/admin/docs", get(admin_docs_handler))
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
        .route("/admin/health", get(|| async { "ok" }))
        .route("/admin/auth/login", post(|| async { "login" }))
        .route("/admin/auth/me", get(|| async { "me" }))
        .route(
            "/admin/auth/change-password",
            post(|| async { "change-password" }),
        )
        .route("/admin/tenants", get(|| async { "tenants" }))
        .route("/admin/projects", get(|| async { "projects" }))
        .route("/admin/api-keys", get(|| async { "api-keys" }))
        .route("/admin/channels", get(|| async { "channels" }))
        .route("/admin/providers", get(|| async { "providers" }))
        .route("/admin/credentials", get(|| async { "credentials" }))
        .route("/admin/channel-models", get(|| async { "channel-models" }))
        .route("/admin/models", get(|| async { "models" }))
        .route("/admin/model-prices", get(|| async { "model-prices" }))
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
        .route(
            "/admin/extensions/runtime-reloads",
            post(|| async { "extension-runtime-reloads" }),
        )
        .route(
            "/admin/runtime-config/rollouts",
            get(|| async { "runtime-config-rollouts" })
                .post(|| async { "runtime-config-rollouts-create" }),
        )
        .route(
            "/admin/runtime-config/rollouts/{rollout_id}",
            get(|| async { "runtime-config-rollout" }),
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
        .layer(axum::middleware::from_fn_with_state(
            metrics,
            observe_http_metrics,
        ))
        .layer(axum::middleware::from_fn_with_state(
            service_name,
            observe_http_tracing,
        ))
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
    admin_router_with_state(AdminApiState::with_store_and_secret_manager_and_jwt_secret(
        store,
        secret_manager,
        jwt_signing_secret,
    ))
}

pub fn admin_router_with_state(state: AdminApiState) -> Router {
    let service_name: Arc<str> = Arc::from("admin");
    let metrics = Arc::new(HttpMetricsRegistry::new("admin"));
    Router::new()
        .route("/admin/openapi.json", get(admin_openapi_handler))
        .route("/admin/docs", get(admin_docs_handler))
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
        .route("/admin/health", get(|| async { "ok" }))
        .route("/admin/auth/login", post(login_handler))
        .route("/admin/auth/me", get(me_handler))
        .route("/admin/auth/change-password", post(change_password_handler))
        .route(
            "/admin/users/operators",
            get(list_operator_users_handler).post(upsert_operator_user_handler),
        )
        .route(
            "/admin/users/operators/{user_id}",
            delete(delete_operator_user_handler),
        )
        .route(
            "/admin/users/operators/{user_id}/status",
            post(update_operator_user_status_handler),
        )
        .route(
            "/admin/users/operators/{user_id}/password",
            post(reset_operator_user_password_handler),
        )
        .route(
            "/admin/users/portal",
            get(list_portal_users_handler).post(upsert_portal_user_handler),
        )
        .route(
            "/admin/users/portal/{user_id}",
            delete(delete_portal_user_handler),
        )
        .route(
            "/admin/users/portal/{user_id}/status",
            post(update_portal_user_status_handler),
        )
        .route(
            "/admin/users/portal/{user_id}/password",
            post(reset_portal_user_password_handler),
        )
        .route(
            "/admin/coupons",
            get(list_coupons_handler).post(create_coupon_handler),
        )
        .route("/admin/coupons/{coupon_id}", delete(delete_coupon_handler))
        .route(
            "/admin/tenants",
            get(list_tenants_handler).post(create_tenant_handler),
        )
        .route("/admin/tenants/{tenant_id}", delete(delete_tenant_handler))
        .route(
            "/admin/projects",
            get(list_projects_handler).post(create_project_handler),
        )
        .route(
            "/admin/projects/{project_id}",
            delete(delete_project_handler),
        )
        .route(
            "/admin/api-keys",
            get(list_api_keys_handler).post(create_api_key_handler),
        )
        .route(
            "/admin/api-keys/{hashed_key}/status",
            post(update_api_key_status_handler),
        )
        .route(
            "/admin/api-keys/{hashed_key}",
            put(update_api_key_handler).delete(delete_api_key_handler),
        )
        .route(
            "/admin/channels",
            get(list_channels_handler).post(create_channel_handler),
        )
        .route(
            "/admin/channels/{channel_id}",
            delete(delete_channel_handler),
        )
        .route(
            "/admin/providers",
            get(list_providers_handler).post(create_provider_handler),
        )
        .route(
            "/admin/providers/{provider_id}",
            delete(delete_provider_handler),
        )
        .route(
            "/admin/credentials",
            get(list_credentials_handler).post(create_credential_handler),
        )
        .route(
            "/admin/credentials/{tenant_id}/providers/{provider_id}/keys/{key_reference}",
            delete(delete_credential_handler),
        )
        .route(
            "/admin/channel-models",
            get(list_channel_models_handler).post(create_channel_model_handler),
        )
        .route(
            "/admin/channel-models/{channel_id}/models/{model_id}",
            delete(delete_channel_model_handler),
        )
        .route(
            "/admin/models",
            get(list_models_handler).post(create_model_handler),
        )
        .route(
            "/admin/models/{external_name}/providers/{provider_id}",
            delete(delete_model_handler),
        )
        .route(
            "/admin/model-prices",
            get(list_model_prices_handler).post(create_model_price_handler),
        )
        .route(
            "/admin/model-prices/{channel_id}/models/{model_id}/providers/{proxy_provider_id}",
            delete(delete_model_price_handler),
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
        .route(
            "/admin/extensions/runtime-reloads",
            post(reload_extension_runtimes_handler),
        )
        .route(
            "/admin/extensions/runtime-rollouts",
            get(list_extension_runtime_rollouts_handler)
                .post(create_extension_runtime_rollout_handler),
        )
        .route(
            "/admin/extensions/runtime-rollouts/{rollout_id}",
            get(get_extension_runtime_rollout_handler),
        )
        .route(
            "/admin/runtime-config/rollouts",
            get(list_standalone_config_rollouts_handler)
                .post(create_standalone_config_rollout_handler),
        )
        .route(
            "/admin/runtime-config/rollouts/{rollout_id}",
            get(get_standalone_config_rollout_handler),
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

async fn login_handler(
    State(state): State<AdminApiState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    let session = login_admin_user(
        state.store.as_ref(),
        &request.email,
        &request.password,
        &state.jwt_signing_secret,
    )
    .await
    .map_err(admin_error_response)?;
    let token = session.token.clone();
    let claims = verify_jwt(&token, &state.jwt_signing_secret)
        .map_err(|error| admin_error_response(AdminIdentityError::Storage(error)))?;
    Ok(Json(LoginResponse {
        token,
        claims,
        user: session.user,
    }))
}

async fn me_handler(
    claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<AdminUserProfile>, StatusCode> {
    load_admin_user_profile(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(Json)
        .ok_or(StatusCode::UNAUTHORIZED)
}

async fn change_password_handler(
    claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<ChangePasswordRequest>,
) -> Result<Json<AdminUserProfile>, (StatusCode, Json<ErrorResponse>)> {
    change_admin_password(
        state.store.as_ref(),
        &claims.claims().sub,
        &request.current_password,
        &request.new_password,
    )
    .await
    .map(Json)
    .map_err(admin_error_response)
}

async fn list_operator_users_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<AdminUserProfile>>, (StatusCode, Json<ErrorResponse>)> {
    list_admin_user_profiles(state.store.as_ref())
        .await
        .map(Json)
        .map_err(admin_error_response)
}

async fn upsert_operator_user_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<UpsertOperatorUserRequest>,
) -> Result<(StatusCode, Json<AdminUserProfile>), (StatusCode, Json<ErrorResponse>)> {
    upsert_admin_user(
        state.store.as_ref(),
        request.id.as_deref(),
        &request.email,
        &request.display_name,
        request.password.as_deref(),
        request.active,
    )
    .await
    .map(|user| (StatusCode::CREATED, Json(user)))
    .map_err(admin_error_response)
}

async fn update_operator_user_status_handler(
    _claims: AuthenticatedAdminClaims,
    Path(user_id): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<UpdateUserStatusRequest>,
) -> Result<Json<AdminUserProfile>, (StatusCode, Json<ErrorResponse>)> {
    set_admin_user_active(state.store.as_ref(), &user_id, request.active)
        .await
        .map(Json)
        .map_err(admin_error_response)
}

async fn reset_operator_user_password_handler(
    _claims: AuthenticatedAdminClaims,
    Path(user_id): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<ResetUserPasswordRequest>,
) -> Result<Json<AdminUserProfile>, (StatusCode, Json<ErrorResponse>)> {
    reset_admin_user_password(state.store.as_ref(), &user_id, &request.new_password)
        .await
        .map(Json)
        .map_err(admin_error_response)
}

async fn delete_operator_user_handler(
    claims: AuthenticatedAdminClaims,
    Path(user_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    if claims.claims().sub == user_id {
        return Err(admin_error_response(AdminIdentityError::Protected(
            "current admin session cannot be deleted".to_owned(),
        )));
    }

    match delete_admin_user(state.store.as_ref(), &user_id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(admin_error_response(AdminIdentityError::NotFound(
            "admin user not found".to_owned(),
        ))),
        Err(error) => Err(admin_error_response(error)),
    }
}

async fn list_portal_users_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<PortalUserProfile>>, (StatusCode, Json<ErrorResponse>)> {
    list_portal_user_profiles(state.store.as_ref())
        .await
        .map(Json)
        .map_err(portal_admin_error_response)
}

async fn upsert_portal_user_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<UpsertPortalUserRequest>,
) -> Result<(StatusCode, Json<PortalUserProfile>), (StatusCode, Json<ErrorResponse>)> {
    upsert_portal_user(
        state.store.as_ref(),
        request.id.as_deref(),
        &request.email,
        &request.display_name,
        request.password.as_deref(),
        &request.workspace_tenant_id,
        &request.workspace_project_id,
        request.active,
    )
    .await
    .map(|user| (StatusCode::CREATED, Json(user)))
    .map_err(portal_admin_error_response)
}

async fn update_portal_user_status_handler(
    _claims: AuthenticatedAdminClaims,
    Path(user_id): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<UpdateUserStatusRequest>,
) -> Result<Json<PortalUserProfile>, (StatusCode, Json<ErrorResponse>)> {
    set_portal_user_active(state.store.as_ref(), &user_id, request.active)
        .await
        .map(Json)
        .map_err(portal_admin_error_response)
}

async fn reset_portal_user_password_handler(
    _claims: AuthenticatedAdminClaims,
    Path(user_id): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<ResetUserPasswordRequest>,
) -> Result<Json<PortalUserProfile>, (StatusCode, Json<ErrorResponse>)> {
    reset_portal_user_password(state.store.as_ref(), &user_id, &request.new_password)
        .await
        .map(Json)
        .map_err(portal_admin_error_response)
}

async fn delete_portal_user_handler(
    _claims: AuthenticatedAdminClaims,
    Path(user_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    match delete_portal_user(state.store.as_ref(), &user_id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(portal_admin_error_response(PortalIdentityError::NotFound(
            "portal user not found".to_owned(),
        ))),
        Err(error) => Err(portal_admin_error_response(error)),
    }
}

async fn list_coupons_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CouponCampaign>>, StatusCode> {
    list_coupons(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_coupon_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateCouponRequest>,
) -> Result<(StatusCode, Json<CouponCampaign>), StatusCode> {
    let coupon = persist_coupon(
        state.store.as_ref(),
        &CouponCampaign::new(
            &request.id,
            &request.code,
            &request.discount_label,
            &request.audience,
            request.remaining,
            request.active,
            &request.note,
            &request.expires_on,
        ),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(coupon)))
}

async fn delete_coupon_handler(
    _claims: AuthenticatedAdminClaims,
    Path(coupon_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<StatusCode, StatusCode> {
    let deleted = delete_coupon(state.store.as_ref(), &coupon_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
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

fn admin_error_response(error: AdminIdentityError) -> (StatusCode, Json<ErrorResponse>) {
    let status = match error {
        AdminIdentityError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        AdminIdentityError::DuplicateEmail => StatusCode::CONFLICT,
        AdminIdentityError::Protected(_) => StatusCode::CONFLICT,
        AdminIdentityError::InvalidCredentials | AdminIdentityError::InactiveUser => {
            StatusCode::UNAUTHORIZED
        }
        AdminIdentityError::NotFound(_) => StatusCode::NOT_FOUND,
        AdminIdentityError::Storage(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };
    let body = ErrorResponse {
        error: ErrorBody {
            message: error.to_string(),
        },
    };
    (status, Json(body))
}

fn portal_admin_error_response(error: PortalIdentityError) -> (StatusCode, Json<ErrorResponse>) {
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

async fn delete_channel_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path(channel_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let deleted = delete_catalog_channel(state.store.as_ref(), &channel_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
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
        PersistProviderWithBindingsRequest {
            id: &request.id,
            channel_id: primary_channel_id,
            adapter_kind: &request.adapter_kind,
            extension_id: request.extension_id.as_deref(),
            base_url: &request.base_url,
            display_name: &request.display_name,
            channel_bindings: &bindings,
        },
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(provider)))
}

async fn delete_provider_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path(provider_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let provider_exists = state
        .store
        .find_provider(&provider_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .is_some();
    if !provider_exists {
        return Err(StatusCode::NOT_FOUND);
    }

    delete_provider_credentials_with_manager(
        state.store.as_ref(),
        &state.secret_manager,
        &provider_id,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let deleted = delete_catalog_provider(state.store.as_ref(), &provider_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
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

async fn delete_credential_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path((tenant_id, provider_id, key_reference)): Path<(String, String, String)>,
) -> Result<StatusCode, StatusCode> {
    let deleted = delete_credential_with_manager(
        state.store.as_ref(),
        &state.secret_manager,
        &tenant_id,
        &provider_id,
        &key_reference,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn list_channel_models_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<ChannelModelRecord>>, StatusCode> {
    list_channel_models(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_channel_model_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateChannelModelRequest>,
) -> Result<(StatusCode, Json<ChannelModelRecord>), StatusCode> {
    let record = persist_channel_model_with_metadata(
        state.store.as_ref(),
        &request.channel_id,
        &request.model_id,
        &request.model_display_name,
        &request.capabilities,
        request.streaming,
        request.context_window,
        request.description.as_deref(),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(record)))
}

async fn delete_channel_model_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path((channel_id, model_id)): Path<(String, String)>,
) -> Result<StatusCode, StatusCode> {
    let deleted = delete_catalog_channel_model(state.store.as_ref(), &channel_id, &model_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
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

async fn delete_model_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path((external_name, provider_id)): Path<(String, String)>,
) -> Result<StatusCode, StatusCode> {
    let deleted = delete_model_variant(state.store.as_ref(), &external_name, &provider_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn list_model_prices_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<ModelPriceRecord>>, StatusCode> {
    list_model_prices(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_model_price_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateModelPriceRequest>,
) -> Result<(StatusCode, Json<ModelPriceRecord>), StatusCode> {
    let record = persist_model_price_with_rates(
        state.store.as_ref(),
        &request.channel_id,
        &request.model_id,
        &request.proxy_provider_id,
        &request.currency_code,
        &request.price_unit,
        request.input_price,
        request.output_price,
        request.cache_read_price,
        request.cache_write_price,
        request.request_price,
        request.is_active,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(record)))
}

async fn delete_model_price_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path((channel_id, model_id, proxy_provider_id)): Path<(String, String, String)>,
) -> Result<StatusCode, StatusCode> {
    let deleted = delete_catalog_model_price(
        state.store.as_ref(),
        &channel_id,
        &model_id,
        &proxy_provider_id,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
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

async fn delete_tenant_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path(tenant_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let tenant_exists = state
        .store
        .list_tenants()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .any(|tenant| tenant.id == tenant_id);
    if !tenant_exists {
        return Err(StatusCode::NOT_FOUND);
    }

    delete_tenant_credentials_with_manager(state.store.as_ref(), &state.secret_manager, &tenant_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let deleted = delete_workspace_tenant(state.store.as_ref(), &tenant_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
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

async fn delete_project_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path(project_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let deleted = delete_tenant_project(state.store.as_ref(), &project_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
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
    let metadata_label = request
        .label
        .as_deref()
        .map(str::trim)
        .filter(|label| !label.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| format!("{} gateway key", request.environment.trim()));
    let created = if request.label.is_some()
        || request.expires_at_ms.is_some()
        || request.notes.is_some()
        || request.plaintext_key.is_some()
    {
        sdkwork_api_app_identity::persist_gateway_api_key_with_metadata(
            state.store.as_ref(),
            &request.tenant_id,
            &request.project_id,
            &request.environment,
            &metadata_label,
            request.expires_at_ms,
            request.plaintext_key.as_deref(),
            request.notes.as_deref(),
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        persist_gateway_api_key(
            state.store.as_ref(),
            &request.tenant_id,
            &request.project_id,
            &request.environment,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };
    Ok((StatusCode::CREATED, Json(created)))
}

async fn update_api_key_handler(
    _claims: AuthenticatedAdminClaims,
    Path(hashed_key): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<UpdateApiKeyRequest>,
) -> Result<Json<GatewayApiKeyRecord>, (StatusCode, Json<ErrorResponse>)> {
    match update_gateway_api_key_metadata(
        state.store.as_ref(),
        &hashed_key,
        &request.tenant_id,
        &request.project_id,
        &request.environment,
        &request.label,
        request.expires_at_ms,
        request.notes.as_deref(),
    )
    .await
    {
        Ok(Some(record)) => Ok(Json(record)),
        Ok(None) => Err(admin_error_response(AdminIdentityError::NotFound(
            "gateway api key not found".to_owned(),
        ))),
        Err(error) => Err(admin_error_response(error)),
    }
}

async fn update_api_key_status_handler(
    _claims: AuthenticatedAdminClaims,
    Path(hashed_key): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<UpdateUserStatusRequest>,
) -> Result<Json<GatewayApiKeyRecord>, StatusCode> {
    match set_gateway_api_key_active(state.store.as_ref(), &hashed_key, request.active)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        Some(record) => Ok(Json(record)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn delete_api_key_handler(
    _claims: AuthenticatedAdminClaims,
    Path(hashed_key): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<StatusCode, StatusCode> {
    let deleted = delete_gateway_api_key(state.store.as_ref(), &hashed_key)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
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
    _state: State<AdminApiState>,
) -> Result<Json<Vec<sdkwork_api_app_extension::DiscoveredExtensionPackageRecord>>, StatusCode> {
    let policy = configured_extension_discovery_policy_from_env();
    list_discovered_extension_packages(&policy)
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

async fn reload_extension_runtimes_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    body: Bytes,
) -> Result<Json<ExtensionRuntimeReloadResponse>, StatusCode> {
    let request = parse_extension_runtime_reload_request(&body)?;
    let resolved = resolve_extension_runtime_reload_request(state.store.as_ref(), request).await?;
    let report = reload_extension_host_with_scope(&resolved.gateway_scope)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let runtime_statuses =
        list_extension_runtime_statuses().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ExtensionRuntimeReloadResponse {
        scope: resolved.scope,
        requested_extension_id: resolved.requested_extension_id,
        requested_instance_id: resolved.requested_instance_id,
        resolved_extension_id: resolved.resolved_extension_id,
        discovered_package_count: report.discovered_package_count,
        loadable_package_count: report.loadable_package_count,
        active_runtime_count: runtime_statuses.len(),
        reloaded_at_ms: unix_timestamp_ms(),
        runtime_statuses,
    }))
}

async fn create_extension_runtime_rollout_handler(
    claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    body: Bytes,
) -> Result<(StatusCode, Json<ExtensionRuntimeRolloutResponse>), StatusCode> {
    let request = parse_extension_runtime_rollout_create_request(&body)?;
    let resolved = resolve_extension_runtime_reload_request(
        state.store.as_ref(),
        ExtensionRuntimeReloadRequest {
            extension_id: request.extension_id,
            instance_id: request.instance_id,
        },
    )
    .await?;

    let rollout = create_extension_runtime_rollout_with_request(
        state.store.as_ref(),
        &claims.claims().sub,
        CreateExtensionRuntimeRolloutRequest {
            scope: resolved.gateway_scope,
            requested_extension_id: resolved.requested_extension_id,
            requested_instance_id: resolved.requested_instance_id,
            resolved_extension_id: resolved.resolved_extension_id,
            timeout_secs: request.timeout_secs.unwrap_or(30),
        },
    )
    .await
    .map_err(map_extension_runtime_rollout_creation_error)?;

    Ok((StatusCode::CREATED, Json(rollout.into())))
}

async fn create_standalone_config_rollout_handler(
    claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    body: Bytes,
) -> Result<(StatusCode, Json<StandaloneConfigRolloutResponse>), StatusCode> {
    let request = parse_standalone_config_rollout_create_request(&body)?;
    let requested_service_kind = validate_standalone_service_kind(request.service_kind)?;
    let rollout = create_standalone_config_rollout(
        state.store.as_ref(),
        &claims.claims().sub,
        CreateStandaloneConfigRolloutRequest::new(
            requested_service_kind,
            request.timeout_secs.unwrap_or(30),
        ),
    )
    .await
    .map_err(map_standalone_config_rollout_creation_error)?;

    Ok((StatusCode::CREATED, Json(rollout.into())))
}

async fn list_extension_runtime_rollouts_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<ExtensionRuntimeRolloutResponse>>, StatusCode> {
    list_extension_runtime_rollouts(state.store.as_ref())
        .await
        .map(|rollouts| {
            Json(
                rollouts
                    .into_iter()
                    .map(ExtensionRuntimeRolloutResponse::from)
                    .collect(),
            )
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn list_standalone_config_rollouts_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<StandaloneConfigRolloutResponse>>, StatusCode> {
    list_standalone_config_rollouts(state.store.as_ref())
        .await
        .map(|rollouts| {
            Json(
                rollouts
                    .into_iter()
                    .map(StandaloneConfigRolloutResponse::from)
                    .collect(),
            )
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn get_extension_runtime_rollout_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path(rollout_id): Path<String>,
) -> Result<Json<ExtensionRuntimeRolloutResponse>, StatusCode> {
    let rollout = find_extension_runtime_rollout(state.store.as_ref(), &rollout_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(rollout.into()))
}

async fn get_standalone_config_rollout_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Path(rollout_id): Path<String>,
) -> Result<Json<StandaloneConfigRolloutResponse>, StatusCode> {
    let rollout = find_standalone_config_rollout(state.store.as_ref(), &rollout_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(rollout.into()))
}

fn parse_extension_runtime_reload_request(
    body: &[u8],
) -> Result<ExtensionRuntimeReloadRequest, StatusCode> {
    if body.is_empty() {
        return Ok(ExtensionRuntimeReloadRequest::default());
    }

    serde_json::from_slice(body).map_err(|_| StatusCode::BAD_REQUEST)
}

fn parse_extension_runtime_rollout_create_request(
    body: &[u8],
) -> Result<ExtensionRuntimeRolloutCreateRequest, StatusCode> {
    if body.is_empty() {
        return Ok(ExtensionRuntimeRolloutCreateRequest::default());
    }

    serde_json::from_slice(body).map_err(|_| StatusCode::BAD_REQUEST)
}

fn parse_standalone_config_rollout_create_request(
    body: &[u8],
) -> Result<StandaloneConfigRolloutCreateRequest, StatusCode> {
    if body.is_empty() {
        return Ok(StandaloneConfigRolloutCreateRequest::default());
    }

    serde_json::from_slice(body).map_err(|_| StatusCode::BAD_REQUEST)
}

fn map_extension_runtime_rollout_creation_error(error: anyhow::Error) -> StatusCode {
    if error
        .to_string()
        .contains("no active gateway or admin nodes available")
    {
        StatusCode::CONFLICT
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

fn map_standalone_config_rollout_creation_error(error: anyhow::Error) -> StatusCode {
    if error
        .to_string()
        .contains("no active standalone nodes available")
    {
        StatusCode::CONFLICT
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

async fn resolve_extension_runtime_reload_request(
    store: &dyn AdminStore,
    request: ExtensionRuntimeReloadRequest,
) -> Result<ResolvedExtensionRuntimeReloadRequest, StatusCode> {
    let extension_id = validate_reload_identifier(request.extension_id)?;
    let instance_id = validate_reload_identifier(request.instance_id)?;

    match (extension_id, instance_id) {
        (Some(_), Some(_)) => Err(StatusCode::BAD_REQUEST),
        (Some(extension_id), None) => Ok(ResolvedExtensionRuntimeReloadRequest {
            scope: ExtensionRuntimeReloadScope::Extension,
            requested_extension_id: Some(extension_id.clone()),
            requested_instance_id: None,
            resolved_extension_id: Some(extension_id.clone()),
            gateway_scope: ConfiguredExtensionHostReloadScope::Extension { extension_id },
        }),
        (None, Some(instance_id)) => {
            let instance = list_extension_instances(store)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                .into_iter()
                .find(|instance| instance.instance_id == instance_id)
                .ok_or(StatusCode::BAD_REQUEST)?;
            let installation = list_extension_installations(store)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                .into_iter()
                .find(|installation| installation.installation_id == instance.installation_id)
                .ok_or(StatusCode::BAD_REQUEST)?;
            let resolved_extension_id = installation.extension_id.clone();

            let (scope, gateway_scope) = match installation.runtime {
                ExtensionRuntime::Connector => (
                    ExtensionRuntimeReloadScope::Instance,
                    ConfiguredExtensionHostReloadScope::Instance {
                        instance_id: instance_id.clone(),
                    },
                ),
                ExtensionRuntime::Builtin | ExtensionRuntime::NativeDynamic => (
                    ExtensionRuntimeReloadScope::Extension,
                    ConfiguredExtensionHostReloadScope::Extension {
                        extension_id: resolved_extension_id.clone(),
                    },
                ),
            };

            Ok(ResolvedExtensionRuntimeReloadRequest {
                scope,
                requested_extension_id: None,
                requested_instance_id: Some(instance_id),
                resolved_extension_id: Some(resolved_extension_id),
                gateway_scope,
            })
        }
        (None, None) => Ok(ResolvedExtensionRuntimeReloadRequest {
            scope: ExtensionRuntimeReloadScope::All,
            requested_extension_id: None,
            requested_instance_id: None,
            resolved_extension_id: None,
            gateway_scope: ConfiguredExtensionHostReloadScope::All,
        }),
    }
}

fn validate_reload_identifier(value: Option<String>) -> Result<Option<String>, StatusCode> {
    match value {
        Some(value) => {
            let value = value.trim();
            if value.is_empty() {
                Err(StatusCode::BAD_REQUEST)
            } else {
                Ok(Some(value.to_owned()))
            }
        }
        None => Ok(None),
    }
}

fn validate_standalone_service_kind(value: Option<String>) -> Result<Option<String>, StatusCode> {
    match value {
        Some(value) => {
            let value = value.trim();
            if value.is_empty() {
                return Err(StatusCode::BAD_REQUEST);
            }

            match value {
                "gateway" | "admin" | "portal" => Ok(Some(value.to_owned())),
                _ => Err(StatusCode::BAD_REQUEST),
            }
        }
        None => Ok(None),
    }
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

fn unix_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("unix time")
        .as_millis() as u64
}

async fn simulate_routing_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<RoutingSimulationRequest>,
) -> Result<Json<RoutingSimulationResponse>, StatusCode> {
    let decision = select_route_with_store_context(
        state.store.as_ref(),
        &request.capability,
        &request.model,
        RouteSelectionContext::new(RoutingDecisionSource::AdminSimulation)
            .with_requested_region_option(request.requested_region.as_deref())
            .with_selection_seed_option(request.selection_seed),
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
        requested_region: decision.requested_region,
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
