async fn container_files_with_state_handler(
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
            let usage_model = response_usage_id_or_single_data_item_id(&response)
                .unwrap_or(request.file_id.as_str());
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

    let response = match local_container_file_create_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
        &request,
    ) {
        Ok(response) => response,
        Err(response) => return response,
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

async fn container_files_list_with_state_handler(
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

    let response = match local_container_files_list_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
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

async fn container_file_retrieve_with_state_handler(
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

    let response = match local_container_file_retrieve_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
        &file_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
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

async fn container_file_delete_with_state_handler(
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

    let response = match local_container_file_delete_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
        &file_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
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

async fn container_file_content_with_state_handler(
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

    let bytes = match local_container_file_content_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
        &file_id,
    ) {
        Ok(bytes) => bytes,
        Err(response) => return response,
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

    Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .body(Body::from(bytes))
        .expect("valid local container file content response")
}
