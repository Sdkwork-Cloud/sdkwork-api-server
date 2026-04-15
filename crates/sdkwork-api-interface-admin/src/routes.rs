use super::*;

mod billing_routes;
mod catalog_routes;
mod commerce_routes;
mod marketing_routes;

const ADMIN_DEFAULT_BODY_LIMIT_BYTES: usize = 2 * 1024 * 1024;

fn http_exposure_config() -> anyhow::Result<HttpExposureConfig> {
    HttpExposureConfig::from_env()
}

fn metrics_route<S>(
    metrics: Arc<HttpMetricsRegistry>,
    http_exposure: &HttpExposureConfig,
) -> axum::routing::MethodRouter<S>
where
    S: Clone + Send + Sync + 'static,
{
    let expected_token: Arc<str> = Arc::from(http_exposure.metrics_bearer_token.clone());
    get(move |headers: HeaderMap| {
        let metrics = metrics.clone();
        let expected_token = expected_token.clone();
        async move {
            if !metrics_request_authorized(&headers, expected_token.as_ref()) {
                return (
                    StatusCode::UNAUTHORIZED,
                    [(header::WWW_AUTHENTICATE, "Bearer")],
                    "metrics bearer token required",
                )
                    .into_response();
            }

            (
                [(
                    header::CONTENT_TYPE,
                    "text/plain; version=0.0.4; charset=utf-8",
                )],
                metrics.render_prometheus(),
            )
                .into_response()
        }
    })
}

fn metrics_route_with_state(
    metrics: Arc<HttpMetricsRegistry>,
    http_exposure: &HttpExposureConfig,
) -> axum::routing::MethodRouter<AdminApiState> {
    let expected_token: Arc<str> = Arc::from(http_exposure.metrics_bearer_token.clone());
    get(
        move |headers: HeaderMap, State(state): State<AdminApiState>| {
            let metrics = metrics.clone();
            let expected_token = expected_token.clone();
            async move {
                if !metrics_request_authorized(&headers, expected_token.as_ref()) {
                    return (
                        StatusCode::UNAUTHORIZED,
                        [(header::WWW_AUTHENTICATE, "Bearer")],
                        "metrics bearer token required",
                    )
                        .into_response();
                }

                (
                    [(
                        header::CONTENT_TYPE,
                        "text/plain; version=0.0.4; charset=utf-8",
                    )],
                    payments::render_admin_metrics_payload(metrics.as_ref(), &state).await,
                )
                    .into_response()
            }
        },
    )
}

fn metrics_request_authorized(headers: &HeaderMap, expected_token: &str) -> bool {
    if expected_token.is_empty() {
        return false;
    }

    let Some(value) = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
    else {
        return false;
    };
    let Some((scheme, token)) = value.trim().split_once(' ') else {
        return false;
    };
    scheme.eq_ignore_ascii_case("Bearer") && token.trim() == expected_token
}

