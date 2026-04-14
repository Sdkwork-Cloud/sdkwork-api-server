use super::*;

fn local_container_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_container_request",
        "Requested container was not found.",
    )
}

fn local_container_file_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_container_request",
        "Requested container file was not found.",
    )
}

pub(crate) async fn container_files_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(container_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateContainerFileRequest>,
) -> Response {
    match sdkwork_api_app_gateway::relay_container_file_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let Some(usage_model) = response_usage_id_or_single_data_item_id(&response) else {
                return bad_gateway_openai_response(
                    "upstream container file response missing usage id",
                );
            };
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "containers",
                &container_id,
                usage_model,
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
            return bad_gateway_openai_response("failed to relay upstream container file create");
        }
    }

    let response = match sdkwork_api_app_gateway::create_container_file(
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
        &request,
    ) {
        Ok(response) => response,
        Err(error) => return local_container_file_error_response(error),
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "containers",
        &container_id,
        response.id.as_str(),
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

pub(crate) async fn container_files_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(container_id): Path<String>,
) -> Response {
    match sdkwork_api_app_gateway::relay_list_container_files_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "containers",
                &container_id,
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
            return bad_gateway_openai_response("failed to relay upstream container files list");
        }
    }

    let response = match sdkwork_api_app_gateway::list_container_files(
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_container_error_response(error),
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "containers",
        &container_id,
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

pub(crate) async fn container_file_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((container_id, file_id)): Path<(String, String)>,
) -> Response {
    match sdkwork_api_app_gateway::relay_get_container_file_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
        &file_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "containers",
                &container_id,
                &file_id,
                3,
                0.003,
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
            return bad_gateway_openai_response("failed to relay upstream container file retrieve");
        }
    }

    let response = match sdkwork_api_app_gateway::get_container_file(
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
        &file_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_container_file_error_response(error),
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "containers",
        &container_id,
        &file_id,
        3,
        0.003,
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

pub(crate) async fn container_file_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((container_id, file_id)): Path<(String, String)>,
) -> Response {
    match sdkwork_api_app_gateway::relay_delete_container_file_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
        &file_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "containers",
                &container_id,
                &file_id,
                3,
                0.003,
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
            return bad_gateway_openai_response("failed to relay upstream container file delete");
        }
    }

    let response = match sdkwork_api_app_gateway::delete_container_file(
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
        &file_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_container_file_error_response(error),
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "containers",
        &container_id,
        &file_id,
        3,
        0.003,
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

pub(crate) async fn container_file_content_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((container_id, file_id)): Path<(String, String)>,
) -> Response {
    match sdkwork_api_app_gateway::relay_container_file_content_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
        &file_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "containers",
                &container_id,
                &file_id,
                3,
                0.003,
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

            return upstream_passthrough_response(response);
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream container file content");
        }
    }

    let response = local_container_file_content_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
        &file_id,
    );
    if !response.status().is_success() {
        return response;
    }

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "containers",
        &container_id,
        &file_id,
        3,
        0.003,
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

    response
}
