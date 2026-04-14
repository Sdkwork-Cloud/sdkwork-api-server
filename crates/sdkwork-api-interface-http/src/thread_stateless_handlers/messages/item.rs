use super::*;

fn local_thread_message_error_response(error: anyhow::Error) -> Response {
    let message = error.to_string();
    if local_gateway_error_is_invalid_request(&message) {
        return invalid_request_openai_response(message, "invalid_thread_request");
    }

    if message
        .to_ascii_lowercase()
        .contains("thread message not found")
    {
        return local_gateway_error_response(error, "Requested thread message was not found.");
    }

    local_gateway_error_response(error, "Requested thread was not found.")
}

pub(crate) async fn thread_message_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((thread_id, message_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadMessagesRetrieve(&thread_id, &message_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread message retrieve");
        }
    }

    let response = match get_thread_message(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &message_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_thread_message_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn thread_message_update_handler(
    request_context: StatelessGatewayRequest,
    Path((thread_id, message_id)): Path<(String, String)>,
    ExtractJson(request): ExtractJson<UpdateThreadMessageRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadMessagesUpdate(&thread_id, &message_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread message update");
        }
    }

    let response = match update_thread_message(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &message_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_thread_message_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn thread_message_delete_handler(
    request_context: StatelessGatewayRequest,
    Path((thread_id, message_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadMessagesDelete(&thread_id, &message_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread message delete");
        }
    }

    let response = match delete_thread_message(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &message_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_thread_message_error_response(error),
    };

    Json(response).into_response()
}
