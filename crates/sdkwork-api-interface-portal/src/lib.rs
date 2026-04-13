use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::sync::Arc;

mod auth;
mod auth_types;
mod billing;
mod commerce;
mod gateway;
mod http;
mod jobs;
mod marketing;
mod marketing_types;
mod openapi;
mod routing;
mod state;
mod workspace;

pub(crate) use auth_types::*;
pub(crate) use commerce::{PortalCommerceReconciliationSummary, PortalOrderCenterEntry};
pub(crate) use marketing_types::*;
pub use state::PortalApiState;
pub(crate) use state::{AuthenticatedPortalClaims, DEFAULT_PORTAL_JWT_SIGNING_SECRET};

use axum::{
    body::Bytes,
    extract::{FromRequestParts, Path, Query, State},
    http::{header, request::Parts, HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse},
    routing::{delete, get, patch, post},
    Json, Router,
};
use sdkwork_api_app_billing::{
    list_account_ledger_history, list_billing_events, summarize_account_balance,
    summarize_billing_events, summarize_billing_snapshot, synchronize_due_pricing_plan_lifecycle,
    AccountBalanceSnapshot, AccountLedgerHistoryEntry, AccountLotBalanceSnapshot,
    CommercialBillingAdminKernel,
};
use sdkwork_api_app_commerce::{
    apply_portal_commerce_payment_event_with_billing, cancel_portal_commerce_order,
    create_portal_commerce_payment_attempt, list_portal_commerce_payment_attempts,
    list_portal_commerce_payment_methods, list_project_commerce_orders,
    load_portal_commerce_catalog, load_portal_commerce_checkout_session_with_policy,
    load_portal_commerce_order, load_portal_commerce_payment_attempt, load_project_membership,
    portal_commerce_product_kind, portal_commerce_transaction_kind, preview_portal_commerce_quote,
    process_portal_stripe_webhook, reclaim_expired_coupon_reservations_for_code_if_needed,
    settle_portal_commerce_order_with_billing, submit_portal_commerce_order, CommerceError,
    CommercePaymentAttemptRecord, PaymentMethodRecord, PortalCommerceCatalog,
    PortalCommerceCheckoutSession, PortalCommerceOrderRecord,
    PortalCommercePaymentAttemptCreateRequest, PortalCommercePaymentEventRecord,
    PortalCommercePaymentEventRequest, PortalCommerceQuote, PortalCommerceQuoteRequest,
    PortalCommerceWebhookAck, PortalProjectMembershipRecord,
};
use sdkwork_api_app_credential::CredentialSecretManager;
use sdkwork_api_app_identity::{
    change_portal_password, create_portal_api_key_group, create_portal_api_key_with_metadata,
    delete_portal_api_key, delete_portal_api_key_group, gateway_auth_subject_from_request_context,
    list_portal_api_key_groups, list_portal_api_keys, load_portal_user_profile,
    load_portal_workspace_summary, login_portal_user, register_portal_user,
    set_portal_api_key_active, set_portal_api_key_group_active, update_portal_api_key_group,
    verify_portal_jwt, CreatedGatewayApiKey, GatewayRequestContext, PortalApiKeyGroupInput,
    PortalAuthSession, PortalClaims, PortalIdentityError, PortalWorkspaceSummary,
};
use sdkwork_api_app_jobs::{
    find_async_job, list_async_job_assets, list_async_job_attempts, list_async_jobs,
};
use sdkwork_api_app_marketing::{
    confirm_coupon_redemption, reserve_coupon_redemption, rollback_coupon_redemption,
    validate_coupon_stack, CouponValidationDecision,
};
use sdkwork_api_app_payment::{
    ensure_commerce_payment_checkout, ensure_portal_payment_subject_scope,
    list_portal_commerce_order_payment_events, list_project_commerce_order_center,
    request_portal_commerce_order_refund, CommerceOrderCenterEntry, PaymentAttemptTimelineEntry,
    PortalAccountHistorySnapshot,
};
use sdkwork_api_app_rate_limit::{
    check_coupon_rate_limit, coupon_actor_bucket, CouponRateLimitAction,
};
use sdkwork_api_app_routing::{
    create_routing_profile, list_compiled_routing_snapshots, list_routing_profiles,
    persist_routing_profile, select_route_with_store_context,
    simulate_route_with_store_selection_context, CreateRoutingProfileInput, RouteSelectionContext,
};
use sdkwork_api_app_usage::summarize_usage_records;
use sdkwork_api_config::HttpExposureConfig;
use sdkwork_api_domain_billing::{
    AccountBenefitLotRecord, AccountHoldRecord, AccountLedgerAllocationRecord,
    AccountLedgerEntryRecord, AccountRecord, AccountType, BillingEventRecord, BillingEventSummary,
    LedgerEntry, PricingPlanRecord, PricingRateRecord, ProjectBillingSummary,
    RequestSettlementRecord,
};
use sdkwork_api_domain_catalog::ProxyProvider;
use sdkwork_api_domain_identity::{ApiKeyGroupRecord, GatewayApiKeyRecord, PortalUserProfile};
use sdkwork_api_domain_jobs::{AsyncJobAssetRecord, AsyncJobAttemptRecord, AsyncJobRecord};
use sdkwork_api_domain_marketing::{
    CampaignBudgetRecord, CampaignBudgetStatus, CouponCodeRecord, CouponCodeStatus,
    CouponDistributionKind, CouponRedemptionRecord, CouponRedemptionStatus,
    CouponReservationRecord, CouponRollbackRecord, CouponRollbackType, CouponTemplateRecord,
    MarketingCampaignRecord, MarketingSubjectScope,
};
use sdkwork_api_domain_payment::{
    PaymentCallbackEventRecord, PaymentOrderRecord, PaymentSessionRecord, PaymentTransactionRecord,
    RefundOrderRecord,
};
use sdkwork_api_domain_rate_limit::{RateLimitPolicy, RateLimitWindowSnapshot};
use sdkwork_api_domain_routing::{
    CompiledRoutingSnapshotRecord, ProjectRoutingPreferences, RoutingDecision, RoutingDecisionLog,
    RoutingDecisionSource, RoutingProfileRecord, RoutingStrategy,
};
use sdkwork_api_domain_usage::{UsageRecord, UsageSummary};
use sdkwork_api_observability::{observe_http_metrics, observe_http_tracing, HttpMetricsRegistry};
use sdkwork_api_storage_core::{
    AdminStore, AtomicCouponConfirmationCommand, AtomicCouponReservationCommand,
    AtomicCouponRollbackCommand, CommercialKernelStore, IdentityKernelStore, Reloadable,
};
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;

