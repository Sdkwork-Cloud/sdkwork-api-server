async fn assistants_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateAssistantRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Assistants(&request))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream assistant");
        }
    }

    Json(
        create_assistant(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.name,
            &request.model,
        )
        .expect("assistant"),
    )
    .into_response()
}

async fn assistants_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::AssistantsList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream assistants list");
        }
    }

    Json(
        list_assistants(request_context.tenant_id(), request_context.project_id())
            .expect("assistants list"),
    )
    .into_response()
}

fn local_assistant_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_error_response(error, "Requested assistant was not found.")
}

fn local_assistant_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    assistant_id: &str,
) -> std::result::Result<AssistantObject, Response> {
    get_assistant(tenant_id, project_id, assistant_id).map_err(local_assistant_not_found_response)
}

fn local_assistant_retrieve_response(
    tenant_id: &str,
    project_id: &str,
    assistant_id: &str,
) -> Response {
    match local_assistant_retrieve_result(tenant_id, project_id, assistant_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_assistant_update_result(
    tenant_id: &str,
    project_id: &str,
    assistant_id: &str,
    name: &str,
) -> std::result::Result<AssistantObject, Response> {
    update_assistant(tenant_id, project_id, assistant_id, name)
        .map_err(local_assistant_not_found_response)
}

fn local_assistant_update_response(
    tenant_id: &str,
    project_id: &str,
    assistant_id: &str,
    name: &str,
) -> Response {
    match local_assistant_update_result(tenant_id, project_id, assistant_id, name) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_assistant_delete_result(
    tenant_id: &str,
    project_id: &str,
    assistant_id: &str,
) -> std::result::Result<DeleteAssistantResponse, Response> {
    delete_assistant(tenant_id, project_id, assistant_id)
        .map_err(local_assistant_not_found_response)
}

fn local_assistant_delete_response(
    tenant_id: &str,
    project_id: &str,
    assistant_id: &str,
) -> Response {
    match local_assistant_delete_result(tenant_id, project_id, assistant_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

async fn assistant_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(assistant_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::AssistantsRetrieve(&assistant_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream assistant retrieve");
        }
    }

    local_assistant_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &assistant_id,
    )
}

async fn assistant_update_handler(
    request_context: StatelessGatewayRequest,
    Path(assistant_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateAssistantRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::AssistantsUpdate(&assistant_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream assistant update");
        }
    }

    local_assistant_update_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &assistant_id,
        request.name.as_deref().unwrap_or("assistant"),
    )
}

async fn assistant_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(assistant_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::AssistantsDelete(&assistant_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream assistant delete");
        }
    }

    local_assistant_delete_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &assistant_id,
    )
}


async fn assistants_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateAssistantRequest>,
) -> Response {
    match relay_assistant_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let assistant_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(request.model.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &request.model,
                assistant_id,
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
            return bad_gateway_openai_response("failed to relay upstream assistant");
        }
    }

    let response = create_assistant(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.name,
        &request.model,
    )
    .expect("assistant");

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &request.model,
        response.id.as_str(),
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

async fn assistants_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Response {
    match relay_list_assistants_from_store(
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
                "assistants",
                "assistants",
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
            return bad_gateway_openai_response("failed to relay upstream assistants list");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        "assistants",
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

    Json(
        list_assistants(request_context.tenant_id(), request_context.project_id())
            .expect("assistants list"),
    )
    .into_response()
}

async fn assistant_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(assistant_id): Path<String>,
) -> Response {
    match relay_get_assistant_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &assistant_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &assistant_id,
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
            return bad_gateway_openai_response("failed to relay upstream assistant retrieve");
        }
    }

    let response = match local_assistant_retrieve_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &assistant_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &assistant_id,
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

async fn assistant_update_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(assistant_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateAssistantRequest>,
) -> Response {
    match relay_update_assistant_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &assistant_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_target = request.model.as_deref().unwrap_or(assistant_id.as_str());
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                usage_target,
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
            return bad_gateway_openai_response("failed to relay upstream assistant update");
        }
    }

    let usage_target = request.model.as_deref().unwrap_or(assistant_id.as_str());
    let response = match local_assistant_update_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &assistant_id,
        request.name.as_deref().unwrap_or("assistant"),
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        usage_target,
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

async fn assistant_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(assistant_id): Path<String>,
) -> Response {
    match relay_delete_assistant_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &assistant_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &assistant_id,
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
            return bad_gateway_openai_response("failed to relay upstream assistant delete");
        }
    }

    let response = match local_assistant_delete_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &assistant_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &assistant_id,
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

