use super::*;

fn local_conversation_item_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_conversation_request",
        "Requested conversation item was not found.",
    )
}

pub(crate) async fn conversation_items_handler(
    request_context: StatelessGatewayRequest,
    Path(conversation_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateConversationItemsRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ConversationItems(&conversation_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream conversation items");
        }
    }

    let response = match create_conversation_items(
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_conversation_item_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn conversation_items_list_handler(
    request_context: StatelessGatewayRequest,
    Path(conversation_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ConversationItemsList(&conversation_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream conversation items list");
        }
    }

    let response = match list_conversation_items(
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_conversation_item_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn conversation_item_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((conversation_id, item_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ConversationItemsRetrieve(&conversation_id, &item_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream conversation item retrieve",
            );
        }
    }

    let response = match get_conversation_item(
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
        &item_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_conversation_item_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn conversation_item_delete_handler(
    request_context: StatelessGatewayRequest,
    Path((conversation_id, item_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ConversationItemsDelete(&conversation_id, &item_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream conversation item delete",
            );
        }
    }

    let response = match delete_conversation_item(
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
        &item_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_conversation_item_error_response(error),
    };

    Json(response).into_response()
}
