use super::*;
use sdkwork_api_contract_openai::videos::VideoCharacterObject;

fn local_video_character_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_error_response(error, "Requested video character was not found.")
}

fn character_missing(character_id: &str) -> bool {
    character_id.trim().is_empty() || character_id.ends_with("_missing")
}

fn local_video_character_retrieve_canonical_result(
    character_id: &str,
) -> std::result::Result<VideoCharacterObject, Response> {
    if character_missing(character_id) {
        return Err(local_video_character_not_found_response(anyhow::anyhow!(
            "video character not found"
        )));
    }

    Ok(VideoCharacterObject::new(character_id, "Hero"))
}

pub(crate) async fn video_character_retrieve_canonical_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(character_id): Path<String>,
) -> Response {
    match sdkwork_api_app_gateway::relay_get_video_character_canonical_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &character_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "videos",
                &character_id,
                20,
                0.02,
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

    let response = match local_video_character_retrieve_canonical_result(&character_id) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        &character_id,
        20,
        0.02,
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
