struct AuthenticatedGatewayRequest(IdentityGatewayRequestContext);

impl AuthenticatedGatewayRequest {
    fn tenant_id(&self) -> &str {
        self.0.tenant_id()
    }

    fn project_id(&self) -> &str {
        self.0.project_id()
    }

    fn context(&self) -> &IdentityGatewayRequestContext {
        &self.0
    }
}

impl FromRequestParts<GatewayApiState> for AuthenticatedGatewayRequest {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &GatewayApiState,
    ) -> Result<Self, Self::Rejection> {
        let context = if let Some(context) = parts
            .extensions
            .get::<IdentityGatewayRequestContext>()
            .cloned()
        {
            context
        } else {
            let Some(header_value) = parts.headers.get(header::AUTHORIZATION) else {
                return Err(StatusCode::UNAUTHORIZED.into_response());
            };
            let Ok(header_value) = header_value.to_str() else {
                return Err(StatusCode::UNAUTHORIZED.into_response());
            };
            let Some(token) = header_value
                .strip_prefix("Bearer ")
                .or_else(|| header_value.strip_prefix("bearer "))
            else {
                return Err(StatusCode::UNAUTHORIZED.into_response());
            };

            let Some(context) = resolve_gateway_request_context(state.store.as_ref(), token)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?
            else {
                return Err(StatusCode::UNAUTHORIZED.into_response());
            };
            context
        };

        Ok(Self(context))
    }
}

#[derive(Clone, Debug)]
struct CompatAuthenticatedGatewayRequest(IdentityGatewayRequestContext);

impl CompatAuthenticatedGatewayRequest {
    fn tenant_id(&self) -> &str {
        self.0.tenant_id()
    }

    fn project_id(&self) -> &str {
        self.0.project_id()
    }

    fn context(&self) -> &IdentityGatewayRequestContext {
        &self.0
    }
}

impl FromRequestParts<GatewayApiState> for CompatAuthenticatedGatewayRequest {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &GatewayApiState,
    ) -> Result<Self, Self::Rejection> {
        let context = if let Some(context) = parts
            .extensions
            .get::<IdentityGatewayRequestContext>()
            .cloned()
        {
            context
        } else {
            let Some(token) = extract_compat_gateway_token(parts) else {
                return Err(StatusCode::UNAUTHORIZED.into_response());
            };

            let Some(context) = resolve_gateway_request_context(state.store.as_ref(), &token)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?
            else {
                return Err(StatusCode::UNAUTHORIZED.into_response());
            };
            context
        };

        Ok(Self(context))
    }
}

fn extract_compat_gateway_token(parts: &Parts) -> Option<String> {
    extract_bearer_token(&parts.headers)
        .or_else(|| header_value(parts.headers.get("x-api-key")))
        .or_else(|| header_value(parts.headers.get("x-goog-api-key")))
        .or_else(|| query_parameter(parts.uri.query(), "key"))
}

async fn evaluate_gateway_request_rate_limit(
    store: &dyn AdminStore,
    context: &IdentityGatewayRequestContext,
    route_key: &str,
) -> anyhow::Result<RateLimitCheckResult> {
    check_rate_limit(
        store,
        context.project_id(),
        Some(context.api_key_hash()),
        route_key,
        None,
        1,
    )
    .await
}

fn rate_limit_exceeded_response(
    project_id: &str,
    route_key: &str,
    evaluation: &RateLimitCheckResult,
) -> Response {
    let mut error = OpenAiErrorResponse::new(
        rate_limit_exceeded_message(project_id, route_key, evaluation),
        "rate_limit_exceeded",
    );
    error.error.code = Some("rate_limit_exceeded".to_owned());
    (axum::http::StatusCode::TOO_MANY_REQUESTS, Json(error)).into_response()
}

