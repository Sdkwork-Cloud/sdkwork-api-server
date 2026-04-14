use super::*;

pub(crate) fn apply_stateless_thread_and_response_routes(
    router: Router<StatelessGatewayContext>,
) -> Router<StatelessGatewayContext> {
    router
        .route("/v1/threads", post(threads_handler))
        .route(
            "/v1/threads/{thread_id}",
            get(thread_retrieve_handler)
                .post(thread_update_handler)
                .delete(thread_delete_handler),
        )
        .route(
            "/v1/threads/{thread_id}/messages",
            get(thread_messages_list_handler).post(thread_messages_handler),
        )
        .route(
            "/v1/threads/{thread_id}/messages/{message_id}",
            get(thread_message_retrieve_handler)
                .post(thread_message_update_handler)
                .delete(thread_message_delete_handler),
        )
        .route("/v1/threads/runs", post(thread_and_run_handler))
        .route(
            "/v1/threads/{thread_id}/runs",
            get(thread_runs_list_handler).post(thread_runs_handler),
        )
        .route(
            "/v1/threads/{thread_id}/runs/{run_id}",
            get(thread_run_retrieve_handler).post(thread_run_update_handler),
        )
        .route(
            "/v1/threads/{thread_id}/runs/{run_id}/cancel",
            post(thread_run_cancel_handler),
        )
        .route(
            "/v1/threads/{thread_id}/runs/{run_id}/submit_tool_outputs",
            post(thread_run_submit_tool_outputs_handler),
        )
        .route(
            "/v1/threads/{thread_id}/runs/{run_id}/steps",
            get(thread_run_steps_list_handler),
        )
        .route(
            "/v1/threads/{thread_id}/runs/{run_id}/steps/{step_id}",
            get(thread_run_step_retrieve_handler),
        )
        .route("/v1/responses", post(responses_handler))
        .route(
            "/v1/responses/input_tokens",
            post(response_input_tokens_handler),
        )
        .route("/v1/responses/compact", post(response_compact_handler))
        .route(
            "/v1/responses/{response_id}",
            get(response_retrieve_handler).delete(response_delete_handler),
        )
        .route(
            "/v1/responses/{response_id}/input_items",
            get(response_input_items_list_handler),
        )
        .route(
            "/v1/responses/{response_id}/cancel",
            post(response_cancel_handler),
        )
}
