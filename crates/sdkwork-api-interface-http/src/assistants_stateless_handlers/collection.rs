use super::*;

fn local_assistant_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_assistant_request",
        "Requested assistant was not found.",
    )
}

pub(crate) async fn assistants_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateAssistantRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Assistants(&request))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream assistant");
        }
    }

    let response = match create_assistant(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.name,
        &request.model,
    ) {
        Ok(response) => response,
        Err(error) => return local_assistant_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn assistants_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::AssistantsList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream assistants list");
        }
    }

    let response = match list_assistants(request_context.tenant_id(), request_context.project_id())
    {
        Ok(response) => response,
        Err(error) => return local_assistant_error_response(error),
    };

    Json(response).into_response()
}
