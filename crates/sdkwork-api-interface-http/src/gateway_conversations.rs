fn local_conversation_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_conversation_request",
        "Requested conversation was not found.",
    )
}

async fn conversations_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateConversationRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Conversations(&request))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream conversation");
        }
    }

    let response = match create_conversation(request_context.tenant_id(), request_context.project_id())
    {
        Ok(response) => response,
        Err(error) => return local_conversation_error_response(error),
    };

    Json(response).into_response()
}

async fn conversations_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::ConversationsList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream conversation list");
        }
    }

    let response = match list_conversations(request_context.tenant_id(), request_context.project_id())
    {
        Ok(response) => response,
        Err(error) => return local_conversation_error_response(error),
    };

    Json(response).into_response()
}

async fn conversation_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(conversation_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ConversationsRetrieve(&conversation_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream conversation retrieve");
        }
    }

    local_conversation_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
    )
}

async fn conversation_update_handler(
    request_context: StatelessGatewayRequest,
    Path(conversation_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateConversationRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ConversationsUpdate(&conversation_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream conversation update");
        }
    }

    local_conversation_update_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
        request.metadata,
    )
}

async fn conversation_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(conversation_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ConversationsDelete(&conversation_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream conversation delete");
        }
    }

    local_conversation_delete_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
    )
}

async fn conversation_items_handler(
    request_context: StatelessGatewayRequest,
    Path(conversation_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateConversationItemsRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ConversationItems(&conversation_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream conversation items");
        }
    }

    local_conversation_items_create_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
    )
}

async fn conversation_items_list_handler(
    request_context: StatelessGatewayRequest,
    Path(conversation_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ConversationItemsList(&conversation_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream conversation items list");
        }
    }

    local_conversation_items_list_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
    )
}

async fn conversation_item_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((conversation_id, item_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ConversationItemsRetrieve(&conversation_id, &item_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream conversation item retrieve",
            );
        }
    }

    local_conversation_item_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
        &item_id,
    )
}

async fn conversation_item_delete_handler(
    request_context: StatelessGatewayRequest,
    Path((conversation_id, item_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ConversationItemsDelete(&conversation_id, &item_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream conversation item delete",
            );
        }
    }

    local_conversation_item_delete_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
        &item_id,
    )
}


async fn conversations_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateConversationRequest>,
) -> Response {
    match relay_conversation_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let Some(conversation_id) = response.get("id").and_then(Value::as_str) else {
                return bad_gateway_openai_response("upstream conversation response missing id");
            };
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "responses",
                "conversations",
                conversation_id,
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
            return bad_gateway_openai_response("failed to relay upstream conversation");
        }
    }

    let response = match create_conversation(request_context.tenant_id(), request_context.project_id())
    {
        Ok(response) => response,
        Err(error) => return local_conversation_error_response(error),
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        "conversations",
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

async fn conversations_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Response {
    match relay_list_conversations_from_store(
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
                "responses",
                "conversations",
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
            return bad_gateway_openai_response("failed to relay upstream conversation list");
        }
    }

    let response = match list_conversations(request_context.tenant_id(), request_context.project_id())
    {
        Ok(response) => response,
        Err(error) => return local_conversation_error_response(error),
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        "conversations",
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

async fn conversation_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(conversation_id): Path<String>,
) -> Response {
    match relay_get_conversation_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
    )
    .await
    {
        Ok(Some(response)) => {
            let Some(usage_model) = response_usage_id_or_single_data_item_id(&response) else {
                return bad_gateway_openai_response(
                    "upstream conversation response missing usage id",
                );
            };
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "responses",
                &conversation_id,
                usage_model,
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
            return bad_gateway_openai_response("failed to relay upstream conversation retrieve");
        }
    }

    let local_response = match local_conversation_retrieve_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        &conversation_id,
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

    Json(local_response).into_response()
}

async fn conversation_update_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(conversation_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateConversationRequest>,
) -> Response {
    match relay_update_conversation_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let Some(usage_model) = response_usage_id_or_single_data_item_id(&response) else {
                return bad_gateway_openai_response(
                    "upstream conversation response missing usage id",
                );
            };
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "responses",
                &conversation_id,
                usage_model,
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
            return bad_gateway_openai_response("failed to relay upstream conversation update");
        }
    }

    let local_response = match local_conversation_update_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
        request.metadata,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        &conversation_id,
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

    Json(local_response).into_response()
}

async fn conversation_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(conversation_id): Path<String>,
) -> Response {
    match relay_delete_conversation_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
    )
    .await
    {
        Ok(Some(response)) => {
            let Some(usage_model) = response_usage_id_or_single_data_item_id(&response) else {
                return bad_gateway_openai_response(
                    "upstream conversation response missing usage id",
                );
            };
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "responses",
                &conversation_id,
                usage_model,
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
            return bad_gateway_openai_response("failed to relay upstream conversation delete");
        }
    }

    let local_response = match local_conversation_delete_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        &conversation_id,
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

    Json(local_response).into_response()
}

async fn conversation_items_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(conversation_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateConversationItemsRequest>,
) -> Response {
    match relay_conversation_items_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let Some(usage_model) = response_usage_id_or_single_data_item_id(&response) else {
                return bad_gateway_openai_response(
                    "upstream conversation response missing usage id",
                );
            };
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "responses",
                &conversation_id,
                usage_model,
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
            return bad_gateway_openai_response("failed to relay upstream conversation items");
        }
    }

    let response = match local_conversation_items_create_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };
    let usage_model = match response.data.as_slice() {
        [item] => item.id.as_str(),
        _ => conversation_id.as_str(),
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        &conversation_id,
        usage_model,
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

async fn conversation_items_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(conversation_id): Path<String>,
) -> Response {
    match relay_list_conversation_items_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "responses",
                &conversation_id,
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
            return bad_gateway_openai_response("failed to relay upstream conversation items list");
        }
    }

    let local_response = match local_conversation_items_list_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        &conversation_id,
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

    Json(local_response).into_response()
}

async fn conversation_item_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((conversation_id, item_id)): Path<(String, String)>,
) -> Response {
    match relay_get_conversation_item_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
        &item_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "responses",
                &conversation_id,
                &item_id,
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
                "failed to relay upstream conversation item retrieve",
            );
        }
    }

    let local_response = match local_conversation_item_retrieve_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
        &item_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        &conversation_id,
        &item_id,
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

    Json(local_response).into_response()
}

async fn conversation_item_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((conversation_id, item_id)): Path<(String, String)>,
) -> Response {
    match relay_delete_conversation_item_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
        &item_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "responses",
                &conversation_id,
                &item_id,
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
                "failed to relay upstream conversation item delete",
            );
        }
    }

    let local_response = match local_conversation_item_delete_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
        &item_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "responses",
        &conversation_id,
        &item_id,
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

    Json(local_response).into_response()
}

