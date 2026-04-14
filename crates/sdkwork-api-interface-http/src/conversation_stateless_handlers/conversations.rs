use super::*;

fn local_conversation_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_conversation_request",
        "Requested conversation was not found.",
    )
}

pub(crate) async fn conversations_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateConversationRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Conversations(&request))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream conversation");
        }
    }

    let response =
        match create_conversation(request_context.tenant_id(), request_context.project_id()) {
            Ok(response) => response,
            Err(error) => return local_conversation_error_response(error),
        };

    Json(response).into_response()
}

pub(crate) async fn conversations_list_handler(
    request_context: StatelessGatewayRequest,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::ConversationsList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream conversation list");
        }
    }

    let response =
        match list_conversations(request_context.tenant_id(), request_context.project_id()) {
            Ok(response) => response,
            Err(error) => return local_conversation_error_response(error),
        };

    Json(response).into_response()
}

pub(crate) async fn conversation_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(conversation_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ConversationsRetrieve(&conversation_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream conversation retrieve");
        }
    }

    let response = match get_conversation(
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_conversation_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn conversation_update_handler(
    request_context: StatelessGatewayRequest,
    Path(conversation_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateConversationRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ConversationsUpdate(&conversation_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream conversation update");
        }
    }

    let response = match update_conversation(
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
        request.metadata,
    ) {
        Ok(response) => response,
        Err(error) => return local_conversation_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn conversation_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(conversation_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ConversationsDelete(&conversation_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream conversation delete");
        }
    }

    let response = match delete_conversation(
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_conversation_error_response(error),
    };

    Json(response).into_response()
}
