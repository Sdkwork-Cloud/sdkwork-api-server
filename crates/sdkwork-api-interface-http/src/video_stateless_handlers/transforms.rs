use super::*;
use sdkwork_api_contract_openai::videos::{VideoObject, VideosResponse};

fn video_missing(video_id: &str) -> bool {
    video_id.trim().is_empty() || video_id.ends_with("_missing")
}

fn local_video_variant(video_id: &str, suffix: &str) -> VideoObject {
    let id = format!("{video_id}_{suffix}");
    VideoObject::new(id.clone(), format!("https://example.com/{id}.mp4"))
}

fn local_video_transform_result(
    video_id: &str,
    prompt: &str,
    suffix: &str,
) -> std::result::Result<VideosResponse, Response> {
    if video_missing(video_id) {
        return Err(local_gateway_error_response(
            anyhow::anyhow!("video not found"),
            "Requested video was not found.",
        ));
    }
    if prompt.trim().is_empty() {
        return Err(local_gateway_invalid_or_bad_gateway_response(
            anyhow::anyhow!("Video prompt is required."),
            "invalid_video_request",
        ));
    }

    Ok(VideosResponse::new(vec![local_video_variant(
        video_id, suffix,
    )]))
}

fn local_video_edits_result(
    request: &EditVideoRequest,
) -> std::result::Result<VideosResponse, Response> {
    local_video_transform_result(&request.video_id, &request.prompt, "edited")
}

fn local_video_extensions_result(
    request: &ExtendVideoRequest,
) -> std::result::Result<VideosResponse, Response> {
    if let Some(video_id) = request.video_id.as_deref() {
        return local_video_transform_result(video_id, &request.prompt, "extended");
    }
    if request.prompt.trim().is_empty() {
        return Err(local_gateway_invalid_or_bad_gateway_response(
            anyhow::anyhow!("Video prompt is required."),
            "invalid_video_request",
        ));
    }

    Ok(VideosResponse::new(vec![local_video_variant(
        "video_1", "extended",
    )]))
}

pub(crate) async fn video_remix_handler(
    request_context: StatelessGatewayRequest,
    Path(video_id): Path<String>,
    ExtractJson(request): ExtractJson<RemixVideoRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VideosRemix(&video_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video remix");
        }
    }

    let response = match local_video_transform_result(&video_id, &request.prompt, "remix") {
        Ok(response) => response,
        Err(response) => return response,
    };

    Json(response).into_response()
}

pub(crate) async fn video_extend_handler(
    request_context: StatelessGatewayRequest,
    Path(video_id): Path<String>,
    ExtractJson(request): ExtractJson<ExtendVideoRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VideosExtend(&video_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video extend");
        }
    }

    let response = match local_video_transform_result(&video_id, &request.prompt, "extended") {
        Ok(response) => response,
        Err(response) => return response,
    };

    Json(response).into_response()
}

pub(crate) async fn video_edits_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<EditVideoRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::VideosEdits(&request))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video edits");
        }
    }

    let response = match local_video_edits_result(&request) {
        Ok(response) => response,
        Err(response) => return response,
    };

    Json(response).into_response()
}

pub(crate) async fn video_extensions_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<ExtendVideoRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VideosExtensions(&request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video extensions");
        }
    }

    let response = match local_video_extensions_result(&request) {
        Ok(response) => response,
        Err(response) => return response,
    };

    Json(response).into_response()
}
