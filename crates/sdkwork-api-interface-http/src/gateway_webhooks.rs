fn local_webhook_list_error_response() -> Response {
    invalid_request_openai_response(
        "Local webhook listing fallback is not supported without an upstream provider.",
        "invalid_webhook_request",
    )
}

async fn webhooks_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateWebhookRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Webhooks(&request)).await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream webhook");
        }
    }

    match create_webhook(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.url,
        &request.events,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(error) => local_gateway_invalid_or_bad_gateway_response(error, "invalid_webhook_request"),
    }
}

async fn webhooks_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::WebhooksList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream webhooks list");
        }
    }

    let response = match list_webhooks(request_context.tenant_id(), request_context.project_id()) {
        Ok(response) => response,
        Err(_) => return local_webhook_list_error_response(),
    };

    Json(response).into_response()
}

fn local_webhook_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_webhook_request",
        "Requested webhook was not found.",
    )
}

fn local_webhook_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    webhook_id: &str,
) -> std::result::Result<WebhookObject, Response> {
    get_webhook(tenant_id, project_id, webhook_id).map_err(local_webhook_not_found_response)
}

fn local_webhook_retrieve_response(
    tenant_id: &str,
    project_id: &str,
    webhook_id: &str,
) -> Response {
    match local_webhook_retrieve_result(tenant_id, project_id, webhook_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_webhook_update_result(
    tenant_id: &str,
    project_id: &str,
    webhook_id: &str,
    url: &str,
) -> std::result::Result<WebhookObject, Response> {
    update_webhook(tenant_id, project_id, webhook_id, url).map_err(local_webhook_not_found_response)
}

fn local_webhook_update_response(
    tenant_id: &str,
    project_id: &str,
    webhook_id: &str,
    url: &str,
) -> Response {
    match local_webhook_update_result(tenant_id, project_id, webhook_id, url) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_webhook_update_url<'a>(request: &'a UpdateWebhookRequest) -> Result<&'a str, Response> {
    request.url.as_deref().ok_or_else(|| {
        invalid_request_openai_response(
            "Webhook url is required for local fallback updates.",
            "invalid_webhook_request",
        )
    })
}

fn local_webhook_delete_result(
    tenant_id: &str,
    project_id: &str,
    webhook_id: &str,
) -> std::result::Result<DeleteWebhookResponse, Response> {
    delete_webhook(tenant_id, project_id, webhook_id).map_err(local_webhook_not_found_response)
}

fn local_webhook_delete_response(tenant_id: &str, project_id: &str, webhook_id: &str) -> Response {
    match local_webhook_delete_result(tenant_id, project_id, webhook_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

async fn webhook_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(webhook_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::WebhooksRetrieve(&webhook_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream webhook retrieve");
        }
    }

    local_webhook_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &webhook_id,
    )
}

async fn webhook_update_handler(
    request_context: StatelessGatewayRequest,
    Path(webhook_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateWebhookRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::WebhooksUpdate(&webhook_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream webhook update");
        }
    }

    let webhook_url = match local_webhook_update_url(&request) {
        Ok(url) => url,
        Err(response) => return response,
    };

    local_webhook_update_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &webhook_id,
        webhook_url,
    )
}

async fn webhook_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(webhook_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::WebhooksDelete(&webhook_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream webhook delete");
        }
    }

    local_webhook_delete_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &webhook_id,
    )
}


async fn webhooks_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateWebhookRequest>,
) -> Response {
    match relay_webhook_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let Some(webhook_id) = response.get("id").and_then(Value::as_str) else {
                return bad_gateway_openai_response("upstream webhook response missing id");
            };
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "webhooks",
                &request.url,
                webhook_id,
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
            return bad_gateway_openai_response("failed to relay upstream webhook");
        }
    }

    let response = match create_webhook(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.url,
        &request.events,
    ) {
        Ok(response) => response,
        Err(error) => {
            return local_gateway_invalid_or_bad_gateway_response(
                error,
                "invalid_webhook_request",
            );
        }
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "webhooks",
        &request.url,
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

async fn webhooks_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Response {
    match relay_list_webhooks_from_store(
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
                "webhooks",
                "webhooks",
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
            return bad_gateway_openai_response("failed to relay upstream webhooks list");
        }
    }

    let response = match list_webhooks(request_context.tenant_id(), request_context.project_id()) {
        Ok(response) => response,
        Err(_) => return local_webhook_list_error_response(),
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "webhooks",
        "webhooks",
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

async fn webhook_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(webhook_id): Path<String>,
) -> Response {
    match relay_get_webhook_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &webhook_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "webhooks",
                &webhook_id,
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
            return bad_gateway_openai_response("failed to relay upstream webhook retrieve");
        }
    }

    let response = match local_webhook_retrieve_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &webhook_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "webhooks",
        &webhook_id,
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

async fn webhook_update_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(webhook_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateWebhookRequest>,
) -> Response {
    match relay_update_webhook_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &webhook_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_target = request.url.as_deref().unwrap_or(webhook_id.as_str());
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "webhooks",
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
            return bad_gateway_openai_response("failed to relay upstream webhook update");
        }
    }

    let usage_target = request.url.as_deref().unwrap_or(webhook_id.as_str());
    let webhook_url = match local_webhook_update_url(&request) {
        Ok(url) => url,
        Err(response) => return response,
    };
    let response = match local_webhook_update_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &webhook_id,
        webhook_url,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "webhooks",
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

async fn webhook_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(webhook_id): Path<String>,
) -> Response {
    match relay_delete_webhook_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &webhook_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "webhooks",
                &webhook_id,
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
            return bad_gateway_openai_response("failed to relay upstream webhook delete");
        }
    }

    let response = match local_webhook_delete_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &webhook_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "webhooks",
        &webhook_id,
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

