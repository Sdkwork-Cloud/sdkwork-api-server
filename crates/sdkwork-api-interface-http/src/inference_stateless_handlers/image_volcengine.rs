use super::*;
use crate::gateway_provider_mirror_relay::{
    relay_provider_mirror_json_request, ProviderMirrorJsonRelayOutcome,
};

const VOLCENGINE_MIRROR_PROTOCOL_IDENTITY: &str = "volcengine";

fn volcengine_image_stateless_bad_gateway_response(action: &str) -> Response {
    bad_gateway_openai_response(format!(
        "failed to relay upstream volcengine image {action} request"
    ))
}

pub(crate) async fn volcengine_image_generation_handler(
    request_context: StatelessGatewayRequest,
    request: Request<Body>,
) -> Response {
    let Some(upstream) = request_context.upstream() else {
        return volcengine_image_stateless_bad_gateway_response("generation");
    };
    if upstream.mirror_protocol_identity() != VOLCENGINE_MIRROR_PROTOCOL_IDENTITY {
        return volcengine_image_stateless_bad_gateway_response("generation");
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
        Err(_) => volcengine_image_stateless_bad_gateway_response("generation"),
    }
}
