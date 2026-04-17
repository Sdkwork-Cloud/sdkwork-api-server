use super::*;

#[utoipa::path(
        post,
        path = "/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:predict",
        operation_id = "music_google_predict_create",
        tag = "music.google",
        params(
            ("project" = String, Path, description = "Google Cloud project identifier."),
            ("location" = String, Path, description = "Vertex AI location."),
            ("model" = String, Path, description = "Google music model such as `lyria-002`.")
        ),
        request_body = Value,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Accepted Google music official predict request.", body = Value),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to relay the Google music predict request.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn music_google_predict_create() {}
