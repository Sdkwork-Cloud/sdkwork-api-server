use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use axum::{
    extract::{FromRequestParts, Path, Query, State},
    http::{header, request::Parts, HeaderMap, StatusCode},
    routing::{delete, get, patch, post},
    Json, Router,
};
use sdkwork_api_app_billing::{
    list_billing_events, summarize_account_balance, summarize_billing_events,
    summarize_billing_snapshot,
};
use sdkwork_api_app_commerce::{
    apply_portal_commerce_payment_event, cancel_portal_commerce_order,
    list_project_commerce_orders, load_portal_commerce_catalog,
    load_portal_commerce_checkout_session, load_portal_commerce_order, load_project_membership,
    preview_portal_commerce_quote_for_user, settle_portal_commerce_order_from_session,
    submit_portal_commerce_order, CommerceError, PortalCommerceCatalog,
    PortalCommerceCheckoutSession, PortalCommerceOrderRecord, PortalCommercePaymentEventRequest,
    PortalCommerceQuote, PortalCommerceQuoteRequest, PortalProjectMembershipRecord,
};
use sdkwork_api_app_identity::{
    change_portal_password, create_portal_api_key_group, create_portal_api_key_with_metadata,
    delete_portal_api_key, delete_portal_api_key_group, list_portal_api_key_groups,
    list_portal_api_keys, list_portal_workspace_memberships, load_portal_user_profile,
    load_portal_workspace_summary, login_portal_user_with_bootstrap, register_portal_user,
    select_portal_workspace, set_portal_api_key_active, set_portal_api_key_group_active,
    update_portal_api_key_group, verify_portal_jwt, CreatedGatewayApiKey, PortalApiKeyGroupInput,
    PortalAuthSession, PortalClaims, PortalIdentityError, PortalWorkspaceMembershipSummary,
    PortalWorkspaceSummary,
};
use sdkwork_api_app_marketing::{
    list_coupon_codes, list_coupon_redemptions, summarize_coupon_codes,
    summarize_coupon_redemptions, CouponCodeSummary, CouponRedemptionSummary, ListCouponCodesInput,
    ListCouponRedemptionsInput,
};
use sdkwork_api_app_payment::{
    apply_alipay_notification, apply_stripe_webhook, apply_wechatpay_notification,
    provision_stripe_checkout_session, stripe_signature_header_name, wechatpay_nonce_header_name,
    wechatpay_signature_header_name, wechatpay_timestamp_header_name, AlipayPaymentConfig,
    PaymentError, StripePaymentConfig, WeChatPayPaymentConfig,
};
use sdkwork_api_app_rate_limit::{GatewayTrafficController, InMemoryGatewayTrafficController};
use sdkwork_api_app_routing::{
    create_routing_profile, list_compiled_routing_snapshots, list_routing_profiles,
    persist_routing_profile, select_route_with_store_context,
    simulate_route_with_store_selection_context, CreateRoutingProfileInput, RouteSelectionContext,
};
use sdkwork_api_app_usage::summarize_usage_records;
use sdkwork_api_domain_billing::{
    AccountType, BillingEventRecord, BillingEventSummary, LedgerEntry, ProjectBillingSummary,
};
use sdkwork_api_domain_catalog::ProxyProvider;
use sdkwork_api_domain_identity::{ApiKeyGroupRecord, GatewayApiKeyRecord, PortalUserProfile};
use sdkwork_api_domain_marketing::{
    CouponCodeRecord, CouponCodeStatus, CouponRedemptionRecord, CouponRedemptionStatus,
};
use sdkwork_api_domain_rate_limit::{
    CommercialPressureScopeKind, RateLimitPolicy, RateLimitWindowSnapshot, TrafficPressureSnapshot,
};
use sdkwork_api_domain_routing::{
    CompiledRoutingSnapshotRecord, ProjectRoutingPreferences, RoutingDecision, RoutingDecisionLog,
    RoutingDecisionSource, RoutingProfileRecord, RoutingStrategy,
};
use sdkwork_api_domain_usage::{UsageRecord, UsageSummary};
use sdkwork_api_observability::{observe_http_metrics, observe_http_tracing, HttpMetricsRegistry};
use sdkwork_api_storage_core::{AdminStore, Reloadable};
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

const PORTAL_WORKSPACE_IDENTITY_BINDING_TYPE: &str = "portal_workspace_user";
const PORTAL_WORKSPACE_IDENTITY_BINDING_ISSUER: &str = "sdkwork-api-portal";
use tower_http::cors::{Any, CorsLayer};

const DEFAULT_PORTAL_JWT_SIGNING_SECRET: &str = "local-dev-portal-jwt-secret";
const PORTAL_ALLOW_MANUAL_SETTLEMENT_ENV: &str = "SDKWORK_PORTAL_ALLOW_MANUAL_SETTLEMENT";
const PORTAL_PAYMENT_CALLBACK_SECRET_ENV: &str = "SDKWORK_PORTAL_PAYMENT_CALLBACK_SECRET";
const PORTAL_PAYMENT_CALLBACK_SECRET_HEADER: &str = "x-sdkwork-payment-callback-secret";

pub struct PortalApiState {
    live_store: Reloadable<Arc<dyn AdminStore>>,
    store: Arc<dyn AdminStore>,
    live_jwt_signing_secret: Reloadable<String>,
    jwt_signing_secret: String,
    traffic_controller: Arc<dyn GatewayTrafficController>,
    allow_local_dev_bootstrap: bool,
    allow_manual_paid_settlement: bool,
    payment_callback_secret: Option<String>,
    alipay_payment: Option<AlipayPaymentConfig>,
    stripe_payment: Option<StripePaymentConfig>,
    wechatpay_payment: Option<WeChatPayPaymentConfig>,
}

