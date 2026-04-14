async fn batches_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateBatchRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Batches(&request)).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream batch");
        }
    }

    match create_batch(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(error) => local_batch_not_found_response(error),
    }
}

async fn batches_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::BatchesList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream batches list");
        }
    }

    match list_batches(request_context.tenant_id(), request_context.project_id()) {
        Ok(response) => Json(response).into_response(),
        Err(error) => local_batch_not_found_response(error),
    }
}

fn local_batch_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_batch_request",
        "Requested batch was not found.",
    )
}

fn local_batch_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    batch_id: &str,
) -> std::result::Result<BatchObject, Response> {
    get_batch(tenant_id, project_id, batch_id).map_err(local_batch_not_found_response)
}

fn local_batch_retrieve_response(tenant_id: &str, project_id: &str, batch_id: &str) -> Response {
    match local_batch_retrieve_result(tenant_id, project_id, batch_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_batch_cancel_result(
    tenant_id: &str,
    project_id: &str,
    batch_id: &str,
) -> std::result::Result<BatchObject, Response> {
    cancel_batch(tenant_id, project_id, batch_id).map_err(local_batch_not_found_response)
}

fn local_batch_cancel_response(tenant_id: &str, project_id: &str, batch_id: &str) -> Response {
    match local_batch_cancel_result(tenant_id, project_id, batch_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

async fn batch_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(batch_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::BatchesRetrieve(&batch_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream batch retrieve");
        }
    }

    local_batch_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &batch_id,
    )
}

async fn batch_cancel_handler(
    request_context: StatelessGatewayRequest,
    Path(batch_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::BatchesCancel(&batch_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream batch cancel");
        }
    }

    local_batch_cancel_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &batch_id,
    )
}


async fn batches_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateBatchRequest>,
) -> Response {
    match relay_batch_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let Some(batch_id) = response.get("id").and_then(Value::as_str) else {
                return bad_gateway_openai_response("upstream batch response missing id");
            };
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "batches",
                &request.endpoint,
                batch_id,
                60,
                0.06,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream batch");
        }
    }

    let response = match create_batch(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => response,
        Err(error) => return local_batch_not_found_response(error),
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "batches",
        &request.endpoint,
        response.id.as_str(),
        60,
        0.06,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn batches_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Response {
    match relay_list_batches_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "batches",
                "batches",
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream batches list");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "batches",
        "batches",
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    match list_batches(request_context.tenant_id(), request_context.project_id()) {
        Ok(response) => Json(response).into_response(),
        Err(error) => local_batch_not_found_response(error),
    }
}

async fn batch_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(batch_id): Path<String>,
) -> Response {
    match relay_get_batch_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &batch_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "batches",
                &batch_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream batch retrieve");
        }
    }

    let response = match local_batch_retrieve_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &batch_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "batches",
        &batch_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn batch_cancel_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(batch_id): Path<String>,
) -> Response {
    match relay_cancel_batch_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &batch_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "batches",
                &batch_id,
                20,
                0.02,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream batch cancel");
        }
    }

    let response = match local_batch_cancel_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &batch_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "batches",
        &batch_id,
        20,
        0.02,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

