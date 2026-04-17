use super::*;
use crate::gateway_commercial::record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context;
use crate::gateway_provider_mirror_relay::{
    relay_provider_mirror_json_request, ProviderMirrorJsonRelayOutcome,
};
use sdkwork_api_app_gateway::{
    persist_planned_execution_decision_log, PlannedExecutionProviderContext,
};

const SUNO_MIRROR_PROTOCOL_IDENTITY: &str = "suno";
const MUSIC_SUNO_GENERATE_ROUTE_KEY: &str = "music.suno.generate";
const MUSIC_SUNO_GENERATE_RECORD_INFO_ROUTE_KEY: &str = "music.suno.generate.record-info";
const MUSIC_SUNO_LYRICS_ROUTE_KEY: &str = "music.suno.lyrics";
const MUSIC_SUNO_LYRICS_RECORD_INFO_ROUTE_KEY: &str = "music.suno.lyrics.record-info";

fn suno_bad_gateway_response(action: &str) -> Response {
    bad_gateway_openai_response(format!("failed to relay upstream suno {action} request"))
}

fn suno_usage_reference_id(response: &Value) -> Option<&str> {
    response
        .get("taskId")
        .and_then(Value::as_str)
        .or_else(|| response.get("id").and_then(Value::as_str))
}

async fn planned_music_suno_provider_context(
    state: &GatewayApiState,
    request_context: &AuthenticatedGatewayRequest,
    route_key: &str,
) -> anyhow::Result<Option<PlannedExecutionProviderContext>> {
    planned_execution_provider_context_for_route_and_mirror_identity_without_log(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        "music",
        route_key,
        SUNO_MIRROR_PROTOCOL_IDENTITY,
    )
    .await
}

async fn relay_stateful_suno_json_request(
    request_context: AuthenticatedGatewayRequest,
    state: GatewayApiState,
    request: Request<Body>,
    route_key: &str,
    action: &str,
) -> Response {
    let planned_context =
        match planned_music_suno_provider_context(&state, &request_context, route_key).await {
            Ok(Some(planned_context)) => planned_context,
            Ok(None) => return suno_bad_gateway_response(action),
            Err(_) => return suno_bad_gateway_response(action),
        };
    if planned_context.execution.local_fallback {
        return suno_bad_gateway_response(action);
    }

    if persist_planned_execution_decision_log(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "music",
        route_key,
        &planned_context.decision,
    )
    .await
    .is_err()
    {
        return suno_bad_gateway_response(action);
    }

    match relay_provider_mirror_json_request(
        SUNO_MIRROR_PROTOCOL_IDENTITY,
        &planned_context.execution.base_url,
        &planned_context.api_key,
        request,
    )
    .await
    {
        Ok(ProviderMirrorJsonRelayOutcome::Json(response)) => {
            if record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "music",
                route_key,
                route_key,
                1,
                0.0,
                None,
                suno_usage_reference_id(&response),
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
        Err(_) => suno_bad_gateway_response(action),
    }
}

pub(crate) async fn music_suno_generate_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    request: Request<Body>,
) -> Response {
    relay_stateful_suno_json_request(
        request_context,
        state,
        request,
        MUSIC_SUNO_GENERATE_ROUTE_KEY,
        "generate",
    )
    .await
}

pub(crate) async fn music_suno_generate_record_info_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    request: Request<Body>,
) -> Response {
    relay_stateful_suno_json_request(
        request_context,
        state,
        request,
        MUSIC_SUNO_GENERATE_RECORD_INFO_ROUTE_KEY,
        "generate record-info",
    )
    .await
}

pub(crate) async fn music_suno_lyrics_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    request: Request<Body>,
) -> Response {
    relay_stateful_suno_json_request(
        request_context,
        state,
        request,
        MUSIC_SUNO_LYRICS_ROUTE_KEY,
        "lyrics",
    )
    .await
}

pub(crate) async fn music_suno_lyrics_record_info_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    request: Request<Body>,
) -> Response {
    relay_stateful_suno_json_request(
        request_context,
        state,
        request,
        MUSIC_SUNO_LYRICS_RECORD_INFO_ROUTE_KEY,
        "lyrics record-info",
    )
    .await
}
