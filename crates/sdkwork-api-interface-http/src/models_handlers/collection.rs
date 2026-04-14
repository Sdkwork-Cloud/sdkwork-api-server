use super::*;

pub(crate) async fn list_models_from_store_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Result<Json<sdkwork_api_contract_openai::models::ListModelsResponse>, Response> {
    list_models_from_store(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
    )
    .await
    .map(Json)
    .map_err(|_| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to load models",
        )
            .into_response()
    })
}