fn apply_rate_limit_headers(
    headers: &mut HeaderMap,
    evaluation: &RateLimitCheckResult,
    include_retry_after: bool,
) {
    insert_optional_rate_limit_header(
        headers,
        "x-ratelimit-policy",
        evaluation.policy_id.as_deref(),
    );
    insert_optional_rate_limit_header(
        headers,
        "x-ratelimit-limit",
        evaluation.limit_requests.map(|value| value.to_string()),
    );
    insert_optional_rate_limit_header(
        headers,
        "x-ratelimit-remaining",
        evaluation.remaining_requests.map(|value| value.to_string()),
    );

    let reset_after_secs = rate_limit_reset_after_secs(evaluation);
    insert_optional_rate_limit_header(
        headers,
        "x-ratelimit-reset",
        reset_after_secs.map(|value| value.to_string()),
    );

    if include_retry_after {
        if let Some(retry_after) = reset_after_secs {
            if let Ok(value) = HeaderValue::from_str(&retry_after.to_string()) {
                headers.insert(header::RETRY_AFTER, value);
            }
        }
    }
}

fn insert_optional_rate_limit_header<T>(
    headers: &mut HeaderMap,
    name: &'static str,
    value: Option<T>,
) where
    T: Into<String>,
{
    let Some(value) = value.map(Into::into) else {
        return;
    };
    if let Ok(value) = HeaderValue::from_str(&value) {
        headers.insert(name, value);
    }
}

fn rate_limit_reset_after_secs(evaluation: &RateLimitCheckResult) -> Option<u64> {
    let window_end_ms = evaluation.window_end_ms?;
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_millis() as u64;
    let remaining_ms = window_end_ms.saturating_sub(now_ms);
    Some(remaining_ms.saturating_add(999).saturating_div(1000).max(1))
}

fn rate_limit_exceeded_message(
    project_id: &str,
    route_key: &str,
    evaluation: &RateLimitCheckResult,
) -> String {
    match (evaluation.policy_id.as_deref(), evaluation.limit_requests) {
        (Some(policy_id), Some(limit_requests)) => format!(
            "Rate limit exceeded for project {project_id} on route {route_key} under policy {policy_id}: requested {} requests with {} already used against a limit of {limit_requests}.",
            evaluation.requested_requests, evaluation.used_requests,
        ),
        (_, Some(limit_requests)) => format!(
            "Rate limit exceeded for project {project_id} on route {route_key}: requested {} requests with {} already used against a limit of {limit_requests}.",
            evaluation.requested_requests, evaluation.used_requests,
        ),
        _ => format!(
            "Rate limit exceeded for project {project_id} on route {route_key}: requested {} requests with {} already used.",
            evaluation.requested_requests, evaluation.used_requests,
        ),
    }
}

fn extract_bearer_token(headers: &axum::http::HeaderMap) -> Option<String> {
    let header_value = header_value(headers.get(header::AUTHORIZATION))?;
    header_value
        .strip_prefix("Bearer ")
        .or_else(|| header_value.strip_prefix("bearer "))
        .map(ToOwned::to_owned)
}

fn header_value(value: Option<&axum::http::HeaderValue>) -> Option<String> {
    value
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
}

fn request_options_from_header_names(
    headers: &HeaderMap,
    header_names: &[&str],
) -> ProviderRequestOptions {
    header_names.iter().fold(
        ProviderRequestOptions::default(),
        |options, name| match header_value(headers.get(*name)) {
            Some(value) => options.with_header(*name, value),
            None => options,
        },
    )
}

fn anthropic_request_options(headers: &HeaderMap) -> ProviderRequestOptions {
    request_options_from_header_names(headers, &["anthropic-version", "anthropic-beta"])
}

fn query_parameter(query: Option<&str>, key: &str) -> Option<String> {
    let query = query?;
    query.split('&').find_map(|pair| {
        let (name, value) = pair.split_once('=')?;
        if name == key {
            Some(value.to_owned())
        } else {
            None
        }
    })
}

