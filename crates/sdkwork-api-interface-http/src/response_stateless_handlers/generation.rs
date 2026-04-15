use super::*;

fn local_response_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_bad_gateway_response(error, "invalid_model")
}

pub(crate) async fn responses_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateResponseRequest>,
) -> Response {
    if request.model.trim().is_empty() {
        return invalid_request_openai_response("Response model is required.", "invalid_model");
    }

    if request.stream.unwrap_or(false) {
        match relay_stateless_stream_request(
            &request_context,
            ProviderRequest::ResponsesStream(&request),
        )
        .await
        {
            Ok(Some(response)) => return upstream_passthrough_response(response),
            Ok(None) => {}
            Err(_) => {
                return bad_gateway_openai_response("failed to relay upstream response stream");
            }
        }

        return invalid_request_openai_response(
            "Local response streaming fallback is not supported without an upstream provider.",
            "invalid_model",
        );
    }

    match relay_stateless_json_request(&request_context, ProviderRequest::Responses(&request)).await
    {
        Ok(Some(response)) => Json(response).into_response(),
        Ok(None) => {
            let response = match create_response(
                request_context.tenant_id(),
                request_context.project_id(),
                &request.model,
            ) {
                Ok(response) => response,
                Err(error) => return local_response_error_response(error),
            };
            Json(response).into_response()
        }
        Err(_) => bad_gateway_openai_response("failed to relay upstream response"),
    }
}

pub(crate) async fn response_input_tokens_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CountResponseInputTokensRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ResponsesInputTokens(&request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream response input tokens");
        }
    }

    let response = match count_response_input_tokens(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
    ) {
        Ok(response) => response,
        Err(error) => return local_response_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn response_compact_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CompactResponseRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ResponsesCompact(&request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream response compact");
        }
    }

    let response = match compact_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
    ) {
        Ok(response) => response,
        Err(error) => return local_response_error_response(error),
    };

    Json(response).into_response()
}