impl Clone for PortalApiState {
    fn clone(&self) -> Self {
        Self {
            live_store: self.live_store.clone(),
            store: self.live_store.snapshot(),
            live_jwt_signing_secret: self.live_jwt_signing_secret.clone(),
            jwt_signing_secret: self.live_jwt_signing_secret.snapshot(),
            traffic_controller: Arc::clone(&self.traffic_controller),
            allow_local_dev_bootstrap: self.allow_local_dev_bootstrap,
            allow_manual_paid_settlement: self.allow_manual_paid_settlement,
            payment_callback_secret: self.payment_callback_secret.clone(),
            alipay_payment: self.alipay_payment.clone(),
            stripe_payment: self.stripe_payment.clone(),
            wechatpay_payment: self.wechatpay_payment.clone(),
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
        Self::with_store_and_jwt_secret_and_traffic_controller(
            store,
            jwt_signing_secret,
            Arc::new(InMemoryGatewayTrafficController::new()),
        )
    }

    pub fn with_store_and_jwt_secret_and_traffic_controller(
        store: Arc<dyn AdminStore>,
        jwt_signing_secret: impl Into<String>,
        traffic_controller: Arc<dyn GatewayTrafficController>,
    ) -> Self {
        Self::with_store_and_jwt_secret_and_local_bootstrap_and_traffic_controller(
            store,
            jwt_signing_secret,
            true,
            traffic_controller,
        )
    }

    pub fn with_store_and_jwt_secret_and_local_bootstrap(
        store: Arc<dyn AdminStore>,
        jwt_signing_secret: impl Into<String>,
        allow_local_dev_bootstrap: bool,
    ) -> Self {
        Self::with_store_and_jwt_secret_and_local_bootstrap_and_traffic_controller(
            store,
            jwt_signing_secret,
            allow_local_dev_bootstrap,
            Arc::new(InMemoryGatewayTrafficController::new()),
        )
    }

    pub fn with_store_and_jwt_secret_and_local_bootstrap_and_traffic_controller(
        store: Arc<dyn AdminStore>,
        jwt_signing_secret: impl Into<String>,
        allow_local_dev_bootstrap: bool,
        traffic_controller: Arc<dyn GatewayTrafficController>,
    ) -> Self {
        Self::with_live_store_and_jwt_secret_handle_and_local_bootstrap_and_traffic_controller(
            Reloadable::new(store),
            Reloadable::new(jwt_signing_secret.into()),
            allow_local_dev_bootstrap,
            traffic_controller,
        )
    }

    pub fn with_live_store_and_jwt_secret_handle(
        live_store: Reloadable<Arc<dyn AdminStore>>,
        live_jwt_signing_secret: Reloadable<String>,
    ) -> Self {
        Self::with_live_store_and_jwt_secret_handle_and_traffic_controller(
            live_store,
            live_jwt_signing_secret,
            Arc::new(InMemoryGatewayTrafficController::new()),
        )
    }

    pub fn with_live_store_and_jwt_secret_handle_and_traffic_controller(
        live_store: Reloadable<Arc<dyn AdminStore>>,
        live_jwt_signing_secret: Reloadable<String>,
        traffic_controller: Arc<dyn GatewayTrafficController>,
    ) -> Self {
        Self::with_live_store_and_jwt_secret_handle_and_local_bootstrap_and_traffic_controller(
            live_store,
            live_jwt_signing_secret,
            true,
            traffic_controller,
        )
    }

    pub fn with_live_store_and_jwt_secret_handle_and_local_bootstrap(
        live_store: Reloadable<Arc<dyn AdminStore>>,
        live_jwt_signing_secret: Reloadable<String>,
        allow_local_dev_bootstrap: bool,
    ) -> Self {
        Self::with_live_store_and_jwt_secret_handle_and_local_bootstrap_and_traffic_controller(
            live_store,
            live_jwt_signing_secret,
            allow_local_dev_bootstrap,
            Arc::new(InMemoryGatewayTrafficController::new()),
        )
    }

    pub fn with_live_store_and_jwt_secret_handle_and_local_bootstrap_and_traffic_controller(
        live_store: Reloadable<Arc<dyn AdminStore>>,
        live_jwt_signing_secret: Reloadable<String>,
        allow_local_dev_bootstrap: bool,
        traffic_controller: Arc<dyn GatewayTrafficController>,
    ) -> Self {
        Self {
            store: live_store.snapshot(),
            live_store,
            jwt_signing_secret: live_jwt_signing_secret.snapshot(),
            live_jwt_signing_secret,
            traffic_controller,
            allow_local_dev_bootstrap,
            allow_manual_paid_settlement: env_flag_is_truthy(PORTAL_ALLOW_MANUAL_SETTLEMENT_ENV),
            payment_callback_secret: optional_env_value(PORTAL_PAYMENT_CALLBACK_SECRET_ENV),
            alipay_payment: AlipayPaymentConfig::from_env(),
            stripe_payment: StripePaymentConfig::from_env(),
            wechatpay_payment: WeChatPayPaymentConfig::from_env(),
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

        let claims = verify_portal_jwt(token, &state.jwt_signing_secret)
            .map_err(|_| StatusCode::UNAUTHORIZED)?;
        let user = state
            .store
            .find_portal_user_by_id(&claims.sub)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let Some(user) = user else {
            return Err(StatusCode::UNAUTHORIZED);
        };
        if !user.active
            || user.workspace_tenant_id != claims.workspace_tenant_id
            || user.workspace_project_id != claims.workspace_project_id
        {
            return Err(StatusCode::UNAUTHORIZED);
        }

        Ok(Self(claims))
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
    api_key: Option<String>,
    #[serde(default)]
    api_key_group_id: Option<String>,
    #[serde(default)]
    notes: Option<String>,
    #[serde(default)]
    expires_at_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct UpdateApiKeyStatusRequest {
    active: bool,
}

#[derive(Debug, Deserialize)]
struct CreateApiKeyGroupRequest {
    environment: String,
    name: String,
    #[serde(default)]
    slug: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    color: Option<String>,
    #[serde(default)]
    default_capability_scope: Option<String>,
    #[serde(default)]
    default_accounting_mode: Option<String>,
    #[serde(default)]
    default_routing_profile_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UpdateApiKeyGroupRequest {
    environment: String,
    name: String,
    #[serde(default)]
    slug: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    color: Option<String>,
    #[serde(default)]
    default_capability_scope: Option<String>,
    #[serde(default)]
    default_accounting_mode: Option<String>,
    #[serde(default)]
    default_routing_profile_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChangePasswordRequest {
    current_password: String,
    new_password: String,
}

#[derive(Debug, Deserialize)]
struct SelectWorkspaceRequest {
    tenant_id: String,
    project_id: String,
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

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
struct CreatePortalRoutingProfileRequest {
    name: String,
    #[serde(default)]
    slug: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default = "default_true")]
    active: bool,
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
    #[serde(default)]
    preferred_region: Option<String>,
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
struct PortalGatewayRateLimitSnapshot {
    project_id: String,
    policy_count: usize,
    active_policy_count: usize,
    window_count: usize,
    exceeded_window_count: usize,
    headline: String,
    detail: String,
    generated_at_ms: u64,
    policies: Vec<RateLimitPolicy>,
    windows: Vec<RateLimitWindowSnapshot>,
}

#[derive(Debug, Serialize)]
struct PortalGatewayTrafficPressureResponse {
    project_id: String,
    snapshot_count: usize,
    saturated_snapshot_count: usize,
    throttled_api_key_count: usize,
    saturated_provider_count: usize,
    headline: String,
    detail: String,
    generated_at_ms: u64,
    snapshots: Vec<TrafficPressureSnapshot>,
    throttled_api_keys: Vec<TrafficPressureSnapshot>,
    saturated_providers: Vec<TrafficPressureSnapshot>,
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

#[derive(Debug, Deserialize, Default)]
struct PortalMarketingRedemptionsQuery {
    #[serde(default)]
    status: Option<CouponRedemptionStatus>,
}

#[derive(Debug, Deserialize, Default)]
struct PortalMarketingCodesQuery {
    #[serde(default)]
    status: Option<CouponCodeStatus>,
}

#[derive(Debug, Serialize)]
struct PortalMarketingRedemptionsResponse {
    summary: CouponRedemptionSummary,
    items: Vec<CouponRedemptionRecord>,
}

#[derive(Debug, Serialize)]
struct PortalMarketingCodeItem {
    code: CouponCodeRecord,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    latest_redemption: Option<CouponRedemptionRecord>,
}

#[derive(Debug, Serialize)]
struct PortalMarketingCodesResponse {
    summary: CouponCodeSummary,
    items: Vec<PortalMarketingCodeItem>,
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
        .route("/portal/workspaces", get(|| async { "workspaces" }))
        .route(
            "/portal/workspaces/select",
            post(|| async { "workspace-select" }),
        )
        .route(
            "/portal/marketing/redemptions",
            get(|| async { "marketing-redemptions" }),
        )
        .route(
            "/portal/marketing/codes",
            get(|| async { "marketing-codes" }),
        )
        .route(
            "/portal/commerce/catalog",
            get(|| async { "commerce-catalog" }),
        )
        .route(
            "/portal/commerce/quote",
            post(|| async { "commerce-quote" }),
        )
        .route(
            "/portal/commerce/orders",
            get(|| async { "commerce-orders" }).post(|| async { "commerce-orders" }),
        )
        .route(
            "/portal/commerce/orders/{order_id}/settle",
            post(|| async { "commerce-order-settle" }),
        )
        .route(
            "/portal/commerce/orders/{order_id}/cancel",
            post(|| async { "commerce-order-cancel" }),
        )
        .route(
            "/portal/commerce/orders/{order_id}/payment-events",
            post(|| async { "commerce-order-payment-events" }),
        )
        .route(
            "/portal/internal/commerce/orders/{order_id}/payment-events",
            post(|| async { "commerce-order-provider-payment-events" }),
        )
        .route(
            "/portal/internal/payments/stripe/webhook",
            post(|| async { "stripe-webhook" }),
        )
        .route(
            "/portal/internal/payments/alipay/notify",
            post(|| async { "alipay-notify" }),
        )
        .route(
            "/portal/internal/payments/wechat/notify",
            post(|| async { "wechatpay-notify" }),
        )
        .route(
            "/portal/commerce/orders/{order_id}/checkout-session",
            get(|| async { "commerce-order-checkout-session" }),
        )
        .route(
            "/portal/commerce/membership",
            get(|| async { "commerce-membership" }),
        )
        .route("/portal/api-keys", get(|| async { "api-keys" }))
        .route("/portal/api-key-groups", get(|| async { "api-key-groups" }))
        .route(
            "/portal/api-key-groups/{group_id}",
            patch(|| async { "api-key-groups" }).delete(|| async { "api-key-groups" }),
        )
        .route(
            "/portal/api-key-groups/{group_id}/status",
            post(|| async { "api-key-groups-status" }),
        )
        .route("/portal/usage/records", get(|| async { "usage-records" }))
        .route("/portal/usage/summary", get(|| async { "usage-summary" }))
        .route(
            "/portal/billing/summary",
            get(|| async { "billing-summary" }),
        )
        .route("/portal/billing/ledger", get(|| async { "billing-ledger" }))
        .route("/portal/billing/events", get(|| async { "billing-events" }))
        .route(
            "/portal/billing/events/summary",
            get(|| async { "billing-events-summary" }),
        )
        .route(
            "/portal/routing/summary",
            get(|| async { "routing-summary" }),
        )
        .route(
            "/portal/routing/profiles",
            get(|| async { "routing-profiles" }).post(|| async { "routing-profiles" }),
        )
        .route(
            "/portal/routing/preferences",
            get(|| async { "routing-preferences" }).post(|| async { "routing-preferences" }),
        )
        .route(
            "/portal/routing/snapshots",
            get(|| async { "routing-snapshots" }),
        )
        .route(
            "/portal/routing/preview",
            post(|| async { "routing-preview" }),
        )
        .route(
            "/portal/routing/decision-logs",
            get(|| async { "routing-decision-logs" }),
        )
        .route(
            "/portal/gateway/traffic-pressure",
            get(|| async { "gateway-traffic-pressure" }),
        )
        .layer(axum::middleware::from_fn_with_state(
            metrics,
            observe_http_metrics,
        ))
        .layer(browser_cors_layer())
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
        .route("/portal/workspaces", get(list_workspaces_handler))
        .route("/portal/workspaces/select", post(select_workspace_handler))
        .route(
            "/portal/marketing/redemptions",
            get(list_marketing_redemptions_handler),
        )
        .route("/portal/marketing/codes", get(list_marketing_codes_handler))
        .route(
            "/portal/gateway/rate-limit-snapshot",
            get(gateway_rate_limit_snapshot_handler),
        )
        .route(
            "/portal/gateway/traffic-pressure",
            get(gateway_traffic_pressure_handler),
        )
        .route("/portal/commerce/catalog", get(commerce_catalog_handler))
        .route("/portal/commerce/quote", post(commerce_quote_handler))
        .route(
            "/portal/commerce/orders",
            get(list_commerce_orders_handler).post(create_commerce_order_handler),
        )
        .route(
            "/portal/commerce/orders/{order_id}/settle",
            post(settle_commerce_order_handler),
        )
        .route(
            "/portal/commerce/orders/{order_id}/cancel",
            post(cancel_commerce_order_handler),
        )
        .route(
            "/portal/commerce/orders/{order_id}/payment-events",
            post(apply_commerce_payment_event_handler),
        )
        .route(
            "/portal/internal/commerce/orders/{order_id}/payment-events",
            post(apply_provider_commerce_payment_event_handler),
        )
        .route(
            "/portal/internal/payments/stripe/webhook",
            post(apply_stripe_webhook_handler),
        )
        .route(
            "/portal/internal/payments/alipay/notify",
            post(apply_alipay_notification_handler),
        )
        .route(
            "/portal/internal/payments/wechat/notify",
            post(apply_wechatpay_notification_handler),
        )
        .route(
            "/portal/commerce/orders/{order_id}/checkout-session",
            get(get_commerce_checkout_session_handler),
        )
        .route(
            "/portal/commerce/membership",
            get(get_project_membership_handler),
        )
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
        .route(
            "/portal/api-key-groups",
            get(list_api_key_groups_handler).post(create_api_key_group_handler),
        )
        .route(
            "/portal/api-key-groups/{group_id}",
            patch(update_api_key_group_handler).delete(delete_api_key_group_handler),
        )
        .route(
            "/portal/api-key-groups/{group_id}/status",
            post(update_api_key_group_status_handler),
        )
        .route("/portal/usage/records", get(list_usage_records_handler))
        .route("/portal/usage/summary", get(usage_summary_handler))
        .route("/portal/billing/summary", get(billing_summary_handler))
        .route("/portal/billing/ledger", get(list_billing_ledger_handler))
        .route("/portal/billing/events", get(list_billing_events_handler))
        .route(
            "/portal/billing/events/summary",
            get(billing_events_summary_handler),
        )
        .route("/portal/routing/summary", get(routing_summary_handler))
        .route(
            "/portal/routing/profiles",
            get(list_routing_profiles_handler).post(create_routing_profile_handler),
        )
        .route(
            "/portal/routing/preferences",
            get(get_routing_preferences_handler).post(save_routing_preferences_handler),
        )
        .route(
            "/portal/routing/snapshots",
            get(list_routing_snapshots_handler),
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
        .layer(browser_cors_layer())
        .layer(axum::middleware::from_fn_with_state(
            service_name,
            observe_http_tracing,
        ))
        .with_state(state)
}

fn browser_cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
}

fn env_flag_is_truthy(key: &str) -> bool {
    std::env::var(key)
        .ok()
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn optional_env_value(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
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
    login_portal_user_with_bootstrap(
        state.store.as_ref(),
        &request.email,
        &request.password,
        &state.jwt_signing_secret,
        state.allow_local_dev_bootstrap,
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

async fn list_workspaces_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<PortalWorkspaceMembershipSummary>>, (StatusCode, Json<ErrorResponse>)> {
    list_portal_workspace_memberships(state.store.as_ref(), &claims.claims().sub)
        .await
        .map(Json)
        .map_err(portal_error_response)
}

async fn select_workspace_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    Json(request): Json<SelectWorkspaceRequest>,
) -> Result<Json<PortalAuthSession>, (StatusCode, Json<ErrorResponse>)> {
    select_portal_workspace(
        state.store.as_ref(),
        &claims.claims().sub,
        &request.tenant_id,
        &request.project_id,
        &state.jwt_signing_secret,
    )
    .await
    .map(Json)
    .map_err(portal_error_response)
}

async fn list_marketing_redemptions_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    Query(query): Query<PortalMarketingRedemptionsQuery>,
) -> Result<Json<PortalMarketingRedemptionsResponse>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let input = ListCouponRedemptionsInput::new()
        .with_subject("user", claims.claims().sub.clone())
        .with_project_id(Some(workspace.project.id.clone()))
        .with_status(query.status);
    let mut items = list_coupon_redemptions(state.store.as_ref(), &input)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    items.sort_by(|left, right| {
        right
            .created_at_ms
            .cmp(&left.created_at_ms)
            .then_with(|| right.coupon_redemption_id.cmp(&left.coupon_redemption_id))
    });
    let summary = summarize_coupon_redemptions(state.store.as_ref(), &input)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(PortalMarketingRedemptionsResponse { summary, items }))
}

async fn list_marketing_codes_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    Query(query): Query<PortalMarketingCodesQuery>,
) -> Result<Json<PortalMarketingCodesResponse>, StatusCode> {
    let _workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let input = ListCouponCodesInput::new()
        .with_subject("user", claims.claims().sub.clone())
        .with_status(query.status);
    let codes = list_coupon_codes(state.store.as_ref(), &input)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let redemptions = list_coupon_redemptions(
        state.store.as_ref(),
        &ListCouponRedemptionsInput::new().with_subject("user", claims.claims().sub.clone()),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut latest_redemptions = HashMap::new();
    for redemption in redemptions {
        latest_redemptions
            .entry(redemption.coupon_code_id)
            .and_modify(|current: &mut CouponRedemptionRecord| {
                if redemption.updated_at_ms > current.updated_at_ms
                    || (redemption.updated_at_ms == current.updated_at_ms
                        && redemption.coupon_redemption_id > current.coupon_redemption_id)
                {
                    *current = redemption.clone();
                }
            })
            .or_insert(redemption);
    }

    let items = codes
        .into_iter()
        .map(|code| PortalMarketingCodeItem {
            latest_redemption: latest_redemptions.get(&code.coupon_code_id).cloned(),
            code,
        })
        .collect::<Vec<_>>();
    let summary = summarize_coupon_codes(state.store.as_ref(), &input)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(PortalMarketingCodesResponse { summary, items }))
}

async fn commerce_catalog_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalCommerceCatalog>, StatusCode> {
    let _workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    load_portal_commerce_catalog(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn commerce_quote_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    Json(request): Json<PortalCommerceQuoteRequest>,
) -> Result<Json<PortalCommerceQuote>, (StatusCode, Json<ErrorResponse>)> {
    let _workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|status| {
            (
                status,
                Json(ErrorResponse {
                    error: ErrorBody {
                        message: "portal workspace is unavailable".to_owned(),
                    },
                }),
            )
        })?;

    preview_portal_commerce_quote_for_user(state.store.as_ref(), &claims.claims().sub, &request)
        .await
        .map(Json)
        .map_err(portal_commerce_error_response)
}

async fn list_commerce_orders_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<PortalCommerceOrderRecord>>, (StatusCode, Json<ErrorResponse>)> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|status| {
            (
                status,
                Json(ErrorResponse {
                    error: ErrorBody {
                        message: "portal workspace is unavailable".to_owned(),
                    },
                }),
            )
        })?;

    list_project_commerce_orders(state.store.as_ref(), &workspace.project.id)
        .await
        .map(Json)
        .map_err(portal_commerce_error_response)
}

async fn create_commerce_order_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    Json(request): Json<PortalCommerceQuoteRequest>,
) -> Result<(StatusCode, Json<PortalCommerceOrderRecord>), (StatusCode, Json<ErrorResponse>)> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|status| {
            (
                status,
                Json(ErrorResponse {
                    error: ErrorBody {
                        message: "portal workspace is unavailable".to_owned(),
                    },
                }),
            )
        })?;

