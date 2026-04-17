use super::*;
use crate::gateway_provider_mirror_relay::{
    relay_provider_mirror_json_request, ProviderMirrorJsonRelayOutcome,
};

const VIDU_MIRROR_PROTOCOL_IDENTITY: &str = "vidu";

fn vidu_stateless_bad_gateway_response(action: &str) -> Response {
    bad_gateway_openai_response(format!("failed to relay upstream vidu {action} request"))
}

async fn relay_stateless_vidu_json_request(
    request_context: StatelessGatewayRequest,
    request: Request<Body>,
    action: &str,
) -> Response {
    let Some(upstream) = request_context.upstream() else {
        return vidu_stateless_bad_gateway_response(action);
    };
    if upstream.mirror_protocol_identity() != VIDU_MIRROR_PROTOCOL_IDENTITY {
        return vidu_stateless_bad_gateway_response(action);
    }

    match relay_provider_mirror_json_request(
        VIDU_MIRROR_PROTOCOL_IDENTITY,
        upstream.base_url(),
        upstream.api_key(),
        request,
    )
    .await
    {
        Ok(ProviderMirrorJsonRelayOutcome::Json(response)) => Json(response).into_response(),
        Ok(ProviderMirrorJsonRelayOutcome::Error(response)) => response,
        Err(_) => vidu_stateless_bad_gateway_response(action),
    }
}

pub(crate) async fn video_vidu_text2video_handler(
    request_context: StatelessGatewayRequest,
    request: Request<Body>,
) -> Response {
    relay_stateless_vidu_json_request(request_context, request, "text2video").await
}

pub(crate) async fn video_vidu_img2video_handler(
    request_context: StatelessGatewayRequest,
    request: Request<Body>,
) -> Response {
    relay_stateless_vidu_json_request(request_context, request, "img2video").await
}

pub(crate) async fn video_vidu_reference2video_handler(
    request_context: StatelessGatewayRequest,
    request: Request<Body>,
) -> Response {
    relay_stateless_vidu_json_request(request_context, request, "reference2video").await
}

pub(crate) async fn video_vidu_task_creations_handler(
    request_context: StatelessGatewayRequest,
    request: Request<Body>,
) -> Response {
    relay_stateless_vidu_json_request(request_context, request, "task creations").await
}

pub(crate) async fn video_vidu_task_cancel_handler(
    request_context: StatelessGatewayRequest,
    request: Request<Body>,
) -> Response {
    relay_stateless_vidu_json_request(request_context, request, "task cancel").await
}
