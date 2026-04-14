use super::shared::{parse_gemini_compat_tail, GeminiCompatAction};
use super::*;

fn local_gemini_error_response(error: anyhow::Error) -> Response {
    let message = error.to_string();
    if local_gateway_error_is_invalid_request(&message) {
        return gemini_invalid_request_response(message);
    }

    gemini_bad_gateway_response("failed to process local gemini fallback")
}

fn local_gemini_encoding_error_response() -> Response {
    gemini_bad_gateway_response("failed to encode local gemini response")
}

pub(crate) async fn gemini_models_compat_handler(
    request_context: StatelessGatewayRequest,
    Path(tail): Path<String>,
    ExtractJson(payload): ExtractJson<Value>,
) -> Response {
    let Some((model, action)) = parse_gemini_compat_tail(&tail) else {
        return gemini_invalid_request_response("unsupported gemini compatibility route");
    };

    match action {
        GeminiCompatAction::GenerateContent => {
            let request = match gemini_request_to_chat_completion(&model, &payload) {
                Ok(request) => request,
                Err(error) => return gemini_invalid_request_response(error.to_string()),
            };

            match relay_stateless_json_request(
                &request_context,
                ProviderRequest::ChatCompletions(&request),
            )
            .await
            {
                Ok(Some(response)) => {
                    Json(openai_chat_response_to_gemini(&response)).into_response()
                }
                Ok(None) => {
                    let response = match create_chat_completion(
                        request_context.tenant_id(),
                        request_context.project_id(),
                        &request.model,
                    ) {
                        Ok(response) => response,
                        Err(error) => return local_gemini_error_response(error),
                    };
                    let response_value = match serde_json::to_value(response) {
                        Ok(value) => value,
                        Err(_) => return local_gemini_encoding_error_response(),
                    };
                    Json(openai_chat_response_to_gemini(&response_value)).into_response()
                }
                Err(_) => gemini_bad_gateway_response(
                    "failed to relay upstream gemini generateContent request",
                ),
            }
        }
        GeminiCompatAction::StreamGenerateContent => {
            let mut request = match gemini_request_to_chat_completion(&model, &payload) {
                Ok(request) => request,
                Err(error) => return gemini_invalid_request_response(error.to_string()),
            };
            request.stream = Some(true);

            match relay_stateless_stream_request(
                &request_context,
                ProviderRequest::ChatCompletionsStream(&request),
            )
            .await
            {
                Ok(Some(response)) => {
                    upstream_passthrough_response(gemini_stream_from_openai(response))
                }
                Ok(None) => gemini_invalid_request_response(
                    "Local Gemini streamGenerateContent fallback is not supported without an upstream provider.",
                ),
                Err(_) => gemini_bad_gateway_response(
                    "failed to relay upstream gemini streamGenerateContent request",
                ),
            }
        }
        GeminiCompatAction::CountTokens => {
            let request = gemini_count_tokens_request(&model, &payload);
            match relay_stateless_json_request(
                &request_context,
                ProviderRequest::ResponsesInputTokens(&request),
            )
            .await
            {
                Ok(Some(response)) => {
                    Json(openai_count_tokens_to_gemini(&response)).into_response()
                }
                Ok(None) => {
                    let response = match count_response_input_tokens(
                        request_context.tenant_id(),
                        request_context.project_id(),
                        &request.model,
                    ) {
                        Ok(response) => response,
                        Err(error) => return local_gemini_error_response(error),
                    };
                    let response_value = match serde_json::to_value(response) {
                        Ok(value) => value,
                        Err(_) => return local_gemini_encoding_error_response(),
                    };
                    Json(openai_count_tokens_to_gemini(&response_value)).into_response()
                }
                Err(_) => gemini_bad_gateway_response(
                    "failed to relay upstream gemini countTokens request",
                ),
            }
        }
    }
}