pub fn try_admin_router() -> anyhow::Result<Router> {
    let service_name: Arc<str> = Arc::from("admin");
    let metrics = Arc::new(HttpMetricsRegistry::new("admin"));
    let http_exposure = http_exposure_config()?;
    Ok(Router::new()
        .merge(openapi::admin_docs_router())
        .route("/metrics", metrics_route(metrics.clone(), &http_exposure))
        .route("/admin/health", get(|| async { "ok" }))
        .route("/admin/auth/login", post(|| async { "login" }))
        .route("/admin/auth/me", get(|| async { "me" }))
        .route(
            "/admin/auth/change-password",
            post(|| async { "change-password" }),
        )
        .route("/admin/audit/events", get(|| async { "audit-events" }))
        .route("/admin/tenants", get(|| async { "tenants" }))
        .route(
            "/admin/tenants/{tenant_id}/providers/readiness",
            get(|| async { "tenant-provider-readiness" }),
        )
        .route("/admin/projects", get(|| async { "projects" }))
        .route("/admin/api-keys", get(|| async { "api-keys" }))
        .route("/admin/api-key-groups", get(|| async { "api-key-groups" }))
        .route(
            "/admin/api-key-groups/{group_id}",
            patch(|| async { "api-key-groups" }).delete(|| async { "api-key-groups" }),
        )
        .route(
            "/admin/api-key-groups/{group_id}/status",
            post(|| async { "api-key-groups-status" }),
        )
        .route("/admin/channels", get(|| async { "channels" }))
        .route("/admin/providers", get(|| async { "providers" }))
        .route("/admin/credentials", get(|| async { "credentials" }))
        .route("/admin/channel-models", get(|| async { "channel-models" }))
        .route(
            "/admin/provider-accounts",
            get(|| async { "provider-accounts" }),
        )
        .route(
            "/admin/provider-models",
            get(|| async { "provider-models" }),
        )
        .route(
            "/admin/provider-accounts/{provider_account_id}",
            delete(|| async { "provider-accounts-delete" }),
        )
        .route(
            "/admin/provider-models/{proxy_provider_id}/channels/{channel_id}/models/{model_id}",
            delete(|| async { "provider-models-delete" }),
        )
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
        .route("/admin/billing/events", get(|| async { "billing-events" }))
        .route(
            "/admin/billing/events/summary",
            get(|| async { "billing-events-summary" }),
        )
        .route("/admin/billing/ledger", get(|| async { "billing-ledger" }))
        .route(
            "/admin/billing/summary",
            get(|| async { "billing-summary" }),
        )
        .route(
            "/admin/billing/accounts",
            get(|| async { "billing-accounts" }),
        )
        .route(
            "/admin/billing/accounts/{account_id}/balance",
            get(|| async { "billing-account-balance" }),
        )
        .route(
            "/admin/billing/accounts/{account_id}/benefit-lots",
            get(|| async { "billing-account-benefit-lots" }),
        )
        .route(
            "/admin/billing/accounts/{account_id}/ledger",
            get(|| async { "billing-account-ledger" }),
        )
        .route(
            "/admin/billing/account-holds",
            get(|| async { "billing-account-holds" }),
        )
        .route(
            "/admin/billing/request-settlements",
            get(|| async { "billing-request-settlements" }),
        )
        .route(
            "/admin/commerce/orders",
            get(|| async { "commerce-orders" }),
        )
        .route(
            "/admin/commerce/catalog-publications",
            get(|| async { "commerce-catalog-publications" }),
        )
        .route(
            "/admin/commerce/catalog-publications/{publication_id}",
            get(|| async { "commerce-catalog-publication-detail" }),
        )
        .route(
            "/admin/commerce/catalog-publications/{publication_id}/publish",
            post(|| async { "commerce-catalog-publication-publish" }),
        )
        .route(
            "/admin/commerce/catalog-publications/{publication_id}/schedule",
            post(|| async { "commerce-catalog-publication-schedule" }),
        )
        .route(
            "/admin/commerce/catalog-publications/{publication_id}/retire",
            post(|| async { "commerce-catalog-publication-retire" }),
        )
        .route(
            "/admin/commerce/payment-methods",
            get(|| async { "commerce-payment-methods" }),
        )
        .route(
            "/admin/commerce/payment-methods/{payment_method_id}",
            put(|| async { "commerce-payment-method-put" })
                .delete(|| async { "commerce-payment-method-delete" }),
        )
        .route(
            "/admin/commerce/payment-methods/{payment_method_id}/credential-bindings",
            get(|| async { "commerce-payment-method-bindings" })
                .put(|| async { "commerce-payment-method-bindings-replace" }),
        )
        .route(
            "/admin/commerce/orders/{order_id}/payment-events",
            get(|| async { "commerce-order-payment-events" }),
        )
        .route(
            "/admin/commerce/orders/{order_id}/payment-attempts",
            get(|| async { "commerce-order-payment-attempts" }),
        )
        .route(
            "/admin/commerce/orders/{order_id}/refunds",
            get(|| async { "commerce-order-refunds" })
                .post(|| async { "commerce-order-refunds-create" }),
        )
        .route(
            "/admin/commerce/orders/{order_id}/audit",
            get(|| async { "commerce-order-audit" }),
        )
        .route(
            "/admin/commerce/webhook-inbox",
            get(|| async { "commerce-webhook-inbox" }),
        )
        .route(
            "/admin/commerce/webhook-inbox/{webhook_inbox_id}/delivery-attempts",
            get(|| async { "commerce-webhook-delivery-attempts" }),
        )
        .route(
            "/admin/commerce/reconciliation-runs",
            get(|| async { "commerce-reconciliation-runs" })
                .post(|| async { "commerce-reconciliation-runs-create" }),
        )
        .route(
            "/admin/commerce/reconciliation-runs/{reconciliation_run_id}/items",
            get(|| async { "commerce-reconciliation-items" }),
        )
        .route("/admin/async-jobs", get(|| async { "async-jobs" }))
        .route(
            "/admin/async-jobs/{job_id}/attempts",
            get(|| async { "async-job-attempts" }),
        )
        .route(
            "/admin/async-jobs/{job_id}/assets",
            get(|| async { "async-job-assets" }),
        )
        .route(
            "/admin/async-jobs/{job_id}/callbacks",
            get(|| async { "async-job-callbacks" }),
        )
        .route(
            "/admin/billing/pricing-lifecycle/synchronize",
            post(|| async { "billing-pricing-lifecycle-synchronize" }),
        )
        .route(
            "/admin/billing/pricing-plans",
            get(|| async { "billing-pricing-plans" })
                .post(|| async { "billing-pricing-plans-create" }),
        )
        .route(
            "/admin/billing/pricing-plans/{pricing_plan_id}",
            put(|| async { "billing-pricing-plans-update" }),
        )
        .route(
            "/admin/billing/pricing-plans/{pricing_plan_id}/clone",
            post(|| async { "billing-pricing-plans-clone" }),
        )
        .route(
            "/admin/billing/pricing-plans/{pricing_plan_id}/schedule",
            post(|| async { "billing-pricing-plans-schedule" }),
        )
        .route(
            "/admin/billing/pricing-plans/{pricing_plan_id}/publish",
            post(|| async { "billing-pricing-plans-publish" }),
        )
        .route(
            "/admin/billing/pricing-plans/{pricing_plan_id}/retire",
            post(|| async { "billing-pricing-plans-retire" }),
        )
        .route(
            "/admin/billing/pricing-rates",
            get(|| async { "billing-pricing-rates" })
                .post(|| async { "billing-pricing-rates-create" }),
        )
        .route(
            "/admin/billing/pricing-rates/{pricing_rate_id}",
            put(|| async { "billing-pricing-rates-update" }),
        )
        .route(
            "/admin/billing/quota-policies",
            get(|| async { "billing-quota-policies" }),
        )
        .route(
            "/admin/gateway/rate-limit-policies",
            get(|| async { "gateway-rate-limit-policies" }),
        )
        .route("/admin/routing/policies", get(|| async { "policies" }))
        .route("/admin/routing/profiles", get(|| async { "profiles" }))
        .route("/admin/routing/snapshots", get(|| async { "snapshots" }))
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
        .layer(axum::extract::DefaultBodyLimit::max(
            ADMIN_DEFAULT_BODY_LIMIT_BYTES,
        ))
        .layer(axum::middleware::from_fn_with_state(
            service_name,
            observe_http_tracing,
        )))
}

