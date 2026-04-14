use super::*;

pub(crate) fn apply_stateful_thread_and_response_routes(
    router: Router<GatewayApiState>,
) -> Router<GatewayApiState> {
    router
        .route("/v1/threads", post(threads_with_state_handler))
        .route(
            "/v1/threads/{thread_id}",
            get(thread_retrieve_with_state_handler)
                .post(thread_update_with_state_handler)
                .delete(thread_delete_with_state_handler),
        )
        .route(
            "/v1/threads/{thread_id}/messages",
            get(thread_messages_list_with_state_handler).post(thread_messages_with_state_handler),
        )
        .route(
            "/v1/threads/{thread_id}/messages/{message_id}",
            get(thread_message_retrieve_with_state_handler)
                .post(thread_message_update_with_state_handler)
                .delete(thread_message_delete_with_state_handler),
        )
        .route("/v1/threads/runs", post(thread_and_run_with_state_handler))
        .route(
            "/v1/threads/{thread_id}/runs",
            get(thread_runs_list_with_state_handler).post(thread_runs_with_state_handler),
        )
        .route(
            "/v1/threads/{thread_id}/runs/{run_id}",
            get(thread_run_retrieve_with_state_handler).post(thread_run_update_with_state_handler),
        )
        .route(
            "/v1/threads/{thread_id}/runs/{run_id}/cancel",
            post(thread_run_cancel_with_state_handler),
        )
        .route(
            "/v1/threads/{thread_id}/runs/{run_id}/submit_tool_outputs",
            post(thread_run_submit_tool_outputs_with_state_handler),
        )
        .route(
            "/v1/threads/{thread_id}/runs/{run_id}/steps",
            get(thread_run_steps_list_with_state_handler),
        )
        .route(
            "/v1/threads/{thread_id}/runs/{run_id}/steps/{step_id}",
            get(thread_run_step_retrieve_with_state_handler),
        )
        .route("/v1/responses", post(responses_with_state_handler))
        .route(
            "/v1/responses/input_tokens",
            post(response_input_tokens_with_state_handler),
        )
        .route(
            "/v1/responses/compact",
            post(response_compact_with_state_handler),
        )
        .route(
            "/v1/responses/{response_id}",
            get(response_retrieve_with_state_handler).delete(response_delete_with_state_handler),
        )
        .route(
            "/v1/responses/{response_id}/input_items",
            get(response_input_items_list_with_state_handler),
        )
        .route(
            "/v1/responses/{response_id}/cancel",
            post(response_cancel_with_state_handler),
        )
}
