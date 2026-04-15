use super::auth_utils::{
    append_gateway_rate_limit_headers, evaluate_gateway_request_rate_limit, extract_bearer_token,
    header_value, query_parameter, resolve_authenticated_gateway_request_context,
};
use super::*;

tokio::task_local! {
    static CURRENT_GATEWAY_REQUEST_CONTEXT: IdentityGatewayRequestContext;
}

tokio::task_local! {
    static CURRENT_GATEWAY_REQUEST_STARTED_AT: Instant;
}

pub(crate) fn current_gateway_request_context() -> Option<IdentityGatewayRequestContext> {
    CURRENT_GATEWAY_REQUEST_CONTEXT.try_with(Clone::clone).ok()
}

pub(crate) fn current_gateway_request_latency_ms() -> Option<u64> {
    CURRENT_GATEWAY_REQUEST_STARTED_AT
        .try_with(|started_at| started_at.elapsed().as_millis() as u64)
        .ok()
}

pub(crate) async fn apply_gateway_request_context(
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

    let Ok(Some(context)) = resolve_authenticated_gateway_request_context(&state, &token).await
    else {
        return next.run(request).await;
    };

    let route_key = request.uri().path().to_owned();
    let rate_limit_evaluation =
        match evaluate_gateway_request_rate_limit(state.store.as_ref(), &context, &route_key).await
        {
            Ok(result) => result,
            Err(response) => return response,
        };

    let api_key_group_id = context.api_key_group_id().map(ToOwned::to_owned);
    request.extensions_mut().insert(context.clone());
    let mut response = CURRENT_GATEWAY_REQUEST_CONTEXT
        .scope(
            context,
            with_request_api_key_group_id(
                api_key_group_id,
                CURRENT_GATEWAY_REQUEST_STARTED_AT.scope(Instant::now(), next.run(request)),
            ),
        )
        .await;
    if let Some(evaluation) = rate_limit_evaluation.as_ref() {
        append_gateway_rate_limit_headers(&mut response, evaluation);
    }
    response
}
