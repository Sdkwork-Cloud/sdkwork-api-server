use super::*;
use sdkwork_api_contract_openai::videos::VideoCharacterObject;

fn local_video_character_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_video_character_request",
        "Requested video character was not found.",
    )
}

fn character_missing(character_id: &str) -> bool {
    character_id.trim().is_empty() || character_id.ends_with("_missing")
}

fn local_video_character_retrieve_canonical_result(
    character_id: &str,
) -> std::result::Result<VideoCharacterObject, Response> {
    if character_missing(character_id) {
        return Err(local_video_character_error_response(anyhow::anyhow!(
            "video character not found"
        )));
    }

    Ok(VideoCharacterObject::new(character_id, "Hero"))
}

pub(crate) async fn video_character_retrieve_canonical_handler(
    request_context: StatelessGatewayRequest,
    Path(character_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VideoCharactersCanonicalRetrieve(&character_id),
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

    let response = match local_video_character_retrieve_canonical_result(&character_id) {
        Ok(response) => response,
        Err(response) => return response,
    };

    Json(response).into_response()
}
