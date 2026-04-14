async fn evals_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateEvalRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Evals(&request)).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval");
        }
    }

    match create_eval(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(error) => local_eval_not_found_response(error),
    }
}

async fn evals_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::EvalsList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream evals list");
        }
    }

    match list_evals(request_context.tenant_id(), request_context.project_id()) {
        Ok(response) => Json(response).into_response(),
        Err(error) => local_gateway_invalid_or_bad_gateway_response(error, "invalid_eval_request"),
    }
}

fn local_eval_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_eval_request",
        "Requested eval was not found.",
    )
}

fn local_eval_run_not_found_response(error: anyhow::Error) -> Response {
    let message = error.to_string();
    if local_gateway_error_is_invalid_request(&message) {
        return invalid_request_openai_response(message, "invalid_eval_request");
    }
    if message.to_ascii_lowercase().contains("eval run not found") {
        return not_found_openai_response("Requested eval run was not found.");
    }

    local_eval_not_found_response(error)
}

fn local_eval_run_output_item_not_found_response(error: anyhow::Error) -> Response {
    let message = error.to_string();
    if local_gateway_error_is_invalid_request(&message) {
        return invalid_request_openai_response(message, "invalid_eval_request");
    }
    if message
        .to_ascii_lowercase()
        .contains("eval run output item not found")
    {
        return not_found_openai_response("Requested eval run output item was not found.");
    }

    local_eval_run_not_found_response(error)
}

fn local_eval_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    eval_id: &str,
) -> std::result::Result<EvalObject, Response> {
    get_eval(tenant_id, project_id, eval_id).map_err(local_eval_not_found_response)
}

fn local_eval_retrieve_response(tenant_id: &str, project_id: &str, eval_id: &str) -> Response {
    match local_eval_retrieve_result(tenant_id, project_id, eval_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_eval_update_result(
    tenant_id: &str,
    project_id: &str,
    eval_id: &str,
    request: &UpdateEvalRequest,
) -> std::result::Result<EvalObject, Response> {
    update_eval(tenant_id, project_id, eval_id, request).map_err(local_eval_not_found_response)
}

fn local_eval_update_response(
    tenant_id: &str,
    project_id: &str,
    eval_id: &str,
    request: &UpdateEvalRequest,
) -> Response {
    match local_eval_update_result(tenant_id, project_id, eval_id, request) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_eval_delete_result(
    tenant_id: &str,
    project_id: &str,
    eval_id: &str,
) -> std::result::Result<DeleteEvalResponse, Response> {
    delete_eval(tenant_id, project_id, eval_id).map_err(local_eval_not_found_response)
}

fn local_eval_delete_response(tenant_id: &str, project_id: &str, eval_id: &str) -> Response {
    match local_eval_delete_result(tenant_id, project_id, eval_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}


async fn evals_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateEvalRequest>,
) -> Response {
    match relay_eval_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let Some(eval_id) = response.get("id").and_then(Value::as_str) else {
                return bad_gateway_openai_response("upstream eval response missing id");
            };
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "evals",
                &request.name,
                eval_id,
                40,
                0.04,
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
            return bad_gateway_openai_response("failed to relay upstream eval");
        }
    }

    let response = match create_eval(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => response,
        Err(error) => return local_eval_not_found_response(error),
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "evals",
        &request.name,
        response.id.as_str(),
        40,
        0.04,
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

async fn evals_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Response {
    match relay_list_evals_from_store(
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
                "evals",
                "evals",
                15,
                0.015,
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
            return bad_gateway_openai_response("failed to relay upstream evals list");
        }
    }

    let response = match list_evals(request_context.tenant_id(), request_context.project_id()) {
        Ok(response) => response,
        Err(error) => {
            return local_gateway_invalid_or_bad_gateway_response(error, "invalid_eval_request");
        }
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "evals",
        "evals",
        15,
        0.015,
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

async fn eval_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(eval_id): Path<String>,
) -> Response {
    match relay_get_eval_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "evals",
                &eval_id,
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
            return bad_gateway_openai_response("failed to relay upstream eval retrieve");
        }
    }

    let response = match local_eval_retrieve_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "evals",
        &eval_id,
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

async fn eval_update_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(eval_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateEvalRequest>,
) -> Response {
    match relay_update_eval_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "evals",
                &eval_id,
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
            return bad_gateway_openai_response("failed to relay upstream eval update");
        }
    }

    let response = match local_eval_update_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &request,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "evals",
        &eval_id,
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

async fn eval_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(eval_id): Path<String>,
) -> Response {
    match relay_delete_eval_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "evals",
                &eval_id,
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
            return bad_gateway_openai_response("failed to relay upstream eval delete");
        }
    }

    let response = match local_eval_delete_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "evals",
        &eval_id,
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

