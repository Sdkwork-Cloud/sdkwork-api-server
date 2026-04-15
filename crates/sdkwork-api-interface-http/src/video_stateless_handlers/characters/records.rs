use super::*;
use sdkwork_api_contract_openai::videos::{VideoCharacterObject, VideoCharactersResponse};

fn local_video_character_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_video_character_request",
        "Requested video character was not found.",
    )
}

fn video_missing(video_id: &str) -> bool {
    video_id.trim().is_empty() || video_id.ends_with("_missing")
}

fn character_missing(character_id: &str) -> bool {
    character_id.trim().is_empty() || character_id.ends_with("_missing")
}

fn local_video_character_placeholder(character_id: &str, name: &str) -> VideoCharacterObject {
    VideoCharacterObject::new(character_id, name)
}

fn local_video_characters_placeholder() -> VideoCharactersResponse {
    VideoCharactersResponse::new(vec![VideoCharacterObject::new("char_1", "Hero")])
}

fn local_video_characters_list_result(
    video_id: &str,
) -> std::result::Result<VideoCharactersResponse, Response> {
    if video_missing(video_id) {
        return Err(local_video_character_error_response(anyhow::anyhow!(
            "video character not found"
        )));
    }

    Ok(local_video_characters_placeholder())
}

fn local_video_character_retrieve_result(
    video_id: &str,
    character_id: &str,
) -> std::result::Result<VideoCharacterObject, Response> {
    if video_missing(video_id) || character_missing(character_id) {
        return Err(local_video_character_error_response(anyhow::anyhow!(
            "video character not found"
        )));
    }

    Ok(local_video_character_placeholder(character_id, "Hero"))
}

fn local_video_character_update_result(
    video_id: &str,
    character_id: &str,
    request: &UpdateVideoCharacterRequest,
) -> std::result::Result<VideoCharacterObject, Response> {
    if video_missing(video_id) || character_missing(character_id) {
        return Err(local_video_character_error_response(anyhow::anyhow!(
            "video character not found"
        )));
    }
    let Some(name) = request
        .name
        .as_deref()
        .filter(|name| !name.trim().is_empty())
    else {
        return Err(invalid_request_openai_response(
            "Video character name is required.",
            "invalid_video_character_request",
        ));
    };

    Ok(local_video_character_placeholder(character_id, name))
}

fn local_video_character_create_result(
    request: &CreateVideoCharacterRequest,
) -> std::result::Result<VideoCharacterObject, Response> {
    if video_missing(&request.video_id) {
        return Err(local_video_character_error_response(anyhow::anyhow!(
            "video character not found"
        )));
    }
    if request.name.trim().is_empty() {
        return Err(invalid_request_openai_response(
            "Video character name is required.",
            "invalid_video_character_request",
        ));
    }

    Ok(local_video_character_placeholder("char_1", &request.name))
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

    let response = match local_video_characters_list_result(&video_id) {
        Ok(response) => response,
        Err(response) => return response,
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

    let response = match local_video_character_retrieve_result(&video_id, &character_id) {
        Ok(response) => response,
        Err(response) => return response,
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

    let response = match local_video_character_update_result(&video_id, &character_id, &request) {
        Ok(response) => response,
        Err(response) => return response,
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

    let response = match local_video_character_create_result(&request) {
        Ok(response) => response,
        Err(response) => return response,
    };

    Json(response).into_response()
}
