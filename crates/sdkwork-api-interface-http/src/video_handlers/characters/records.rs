use super::*;
use sdkwork_api_contract_openai::videos::{VideoCharacterObject, VideoCharactersResponse};

fn local_video_character_error_response(error: anyhow::Error) -> Response {
    let message = error.to_string();
    if local_gateway_error_is_invalid_request(&message) {
        return invalid_request_openai_response(message, "invalid_video_request");
    }

    local_gateway_error_response(error, "Requested video character was not found.")
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
            "invalid_video_request",
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
            "invalid_video_request",
        ));
    }

    Ok(local_video_character_placeholder("char_1", &request.name))
}

pub(crate) async fn video_characters_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(video_id): Path<String>,
) -> Response {
    match relay_list_video_characters_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "videos",
                &video_id,
                60,
                0.06,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video characters list");
        }
    }

    let response = match local_video_characters_list_result(&video_id) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        &video_id,
        60,
        0.06,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

pub(crate) async fn video_character_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((video_id, character_id)): Path<(String, String)>,
) -> Response {
    match relay_get_video_character_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
        &character_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "videos",
                &video_id,
                &character_id,
                60,
                0.06,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
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

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        &video_id,
        &character_id,
        60,
        0.06,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

pub(crate) async fn video_character_update_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((video_id, character_id)): Path<(String, String)>,
    ExtractJson(request): ExtractJson<UpdateVideoCharacterRequest>,
) -> Response {
    match relay_update_video_character_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
        &character_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "videos",
                &video_id,
                &character_id,
                60,
                0.06,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video character update");
        }
    }

    let response = match local_video_character_update_result(&video_id, &character_id, &request) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        &video_id,
        &character_id,
        60,
        0.06,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

pub(crate) async fn video_character_create_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateVideoCharacterRequest>,
) -> Response {
    match sdkwork_api_app_gateway::relay_create_video_character_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let Some(character_id) = response.get("id").and_then(Value::as_str) else {
                return bad_gateway_openai_response("upstream video character response missing id");
            };
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "videos",
                &request.video_id,
                character_id,
                40,
                0.04,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video character create");
        }
    }

    let response = match local_video_character_create_result(&request) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        &request.video_id,
        response.id.as_str(),
        40,
        0.04,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}
