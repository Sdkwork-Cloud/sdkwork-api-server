use super::*;

fn upstream_stream_response_build_error_response() -> Response {
    bad_gateway_openai_response("failed to build upstream stream response")
}

pub(crate) fn upstream_passthrough_response(response: ProviderStreamOutput) -> Response {
    let content_type = response.content_type().to_owned();
    match Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .body(Body::from_stream(response.into_body_stream()))
    {
        Ok(response) => response,
        Err(_) => upstream_stream_response_build_error_response(),
    }
}
