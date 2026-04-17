use super::*;
use crate::gateway_commercial::record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context;
use crate::gateway_provider_mirror_relay::{
    relay_provider_mirror_json_request, ProviderMirrorJsonRelayOutcome,
};
use sdkwork_api_app_gateway::{
    persist_planned_execution_decision_log, PlannedExecutionProviderContext,
};

const DASHSCOPE_VIDEO_MIRROR_PROTOCOL_IDENTITIES: &[&str] = &["kling", "aliyun"];

fn dashscope_video_bad_gateway_response(action: &str) -> Response {
    bad_gateway_openai_response(format!(
        "failed to relay upstream dashscope {action} request"
    ))
}

fn dashscope_video_invalid_request_response(message: impl Into<String>) -> Response {
    invalid_request_openai_response(message, "invalid_video_request")
}

fn dashscope_video_synthesis_route_key(mirror_protocol_identity: &str) -> String {
    format!("video.{mirror_protocol_identity}.synthesis")
}

fn dashscope_video_task_id(response: &Value) -> Option<&str> {
    response
        .get("output")
        .and_then(|value| value.get("task_id"))
        .and_then(Value::as_str)
        .or_else(|| response.get("task_id").and_then(Value::as_str))
}

async fn parse_dashscope_video_synthesis_request(
    request: Request<Body>,
) -> Result<(Request<Body>, String), Response> {
    let (parts, body) = request.into_parts();
    let body_bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .map_err(|_| dashscope_video_bad_gateway_response("video synthesis"))?;
    let payload: Value = serde_json::from_slice(&body_bytes)
        .map_err(|error| dashscope_video_invalid_request_response(error.to_string()))?;
    let Some(model) = payload
        .get("model")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|model| !model.is_empty())
    else {
        return Err(dashscope_video_invalid_request_response(
            "Video synthesis model is required.",
        ));
    };

    Ok((
        Request::from_parts(parts, Body::from(body_bytes)),
        model.to_owned(),
    ))
}

async fn planned_dashscope_video_provider_context_for_model(
    state: &GatewayApiState,
    request_context: &AuthenticatedGatewayRequest,
    model: &str,
) -> anyhow::Result<Option<PlannedExecutionProviderContext>> {
    planned_execution_provider_context_for_route_and_mirror_identities_without_log(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        model,
        DASHSCOPE_VIDEO_MIRROR_PROTOCOL_IDENTITIES,
    )
    .await
}

async fn record_dashscope_video_usage(
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

async fn relay_stateful_dashscope_video_json_request(
    request_context: AuthenticatedGatewayRequest,
    state: GatewayApiState,
    request: Request<Body>,
    planned_context: PlannedExecutionProviderContext,
    route_key: String,
    action: &str,
    reference_id: Option<&str>,
) -> Response {
    if planned_context.execution.local_fallback {
        return dashscope_video_bad_gateway_response(action);
    }

    if persist_planned_execution_decision_log(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        &route_key,
        &planned_context.decision,
    )
    .await
    .is_err()
    {
        return dashscope_video_bad_gateway_response(action);
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
            let reference_id = reference_id.or_else(|| dashscope_video_task_id(&response));
            if record_dashscope_video_usage(
                &state,
                &request_context,
                &planned_context,
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
        Err(_) => dashscope_video_bad_gateway_response(action),
    }
}

pub(crate) async fn dashscope_video_synthesis_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    request: Request<Body>,
) -> Response {
    let (request, model) = match parse_dashscope_video_synthesis_request(request).await {
        Ok(parsed) => parsed,
        Err(response) => return response,
    };
    let planned_context =
        match planned_dashscope_video_provider_context_for_model(&state, &request_context, &model)
            .await
        {
            Ok(Some(planned_context)) => planned_context,
            Ok(None) => return dashscope_video_bad_gateway_response("video synthesis"),
            Err(_) => return dashscope_video_bad_gateway_response("video synthesis"),
        };
    let route_key =
        dashscope_video_synthesis_route_key(&planned_context.provider.mirror_protocol_identity());

    relay_stateful_dashscope_video_json_request(
        request_context,
        state,
        request,
        planned_context,
        route_key,
        "video synthesis",
        None,
    )
    .await
}
