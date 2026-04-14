use super::*;

fn local_eval_run_error_response(error: anyhow::Error) -> Response {
    let message = error.to_string();
    if local_gateway_error_is_invalid_request(&message) {
        return invalid_request_openai_response(message, "invalid_eval_request");
    }

    let lower = message.to_ascii_lowercase();
    if lower.contains("eval run not found") {
        return local_gateway_error_response(error, "Requested eval run was not found.");
    }

    local_gateway_error_response(error, "Requested eval was not found.")
}

pub(crate) async fn eval_runs_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(eval_id): Path<String>,
) -> Response {
    match relay_list_eval_runs_from_store(
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
            return bad_gateway_openai_response("failed to relay upstream eval runs list");
        }
    }

    let response = match list_eval_runs(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_eval_run_error_response(error),
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "evals",
        &eval_id,
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

pub(crate) async fn eval_runs_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(eval_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateEvalRunRequest>,
) -> Response {
    match relay_eval_run_from_store(
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
            let Some(run_id) = response.get("id").and_then(Value::as_str) else {
                return bad_gateway_openai_response("upstream eval run response missing id");
            };
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "evals",
                &eval_id,
                run_id,
                25,
                0.025,
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
            return bad_gateway_openai_response("failed to relay upstream eval run create");
        }
    }

    let response = match create_eval_run(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &request,
    ) {
        Ok(response) => response,
        Err(error) => return local_eval_run_error_response(error),
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "evals",
        &eval_id,
        response.id.as_str(),
        25,
        0.025,
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

pub(crate) async fn eval_run_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((eval_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_get_eval_run_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &run_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "evals",
                &eval_id,
                &run_id,
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
            return bad_gateway_openai_response("failed to relay upstream eval run retrieve");
        }
    }

    let response = match get_eval_run(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &run_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_eval_run_error_response(error),
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "evals",
        &eval_id,
        &run_id,
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

pub(crate) async fn eval_run_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((eval_id, run_id)): Path<(String, String)>,
) -> Response {
    match sdkwork_api_app_gateway::relay_delete_eval_run_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &run_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "evals",
                &eval_id,
                &run_id,
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
            return bad_gateway_openai_response("failed to relay upstream eval run delete");
        }
    }

    let response = match sdkwork_api_app_gateway::delete_eval_run(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &run_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_eval_run_error_response(error),
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "evals",
        &eval_id,
        &run_id,
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

pub(crate) async fn eval_run_cancel_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((eval_id, run_id)): Path<(String, String)>,
) -> Response {
    match sdkwork_api_app_gateway::relay_cancel_eval_run_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &run_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "evals",
                &eval_id,
                &run_id,
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
            return bad_gateway_openai_response("failed to relay upstream eval run cancel");
        }
    }

    let response = match sdkwork_api_app_gateway::cancel_eval_run(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &run_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_eval_run_error_response(error),
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "evals",
        &eval_id,
        &run_id,
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
