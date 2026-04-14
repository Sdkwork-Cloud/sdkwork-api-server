use super::*;

pub(crate) fn apply_stateful_compat_and_model_routes(
    router: Router<GatewayApiState>,
) -> Router<GatewayApiState> {
    router
        .route("/v1/messages", post(anthropic_messages_with_state_handler))
        .route(
            "/v1/messages/count_tokens",
            post(anthropic_count_tokens_with_state_handler),
        )
        .route(
            "/v1beta/models/{*tail}",
            post(gemini_models_compat_with_state_handler),
        )
        .route("/v1/models", get(list_models_from_store_handler))
        .route(
            "/v1/models/{model_id}",
            get(model_retrieve_from_store_handler).delete(model_delete_from_store_handler),
        )
}
