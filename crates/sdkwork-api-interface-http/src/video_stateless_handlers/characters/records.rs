use super::*;

fn local_video_character_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_video_character_request",
        "Requested video character was not found.",
    )
}

pub(crate) async fn video_characters_list_handler(
    request_context: StatelessGatewayRequest,
    Path(video_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VideoCharactersList(&video_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video characters list");
        }
    }

    let response = match list_video_characters(
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_video_character_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn video_character_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((video_id, character_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VideoCharactersRetrieve(&video_id, &character_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream video character retrieve",
            );
        }
    }

    let response = match get_video_character(
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
        &character_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_video_character_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn video_character_update_handler(
    request_context: StatelessGatewayRequest,
    Path((video_id, character_id)): Path<(String, String)>,
    ExtractJson(request): ExtractJson<UpdateVideoCharacterRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VideoCharactersUpdate(&video_id, &character_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video character update");
        }
    }

    let response = match update_video_character(
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
        &character_id,
        &request,
    ) {
        Ok(response) => response,
        Err(error) => return local_video_character_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn video_character_create_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateVideoCharacterRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VideoCharactersCreate(&request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video character create");
        }
    }

    let response = match sdkwork_api_app_gateway::create_video_character(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => response,
        Err(error) => return local_video_character_error_response(error),
    };

    Json(response).into_response()
}
