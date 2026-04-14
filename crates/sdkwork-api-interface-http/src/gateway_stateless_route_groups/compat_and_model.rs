use super::*;

pub(crate) fn apply_stateless_compat_and_model_routes(
    router: Router<StatelessGatewayContext>,
) -> Router<StatelessGatewayContext> {
    router
        .route("/v1/messages", post(anthropic_messages_handler))
        .route(
            "/v1/messages/count_tokens",
            post(anthropic_count_tokens_handler),
        )
        .route("/v1beta/models/{*tail}", post(gemini_models_compat_handler))
        .route("/v1/models", get(list_models_handler))
        .route(
            "/v1/models/{model_id}",
            get(model_retrieve_handler).delete(model_delete_handler),
        )
}
