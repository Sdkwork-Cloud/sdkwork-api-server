fn local_fine_tuning_checkpoint_permissions_create_result(
    tenant_id: &str,
    project_id: &str,
    fine_tuned_model_checkpoint: &str,
    request: &CreateFineTuningCheckpointPermissionsRequest,
) -> std::result::Result<ListFineTuningCheckpointPermissionsResponse, Response> {
    sdkwork_api_app_gateway::create_fine_tuning_checkpoint_permissions(
        tenant_id,
        project_id,
        fine_tuned_model_checkpoint,
        request,
    )
    .map_err(local_fine_tuning_checkpoint_not_found_response)
}

fn local_fine_tuning_checkpoint_permissions_create_response(
    tenant_id: &str,
    project_id: &str,
    fine_tuned_model_checkpoint: &str,
    request: &CreateFineTuningCheckpointPermissionsRequest,
) -> Response {
    match local_fine_tuning_checkpoint_permissions_create_result(
        tenant_id,
        project_id,
        fine_tuned_model_checkpoint,
        request,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_fine_tuning_checkpoint_permissions_list_result(
    tenant_id: &str,
    project_id: &str,
    fine_tuned_model_checkpoint: &str,
) -> std::result::Result<ListFineTuningCheckpointPermissionsResponse, Response> {
    sdkwork_api_app_gateway::list_fine_tuning_checkpoint_permissions(
        tenant_id,
        project_id,
        fine_tuned_model_checkpoint,
    )
    .map_err(local_fine_tuning_checkpoint_not_found_response)
}

fn local_fine_tuning_checkpoint_permissions_list_response(
    tenant_id: &str,
    project_id: &str,
    fine_tuned_model_checkpoint: &str,
) -> Response {
    match local_fine_tuning_checkpoint_permissions_list_result(
        tenant_id,
        project_id,
        fine_tuned_model_checkpoint,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_fine_tuning_checkpoint_permission_delete_result(
    tenant_id: &str,
    project_id: &str,
    fine_tuned_model_checkpoint: &str,
    permission_id: &str,
) -> std::result::Result<DeleteFineTuningCheckpointPermissionResponse, Response> {
    sdkwork_api_app_gateway::delete_fine_tuning_checkpoint_permission(
        tenant_id,
        project_id,
        fine_tuned_model_checkpoint,
        permission_id,
    )
    .map_err(local_fine_tuning_checkpoint_permission_not_found_response)
}

fn local_fine_tuning_checkpoint_permission_delete_response(
    tenant_id: &str,
    project_id: &str,
    fine_tuned_model_checkpoint: &str,
    permission_id: &str,
) -> Response {
    match local_fine_tuning_checkpoint_permission_delete_result(
        tenant_id,
        project_id,
        fine_tuned_model_checkpoint,
        permission_id,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

async fn fine_tuning_jobs_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateFineTuningJobRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::FineTuningJobs(&request))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream fine tuning job");
        }
    }

    Json(
        create_fine_tuning_job(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        )
        .expect("fine tuning"),
    )
    .into_response()
}

async fn fine_tuning_jobs_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::FineTuningJobsList).await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream fine tuning jobs list");
        }
    }

    Json(
        list_fine_tuning_jobs(request_context.tenant_id(), request_context.project_id())
            .expect("fine tuning list"),
    )
    .into_response()
}

async fn fine_tuning_job_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::FineTuningJobsRetrieve(&fine_tuning_job_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream fine tuning job retrieve",
            );
        }
    }

    local_fine_tuning_job_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    )
}

async fn fine_tuning_job_cancel_handler(
    request_context: StatelessGatewayRequest,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::FineTuningJobsCancel(&fine_tuning_job_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream fine tuning job cancel");
        }
    }

    local_fine_tuning_job_cancel_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    )
}

async fn fine_tuning_job_events_handler(
    request_context: StatelessGatewayRequest,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::FineTuningJobsEvents(&fine_tuning_job_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream fine tuning job events");
        }
    }

    local_fine_tuning_job_events_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    )
}

async fn fine_tuning_job_checkpoints_handler(
    request_context: StatelessGatewayRequest,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::FineTuningJobsCheckpoints(&fine_tuning_job_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream fine tuning job checkpoints",
            );
        }
    }

    local_fine_tuning_job_checkpoints_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    )
}

