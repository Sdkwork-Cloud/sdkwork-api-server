use super::*;

pub(crate) fn apply_stateful_chat_and_conversation_routes(
    router: Router<GatewayApiState>,
) -> Router<GatewayApiState> {
    router
        .route(
            "/v1/chat/completions",
            get(chat_completions_list_with_state_handler).post(chat_completions_with_state_handler),
        )
        .route(
            "/v1/chat/completions/{completion_id}",
            get(chat_completion_retrieve_with_state_handler)
                .post(chat_completion_update_with_state_handler)
                .delete(chat_completion_delete_with_state_handler),
        )
        .route(
            "/v1/chat/completions/{completion_id}/messages",
            get(chat_completion_messages_list_with_state_handler),
        )
        .route("/v1/completions", post(completions_with_state_handler))
        .route(
            "/v1/conversations",
            get(conversations_list_with_state_handler).post(conversations_with_state_handler),
        )
        .route(
            "/v1/conversations/{conversation_id}",
            get(conversation_retrieve_with_state_handler)
                .post(conversation_update_with_state_handler)
                .delete(conversation_delete_with_state_handler),
        )
        .route(
            "/v1/conversations/{conversation_id}/items",
            get(conversation_items_list_with_state_handler)
                .post(conversation_items_with_state_handler),
        )
        .route(
            "/v1/conversations/{conversation_id}/items/{item_id}",
            get(conversation_item_retrieve_with_state_handler)
                .delete(conversation_item_delete_with_state_handler),
        )
}
