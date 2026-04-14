use super::*;

fn local_conversation_item_error_response(error: anyhow::Error) -> Response {
    let message = error.to_string();
    if message
        .to_ascii_lowercase()
        .contains("conversation item not found")
    {
        return local_gateway_invalid_or_not_found_response(
            error,
            "invalid_conversation_request",
            "Requested conversation item was not found.",
        );
    }

    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_conversation_request",
        "Requested conversation was not found.",
    )
}

pub(crate) async fn conversation_items_with_state_handler(
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
                    "upstream conversation item response missing usage id",
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

    let response = match create_conversation_items(
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_conversation_item_error_response(error),
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

pub(crate) async fn conversation_items_list_with_state_handler(
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

    let response = match list_conversation_items(
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_conversation_item_error_response(error),
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

    Json(response).into_response()
}

pub(crate) async fn conversation_item_retrieve_with_state_handler(
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

    let response = match get_conversation_item(
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
        &item_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_conversation_item_error_response(error),
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

    Json(response).into_response()
}

pub(crate) async fn conversation_item_delete_with_state_handler(
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

    let response = match delete_conversation_item(
        request_context.tenant_id(),
        request_context.project_id(),
        &conversation_id,
        &item_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_conversation_item_error_response(error),
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

    Json(response).into_response()
}