async fn fine_tuning_job_pause_handler(
    request_context: StatelessGatewayRequest,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::FineTuningJobsPause(&fine_tuning_job_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream fine tuning job pause");
        }
    }

    local_fine_tuning_job_pause_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    )
}

async fn fine_tuning_job_resume_handler(
    request_context: StatelessGatewayRequest,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::FineTuningJobsResume(&fine_tuning_job_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream fine tuning job resume");
        }
    }

    local_fine_tuning_job_resume_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    )
}

async fn fine_tuning_checkpoint_permissions_handler(
    request_context: StatelessGatewayRequest,
    Path(fine_tuned_model_checkpoint): Path<String>,
    ExtractJson(request): ExtractJson<CreateFineTuningCheckpointPermissionsRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::FineTuningCheckpointPermissions(&fine_tuned_model_checkpoint, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream fine tuning checkpoint permissions create",
            );
        }
    }

    local_fine_tuning_checkpoint_permissions_create_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuned_model_checkpoint,
        &request,
    )
}

async fn fine_tuning_checkpoint_permissions_list_handler(
    request_context: StatelessGatewayRequest,
    Path(fine_tuned_model_checkpoint): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::FineTuningCheckpointPermissionsList(&fine_tuned_model_checkpoint),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream fine tuning checkpoint permissions list",
            );
        }
    }

    local_fine_tuning_checkpoint_permissions_list_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuned_model_checkpoint,
    )
}

async fn fine_tuning_checkpoint_permission_delete_handler(
    request_context: StatelessGatewayRequest,
    Path((fine_tuned_model_checkpoint, permission_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::FineTuningCheckpointPermissionsDelete(
            &fine_tuned_model_checkpoint,
            &permission_id,
        ),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream fine tuning checkpoint permission delete",
            );
        }
    }

    local_fine_tuning_checkpoint_permission_delete_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuned_model_checkpoint,
        &permission_id,
    )
}


async fn fine_tuning_checkpoint_permissions_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(fine_tuned_model_checkpoint): Path<String>,
    ExtractJson(request): ExtractJson<CreateFineTuningCheckpointPermissionsRequest>,
) -> Response {
    match sdkwork_api_app_gateway::relay_fine_tuning_checkpoint_permissions_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuned_model_checkpoint,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_model = response_usage_id_or_single_data_item_id(&response)
                .unwrap_or(fine_tuned_model_checkpoint.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "fine_tuning",
                &fine_tuned_model_checkpoint,
                usage_model,
                5,
                0.005,
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
                "failed to relay upstream fine tuning checkpoint permissions create",
            );
        }
    }

    let response = match local_fine_tuning_checkpoint_permissions_create_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuned_model_checkpoint,
        &request,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };
    let usage_model = match response.data.as_slice() {
        [permission] => permission.id.as_str(),
        _ => fine_tuned_model_checkpoint.as_str(),
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "fine_tuning",
        &fine_tuned_model_checkpoint,
        usage_model,
        5,
        0.005,
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

async fn fine_tuning_checkpoint_permissions_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(fine_tuned_model_checkpoint): Path<String>,
) -> Response {
    match sdkwork_api_app_gateway::relay_list_fine_tuning_checkpoint_permissions_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuned_model_checkpoint,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "fine_tuning",
                &fine_tuned_model_checkpoint,
                4,
                0.004,
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
                "failed to relay upstream fine tuning checkpoint permissions list",
            );
        }
    }

    let response = match local_fine_tuning_checkpoint_permissions_list_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuned_model_checkpoint,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "fine_tuning",
        &fine_tuned_model_checkpoint,
        4,
        0.004,
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

async fn fine_tuning_checkpoint_permission_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((fine_tuned_model_checkpoint, permission_id)): Path<(String, String)>,
) -> Response {
    match sdkwork_api_app_gateway::relay_delete_fine_tuning_checkpoint_permission_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuned_model_checkpoint,
        &permission_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "fine_tuning",
                &fine_tuned_model_checkpoint,
                &permission_id,
                4,
                0.004,
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
                "failed to relay upstream fine tuning checkpoint permission delete",
            );
        }
    }

    let response = match local_fine_tuning_checkpoint_permission_delete_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuned_model_checkpoint,
        &permission_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "fine_tuning",
        &fine_tuned_model_checkpoint,
        &permission_id,
        4,
        0.004,
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

