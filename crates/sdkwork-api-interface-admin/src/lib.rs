use anyhow::ensure;
use std::collections::BTreeMap;
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

mod audit;
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
    extract::Request,
    extract::State,
    http::header,
    http::request::Parts,
    http::HeaderMap,
    http::Method,
    http::StatusCode,
    middleware::Next,
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
    AdminAuditEventRecord, AdminUserProfile, AdminUserRole, ApiKeyGroupRecord, GatewayApiKeyRecord,
    PortalUserProfile,
};
use sdkwork_api_domain_jobs::{
    AsyncJobAssetRecord, AsyncJobAttemptRecord, AsyncJobCallbackRecord, AsyncJobRecord,
};
use sdkwork_api_domain_marketing::{
    CampaignBudgetLifecycleAction, CampaignBudgetLifecycleAuditOutcome,
    CampaignBudgetLifecycleAuditRecord, CampaignBudgetRecord, CampaignBudgetStatus,
    CouponCodeLifecycleAction, CouponCodeLifecycleAuditOutcome, CouponCodeLifecycleAuditRecord,
    CouponCodeRecord, CouponCodeStatus, CouponRedemptionRecord, CouponReservationRecord,
    CouponRollbackRecord, CouponTemplateApprovalState, CouponTemplateLifecycleAction,
    CouponTemplateLifecycleAuditOutcome, CouponTemplateLifecycleAuditRecord, CouponTemplateRecord,
    CouponTemplateStatus, MarketingCampaignApprovalState, MarketingCampaignLifecycleAction,
    MarketingCampaignLifecycleAuditOutcome, MarketingCampaignLifecycleAuditRecord,
    MarketingCampaignRecord, MarketingCampaignStatus,
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
use sdkwork_api_observability::{observe_http_metrics, observe_http_tracing, HttpMetricsRegistry};
use sdkwork_api_storage_core::{AdminStore, CommercialKernelStore, Reloadable};
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
    try_admin_router_with_pool, try_admin_router_with_pool_and_master_key,
    try_admin_router_with_pool_and_secret_manager, try_admin_router_with_state,
    try_admin_router_with_store, try_admin_router_with_store_and_secret_manager,
    try_admin_router_with_store_and_secret_manager_and_jwt_secret,
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
struct AuthenticatedAdminClaims {
    claims: Claims,
    user: AdminUserProfile,
}

impl AuthenticatedAdminClaims {
    fn claims(&self) -> &Claims {
        &self.claims
    }

    fn user(&self) -> &AdminUserProfile {
        &self.user
    }

    fn role(&self) -> AdminUserRole {
        self.user.role
    }
}

async fn load_authenticated_admin_claims(
    headers: &HeaderMap,
    state: &AdminApiState,
) -> Result<AuthenticatedAdminClaims, StatusCode> {
    let Some(header_value) = headers.get(header::AUTHORIZATION) else {
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

    let claims =
        verify_jwt(token, &state.jwt_signing_secret).map_err(|_| StatusCode::UNAUTHORIZED)?;
    let user = load_admin_user_profile(state.store.as_ref(), &claims.sub)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?
        .filter(|user| user.active)
        .ok_or(StatusCode::UNAUTHORIZED)?;

    Ok(AuthenticatedAdminClaims { claims, user })
}

impl FromRequestParts<AdminApiState> for AuthenticatedAdminClaims {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AdminApiState,
    ) -> Result<Self, Self::Rejection> {
        if let Some(claims) = parts.extensions.get::<AuthenticatedAdminClaims>() {
            return Ok(claims.clone());
        }

        load_authenticated_admin_claims(&parts.headers, state).await
    }
}

#[derive(Clone, Copy)]
enum AdminPrivilege {
    BillingRead,
    CatalogRead,
    CatalogWrite,
    CommerceRead,
    CommerceWrite,
    FinanceWrite,
    SecretRead,
    SecretWrite,
    IdentityRead,
    IdentityWrite,
    RuntimeRead,
    RuntimeWrite,
    RoutingRead,
    RoutingWrite,
    MarketingRead,
    MarketingWrite,
}

fn role_allows_privilege(role: AdminUserRole, privilege: AdminPrivilege) -> bool {
    match role {
        AdminUserRole::SuperAdmin => true,
        AdminUserRole::FinanceOperator => matches!(
            privilege,
            AdminPrivilege::BillingRead
                | AdminPrivilege::CatalogRead
                | AdminPrivilege::CommerceRead
                | AdminPrivilege::CommerceWrite
                | AdminPrivilege::FinanceWrite
        ),
        AdminUserRole::PlatformOperator => matches!(
            privilege,
            AdminPrivilege::CatalogRead
                | AdminPrivilege::CatalogWrite
                | AdminPrivilege::RuntimeRead
                | AdminPrivilege::RuntimeWrite
                | AdminPrivilege::RoutingRead
                | AdminPrivilege::RoutingWrite
                | AdminPrivilege::MarketingRead
                | AdminPrivilege::MarketingWrite
                | AdminPrivilege::CommerceRead
        ),
        AdminUserRole::ReadOnlyOperator => matches!(
            privilege,
            AdminPrivilege::CatalogRead
                | AdminPrivilege::RuntimeRead
                | AdminPrivilege::RoutingRead
                | AdminPrivilege::MarketingRead
                | AdminPrivilege::CommerceRead
        ),
    }
}

fn require_admin_privilege(
    claims: &AuthenticatedAdminClaims,
    privilege: AdminPrivilege,
) -> Result<(), StatusCode> {
    if role_allows_privilege(claims.role(), privilege) {
        Ok(())
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

fn admin_forbidden_response() -> (StatusCode, Json<ErrorResponse>) {
    error_response(StatusCode::FORBIDDEN, "forbidden")
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

fn unix_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("unix time")
        .as_millis() as u64
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

#[derive(Clone, Copy)]
enum AdminRouteAccess {
    Public,
    Authenticated,
    Privileged(AdminPrivilege),
}

fn admin_read_method(method: &Method) -> bool {
    method == Method::GET || method == Method::HEAD
}

fn admin_route_access(method: &Method, path: &str) -> AdminRouteAccess {
    if path == "/admin/health" || path == "/admin/auth/login" {
        return AdminRouteAccess::Public;
    }

    if path == "/admin/auth/me" || path == "/admin/auth/change-password" {
        return AdminRouteAccess::Authenticated;
    }

    let read = admin_read_method(method);

    if path == "/admin/api-keys" || path.starts_with("/admin/api-keys/") {
        return AdminRouteAccess::Privileged(if read {
            AdminPrivilege::SecretRead
        } else {
            AdminPrivilege::SecretWrite
        });
    }

    if path == "/admin/credentials"
        || path.starts_with("/admin/credentials/")
        || path == "/admin/providers/official-configs"
    {
        return AdminRouteAccess::Privileged(if read {
            AdminPrivilege::SecretRead
        } else {
            AdminPrivilege::SecretWrite
        });
    }

    if path.starts_with("/admin/tenants/") && path.ends_with("/providers/readiness") {
        return AdminRouteAccess::Privileged(AdminPrivilege::CatalogRead);
    }

    if path.starts_with("/admin/users/")
        || path == "/admin/tenants"
        || path.starts_with("/admin/tenants/")
        || path == "/admin/projects"
        || path.starts_with("/admin/projects/")
        || path == "/admin/api-key-groups"
        || path.starts_with("/admin/api-key-groups/")
    {
        return AdminRouteAccess::Privileged(if read {
            AdminPrivilege::IdentityRead
        } else {
            AdminPrivilege::IdentityWrite
        });
    }

    if path.starts_with("/admin/extensions/")
        || path.starts_with("/admin/runtime-config/")
        || path == "/admin/async-jobs"
        || path.starts_with("/admin/async-jobs/")
    {
        return AdminRouteAccess::Privileged(if read {
            AdminPrivilege::RuntimeRead
        } else {
            AdminPrivilege::RuntimeWrite
        });
    }

    if path.starts_with("/admin/gateway/rate-limit-") || path.starts_with("/admin/routing/") {
        return AdminRouteAccess::Privileged(if read {
            AdminPrivilege::RoutingRead
        } else {
            AdminPrivilege::RoutingWrite
        });
    }

    if path.starts_with("/admin/billing/")
        || path.starts_with("/admin/usage/")
        || path.starts_with("/admin/payments/")
    {
        return AdminRouteAccess::Privileged(if read {
            AdminPrivilege::BillingRead
        } else {
            AdminPrivilege::FinanceWrite
        });
    }

    if path.starts_with("/admin/commerce/") {
        return AdminRouteAccess::Privileged(if read {
            AdminPrivilege::CommerceRead
        } else {
            AdminPrivilege::CommerceWrite
        });
    }

    if path.starts_with("/admin/marketing/") {
        return AdminRouteAccess::Privileged(if read {
            AdminPrivilege::MarketingRead
        } else {
            AdminPrivilege::MarketingWrite
        });
    }

    if path == "/admin/channels"
        || path.starts_with("/admin/channels/")
        || path == "/admin/providers"
        || path.starts_with("/admin/providers/")
        || path == "/admin/channel-models"
        || path.starts_with("/admin/channel-models/")
        || path == "/admin/provider-accounts"
        || path.starts_with("/admin/provider-accounts/")
        || path == "/admin/provider-models"
        || path.starts_with("/admin/provider-models/")
        || path == "/admin/models"
        || path.starts_with("/admin/models/")
        || path == "/admin/model-prices"
        || path.starts_with("/admin/model-prices/")
    {
        return AdminRouteAccess::Privileged(if read {
            AdminPrivilege::CatalogRead
        } else {
            AdminPrivilege::CatalogWrite
        });
    }

    if path.starts_with("/admin/") {
        AdminRouteAccess::Authenticated
    } else {
        AdminRouteAccess::Public
    }
}

fn admin_status_only_response(status: StatusCode) -> axum::response::Response {
    let message = match status {
        StatusCode::UNAUTHORIZED => "unauthorized",
        StatusCode::FORBIDDEN => "forbidden",
        _ => "request failed",
    };
    error_response(status, message).into_response()
}

async fn enforce_admin_route_access(
    State(state): State<AdminApiState>,
    mut request: Request,
    next: Next,
) -> axum::response::Response {
    let path = request.uri().path().to_owned();
    let method = request.method().clone();
    let access = admin_route_access(&method, &path);

    if matches!(access, AdminRouteAccess::Public) {
        return next.run(request).await;
    }

    let claims = match load_authenticated_admin_claims(request.headers(), &state).await {
        Ok(claims) => claims,
        Err(status) => return admin_status_only_response(status),
    };

    if let AdminRouteAccess::Privileged(privilege) = access {
        if require_admin_privilege(&claims, privilege).is_err() {
            return admin_forbidden_response().into_response();
        }
    }

    request.extensions_mut().insert(claims);
    next.run(request).await
}
