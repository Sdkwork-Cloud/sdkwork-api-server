use super::*;
use crate::gateway_provider_mirror_relay::{
    relay_provider_mirror_json_request, ProviderMirrorJsonRelayOutcome,
};

const SUNO_MIRROR_PROTOCOL_IDENTITY: &str = "suno";

fn suno_stateless_bad_gateway_response(action: &str) -> Response {
    bad_gateway_openai_response(format!("failed to relay upstream suno {action} request"))
}

async fn relay_stateless_suno_json_request(
    request_context: StatelessGatewayRequest,
    request: Request<Body>,
    action: &str,
) -> Response {
    let Some(upstream) = request_context.upstream() else {
        return suno_stateless_bad_gateway_response(action);
    };
    if upstream.mirror_protocol_identity() != SUNO_MIRROR_PROTOCOL_IDENTITY {
        return suno_stateless_bad_gateway_response(action);
    }

    match relay_provider_mirror_json_request(
        SUNO_MIRROR_PROTOCOL_IDENTITY,
        upstream.base_url(),
        upstream.api_key(),
        request,
    )
    .await
    {
        Ok(ProviderMirrorJsonRelayOutcome::Json(response)) => Json(response).into_response(),
        Ok(ProviderMirrorJsonRelayOutcome::Error(response)) => response,
        Err(_) => suno_stateless_bad_gateway_response(action),
    }
}

pub(crate) async fn music_suno_generate_handler(
    request_context: StatelessGatewayRequest,
    request: Request<Body>,
) -> Response {
    relay_stateless_suno_json_request(request_context, request, "generate").await
}

pub(crate) async fn music_suno_generate_record_info_handler(
    request_context: StatelessGatewayRequest,
    request: Request<Body>,
) -> Response {
    relay_stateless_suno_json_request(request_context, request, "generate record-info").await
}

pub(crate) async fn music_suno_lyrics_handler(
    request_context: StatelessGatewayRequest,
    request: Request<Body>,
) -> Response {
    relay_stateless_suno_json_request(request_context, request, "lyrics").await
}

pub(crate) async fn music_suno_lyrics_record_info_handler(
    request_context: StatelessGatewayRequest,
    request: Request<Body>,
) -> Response {
    relay_stateless_suno_json_request(request_context, request, "lyrics record-info").await
}
