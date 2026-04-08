fn http_exposure_config() -> anyhow::Result<HttpExposureConfig> {
    HttpExposureConfig::from_env()
}

fn browser_cors_layer(http_exposure: &HttpExposureConfig) -> CorsLayer {
    let layer = CorsLayer::new().allow_methods(Any).allow_headers(Any);
    if http_exposure.browser_allowed_origins.is_empty() {
        return layer;
    }

    let origins = http_exposure
        .browser_allowed_origins
        .iter()
        .filter_map(|origin| match HeaderValue::from_str(origin) {
            Ok(value) => Some(value),
            Err(error) => {
                eprintln!(
                    "ignoring invalid browser allowed origin while building gateway cors layer: {origin} ({error})"
                );
                None
            }
        })
        .collect::<Vec<_>>();
    if origins.is_empty() {
        return layer;
    }
    layer.allow_origin(origins)
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

