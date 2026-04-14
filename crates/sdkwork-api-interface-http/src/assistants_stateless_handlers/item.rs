use super::*;

fn local_assistant_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_assistant_request",
        "Requested assistant was not found.",
    )
}

pub(crate) async fn assistant_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(assistant_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::AssistantsRetrieve(&assistant_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream assistant retrieve");
        }
    }

    let response = match get_assistant(
        request_context.tenant_id(),
        request_context.project_id(),
        &assistant_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_assistant_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn assistant_update_handler(
    request_context: StatelessGatewayRequest,
    Path(assistant_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateAssistantRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::AssistantsUpdate(&assistant_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream assistant update");
        }
    }

    let response = match update_assistant(
        request_context.tenant_id(),
        request_context.project_id(),
        &assistant_id,
        request.name.as_deref().unwrap_or("assistant"),
    ) {
        Ok(response) => response,
        Err(error) => return local_assistant_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn assistant_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(assistant_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::AssistantsDelete(&assistant_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream assistant delete");
        }
    }

    let response = match delete_assistant(
        request_context.tenant_id(),
        request_context.project_id(),
        &assistant_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_assistant_error_response(error),
    };

    Json(response).into_response()
}