#[derive(Debug, Deserialize)]
struct CreateCommerceOrderRefundRequest {
    refund_reason_code: String,
    requested_amount_minor: u64,
}

#[derive(Debug, Serialize)]
struct PortalPaymentAttemptTimelineEntryResponse {
    attempt: sdkwork_api_domain_payment::PaymentAttemptRecord,
    sessions: Vec<PaymentSessionRecord>,
}

#[derive(Debug, Serialize)]
struct PortalCommerceOrderCenterEntryResponse {
    order: commerce::PortalCommerceOrderView,
    payment_events: Vec<PortalCommercePaymentEventRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    latest_payment_event: Option<PortalCommercePaymentEventRecord>,
    checkout_session: PortalCommerceCheckoutSession,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    payment_order: Option<PaymentOrderRecord>,
    payment_attempts: Vec<PortalPaymentAttemptTimelineEntryResponse>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    active_payment_session: Option<PaymentSessionRecord>,
    payment_transactions: Vec<PaymentTransactionRecord>,
    refunds: Vec<RefundOrderRecord>,
    refundable_amount_minor: u64,
}

#[derive(Debug, Serialize)]
struct PortalCommerceOrderCenterResponse {
    project_id: String,
    payment_simulation_enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    membership: Option<PortalProjectMembershipRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reconciliation: Option<PortalCommerceReconciliationSummary>,
    orders: Vec<PortalCommerceOrderCenterEntryResponse>,
}

#[derive(Debug, Serialize)]
struct PortalAccountLotBalanceSnapshotResponse {
    lot_id: u64,
    benefit_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    scope_json: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    expires_at_ms: Option<u64>,
    original_quantity: f64,
    remaining_quantity: f64,
    held_quantity: f64,
    available_quantity: f64,
}

#[derive(Debug, Serialize)]
struct PortalAccountBalanceSnapshotResponse {
    account_id: u64,
    available_balance: f64,
    held_balance: f64,
    consumed_balance: f64,
    grant_balance: f64,
    active_lot_count: u64,
    lots: Vec<PortalAccountLotBalanceSnapshotResponse>,
}

#[derive(Debug, Serialize)]
struct PortalAccountHistoryResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    account: Option<AccountRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    balance: Option<PortalAccountBalanceSnapshotResponse>,
    benefit_lots: Vec<AccountBenefitLotRecord>,
    holds: Vec<AccountHoldRecord>,
    request_settlements: Vec<RequestSettlementRecord>,
    ledger: Vec<AccountLedgerHistoryEntry>,
    lots: Vec<AccountBenefitLotRecord>,
    ledger_entries: Vec<AccountLedgerEntryRecord>,
    ledger_allocations: Vec<AccountLedgerAllocationRecord>,
    refunds: Vec<RefundOrderRecord>,
}

