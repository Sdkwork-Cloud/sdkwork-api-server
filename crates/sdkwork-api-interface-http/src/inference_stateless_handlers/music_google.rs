use super::*;
use crate::gateway_provider_mirror_relay::{
    relay_provider_mirror_json_request, ProviderMirrorJsonRelayOutcome,
};

const GOOGLE_MUSIC_MIRROR_PROTOCOL_IDENTITY: &str = "google";

fn google_music_stateless_bad_gateway_response(action: &str) -> Response {
    bad_gateway_openai_response(format!(
        "failed to relay upstream google music {action} request"
    ))
}

pub(crate) async fn relay_stateless_google_music_predict_request(
    request_context: StatelessGatewayRequest,
    request: Request<Body>,
) -> Response {
    let Some(upstream) = request_context.upstream() else {
        return google_music_stateless_bad_gateway_response("predict");
    };
    if upstream.mirror_protocol_identity() != GOOGLE_MUSIC_MIRROR_PROTOCOL_IDENTITY {
        return google_music_stateless_bad_gateway_response("predict");
    }

    match relay_provider_mirror_json_request(
        GOOGLE_MUSIC_MIRROR_PROTOCOL_IDENTITY,
        upstream.base_url(),
        upstream.api_key(),
        request,
    )
    .await
    {
        Ok(ProviderMirrorJsonRelayOutcome::Json(response)) => Json(response).into_response(),
        Ok(ProviderMirrorJsonRelayOutcome::Error(response)) => response,
        Err(_) => google_music_stateless_bad_gateway_response("predict"),
    }
}
