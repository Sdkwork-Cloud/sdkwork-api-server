use super::*;

fn local_video_character_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_video_character_request",
        "Requested video character was not found.",
    )
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

    let response = match sdkwork_api_app_gateway::get_video_character_canonical(
        request_context.tenant_id(),
        request_context.project_id(),
        &character_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_video_character_error_response(error),
    };

    Json(response).into_response()
}