    submit_portal_commerce_order(
        state.store.as_ref(),
        &claims.claims().sub,
        &workspace.project.id,
        &request,
    )
    .await
    .map(|order| (StatusCode::CREATED, Json(order)))
    .map_err(portal_commerce_error_response)
}

async fn settle_commerce_order_handler(
    claims: AuthenticatedPortalClaims,
    Path(order_id): Path<String>,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalCommerceOrderRecord>, (StatusCode, Json<ErrorResponse>)> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|status| {
            (
                status,
                Json(ErrorResponse {
                    error: ErrorBody {
                        message: "portal workspace is unavailable".to_owned(),
                    },
                }),
            )
        })?;

    settle_portal_commerce_order_from_session(
        state.store.as_ref(),
        &claims.claims().sub,
        &workspace.project.id,
        &order_id,
        state.allow_manual_paid_settlement,
    )
    .await
    .map(Json)
    .map_err(portal_commerce_error_response)
}

async fn cancel_commerce_order_handler(
    claims: AuthenticatedPortalClaims,
    Path(order_id): Path<String>,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalCommerceOrderRecord>, (StatusCode, Json<ErrorResponse>)> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|status| {
            (
                status,
                Json(ErrorResponse {
                    error: ErrorBody {
                        message: "portal workspace is unavailable".to_owned(),
                    },
                }),
            )
        })?;

    cancel_portal_commerce_order(
        state.store.as_ref(),
        &claims.claims().sub,
        &workspace.project.id,
        &order_id,
    )
    .await
    .map(Json)
    .map_err(portal_commerce_error_response)
}

