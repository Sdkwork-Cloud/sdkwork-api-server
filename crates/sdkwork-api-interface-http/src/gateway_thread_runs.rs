async fn thread_and_run_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateThreadAndRunRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::ThreadsRuns(&request))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread and run");
        }
    }

    local_thread_and_run_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.assistant_id,
        request.model.as_deref(),
    )
}

fn local_thread_and_run_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_bad_gateway_response(error, "invalid_assistant_id")
}

fn local_thread_and_run_result(
    tenant_id: &str,
    project_id: &str,
    assistant_id: &str,
    model: Option<&str>,
) -> std::result::Result<RunObject, Response> {
    create_thread_and_run(tenant_id, project_id, assistant_id, model)
        .map_err(local_thread_and_run_error_response)
}

fn local_thread_and_run_response(
    tenant_id: &str,
    project_id: &str,
    assistant_id: &str,
    model: Option<&str>,
) -> Response {
    match local_thread_and_run_result(tenant_id, project_id, assistant_id, model) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

async fn thread_runs_handler(
    request_context: StatelessGatewayRequest,
    Path(thread_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateRunRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadRuns(&thread_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread run");
        }
    }

    match local_thread_runs_create_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &request.assistant_id,
        request.model.as_deref(),
    ) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

async fn thread_runs_list_handler(
    request_context: StatelessGatewayRequest,
    Path(thread_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadRunsList(&thread_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread runs list");
        }
    }

    local_thread_runs_list_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
    )
}

async fn thread_run_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((thread_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadRunsRetrieve(&thread_id, &run_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread run retrieve");
        }
    }

    local_thread_run_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
    )
}

async fn thread_run_update_handler(
    request_context: StatelessGatewayRequest,
    Path((thread_id, run_id)): Path<(String, String)>,
    ExtractJson(request): ExtractJson<UpdateRunRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadRunsUpdate(&thread_id, &run_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread run update");
        }
    }

    local_thread_run_update_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
    )
}

async fn thread_run_cancel_handler(
    request_context: StatelessGatewayRequest,
    Path((thread_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadRunsCancel(&thread_id, &run_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread run cancel");
        }
    }

    local_thread_run_cancel_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
    )
}

async fn thread_run_submit_tool_outputs_handler(
    request_context: StatelessGatewayRequest,
    Path((thread_id, run_id)): Path<(String, String)>,
    ExtractJson(request): ExtractJson<SubmitToolOutputsRunRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadRunsSubmitToolOutputs(&thread_id, &run_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream thread run submit tool outputs",
            );
        }
    }

    let tool_outputs = request
        .tool_outputs
        .iter()
        .map(|output| (output.tool_call_id.as_str(), output.output.as_str()))
        .collect();
    match local_thread_run_submit_tool_outputs_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
        tool_outputs,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}


async fn thread_and_run_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateThreadAndRunRequest>,
) -> Response {
    match relay_thread_and_run_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let Some(usage_model) = response_usage_id_or_single_data_item_id(&response) else {
                return bad_gateway_openai_response(
                    "upstream thread-and-run response missing usage id",
                );
            };
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                "threads/runs",
                usage_model,
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
            return bad_gateway_openai_response("failed to relay upstream thread and run");
        }
    }

    let response = match local_thread_and_run_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.assistant_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        "threads/runs",
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

async fn thread_runs_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(thread_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateRunRequest>,
) -> Response {
    match relay_thread_run_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let Some(run_id) = response.get("id").and_then(Value::as_str) else {
                return bad_gateway_openai_response("upstream thread run response missing id");
            };
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &thread_id,
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
            return bad_gateway_openai_response("failed to relay upstream thread run");
        }
    }

    let response = create_thread_run(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &request.assistant_id,
        request.model.as_deref(),
    );

    let response = match response {
        Ok(response) => response,
        Err(error) => return local_thread_not_found_response(error),
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &thread_id,
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

async fn thread_runs_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(thread_id): Path<String>,
) -> Response {
    match relay_list_thread_runs_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &thread_id,
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
            return bad_gateway_openai_response("failed to relay upstream thread runs list");
        }
    }

    let response = match local_thread_runs_list_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &thread_id,
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

async fn thread_run_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((thread_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_get_thread_run_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &thread_id,
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
            return bad_gateway_openai_response("failed to relay upstream thread run retrieve");
        }
    }

    let response = match local_thread_run_retrieve_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &thread_id,
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

async fn thread_run_update_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((thread_id, run_id)): Path<(String, String)>,
    ExtractJson(request): ExtractJson<UpdateRunRequest>,
) -> Response {
    match relay_update_thread_run_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &thread_id,
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
            return bad_gateway_openai_response("failed to relay upstream thread run update");
        }
    }

    let response = match local_thread_run_update_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &thread_id,
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

async fn thread_run_cancel_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((thread_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_cancel_thread_run_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &thread_id,
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
            return bad_gateway_openai_response("failed to relay upstream thread run cancel");
        }
    }

    let response = match local_thread_run_cancel_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &thread_id,
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

async fn thread_run_submit_tool_outputs_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((thread_id, run_id)): Path<(String, String)>,
    ExtractJson(request): ExtractJson<SubmitToolOutputsRunRequest>,
) -> Response {
    match relay_submit_thread_run_tool_outputs_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &thread_id,
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
            return bad_gateway_openai_response("failed to relay upstream thread run tool outputs");
        }
    }

    let tool_outputs = request
        .tool_outputs
        .iter()
        .map(|output| (output.tool_call_id.as_str(), output.output.as_str()))
        .collect();
    let response = match local_thread_run_submit_tool_outputs_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
        tool_outputs,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &thread_id,
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

