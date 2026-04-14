use super::*;

fn local_chat_completion_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_chat_completion_request",
        "Requested chat completion was not found.",
    )
}

pub(crate) async fn chat_completion_messages_list_handler(
    request_context: StatelessGatewayRequest,
    Path(completion_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ChatCompletionsMessagesList(&completion_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream chat completion messages",
            );
        }
    }

    let response = match list_chat_completion_messages(
        request_context.tenant_id(),
        request_context.project_id(),
        &completion_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_chat_completion_not_found_response(error),
    };

    Json(response).into_response()
}
