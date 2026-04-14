use super::*;

    #[utoipa::path(
        get,
        path = "/health",
        tag = "system",
        responses((status = 200, description = "Gateway health check."))
    )]
    pub(crate) async fn health() {}

    #[utoipa::path(
        get,
        path = "/v1/models",
        tag = "models",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible model catalog.", body = ListModelsResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load model catalog.", body = OpenAiErrorResponse)
        )
    )]
    pub(crate) async fn list_models() {}

    #[utoipa::path(
        get,
        path = "/v1/models/{model_id}",
        tag = "models",
        params(("model_id" = String, Path, description = "Model identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible model metadata.", body = sdkwork_api_contract_openai::models::ModelObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested model was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load model metadata.", body = OpenAiErrorResponse)
        )
    )]
    pub(crate) async fn get_model() {}

    #[utoipa::path(
        post,
        path = "/v1/chat/completions",
        tag = "chat",
        request_body = CreateChatCompletionRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Chat completion response.", body = ChatCompletionResponse),
            (status = 400, description = "Invalid completion payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the chat completion.", body = OpenAiErrorResponse)
        )
    )]
    pub(crate) async fn chat_completions() {}

    #[utoipa::path(
        post,
        path = "/v1/completions",
        tag = "completions",
        request_body = CreateCompletionRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Text completion response.", body = CompletionObject),
            (status = 400, description = "Invalid completion payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the completion.", body = OpenAiErrorResponse)
        )
    )]
    pub(crate) async fn completions() {}

    #[utoipa::path(
        post,
        path = "/v1/responses",
        tag = "responses",
        request_body = CreateResponseRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Response generation result.", body = ResponseObject),
            (status = 400, description = "Invalid response payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the response.", body = OpenAiErrorResponse)
        )
    )]
    pub(crate) async fn responses() {}

    #[utoipa::path(
        post,
        path = "/v1/responses/input_tokens",
        tag = "responses",
        request_body = CountResponseInputTokensRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Response input token count result.", body = ResponseInputTokensObject),
            (status = 400, description = "Invalid response token count payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to count response input tokens.", body = OpenAiErrorResponse)
        )
    )]
    pub(crate) async fn responses_input_tokens() {}

    #[utoipa::path(
        post,
        path = "/v1/responses/compact",
        tag = "responses",
        request_body = CompactResponseRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Response compaction result.", body = ResponseCompactionObject),
            (status = 400, description = "Invalid response compaction payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to compact the response.", body = OpenAiErrorResponse)
        )
    )]
    pub(crate) async fn responses_compact() {}

    #[utoipa::path(
        get,
        path = "/v1/responses/{response_id}",
        tag = "responses",
        params(("response_id" = String, Path, description = "Response identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible response.", body = ResponseObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested response was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the response.", body = OpenAiErrorResponse)
        )
    )]
    pub(crate) async fn response_get() {}

    #[utoipa::path(
        delete,
        path = "/v1/responses/{response_id}",
        tag = "responses",
        params(("response_id" = String, Path, description = "Response identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Deleted response.", body = DeleteResponseResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested response was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to delete the response.", body = OpenAiErrorResponse)
        )
    )]
    pub(crate) async fn response_delete() {}

    #[utoipa::path(
        get,
        path = "/v1/responses/{response_id}/input_items",
        tag = "responses",
        params(("response_id" = String, Path, description = "Response identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible response input items.", body = ListResponseInputItemsResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested response was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load response input items.", body = OpenAiErrorResponse)
        )
    )]
    pub(crate) async fn response_input_items() {}

    #[utoipa::path(
        post,
        path = "/v1/responses/{response_id}/cancel",
        tag = "responses",
        params(("response_id" = String, Path, description = "Response identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Cancelled response.", body = ResponseObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested response was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to cancel the response.", body = OpenAiErrorResponse)
        )
    )]
    pub(crate) async fn response_cancel() {}

    #[utoipa::path(
        post,
        path = "/v1/embeddings",
        tag = "embeddings",
        request_body = CreateEmbeddingRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Embedding generation result.", body = CreateEmbeddingResponse),
            (status = 400, description = "Invalid embedding payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create embeddings.", body = OpenAiErrorResponse)
        )
    )]
    pub(crate) async fn embeddings() {}

    #[utoipa::path(
        post,
        path = "/v1/moderations",
        tag = "moderations",
        request_body = CreateModerationRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Moderation result.", body = ModerationResponse),
            (status = 400, description = "Invalid moderation payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the moderation.", body = OpenAiErrorResponse)
        )
    )]
    pub(crate) async fn moderations() {}

