use super::*;

#[utoipa::path(
        post,
        path = "/api/v1/services/aigc/video-generation/video-synthesis",
        operation_id = "video_dashscope_synthesis_create",
        tag = "video.kling",
        tags = ["video.aliyun"],
        request_body = Value,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Accepted shared DashScope video synthesis request.", body = Value),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to relay the shared DashScope video synthesis request.", body = OpenAiErrorResponse)
        )
    )]
pub(crate) async fn video_dashscope_synthesis_create() {}
