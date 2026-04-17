use super::*;
use crate::gateway_commercial::record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context;
use crate::gateway_provider_mirror_relay::{
    relay_provider_mirror_json_request, ProviderMirrorJsonRelayOutcome,
};
use sdkwork_api_app_gateway::{
    persist_planned_execution_decision_log, PlannedExecutionProviderContext,
};

const VOLCENGINE_MIRROR_PROTOCOL_IDENTITY: &str = "volcengine";
const VIDEO_VOLCENGINE_TASKS_CREATE_ROUTE_KEY: &str = "video.volcengine.tasks.create";
const VIDEO_VOLCENGINE_TASKS_GET_ROUTE_KEY: &str = "video.volcengine.tasks.get";

fn volcengine_bad_gateway_response(action: &str) -> Response {
    bad_gateway_openai_response(format!(
        "failed to relay upstream volcengine {action} request"
    ))
}

fn volcengine_invalid_request_response(message: impl Into<String>) -> Response {
    invalid_request_openai_response(message, "invalid_video_request")
}

fn volcengine_task_id(response: &Value) -> Option<&str> {
    response
        .get("id")
        .and_then(Value::as_str)
        .or_else(|| response.get("task_id").and_then(Value::as_str))
}

async fn parse_volcengine_task_create_request(
    request: Request<Body>,
) -> Result<(Request<Body>, String), Response> {
    let (parts, body) = request.into_parts();
    let body_bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .map_err(|_| volcengine_bad_gateway_response("task create"))?;
    let payload: Value = serde_json::from_slice(&body_bytes)
        .map_err(|error| volcengine_invalid_request_response(error.to_string()))?;
    let Some(model) = payload
        .get("model")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|model| !model.is_empty())
    else {
        return Err(volcengine_invalid_request_response(
            "Volcengine video model is required.",
        ));
    };

    Ok((
        Request::from_parts(parts, Body::from(body_bytes)),
        model.to_owned(),
    ))
}

async fn planned_volcengine_provider_context_for_model(
    state: &GatewayApiState,
    request_context: &AuthenticatedGatewayRequest,
    model: &str,
) -> anyhow::Result<Option<PlannedExecutionProviderContext>> {
    planned_execution_provider_context_for_route_and_mirror_identity_without_log(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        model,
        VOLCENGINE_MIRROR_PROTOCOL_IDENTITY,
    )
    .await
}

async fn latest_volcengine_provider_id_for_reference(
    state: &GatewayApiState,
    request_context: &AuthenticatedGatewayRequest,
    reference_id: &str,
) -> anyhow::Result<Option<String>> {
    Ok(state
        .store
        .list_billing_events()
        .await?
        .into_iter()
        .filter(|event| event.tenant_id == request_context.tenant_id())
        .filter(|event| event.project_id == request_context.project_id())
        .filter(|event| event.capability == "videos")
        .filter(|event| event.reference_id.as_deref() == Some(reference_id))
        .max_by_key(|event| event.created_at_ms)
        .map(|event| event.provider_id))
}

async fn planned_volcengine_provider_context_for_task(
    state: &GatewayApiState,
    request_context: &AuthenticatedGatewayRequest,
    task_id: &str,
) -> anyhow::Result<Option<PlannedExecutionProviderContext>> {
    let Some(provider_id) =
        latest_volcengine_provider_id_for_reference(state, request_context, task_id).await?
    else {
        return Ok(None);
    };

    planned_execution_provider_context_for_route_and_provider_id_without_log(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        VIDEO_VOLCENGINE_TASKS_GET_ROUTE_KEY,
        &provider_id,
    )
    .await
}

async fn record_volcengine_video_usage(
    state: &GatewayApiState,
    request_context: &AuthenticatedGatewayRequest,
    planned_context: &PlannedExecutionProviderContext,
    route_key: &str,
    reference_id: Option<&str>,
) -> anyhow::Result<()> {
    record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        route_key,
        route_key,
        1,
        0.0,
        None,
        reference_id,
        Some(&planned_context.usage_context),
    )
    .await
}

async fn relay_stateful_volcengine_json_request(
    request_context: AuthenticatedGatewayRequest,
    state: GatewayApiState,
    request: Request<Body>,
    planned_context: PlannedExecutionProviderContext,
    route_key: &str,
    action: &str,
    reference_id: Option<&str>,
) -> Response {
    if planned_context.execution.local_fallback {
        return volcengine_bad_gateway_response(action);
    }

    if persist_planned_execution_decision_log(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        route_key,
        &planned_context.decision,
    )
    .await
    .is_err()
    {
        return volcengine_bad_gateway_response(action);
    }

    match relay_provider_mirror_json_request(
        VOLCENGINE_MIRROR_PROTOCOL_IDENTITY,
        &planned_context.execution.base_url,
        &planned_context.api_key,
        request,
    )
    .await
    {
        Ok(ProviderMirrorJsonRelayOutcome::Json(response)) => {
            let usage_reference_id = reference_id.or_else(|| volcengine_task_id(&response));
            if record_volcengine_video_usage(
                &state,
                &request_context,
                &planned_context,
                route_key,
                usage_reference_id,
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
        Err(_) => volcengine_bad_gateway_response(action),
    }
}

pub(crate) async fn video_volcengine_task_create_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    request: Request<Body>,
) -> Response {
    let (request, model) = match parse_volcengine_task_create_request(request).await {
        Ok(parsed) => parsed,
        Err(response) => return response,
    };
    let planned_context =
        match planned_volcengine_provider_context_for_model(&state, &request_context, &model).await
        {
            Ok(Some(planned_context)) => planned_context,
            Ok(None) => return volcengine_bad_gateway_response("task create"),
            Err(_) => return volcengine_bad_gateway_response("task create"),
        };

    relay_stateful_volcengine_json_request(
        request_context,
        state,
        request,
        planned_context,
        VIDEO_VOLCENGINE_TASKS_CREATE_ROUTE_KEY,
        "task create",
        None,
    )
    .await
}

pub(crate) async fn video_volcengine_task_get_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(id): Path<String>,
    request: Request<Body>,
) -> Response {
    let planned_context =
        match planned_volcengine_provider_context_for_task(&state, &request_context, &id).await {
            Ok(Some(planned_context)) => planned_context,
            Ok(None) => return volcengine_bad_gateway_response("task get"),
            Err(_) => return volcengine_bad_gateway_response("task get"),
        };

    relay_stateful_volcengine_json_request(
        request_context,
        state,
        request,
        planned_context,
        VIDEO_VOLCENGINE_TASKS_GET_ROUTE_KEY,
        "task get",
        Some(id.as_str()),
    )
    .await
}
