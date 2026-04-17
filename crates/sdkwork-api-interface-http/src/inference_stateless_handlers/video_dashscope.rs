use super::*;
use crate::gateway_provider_mirror_relay::{
    relay_provider_mirror_json_request, ProviderMirrorJsonRelayOutcome,
};

const DASHSCOPE_VIDEO_MIRROR_PROTOCOL_IDENTITIES: &[&str] = &["kling", "aliyun"];

fn dashscope_video_stateless_bad_gateway_response(action: &str) -> Response {
    bad_gateway_openai_response(format!(
        "failed to relay upstream dashscope {action} request"
    ))
}

fn is_dashscope_video_mirror_identity(mirror_protocol_identity: &str) -> bool {
    DASHSCOPE_VIDEO_MIRROR_PROTOCOL_IDENTITIES
        .iter()
        .any(|identity| mirror_protocol_identity.trim() == *identity)
}

async fn relay_stateless_dashscope_video_json_request(
    request_context: StatelessGatewayRequest,
    request: Request<Body>,
    action: &str,
) -> Response {
    let Some(upstream) = request_context.upstream() else {
        return dashscope_video_stateless_bad_gateway_response(action);
    };
    let mirror_protocol_identity = upstream.mirror_protocol_identity();
    if !is_dashscope_video_mirror_identity(mirror_protocol_identity) {
        return dashscope_video_stateless_bad_gateway_response(action);
    }

    match relay_provider_mirror_json_request(
        mirror_protocol_identity,
        upstream.base_url(),
        upstream.api_key(),
        request,
    )
    .await
    {
        Ok(ProviderMirrorJsonRelayOutcome::Json(response)) => Json(response).into_response(),
        Ok(ProviderMirrorJsonRelayOutcome::Error(response)) => response,
        Err(_) => dashscope_video_stateless_bad_gateway_response(action),
    }
}

pub(crate) async fn dashscope_video_synthesis_handler(
    request_context: StatelessGatewayRequest,
    request: Request<Body>,
) -> Response {
    relay_stateless_dashscope_video_json_request(request_context, request, "video synthesis").await
}
