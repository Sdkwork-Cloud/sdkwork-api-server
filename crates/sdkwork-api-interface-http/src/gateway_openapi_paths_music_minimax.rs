use super::*;

#[utoipa::path(
        post,
        path = "/v1/music_generation",
        operation_id = "music_minimax_generation_create",
        tag = "music.minimax",
        request_body = Value,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Accepted MiniMax music generation request.", body = Value),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to relay the MiniMax music generation request.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn music_minimax_generation_create() {}

#[utoipa::path(
        post,
        path = "/v1/lyrics_generation",
        operation_id = "music_minimax_lyrics_create",
        tag = "music.minimax",
        request_body = Value,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Accepted MiniMax lyrics generation request.", body = Value),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to relay the MiniMax lyrics generation request.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn music_minimax_lyrics_create() {}
