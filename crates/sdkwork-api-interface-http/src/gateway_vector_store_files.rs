async fn vector_store_files_handler(
    request_context: StatelessGatewayRequest,
    Path(vector_store_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateVectorStoreFileRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoreFiles(&vector_store_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store file");
        }
    }

    match create_vector_store_file(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &request.file_id,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(error) => local_vector_store_file_not_found_response(error),
    }
}

async fn vector_store_files_list_handler(
    request_context: StatelessGatewayRequest,
    Path(vector_store_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoreFilesList(&vector_store_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store files list");
        }
    }

    match list_vector_store_files(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(error) => local_vector_store_file_not_found_response(error),
    }
}

async fn vector_store_file_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((vector_store_id, file_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoreFilesRetrieve(&vector_store_id, &file_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream vector store file retrieve",
            );
        }
    }

    local_vector_store_file_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &file_id,
    )
}

async fn vector_store_file_delete_handler(
    request_context: StatelessGatewayRequest,
    Path((vector_store_id, file_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoreFilesDelete(&vector_store_id, &file_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream vector store file delete",
            );
        }
    }

    local_vector_store_file_delete_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &file_id,
    )
}

async fn vector_store_file_batches_handler(
    request_context: StatelessGatewayRequest,
    Path(vector_store_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateVectorStoreFileBatchRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoreFileBatches(&vector_store_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store file batch");
        }
    }

    match create_vector_store_file_batch(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &request.file_ids,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(error) => local_vector_store_file_batch_not_found_response(error),
    }
}

async fn vector_store_file_batch_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((vector_store_id, batch_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoreFileBatchesRetrieve(&vector_store_id, &batch_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream vector store file batch retrieve",
            );
        }
    }

    local_vector_store_file_batch_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &batch_id,
    )
}

async fn vector_store_file_batch_cancel_handler(
    request_context: StatelessGatewayRequest,
    Path((vector_store_id, batch_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoreFileBatchesCancel(&vector_store_id, &batch_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream vector store file batch cancel",
            );
        }
    }

    local_vector_store_file_batch_cancel_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &batch_id,
    )
}

async fn vector_store_file_batch_files_handler(
    request_context: StatelessGatewayRequest,
    Path((vector_store_id, batch_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoreFileBatchesListFiles(&vector_store_id, &batch_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream vector store file batch files",
            );
        }
    }

    local_vector_store_file_batch_files_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &batch_id,
    )
}


fn local_vector_store_file_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_vector_store_request",
        "Requested vector store file was not found.",
    )
}

fn local_vector_store_file_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    vector_store_id: &str,
    file_id: &str,
) -> std::result::Result<VectorStoreFileObject, Response> {
    get_vector_store_file(tenant_id, project_id, vector_store_id, file_id)
        .map_err(local_vector_store_file_not_found_response)
}

