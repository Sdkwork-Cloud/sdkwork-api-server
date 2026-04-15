use super::*;
use std::time::{SystemTime, UNIX_EPOCH};

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
    let identity_store = state
        .identity_store
        .as_ref()
        .map(|store| store.as_ref() as &dyn IdentityKernelStore);

    if let Some(context) = resolve_gateway_request_context(state.store.as_ref(), token).await? {
        if let Some(identity_store) = identity_store {
            return Ok(Some(
                enrich_gateway_request_context_with_canonical_subject(
                    identity_store,
                    token,
                    context,
                )
                .await?,
            ));
        }

        return Ok(Some(context));
    }

    let Some(identity_store) = identity_store else {
        return Ok(None);
    };

    resolve_canonical_gateway_request_context_from_api_key(identity_store, token).await
}

async fn enrich_gateway_request_context_with_canonical_subject(
    identity_store: &dyn IdentityKernelStore,
    token: &str,
    context: IdentityGatewayRequestContext,
) -> anyhow::Result<IdentityGatewayRequestContext> {
    let Some(subject) =
        sdkwork_api_app_identity::resolve_gateway_auth_subject_from_api_key(identity_store, token)
            .await?
    else {
        return Ok(context);
    };

    Ok(context.with_canonical_subject(
        subject.tenant_id,
        subject.organization_id,
        subject.user_id,
        subject.api_key_id,
    ))
}

pub(crate) async fn enforce_gateway_request_rate_limit(
    store: &dyn AdminStore,
    context: &IdentityGatewayRequestContext,
    route_key: &str,
) -> Result<(), Response> {
    evaluate_gateway_request_rate_limit(store, context, route_key)
        .await
        .map(|_| ())
}

pub(crate) async fn evaluate_gateway_request_rate_limit(
    store: &dyn AdminStore,
    context: &IdentityGatewayRequestContext,
    route_key: &str,
) -> Result<Option<RateLimitCheckResult>, Response> {
    if gateway_rate_limit_is_excluded_route(route_key) {
        return Ok(None);
    }

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
        Ok(result) if result.allowed => Ok(Some(result)),
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
    let mut response = (axum::http::StatusCode::TOO_MANY_REQUESTS, Json(error)).into_response();
    append_gateway_rate_limit_headers(&mut response, evaluation);
    response
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

pub(crate) fn append_gateway_rate_limit_headers(
    response: &mut Response,
    evaluation: &RateLimitCheckResult,
) {
    let headers = response.headers_mut();

    if let Some(policy_id) = evaluation.policy_id.as_deref() {
        if let Ok(value) = axum::http::HeaderValue::from_str(policy_id) {
            headers.insert("x-ratelimit-policy", value);
        }
    }

    if let Some(limit_requests) = evaluation.limit_requests {
        if let Ok(value) = axum::http::HeaderValue::from_str(&limit_requests.to_string()) {
            headers.insert("x-ratelimit-limit", value);
        }
    }

    if let Some(remaining_requests) = evaluation.remaining_requests {
        if let Ok(value) = axum::http::HeaderValue::from_str(&remaining_requests.to_string()) {
            headers.insert("x-ratelimit-remaining", value);
        }
    }

    if let Some(reset_seconds) = rate_limit_reset_seconds(evaluation) {
        if let Ok(value) = axum::http::HeaderValue::from_str(&reset_seconds.to_string()) {
            headers.insert("x-ratelimit-reset", value.clone());
            if !evaluation.allowed {
                headers.insert("retry-after", value);
            }
        }
    }
}

fn gateway_rate_limit_is_excluded_route(route_key: &str) -> bool {
    matches!(
        route_key,
        "/openapi.json" | "/docs" | "/health" | "/metrics"
    )
}

fn rate_limit_reset_seconds(evaluation: &RateLimitCheckResult) -> Option<u64> {
    let window_end_ms = evaluation.window_end_ms?;
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0);
    let remaining_ms = window_end_ms.saturating_sub(now_ms);
    Some(remaining_ms.div_ceil(1000))
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