fn current_gateway_request_context() -> Option<IdentityGatewayRequestContext> {
    CURRENT_GATEWAY_REQUEST_CONTEXT.try_with(Clone::clone).ok()
}

fn current_gateway_request_latency_ms() -> Option<u64> {
    CURRENT_GATEWAY_REQUEST_STARTED_AT
        .try_with(|started_at| started_at.elapsed().as_millis() as u64)
        .ok()
}

async fn apply_gateway_request_context(
    State(state): State<GatewayApiState>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    let token = extract_bearer_token(request.headers())
        .or_else(|| header_value(request.headers().get("x-api-key")))
        .or_else(|| header_value(request.headers().get("x-goog-api-key")))
        .or_else(|| query_parameter(request.uri().query(), "key"));

    let Some(token) = token else {
        return next.run(request).await;
    };

    let Ok(Some(context)) = resolve_gateway_request_context(state.store.as_ref(), &token).await
    else {
        return next.run(request).await;
    };

    request.extensions_mut().insert(context.clone());
    CURRENT_GATEWAY_REQUEST_CONTEXT
        .scope(
            context,
            with_request_api_key_group_id(
                request
                    .extensions()
                    .get::<IdentityGatewayRequestContext>()
                    .and_then(|context| context.api_key_group_id.clone()),
                CURRENT_GATEWAY_REQUEST_STARTED_AT.scope(Instant::now(), next.run(request)),
            ),
        )
        .await
}

async fn apply_gateway_rate_limit(
    State(state): State<GatewayApiState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if !request.uri().path().starts_with("/v1") {
        return next.run(request).await;
    }

    let Some(context) = request
        .extensions()
        .get::<IdentityGatewayRequestContext>()
        .cloned()
    else {
        return next.run(request).await;
    };

    let route_key = request.uri().path().to_owned();
    let evaluation =
        match evaluate_gateway_request_rate_limit(state.store.as_ref(), &context, &route_key).await
        {
            Ok(result) => result,
            Err(_) => {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to evaluate rate limit",
                )
                    .into_response();
            }
        };

    if !evaluation.allowed {
        let mut response =
            rate_limit_exceeded_response(context.project_id(), &route_key, &evaluation);
        apply_rate_limit_headers(response.headers_mut(), &evaluation, true);
        return response;
    }

    let mut response = next.run(request).await;
    apply_rate_limit_headers(response.headers_mut(), &evaluation, false);
    response
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatelessGatewayUpstream {
    runtime_key: String,
    protocol_kind: String,
    base_url: String,
    api_key: String,
}

impl StatelessGatewayUpstream {
    pub fn new(
        runtime_key: impl Into<String>,
        base_url: impl Into<String>,
        api_key: impl Into<String>,
    ) -> Self {
        let runtime_key = runtime_key.into();
        Self {
            protocol_kind: derive_stateless_protocol_kind(&runtime_key).to_owned(),
            runtime_key,
            base_url: base_url.into(),
            api_key: api_key.into(),
        }
    }

    pub fn new_with_protocol_kind(
        runtime_key: impl Into<String>,
        protocol_kind: impl Into<String>,
        base_url: impl Into<String>,
        api_key: impl Into<String>,
    ) -> Self {
        let runtime_key = runtime_key.into();
        Self {
            protocol_kind: normalize_stateless_protocol_kind(protocol_kind, &runtime_key),
            runtime_key,
            base_url: base_url.into(),
            api_key: api_key.into(),
        }
    }

    pub fn from_adapter_kind(
        adapter_kind: impl Into<String>,
        base_url: impl Into<String>,
        api_key: impl Into<String>,
    ) -> Self {
        let adapter_kind = adapter_kind.into();
        Self::new_with_protocol_kind(
            adapter_kind.clone(),
            derive_stateless_protocol_kind(&adapter_kind),
            base_url,
            api_key,
        )
    }

