use super::*;

pub(crate) fn apply_stateless_chat_and_conversation_routes(
    router: Router<StatelessGatewayContext>,
) -> Router<StatelessGatewayContext> {
    router
        .route(
            "/v1/chat/completions",
            get(chat_completions_list_handler).post(chat_completions_handler),
        )
        .route(
            "/v1/chat/completions/{completion_id}",
            get(chat_completion_retrieve_handler)
                .post(chat_completion_update_handler)
                .delete(chat_completion_delete_handler),
        )
        .route(
            "/v1/chat/completions/{completion_id}/messages",
            get(chat_completion_messages_list_handler),
        )
        .route("/v1/completions", post(completions_handler))
        .route(
            "/v1/conversations",
            get(conversations_list_handler).post(conversations_handler),
        )
        .route(
            "/v1/conversations/{conversation_id}",
            get(conversation_retrieve_handler)
                .post(conversation_update_handler)
                .delete(conversation_delete_handler),
        )
        .route(
            "/v1/conversations/{conversation_id}/items",
            get(conversation_items_list_handler).post(conversation_items_handler),
        )
        .route(
            "/v1/conversations/{conversation_id}/items/{item_id}",
            get(conversation_item_retrieve_handler).delete(conversation_item_delete_handler),
        )
}
