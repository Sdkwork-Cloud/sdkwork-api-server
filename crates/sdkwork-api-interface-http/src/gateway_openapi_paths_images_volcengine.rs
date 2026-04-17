use super::*;

#[utoipa::path(
        post,
        path = "/api/v3/images/generations",
        operation_id = "images_volcengine_generate_create",
        tag = "images.volcengine",
        request_body = Value,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Accepted Volcengine image generation request.", body = Value),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to relay the Volcengine image generation request.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn images_volcengine_generate_create() {}