fn local_vector_store_file_retrieve_response(
    tenant_id: &str,
    project_id: &str,
    vector_store_id: &str,
    file_id: &str,
) -> Response {
    match local_vector_store_file_retrieve_result(tenant_id, project_id, vector_store_id, file_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_vector_store_file_delete_result(
    tenant_id: &str,
    project_id: &str,
    vector_store_id: &str,
    file_id: &str,
) -> std::result::Result<DeleteVectorStoreFileResponse, Response> {
    delete_vector_store_file(tenant_id, project_id, vector_store_id, file_id)
        .map_err(local_vector_store_file_not_found_response)
}

fn local_vector_store_file_delete_response(
    tenant_id: &str,
    project_id: &str,
    vector_store_id: &str,
    file_id: &str,
) -> Response {
    match local_vector_store_file_delete_result(tenant_id, project_id, vector_store_id, file_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_vector_store_file_batch_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_vector_store_request",
        "Requested vector store file batch was not found.",
    )
}

fn local_vector_store_file_batch_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    vector_store_id: &str,
    batch_id: &str,
) -> std::result::Result<VectorStoreFileBatchObject, Response> {
    get_vector_store_file_batch(tenant_id, project_id, vector_store_id, batch_id)
        .map_err(local_vector_store_file_batch_not_found_response)
}

fn local_vector_store_file_batch_retrieve_response(
    tenant_id: &str,
    project_id: &str,
    vector_store_id: &str,
    batch_id: &str,
) -> Response {
    match local_vector_store_file_batch_retrieve_result(
        tenant_id,
        project_id,
        vector_store_id,
        batch_id,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_vector_store_file_batch_cancel_result(
    tenant_id: &str,
    project_id: &str,
    vector_store_id: &str,
    batch_id: &str,
) -> std::result::Result<VectorStoreFileBatchObject, Response> {
    cancel_vector_store_file_batch(tenant_id, project_id, vector_store_id, batch_id)
        .map_err(local_vector_store_file_batch_not_found_response)
}

fn local_vector_store_file_batch_cancel_response(
    tenant_id: &str,
    project_id: &str,
    vector_store_id: &str,
    batch_id: &str,
) -> Response {
    match local_vector_store_file_batch_cancel_result(
        tenant_id,
        project_id,
        vector_store_id,
        batch_id,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_vector_store_file_batch_files_result(
    tenant_id: &str,
    project_id: &str,
    vector_store_id: &str,
    batch_id: &str,
) -> std::result::Result<ListVectorStoreFilesResponse, Response> {
    list_vector_store_file_batch_files(tenant_id, project_id, vector_store_id, batch_id)
        .map_err(local_vector_store_file_batch_not_found_response)
}

fn local_vector_store_file_batch_files_response(
    tenant_id: &str,
    project_id: &str,
    vector_store_id: &str,
    batch_id: &str,
) -> Response {
    match local_vector_store_file_batch_files_result(
        tenant_id,
        project_id,
        vector_store_id,
        batch_id,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}


async fn vector_store_files_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(vector_store_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateVectorStoreFileRequest>,
) -> Response {
    match relay_vector_store_file_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let Some(usage_model) = response_usage_id_or_single_data_item_id(&response) else {
                return bad_gateway_openai_response(
                    "upstream vector store file response missing usage id",
                );
            };
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "vector_store_files",
                &vector_store_id,
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
            return bad_gateway_openai_response("failed to relay upstream vector store file");
        }
    }

    let response = match create_vector_store_file(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &request.file_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_vector_store_file_not_found_response(error),
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_store_files",
        &vector_store_id,
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

async fn vector_store_files_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(vector_store_id): Path<String>,
) -> Response {
    match relay_list_vector_store_files_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "vector_store_files",
                &vector_store_id,
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
            return bad_gateway_openai_response("failed to relay upstream vector store files list");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_store_files",
        &vector_store_id,
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

    let response = match list_vector_store_files(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_vector_store_file_not_found_response(error),
    };

    Json(response).into_response()
}

async fn vector_store_file_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((vector_store_id, file_id)): Path<(String, String)>,
) -> Response {
    match relay_get_vector_store_file_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &file_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "vector_store_files",
                &vector_store_id,
                &file_id,
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
                "failed to relay upstream vector store file retrieve",
            );
        }
    }

    let response = match local_vector_store_file_retrieve_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &file_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_store_files",
        &vector_store_id,
        &file_id,
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

async fn vector_store_file_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((vector_store_id, file_id)): Path<(String, String)>,
) -> Response {
    match relay_delete_vector_store_file_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &file_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "vector_store_files",
                &vector_store_id,
                &file_id,
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
                "failed to relay upstream vector store file delete",
            );
        }
    }

    let response = match local_vector_store_file_delete_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &file_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_store_files",
        &vector_store_id,
        &file_id,
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

async fn vector_store_file_batches_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(vector_store_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateVectorStoreFileBatchRequest>,
) -> Response {
    match relay_vector_store_file_batch_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let Some(batch_id) = response.get("id").and_then(Value::as_str) else {
                return bad_gateway_openai_response(
                    "upstream vector store file batch response missing id",
                );
            };
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "vector_store_file_batches",
                &vector_store_id,
                batch_id,
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
            return bad_gateway_openai_response("failed to relay upstream vector store file batch");
        }
    }

    let response = match create_vector_store_file_batch(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &request.file_ids,
    ) {
        Ok(response) => response,
        Err(error) => return local_vector_store_file_batch_not_found_response(error),
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_store_file_batches",
        &vector_store_id,
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

async fn vector_store_file_batch_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((vector_store_id, batch_id)): Path<(String, String)>,
) -> Response {
    match relay_get_vector_store_file_batch_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &batch_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "vector_store_file_batches",
                &vector_store_id,
                &batch_id,
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
                "failed to relay upstream vector store file batch retrieve",
            );
        }
    }

    let response = match local_vector_store_file_batch_retrieve_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &batch_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_store_file_batches",
        &vector_store_id,
        &batch_id,
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

async fn vector_store_file_batch_cancel_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((vector_store_id, batch_id)): Path<(String, String)>,
) -> Response {
    match relay_cancel_vector_store_file_batch_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &batch_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "vector_store_file_batches",
                &vector_store_id,
                &batch_id,
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
                "failed to relay upstream vector store file batch cancel",
            );
        }
    }

    let response = match local_vector_store_file_batch_cancel_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &batch_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_store_file_batches",
        &vector_store_id,
        &batch_id,
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

async fn vector_store_file_batch_files_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((vector_store_id, batch_id)): Path<(String, String)>,
) -> Response {
    match relay_list_vector_store_file_batch_files_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &batch_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "vector_store_file_batches",
                &vector_store_id,
                &batch_id,
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
                "failed to relay upstream vector store file batch files",
            );
        }
    }

    let response = match local_vector_store_file_batch_files_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &batch_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_store_file_batches",
        &vector_store_id,
        &batch_id,
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