async fn apply_commerce_payment_event_handler(
    _claims: AuthenticatedPortalClaims,
    Path(_order_id): Path<String>,
    Json(_request): Json<PortalCommercePaymentEventRequest>,
) -> Result<Json<PortalCommerceOrderRecord>, (StatusCode, Json<ErrorResponse>)> {
    Err((
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
            error: ErrorBody {
                message: "payment provider callbacks must use the internal callback endpoint"
                    .to_owned(),
            },
        }),
    ))
}

async fn apply_provider_commerce_payment_event_handler(
    Path(order_id): Path<String>,
    State(state): State<PortalApiState>,
    headers: HeaderMap,
    Json(request): Json<PortalCommercePaymentEventRequest>,
) -> Result<Json<PortalCommerceOrderRecord>, (StatusCode, Json<ErrorResponse>)> {
    authorize_portal_payment_callback(&state, &headers)?;

    apply_portal_commerce_payment_event(state.store.as_ref(), &order_id, &request)
        .await
        .map(Json)
        .map_err(portal_commerce_error_response)
}

async fn apply_stripe_webhook_handler(
    State(state): State<PortalApiState>,
    headers: HeaderMap,
    payload: String,
) -> Result<Json<PortalCommerceOrderRecord>, (StatusCode, Json<ErrorResponse>)> {
    let stripe_payment = state.stripe_payment.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: ErrorBody {
                    message: "stripe payments are not enabled".to_owned(),
                },
            }),
        )
    })?;
    let signature = headers
        .get(stripe_signature_header_name())
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();

    apply_stripe_webhook(state.store.as_ref(), stripe_payment, signature, &payload)
        .await
        .map(Json)
        .map_err(portal_payment_error_response)
}

