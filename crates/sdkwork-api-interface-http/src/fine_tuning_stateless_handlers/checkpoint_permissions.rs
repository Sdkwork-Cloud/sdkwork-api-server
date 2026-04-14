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

pub(crate) async fn fine_tuning_checkpoint_permissions_handler(
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

    let response = match sdkwork_api_app_gateway::create_fine_tuning_checkpoint_permissions(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuned_model_checkpoint,
        &request,
    ) {
        Ok(response) => response,
        Err(error) => return local_fine_tuning_checkpoint_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn fine_tuning_checkpoint_permissions_list_handler(
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

    let response = match sdkwork_api_app_gateway::list_fine_tuning_checkpoint_permissions(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuned_model_checkpoint,
    ) {
        Ok(response) => response,
        Err(error) => return local_fine_tuning_checkpoint_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn fine_tuning_checkpoint_permission_delete_handler(
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

    let response = match sdkwork_api_app_gateway::delete_fine_tuning_checkpoint_permission(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuned_model_checkpoint,
        &permission_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_fine_tuning_checkpoint_permission_error_response(error),
    };

    Json(response).into_response()
}
