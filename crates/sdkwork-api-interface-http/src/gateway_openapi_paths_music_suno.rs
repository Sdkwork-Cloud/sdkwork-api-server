use super::*;

#[utoipa::path(
        post,
        path = "/api/v1/generate",
        operation_id = "music_suno_generate_create",
        tag = "music.suno",
        request_body = Value,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Accepted Suno music generation request.", body = Value),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to relay the Suno music generation request.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn music_suno_generate_create() {}

#[utoipa::path(
        get,
        path = "/api/v1/generate/record-info",
        operation_id = "music_suno_generate_record_info_get",
        tag = "music.suno",
        params(("taskId" = String, Query, description = "Suno task identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Suno music generation record details.", body = Value),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to relay the Suno record query.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn music_suno_generate_record_info_get() {}

#[utoipa::path(
        post,
        path = "/api/v1/lyrics",
        operation_id = "music_suno_lyrics_create",
        tag = "music.suno",
        request_body = Value,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Accepted Suno lyrics generation request.", body = Value),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to relay the Suno lyrics generation request.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn music_suno_lyrics_create() {}

#[utoipa::path(
        get,
        path = "/api/v1/lyrics/record-info",
        operation_id = "music_suno_lyrics_record_info_get",
        tag = "music.suno",
        params(("taskId" = String, Query, description = "Suno task identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Suno lyrics generation record details.", body = Value),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to relay the Suno lyrics record query.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn music_suno_lyrics_record_info_get() {}
