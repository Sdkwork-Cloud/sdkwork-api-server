use super::*;

fn local_anthropic_error_response(error: anyhow::Error) -> Response {
    let message = error.to_string();
    if local_gateway_error_is_invalid_request(&message) {
        return anthropic_invalid_request_response(message);
    }

    anthropic_bad_gateway_response("failed to process local anthropic fallback")
}

fn local_anthropic_encoding_error_response() -> Response {
    anthropic_bad_gateway_response("failed to encode local anthropic response")
}

pub(crate) async fn anthropic_messages_handler(
    request_context: StatelessGatewayRequest,
    headers: HeaderMap,
    ExtractJson(payload): ExtractJson<Value>,
) -> Response {
    let request = match anthropic_request_to_chat_completion(&payload) {
        Ok(request) => request,
        Err(error) => return anthropic_invalid_request_response(error.to_string()),
    };
    let options = anthropic_request_options(&headers);

    if request.stream.unwrap_or(false) {
        match relay_stateless_stream_request_with_options(
            &request_context,
            ProviderRequest::ChatCompletionsStream(&request),
            &options,
        )
        .await
        {
            Ok(Some(response)) => {
                return upstream_passthrough_response(anthropic_stream_from_openai(response));
            }
            Ok(None) => {
                return anthropic_invalid_request_response(
                    "Local anthropic message streaming fallback is not supported without an upstream provider.",
                );
            }
            Err(_) => {
                return anthropic_bad_gateway_response(
                    "failed to relay upstream anthropic message stream",
                );
            }
        }
    }

    match relay_stateless_json_request_with_options(
        &request_context,
        ProviderRequest::ChatCompletions(&request),
        &options,
    )
    .await
    {
        Ok(Some(response)) => Json(openai_chat_response_to_anthropic(&response)).into_response(),
        Ok(None) => {
            let response = match create_chat_completion(
                request_context.tenant_id(),
                request_context.project_id(),
                &request.model,
            ) {
                Ok(response) => response,
                Err(error) => return local_anthropic_error_response(error),
            };
            let response_value = match serde_json::to_value(response) {
                Ok(value) => value,
                Err(_) => return local_anthropic_encoding_error_response(),
            };
            Json(openai_chat_response_to_anthropic(&response_value)).into_response()
        }
        Err(_) => anthropic_bad_gateway_response("failed to relay upstream anthropic message"),
    }
}

pub(crate) async fn anthropic_count_tokens_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(payload): ExtractJson<Value>,
) -> Response {
    let request = match anthropic_count_tokens_request(&payload) {
        Ok(request) => request,
        Err(error) => return anthropic_invalid_request_response(error.to_string()),
    };

    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ResponsesInputTokens(&request),
    )
    .await
    {
        Ok(Some(response)) => Json(openai_count_tokens_to_anthropic(&response)).into_response(),
        Ok(None) => {
            let response = match count_response_input_tokens(
                request_context.tenant_id(),
                request_context.project_id(),
                &request.model,
            ) {
                Ok(response) => response,
                Err(error) => return local_anthropic_error_response(error),
            };
            let response_value = match serde_json::to_value(response) {
                Ok(value) => value,
                Err(_) => return local_anthropic_encoding_error_response(),
            };
            Json(openai_count_tokens_to_anthropic(&response_value)).into_response()
        }
        Err(_) => anthropic_bad_gateway_response(
            "failed to relay upstream anthropic count tokens request",
        ),
    }
}
