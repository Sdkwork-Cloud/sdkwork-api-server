use super::*;
use crate::gateway_provider_mirror_relay::{
    relay_provider_mirror_json_request, ProviderMirrorJsonRelayOutcome,
};

const MINIMAX_MIRROR_PROTOCOL_IDENTITY: &str = "minimax";

fn minimax_stateless_bad_gateway_response(action: &str) -> Response {
    bad_gateway_openai_response(format!("failed to relay upstream minimax {action} request"))
}

async fn relay_stateless_minimax_json_request(
    request_context: StatelessGatewayRequest,
    request: Request<Body>,
    action: &str,
) -> Response {
    let Some(upstream) = request_context.upstream() else {
        return minimax_stateless_bad_gateway_response(action);
    };
    if upstream.mirror_protocol_identity() != MINIMAX_MIRROR_PROTOCOL_IDENTITY {
        return minimax_stateless_bad_gateway_response(action);
    }

    match relay_provider_mirror_json_request(
        MINIMAX_MIRROR_PROTOCOL_IDENTITY,
        upstream.base_url(),
        upstream.api_key(),
        request,
    )
    .await
    {
        Ok(ProviderMirrorJsonRelayOutcome::Json(response)) => Json(response).into_response(),
        Ok(ProviderMirrorJsonRelayOutcome::Error(response)) => response,
        Err(_) => minimax_stateless_bad_gateway_response(action),
    }
}

pub(crate) async fn video_minimax_generation_handler(
    request_context: StatelessGatewayRequest,
    request: Request<Body>,
) -> Response {
    relay_stateless_minimax_json_request(request_context, request, "video generation").await
}

pub(crate) async fn video_minimax_generation_query_handler(
    request_context: StatelessGatewayRequest,
    request: Request<Body>,
) -> Response {
    relay_stateless_minimax_json_request(request_context, request, "video generation query").await
}

pub(crate) async fn video_minimax_file_retrieve_handler(
    request_context: StatelessGatewayRequest,
    request: Request<Body>,
) -> Response {
    relay_stateless_minimax_json_request(request_context, request, "file retrieve").await
}
