fn local_response_compact_response(tenant_id: &str, project_id: &str, model: &str) -> Response {
    match compact_response(tenant_id, project_id, model) {
        Ok(response) => Json(response).into_response(),
        Err(error) => local_response_invalid_model_response(error),
    }
}

fn local_response_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_error_response(error, "Requested response was not found.")
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
    let message = error.to_string();
    if local_gateway_error_is_invalid_request(&message) {
        return invalid_request_openai_response(message, "invalid_model");
    }

    bad_gateway_openai_response(message)
}

fn local_gateway_error_is_invalid_request(message: &str) -> bool {
    message.to_ascii_lowercase().contains("required")
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
    match local_chat_completion_result(tenant_id, project_id, model) {
        Ok(_) => local_chat_completion_stream_body_response(),
        Err(response) => response,
    }
}

fn local_chat_completion_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_error_response(error, "Requested chat completion was not found.")
}

fn local_response_stream_response(tenant_id: &str, project_id: &str, model: &str) -> Response {
    match local_response_create_result(tenant_id, project_id, model) {
        Ok(_) => local_response_stream_body_response("resp_1", model),
        Err(response) => response,
    }
}

fn local_response_stream_body_response(response_id: &str, model: &str) -> Response {
    let created = serde_json::json!({
        "type":"response.created",
        "response": {
            "id": response_id,
            "object": "response",
            "model": model
        }
    })
    .to_string();
    let delta = serde_json::json!({
        "type":"response.output_text.delta",
        "delta":"hello"
    })
    .to_string();
    let completed = serde_json::json!({
        "type":"response.completed",
        "response": {
            "id": response_id
        }
    })
    .to_string();
    let body = format!(
        "{}{}{}",
        SseFrame::data(&created),
        SseFrame::data(&delta),
        SseFrame::data(&completed)
    );
    ([(header::CONTENT_TYPE, "text/event-stream")], body).into_response()
}
