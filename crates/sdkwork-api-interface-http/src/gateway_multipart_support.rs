fn bad_multipart(error: axum::extract::multipart::MultipartError) -> Response {
    (
        axum::http::StatusCode::BAD_REQUEST,
        format!("invalid multipart payload: {error}"),
    )
        .into_response()
}

fn missing_multipart_field() -> Response {
    (
        axum::http::StatusCode::BAD_REQUEST,
        "missing multipart field",
    )
        .into_response()
}
