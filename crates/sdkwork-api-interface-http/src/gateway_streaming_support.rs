fn upstream_passthrough_response(response: ProviderStreamOutput) -> Response {
    let content_type = response.content_type().to_owned();
    match Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .body(Body::from_stream(response.into_body_stream()))
    {
        Ok(response) => response,
        Err(_) => bad_gateway_openai_response("failed to process upstream stream response"),
    }
}

