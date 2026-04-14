fn local_fine_tuning_job_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_fine_tuning_request",
        "Requested fine-tuning job was not found.",
    )
}

fn local_fine_tuning_job_create_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_bad_gateway_response(error, "invalid_fine_tuning_request")
}

fn local_fine_tuning_checkpoint_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_fine_tuning_request",
        "Requested fine-tuning checkpoint was not found.",
    )
}

fn local_fine_tuning_checkpoint_permission_not_found_response(error: anyhow::Error) -> Response {
    let message = error.to_string();
    if local_gateway_error_is_invalid_request(&message) {
        return invalid_request_openai_response(message, "invalid_fine_tuning_request");
    }
    if message
        .to_ascii_lowercase()
        .contains("fine tuning checkpoint permission not found")
    {
        return not_found_openai_response(
            "Requested fine-tuning checkpoint permission was not found.",
        );
    }

    local_fine_tuning_checkpoint_not_found_response(error)
}

fn local_fine_tuning_job_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    fine_tuning_job_id: &str,
) -> std::result::Result<FineTuningJobObject, Response> {
    get_fine_tuning_job(tenant_id, project_id, fine_tuning_job_id)
        .map_err(local_fine_tuning_job_not_found_response)
}

fn local_fine_tuning_job_retrieve_response(
    tenant_id: &str,
    project_id: &str,
    fine_tuning_job_id: &str,
) -> Response {
    match local_fine_tuning_job_retrieve_result(tenant_id, project_id, fine_tuning_job_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_fine_tuning_job_cancel_result(
    tenant_id: &str,
    project_id: &str,
    fine_tuning_job_id: &str,
) -> std::result::Result<FineTuningJobObject, Response> {
    cancel_fine_tuning_job(tenant_id, project_id, fine_tuning_job_id)
        .map_err(local_fine_tuning_job_not_found_response)
}

fn local_fine_tuning_job_cancel_response(
    tenant_id: &str,
    project_id: &str,
    fine_tuning_job_id: &str,
) -> Response {
    match local_fine_tuning_job_cancel_result(tenant_id, project_id, fine_tuning_job_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_fine_tuning_job_events_result(
    tenant_id: &str,
    project_id: &str,
    fine_tuning_job_id: &str,
) -> std::result::Result<ListFineTuningJobEventsResponse, Response> {
    list_fine_tuning_job_events(tenant_id, project_id, fine_tuning_job_id)
        .map_err(local_fine_tuning_job_not_found_response)
}

fn local_fine_tuning_job_events_response(
    tenant_id: &str,
    project_id: &str,
    fine_tuning_job_id: &str,
) -> Response {
    match local_fine_tuning_job_events_result(tenant_id, project_id, fine_tuning_job_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_fine_tuning_job_checkpoints_result(
    tenant_id: &str,
    project_id: &str,
    fine_tuning_job_id: &str,
) -> std::result::Result<ListFineTuningJobCheckpointsResponse, Response> {
    list_fine_tuning_job_checkpoints(tenant_id, project_id, fine_tuning_job_id)
        .map_err(local_fine_tuning_job_not_found_response)
}

fn local_fine_tuning_job_checkpoints_response(
    tenant_id: &str,
    project_id: &str,
    fine_tuning_job_id: &str,
) -> Response {
    match local_fine_tuning_job_checkpoints_result(tenant_id, project_id, fine_tuning_job_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_fine_tuning_job_pause_result(
    tenant_id: &str,
    project_id: &str,
    fine_tuning_job_id: &str,
) -> std::result::Result<FineTuningJobObject, Response> {
    sdkwork_api_app_gateway::pause_fine_tuning_job(tenant_id, project_id, fine_tuning_job_id)
        .map_err(local_fine_tuning_job_not_found_response)
}

fn local_fine_tuning_job_pause_response(
    tenant_id: &str,
    project_id: &str,
    fine_tuning_job_id: &str,
) -> Response {
    match local_fine_tuning_job_pause_result(tenant_id, project_id, fine_tuning_job_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_fine_tuning_job_resume_result(
    tenant_id: &str,
    project_id: &str,
    fine_tuning_job_id: &str,
) -> std::result::Result<FineTuningJobObject, Response> {
    sdkwork_api_app_gateway::resume_fine_tuning_job(tenant_id, project_id, fine_tuning_job_id)
        .map_err(local_fine_tuning_job_not_found_response)
}

fn local_fine_tuning_job_resume_response(
    tenant_id: &str,
    project_id: &str,
    fine_tuning_job_id: &str,
) -> Response {
    match local_fine_tuning_job_resume_result(tenant_id, project_id, fine_tuning_job_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}


async fn fine_tuning_jobs_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateFineTuningJobRequest>,
) -> Response {
    match relay_fine_tuning_job_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let Some(fine_tuning_job_id) = response.get("id").and_then(Value::as_str) else {
                return bad_gateway_openai_response(
                    "upstream fine tuning job response missing id",
                );
            };
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "fine_tuning",
                &request.model,
                fine_tuning_job_id,
                200,
                0.2,
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
            return bad_gateway_openai_response("failed to relay upstream fine tuning job");
        }
    }

    let response = match create_fine_tuning_job(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
    ) {
        Ok(response) => response,
        Err(error) => return local_fine_tuning_job_create_error_response(error),
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "fine_tuning",
        &request.model,
        response.id.as_str(),
        200,
        0.2,
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

async fn fine_tuning_jobs_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Response {
    match relay_list_fine_tuning_jobs_from_store(
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
                "fine_tuning",
                "jobs",
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
            return bad_gateway_openai_response("failed to relay upstream fine tuning jobs list");
        }
    }

    let response =
        match list_fine_tuning_jobs(request_context.tenant_id(), request_context.project_id()) {
            Ok(response) => response,
            Err(error) => return local_fine_tuning_job_not_found_response(error),
        };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "fine_tuning",
        "jobs",
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

async fn fine_tuning_job_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match relay_get_fine_tuning_job_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "fine_tuning",
                &fine_tuning_job_id,
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
            return bad_gateway_openai_response(
                "failed to relay upstream fine tuning job retrieve",
            );
        }
    }

    let response = match local_fine_tuning_job_retrieve_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "fine_tuning",
        &fine_tuning_job_id,
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

async fn fine_tuning_job_cancel_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match relay_cancel_fine_tuning_job_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "fine_tuning",
                &fine_tuning_job_id,
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
            return bad_gateway_openai_response("failed to relay upstream fine tuning job cancel");
        }
    }

    let response = match local_fine_tuning_job_cancel_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "fine_tuning",
        &fine_tuning_job_id,
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

async fn fine_tuning_job_events_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match relay_list_fine_tuning_job_events_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "fine_tuning",
                &fine_tuning_job_id,
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
            return bad_gateway_openai_response("failed to relay upstream fine tuning job events");
        }
    }

    let response = match local_fine_tuning_job_events_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "fine_tuning",
        &fine_tuning_job_id,
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

async fn fine_tuning_job_checkpoints_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match relay_list_fine_tuning_job_checkpoints_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "fine_tuning",
                &fine_tuning_job_id,
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
            return bad_gateway_openai_response(
                "failed to relay upstream fine tuning job checkpoints",
            );
        }
    }

    let response = match local_fine_tuning_job_checkpoints_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "fine_tuning",
        &fine_tuning_job_id,
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

async fn fine_tuning_job_pause_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match sdkwork_api_app_gateway::relay_pause_fine_tuning_job_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "fine_tuning",
                &fine_tuning_job_id,
                8,
                0.008,
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
            return bad_gateway_openai_response("failed to relay upstream fine tuning job pause");
        }
    }

    let response = match local_fine_tuning_job_pause_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "fine_tuning",
        &fine_tuning_job_id,
        8,
        0.008,
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

async fn fine_tuning_job_resume_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match sdkwork_api_app_gateway::relay_resume_fine_tuning_job_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "fine_tuning",
                &fine_tuning_job_id,
                8,
                0.008,
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
            return bad_gateway_openai_response("failed to relay upstream fine tuning job resume");
        }
    }

    let response = match local_fine_tuning_job_resume_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "fine_tuning",
        &fine_tuning_job_id,
        8,
        0.008,
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

