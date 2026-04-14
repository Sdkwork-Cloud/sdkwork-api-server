use super::*;

fn local_music_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_music_request",
        "Requested music was not found.",
    )
}

pub(crate) async fn music_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateMusicRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Music(&request)).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream music create");
        }
    }

    let response = match create_music(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => response,
        Err(error) => return local_music_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn music_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::MusicList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream music list");
        }
    }

    let response = match list_music(request_context.tenant_id(), request_context.project_id()) {
        Ok(response) => response,
        Err(error) => return local_music_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn music_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(music_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::MusicRetrieve(&music_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream music retrieve");
        }
    }

    let response = match get_music(
        request_context.tenant_id(),
        request_context.project_id(),
        &music_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_music_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn music_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(music_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::MusicDelete(&music_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream music delete");
        }
    }

    let response = match delete_music(
        request_context.tenant_id(),
        request_context.project_id(),
        &music_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_music_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn music_content_handler(
    request_context: StatelessGatewayRequest,
    Path(music_id): Path<String>,
) -> Response {
    match relay_stateless_stream_request(&request_context, ProviderRequest::MusicContent(&music_id))
        .await
    {
        Ok(Some(response)) => return upstream_passthrough_response(response),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream music content");
        }
    }

    local_music_content_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &music_id,
    )
}

pub(crate) async fn music_lyrics_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateMusicLyricsRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::MusicLyrics(&request))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream music lyrics");
        }
    }

    let response = match create_music_lyrics(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => response,
        Err(error) => return local_music_error_response(error),
    };

    Json(response).into_response()
}
