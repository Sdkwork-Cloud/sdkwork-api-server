use super::*;

fn local_video_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_video_request",
        "Requested video was not found.",
    )
}

pub(crate) async fn videos_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateVideoRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Videos(&request)).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video");
        }
    }

    let response = match create_video(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
        &request.prompt,
    ) {
        Ok(response) => response,
        Err(error) => return local_video_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn videos_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::VideosList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream videos list");
        }
    }

    let response = match list_videos(request_context.tenant_id(), request_context.project_id()) {
        Ok(response) => response,
        Err(error) => return local_video_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn video_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(video_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::VideosRetrieve(&video_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video retrieve");
        }
    }

    let response = match get_video(
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_video_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn video_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(video_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::VideosDelete(&video_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video delete");
        }
    }

    let response = match delete_video(
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_video_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn video_content_handler(
    request_context: StatelessGatewayRequest,
    Path(video_id): Path<String>,
) -> Response {
    match relay_stateless_stream_request(
        &request_context,
        ProviderRequest::VideosContent(&video_id),
    )
    .await
    {
        Ok(Some(response)) => return upstream_passthrough_response(response),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video content");
        }
    }

    local_video_content_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
    )
}
