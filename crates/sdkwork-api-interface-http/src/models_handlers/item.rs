use super::*;

pub(crate) async fn model_retrieve_from_store_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(model_id): Path<String>,
) -> Result<Json<sdkwork_api_contract_openai::models::ModelObject>, Response> {
    get_model_from_store(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        &model_id,
    )
    .await
    .map_err(|_| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to load model",
        )
            .into_response()
    })?
    .map(Json)
    .ok_or_else(|| (axum::http::StatusCode::NOT_FOUND, "model not found").into_response())
}

pub(crate) async fn model_delete_from_store_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(model_id): Path<String>,
) -> Result<Json<Value>, Response> {
    delete_model_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &model_id,
    )
    .await
    .map_err(|_| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to delete model",
        )
            .into_response()
    })?
    .map(Json)
    .ok_or_else(|| (axum::http::StatusCode::NOT_FOUND, "model not found").into_response())
}