pub fn admin_router() -> Router {
    try_admin_router().expect("http exposure config should load from process env")
}

pub fn admin_router_with_pool(pool: SqlitePool) -> Router {
    admin_router_with_pool_and_master_key(pool, "local-dev-master-key")
}

pub fn try_admin_router_with_pool(pool: SqlitePool) -> anyhow::Result<Router> {
    try_admin_router_with_pool_and_master_key(pool, "local-dev-master-key")
}

pub fn admin_router_with_store(store: Arc<dyn AdminStore>) -> Router {
    admin_router_with_store_and_secret_manager(
        store,
        CredentialSecretManager::database_encrypted("local-dev-master-key"),
    )
}

pub fn try_admin_router_with_store(store: Arc<dyn AdminStore>) -> anyhow::Result<Router> {
    try_admin_router_with_store_and_secret_manager(
        store,
        CredentialSecretManager::database_encrypted("local-dev-master-key"),
    )
}

pub fn admin_router_with_pool_and_master_key(
    pool: SqlitePool,
    credential_master_key: impl Into<String>,
) -> Router {
    admin_router_with_state(AdminApiState::with_master_key(pool, credential_master_key))
}

pub fn try_admin_router_with_pool_and_master_key(
    pool: SqlitePool,
    credential_master_key: impl Into<String>,
) -> anyhow::Result<Router> {
    try_admin_router_with_state(AdminApiState::with_master_key(pool, credential_master_key))
}