async fn apply_alipay_notification_handler(
    State(state): State<PortalApiState>,
    payload: String,
) -> Result<Json<PortalCommerceOrderRecord>, (StatusCode, Json<ErrorResponse>)> {
    let alipay_payment = state.alipay_payment.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: ErrorBody {
                    message: "alipay payments are not enabled".to_owned(),
                },
            }),
        )
    })?;

    apply_alipay_notification(state.store.as_ref(), alipay_payment, &payload)
        .await
        .map(Json)
        .map_err(portal_payment_error_response)
}

async fn apply_wechatpay_notification_handler(
    State(state): State<PortalApiState>,
    headers: HeaderMap,
    payload: String,
) -> Result<Json<PortalCommerceOrderRecord>, (StatusCode, Json<ErrorResponse>)> {
    let wechatpay_payment = state.wechatpay_payment.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: ErrorBody {
                    message: "wechatpay payments are not enabled".to_owned(),
                },
            }),
        )
    })?;
    let signature = headers
        .get(wechatpay_signature_header_name())
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let timestamp = headers
        .get(wechatpay_timestamp_header_name())
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let nonce = headers
        .get(wechatpay_nonce_header_name())
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();

    apply_wechatpay_notification(
        state.store.as_ref(),
        wechatpay_payment,
        signature,
        timestamp,
        nonce,
        &payload,
    )
    .await
    .map(Json)
    .map_err(portal_payment_error_response)
}

async fn get_commerce_checkout_session_handler(
    claims: AuthenticatedPortalClaims,
    Path(order_id): Path<String>,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalCommerceCheckoutSession>, (StatusCode, Json<ErrorResponse>)> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|status| {
            (
                status,
                Json(ErrorResponse {
                    error: ErrorBody {
                        message: "portal workspace is unavailable".to_owned(),
                    },
                }),
            )
        })?;

    if let Some(stripe_payment) = state.stripe_payment.as_ref() {
        let order = load_portal_commerce_order(
            state.store.as_ref(),
            &claims.claims().sub,
            &workspace.project.id,
            &order_id,
        )
        .await
        .map_err(portal_commerce_error_response)?;
        if order.status == "pending_payment" && order.payable_price_cents > 0 {
            let checkout =
                provision_stripe_checkout_session(state.store.as_ref(), &order, stripe_payment)
                    .await
                    .map_err(portal_payment_error_response)?;
            return Ok(Json(PortalCommerceCheckoutSession {
                order_id: order.order_id.clone(),
                order_status: order.status.clone(),
                session_status: "open".to_owned(),
                provider: "stripe".to_owned(),
                mode: "hosted_checkout".to_owned(),
                reference: checkout.payment_order_id,
                checkout_url: Some(checkout.checkout_url),
                payable_price_label: order.payable_price_label.clone(),
                guidance: "Complete the hosted Stripe checkout to finalize the order."
                    .to_owned(),
                methods: vec![
                    sdkwork_api_app_commerce::PortalCommerceCheckoutSessionMethod {
                        id: "redirect_checkout".to_owned(),
                        label: "Open Stripe checkout".to_owned(),
                        detail:
                            "Use the hosted Stripe checkout session to settle the current order."
                                .to_owned(),
                        action: "redirect_checkout".to_owned(),
                        availability: "available".to_owned(),
                    },
                    sdkwork_api_app_commerce::PortalCommerceCheckoutSessionMethod {
                        id: "cancel_order".to_owned(),
                        label: "Cancel checkout".to_owned(),
                        detail:
                            "Close the pending order without applying quota or membership side effects."
                                .to_owned(),
                        action: "cancel_order".to_owned(),
                        availability: "available".to_owned(),
                    },
                ],
            }));
        }
    }

    load_portal_commerce_checkout_session(
        state.store.as_ref(),
        &claims.claims().sub,
        &workspace.project.id,
        &order_id,
        state.allow_manual_paid_settlement,
    )
    .await
    .map(Json)
    .map_err(portal_commerce_error_response)
}

