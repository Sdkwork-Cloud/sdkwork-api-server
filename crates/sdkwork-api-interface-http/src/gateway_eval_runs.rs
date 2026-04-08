fn local_eval_runs_list_result(
    tenant_id: &str,
    project_id: &str,
    eval_id: &str,
) -> std::result::Result<ListEvalRunsResponse, Response> {
    list_eval_runs(tenant_id, project_id, eval_id).map_err(local_eval_not_found_response)
}

fn local_eval_runs_list_response(tenant_id: &str, project_id: &str, eval_id: &str) -> Response {
    match local_eval_runs_list_result(tenant_id, project_id, eval_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_eval_run_create_result(
    tenant_id: &str,
    project_id: &str,
    eval_id: &str,
    request: &CreateEvalRunRequest,
) -> std::result::Result<EvalRunObject, Response> {
    create_eval_run(tenant_id, project_id, eval_id, request).map_err(local_eval_not_found_response)
}

fn local_eval_run_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    eval_id: &str,
    run_id: &str,
) -> std::result::Result<EvalRunObject, Response> {
    get_eval_run(tenant_id, project_id, eval_id, run_id).map_err(local_eval_run_not_found_response)
}

fn local_eval_run_retrieve_response(
    tenant_id: &str,
    project_id: &str,
    eval_id: &str,
    run_id: &str,
) -> Response {
    match local_eval_run_retrieve_result(tenant_id, project_id, eval_id, run_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_eval_run_delete_result(
    tenant_id: &str,
    project_id: &str,
    eval_id: &str,
    run_id: &str,
) -> std::result::Result<DeleteEvalRunResponse, Response> {
    sdkwork_api_app_gateway::delete_eval_run(tenant_id, project_id, eval_id, run_id)
        .map_err(local_eval_run_not_found_response)
}

fn local_eval_run_delete_response(
    tenant_id: &str,
    project_id: &str,
    eval_id: &str,
    run_id: &str,
) -> Response {
    match local_eval_run_delete_result(tenant_id, project_id, eval_id, run_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_eval_run_cancel_result(
    tenant_id: &str,
    project_id: &str,
    eval_id: &str,
    run_id: &str,
) -> std::result::Result<EvalRunObject, Response> {
    sdkwork_api_app_gateway::cancel_eval_run(tenant_id, project_id, eval_id, run_id)
        .map_err(local_eval_run_not_found_response)
}

fn local_eval_run_cancel_response(
    tenant_id: &str,
    project_id: &str,
    eval_id: &str,
    run_id: &str,
) -> Response {
    match local_eval_run_cancel_result(tenant_id, project_id, eval_id, run_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_eval_run_output_items_list_result(
    tenant_id: &str,
    project_id: &str,
    eval_id: &str,
    run_id: &str,
) -> std::result::Result<ListEvalRunOutputItemsResponse, Response> {
    sdkwork_api_app_gateway::list_eval_run_output_items(tenant_id, project_id, eval_id, run_id)
        .map_err(local_eval_run_not_found_response)
}

fn local_eval_run_output_items_list_response(
    tenant_id: &str,
    project_id: &str,
    eval_id: &str,
    run_id: &str,
) -> Response {
    match local_eval_run_output_items_list_result(tenant_id, project_id, eval_id, run_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_eval_run_output_item_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    eval_id: &str,
    run_id: &str,
    output_item_id: &str,
) -> std::result::Result<EvalRunOutputItemObject, Response> {
    sdkwork_api_app_gateway::get_eval_run_output_item(
        tenant_id,
        project_id,
        eval_id,
        run_id,
        output_item_id,
    )
    .map_err(local_eval_run_output_item_not_found_response)
}

fn local_eval_run_output_item_retrieve_response(
    tenant_id: &str,
    project_id: &str,
    eval_id: &str,
    run_id: &str,
    output_item_id: &str,
) -> Response {
    match local_eval_run_output_item_retrieve_result(
        tenant_id,
        project_id,
        eval_id,
        run_id,
        output_item_id,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

async fn eval_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(eval_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::EvalsRetrieve(&eval_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval retrieve");
        }
    }

    local_eval_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
    )
}

async fn eval_update_handler(
    request_context: StatelessGatewayRequest,
    Path(eval_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateEvalRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::EvalsUpdate(&eval_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval update");
        }
    }

    local_eval_update_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &request,
    )
}

async fn eval_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(eval_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::EvalsDelete(&eval_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval delete");
        }
    }

    local_eval_delete_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
    )
}

async fn eval_runs_list_handler(
    request_context: StatelessGatewayRequest,
    Path(eval_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::EvalRunsList(&eval_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval runs list");
        }
    }

    local_eval_runs_list_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
    )
}

async fn eval_runs_handler(
    request_context: StatelessGatewayRequest,
    Path(eval_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateEvalRunRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::EvalRuns(&eval_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval run create");
        }
    }

    match local_eval_run_create_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &request,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

async fn eval_run_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((eval_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::EvalRunsRetrieve(&eval_id, &run_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval run retrieve");
        }
    }

    local_eval_run_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &run_id,
    )
}

async fn eval_run_delete_handler(
    request_context: StatelessGatewayRequest,
    Path((eval_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::EvalRunsDelete(&eval_id, &run_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval run delete");
        }
    }

    local_eval_run_delete_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &run_id,
    )
}

async fn eval_run_cancel_handler(
    request_context: StatelessGatewayRequest,
    Path((eval_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::EvalRunsCancel(&eval_id, &run_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream eval run cancel");
        }
    }

    local_eval_run_cancel_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &run_id,
    )
}

async fn eval_run_output_items_list_handler(
    request_context: StatelessGatewayRequest,
    Path((eval_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::EvalRunOutputItemsList(&eval_id, &run_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream eval run output items list",
            );
        }
    }

    local_eval_run_output_items_list_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &run_id,
    )
}

async fn eval_run_output_item_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((eval_id, run_id, output_item_id)): Path<(String, String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::EvalRunOutputItemsRetrieve(&eval_id, &run_id, &output_item_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream eval run output item retrieve",
            );
        }
    }

    local_eval_run_output_item_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &run_id,
        &output_item_id,
    )
}


