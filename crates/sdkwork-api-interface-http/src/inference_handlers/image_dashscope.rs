use super::*;
use crate::gateway_commercial::record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context;
use crate::gateway_provider_mirror_relay::{
    relay_provider_mirror_json_request, ProviderMirrorJsonRelayOutcome,
};
use sdkwork_api_app_gateway::{
    persist_planned_execution_decision_log, PlannedExecutionProviderContext,
};

const DASHSCOPE_IMAGE_MIRROR_PROTOCOL_IDENTITIES: &[&str] = &["kling", "aliyun"];
const DASHSCOPE_MEDIA_TASK_CAPABILITIES: &[&str] = &["images", "videos"];

fn dashscope_image_bad_gateway_response(action: &str) -> Response {
    bad_gateway_openai_response(format!(
        "failed to relay upstream dashscope {action} request"
    ))
}

fn dashscope_image_invalid_request_response(message: impl Into<String>) -> Response {
    invalid_request_openai_response(message, "invalid_image_request")
}

fn is_dashscope_image_mirror_identity(mirror_protocol_identity: &str) -> bool {
    DASHSCOPE_IMAGE_MIRROR_PROTOCOL_IDENTITIES
        .iter()
        .any(|identity| mirror_protocol_identity.trim() == *identity)
}

fn dashscope_image_generation_route_key(mirror_protocol_identity: &str) -> String {
    format!("images.{mirror_protocol_identity}.generation")
}

fn dashscope_image_task_route_key(mirror_protocol_identity: &str) -> String {
    format!("provider.{mirror_protocol_identity}.tasks.get")
}

fn dashscope_image_task_id(response: &Value) -> Option<&str> {
    response
        .get("output")
        .and_then(|value| value.get("task_id"))
        .and_then(Value::as_str)
        .or_else(|| response.get("task_id").and_then(Value::as_str))
}

async fn parse_dashscope_image_generation_request(
    request: Request<Body>,
) -> Result<(Request<Body>, String), Response> {
    let (parts, body) = request.into_parts();
    let body_bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .map_err(|_| dashscope_image_bad_gateway_response("image generation"))?;
    let payload: Value = serde_json::from_slice(&body_bytes)
        .map_err(|error| dashscope_image_invalid_request_response(error.to_string()))?;
    let Some(model) = payload
        .get("model")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|model| !model.is_empty())
    else {
        return Err(dashscope_image_invalid_request_response(
            "Image generation model is required.",
        ));
    };

    Ok((
        Request::from_parts(parts, Body::from(body_bytes)),
        model.to_owned(),
    ))
}

async fn planned_dashscope_image_provider_context_for_model(
    state: &GatewayApiState,
    request_context: &AuthenticatedGatewayRequest,
    model: &str,
) -> anyhow::Result<Option<PlannedExecutionProviderContext>> {
    planned_execution_provider_context_for_route_and_mirror_identities_without_log(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        "images",
        model,
        DASHSCOPE_IMAGE_MIRROR_PROTOCOL_IDENTITIES,
    )
    .await
}

async fn latest_dashscope_media_provider_ownership_for_reference(
    state: &GatewayApiState,
    request_context: &AuthenticatedGatewayRequest,
    reference_id: &str,
) -> anyhow::Result<Option<(String, String)>> {
    Ok(state
        .store
        .list_billing_events()
        .await?
        .into_iter()
        .filter(|event| event.tenant_id == request_context.tenant_id())
        .filter(|event| event.project_id == request_context.project_id())
        .filter(|event| DASHSCOPE_MEDIA_TASK_CAPABILITIES.contains(&event.capability.as_str()))
        .filter(|event| event.reference_id.as_deref() == Some(reference_id))
        .max_by_key(|event| event.created_at_ms)
        .map(|event| (event.provider_id, event.capability)))
}

