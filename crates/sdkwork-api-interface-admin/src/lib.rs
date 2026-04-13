use anyhow::ensure;
use std::collections::BTreeMap;
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

mod auth;
mod billing;
mod catalog;
mod commerce;
mod gateway;
mod jobs;
mod marketing;
mod openapi;
mod payments;
mod pricing;
mod routes;
mod routing;
mod runtime;
mod tenant;
mod types;
mod users;

use axum::{
    body::Bytes,
    extract::FromRequestParts,
    extract::Path,
    extract::Query,
    extract::State,
    http::header,
    http::request::Parts,
    http::HeaderMap,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{delete, get, patch, post, put},
    Json, Router,
};
use sdkwork_api_app_billing::{
    create_quota_policy, list_billing_events, list_ledger_entries, list_quota_policies,
    persist_quota_policy, summarize_billing_events_from_store, summarize_billing_from_store,
    synchronize_due_pricing_plan_lifecycle, synchronize_due_pricing_plan_lifecycle_with_report,
    AccountBalanceSnapshot, AccountLedgerHistoryEntry, CommercialBillingAdminKernel,
    PricingLifecycleSynchronizationReport,
};
use sdkwork_api_app_catalog::{
    delete_channel as delete_catalog_channel, delete_channel_model as delete_catalog_channel_model,
    delete_model_price as delete_catalog_model_price, delete_model_variant,
    delete_provider as delete_catalog_provider,
    delete_provider_account as delete_catalog_provider_account, delete_provider_model,
    list_channel_models, list_channels, list_model_entries, list_model_prices,
    list_provider_accounts as list_catalog_provider_accounts, list_provider_models, list_providers,
    normalize_provider_integration, persist_channel, persist_channel_model_with_metadata,
    persist_model_price_with_rates_and_metadata, persist_model_with_metadata,
    persist_provider_account, persist_provider_model_with_metadata,
    persist_provider_with_bindings_and_extension_id, PersistProviderWithBindingsRequest,
};
use sdkwork_api_app_commerce::{
    AdminCommerceReconciliationRunCreateRequest, AdminCommerceRefundCreateRequest, CommerceError,
};
use sdkwork_api_app_credential::CredentialSecretManager;
use sdkwork_api_app_credential::{
    delete_credential_with_manager, delete_provider_credentials_with_manager,
    delete_tenant_credentials_with_manager, list_credentials, list_official_provider_configs,
    official_provider_secret_configured, persist_credential_with_secret_and_manager,
    persist_official_provider_config_with_secret_and_manager,
};
use sdkwork_api_app_extension::{
    configured_extension_discovery_policy_from_env, list_discovered_extension_packages,
    list_extension_installations, list_extension_instances, list_extension_runtime_statuses,
    list_provider_health_snapshots, persist_extension_installation, persist_extension_instance,
    PersistExtensionInstanceInput,
};
use sdkwork_api_app_gateway::{
    inspect_provider_execution_views, invalidate_capability_catalog_cache,
    reload_extension_host_with_scope, ConfiguredExtensionHostReloadScope,
};
use sdkwork_api_app_identity::{
    change_admin_password, create_api_key_group, delete_admin_user, delete_api_key_group,
    delete_gateway_api_key, delete_portal_user, list_admin_user_profiles, list_api_key_groups,
    list_gateway_api_keys, list_portal_user_profiles, load_admin_user_profile, login_admin_user,
    reset_admin_user_password, reset_portal_user_password, set_admin_user_active,
    set_api_key_group_active, set_gateway_api_key_active, set_portal_user_active,
    update_api_key_group, update_gateway_api_key_metadata, upsert_admin_user, upsert_portal_user,
    verify_jwt, AdminIdentityError, ApiKeyGroupInput, Claims, CreatedGatewayApiKey,
    PortalIdentityError,
};
use sdkwork_api_app_jobs::{
    find_async_job, list_async_job_assets, list_async_job_attempts, list_async_job_callbacks,
    list_async_jobs,
};
use sdkwork_api_app_payment::{
    approve_refund_order_request, cancel_refund_order_request, load_admin_payment_order_dossier,
    start_refund_order_execution, AdminPaymentOrderDossier,
};
use sdkwork_api_app_rate_limit::{
    create_rate_limit_policy, list_rate_limit_policies, persist_rate_limit_policy,
};
use sdkwork_api_app_routing::{
    create_routing_policy, create_routing_profile, list_compiled_routing_snapshots,
    list_routing_decision_logs, list_routing_policies, list_routing_profiles,
    persist_routing_policy, persist_routing_profile, select_route_with_store_context,
    CreateRoutingPolicyInput, CreateRoutingProfileInput, RouteSelectionContext,
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
use sdkwork_api_config::HttpExposureConfig;
use sdkwork_api_domain_billing::{
    AccountBenefitLotRecord, AccountHoldRecord, AccountRecord, BillingEventRecord,
    BillingEventSummary, BillingSummary, LedgerEntry, PricingPlanRecord, PricingRateRecord,
    QuotaPolicy, RequestSettlementRecord,
};
use sdkwork_api_domain_catalog::{
    Channel, ChannelModelRecord, ModelCapability, ModelCatalogEntry, ModelPriceRecord,
    ModelPriceTier, ProviderAccountRecord, ProviderChannelBinding, ProviderModelRecord,
    ProxyProvider,
};
use sdkwork_api_domain_commerce::{
    CommerceOrderRecord, CommercePaymentAttemptRecord, CommercePaymentEventRecord,
    CommerceReconciliationItemRecord, CommerceReconciliationRunRecord, CommerceRefundRecord,
    CommerceWebhookDeliveryAttemptRecord, CommerceWebhookInboxRecord,
    PaymentMethodCredentialBindingRecord, PaymentMethodRecord,
};
use sdkwork_api_domain_credential::{OfficialProviderConfig, UpstreamCredential};
use sdkwork_api_domain_identity::{
    AdminUserProfile, ApiKeyGroupRecord, GatewayApiKeyRecord, PortalUserProfile,
};
use sdkwork_api_domain_jobs::{
    AsyncJobAssetRecord, AsyncJobAttemptRecord, AsyncJobCallbackRecord, AsyncJobRecord,
};
use sdkwork_api_domain_marketing::{
    CampaignBudgetLifecycleAction, CampaignBudgetLifecycleAuditOutcome,
    CampaignBudgetLifecycleAuditRecord, CampaignBudgetRecord, CampaignBudgetStatus,
    CouponCodeLifecycleAction, CouponCodeLifecycleAuditOutcome,
    CouponCodeLifecycleAuditRecord, CouponCodeRecord, CouponCodeStatus,
    CouponRedemptionRecord, CouponReservationRecord, CouponRollbackRecord,
    CouponTemplateApprovalState,
    CouponTemplateLifecycleAuditOutcome, CouponTemplateLifecycleAuditRecord,
    CouponTemplateLifecycleAction, CouponTemplateRecord, CouponTemplateStatus,
    MarketingCampaignLifecycleAction,
    MarketingCampaignLifecycleAuditOutcome, MarketingCampaignLifecycleAuditRecord,
    MarketingCampaignApprovalState, MarketingCampaignRecord, MarketingCampaignStatus,
};
use sdkwork_api_domain_payment::{
    PaymentChannelPolicyRecord, PaymentGatewayAccountRecord, PaymentOrderRecord,
    PaymentProviderCode, ReconciliationMatchStatus, ReconciliationMatchSummaryRecord,
    RefundOrderRecord, RefundOrderStatus,
};
use sdkwork_api_domain_rate_limit::{RateLimitPolicy, RateLimitWindowSnapshot};
use sdkwork_api_domain_routing::{
    CompiledRoutingSnapshotRecord, ProviderHealthSnapshot, RoutingCandidateAssessment,
    RoutingDecisionLog, RoutingDecisionSource, RoutingPolicy, RoutingProfileRecord,
    RoutingStrategy,
};
use sdkwork_api_domain_tenant::{Project, Tenant};
use sdkwork_api_domain_usage::{UsageRecord, UsageSummary};
use sdkwork_api_extension_core::{ExtensionInstallation, ExtensionInstance, ExtensionRuntime};
use sdkwork_api_observability::{
    observe_http_metrics, observe_http_tracing, HttpMetricsRegistry,
};
use sdkwork_api_storage_core::{AdminStore, CommercialKernelStore, Reloadable};
use sdkwork_api_storage_postgres::PostgresAdminStore;
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::openapi::Server;
use utoipa::{Modify, OpenApi, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::{Config as SwaggerUiConfig, SwaggerUi, Url as SwaggerUiUrl};

use commerce::{
    CommerceOrderAuditRecord, CommercialCatalogPublicationDetail,
    CommercialCatalogPublicationMutationResult, CommercialCatalogPublicationProjection,
};
use types::*;

pub use routes::{
    admin_router, admin_router_with_pool, admin_router_with_pool_and_master_key,
    admin_router_with_pool_and_secret_manager, admin_router_with_state,
    admin_router_with_state_and_http_exposure, admin_router_with_store,
    admin_router_with_store_and_secret_manager,
    admin_router_with_store_and_secret_manager_and_jwt_secret, try_admin_router,
    try_admin_router_with_state,
};

const DEFAULT_ADMIN_JWT_SIGNING_SECRET: &str = "local-dev-admin-jwt-secret";
static ADMIN_PRICING_ID_SEQUENCE: AtomicU64 = AtomicU64::new(1);

pub struct AdminApiState {
    live_store: Reloadable<Arc<dyn AdminStore>>,
    live_secret_manager: Reloadable<CredentialSecretManager>,
    live_commercial_billing: Option<Reloadable<Arc<dyn CommercialBillingAdminKernel>>>,
    live_payment_store: Option<Reloadable<Arc<dyn CommercialKernelStore>>>,
    store: Arc<dyn AdminStore>,
    secret_manager: CredentialSecretManager,
    commercial_billing: Option<Arc<dyn CommercialBillingAdminKernel>>,
    payment_store: Option<Arc<dyn CommercialKernelStore>>,
    live_jwt_signing_secret: Reloadable<String>,
    jwt_signing_secret: String,
}

impl Clone for AdminApiState {
    fn clone(&self) -> Self {
        Self {
            live_store: self.live_store.clone(),
            live_secret_manager: self.live_secret_manager.clone(),
            live_commercial_billing: self.live_commercial_billing.clone(),
            live_payment_store: self.live_payment_store.clone(),
            store: self.live_store.snapshot(),
            secret_manager: self.live_secret_manager.snapshot(),
            commercial_billing: self
                .live_commercial_billing
                .as_ref()
                .map(Reloadable::snapshot)
                .or_else(|| self.commercial_billing.clone()),
            payment_store: self
                .live_payment_store
                .as_ref()
                .map(Reloadable::snapshot)
                .or_else(|| self.payment_store.clone()),
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
        let store = Arc::new(SqliteAdminStore::new(pool));
        let admin_store: Arc<dyn AdminStore> = store.clone();
        let commercial_billing: Arc<dyn CommercialBillingAdminKernel> = store.clone();
        let payment_store: Arc<dyn CommercialKernelStore> = store;
        Self::with_store_and_secret_manager_and_jwt_secret_and_kernel_handles(
            admin_store,
            CredentialSecretManager::database_encrypted(credential_master_key),
            DEFAULT_ADMIN_JWT_SIGNING_SECRET,
            Some(commercial_billing),
            Some(payment_store),
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
        let store = Arc::new(SqliteAdminStore::new(pool));
        let admin_store: Arc<dyn AdminStore> = store.clone();
        let commercial_billing: Arc<dyn CommercialBillingAdminKernel> = store.clone();
        let payment_store: Arc<dyn CommercialKernelStore> = store;
        Self::with_store_and_secret_manager_and_jwt_secret_and_kernel_handles(
            admin_store,
            secret_manager,
            jwt_signing_secret,
            Some(commercial_billing),
            Some(payment_store),
        )
    }

    pub fn with_store_and_secret_manager(
        store: Arc<dyn AdminStore>,
        secret_manager: CredentialSecretManager,
    ) -> Self {
        Self::with_store_and_secret_manager_and_jwt_secret_and_kernel_handles(
            store,
            secret_manager,
            DEFAULT_ADMIN_JWT_SIGNING_SECRET,
            None,
            None,
        )
    }

    pub fn with_store_and_secret_manager_and_jwt_secret(
        store: Arc<dyn AdminStore>,
        secret_manager: CredentialSecretManager,
        jwt_signing_secret: impl Into<String>,
    ) -> Self {
        Self::with_store_and_secret_manager_and_jwt_secret_and_kernel_handles(
            store,
            secret_manager,
            jwt_signing_secret,
            None,
            None,
        )
    }

    pub fn with_store_and_secret_manager_and_jwt_secret_and_commercial_billing(
        store: Arc<dyn AdminStore>,
        secret_manager: CredentialSecretManager,
        jwt_signing_secret: impl Into<String>,
        commercial_billing: Option<Arc<dyn CommercialBillingAdminKernel>>,
    ) -> Self {
        Self::with_store_and_secret_manager_and_jwt_secret_and_kernel_handles(
            store,
            secret_manager,
            jwt_signing_secret,
            commercial_billing,
            None,
        )
    }

    fn with_store_and_secret_manager_and_jwt_secret_and_kernel_handles(
        store: Arc<dyn AdminStore>,
        secret_manager: CredentialSecretManager,
        jwt_signing_secret: impl Into<String>,
        commercial_billing: Option<Arc<dyn CommercialBillingAdminKernel>>,
        payment_store: Option<Arc<dyn CommercialKernelStore>>,
    ) -> Self {
        Self::with_live_store_and_secret_manager_handle_and_commercial_billing_payment_store_and_jwt_secret_handle(
            Reloadable::new(store),
            Reloadable::new(secret_manager),
            commercial_billing.map(Reloadable::new),
            payment_store.map(Reloadable::new),
            Reloadable::new(jwt_signing_secret.into()),
        )
    }

    pub fn with_live_store_and_secret_manager_and_jwt_secret_handle(
        live_store: Reloadable<Arc<dyn AdminStore>>,
        secret_manager: CredentialSecretManager,
        live_jwt_signing_secret: Reloadable<String>,
    ) -> Self {
        Self::with_live_store_and_secret_manager_handle_and_commercial_billing_payment_store_and_jwt_secret_handle(
            live_store,
            Reloadable::new(secret_manager),
            None,
            None,
            live_jwt_signing_secret,
        )
    }

    pub fn with_live_store_and_secret_manager_handle_and_jwt_secret_handle(
        live_store: Reloadable<Arc<dyn AdminStore>>,
        live_secret_manager: Reloadable<CredentialSecretManager>,
        live_jwt_signing_secret: Reloadable<String>,
    ) -> Self {
        Self::with_live_store_and_secret_manager_handle_and_commercial_billing_payment_store_and_jwt_secret_handle(
            live_store,
            live_secret_manager,
            None,
            None,
            live_jwt_signing_secret,
        )
    }

    pub fn with_live_store_and_secret_manager_handle_and_commercial_billing_and_jwt_secret_handle(
        live_store: Reloadable<Arc<dyn AdminStore>>,
        live_secret_manager: Reloadable<CredentialSecretManager>,
        live_commercial_billing: Option<Reloadable<Arc<dyn CommercialBillingAdminKernel>>>,
        live_jwt_signing_secret: Reloadable<String>,
    ) -> Self {
        Self::with_live_store_and_secret_manager_handle_and_commercial_billing_payment_store_and_jwt_secret_handle(
            live_store,
            live_secret_manager,
            live_commercial_billing,
            None,
            live_jwt_signing_secret,
        )
    }

    pub fn with_live_store_and_secret_manager_handle_and_commercial_billing_payment_store_and_jwt_secret_handle(
        live_store: Reloadable<Arc<dyn AdminStore>>,
        live_secret_manager: Reloadable<CredentialSecretManager>,
        live_commercial_billing: Option<Reloadable<Arc<dyn CommercialBillingAdminKernel>>>,
        live_payment_store: Option<Reloadable<Arc<dyn CommercialKernelStore>>>,
        live_jwt_signing_secret: Reloadable<String>,
    ) -> Self {
        Self {
            store: live_store.snapshot(),
            secret_manager: live_secret_manager.snapshot(),
            live_store,
            live_secret_manager,
            commercial_billing: live_commercial_billing.as_ref().map(Reloadable::snapshot),
            live_commercial_billing,
            payment_store: live_payment_store.as_ref().map(Reloadable::snapshot),
            live_payment_store,
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

#[derive(Debug, Serialize)]
struct RoutingSimulationResponse {
    selected_provider_id: String,
    candidate_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    matched_policy_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    applied_routing_profile_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    compiled_routing_snapshot_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    strategy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    selection_seed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    selection_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fallback_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    requested_region: Option<String>,
    #[serde(default)]
    slo_applied: bool,
    #[serde(default)]
    slo_degraded: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    selected_candidate: Option<RoutingCandidateAssessment>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    rejected_candidates: Vec<RoutingCandidateAssessment>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    assessments: Vec<RoutingCandidateAssessment>,
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

async fn invalidate_catalog_cache_after_mutation() {
    invalidate_capability_catalog_cache().await;
}

fn gateway_api_key_create_error_response(
    error: anyhow::Error,
) -> (StatusCode, Json<ErrorResponse>) {
    let message = error.to_string();
    let status = if gateway::looks_like_gateway_api_key_input_error(&message) {
        StatusCode::BAD_REQUEST
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };
    let body = ErrorResponse {
        error: ErrorBody { message },
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

fn error_response(
    status: StatusCode,
    message: impl Into<String>,
) -> (StatusCode, Json<ErrorResponse>) {
    (
        status,
        Json(ErrorResponse {
            error: ErrorBody {
                message: message.into(),
            },
        }),
    )
}

async fn create_channel_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateChannelRequest>,
) -> Result<(StatusCode, Json<Channel>), StatusCode> {
    let channel = persist_channel(state.store.as_ref(), &request.id, &request.name)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    invalidate_catalog_cache_after_mutation().await;
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
        invalidate_catalog_cache_after_mutation().await;
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
    let normalized = normalize_provider_integration(
        request.adapter_kind.as_deref(),
        request.protocol_kind.as_deref(),
        request.extension_id.as_deref(),
        request.default_plugin_family.as_deref(),
    )
    .map_err(|_| StatusCode::BAD_REQUEST)?;
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
            adapter_kind: &normalized.adapter_kind,
            protocol_kind: normalized.protocol_kind.as_deref(),
            extension_id: normalized.extension_id.as_deref(),
            base_url: &request.base_url,
            display_name: &request.display_name,
            channel_bindings: &bindings,
        },
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    invalidate_catalog_cache_after_mutation().await;
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
        invalidate_catalog_cache_after_mutation().await;
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

fn commercial_billing_kernel(
    state: &AdminApiState,
) -> Result<&Arc<dyn CommercialBillingAdminKernel>, (StatusCode, Json<ErrorResponse>)> {
    state.commercial_billing.as_ref().ok_or_else(|| {
        error_response(
            StatusCode::NOT_IMPLEMENTED,
            "commercial billing control plane is unavailable for the current storage runtime",
        )
    })
}

fn commercial_billing_error_response(error: anyhow::Error) -> (StatusCode, Json<ErrorResponse>) {
    let message = error.to_string();
    let status = if message.starts_with("account ") && message.ends_with(" does not exist") {
        StatusCode::NOT_FOUND
    } else if message.contains("does not implement canonical account kernel method") {
        StatusCode::NOT_IMPLEMENTED
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };
    error_response(status, message)
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
    invalidate_catalog_cache_after_mutation().await;
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
        invalidate_catalog_cache_after_mutation().await;
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
    invalidate_catalog_cache_after_mutation().await;
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
        invalidate_catalog_cache_after_mutation().await;
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
    let record = persist_model_price_with_rates_and_metadata(
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
        &request.price_source_kind,
        request.billing_notes.as_deref(),
        request.pricing_tiers,
        request.is_active,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    invalidate_catalog_cache_after_mutation().await;
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
        invalidate_catalog_cache_after_mutation().await;
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

async fn list_api_key_groups_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<ApiKeyGroupRecord>>, StatusCode> {
    list_api_key_groups(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_api_key_group_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateApiKeyGroupRequest>,
) -> Result<(StatusCode, Json<ApiKeyGroupRecord>), (StatusCode, Json<ErrorResponse>)> {
    create_api_key_group(
        state.store.as_ref(),
        ApiKeyGroupInput {
            tenant_id: request.tenant_id,
            project_id: request.project_id,
            environment: request.environment,
            name: request.name,
            slug: request.slug,
            description: request.description,
            color: request.color,
            default_capability_scope: request.default_capability_scope,
            default_routing_profile_id: request.default_routing_profile_id,
            default_accounting_mode: request.default_accounting_mode,
        },
    )
    .await
    .map(|group| (StatusCode::CREATED, Json(group)))
    .map_err(admin_error_response)
}

async fn update_api_key_group_handler(
    _claims: AuthenticatedAdminClaims,
    Path(group_id): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<UpdateApiKeyGroupRequest>,
) -> Result<Json<ApiKeyGroupRecord>, (StatusCode, Json<ErrorResponse>)> {
    match update_api_key_group(
        state.store.as_ref(),
        &group_id,
        ApiKeyGroupInput {
            tenant_id: request.tenant_id,
            project_id: request.project_id,
            environment: request.environment,
            name: request.name,
            slug: request.slug,
            description: request.description,
            color: request.color,
            default_capability_scope: request.default_capability_scope,
            default_routing_profile_id: request.default_routing_profile_id,
            default_accounting_mode: request.default_accounting_mode,
        },
    )
    .await
    {
        Ok(Some(group)) => Ok(Json(group)),
        Ok(None) => Err(admin_error_response(AdminIdentityError::NotFound(
            "api key group not found".to_owned(),
        ))),
        Err(error) => Err(admin_error_response(error)),
    }
}

async fn update_api_key_group_status_handler(
    _claims: AuthenticatedAdminClaims,
    Path(group_id): Path<String>,
    State(state): State<AdminApiState>,
    Json(request): Json<UpdateUserStatusRequest>,
) -> Result<Json<ApiKeyGroupRecord>, (StatusCode, Json<ErrorResponse>)> {
    match set_api_key_group_active(state.store.as_ref(), &group_id, request.active).await {
        Ok(Some(group)) => Ok(Json(group)),
        Ok(None) => Err(admin_error_response(AdminIdentityError::NotFound(
            "api key group not found".to_owned(),
        ))),
        Err(error) => Err(admin_error_response(error)),
    }
}

async fn delete_api_key_group_handler(
    _claims: AuthenticatedAdminClaims,
    Path(group_id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    match delete_api_key_group(state.store.as_ref(), &group_id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(admin_error_response(AdminIdentityError::NotFound(
            "api key group not found".to_owned(),
        ))),
        Err(error) => Err(admin_error_response(error)),
    }
}

async fn create_api_key_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateApiKeyRequest>,
) -> Result<(StatusCode, Json<CreatedGatewayApiKey>), (StatusCode, Json<ErrorResponse>)> {
    let metadata_label = request
        .label
        .as_deref()
        .map(str::trim)
        .filter(|label| !label.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| format!("{} gateway key", request.environment.trim()));
    let created = sdkwork_api_app_identity::persist_gateway_api_key_with_metadata(
        state.store.as_ref(),
        &request.tenant_id,
        &request.project_id,
        &request.environment,
        &metadata_label,
        request.expires_at_ms,
        request.plaintext_key.as_deref(),
        request.notes.as_deref(),
        request.api_key_group_id.as_deref(),
    )
    .await
    .map_err(gateway_api_key_create_error_response)?;
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
        request.api_key_group_id.as_deref(),
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
            .with_tenant_id_option(request.tenant_id.as_deref())
            .with_project_id_option(request.project_id.as_deref())
            .with_api_key_group_id_option(request.api_key_group_id.as_deref())
            .with_requested_region_option(request.requested_region.as_deref())
            .with_selection_seed_option(request.selection_seed),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let (selected_candidate, rejected_candidates) =
        split_routing_assessments(&decision.selected_provider_id, &decision.assessments);
    Ok(Json(RoutingSimulationResponse {
        selected_provider_id: decision.selected_provider_id,
        candidate_ids: decision.candidate_ids,
        matched_policy_id: decision.matched_policy_id,
        applied_routing_profile_id: decision.applied_routing_profile_id,
        compiled_routing_snapshot_id: decision.compiled_routing_snapshot_id,
        strategy: decision.strategy,
        selection_seed: decision.selection_seed,
        selection_reason: decision.selection_reason,
        fallback_reason: decision.fallback_reason,
        requested_region: decision.requested_region,
        slo_applied: decision.slo_applied,
        slo_degraded: decision.slo_degraded,
        selected_candidate,
        rejected_candidates,
        assessments: decision.assessments,
    }))
}

fn split_routing_assessments(
    selected_provider_id: &str,
    assessments: &[RoutingCandidateAssessment],
) -> (
    Option<RoutingCandidateAssessment>,
    Vec<RoutingCandidateAssessment>,
) {
    let mut selected_candidate = None;
    let mut rejected_candidates = Vec::new();
    for assessment in assessments {
        if assessment.provider_id == selected_provider_id && selected_candidate.is_none() {
            selected_candidate = Some(assessment.clone());
        } else {
            rejected_candidates.push(assessment.clone());
        }
    }
    (selected_candidate, rejected_candidates)
}

async fn list_compiled_routing_snapshots_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CompiledRoutingSnapshotRecord>>, StatusCode> {
    list_compiled_routing_snapshots(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
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

async fn list_routing_profiles_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<RoutingProfileRecord>>, StatusCode> {
    list_routing_profiles(state.store.as_ref())
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
        execution_failover_enabled: request.execution_failover_enabled,
        upstream_retry_max_attempts: request.upstream_retry_max_attempts,
        upstream_retry_base_delay_ms: request.upstream_retry_base_delay_ms,
        upstream_retry_max_delay_ms: request.upstream_retry_max_delay_ms,
    })
    .map_err(|_| StatusCode::BAD_REQUEST)?;
    let policy = persist_routing_policy(state.store.as_ref(), &policy)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(policy)))
}

async fn create_routing_profile_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateRoutingProfileRequest>,
) -> Result<(StatusCode, Json<RoutingProfileRecord>), StatusCode> {
    let profile = create_routing_profile(CreateRoutingProfileInput {
        profile_id: &request.profile_id,
        tenant_id: &request.tenant_id,
        project_id: &request.project_id,
        name: &request.name,
        slug: &request.slug,
        description: request.description.as_deref(),
        active: request.active,
        strategy: request.strategy,
        ordered_provider_ids: &request.ordered_provider_ids,
        default_provider_id: request.default_provider_id.as_deref(),
        max_cost: request.max_cost,
        max_latency_ms: request.max_latency_ms,
        require_healthy: request.require_healthy,
        preferred_region: request.preferred_region.as_deref(),
    })
    .map_err(|_| StatusCode::BAD_REQUEST)?;
    let profile = persist_routing_profile(state.store.as_ref(), &profile)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(profile)))
}
fn admin_commerce_error_response(error: CommerceError) -> (StatusCode, Json<ErrorResponse>) {
    let status = match error {
        CommerceError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        CommerceError::NotFound(_) => StatusCode::NOT_FOUND,
        CommerceError::Conflict(_) => StatusCode::CONFLICT,
        CommerceError::Storage(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };
    error_response(status, error.to_string())
}

fn next_admin_pricing_record_id(now_ms: u64) -> u64 {
    let sequence = ADMIN_PRICING_ID_SEQUENCE.fetch_add(1, Ordering::Relaxed) & 0x000f_ffff;
    (now_ms << 20) | sequence
}

fn normalize_optional_admin_text(value: Option<String>) -> Option<String> {
    value.and_then(|text| {
        let trimmed = text.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_owned())
    })
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

async fn list_billing_events_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<BillingEventRecord>>, StatusCode> {
    list_billing_events(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn billing_events_summary_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<BillingEventSummary>, StatusCode> {
    summarize_billing_events_from_store(state.store.as_ref())
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

async fn list_rate_limit_policies_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<RateLimitPolicy>>, StatusCode> {
    list_rate_limit_policies(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn list_rate_limit_window_snapshots_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<RateLimitWindowSnapshot>>, StatusCode> {
    state
        .store
        .list_rate_limit_window_snapshots()
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_rate_limit_policy_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateRateLimitPolicyRequest>,
) -> Result<(StatusCode, Json<RateLimitPolicy>), StatusCode> {
    let policy = create_rate_limit_policy(
        &request.policy_id,
        &request.project_id,
        request.requests_per_window,
        request.window_seconds,
        request.burst_requests,
        request.enabled,
        request.route_key.as_deref(),
        request.api_key_hash.as_deref(),
        request.model_name.as_deref(),
        request.notes.as_deref(),
    )
    .map_err(|_| StatusCode::BAD_REQUEST)?;
    let policy = persist_rate_limit_policy(state.store.as_ref(), &policy)
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

fn default_window_seconds() -> u64 {
    60
}