async fn get_project_membership_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Option<PortalProjectMembershipRecord>>, (StatusCode, Json<ErrorResponse>)> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|status| {
            (
                status,
                Json(ErrorResponse {
                    error: ErrorBody {
                        message: "portal workspace is unavailable".to_owned(),
                    },
                }),
            )
        })?;

    load_project_membership(state.store.as_ref(), &workspace.project.id)
        .await
        .map(Json)
        .map_err(portal_commerce_error_response)
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

async fn list_api_key_groups_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<ApiKeyGroupRecord>>, (StatusCode, Json<ErrorResponse>)> {
    list_portal_api_key_groups(state.store.as_ref(), &claims.claims().sub)
        .await
        .map(Json)
        .map_err(portal_error_response)
}

async fn create_api_key_group_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    Json(request): Json<CreateApiKeyGroupRequest>,
) -> Result<(StatusCode, Json<ApiKeyGroupRecord>), (StatusCode, Json<ErrorResponse>)> {
    create_portal_api_key_group(
        state.store.as_ref(),
        &claims.claims().sub,
        PortalApiKeyGroupInput {
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
    .map_err(portal_error_response)
}

async fn update_api_key_group_handler(
    claims: AuthenticatedPortalClaims,
    Path(group_id): Path<String>,
    State(state): State<PortalApiState>,
    Json(request): Json<UpdateApiKeyGroupRequest>,
) -> Result<Json<ApiKeyGroupRecord>, (StatusCode, Json<ErrorResponse>)> {
    match update_portal_api_key_group(
        state.store.as_ref(),
        &claims.claims().sub,
        &group_id,
        PortalApiKeyGroupInput {
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
    .map_err(portal_error_response)?
    {
        Some(group) => Ok(Json(group)),
        None => Err(portal_error_response(PortalIdentityError::NotFound(
            "api key group not found".to_owned(),
        ))),
    }
}

async fn update_api_key_group_status_handler(
    claims: AuthenticatedPortalClaims,
    Path(group_id): Path<String>,
    State(state): State<PortalApiState>,
    Json(request): Json<UpdateApiKeyStatusRequest>,
) -> Result<Json<ApiKeyGroupRecord>, (StatusCode, Json<ErrorResponse>)> {
    match set_portal_api_key_group_active(
        state.store.as_ref(),
        &claims.claims().sub,
        &group_id,
        request.active,
    )
    .await
    .map_err(portal_error_response)?
    {
        Some(group) => Ok(Json(group)),
        None => Err(portal_error_response(PortalIdentityError::NotFound(
            "api key group not found".to_owned(),
        ))),
    }
}

async fn delete_api_key_group_handler(
    claims: AuthenticatedPortalClaims,
    Path(group_id): Path<String>,
    State(state): State<PortalApiState>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let deleted =
        delete_portal_api_key_group(state.store.as_ref(), &claims.claims().sub, &group_id)
            .await
            .map_err(portal_error_response)?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(portal_error_response(PortalIdentityError::NotFound(
            "api key group not found".to_owned(),
        )))
    }
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
        request.api_key.as_deref(),
        request.notes.as_deref(),
        request.api_key_group_id.as_deref(),
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
    let billing_summary = load_project_billing_summary(state.store.as_ref(), &workspace).await?;
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

async fn gateway_rate_limit_snapshot_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalGatewayRateLimitSnapshot>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let policies = state
        .store
        .list_rate_limit_policies_for_project(&workspace.project.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let windows = state
        .store
        .list_rate_limit_window_snapshots_for_project(&workspace.project.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let active_policy_count = policies.iter().filter(|policy| policy.enabled).count();
    let window_count = windows.len();
    let exceeded_window_count = windows.iter().filter(|window| window.exceeded).count();
    let headline = if policies.is_empty() {
        "No rate-limit policies configured yet".to_owned()
    } else if exceeded_window_count > 0 {
        "Rate-limit pressure is visible on the current project".to_owned()
    } else if active_policy_count > 0 {
        "Rate-limit posture is within configured headroom".to_owned()
    } else {
        "Rate-limit policies exist but are currently disabled".to_owned()
    };
    let detail = if policies.is_empty() {
        "The workspace has no visible project-scoped rate-limit policy yet, so the gateway still relies on the default protection surface.".to_owned()
    } else if exceeded_window_count > 0 {
        format!(
            "{} window(s) are currently over limit across {} policy record(s), so the portal is surfacing the live pressure state instead of waiting for a later audit.",
            exceeded_window_count, policies.len()
        )
    } else {
        format!(
            "{} active policy record(s) and {} live window snapshot(s) are currently within the configured limit posture for project {}.",
            active_policy_count,
            window_count,
            workspace.project.id
        )
    };

    Ok(Json(PortalGatewayRateLimitSnapshot {
        project_id: workspace.project.id,
        policy_count: policies.len(),
        active_policy_count,
        window_count,
        exceeded_window_count,
        headline,
        detail,
        generated_at_ms: current_time_millis(),
        policies,
        windows,
    }))
}

async fn gateway_traffic_pressure_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalGatewayTrafficPressureResponse>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let mut snapshots = state
        .traffic_controller
        .list_pressure_snapshots()
        .into_iter()
        .filter(|snapshot| snapshot.project_id == workspace.project.id)
        .collect::<Vec<_>>();
    sort_traffic_pressure_snapshots(&mut snapshots);

    let throttled_api_keys = snapshots
        .iter()
        .filter(|snapshot| {
            snapshot.scope_kind == CommercialPressureScopeKind::ApiKey && snapshot.saturated
        })
        .cloned()
        .collect::<Vec<_>>();
    let saturated_providers = snapshots
        .iter()
        .filter(|snapshot| {
            snapshot.scope_kind == CommercialPressureScopeKind::Provider && snapshot.saturated
        })
        .cloned()
        .collect::<Vec<_>>();
    let saturated_snapshot_count = snapshots
        .iter()
        .filter(|snapshot| snapshot.saturated)
        .count();

    let headline = if snapshots.is_empty() {
        "No live gateway pressure is currently visible".to_owned()
    } else if saturated_snapshot_count > 0 {
        "Gateway traffic is pressing against live admission limits".to_owned()
    } else {
        "Gateway traffic is flowing within configured headroom".to_owned()
    };
    let detail = if snapshots.is_empty() {
        format!(
            "Project {} has no live commercial admission counters yet, so the portal currently only needs the persisted gateway policy view.",
            workspace.project.id
        )
    } else if saturated_snapshot_count > 0 {
        format!(
            "Project {} currently has {} live pressure snapshot(s); {} API key scope(s) are throttled and {} provider scope(s) are saturated.",
            workspace.project.id,
            saturated_snapshot_count,
            throttled_api_keys.len(),
            saturated_providers.len()
        )
    } else {
        format!(
            "Project {} currently has {} live pressure snapshot(s), and every scope still has remaining admission headroom.",
            workspace.project.id,
            snapshots.len()
        )
    };

    Ok(Json(PortalGatewayTrafficPressureResponse {
        project_id: workspace.project.id,
        snapshot_count: snapshots.len(),
        saturated_snapshot_count,
        throttled_api_key_count: throttled_api_keys.len(),
        saturated_provider_count: saturated_providers.len(),
        headline,
        detail,
        generated_at_ms: current_time_millis(),
        snapshots,
        throttled_api_keys,
        saturated_providers,
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
    load_project_billing_summary(state.store.as_ref(), &workspace)
        .await
        .map(Json)
}

async fn list_billing_ledger_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<LedgerEntry>>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let ledger = state
        .store
        .list_ledger_entries_for_project(&workspace.project.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ledger))
}

async fn list_billing_events_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<BillingEventRecord>>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    load_project_billing_events(
        state.store.as_ref(),
        &workspace.tenant.id,
        &workspace.project.id,
    )
    .await
    .map(Json)
}

async fn billing_events_summary_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<BillingEventSummary>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let events = load_project_billing_events(
        state.store.as_ref(),
        &workspace.tenant.id,
        &workspace.project.id,
    )
    .await?;
    Ok(Json(summarize_billing_events(&events)))
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

async fn list_routing_profiles_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<RoutingProfileRecord>>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    list_routing_profiles(state.store.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .map(|profiles| {
            profiles
                .into_iter()
                .filter(|profile| {
                    profile.tenant_id == workspace.tenant.id
                        && profile.project_id == workspace.project.id
                })
                .collect::<Vec<_>>()
        })
        .map(Json)
}

async fn list_routing_snapshots_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<CompiledRoutingSnapshotRecord>>, StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    list_compiled_routing_snapshots(state.store.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .map(|snapshots| {
            snapshots
                .into_iter()
                .filter(|snapshot| {
                    snapshot.tenant_id.as_deref() == Some(workspace.tenant.id.as_str())
                        && snapshot.project_id.as_deref() == Some(workspace.project.id.as_str())
                })
                .collect::<Vec<_>>()
        })
        .map(Json)
}

async fn create_routing_profile_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
    Json(request): Json<CreatePortalRoutingProfileRequest>,
) -> Result<(StatusCode, Json<RoutingProfileRecord>), StatusCode> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub).await?;
    let normalized_name = normalize_portal_routing_profile_name(&request.name)?;
    let normalized_slug =
        normalize_portal_routing_profile_slug(&normalized_name, request.slug.as_deref())?;
    let profile_id = format!(
        "routing-profile-{}-{}",
        normalized_slug,
        current_time_millis()
    );

    let profile = create_routing_profile(CreateRoutingProfileInput {
        profile_id: &profile_id,
        tenant_id: &workspace.tenant.id,
        project_id: &workspace.project.id,
        name: &normalized_name,
        slug: &normalized_slug,
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

fn portal_commerce_error_response(error: CommerceError) -> (StatusCode, Json<ErrorResponse>) {
    let status = match error {
        CommerceError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        CommerceError::NotFound(_) => StatusCode::NOT_FOUND,
        CommerceError::Conflict(_) => StatusCode::CONFLICT,
        CommerceError::Forbidden(_) => StatusCode::FORBIDDEN,
        CommerceError::Storage(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };
    let body = ErrorResponse {
        error: ErrorBody {
            message: error.to_string(),
        },
    };
    (status, Json(body))
}

fn portal_payment_error_response(error: PaymentError) -> (StatusCode, Json<ErrorResponse>) {
    let status = match error {
        PaymentError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        PaymentError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
        PaymentError::NotFound(_) => StatusCode::NOT_FOUND,
        PaymentError::Conflict(_) => StatusCode::CONFLICT,
        PaymentError::Storage(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };
    let body = ErrorResponse {
        error: ErrorBody {
            message: error.to_string(),
        },
    };
    (status, Json(body))
}

fn authorize_portal_payment_callback(
    state: &PortalApiState,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let Some(expected_secret) = state.payment_callback_secret.as_deref() else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: ErrorBody {
                    message: "payment provider callbacks are not enabled".to_owned(),
                },
            }),
        ));
    };

    let provided_secret = headers
        .get(PORTAL_PAYMENT_CALLBACK_SECRET_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty());

    if provided_secret == Some(expected_secret) {
        return Ok(());
    }

    Err((
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
            error: ErrorBody {
                message: "invalid payment callback credentials".to_owned(),
            },
        }),
    ))
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
    let usage_records = store
        .list_usage_records_for_project(project_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(usage_records)
}