pub fn try_portal_router() -> anyhow::Result<Router> {
    let service_name: Arc<str> = Arc::from("portal");
    let metrics = Arc::new(HttpMetricsRegistry::new("portal"));
    let http_exposure = http::http_exposure_config()?;
    Ok(Router::new()
        .merge(openapi::portal_docs_router())
        .route(
            "/metrics",
            http::metrics_route(metrics.clone(), &http_exposure),
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
        .route(
            "/portal/marketing/coupon-validations",
            post(|| async { "marketing-coupon-validations" }),
        )
        .route(
            "/portal/marketing/coupon-reservations",
            post(|| async { "marketing-coupon-reservations" }),
        )
        .route(
            "/portal/marketing/coupon-redemptions/confirm",
            post(|| async { "marketing-coupon-redemptions-confirm" }),
        )
        .route(
            "/portal/marketing/coupon-redemptions/rollback",
            post(|| async { "marketing-coupon-redemptions-rollback" }),
        )
        .route(
            "/portal/marketing/my-coupons",
            get(|| async { "marketing-my-coupons" }),
        )
        .route(
            "/portal/marketing/reward-history",
            get(|| async { "marketing-reward-history" }),
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
            "/portal/commerce/orders/{order_id}",
            get(|| async { "commerce-order" }),
        )
        .route(
            "/portal/commerce/order-center",
            get(|| async { "commerce-order-center" }),
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
            get(|| async { "commerce-order-payment-events" })
                .post(|| async { "commerce-order-payment-events" }),
        )
        .route(
            "/portal/commerce/orders/{order_id}/payment-methods",
            get(|| async { "commerce-order-payment-methods" }),
        )
        .route(
            "/portal/commerce/orders/{order_id}/checkout-session",
            get(|| async { "commerce-order-checkout-session" }),
        )
        .route(
            "/portal/commerce/payment-attempts/{payment_attempt_id}",
            get(|| async { "commerce-payment-attempt" }),
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
            "/portal/billing/account",
            get(|| async { "billing-account" }),
        )
        .route(
            "/portal/billing/account-history",
            get(|| async { "billing-account-history" }),
        )
        .route(
            "/portal/billing/account/balance",
            get(|| async { "billing-account-balance" }),
        )
        .route(
            "/portal/billing/account/benefit-lots",
            get(|| async { "billing-account-benefit-lots" }),
        )
        .route(
            "/portal/billing/account/holds",
            get(|| async { "billing-account-holds" }),
        )
        .route(
            "/portal/billing/account/request-settlements",
            get(|| async { "billing-account-request-settlements" }),
        )
        .route(
            "/portal/billing/account/ledger",
            get(|| async { "billing-account-ledger" }),
        )
        .route(
            "/portal/billing/pricing-plans",
            get(|| async { "billing-pricing-plans" }),
        )
        .route(
            "/portal/billing/pricing-rates",
            get(|| async { "billing-pricing-rates" }),
        )
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
        .layer(axum::middleware::from_fn_with_state(
            metrics,
            observe_http_metrics,
        ))
        .layer(http::browser_cors_layer(&http_exposure))
        .layer(axum::middleware::from_fn_with_state(
            service_name,
            observe_http_tracing,
        )))
}

pub fn portal_router() -> Router {
    try_portal_router().expect("http exposure config should load from process env")
}

pub fn try_portal_router_with_pool(pool: SqlitePool) -> anyhow::Result<Router> {
    try_portal_router_with_pool_and_jwt_secret(pool, DEFAULT_PORTAL_JWT_SIGNING_SECRET)
}

pub fn portal_router_with_pool(pool: SqlitePool) -> Router {
    try_portal_router_with_pool(pool).expect("http exposure config should load from process env")
}

pub fn try_portal_router_with_pool_and_jwt_secret(
    pool: SqlitePool,
    jwt_signing_secret: impl Into<String>,
) -> anyhow::Result<Router> {
    try_portal_router_with_store_and_jwt_secret(
        Arc::new(SqliteAdminStore::new(pool)),
        jwt_signing_secret,
    )
}

pub fn portal_router_with_pool_and_jwt_secret(
    pool: SqlitePool,
    jwt_signing_secret: impl Into<String>,
) -> Router {
    try_portal_router_with_pool_and_jwt_secret(pool, jwt_signing_secret)
        .expect("http exposure config should load from process env")
}

pub fn try_portal_router_with_store<S>(store: Arc<S>) -> anyhow::Result<Router>
where
    S: AdminStore
        + CommercialBillingAdminKernel
        + CommercialKernelStore
        + IdentityKernelStore
        + 'static,
{
    try_portal_router_with_store_and_jwt_secret(store, DEFAULT_PORTAL_JWT_SIGNING_SECRET)
}

pub fn portal_router_with_store<S>(store: Arc<S>) -> Router
where
    S: AdminStore
        + CommercialBillingAdminKernel
        + CommercialKernelStore
        + IdentityKernelStore
        + 'static,
{
    try_portal_router_with_store(store).expect("http exposure config should load from process env")
}

pub fn try_portal_router_with_store_and_jwt_secret<S>(
    store: Arc<S>,
    jwt_signing_secret: impl Into<String>,
) -> anyhow::Result<Router>
where
    S: AdminStore
        + CommercialBillingAdminKernel
        + CommercialKernelStore
        + IdentityKernelStore
        + 'static,
{
    try_portal_router_with_state(PortalApiState::with_store_and_jwt_secret(
        store,
        jwt_signing_secret,
    ))
}

pub fn portal_router_with_store_and_jwt_secret<S>(
    store: Arc<S>,
    jwt_signing_secret: impl Into<String>,
) -> Router
where
    S: AdminStore
        + CommercialBillingAdminKernel
        + CommercialKernelStore
        + IdentityKernelStore
        + 'static,
{
    try_portal_router_with_store_and_jwt_secret(store, jwt_signing_secret)
        .expect("http exposure config should load from process env")
}

pub fn try_portal_router_with_state(state: PortalApiState) -> anyhow::Result<Router> {
    Ok(portal_router_with_state_and_http_exposure(
        state,
        http::http_exposure_config()?,
    ))
}

pub fn portal_router_with_state(state: PortalApiState) -> Router {
    try_portal_router_with_state(state).expect("http exposure config should load from process env")
}

pub fn portal_router_with_state_and_http_exposure(
    state: PortalApiState,
    http_exposure: HttpExposureConfig,
) -> Router {
    let service_name: Arc<str> = Arc::from("portal");
    let metrics = Arc::new(HttpMetricsRegistry::new("portal"));
    Router::new()
        .merge(openapi::portal_docs_router())
        .route(
            "/metrics",
            http::metrics_route(metrics.clone(), &http_exposure),
        )
        .route("/portal/health", get(|| async { "ok" }))
        .route("/portal/auth/register", post(auth::register_handler))
        .route("/portal/auth/login", post(auth::login_handler))
        .route("/portal/auth/me", get(auth::me_handler))
        .route(
            "/portal/auth/change-password",
            post(auth::change_password_handler),
        )
        .route("/portal/dashboard", get(workspace::dashboard_handler))
        .route("/portal/workspace", get(workspace::workspace_handler))
        .route(
            "/portal/marketing/coupon-validations",
            post(marketing::validate_marketing_coupon_handler),
        )
        .route(
            "/portal/marketing/coupon-reservations",
            post(marketing::reserve_marketing_coupon_handler),
        )
        .route(
            "/portal/marketing/coupon-redemptions/confirm",
            post(marketing::confirm_marketing_coupon_redemption_handler),
        )
        .route(
            "/portal/marketing/coupon-redemptions/rollback",
            post(marketing::rollback_marketing_coupon_redemption_handler),
        )
        .route(
            "/portal/marketing/my-coupons",
            get(marketing::list_my_coupons_handler),
        )
        .route(
            "/portal/marketing/reward-history",
            get(marketing::list_marketing_reward_history_handler),
        )
        .route(
            "/portal/marketing/redemptions",
            get(marketing::list_marketing_redemptions_handler),
        )
        .route(
            "/portal/marketing/codes",
            get(marketing::list_marketing_codes_handler),
        )
        .route("/portal/async-jobs", get(jobs::list_async_jobs_handler))
        .route(
            "/portal/async-jobs/{job_id}/attempts",
            get(jobs::list_async_job_attempts_handler),
        )
        .route(
            "/portal/async-jobs/{job_id}/assets",
            get(jobs::list_async_job_assets_handler),
        )
        .route(
            "/portal/gateway/rate-limit-snapshot",
            get(workspace::gateway_rate_limit_snapshot_handler),
        )
        .route(
            "/portal/commerce/catalog",
            get(commerce::commerce_catalog_handler),
        )
        .route(
            "/portal/commerce/quote",
            post(commerce::commerce_quote_handler),
        )
        .route(
            "/portal/commerce/orders",
            get(commerce::list_commerce_orders_handler)
                .post(commerce::create_commerce_order_handler),
        )
        .route(
            "/portal/commerce/orders/{order_id}",
            get(commerce::get_commerce_order_handler),
        )
        .route(
            "/portal/commerce/order-center",
            get(list_commerce_order_center_handler),
        )
        .route(
            "/portal/commerce/orders/{order_id}/settle",
            post(commerce::settle_commerce_order_handler),
        )
        .route(
            "/portal/commerce/orders/{order_id}/cancel",
            post(commerce::cancel_commerce_order_handler),
        )
        .route(
            "/portal/commerce/orders/{order_id}/refunds",
            post(create_commerce_order_refund_handler),
        )
        .route(
            "/portal/commerce/orders/{order_id}/payment-events",
            get(list_commerce_payment_events_handler)
                .post(commerce::apply_commerce_payment_event_handler),
        )
        .route(
            "/portal/commerce/orders/{order_id}/payment-methods",
            get(commerce::list_commerce_payment_methods_handler),
        )
        .route(
            "/portal/commerce/orders/{order_id}/payment-attempts",
            get(commerce::list_commerce_payment_attempts_handler)
                .post(commerce::create_commerce_payment_attempt_handler),
        )
        .route(
            "/portal/commerce/orders/{order_id}/checkout-session",
            get(commerce::get_commerce_checkout_session_handler),
        )
        .route(
            "/portal/commerce/payment-attempts/{payment_attempt_id}",
            get(commerce::get_commerce_payment_attempt_handler),
        )
        .route(
            "/portal/commerce/webhooks/stripe/{payment_method_id}",
            post(commerce::stripe_webhook_handler),
        )
        .route(
            "/portal/commerce/membership",
            get(commerce::get_project_membership_handler),
        )
        .route(
            "/portal/api-keys",
            get(gateway::list_api_keys_handler).post(gateway::create_api_key_handler),
        )
        .route(
            "/portal/api-keys/{hashed_key}/status",
            post(gateway::update_api_key_status_handler),
        )
        .route(
            "/portal/api-keys/{hashed_key}",
            delete(gateway::delete_api_key_handler),
        )
        .route(
            "/portal/api-key-groups",
            get(gateway::list_api_key_groups_handler).post(gateway::create_api_key_group_handler),
        )
        .route(
            "/portal/api-key-groups/{group_id}",
            patch(gateway::update_api_key_group_handler)
                .delete(gateway::delete_api_key_group_handler),
        )
        .route(
            "/portal/api-key-groups/{group_id}/status",
            post(gateway::update_api_key_group_status_handler),
        )
        .route(
            "/portal/usage/records",
            get(workspace::list_usage_records_handler),
        )
        .route(
            "/portal/usage/summary",
            get(workspace::usage_summary_handler),
        )
        .route(
            "/portal/billing/account",
            get(billing::billing_account_handler),
        )
        .route(
            "/portal/billing/account-history",
            get(account_history_handler),
        )
        .route(
            "/portal/billing/account/balance",
            get(billing::billing_account_balance_handler),
        )
        .route(
            "/portal/billing/account/benefit-lots",
            get(billing::list_billing_account_benefit_lots_handler),
        )
        .route(
            "/portal/billing/account/holds",
            get(billing::list_billing_account_holds_handler),
        )
        .route(
            "/portal/billing/account/request-settlements",
            get(billing::list_billing_request_settlements_handler),
        )
        .route(
            "/portal/billing/account/ledger",
            get(billing::list_billing_account_ledger_handler),
        )
        .route(
            "/portal/billing/pricing-plans",
            get(billing::list_billing_pricing_plans_handler),
        )
        .route(
            "/portal/billing/pricing-rates",
            get(billing::list_billing_pricing_rates_handler),
        )
        .route(
            "/portal/billing/summary",
            get(billing::billing_summary_handler),
        )
        .route(
            "/portal/billing/ledger",
            get(billing::list_billing_ledger_handler),
        )
        .route(
            "/portal/billing/events",
            get(billing::list_billing_events_handler),
        )
        .route(
            "/portal/billing/events/summary",
            get(billing::billing_events_summary_handler),
        )
        .route(
            "/portal/routing/summary",
            get(routing::routing_summary_handler),
        )
        .route(
            "/portal/routing/profiles",
            get(routing::list_routing_profiles_handler)
                .post(routing::create_routing_profile_handler),
        )
        .route(
            "/portal/routing/preferences",
            get(routing::get_routing_preferences_handler)
                .post(routing::save_routing_preferences_handler),
        )
        .route(
            "/portal/routing/snapshots",
            get(routing::list_routing_snapshots_handler),
        )
        .route(
            "/portal/routing/preview",
            post(routing::preview_routing_handler),
        )
        .route(
            "/portal/routing/decision-logs",
            get(routing::list_routing_decision_logs_handler),
        )
        .layer(axum::middleware::from_fn_with_state(
            metrics,
            observe_http_metrics,
        ))
        .layer(http::browser_cors_layer(&http_exposure))
        .layer(axum::middleware::from_fn_with_state(
            service_name,
            observe_http_tracing,
        ))
        .with_state(state)
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

fn payment_store_kernel(
    state: &PortalApiState,
) -> Result<&Arc<dyn CommercialKernelStore>, (StatusCode, Json<ErrorResponse>)> {
    state.payment_store.as_ref().ok_or_else(|| {
        error_response(
            StatusCode::NOT_IMPLEMENTED,
            "portal payment control plane is unavailable for the current storage runtime",
        )
    })
}

fn identity_store_kernel(
    state: &PortalApiState,
) -> Result<&Arc<dyn IdentityKernelStore>, (StatusCode, Json<ErrorResponse>)> {
    state.identity_store.as_ref().ok_or_else(|| {
        error_response(
            StatusCode::NOT_IMPLEMENTED,
            "portal payment identity bridge is unavailable for the current storage runtime",
        )
    })
}

async fn list_commerce_order_center_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalCommerceOrderCenterResponse>, (StatusCode, Json<ErrorResponse>)> {
    let payment_store = payment_store_kernel(&state)?;
    let identity_store = identity_store_kernel(&state)?;
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

    recover_project_order_checkout_artifacts(
        payment_store.as_ref(),
        identity_store.as_ref(),
        &workspace.project.id,
        current_time_millis(),
    )
    .await;

    let payment_entries = load_project_order_center(payment_store.as_ref(), &workspace.project.id)
        .await
        .map_err(portal_payment_error_response)?;
    let membership = load_project_membership(state.store.as_ref(), &workspace.project.id)
        .await
        .map_err(commerce::portal_commerce_error_response)?;

    let mut orders = Vec::with_capacity(payment_entries.len());
    let mut legacy_orders = Vec::with_capacity(payment_entries.len());
    for entry in payment_entries {
        let order_id = entry.order.order_id.clone();
        let mut payment_events = state
            .store
            .list_commerce_payment_events_for_order(&order_id)
            .await
            .map_err(CommerceError::from)
            .map_err(commerce::portal_commerce_error_response)?;
        payment_events.sort_by(|left, right| {
            right
                .received_at_ms
                .cmp(&left.received_at_ms)
                .then_with(|| right.payment_event_id.cmp(&left.payment_event_id))
        });
        let latest_payment_event = payment_events.first().cloned();
        let checkout_session = load_portal_commerce_checkout_session_with_policy(
            state.store.as_ref(),
            &claims.claims().sub,
            &workspace.project.id,
            &order_id,
            state.payment_simulation_enabled,
        )
        .await
        .map_err(commerce::portal_commerce_error_response)?;
        let order = commerce::PortalCommerceOrderView::from(entry.order);
        legacy_orders.push(PortalOrderCenterEntry {
            order: order.clone(),
            payment_events: payment_events.clone(),
            latest_payment_event: latest_payment_event.clone(),
            checkout_session: checkout_session.clone(),
        });
        orders.push(PortalCommerceOrderCenterEntryResponse {
            order,
            payment_events,
            latest_payment_event,
            checkout_session,
            payment_order: entry.payment_order,
            payment_attempts: entry
                .payment_attempts
                .into_iter()
                .map(PortalPaymentAttemptTimelineEntryResponse::from)
                .collect(),
            active_payment_session: entry.active_payment_session,
            payment_transactions: entry.payment_transactions,
            refunds: entry.refunds,
            refundable_amount_minor: entry.refundable_amount_minor,
        });
    }

    let reconciliation =
        load_portal_commerce_reconciliation_summary(&state, &workspace, &legacy_orders).await?;

    Ok(Json(PortalCommerceOrderCenterResponse {
        project_id: workspace.project.id,
        payment_simulation_enabled: state.payment_simulation_enabled,
        membership,
        reconciliation,
        orders,
    }))
}

async fn create_commerce_order_refund_handler(
    claims: AuthenticatedPortalClaims,
    Path(order_id): Path<String>,
    State(state): State<PortalApiState>,
    Json(request): Json<CreateCommerceOrderRefundRequest>,
) -> Result<(StatusCode, Json<RefundOrderRecord>), (StatusCode, Json<ErrorResponse>)> {
    let payment_store = payment_store_kernel(&state)?;
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

    request_portal_order_refund(
        payment_store.as_ref(),
        &claims.claims().sub,
        &workspace.project.id,
        &order_id,
        &request.refund_reason_code,
        request.requested_amount_minor,
        current_time_millis(),
    )
    .await
    .map(|refund_order| (StatusCode::CREATED, Json(refund_order)))
    .map_err(portal_payment_error_response)
}

async fn list_commerce_payment_events_handler(
    claims: AuthenticatedPortalClaims,
    Path(order_id): Path<String>,
    State(state): State<PortalApiState>,
) -> Result<Json<Vec<PaymentCallbackEventRecord>>, (StatusCode, Json<ErrorResponse>)> {
    let payment_store = payment_store_kernel(&state)?;
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

    load_portal_order_payment_events(
        payment_store.as_ref(),
        &claims.claims().sub,
        &workspace.project.id,
        &order_id,
    )
    .await
    .map(Json)
    .map_err(portal_payment_error_response)
}

async fn account_history_handler(
    claims: AuthenticatedPortalClaims,
    State(state): State<PortalApiState>,
) -> Result<Json<PortalAccountHistoryResponse>, (StatusCode, Json<ErrorResponse>)> {
    let payment_store = payment_store_kernel(&state)?;
    let identity_store = identity_store_kernel(&state)?;
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

    let mut snapshot = load_portal_account_history_for_store(
        payment_store.as_ref(),
        identity_store.as_ref(),
        &claims.claims().sub,
        &workspace.project.id,
        current_time_millis(),
    )
    .await
    .map_err(portal_payment_error_response)?;

    if snapshot.account.is_none() {
        if let Some(commercial_billing) = state.commercial_billing.as_ref() {
            if let Some(account) = commercial_billing
                .resolve_payable_account_for_gateway_request_context(
                    &portal_workspace_request_context(&workspace),
                )
                .await
                .map_err(commercial_billing_error_response)?
            {
                let account_id = account.account_id;
                let balance = commercial_billing
                    .summarize_account_balance(account_id, current_time_millis())
                    .await
                    .map_err(commercial_billing_error_response)?;
                let ledger_entry_ids = payment_store
                    .list_account_ledger_entry_records()
                    .await
                    .map_err(portal_payment_error_response)?
                    .into_iter()
                    .filter(|entry| entry.account_id == account_id)
                    .map(|entry| entry.ledger_entry_id)
                    .collect::<std::collections::BTreeSet<_>>();

                snapshot.account = Some(account);
                snapshot.balance = Some(balance);
                snapshot.lots = payment_store
                    .list_account_benefit_lots()
                    .await
                    .map_err(portal_payment_error_response)?
                    .into_iter()
                    .filter(|lot| lot.account_id == account_id)
                    .collect();
                snapshot.ledger_entries = payment_store
                    .list_account_ledger_entry_records()
                    .await
                    .map_err(portal_payment_error_response)?
                    .into_iter()
                    .filter(|entry| entry.account_id == account_id)
                    .collect();
                snapshot.ledger_allocations = payment_store
                    .list_account_ledger_allocations()
                    .await
                    .map_err(portal_payment_error_response)?
                    .into_iter()
                    .filter(|allocation| ledger_entry_ids.contains(&allocation.ledger_entry_id))
                    .collect();
            }
        }
    }

    let (benefit_lots, holds, request_settlements, ledger) =
        if let Some(account) = snapshot.account.as_ref() {
            let mut benefit_lots = payment_store
                .list_account_benefit_lots()
                .await
                .map_err(portal_payment_error_response)?
                .into_iter()
                .filter(|lot| lot.account_id == account.account_id)
                .collect::<Vec<_>>();
            benefit_lots.sort_by_key(|lot| lot.lot_id);

            let mut holds = payment_store
                .list_account_holds()
                .await
                .map_err(portal_payment_error_response)?
                .into_iter()
                .filter(|hold| hold.account_id == account.account_id)
                .collect::<Vec<_>>();
            holds.sort_by_key(|hold| hold.hold_id);

            let mut request_settlements = payment_store
                .list_request_settlement_records()
                .await
                .map_err(portal_payment_error_response)?
                .into_iter()
                .filter(|settlement| settlement.account_id == account.account_id)
                .collect::<Vec<_>>();
            request_settlements.sort_by_key(|settlement| settlement.request_settlement_id);

            let ledger = list_account_ledger_history(payment_store.as_ref(), account.account_id)
                .await
                .map_err(portal_payment_error_response)?;

            (benefit_lots, holds, request_settlements, ledger)
        } else {
            (Vec::new(), Vec::new(), Vec::new(), Vec::new())
        };

    Ok(Json(PortalAccountHistoryResponse::from_parts(
        snapshot,
        benefit_lots,
        holds,
        request_settlements,
        ledger,
    )))
}

impl From<PaymentAttemptTimelineEntry> for PortalPaymentAttemptTimelineEntryResponse {
    fn from(value: PaymentAttemptTimelineEntry) -> Self {
        Self {
            attempt: value.attempt,
            sessions: value.sessions,
        }
    }
}

impl From<&AccountLotBalanceSnapshot> for PortalAccountLotBalanceSnapshotResponse {
    fn from(value: &AccountLotBalanceSnapshot) -> Self {
        Self {
            lot_id: value.lot_id,
            benefit_type: account_benefit_type_label(value.benefit_type),
            scope_json: value.scope_json.clone(),
            expires_at_ms: value.expires_at_ms,
            original_quantity: value.original_quantity,
            remaining_quantity: value.remaining_quantity,
            held_quantity: value.held_quantity,
            available_quantity: value.available_quantity,
        }
    }
}

fn account_benefit_type_label(
    benefit_type: sdkwork_api_domain_billing::AccountBenefitType,
) -> String {
    match benefit_type {
        sdkwork_api_domain_billing::AccountBenefitType::CashCredit => "cash_credit",
        sdkwork_api_domain_billing::AccountBenefitType::PromoCredit => "promo_credit",
        sdkwork_api_domain_billing::AccountBenefitType::RequestAllowance => "request_allowance",
        sdkwork_api_domain_billing::AccountBenefitType::TokenAllowance => "token_allowance",
        sdkwork_api_domain_billing::AccountBenefitType::ImageAllowance => "image_allowance",
        sdkwork_api_domain_billing::AccountBenefitType::AudioAllowance => "audio_allowance",
        sdkwork_api_domain_billing::AccountBenefitType::VideoAllowance => "video_allowance",
        sdkwork_api_domain_billing::AccountBenefitType::MusicAllowance => "music_allowance",
    }
    .to_owned()
}

impl From<&AccountBalanceSnapshot> for PortalAccountBalanceSnapshotResponse {
    fn from(value: &AccountBalanceSnapshot) -> Self {
        Self {
            account_id: value.account_id,
            available_balance: value.available_balance,
            held_balance: value.held_balance,
            consumed_balance: value.consumed_balance,
            grant_balance: value.grant_balance,
            active_lot_count: value.active_lot_count,
            lots: value
                .lots
                .iter()
                .map(PortalAccountLotBalanceSnapshotResponse::from)
                .collect(),
        }
    }
}

impl From<PortalAccountHistorySnapshot> for PortalAccountHistoryResponse {
    fn from(value: PortalAccountHistorySnapshot) -> Self {
        Self::from_parts(value, Vec::new(), Vec::new(), Vec::new(), Vec::new())
    }
}

impl PortalAccountHistoryResponse {
    fn from_parts(
        value: PortalAccountHistorySnapshot,
        benefit_lots: Vec<AccountBenefitLotRecord>,
        holds: Vec<AccountHoldRecord>,
        request_settlements: Vec<RequestSettlementRecord>,
        ledger: Vec<AccountLedgerHistoryEntry>,
    ) -> Self {
        Self {
            account: value.account,
            balance: value
                .balance
                .as_ref()
                .map(PortalAccountBalanceSnapshotResponse::from),
            benefit_lots,
            holds,
            request_settlements,
            ledger,
            lots: value.lots,
            ledger_entries: value.ledger_entries,
            ledger_allocations: value.ledger_allocations,
            refunds: value.refunds,
        }
    }
}

async fn load_project_order_center(
    store: &dyn CommercialKernelStore,
    project_id: &str,
) -> anyhow::Result<Vec<CommerceOrderCenterEntry>> {
    list_project_commerce_order_center(store, project_id).await
}

async fn recover_project_order_checkout_artifacts(
    payment_store: &dyn CommercialKernelStore,
    identity_store: &dyn IdentityKernelStore,
    project_id: &str,
    observed_at_ms: u64,
) {
    let Ok(orders) = list_project_commerce_orders(payment_store, project_id).await else {
        return;
    };

    for order in orders {
        if order.payable_price_cents == 0 {
            continue;
        }
        let _ = sync_portal_order_checkout(
            payment_store,
            identity_store,
            &order.user_id,
            &order,
            observed_at_ms,
        )
        .await;
    }
}

async fn request_portal_order_refund(
    store: &dyn CommercialKernelStore,
    portal_user_id: &str,
    project_id: &str,
    order_id: &str,
    refund_reason_code: &str,
    requested_amount_minor: u64,
    requested_at_ms: u64,
) -> anyhow::Result<RefundOrderRecord> {
    request_portal_commerce_order_refund(
        store,
        portal_user_id,
        project_id,
        order_id,
        refund_reason_code,
        requested_amount_minor,
        requested_at_ms,
    )
    .await
}

async fn load_portal_order_payment_events(
    store: &dyn CommercialKernelStore,
    portal_user_id: &str,
    project_id: &str,
    order_id: &str,
) -> anyhow::Result<Vec<PaymentCallbackEventRecord>> {
    list_portal_commerce_order_payment_events(store, portal_user_id, project_id, order_id).await
}

async fn load_portal_account_history_for_store(
    payment_store: &dyn CommercialKernelStore,
    identity_store: &dyn IdentityKernelStore,
    portal_user_id: &str,
    project_id: &str,
    now_ms: u64,
) -> anyhow::Result<PortalAccountHistorySnapshot> {
    let scope = ensure_portal_payment_subject_scope(identity_store, portal_user_id, now_ms).await?;

    let mut payment_orders = payment_store
        .list_payment_order_records()
        .await?
        .into_iter()
        .filter(|payment_order| {
            payment_order.project_id == project_id
                && payment_order.tenant_id == scope.tenant_id
                && payment_order.organization_id == scope.organization_id
                && payment_order.user_id == scope.user_id
        })
        .collect::<Vec<_>>();
    payment_orders.sort_by(|left, right| {
        right
            .created_at_ms
            .cmp(&left.created_at_ms)
            .then_with(|| right.payment_order_id.cmp(&left.payment_order_id))
    });

    let mut refunds = Vec::new();
    for payment_order in &payment_orders {
        let mut payment_refunds = payment_store
            .list_refund_order_records_for_payment_order(&payment_order.payment_order_id)
            .await?;
        refunds.append(&mut payment_refunds);
    }
    refunds.sort_by(|left, right| {
        right
            .created_at_ms
            .cmp(&left.created_at_ms)
            .then_with(|| right.refund_order_id.cmp(&left.refund_order_id))
    });

    let Some(account) = payment_store
        .find_account_record_by_owner(
            scope.tenant_id,
            scope.organization_id,
            scope.user_id,
            AccountType::Primary,
        )
        .await?
    else {
        return Ok(PortalAccountHistorySnapshot {
            account: None,
            balance: None,
            lots: Vec::new(),
            ledger_entries: Vec::new(),
            ledger_allocations: Vec::new(),
            refunds,
        });
    };

    let balance = summarize_account_balance(payment_store, account.account_id, now_ms).await?;

    let mut lots = payment_store
        .list_account_benefit_lots()
        .await?
        .into_iter()
        .filter(|lot| lot.account_id == account.account_id)
        .collect::<Vec<_>>();
    lots.sort_by(|left, right| {
        right
            .created_at_ms
            .cmp(&left.created_at_ms)
            .then_with(|| right.lot_id.cmp(&left.lot_id))
    });

    let mut ledger_entries = payment_store
        .list_account_ledger_entry_records()
        .await?
        .into_iter()
        .filter(|entry| entry.account_id == account.account_id)
        .collect::<Vec<_>>();
    ledger_entries.sort_by(|left, right| {
        right
            .created_at_ms
            .cmp(&left.created_at_ms)
            .then_with(|| right.ledger_entry_id.cmp(&left.ledger_entry_id))
    });
    let ledger_entry_ids = ledger_entries
        .iter()
        .map(|entry| entry.ledger_entry_id)
        .collect::<std::collections::BTreeSet<_>>();
    let mut ledger_allocations = payment_store
        .list_account_ledger_allocations()
        .await?
        .into_iter()
        .filter(|allocation| ledger_entry_ids.contains(&allocation.ledger_entry_id))
        .collect::<Vec<_>>();
    ledger_allocations.sort_by(|left, right| {
        right
            .created_at_ms
            .cmp(&left.created_at_ms)
            .then_with(|| right.ledger_allocation_id.cmp(&left.ledger_allocation_id))
    });

    Ok(PortalAccountHistorySnapshot {
        account: Some(account),
        balance: Some(balance),
        lots,
        ledger_entries,
        ledger_allocations,
        refunds,
    })
}

async fn sync_portal_order_checkout(
    payment_store: &dyn CommercialKernelStore,
    identity_store: &dyn IdentityKernelStore,
    portal_user_id: &str,
    order: &PortalCommerceOrderRecord,
    observed_at_ms: u64,
) -> anyhow::Result<()> {
    let scope =
        ensure_portal_payment_subject_scope(identity_store, portal_user_id, observed_at_ms).await?;
    ensure_commerce_payment_checkout(payment_store, &scope, order, "portal_web").await?;
    Ok(())
}

fn portal_payment_error_response(error: anyhow::Error) -> (StatusCode, Json<ErrorResponse>) {
    let message = error.to_string();
    let lowered = message.to_ascii_lowercase();
    let status = if lowered.contains("not found") {
        StatusCode::NOT_FOUND
    } else if lowered.contains("conflict") {
        StatusCode::CONFLICT
    } else if lowered.contains("required")
        || lowered.contains("must")
        || lowered.contains("invalid")
        || lowered.contains("exceeds")
        || lowered.contains("not supported")
        || lowered.contains("outside the current")
        || lowered.contains("does not support")
    {
        StatusCode::BAD_REQUEST
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };

    (
        status,
        Json(ErrorResponse {
            error: ErrorBody { message },
        }),
    )
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

fn marketing_atomic_status(error: anyhow::Error) -> StatusCode {
    let message = error.to_string();
    if message.contains("changed concurrently")
        || message.contains("already exists with different state")
        || message.contains(" is missing")
    {
        StatusCode::CONFLICT
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

fn commercial_billing_kernel(
    state: &PortalApiState,
) -> Result<&Arc<dyn CommercialBillingAdminKernel>, (StatusCode, Json<ErrorResponse>)> {
    state.commercial_billing.as_ref().ok_or_else(|| {
        error_response(
            StatusCode::NOT_IMPLEMENTED,
            "commercial billing portal views are unavailable for the current storage runtime",
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

fn portal_workspace_request_context(workspace: &PortalWorkspaceSummary) -> GatewayRequestContext {
    GatewayRequestContext {
        tenant_id: workspace.tenant.id.clone(),
        project_id: workspace.project.id.clone(),
        environment: "portal".to_owned(),
        api_key_hash: "portal_workspace_scope".to_owned(),
        api_key_group_id: None,
        canonical_tenant_id: None,
        canonical_organization_id: None,
        canonical_user_id: None,
        canonical_api_key_id: None,
    }
}

async fn load_portal_billing_account_context(
    state: &PortalApiState,
    claims: &AuthenticatedPortalClaims,
) -> Result<(AccountRecord, AccountBalanceSnapshot), (StatusCode, Json<ErrorResponse>)> {
    let workspace = load_workspace_for_user(state.store.as_ref(), &claims.claims().sub)
        .await
        .map_err(|status| error_response(status, "portal workspace is unavailable"))?;
    let commercial_billing = commercial_billing_kernel(state)?.clone();
    let account = commercial_billing
        .resolve_payable_account_for_gateway_request_context(&portal_workspace_request_context(
            &workspace,
        ))
        .await
        .map_err(commercial_billing_error_response)?
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                "workspace commercial account is not provisioned",
            )
        })?;
    let balance = commercial_billing
        .summarize_account_balance(account.account_id, current_time_millis())
        .await
        .map_err(commercial_billing_error_response)?;
    Ok((account, balance))
}

async fn load_portal_commerce_reconciliation_summary(
    state: &PortalApiState,
    workspace: &PortalWorkspaceSummary,
    order_center_entries: &[PortalOrderCenterEntry],
) -> Result<Option<PortalCommerceReconciliationSummary>, (StatusCode, Json<ErrorResponse>)> {
    let Some(commercial_billing) = state.commercial_billing.as_ref() else {
        return Ok(None);
    };
    let account = commercial_billing
        .resolve_payable_account_for_gateway_request_context(&portal_workspace_request_context(
            workspace,
        ))
        .await
        .map_err(commercial_billing_error_response)?;
    let Some(account) = account else {
        return Ok(None);
    };
    let checkpoint = commercial_billing
        .find_account_commerce_reconciliation_state(account.account_id, &workspace.project.id)
        .await
        .map_err(commercial_billing_error_response)?;
    let last_reconciled_order_id = checkpoint
        .as_ref()
        .map(|record| record.last_order_id.clone())
        .unwrap_or_default();
    let last_reconciled_order_updated_at_ms = checkpoint
        .as_ref()
        .map(|record| record.last_order_updated_at_ms)
        .unwrap_or_default();
    let last_reconciled_order_created_at_ms = checkpoint
        .as_ref()
        .map(|record| record.last_order_created_at_ms)
        .unwrap_or_default();
    let last_reconciled_at_ms = checkpoint
        .as_ref()
        .map(|record| record.updated_at_ms)
        .unwrap_or_default();
    let latest_order_updated_at_ms = order_center_entries
        .iter()
        .map(|entry| entry.order.order.updated_at_ms)
        .max()
        .unwrap_or_default();
    let backlog_order_count = order_center_entries
        .iter()
        .filter(|entry| entry.order.order.updated_at_ms > last_reconciled_order_updated_at_ms)
        .count();

    Ok(Some(PortalCommerceReconciliationSummary {
        account_id: account.account_id,
        last_reconciled_order_id,
        last_reconciled_order_updated_at_ms,
        last_reconciled_order_created_at_ms,
        last_reconciled_at_ms,
        backlog_order_count,
        checkpoint_lag_ms: latest_order_updated_at_ms
            .saturating_sub(last_reconciled_order_updated_at_ms),
        healthy: backlog_order_count == 0,
    }))
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

async fn load_project_billing_summary(
    store: &dyn AdminStore,
    project_id: &str,
) -> Result<ProjectBillingSummary, StatusCode> {
    let ledger = store
        .list_ledger_entries_for_project(project_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let policies = store
        .list_quota_policies_for_project(project_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let billing = summarize_billing_snapshot(&ledger, &policies);

    Ok(billing
        .projects
        .into_iter()
        .next()
        .unwrap_or_else(|| ProjectBillingSummary::new(project_id.to_owned())))
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
