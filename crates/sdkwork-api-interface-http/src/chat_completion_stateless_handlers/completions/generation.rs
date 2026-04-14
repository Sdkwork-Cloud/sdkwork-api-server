use super::*;

fn local_chat_completion_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_bad_gateway_response(error, "invalid_model")
}

pub(crate) async fn chat_completions_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateChatCompletionRequest>,
) -> Response {
    if request.stream.unwrap_or(false) {
        match relay_stateless_stream_request(
            &request_context,
            ProviderRequest::ChatCompletionsStream(&request),
        )
        .await
        {
            Ok(Some(response)) => return upstream_passthrough_response(response),
            Ok(None) => {}
            Err(_) => {
                return bad_gateway_openai_response(
                    "failed to relay upstream chat completion stream",
                );
            }
        }

        return invalid_request_openai_response(
            "Local chat completion streaming fallback is not supported without an upstream provider.",
            "invalid_chat_completion_request",
        );
    }

    match relay_stateless_json_request(&request_context, ProviderRequest::ChatCompletions(&request))
        .await
    {
        Ok(Some(response)) => Json(response).into_response(),
        Ok(None) => {
            let response = match create_chat_completion(
                request_context.tenant_id(),
                request_context.project_id(),
                &request.model,
            ) {
                Ok(response) => response,
                Err(error) => return local_chat_completion_error_response(error),
            };

            Json(response).into_response()
        }
        Err(_) => bad_gateway_openai_response("failed to relay upstream chat completion"),
    }
}
