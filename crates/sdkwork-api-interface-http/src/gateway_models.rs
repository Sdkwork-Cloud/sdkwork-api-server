fn local_models_list_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_bad_gateway_response(error, "invalid_model")
}

async fn list_models_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::ModelsList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream model list");
        }
    }

    let response = match list_models(request_context.tenant_id(), request_context.project_id()) {
        Ok(response) => response,
        Err(error) => return local_models_list_error_response(error),
    };

    Json(response).into_response()
}

fn local_model_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_model",
        "Requested model was not found.",
    )
}

fn local_model_retrieve_response(tenant_id: &str, project_id: &str, model_id: &str) -> Response {
    match get_model(tenant_id, project_id, model_id).map_err(local_model_not_found_response) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_model_delete_response(tenant_id: &str, project_id: &str, model_id: &str) -> Response {
    match delete_model(tenant_id, project_id, model_id).map_err(local_model_not_found_response) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

async fn model_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(model_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::ModelsRetrieve(&model_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream model");
        }
    }

    local_model_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &model_id,
    )
}

async fn model_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(model_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::ModelsDelete(&model_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream model delete");
        }
    }

    local_model_delete_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &model_id,
    )
}

async fn list_models_from_store_handler(
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

async fn model_retrieve_from_store_handler(
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

async fn model_delete_from_store_handler(
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

