use super::*;

const GATEWAY_DEFAULT_BODY_LIMIT_BYTES: usize = 64 * 1024 * 1024;

pub(crate) fn gateway_service_name() -> Arc<str> {
    Arc::from("gateway")
}

pub(crate) fn gateway_http_metrics() -> Arc<HttpMetricsRegistry> {
    Arc::new(HttpMetricsRegistry::new("gateway"))
}

pub(crate) fn gateway_base_router<S>(
    metrics: Arc<HttpMetricsRegistry>,
    http_exposure: Option<&sdkwork_api_config::HttpExposureConfig>,
) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let metrics_route = match http_exposure {
        Some(http_exposure) => super::gateway_http::metrics_route(metrics.clone(), http_exposure),
        None => get({
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
    };

    Router::<S>::new()
        .route(
            "/openapi.json",
            get(super::gateway_docs::gateway_openapi_handler),
        )
        .route("/docs", get(super::gateway_docs::gateway_docs_handler))
        .route("/metrics", metrics_route)
        .route("/health", get(|| async { "ok" }))
}

pub(crate) fn finalize_stateful_gateway_router(
    router: Router<GatewayApiState>,
    state: GatewayApiState,
    service_name: Arc<str>,
    metrics: Arc<HttpMetricsRegistry>,
    http_exposure: Option<&sdkwork_api_config::HttpExposureConfig>,
) -> Router {
    let router = router
        .layer(axum::extract::DefaultBodyLimit::max(
            GATEWAY_DEFAULT_BODY_LIMIT_BYTES,
        ))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            apply_gateway_request_context,
        ))
        .layer(axum::middleware::from_fn(apply_request_routing_region))
        .layer(axum::middleware::from_fn_with_state(
            metrics,
            observe_http_metrics,
        ))
        .layer(axum::middleware::from_fn_with_state(
            service_name,
            observe_http_tracing,
        ));
    let router = match http_exposure {
        Some(http_exposure) => router.layer(super::gateway_http::browser_cors_layer(http_exposure)),
        None => router.layer(super::gateway_docs::browser_cors_layer()),
    };
    router.with_state(state)
}

pub(crate) fn finalize_stateless_gateway_router(
    router: Router<StatelessGatewayContext>,
    config: StatelessGatewayConfig,
    service_name: Arc<str>,
    metrics: Arc<HttpMetricsRegistry>,
    http_exposure: Option<&sdkwork_api_config::HttpExposureConfig>,
) -> Router {
    let router = router
        .layer(axum::extract::DefaultBodyLimit::max(
            GATEWAY_DEFAULT_BODY_LIMIT_BYTES,
        ))
        .layer(axum::middleware::from_fn(apply_request_routing_region))
        .layer(axum::middleware::from_fn_with_state(
            metrics,
            observe_http_metrics,
        ))
        .layer(axum::middleware::from_fn_with_state(
            service_name,
            observe_http_tracing,
        ));
    let router = match http_exposure {
        Some(http_exposure) => router.layer(super::gateway_http::browser_cors_layer(http_exposure)),
        None => router.layer(super::gateway_docs::browser_cors_layer()),
    };
    router.with_state(config.into_context())
}
