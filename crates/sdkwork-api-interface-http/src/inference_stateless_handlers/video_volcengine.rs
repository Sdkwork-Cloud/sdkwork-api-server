use super::*;
use crate::gateway_provider_mirror_relay::{
    relay_provider_mirror_json_request, ProviderMirrorJsonRelayOutcome,
};

const VOLCENGINE_MIRROR_PROTOCOL_IDENTITY: &str = "volcengine";

fn volcengine_stateless_bad_gateway_response(action: &str) -> Response {
    bad_gateway_openai_response(format!(
        "failed to relay upstream volcengine {action} request"
    ))
}

async fn relay_stateless_volcengine_json_request(
    request_context: StatelessGatewayRequest,
    request: Request<Body>,
    action: &str,
) -> Response {
    let Some(upstream) = request_context.upstream() else {
        return volcengine_stateless_bad_gateway_response(action);
    };
    if upstream.mirror_protocol_identity() != VOLCENGINE_MIRROR_PROTOCOL_IDENTITY {
        return volcengine_stateless_bad_gateway_response(action);
    }

    match relay_provider_mirror_json_request(
        VOLCENGINE_MIRROR_PROTOCOL_IDENTITY,
        upstream.base_url(),
        upstream.api_key(),
        request,
    )
    .await
    {
        Ok(ProviderMirrorJsonRelayOutcome::Json(response)) => Json(response).into_response(),
        Ok(ProviderMirrorJsonRelayOutcome::Error(response)) => response,
        Err(_) => volcengine_stateless_bad_gateway_response(action),
    }
}

pub(crate) async fn video_volcengine_task_create_handler(
    request_context: StatelessGatewayRequest,
    request: Request<Body>,
) -> Response {
    relay_stateless_volcengine_json_request(request_context, request, "task create").await
}

pub(crate) async fn video_volcengine_task_get_handler(
    request_context: StatelessGatewayRequest,
    request: Request<Body>,
) -> Response {
    relay_stateless_volcengine_json_request(request_context, request, "task get").await
}