pub fn admin_router_with_pool_and_secret_manager(
    pool: SqlitePool,
    secret_manager: CredentialSecretManager,
) -> Router {
    admin_router_with_state(AdminApiState::with_secret_manager(pool, secret_manager))
}

pub fn try_admin_router_with_pool_and_secret_manager(
    pool: SqlitePool,
    secret_manager: CredentialSecretManager,
) -> anyhow::Result<Router> {
    try_admin_router_with_state(AdminApiState::with_secret_manager(pool, secret_manager))
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

pub fn try_admin_router_with_store_and_secret_manager(
    store: Arc<dyn AdminStore>,
    secret_manager: CredentialSecretManager,
) -> anyhow::Result<Router> {
    try_admin_router_with_store_and_secret_manager_and_jwt_secret(
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

pub fn try_admin_router_with_store_and_secret_manager_and_jwt_secret(
    store: Arc<dyn AdminStore>,
    secret_manager: CredentialSecretManager,
    jwt_signing_secret: impl Into<String>,
) -> anyhow::Result<Router> {
    try_admin_router_with_state(AdminApiState::with_store_and_secret_manager_and_jwt_secret(
        store,
        secret_manager,
        jwt_signing_secret,
    ))
}

pub fn try_admin_router_with_state(state: AdminApiState) -> anyhow::Result<Router> {
    Ok(admin_router_with_state_and_http_exposure(
        state,
        http_exposure_config()?,
    ))
}

pub fn admin_router_with_state(state: AdminApiState) -> Router {
    try_admin_router_with_state(state).expect("http exposure config should load from process env")
}

pub fn admin_router_with_state_and_http_exposure(
    state: AdminApiState,
    http_exposure: HttpExposureConfig,
) -> Router {
    let service_name: Arc<str> = Arc::from("admin");
    let metrics = Arc::new(HttpMetricsRegistry::new("admin"));
    billing_routes::register_billing_routes(catalog_routes::register_catalog_routes(
        commerce_routes::register_commerce_routes(marketing_routes::register_marketing_routes(
            Router::<AdminApiState>::new(),
        )),
    ))
    .merge(openapi::admin_docs_router())
    .route(
        "/metrics",
        metrics_route_with_state(metrics.clone(), &http_exposure),
    )
    .route("/admin/health", get(|| async { "ok" }))
    .route("/admin/auth/login", post(auth::login_handler))
    .route("/admin/auth/me", get(auth::me_handler))
    .route(
        "/admin/auth/change-password",
        post(auth::change_password_handler),
    )
    .route("/admin/audit/events", get(audit::list_admin_audit_events_handler))
    .route(
        "/admin/users/operators",
        get(users::list_operator_users_handler).post(users::upsert_operator_user_handler),
    )
    .route(
        "/admin/users/operators/{user_id}",
        delete(users::delete_operator_user_handler),
    )
    .route(
        "/admin/users/operators/{user_id}/status",
        post(users::update_operator_user_status_handler),
    )
    .route(
        "/admin/users/operators/{user_id}/password",
        post(users::reset_operator_user_password_handler),
    )
    .route(
        "/admin/users/portal",
        get(users::list_portal_users_handler).post(users::upsert_portal_user_handler),
    )
    .route(
        "/admin/users/portal/{user_id}",
        delete(users::delete_portal_user_handler),
    )
    .route(
        "/admin/users/portal/{user_id}/status",
        post(users::update_portal_user_status_handler),
    )
    .route(
        "/admin/users/portal/{user_id}/password",
        post(users::reset_portal_user_password_handler),
    )
    .route(
        "/admin/tenants",
        get(tenant::list_tenants_handler).post(tenant::create_tenant_handler),
    )
    .route(
        "/admin/tenants/{tenant_id}",
        delete(tenant::delete_tenant_handler),
    )
    .route(
        "/admin/projects",
        get(tenant::list_projects_handler).post(tenant::create_project_handler),
    )
    .route(
        "/admin/projects/{project_id}",
        delete(tenant::delete_project_handler),
    )
    .route(
        "/admin/api-key-groups",
        get(gateway::list_api_key_groups_handler).post(gateway::create_api_key_group_handler),
    )
    .route(
        "/admin/api-key-groups/{group_id}/status",
        post(gateway::update_api_key_group_status_handler),
    )
    .route(
        "/admin/api-key-groups/{group_id}",
        patch(gateway::update_api_key_group_handler).delete(gateway::delete_api_key_group_handler),
    )
    .route(
        "/admin/api-keys",
        get(gateway::list_api_keys_handler).post(gateway::create_api_key_handler),
    )
    .route(
        "/admin/api-keys/{hashed_key}/status",
        post(gateway::update_api_key_status_handler),
    )
    .route(
        "/admin/api-keys/{hashed_key}",
        put(gateway::update_api_key_handler).delete(gateway::delete_api_key_handler),
    )
    .route(
        "/admin/extensions/installations",
        get(runtime::list_extension_installations_handler)
            .post(runtime::create_extension_installation_handler),
    )
    .route(
        "/admin/extensions/packages",
        get(runtime::list_extension_packages_handler),
    )
    .route(
        "/admin/extensions/instances",
        get(runtime::list_extension_instances_handler)
            .post(runtime::create_extension_instance_handler),
    )
    .route(
        "/admin/extensions/runtime-statuses",
        get(runtime::list_extension_runtime_statuses_handler),
    )
    .route(
        "/admin/extensions/runtime-reloads",
        post(runtime::reload_extension_runtimes_handler),
    )
    .route(
        "/admin/extensions/runtime-rollouts",
        get(runtime::list_extension_runtime_rollouts_handler)
            .post(runtime::create_extension_runtime_rollout_handler),
    )
    .route(
        "/admin/extensions/runtime-rollouts/{rollout_id}",
        get(runtime::get_extension_runtime_rollout_handler),
    )
    .route(
        "/admin/runtime-config/rollouts",
        get(runtime::list_standalone_config_rollouts_handler)
            .post(runtime::create_standalone_config_rollout_handler),
    )
    .route(
        "/admin/runtime-config/rollouts/{rollout_id}",
        get(runtime::get_standalone_config_rollout_handler),
    )
    .route("/admin/async-jobs", get(jobs::list_async_jobs_handler))
    .route(
        "/admin/async-jobs/{job_id}/attempts",
        get(jobs::list_async_job_attempts_handler),
    )
    .route(
        "/admin/async-jobs/{job_id}/assets",
        get(jobs::list_async_job_assets_handler),
    )
    .route(
        "/admin/async-jobs/{job_id}/callbacks",
        get(jobs::list_async_job_callbacks_handler),
    )
    .route(
        "/admin/gateway/rate-limit-policies",
        get(gateway::list_rate_limit_policies_handler)
            .post(gateway::create_rate_limit_policy_handler),
    )
    .route(
        "/admin/gateway/rate-limit-windows",
        get(gateway::list_rate_limit_window_snapshots_handler),
    )
    .route(
        "/admin/routing/policies",
        get(routing::list_routing_policies_handler).post(routing::create_routing_policy_handler),
    )
    .route(
        "/admin/routing/profiles",
        get(routing::list_routing_profiles_handler).post(routing::create_routing_profile_handler),
    )
    .route(
        "/admin/routing/snapshots",
        get(routing::list_compiled_routing_snapshots_handler),
    )
    .route(
        "/admin/routing/health-snapshots",
        get(routing::list_provider_health_snapshots_handler),
    )
    .route(
        "/admin/routing/decision-logs",
        get(routing::list_routing_decision_logs_handler),
    )
    .route(
        "/admin/routing/simulations",
        post(routing::simulate_routing_handler),
    )
    .layer(axum::middleware::from_fn_with_state(
        state.clone(),
        enforce_admin_route_access,
    ))
    .layer(axum::middleware::from_fn_with_state(
        metrics,
        observe_http_metrics,
    ))
    .layer(axum::extract::DefaultBodyLimit::max(
        ADMIN_DEFAULT_BODY_LIMIT_BYTES,
    ))
    .layer(axum::middleware::from_fn_with_state(
        service_name,
        observe_http_tracing,
    ))
    .with_state(state)
}
