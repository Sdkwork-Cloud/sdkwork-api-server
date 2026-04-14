use super::*;

pub(crate) fn extract_compat_gateway_token(parts: &Parts) -> Option<String> {
    extract_bearer_token(&parts.headers)
        .or_else(|| header_value(parts.headers.get("x-api-key")))
        .or_else(|| header_value(parts.headers.get("x-goog-api-key")))
        .or_else(|| query_parameter(parts.uri.query(), "key"))
}

pub(crate) async fn resolve_authenticated_gateway_request_context(
    state: &GatewayApiState,
    token: &str,
) -> anyhow::Result<Option<IdentityGatewayRequestContext>> {
    if let Some(context) = resolve_gateway_request_context(state.store.as_ref(), token).await? {
        return Ok(Some(context));
    }

    let Some(identity_store) = state.identity_store.as_ref() else {
        return Ok(None);
    };

    let identity_store: &dyn IdentityKernelStore = identity_store.as_ref();
    resolve_canonical_gateway_request_context_from_api_key(identity_store, token).await
}

pub(crate) async fn enforce_gateway_request_rate_limit(
    store: &dyn AdminStore,
    context: &IdentityGatewayRequestContext,
    route_key: &str,
) -> Result<(), Response> {
    match check_rate_limit(
        store,
        context.project_id(),
        Some(context.api_key_hash()),
        route_key,
        None,
        1,
    )
    .await
    {
        Ok(result) if result.allowed => Ok(()),
        Ok(result) => Err(rate_limit_exceeded_response(
            context.project_id(),
            route_key,
            &result,
        )),
        Err(_) => Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to evaluate rate limit",
        )
            .into_response()),
    }
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

pub(crate) fn extract_bearer_token(headers: &axum::http::HeaderMap) -> Option<String> {
    let header_value = header_value(headers.get(header::AUTHORIZATION))?;
    header_value
        .strip_prefix("Bearer ")
        .or_else(|| header_value.strip_prefix("bearer "))
        .map(ToOwned::to_owned)
}

pub(crate) fn header_value(value: Option<&axum::http::HeaderValue>) -> Option<String> {
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

pub(crate) fn anthropic_request_options(headers: &HeaderMap) -> ProviderRequestOptions {
    request_options_from_header_names(headers, &["anthropic-version", "anthropic-beta"])
}

pub(crate) fn query_parameter(query: Option<&str>, key: &str) -> Option<String> {
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
