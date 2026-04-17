use super::*;
use crate::gateway_commercial::record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context;
use crate::gateway_provider_mirror_relay::{
    relay_provider_mirror_json_request, ProviderMirrorJsonRelayOutcome,
};
use sdkwork_api_app_gateway::{
    persist_planned_execution_decision_log, PlannedExecutionProviderContext,
};

const MINIMAX_MIRROR_PROTOCOL_IDENTITY: &str = "minimax";
const VIDEO_MINIMAX_GENERATE_ROUTE_KEY: &str = "video.minimax.generate";
const VIDEO_MINIMAX_QUERY_ROUTE_KEY: &str = "video.minimax.query";
const VIDEO_MINIMAX_FILE_RETRIEVE_ROUTE_KEY: &str = "video.minimax.files.retrieve";

#[derive(Clone, Copy)]
enum MiniMaxVideoUsageMode {
    Generate,
    Query,
    FileRetrieve,
}

fn minimax_bad_gateway_response(action: &str) -> Response {
    bad_gateway_openai_response(format!("failed to relay upstream minimax {action} request"))
}

fn minimax_video_task_id(response: &Value) -> Option<&str> {
    response.get("task_id").and_then(Value::as_str)
}

fn minimax_video_file_id(response: &Value) -> Option<&str> {
    response
        .get("file")
        .and_then(|value| value.get("file_id"))
        .and_then(Value::as_str)
        .or_else(|| response.get("file_id").and_then(Value::as_str))
}

fn minimax_video_usage_reference_id(
    response: &Value,
    usage_mode: MiniMaxVideoUsageMode,
) -> Option<&str> {
    match usage_mode {
        MiniMaxVideoUsageMode::Generate | MiniMaxVideoUsageMode::Query => {
            minimax_video_task_id(response)
        }
        MiniMaxVideoUsageMode::FileRetrieve => minimax_video_file_id(response),
    }
}

async fn planned_video_minimax_provider_context(
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
        MINIMAX_MIRROR_PROTOCOL_IDENTITY,
    )
    .await
}

async fn record_minimax_video_usage(
    state: &GatewayApiState,
    request_context: &AuthenticatedGatewayRequest,
    planned_context: &PlannedExecutionProviderContext,
    route_key: &str,
    usage_mode: MiniMaxVideoUsageMode,
    response: &Value,
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
        minimax_video_usage_reference_id(response, usage_mode),
        Some(&planned_context.usage_context),
    )
    .await
}

async fn relay_stateful_minimax_json_request(
    request_context: AuthenticatedGatewayRequest,
    state: GatewayApiState,
    request: Request<Body>,
    route_key: &str,
    action: &str,
    usage_mode: MiniMaxVideoUsageMode,
) -> Response {
    let planned_context =
        match planned_video_minimax_provider_context(&state, &request_context, route_key).await {
            Ok(Some(planned_context)) => planned_context,
            Ok(None) => return minimax_bad_gateway_response(action),
            Err(_) => return minimax_bad_gateway_response(action),
        };
    if planned_context.execution.local_fallback {
        return minimax_bad_gateway_response(action);
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
        return minimax_bad_gateway_response(action);
    }

    match relay_provider_mirror_json_request(
        MINIMAX_MIRROR_PROTOCOL_IDENTITY,
        &planned_context.execution.base_url,
        &planned_context.api_key,
        request,
    )
    .await
    {
        Ok(ProviderMirrorJsonRelayOutcome::Json(response)) => {
            if record_minimax_video_usage(
                &state,
                &request_context,
                &planned_context,
                route_key,
                usage_mode,
                &response,
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
        Err(_) => minimax_bad_gateway_response(action),
    }
}

pub(crate) async fn video_minimax_generation_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    request: Request<Body>,
) -> Response {
    relay_stateful_minimax_json_request(
        request_context,
        state,
        request,
        VIDEO_MINIMAX_GENERATE_ROUTE_KEY,
        "video generation",
        MiniMaxVideoUsageMode::Generate,
    )
    .await
}

pub(crate) async fn video_minimax_generation_query_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    request: Request<Body>,
) -> Response {
    relay_stateful_minimax_json_request(
        request_context,
        state,
        request,
        VIDEO_MINIMAX_QUERY_ROUTE_KEY,
        "video generation query",
        MiniMaxVideoUsageMode::Query,
    )
    .await
}

pub(crate) async fn video_minimax_file_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    request: Request<Body>,
) -> Response {
    relay_stateful_minimax_json_request(
        request_context,
        state,
        request,
        VIDEO_MINIMAX_FILE_RETRIEVE_ROUTE_KEY,
        "file retrieve",
        MiniMaxVideoUsageMode::FileRetrieve,
    )
    .await
}
