use super::*;

#[utoipa::path(
        post,
        path = "/v1/video_generation",
        operation_id = "video_minimax_generation_create",
        tag = "video.minimax",
        request_body = Value,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Accepted MiniMax video generation request.", body = Value),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to relay the MiniMax video generation request.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn video_minimax_generation_create() {}

#[utoipa::path(
        get,
        path = "/v1/query/video_generation",
        operation_id = "video_minimax_generation_query",
        tag = "video.minimax",
        params(("task_id" = String, Query, description = "MiniMax video generation task identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible MiniMax video generation task state.", body = Value),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to relay the MiniMax video query request.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn video_minimax_generation_query() {}

#[utoipa::path(
        get,
        path = "/v1/files/retrieve",
        operation_id = "video_minimax_file_retrieve",
        tag = "video.minimax",
        params(("file_id" = String, Query, description = "MiniMax file identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible MiniMax generated file metadata.", body = Value),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to relay the MiniMax file retrieve request.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn video_minimax_file_retrieve() {}
