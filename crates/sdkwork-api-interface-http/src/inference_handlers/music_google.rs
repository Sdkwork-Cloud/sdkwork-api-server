use super::*;
use crate::gateway_commercial::record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context;
use crate::gateway_provider_mirror_relay::{
    relay_provider_mirror_json_request, ProviderMirrorJsonRelayOutcome,
};
use sdkwork_api_app_gateway::{
    persist_planned_execution_decision_log, PlannedExecutionProviderContext,
};

const GOOGLE_MUSIC_MIRROR_PROTOCOL_IDENTITY: &str = "google";
const MUSIC_GOOGLE_PREDICT_ROUTE_KEY: &str = "music.google.predict";

fn google_music_bad_gateway_response(action: &str) -> Response {
    bad_gateway_openai_response(format!(
        "failed to relay upstream google music {action} request"
    ))
}

async fn planned_google_music_provider_context_for_model(
    state: &GatewayApiState,
    request_context: &AuthenticatedGatewayRequest,
    model: &str,
) -> anyhow::Result<Option<PlannedExecutionProviderContext>> {
    planned_execution_provider_context_for_route_and_mirror_identity_without_log(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        "music",
        model,
        GOOGLE_MUSIC_MIRROR_PROTOCOL_IDENTITY,
    )
    .await
}

async fn record_google_music_usage(
    state: &GatewayApiState,
    request_context: &AuthenticatedGatewayRequest,
    planned_context: &PlannedExecutionProviderContext,
) -> anyhow::Result<()> {
    record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "music",
        MUSIC_GOOGLE_PREDICT_ROUTE_KEY,
        MUSIC_GOOGLE_PREDICT_ROUTE_KEY,
        1,
        0.0,
        None,
        None,
        Some(&planned_context.usage_context),
    )
    .await
}

pub(crate) async fn relay_stateful_google_music_predict_request(
    request_context: AuthenticatedGatewayRequest,
    state: GatewayApiState,
    model: String,
    request: Request<Body>,
) -> Response {
    let planned_context =
        match planned_google_music_provider_context_for_model(&state, &request_context, &model)
            .await
        {
            Ok(Some(planned_context)) => planned_context,
            Ok(None) => return google_music_bad_gateway_response("predict"),
            Err(_) => return google_music_bad_gateway_response("predict"),
        };
    if planned_context.execution.local_fallback {
        return google_music_bad_gateway_response("predict");
    }

    if persist_planned_execution_decision_log(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "music",
        MUSIC_GOOGLE_PREDICT_ROUTE_KEY,
        &planned_context.decision,
    )
    .await
    .is_err()
    {
        return google_music_bad_gateway_response("predict");
    }

    match relay_provider_mirror_json_request(
        GOOGLE_MUSIC_MIRROR_PROTOCOL_IDENTITY,
        &planned_context.execution.base_url,
        &planned_context.api_key,
        request,
    )
    .await
    {
        Ok(ProviderMirrorJsonRelayOutcome::Json(response)) => {
            if record_google_music_usage(&state, &request_context, &planned_context)
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
        Err(_) => google_music_bad_gateway_response("predict"),
    }
}