    pub fn from_default_plugin_family(
        default_plugin_family: impl AsRef<str>,
        base_url: impl Into<String>,
        api_key: impl Into<String>,
    ) -> anyhow::Result<Self> {
        let default_plugin_family =
            sdkwork_api_domain_catalog::normalize_provider_default_plugin_family(
                default_plugin_family,
            )
            .ok_or_else(|| anyhow::anyhow!("unsupported default_plugin_family"))?;

        Ok(Self::new_with_protocol_kind(
            default_plugin_family,
            sdkwork_api_domain_catalog::derive_provider_protocol_kind(default_plugin_family),
            base_url,
            api_key,
        ))
    }

    pub fn runtime_key(&self) -> &str {
        &self.runtime_key
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn protocol_kind(&self) -> &str {
        &self.protocol_kind
    }

    pub fn api_key(&self) -> &str {
        &self.api_key
    }
}

fn derive_stateless_protocol_kind(runtime_key: &str) -> &'static str {
    sdkwork_api_domain_catalog::derive_provider_protocol_kind(runtime_key)
}

fn normalize_stateless_protocol_kind(
    protocol_kind: impl Into<String>,
    runtime_key: &str,
) -> String {
    sdkwork_api_domain_catalog::normalize_provider_protocol_kind(protocol_kind, runtime_key)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatelessGatewayConfig {
    tenant_id: String,
    project_id: String,
    upstream: Option<StatelessGatewayUpstream>,
}

impl Default for StatelessGatewayConfig {
    fn default() -> Self {
        Self {
            tenant_id: DEFAULT_STATELESS_TENANT_ID.to_owned(),
            project_id: DEFAULT_STATELESS_PROJECT_ID.to_owned(),
            upstream: None,
        }
    }
}

impl StatelessGatewayConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_identity(
        mut self,
        tenant_id: impl Into<String>,
        project_id: impl Into<String>,
    ) -> Self {
        self.tenant_id = tenant_id.into();
        self.project_id = project_id.into();
        self
    }

    pub fn with_upstream(mut self, upstream: StatelessGatewayUpstream) -> Self {
        self.upstream = Some(upstream);
        self
    }

    pub fn try_with_default_plugin_upstream(
        self,
        default_plugin_family: impl AsRef<str>,
        base_url: impl Into<String>,
        api_key: impl Into<String>,
    ) -> anyhow::Result<Self> {
        Ok(self.with_upstream(StatelessGatewayUpstream::from_default_plugin_family(
            default_plugin_family,
            base_url,
            api_key,
        )?))
    }

    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    pub fn project_id(&self) -> &str {
        &self.project_id
    }

    pub fn upstream(&self) -> Option<&StatelessGatewayUpstream> {
        self.upstream.as_ref()
    }

    fn into_context(self) -> StatelessGatewayContext {
        StatelessGatewayContext {
            tenant_id: Arc::from(self.tenant_id),
            project_id: Arc::from(self.project_id),
            upstream: self.upstream.map(Arc::new),
        }
    }
}

#[derive(Clone, Debug)]
struct StatelessGatewayContext {
    tenant_id: Arc<str>,
    project_id: Arc<str>,
    upstream: Option<Arc<StatelessGatewayUpstream>>,
}

#[derive(Clone, Debug)]
struct StatelessGatewayRequest(StatelessGatewayContext);

impl StatelessGatewayRequest {
    fn tenant_id(&self) -> &str {
        &self.0.tenant_id
    }

    fn project_id(&self) -> &str {
        &self.0.project_id
    }

    fn upstream(&self) -> Option<&StatelessGatewayUpstream> {
        self.0.upstream.as_deref()
    }
}

impl FromRequestParts<StatelessGatewayContext> for StatelessGatewayRequest {
    type Rejection = StatusCode;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &StatelessGatewayContext,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self(state.clone()))
    }
}

async fn apply_request_routing_region(request: Request<Body>, next: Next) -> Response {
    let requested_region = request
        .headers()
        .get("x-sdkwork-region")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    with_request_routing_region(requested_region, next.run(request)).await
}
