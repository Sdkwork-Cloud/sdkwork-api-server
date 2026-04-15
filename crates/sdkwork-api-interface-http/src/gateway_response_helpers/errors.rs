use super::*;

pub(crate) fn bad_gateway_openai_response(message: impl Into<String>) -> Response {
    let mut error = OpenAiErrorResponse::new(message, "server_error");
    error.error.code = Some("bad_gateway".to_owned());
    (StatusCode::BAD_GATEWAY, Json(error)).into_response()
}

pub(crate) fn invalid_request_openai_response(
    message: impl Into<String>,
    code: impl Into<String>,
) -> Response {
    let mut error = OpenAiErrorResponse::new(message, "invalid_request_error");
    error.error.code = Some(code.into());
    (StatusCode::BAD_REQUEST, Json(error)).into_response()
}

pub(crate) fn local_gateway_error_response(
    error: anyhow::Error,
    not_found_message: &'static str,
) -> Response {
    if error.to_string().to_ascii_lowercase().contains("not found") {
        let mut error = OpenAiErrorResponse::new(not_found_message, "invalid_request_error");
        error.error.code = Some("not_found".to_owned());
        return (StatusCode::NOT_FOUND, Json(error)).into_response();
    }

    bad_gateway_openai_response("failed to process local gateway fallback")
}

pub(crate) fn local_gateway_invalid_or_bad_gateway_response(
    error: anyhow::Error,
    invalid_code: &'static str,
) -> Response {
    let message = error.to_string();
    if local_gateway_error_is_invalid_request(&message) {
        return invalid_request_openai_response(message, invalid_code);
    }

    bad_gateway_openai_response("failed to process local gateway fallback")
}

pub(crate) fn local_gateway_invalid_or_not_found_response(
    error: anyhow::Error,
    invalid_code: &'static str,
    not_found_message: &'static str,
) -> Response {
    let message = error.to_string();
    if local_gateway_error_is_invalid_request(&message) {
        return invalid_request_openai_response(message, invalid_code);
    }

    local_gateway_error_response(error, not_found_message)
}

pub(crate) fn local_gateway_error_is_invalid_request(message: &str) -> bool {
    let message = message.to_ascii_lowercase();
    message.contains("required")
        || message.contains("unsupported")
        || message.contains("not supported")
}
