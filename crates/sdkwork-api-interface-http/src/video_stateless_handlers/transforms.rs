use super::*;

fn local_video_transform_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_video_request",
        "Requested video was not found.",
    )
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

    let response = match remix_video(
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
        &request.prompt,
    ) {
        Ok(response) => response,
        Err(error) => return local_video_transform_error_response(error),
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

    let response = match extend_video(
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
        &request.prompt,
    ) {
        Ok(response) => response,
        Err(error) => return local_video_transform_error_response(error),
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

    let response = match sdkwork_api_app_gateway::edit_video(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => response,
        Err(error) => return local_video_transform_error_response(error),
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

    let response = match sdkwork_api_app_gateway::extensions_video(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => response,
        Err(error) => return local_video_transform_error_response(error),
    };

    Json(response).into_response()
}
