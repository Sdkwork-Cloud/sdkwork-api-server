use super::*;
use crate::gateway_commercial::{
    music_billing_amount, music_billing_units,
    record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context,
    record_gateway_usage_for_project_with_route_key_and_tokens_reference_and_media_with_context,
    BillingMediaMetrics,
};
use crate::gateway_provider_mirror_relay::{
    relay_provider_mirror_json_request, ProviderMirrorJsonRelayOutcome,
};
use sdkwork_api_app_gateway::{
    persist_planned_execution_decision_log, PlannedExecutionProviderContext,
};

const MINIMAX_MIRROR_PROTOCOL_IDENTITY: &str = "minimax";
const MUSIC_MINIMAX_GENERATE_ROUTE_KEY: &str = "music.minimax.generate";
const MUSIC_MINIMAX_LYRICS_ROUTE_KEY: &str = "music.minimax.lyrics";

enum MiniMaxUsageMode {
    MusicGeneration,
    LyricsGeneration,
}

fn minimax_bad_gateway_response(action: &str) -> Response {
    bad_gateway_openai_response(format!("failed to relay upstream minimax {action} request"))
}

fn minimax_usage_reference_id(response: &Value) -> Option<&str> {
    response.get("trace_id").and_then(Value::as_str)
}

fn minimax_music_seconds(response: &Value) -> Option<f64> {
    response
        .get("extra_info")
        .and_then(|value| value.get("music_duration"))
        .and_then(|value| {
            value
                .as_f64()
                .or_else(|| value.as_u64().map(|raw| raw as f64))
                .or_else(|| value.as_i64().map(|raw| raw as f64))
        })
        .map(|milliseconds| milliseconds / 1000.0)
        .filter(|seconds| *seconds > 0.0)
}

async fn planned_music_minimax_provider_context(
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
        MINIMAX_MIRROR_PROTOCOL_IDENTITY,
    )
    .await
}

async fn record_minimax_usage(
    state: &GatewayApiState,
    request_context: &AuthenticatedGatewayRequest,
    planned_context: &PlannedExecutionProviderContext,
    route_key: &str,
    usage_mode: MiniMaxUsageMode,
    response: &Value,
) -> anyhow::Result<()> {
    match usage_mode {
        MiniMaxUsageMode::MusicGeneration => {
            if let Some(music_seconds) = minimax_music_seconds(response) {
                record_gateway_usage_for_project_with_route_key_and_tokens_reference_and_media_with_context(
                    state.store.as_ref(),
                    request_context.tenant_id(),
                    request_context.project_id(),
                    "music",
                    route_key,
                    route_key,
                    music_billing_units(music_seconds),
                    music_billing_amount(music_seconds),
                    None,
                    minimax_usage_reference_id(response),
                    BillingMediaMetrics {
                        music_seconds,
                        ..BillingMediaMetrics::default()
                    },
                    Some(&planned_context.usage_context),
                )
                .await
            } else {
                record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context(
                    state.store.as_ref(),
                    request_context.tenant_id(),
                    request_context.project_id(),
                    "music",
                    route_key,
                    route_key,
                    1,
                    0.0,
                    None,
                    minimax_usage_reference_id(response),
                    Some(&planned_context.usage_context),
                )
                .await
            }
        }
        MiniMaxUsageMode::LyricsGeneration => {
            record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "music",
                route_key,
                route_key,
                1,
                0.0,
                None,
                minimax_usage_reference_id(response),
                Some(&planned_context.usage_context),
            )
            .await
        }
    }
}

async fn relay_stateful_minimax_json_request(
    request_context: AuthenticatedGatewayRequest,
    state: GatewayApiState,
    request: Request<Body>,
    route_key: &str,
    action: &str,
    usage_mode: MiniMaxUsageMode,
) -> Response {
    let planned_context =
        match planned_music_minimax_provider_context(&state, &request_context, route_key).await {
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
        "music",
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
            if record_minimax_usage(
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

pub(crate) async fn music_minimax_generation_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    request: Request<Body>,
) -> Response {
    relay_stateful_minimax_json_request(
        request_context,
        state,
        request,
        MUSIC_MINIMAX_GENERATE_ROUTE_KEY,
        "music generation",
        MiniMaxUsageMode::MusicGeneration,
    )
    .await
}

pub(crate) async fn music_minimax_lyrics_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    request: Request<Body>,
) -> Response {
    relay_stateful_minimax_json_request(
        request_context,
        state,
        request,
        MUSIC_MINIMAX_LYRICS_ROUTE_KEY,
        "lyrics generation",
        MiniMaxUsageMode::LyricsGeneration,
    )
    .await
}