async fn eval_runs_list_with_state_handler(
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

    let response = match local_eval_runs_list_result(
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

async fn eval_runs_with_state_handler(
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
            let run_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(eval_id.as_str());
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

    let response = match local_eval_run_create_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &request,
    ) {
        Ok(response) => response,
        Err(response) => return response,
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

async fn eval_run_retrieve_with_state_handler(
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

    let response = match local_eval_run_retrieve_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &run_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
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

async fn eval_run_delete_with_state_handler(
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

    let response = match local_eval_run_delete_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &run_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
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

async fn eval_run_cancel_with_state_handler(
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

    let response = match local_eval_run_cancel_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &run_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
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

async fn eval_run_output_items_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((eval_id, run_id)): Path<(String, String)>,
) -> Response {
    match sdkwork_api_app_gateway::relay_list_eval_run_output_items_from_store(
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
            return bad_gateway_openai_response(
                "failed to relay upstream eval run output items list",
            );
        }
    }

    let response = match local_eval_run_output_items_list_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &run_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "evals",
        &eval_id,
        &run_id,
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

async fn eval_run_output_item_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((eval_id, run_id, output_item_id)): Path<(String, String, String)>,
) -> Response {
    match sdkwork_api_app_gateway::relay_get_eval_run_output_item_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &run_id,
        &output_item_id,
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
                &output_item_id,
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
            return bad_gateway_openai_response(
                "failed to relay upstream eval run output item retrieve",
            );
        }
    }

    let response = match local_eval_run_output_item_retrieve_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &eval_id,
        &run_id,
        &output_item_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "evals",
        &eval_id,
        &output_item_id,
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

