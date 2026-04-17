use super::*;
use crate::gateway_commercial::record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context;
use crate::gateway_provider_mirror_relay::{
    relay_provider_mirror_json_request, ProviderMirrorJsonRelayOutcome,
};
use sdkwork_api_app_gateway::{
    persist_planned_execution_decision_log, PlannedExecutionProviderContext,
};

const GOOGLE_VEO_MIRROR_PROTOCOL_IDENTITY: &str = "google-veo";
const VIDEO_GOOGLE_VEO_PREDICT_LONG_RUNNING_ROUTE_KEY: &str =
    "video.google-veo.predict_long_running";
const VIDEO_GOOGLE_VEO_FETCH_PREDICT_OPERATION_ROUTE_KEY: &str =
    "video.google-veo.fetch_predict_operation";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GoogleVeoAction {
    PredictLongRunning,
    FetchPredictOperation,
    Predict,
}

fn google_veo_bad_gateway_response(action: &str) -> Response {
    bad_gateway_openai_response(format!(
        "failed to relay upstream google veo {action} request"
    ))
}

fn google_veo_invalid_request_response(message: impl Into<String>) -> Response {
    invalid_request_openai_response(message, "invalid_video_request")
}

fn parse_google_veo_models_tail(tail: &str) -> Option<(String, GoogleVeoAction)> {
    let (model, action) = tail.split_once(':')?;
    let action = match action.trim() {
        "predictLongRunning" => GoogleVeoAction::PredictLongRunning,
        "fetchPredictOperation" => GoogleVeoAction::FetchPredictOperation,
        "predict" => GoogleVeoAction::Predict,
        _ => return None,
    };
    let model = model.trim();
    if model.is_empty() {
        return None;
    }
    Some((model.to_owned(), action))
}

fn google_veo_operation_name_from_response(response: &Value) -> Option<&str> {
    response.get("name").and_then(Value::as_str)
}

async fn parse_google_veo_fetch_request(
    request: Request<Body>,
) -> Result<(Request<Body>, String), Response> {
    let (parts, body) = request.into_parts();
    let body_bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .map_err(|_| google_veo_bad_gateway_response("fetchPredictOperation"))?;
    let payload: Value = serde_json::from_slice(&body_bytes)
        .map_err(|error| google_veo_invalid_request_response(error.to_string()))?;
    let Some(operation_name) = payload
        .get("operationName")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Err(google_veo_invalid_request_response(
            "Google Veo operationName is required.",
        ));
    };

    Ok((
        Request::from_parts(parts, Body::from(body_bytes)),
        operation_name.to_owned(),
    ))
}

async fn planned_google_veo_provider_context_for_model(
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
        GOOGLE_VEO_MIRROR_PROTOCOL_IDENTITY,
    )
    .await
}

async fn latest_google_veo_provider_id_for_reference(
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

async fn planned_google_veo_provider_context_for_operation(
    state: &GatewayApiState,
    request_context: &AuthenticatedGatewayRequest,
    operation_name: &str,
) -> anyhow::Result<Option<PlannedExecutionProviderContext>> {
    let Some(provider_id) =
        latest_google_veo_provider_id_for_reference(state, request_context, operation_name).await?
    else {
        return Ok(None);
    };

    planned_execution_provider_context_for_route_and_provider_id_without_log(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        VIDEO_GOOGLE_VEO_FETCH_PREDICT_OPERATION_ROUTE_KEY,
        &provider_id,
    )
    .await
}

async fn record_google_veo_usage(
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

async fn relay_stateful_google_veo_json_request(
    request_context: AuthenticatedGatewayRequest,
    state: GatewayApiState,
    request: Request<Body>,
    planned_context: PlannedExecutionProviderContext,
    route_key: &str,
    action_name: &str,
    reference_id: Option<&str>,
) -> Response {
    if planned_context.execution.local_fallback {
        return google_veo_bad_gateway_response(action_name);
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
        return google_veo_bad_gateway_response(action_name);
    }

    match relay_provider_mirror_json_request(
        GOOGLE_VEO_MIRROR_PROTOCOL_IDENTITY,
        &planned_context.execution.base_url,
        &planned_context.api_key,
        request,
    )
    .await
    {
        Ok(ProviderMirrorJsonRelayOutcome::Json(response)) => {
            let usage_reference_id =
                reference_id.or_else(|| google_veo_operation_name_from_response(&response));
            if record_google_veo_usage(
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
        Err(_) => google_veo_bad_gateway_response(action_name),
    }
}

pub(crate) async fn video_google_veo_models_action_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((_project, _location, tail)): Path<(String, String, String)>,
    request: Request<Body>,
) -> Response {
    let Some((model, action)) = parse_google_veo_models_tail(&tail) else {
        return google_veo_invalid_request_response("unsupported google veo models action");
    };

    match action {
        GoogleVeoAction::PredictLongRunning => {
            let planned_context = match planned_google_veo_provider_context_for_model(
                &state,
                &request_context,
                &model,
            )
            .await
            {
                Ok(Some(planned_context)) => planned_context,
                Ok(None) => return google_veo_bad_gateway_response("predictLongRunning"),
                Err(_) => return google_veo_bad_gateway_response("predictLongRunning"),
            };

            relay_stateful_google_veo_json_request(
                request_context,
                state,
                request,
                planned_context,
                VIDEO_GOOGLE_VEO_PREDICT_LONG_RUNNING_ROUTE_KEY,
                "predictLongRunning",
                None,
            )
            .await
        }
        GoogleVeoAction::FetchPredictOperation => {
            let (request, operation_name) = match parse_google_veo_fetch_request(request).await {
                Ok(parsed) => parsed,
                Err(response) => return response,
            };
            let planned_context = match planned_google_veo_provider_context_for_operation(
                &state,
                &request_context,
                &operation_name,
            )
            .await
            {
                Ok(Some(planned_context)) => planned_context,
                Ok(None) => return google_veo_bad_gateway_response("fetchPredictOperation"),
                Err(_) => return google_veo_bad_gateway_response("fetchPredictOperation"),
            };

            relay_stateful_google_veo_json_request(
                request_context,
                state,
                request,
                planned_context,
                VIDEO_GOOGLE_VEO_FETCH_PREDICT_OPERATION_ROUTE_KEY,
                "fetchPredictOperation",
                Some(operation_name.as_str()),
            )
            .await
        }
        GoogleVeoAction::Predict => {
            super::music_google::relay_stateful_google_music_predict_request(
                request_context,
                state,
                model,
                request,
            )
            .await
        }
    }
}