#[derive(Debug, Clone, Copy)]
struct PortalCanonicalBillingOverlay {
    account_id: u64,
    available_balance: f64,
    held_balance: f64,
    grant_balance: f64,
    consumed_balance: f64,
}

async fn load_project_billing_summary(
    store: &dyn AdminStore,
    workspace: &PortalWorkspaceSummary,
) -> Result<ProjectBillingSummary, StatusCode> {
    let ledger = store
        .list_ledger_entries_for_project(&workspace.project.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let policies = store
        .list_quota_policies_for_project(&workspace.project.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let billing = summarize_billing_snapshot(&ledger, &policies);
    let mut summary = billing
        .projects
        .into_iter()
        .next()
        .unwrap_or_else(|| ProjectBillingSummary::new(workspace.project.id.clone()));

    if let Some(overlay) = load_portal_canonical_billing_overlay(store, workspace).await? {
        summary = apply_portal_canonical_billing_overlay(summary, overlay);
    }

    Ok(summary)
}

async fn load_portal_canonical_billing_overlay(
    store: &dyn AdminStore,
    workspace: &PortalWorkspaceSummary,
) -> Result<Option<PortalCanonicalBillingOverlay>, StatusCode> {
    let Some(identity_store) = store.identity_kernel() else {
        return Ok(None);
    };
    let Some(account_store) = store.account_kernel() else {
        return Ok(None);
    };

    let binding_subject = portal_workspace_binding_subject(workspace);
    let Some(binding) = identity_store
        .find_identity_binding_record(
            PORTAL_WORKSPACE_IDENTITY_BINDING_TYPE,
            Some(PORTAL_WORKSPACE_IDENTITY_BINDING_ISSUER),
            Some(binding_subject.as_str()),
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Ok(None);
    };

    let Some(account) = account_store
        .find_account_record_by_owner(
            binding.tenant_id,
            binding.organization_id,
            binding.user_id,
            AccountType::Primary,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    else {
        return Ok(None);
    };

    let snapshot =
        summarize_account_balance(account_store, account.account_id, current_time_millis())
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Some(PortalCanonicalBillingOverlay {
        account_id: account.account_id,
        available_balance: snapshot.available_balance,
        held_balance: snapshot.held_balance,
        grant_balance: snapshot.grant_balance,
        consumed_balance: snapshot.consumed_balance,
    }))
}

fn apply_portal_canonical_billing_overlay(
    mut summary: ProjectBillingSummary,
    overlay: PortalCanonicalBillingOverlay,
) -> ProjectBillingSummary {
    if summary.quota_remaining_units.is_none() {
        summary.quota_remaining_units = summary.remaining_units;
    }
    summary.balance_source = Some("canonical_account".to_owned());
    summary.remaining_units = Some(effective_remaining_units_from_balance(
        overlay.available_balance,
    ));
    summary.canonical_account_id = Some(overlay.account_id);
    summary.canonical_available_balance = Some(overlay.available_balance);
    summary.canonical_held_balance = Some(overlay.held_balance);
    summary.canonical_grant_balance = Some(overlay.grant_balance);
    summary.canonical_consumed_balance = Some(overlay.consumed_balance);
    summary.exhausted = overlay.available_balance <= f64::EPSILON;
    summary
}

fn portal_workspace_binding_subject(workspace: &PortalWorkspaceSummary) -> String {
    format!(
        "{}:{}:{}",
        workspace.tenant.id, workspace.project.id, workspace.user.id
    )
}

fn effective_remaining_units_from_balance(balance: f64) -> u64 {
    if !balance.is_finite() || balance <= 0.0 {
        0
    } else {
        balance.floor() as u64
    }
}

async fn load_project_billing_events(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
) -> Result<Vec<BillingEventRecord>, StatusCode> {
    let events = list_billing_events(store)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(events
        .into_iter()
        .filter(|event| event.tenant_id == tenant_id && event.project_id == project_id)
        .collect())
}

fn current_time_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn sort_traffic_pressure_snapshots(snapshots: &mut [TrafficPressureSnapshot]) {
    snapshots.sort_by(|left, right| {
        left.project_id
            .cmp(&right.project_id)
            .then_with(|| {
                scope_kind_order(left.scope_kind).cmp(&scope_kind_order(right.scope_kind))
            })
            .then_with(|| left.scope_key.cmp(&right.scope_key))
            .then_with(|| left.policy_id.cmp(&right.policy_id))
    });
}

fn scope_kind_order(kind: CommercialPressureScopeKind) -> u8 {
    match kind {
        CommercialPressureScopeKind::Project => 0,
        CommercialPressureScopeKind::ApiKey => 1,
        CommercialPressureScopeKind::Provider => 2,
    }
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
    let logs = store
        .list_routing_decision_logs_for_project(project_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(logs)
}

async fn load_latest_route_hint(
    store: &dyn AdminStore,
    project_id: &str,
) -> Result<(String, String), StatusCode> {
    if let Some(log) = store
        .find_latest_routing_decision_log_for_project(project_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        return Ok((log.capability.clone(), log.route_key.clone()));
    }

    if let Some(record) = store
        .find_latest_usage_record_for_project(project_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        return Ok(("chat_completion".to_owned(), record.model.clone()));
    }

    if let Some(model) = store
        .find_any_model()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        return Ok(("chat_completion".to_owned(), model.external_name.clone()));
    }

    Ok(("chat_completion".to_owned(), "gpt-4.1".to_owned()))
}

async fn load_routing_provider_options(
    store: &dyn AdminStore,
    model: &str,
    preferences: &ProjectRoutingPreferences,
) -> Result<Vec<PortalRoutingProviderOption>, StatusCode> {
    let mut providers = store
        .list_providers_for_model(model)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .collect::<Vec<_>>();

    if providers.is_empty() {
        providers = store
            .list_providers()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    let preference_ranks = provider_preference_ranks(preferences);
    let preferred_provider_ids = preferences
        .ordered_provider_ids
        .iter()
        .cloned()
        .collect::<HashSet<_>>();
    sort_routing_provider_options(&mut providers, &preference_ranks);

    Ok(providers
        .into_iter()
        .map(|provider| PortalRoutingProviderOption {
            preferred: preferred_provider_ids.contains(&provider.id),
            default_provider: preferences.default_provider_id.as_deref() == Some(&provider.id),
            provider_id: provider.id,
            display_name: provider.display_name,
            channel_id: provider.channel_id,
        })
        .collect())
}

fn sort_routing_provider_options(
    providers: &mut [ProxyProvider],
    preference_ranks: &HashMap<String, usize>,
) {
    providers.sort_by(|left, right| {
        provider_preference_rank(preference_ranks, &left.id)
            .cmp(&provider_preference_rank(preference_ranks, &right.id))
            .then_with(|| left.display_name.cmp(&right.display_name))
            .then_with(|| left.id.cmp(&right.id))
    });
}

fn provider_preference_ranks(preferences: &ProjectRoutingPreferences) -> HashMap<String, usize> {
    preferences
        .ordered_provider_ids
        .iter()
        .enumerate()
        .map(|(index, provider_id)| (provider_id.clone(), index))
        .collect()
}

fn provider_preference_rank(preference_ranks: &HashMap<String, usize>, provider_id: &str) -> usize {
    preference_ranks
        .get(provider_id)
        .copied()
        .unwrap_or(usize::MAX)
}

fn normalize_portal_routing_profile_name(name: &str) -> Result<String, StatusCode> {
    let normalized = name.trim();
    if normalized.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(normalized.to_owned())
}

fn normalize_portal_routing_profile_slug(
    name: &str,
    slug: Option<&str>,
) -> Result<String, StatusCode> {
    let source = normalize_portal_routing_profile_optional_value(slug).unwrap_or(name.to_owned());
    let mut normalized = String::new();
    let mut previous_was_dash = false;

    for ch in source.chars() {
        if ch.is_ascii_alphanumeric() {
            normalized.push(ch.to_ascii_lowercase());
            previous_was_dash = false;
        } else if !normalized.is_empty() && !previous_was_dash {
            normalized.push('-');
            previous_was_dash = true;
        }
    }

    while normalized.ends_with('-') {
        normalized.pop();
    }

    if normalized.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(normalized)
}

fn normalize_portal_routing_profile_optional_value(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}
