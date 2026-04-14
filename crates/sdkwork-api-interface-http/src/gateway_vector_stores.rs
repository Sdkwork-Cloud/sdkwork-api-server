async fn vector_stores_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::VectorStoresList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector stores list");
        }
    }

    match list_vector_stores(request_context.tenant_id(), request_context.project_id()) {
        Ok(response) => Json(response).into_response(),
        Err(error) => local_vector_store_not_found_response(error),
    }
}

async fn vector_stores_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateVectorStoreRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::VectorStores(&request))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store");
        }
    }

    match create_vector_store(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.name,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(error) => local_vector_store_not_found_response(error),
    }
}

fn local_vector_store_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_vector_store_request",
        "Requested vector store was not found.",
    )
}

fn local_vector_store_update_name<'a>(
    request: &'a UpdateVectorStoreRequest,
) -> Result<&'a str, Response> {
    request.name.as_deref().ok_or_else(|| {
        invalid_request_openai_response(
            "Vector store name is required for local fallback updates.",
            "invalid_vector_store_request",
        )
    })
}

fn local_vector_store_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    vector_store_id: &str,
) -> std::result::Result<VectorStoreObject, Response> {
    get_vector_store(tenant_id, project_id, vector_store_id)
        .map_err(local_vector_store_not_found_response)
}

fn local_vector_store_retrieve_response(
    tenant_id: &str,
    project_id: &str,
    vector_store_id: &str,
) -> Response {
    match local_vector_store_retrieve_result(tenant_id, project_id, vector_store_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_vector_store_update_result(
    tenant_id: &str,
    project_id: &str,
    vector_store_id: &str,
    name: &str,
) -> std::result::Result<VectorStoreObject, Response> {
    update_vector_store(tenant_id, project_id, vector_store_id, name)
        .map_err(local_vector_store_not_found_response)
}

fn local_vector_store_update_response(
    tenant_id: &str,
    project_id: &str,
    vector_store_id: &str,
    name: &str,
) -> Response {
    match local_vector_store_update_result(tenant_id, project_id, vector_store_id, name) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_vector_store_delete_result(
    tenant_id: &str,
    project_id: &str,
    vector_store_id: &str,
) -> std::result::Result<DeleteVectorStoreResponse, Response> {
    delete_vector_store(tenant_id, project_id, vector_store_id)
        .map_err(local_vector_store_not_found_response)
}

fn local_vector_store_delete_response(
    tenant_id: &str,
    project_id: &str,
    vector_store_id: &str,
) -> Response {
    match local_vector_store_delete_result(tenant_id, project_id, vector_store_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_vector_store_search_result(
    tenant_id: &str,
    project_id: &str,
    vector_store_id: &str,
    query: &str,
) -> std::result::Result<SearchVectorStoreResponse, Response> {
    search_vector_store(tenant_id, project_id, vector_store_id, query)
        .map_err(local_vector_store_not_found_response)
}

fn local_vector_store_search_response(
    tenant_id: &str,
    project_id: &str,
    vector_store_id: &str,
    query: &str,
) -> Response {
    match local_vector_store_search_result(tenant_id, project_id, vector_store_id, query) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

async fn vector_store_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(vector_store_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoresRetrieve(&vector_store_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store retrieve");
        }
    }

    local_vector_store_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
    )
}

async fn vector_store_update_handler(
    request_context: StatelessGatewayRequest,
    Path(vector_store_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateVectorStoreRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoresUpdate(&vector_store_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store update");
        }
    }

    local_vector_store_update_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        match local_vector_store_update_name(&request) {
            Ok(name) => name,
            Err(response) => return response,
        },
    )
}

async fn vector_store_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(vector_store_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoresDelete(&vector_store_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store delete");
        }
    }

    local_vector_store_delete_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
    )
}

async fn vector_store_search_handler(
    request_context: StatelessGatewayRequest,
    Path(vector_store_id): Path<String>,
    ExtractJson(request): ExtractJson<SearchVectorStoreRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoresSearch(&vector_store_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store search");
        }
    }

    local_vector_store_search_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &request.query,
    )
}

async fn vector_stores_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateVectorStoreRequest>,
) -> Response {
    match relay_vector_store_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let Some(vector_store_id) = response.get("id").and_then(Value::as_str) else {
                return bad_gateway_openai_response("upstream vector store response missing id");
            };
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "vector_stores",
                &request.name,
                vector_store_id,
                35,
                0.035,
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
            return bad_gateway_openai_response("failed to relay upstream vector store");
        }
    }

    let response = match create_vector_store(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.name,
    ) {
        Ok(response) => response,
        Err(error) => return local_vector_store_not_found_response(error),
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_stores",
        &request.name,
        response.id.as_str(),
        35,
        0.035,
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

async fn vector_stores_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Response {
    match relay_list_vector_stores_from_store(
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
                "vector_stores",
                "vector_stores",
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
            return bad_gateway_openai_response("failed to relay upstream vector stores list");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_stores",
        "vector_stores",
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

    match list_vector_stores(request_context.tenant_id(), request_context.project_id()) {
        Ok(response) => Json(response).into_response(),
        Err(error) => local_vector_store_not_found_response(error),
    }
}

async fn vector_store_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(vector_store_id): Path<String>,
) -> Response {
    match relay_get_vector_store_from_store(
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
                "vector_stores",
                &vector_store_id,
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
            return bad_gateway_openai_response("failed to relay upstream vector store retrieve");
        }
    }

    let response = match local_vector_store_retrieve_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_stores",
        &vector_store_id,
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

async fn vector_store_update_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(vector_store_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateVectorStoreRequest>,
) -> Response {
    match relay_update_vector_store_from_store(
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
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "vector_stores",
                &vector_store_id,
                35,
                0.035,
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
            return bad_gateway_openai_response("failed to relay upstream vector store update");
        }
    }

    let response = match local_vector_store_update_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        match local_vector_store_update_name(&request) {
            Ok(name) => name,
            Err(response) => return response,
        },
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_stores",
        &vector_store_id,
        35,
        0.035,
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

async fn vector_store_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(vector_store_id): Path<String>,
) -> Response {
    match relay_delete_vector_store_from_store(
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
                "vector_stores",
                &vector_store_id,
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
            return bad_gateway_openai_response("failed to relay upstream vector store delete");
        }
    }

    let response = match local_vector_store_delete_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_stores",
        &vector_store_id,
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

async fn vector_store_search_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(vector_store_id): Path<String>,
    ExtractJson(request): ExtractJson<SearchVectorStoreRequest>,
) -> Response {
    match relay_search_vector_store_from_store(
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
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "vector_store_search",
                &vector_store_id,
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
            return bad_gateway_openai_response("failed to relay upstream vector store search");
        }
    }

    let response = match local_vector_store_search_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &request.query,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_store_search",
        &vector_store_id,
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
