fn local_response_compact_response(tenant_id: &str, project_id: &str, model: &str) -> Response {
    match compact_response(tenant_id, project_id, model) {
        Ok(response) => Json(response).into_response(),
        Err(error) => local_response_invalid_model_response(error),
    }
}

fn local_response_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_response_request",
        "Requested response was not found.",
    )
}

fn local_response_create_result(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> std::result::Result<ResponseObject, Response> {
    create_response(tenant_id, project_id, model).map_err(local_response_invalid_model_response)
}

fn local_response_create_response(tenant_id: &str, project_id: &str, model: &str) -> Response {
    match local_response_create_result(tenant_id, project_id, model) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_response_input_tokens_result(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> std::result::Result<ResponseInputTokensObject, Response> {
    count_response_input_tokens(tenant_id, project_id, model)
        .map_err(local_response_invalid_model_response)
}

fn local_response_input_tokens_response(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> Response {
    match local_response_input_tokens_result(tenant_id, project_id, model) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_response_invalid_model_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_bad_gateway_response(error, "invalid_model")
}

fn local_chat_completion_gateway_result(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> anyhow::Result<ChatCompletionResponse> {
    if model.trim().is_empty() {
        return Err(anyhow::anyhow!("Chat completion model is required."));
    }

    create_chat_completion(tenant_id, project_id, model)
}

fn local_chat_completion_result(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> std::result::Result<ChatCompletionResponse, Response> {
    local_chat_completion_gateway_result(tenant_id, project_id, model)
        .map_err(local_response_invalid_model_response)
}

fn local_chat_completion_response(tenant_id: &str, project_id: &str, model: &str) -> Response {
    match local_chat_completion_result(tenant_id, project_id, model) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_chat_completion_stream_response(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> Response {
    if model.trim().is_empty() {
        return local_chat_completion_response(tenant_id, project_id, model);
    }

    invalid_request_openai_response(
        "Local chat completion streaming fallback is not supported without an upstream provider.",
        "invalid_chat_completion_request",
    )
}

fn local_chat_completion_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_chat_completion_request",
        "Requested chat completion was not found.",
    )
}

fn local_response_stream_response(_tenant_id: &str, _project_id: &str, _model: &str) -> Response {
    invalid_request_openai_response(
        "Local response streaming fallback is not supported without an upstream provider.",
        "invalid_model",
    )
}
