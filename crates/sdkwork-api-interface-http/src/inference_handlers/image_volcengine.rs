use super::*;
use crate::gateway_commercial::record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context;
use crate::gateway_provider_mirror_relay::{
    relay_provider_mirror_json_request, ProviderMirrorJsonRelayOutcome,
};
use sdkwork_api_app_gateway::{
    persist_planned_execution_decision_log, PlannedExecutionProviderContext,
};

const VOLCENGINE_MIRROR_PROTOCOL_IDENTITY: &str = "volcengine";
const IMAGES_VOLCENGINE_GENERATE_ROUTE_KEY: &str = "images.volcengine.generate";

fn volcengine_image_bad_gateway_response(action: &str) -> Response {
    bad_gateway_openai_response(format!(
        "failed to relay upstream volcengine image {action} request"
    ))
}

fn volcengine_image_invalid_request_response(message: impl Into<String>) -> Response {
    invalid_request_openai_response(message, "invalid_image_request")
}

async fn parse_volcengine_image_generation_request(
    request: Request<Body>,
) -> Result<(Request<Body>, String), Response> {
    let (parts, body) = request.into_parts();
    let body_bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .map_err(|_| volcengine_image_bad_gateway_response("generation"))?;
    let payload: Value = serde_json::from_slice(&body_bytes)
        .map_err(|error| volcengine_image_invalid_request_response(error.to_string()))?;
    let Some(model) = payload
        .get("model")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|model| !model.is_empty())
    else {
        return Err(volcengine_image_invalid_request_response(
            "Image generation model is required.",
        ));
    };

    Ok((
        Request::from_parts(parts, Body::from(body_bytes)),
        model.to_owned(),
    ))
}

async fn planned_volcengine_image_provider_context(
    state: &GatewayApiState,
    request_context: &AuthenticatedGatewayRequest,
    model: &str,
) -> anyhow::Result<Option<PlannedExecutionProviderContext>> {
    planned_execution_provider_context_for_route_and_mirror_identity_without_log(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        "images",
        model,
        VOLCENGINE_MIRROR_PROTOCOL_IDENTITY,
    )
    .await
}

async fn record_volcengine_image_usage(
    state: &GatewayApiState,
    request_context: &AuthenticatedGatewayRequest,
    planned_context: &PlannedExecutionProviderContext,
) -> anyhow::Result<()> {
    record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "images",
        IMAGES_VOLCENGINE_GENERATE_ROUTE_KEY,
        IMAGES_VOLCENGINE_GENERATE_ROUTE_KEY,
        0,
        0.0,
        None,
        None,
        Some(&planned_context.usage_context),
    )
    .await
}

pub(crate) async fn volcengine_image_generation_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    request: Request<Body>,
) -> Response {
    let (request, model) = match parse_volcengine_image_generation_request(request).await {
        Ok(parsed) => parsed,
        Err(response) => return response,
    };
    let planned_context =
        match planned_volcengine_image_provider_context(&state, &request_context, &model).await {
            Ok(Some(planned_context)) => planned_context,
            Ok(None) => return volcengine_image_bad_gateway_response("generation"),
            Err(_) => return volcengine_image_bad_gateway_response("generation"),
        };
    if planned_context.execution.local_fallback {
        return volcengine_image_bad_gateway_response("generation");
    }

    if persist_planned_execution_decision_log(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "images",
        IMAGES_VOLCENGINE_GENERATE_ROUTE_KEY,
        &planned_context.decision,
    )
    .await
    .is_err()
    {
        return volcengine_image_bad_gateway_response("generation");
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
            if record_volcengine_image_usage(&state, &request_context, &planned_context)
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
        Err(_) => volcengine_image_bad_gateway_response("generation"),
    }
}
