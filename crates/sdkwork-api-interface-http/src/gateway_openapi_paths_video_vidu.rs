use super::*;

#[utoipa::path(
        post,
        path = "/ent/v2/text2video",
        operation_id = "video_vidu_text2video_create",
        tag = "video.vidu",
        request_body = Value,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Accepted Vidu text-to-video request.", body = Value),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to relay the Vidu text-to-video request.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn video_vidu_text2video_create() {}

#[utoipa::path(
        post,
        path = "/ent/v2/img2video",
        operation_id = "video_vidu_img2video_create",
        tag = "video.vidu",
        request_body = Value,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Accepted Vidu image-to-video request.", body = Value),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to relay the Vidu image-to-video request.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn video_vidu_img2video_create() {}

#[utoipa::path(
        post,
        path = "/ent/v2/reference2video",
        operation_id = "video_vidu_reference2video_create",
        tag = "video.vidu",
        request_body = Value,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Accepted Vidu reference-to-video request.", body = Value),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to relay the Vidu reference-to-video request.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn video_vidu_reference2video_create() {}

#[utoipa::path(
        get,
        path = "/ent/v2/tasks/{id}/creations",
        operation_id = "video_vidu_task_creations_get",
        tag = "video.vidu",
        params(("id" = String, Path, description = "Vidu task identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible Vidu task creation state.", body = Value),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to relay the Vidu task creations request.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn video_vidu_task_creations_get() {}

#[utoipa::path(
        post,
        path = "/ent/v2/tasks/{id}/cancel",
        operation_id = "video_vidu_task_cancel_create",
        tag = "video.vidu",
        params(("id" = String, Path, description = "Vidu task identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Accepted Vidu task cancel request.", body = Value),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to relay the Vidu task cancel request.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn video_vidu_task_cancel_create() {}
