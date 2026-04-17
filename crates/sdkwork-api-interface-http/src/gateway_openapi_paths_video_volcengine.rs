use super::*;

#[utoipa::path(
        post,
        path = "/api/v1/contents/generations/tasks",
        operation_id = "video_volcengine_tasks_create",
        tag = "video.volcengine",
        request_body = Value,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Accepted Volcengine video generation task request.", body = Value),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to relay the Volcengine video task create request.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn video_volcengine_tasks_create() {}

#[utoipa::path(
        get,
        path = "/api/v1/contents/generations/tasks/{id}",
        operation_id = "video_volcengine_task_get",
        tag = "video.volcengine",
        params(("id" = String, Path, description = "Volcengine video generation task identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible Volcengine video generation task state.", body = Value),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to relay the Volcengine video task query request.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn video_volcengine_task_get() {}
