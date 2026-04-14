use super::*;
use std::borrow::Cow;

fn local_thread_message_error_response(error: anyhow::Error) -> Response {
    let message = error.to_string();
    if message
        .to_ascii_lowercase()
        .contains("thread message not found")
    {
        return local_gateway_invalid_or_not_found_response(
            error,
            "invalid_thread_request",
            "Requested thread message was not found.",
        );
    }

    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_thread_request",
        "Requested thread was not found.",
    )
}

fn local_thread_message_text<'a>(content: &'a Value) -> Result<Cow<'a, str>, Response> {
    if let Some(text) = content.as_str() {
        return Ok(Cow::Borrowed(text));
    }

    let Some(parts) = content.as_array() else {
        return Err(invalid_request_openai_response(
            "Thread message content must be a string or an array of text parts.",
            "invalid_thread_request",
        ));
    };

    let mut texts = Vec::with_capacity(parts.len());
    for part in parts {
        if part.get("type").and_then(Value::as_str) != Some("text") {
            return Err(invalid_request_openai_response(
                "Local thread fallback only supports text content parts.",
                "invalid_thread_request",
            ));
        }

        let text = part.get("text").and_then(Value::as_str).or_else(|| {
            part.get("text")
                .and_then(Value::as_object)
                .and_then(|text| text.get("value"))
                .and_then(Value::as_str)
        });
        let Some(text) = text else {
            return Err(invalid_request_openai_response(
                "Text content parts must include a text value.",
                "invalid_thread_request",
            ));
        };
        texts.push(text);
    }

    if texts.is_empty() {
        return Err(invalid_request_openai_response(
            "Thread message content must include at least one text part.",
            "invalid_thread_request",
        ));
    }

    Ok(Cow::Owned(texts.join("\n")))
}

pub(crate) async fn thread_messages_handler(
    request_context: StatelessGatewayRequest,
    Path(thread_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateThreadMessageRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadMessages(&thread_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread message");
        }
    }

    let text = match local_thread_message_text(&request.content) {
        Ok(text) => text,
        Err(response) => return response,
    };
    let response = match create_thread_message(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &request.role,
        text.as_ref(),
    ) {
        Ok(response) => response,
        Err(error) => return local_thread_message_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn thread_messages_list_handler(
    request_context: StatelessGatewayRequest,
    Path(thread_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadMessagesList(&thread_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread messages list");
        }
    }

    let response = match list_thread_messages(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_thread_message_error_response(error),
    };

    Json(response).into_response()
}
