use super::*;
use crate::gateway_commercial::record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context;
use crate::gateway_provider_mirror_relay::{
    relay_provider_mirror_json_request, ProviderMirrorJsonRelayOutcome,
};
use sdkwork_api_app_gateway::{
    persist_planned_execution_decision_log, PlannedExecutionProviderContext,
};

const VIDU_MIRROR_PROTOCOL_IDENTITY: &str = "vidu";
const VIDEO_VIDU_TEXT2VIDEO_ROUTE_KEY: &str = "video.vidu.text2video";
const VIDEO_VIDU_IMG2VIDEO_ROUTE_KEY: &str = "video.vidu.img2video";
const VIDEO_VIDU_REFERENCE2VIDEO_ROUTE_KEY: &str = "video.vidu.reference2video";
const VIDEO_VIDU_TASK_CREATIONS_ROUTE_KEY: &str = "video.vidu.creations.get";
const VIDEO_VIDU_TASK_CANCEL_ROUTE_KEY: &str = "video.vidu.cancel";

fn vidu_bad_gateway_response(action: &str) -> Response {
    bad_gateway_openai_response(format!("failed to relay upstream vidu {action} request"))
}

fn vidu_task_id(response: &Value) -> Option<&str> {
    response
        .get("task_id")
        .and_then(Value::as_str)
        .or_else(|| response.get("id").and_then(Value::as_str))
}

async fn planned_video_vidu_provider_context(
    state: &GatewayApiState,
    request_context: &AuthenticatedGatewayRequest,
    route_key: &str,
) -> anyhow::Result<Option<PlannedExecutionProviderContext>> {
    planned_execution_provider_context_for_route_and_mirror_identity_without_log(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        route_key,
        VIDU_MIRROR_PROTOCOL_IDENTITY,
    )
    .await
}

async fn relay_stateful_vidu_json_request(
    request_context: AuthenticatedGatewayRequest,
    state: GatewayApiState,
    request: Request<Body>,
    route_key: &str,
    action: &str,
    reference_id: Option<&str>,
) -> Response {
    let planned_context =
        match planned_video_vidu_provider_context(&state, &request_context, route_key).await {
            Ok(Some(planned_context)) => planned_context,
            Ok(None) => return vidu_bad_gateway_response(action),
            Err(_) => return vidu_bad_gateway_response(action),
        };
    if planned_context.execution.local_fallback {
        return vidu_bad_gateway_response(action);
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
        return vidu_bad_gateway_response(action);
    }

    match relay_provider_mirror_json_request(
        VIDU_MIRROR_PROTOCOL_IDENTITY,
        &planned_context.execution.base_url,
        &planned_context.api_key,
        request,
    )
    .await
    {
        Ok(ProviderMirrorJsonRelayOutcome::Json(response)) => {
            let reference_id = reference_id.or_else(|| vidu_task_id(&response));
            if record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context(
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
        Err(_) => vidu_bad_gateway_response(action),
    }
}

pub(crate) async fn video_vidu_text2video_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    request: Request<Body>,
) -> Response {
    relay_stateful_vidu_json_request(
        request_context,
        state,
        request,
        VIDEO_VIDU_TEXT2VIDEO_ROUTE_KEY,
        "text2video",
        None,
    )
    .await
}

pub(crate) async fn video_vidu_img2video_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    request: Request<Body>,
) -> Response {
    relay_stateful_vidu_json_request(
        request_context,
        state,
        request,
        VIDEO_VIDU_IMG2VIDEO_ROUTE_KEY,
        "img2video",
        None,
    )
    .await
}

pub(crate) async fn video_vidu_reference2video_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    request: Request<Body>,
) -> Response {
    relay_stateful_vidu_json_request(
        request_context,
        state,
        request,
        VIDEO_VIDU_REFERENCE2VIDEO_ROUTE_KEY,
        "reference2video",
        None,
    )
    .await
}

pub(crate) async fn video_vidu_task_creations_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(task_id): Path<String>,
    request: Request<Body>,
) -> Response {
    relay_stateful_vidu_json_request(
        request_context,
        state,
        request,
        VIDEO_VIDU_TASK_CREATIONS_ROUTE_KEY,
        "task creations",
        Some(task_id.as_str()),
    )
    .await
}

pub(crate) async fn video_vidu_task_cancel_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(task_id): Path<String>,
    request: Request<Body>,
) -> Response {
    relay_stateful_vidu_json_request(
        request_context,
        state,
        request,
        VIDEO_VIDU_TASK_CANCEL_ROUTE_KEY,
        "task cancel",
        Some(task_id.as_str()),
    )
    .await
}
