use super::*;

    #[utoipa::path(
        get,
        path = "/v1/assistants",
        tag = "assistants",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible assistants.", body = ListAssistantsResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load assistants.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn assistants_list() {}

    #[utoipa::path(
        post,
        path = "/v1/assistants",
        tag = "assistants",
        request_body = CreateAssistantRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created assistant.", body = AssistantObject),
            (status = 400, description = "Invalid assistant payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the assistant.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn assistants_create() {}

    #[utoipa::path(
        get,
        path = "/v1/assistants/{assistant_id}",
        tag = "assistants",
        params(("assistant_id" = String, Path, description = "Assistant identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible assistant metadata.", body = AssistantObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested assistant was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load assistant metadata.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn assistants_get() {}

    #[utoipa::path(
        get,
        path = "/v1/conversations",
        tag = "conversations",
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible conversations.", body = ListConversationsResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load conversations.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn conversations_list() {}

    #[utoipa::path(
        post,
        path = "/v1/conversations",
        tag = "conversations",
        request_body = CreateConversationRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created conversation.", body = ConversationObject),
            (status = 400, description = "Invalid conversation payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the conversation.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn conversations_create() {}

    #[utoipa::path(
        get,
        path = "/v1/conversations/{conversation_id}",
        tag = "conversations",
        params(("conversation_id" = String, Path, description = "Conversation identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible conversation.", body = ConversationObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested conversation was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the conversation.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn conversation_get() {}

    #[utoipa::path(
        post,
        path = "/v1/conversations/{conversation_id}",
        tag = "conversations",
        params(("conversation_id" = String, Path, description = "Conversation identifier.")),
        request_body = UpdateConversationRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Updated conversation.", body = ConversationObject),
            (status = 400, description = "Invalid conversation update payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested conversation was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to update the conversation.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn conversation_update() {}

    #[utoipa::path(
        delete,
        path = "/v1/conversations/{conversation_id}",
        tag = "conversations",
        params(("conversation_id" = String, Path, description = "Conversation identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Deleted conversation.", body = DeleteConversationResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested conversation was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to delete the conversation.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn conversation_delete() {}

    #[utoipa::path(
        get,
        path = "/v1/conversations/{conversation_id}/items",
        tag = "conversations",
        params(("conversation_id" = String, Path, description = "Conversation identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible conversation items.", body = ListConversationItemsResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested conversation was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load conversation items.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn conversation_items_list() {}

    #[utoipa::path(
        post,
        path = "/v1/conversations/{conversation_id}/items",
        tag = "conversations",
        params(("conversation_id" = String, Path, description = "Conversation identifier.")),
        request_body = CreateConversationItemsRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created conversation items.", body = ListConversationItemsResponse),
            (status = 400, description = "Invalid conversation item payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested conversation was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create conversation items.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn conversation_items_create() {}

    #[utoipa::path(
        get,
        path = "/v1/conversations/{conversation_id}/items/{item_id}",
        tag = "conversations",
        params(
            ("conversation_id" = String, Path, description = "Conversation identifier."),
            ("item_id" = String, Path, description = "Conversation item identifier.")
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible conversation item.", body = ConversationItemObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested conversation item was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the conversation item.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn conversation_item_get() {}

    #[utoipa::path(
        delete,
        path = "/v1/conversations/{conversation_id}/items/{item_id}",
        tag = "conversations",
        params(
            ("conversation_id" = String, Path, description = "Conversation identifier."),
            ("item_id" = String, Path, description = "Conversation item identifier.")
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Deleted conversation item.", body = DeleteConversationItemResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested conversation item was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to delete the conversation item.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn conversation_item_delete() {}

    #[utoipa::path(
        post,
        path = "/v1/threads",
        tag = "threads",
        request_body = CreateThreadRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created thread.", body = ThreadObject),
            (status = 400, description = "Invalid thread payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the thread.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn threads_create() {}

    #[utoipa::path(
        get,
        path = "/v1/threads/{thread_id}",
        tag = "threads",
        params(("thread_id" = String, Path, description = "Thread identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible thread metadata.", body = ThreadObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested thread was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the thread.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn thread_get() {}

    #[utoipa::path(
        post,
        path = "/v1/threads/{thread_id}",
        tag = "threads",
        params(("thread_id" = String, Path, description = "Thread identifier.")),
        request_body = UpdateThreadRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Updated thread.", body = ThreadObject),
            (status = 400, description = "Invalid thread update payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested thread was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to update the thread.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn thread_update() {}

    #[utoipa::path(
        delete,
        path = "/v1/threads/{thread_id}",
        tag = "threads",
        params(("thread_id" = String, Path, description = "Thread identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Deleted thread.", body = DeleteThreadResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested thread was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to delete the thread.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn thread_delete() {}

    #[utoipa::path(
        get,
        path = "/v1/threads/{thread_id}/messages",
        tag = "threads",
        params(("thread_id" = String, Path, description = "Thread identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible thread messages.", body = ListThreadMessagesResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested thread was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load thread messages.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn thread_messages_list() {}

    #[utoipa::path(
        post,
        path = "/v1/threads/{thread_id}/messages",
        tag = "threads",
        params(("thread_id" = String, Path, description = "Thread identifier.")),
        request_body = CreateThreadMessageRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created thread message.", body = ThreadMessageObject),
            (status = 400, description = "Invalid thread message payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested thread was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the thread message.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn thread_messages_create() {}

    #[utoipa::path(
        get,
        path = "/v1/threads/{thread_id}/messages/{message_id}",
        tag = "threads",
        params(
            ("thread_id" = String, Path, description = "Thread identifier."),
            ("message_id" = String, Path, description = "Thread message identifier.")
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible thread message metadata.", body = ThreadMessageObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested thread message was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the thread message.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn thread_message_get() {}

    #[utoipa::path(
        post,
        path = "/v1/threads/{thread_id}/messages/{message_id}",
        tag = "threads",
        params(
            ("thread_id" = String, Path, description = "Thread identifier."),
            ("message_id" = String, Path, description = "Thread message identifier.")
        ),
        request_body = UpdateThreadMessageRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Updated thread message.", body = ThreadMessageObject),
            (status = 400, description = "Invalid thread message update payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested thread message was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to update the thread message.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn thread_message_update() {}

    #[utoipa::path(
        delete,
        path = "/v1/threads/{thread_id}/messages/{message_id}",
        tag = "threads",
        params(
            ("thread_id" = String, Path, description = "Thread identifier."),
            ("message_id" = String, Path, description = "Thread message identifier.")
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Deleted thread message.", body = DeleteThreadMessageResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested thread message was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to delete the thread message.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn thread_message_delete() {}

    #[utoipa::path(
        post,
        path = "/v1/threads/runs",
        tag = "runs",
        request_body = CreateThreadAndRunRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created thread and run.", body = RunObject),
            (status = 400, description = "Invalid thread-and-run payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the thread and run.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn thread_and_run_create() {}

    #[utoipa::path(
        get,
        path = "/v1/threads/{thread_id}/runs",
        tag = "runs",
        params(("thread_id" = String, Path, description = "Thread identifier.")),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible thread runs.", body = ListRunsResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested thread was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load thread runs.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn thread_runs_list() {}

    #[utoipa::path(
        post,
        path = "/v1/threads/{thread_id}/runs",
        tag = "runs",
        params(("thread_id" = String, Path, description = "Thread identifier.")),
        request_body = CreateRunRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Created thread run.", body = RunObject),
            (status = 400, description = "Invalid thread run payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested thread was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the thread run.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn thread_runs_create() {}

    #[utoipa::path(
        get,
        path = "/v1/threads/{thread_id}/runs/{run_id}",
        tag = "runs",
        params(
            ("thread_id" = String, Path, description = "Thread identifier."),
            ("run_id" = String, Path, description = "Run identifier.")
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible run metadata.", body = RunObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested run was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the run.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn thread_run_get() {}

    #[utoipa::path(
        post,
        path = "/v1/threads/{thread_id}/runs/{run_id}",
        tag = "runs",
        params(
            ("thread_id" = String, Path, description = "Thread identifier."),
            ("run_id" = String, Path, description = "Run identifier.")
        ),
        request_body = UpdateRunRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Updated run.", body = RunObject),
            (status = 400, description = "Invalid run update payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested run was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to update the run.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn thread_run_update() {}

    #[utoipa::path(
        post,
        path = "/v1/threads/{thread_id}/runs/{run_id}/cancel",
        tag = "runs",
        params(
            ("thread_id" = String, Path, description = "Thread identifier."),
            ("run_id" = String, Path, description = "Run identifier.")
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Cancelled run.", body = RunObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested run was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to cancel the run.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn thread_run_cancel() {}

    #[utoipa::path(
        post,
        path = "/v1/threads/{thread_id}/runs/{run_id}/submit_tool_outputs",
        tag = "runs",
        params(
            ("thread_id" = String, Path, description = "Thread identifier."),
            ("run_id" = String, Path, description = "Run identifier.")
        ),
        request_body = SubmitToolOutputsRunRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Run after tool outputs submission.", body = RunObject),
            (status = 400, description = "Invalid tool outputs payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested run was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to submit tool outputs to the run.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn thread_run_submit_tool_outputs() {}

    #[utoipa::path(
        get,
        path = "/v1/threads/{thread_id}/runs/{run_id}/steps",
        tag = "runs",
        params(
            ("thread_id" = String, Path, description = "Thread identifier."),
            ("run_id" = String, Path, description = "Run identifier.")
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible run steps.", body = ListRunStepsResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested run was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load run steps.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn thread_run_steps_list() {}

    #[utoipa::path(
        get,
        path = "/v1/threads/{thread_id}/runs/{run_id}/steps/{step_id}",
        tag = "runs",
        params(
            ("thread_id" = String, Path, description = "Thread identifier."),
            ("run_id" = String, Path, description = "Run identifier."),
            ("step_id" = String, Path, description = "Run step identifier.")
        ),
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Visible run step metadata.", body = RunStepObject),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 404, description = "Requested run step was not found.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to load the run step.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn thread_run_step_get() {}

    #[utoipa::path(
        post,
        path = "/v1/realtime/sessions",
        tag = "realtime",
        request_body = CreateRealtimeSessionRequest,
        security(("bearerAuth" = [])),
        responses(
            (status = 200, description = "Realtime session bootstrap result.", body = RealtimeSessionObject),
            (status = 400, description = "Invalid realtime session payload.", body = OpenAiErrorResponse),
            (status = 401, description = "Missing or invalid gateway API key.", body = OpenAiErrorResponse),
            (status = 500, description = "Gateway failed to create the realtime session.", body = OpenAiErrorResponse)
        )
    )]
    pub(super) async fn realtime_sessions() {}
