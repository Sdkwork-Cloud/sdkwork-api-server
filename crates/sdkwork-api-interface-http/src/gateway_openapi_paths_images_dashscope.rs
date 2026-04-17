use super::*;

#[utoipa::path(
        post,
        path = "/api/v1/services/aigc/image-generation/generation",
        operation_id = "images_dashscope_generation_create",
        tag = "images.kling",
        tags = ["images.aliyun"],
        request_body = Value,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Accepted shared DashScope image generation request.", body = Value),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to relay the shared DashScope image generation request.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn images_dashscope_generation_create() {}

#[utoipa::path(
        get,
        path = "/api/v1/tasks/{task_id}",
        operation_id = "dashscope_task_get",
        tag = "images.kling",
        tags = ["images.aliyun", "video.kling", "video.aliyun"],
        params(("task_id" = String, Path, description = "DashScope image task identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Shared DashScope media task details.", body = Value),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to relay the shared DashScope image task query.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn images_dashscope_task_get() {}
