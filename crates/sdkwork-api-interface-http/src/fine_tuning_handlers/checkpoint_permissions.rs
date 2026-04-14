use super::*;

fn local_fine_tuning_checkpoint_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_fine_tuning_request",
        "Requested fine tuning checkpoint was not found.",
    )
}

fn local_fine_tuning_checkpoint_permission_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_fine_tuning_request",
        "Requested fine tuning checkpoint permission was not found.",
    )
}

pub(crate) async fn fine_tuning_checkpoint_permissions_with_state_handler(
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
            let Some(usage_model) = response_usage_id_or_single_data_item_id(&response) else {
                return bad_gateway_openai_response(
                    "upstream fine tuning checkpoint permission response missing usage id",
                );
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

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream fine tuning checkpoint permissions create",
            );
        }
    }

    let response = match sdkwork_api_app_gateway::create_fine_tuning_checkpoint_permissions(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuned_model_checkpoint,
        &request,
    ) {
        Ok(response) => response,
        Err(error) => return local_fine_tuning_checkpoint_error_response(error),
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

pub(crate) async fn fine_tuning_checkpoint_permissions_list_with_state_handler(
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

    let response = match sdkwork_api_app_gateway::list_fine_tuning_checkpoint_permissions(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuned_model_checkpoint,
    ) {
        Ok(response) => response,
        Err(error) => return local_fine_tuning_checkpoint_error_response(error),
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

pub(crate) async fn fine_tuning_checkpoint_permission_delete_with_state_handler(
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

    let response = match sdkwork_api_app_gateway::delete_fine_tuning_checkpoint_permission(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuned_model_checkpoint,
        &permission_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_fine_tuning_checkpoint_permission_error_response(error),
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