async fn planned_dashscope_media_provider_context_for_task(
    state: &GatewayApiState,
    request_context: &AuthenticatedGatewayRequest,
    task_id: &str,
) -> anyhow::Result<Option<(PlannedExecutionProviderContext, String, String)>> {
    let Some((provider_id, capability)) =
        latest_dashscope_media_provider_ownership_for_reference(state, request_context, task_id)
            .await?
    else {
        return Ok(None);
    };
    let Some(provider) = state.store.find_provider(&provider_id).await? else {
        return Ok(None);
    };
    let mirror_protocol_identity = provider.mirror_protocol_identity();
    if !is_dashscope_image_mirror_identity(&mirror_protocol_identity) {
        return Ok(None);
    }

    let route_key = dashscope_image_task_route_key(&mirror_protocol_identity);
    Ok(
        planned_execution_provider_context_for_route_and_provider_id_without_log(
            state.store.as_ref(),
            &state.secret_manager,
            request_context.tenant_id(),
            request_context.project_id(),
            &capability,
            &route_key,
            &provider_id,
        )
        .await?
        .map(|planned_context| (planned_context, route_key, capability)),
    )
}

async fn record_dashscope_image_usage(
    state: &GatewayApiState,
    request_context: &AuthenticatedGatewayRequest,
    planned_context: &PlannedExecutionProviderContext,
    capability: &str,
    route_key: &str,
    reference_id: Option<&str>,
) -> anyhow::Result<()> {
    record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        capability,
        route_key,
        route_key,
        0,
        0.0,
        None,
        reference_id,
        Some(&planned_context.usage_context),
    )
    .await
}

async fn relay_stateful_dashscope_image_json_request(
    request_context: AuthenticatedGatewayRequest,
    state: GatewayApiState,
    request: Request<Body>,
    planned_context: PlannedExecutionProviderContext,
    capability: &str,
    route_key: String,
    action: &str,
    reference_id: Option<&str>,
) -> Response {
    if planned_context.execution.local_fallback {
        return dashscope_image_bad_gateway_response(action);
    }

    if persist_planned_execution_decision_log(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        capability,
        &route_key,
        &planned_context.decision,
    )
    .await
    .is_err()
    {
        return dashscope_image_bad_gateway_response(action);
    }

    let mirror_protocol_identity = planned_context.provider.mirror_protocol_identity();
    match relay_provider_mirror_json_request(
        &mirror_protocol_identity,
        &planned_context.execution.base_url,
        &planned_context.api_key,
        request,
    )
    .await
    {
        Ok(ProviderMirrorJsonRelayOutcome::Json(response)) => {
            let reference_id = reference_id.or_else(|| dashscope_image_task_id(&response));
            if record_dashscope_image_usage(
                &state,
                &request_context,
                &planned_context,
                capability,
                &route_key,
                reference_id,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            Json(response).into_response()
        }
        Ok(ProviderMirrorJsonRelayOutcome::Error(response)) => response,
        Err(_) => dashscope_image_bad_gateway_response(action),
    }
}

pub(crate) async fn dashscope_image_generation_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    request: Request<Body>,
) -> Response {
    let (request, model) = match parse_dashscope_image_generation_request(request).await {
        Ok(parsed) => parsed,
        Err(response) => return response,
    };
    let planned_context =
        match planned_dashscope_image_provider_context_for_model(&state, &request_context, &model)
            .await
        {
            Ok(Some(planned_context)) => planned_context,
            Ok(None) => return dashscope_image_bad_gateway_response("image generation"),
            Err(_) => return dashscope_image_bad_gateway_response("image generation"),
        };
    let route_key =
        dashscope_image_generation_route_key(&planned_context.provider.mirror_protocol_identity());

    relay_stateful_dashscope_image_json_request(
        request_context,
        state,
        request,
        planned_context,
        "images",
        route_key,
        "image generation",
        None,
    )
    .await
}

pub(crate) async fn dashscope_image_task_get_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(task_id): Path<String>,
    request: Request<Body>,
) -> Response {
    let (planned_context, route_key, capability) =
        match planned_dashscope_media_provider_context_for_task(&state, &request_context, &task_id)
            .await
        {
            Ok(Some(result)) => result,
            Ok(None) => return dashscope_image_bad_gateway_response("task query"),
            Err(_) => return dashscope_image_bad_gateway_response("task query"),
        };

    relay_stateful_dashscope_image_json_request(
        request_context,
        state,
        request,
        planned_context,
        &capability,
        route_key,
        "task query",
        Some(task_id.as_str()),
    )
    .await
}
