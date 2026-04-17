use super::*;

#[utoipa::path(
        post,
        path = "/v1/projects/{project}/locations/{location}/publishers/google/models/{tail}",
        operation_id = "video_google_veo_models_action_create",
        tag = "video.google-veo",
        params(
            ("project" = String, Path, description = "Google Cloud project identifier."),
            ("location" = String, Path, description = "Vertex AI location."),
            ("tail" = String, Path, description = "Model tail such as `veo-3.0-generate-001:predictLongRunning` or `veo-3.0-generate-001:fetchPredictOperation`.")
        ),
        request_body = Value,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Accepted Google Veo official models action request.", body = Value),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to relay the Google Veo models action request.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn video_google_veo_models_action_create() {}
