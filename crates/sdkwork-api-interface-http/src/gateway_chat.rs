async fn chat_completions_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateChatCompletionRequest>,
) -> Response {
    if request.stream.unwrap_or(false) {
        match relay_stateless_stream_request(
            &request_context,
            ProviderRequest::ChatCompletionsStream(&request),
        )
        .await
        {
            Ok(Some(response)) => return upstream_passthrough_response(response),
            Ok(None) => {}
            Err(_) => {
                return bad_gateway_openai_response(
                    "failed to relay upstream chat completion stream",
                );
            }
        }

        return local_chat_completion_stream_response(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        );
    }

    match relay_stateless_json_request(&request_context, ProviderRequest::ChatCompletions(&request))
        .await
    {
        Ok(Some(response)) => Json(response).into_response(),
        Ok(None) => local_chat_completion_response(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        ),
        Err(_) => bad_gateway_openai_response("failed to relay upstream chat completion"),
    }
}

async fn chat_completions_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::ChatCompletionsList).await
    {
        Ok(Some(response)) => Json(response).into_response(),
        Ok(None) => Json(
            list_chat_completions(request_context.tenant_id(), request_context.project_id())
                .expect("chat completions"),
        )
        .into_response(),
        Err(_) => bad_gateway_openai_response("failed to relay upstream chat completion list"),
    }
}

async fn chat_completion_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(completion_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ChatCompletionsRetrieve(&completion_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream chat completion retrieve",
            );
        }
    }

    local_chat_completion_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &completion_id,
    )
}

async fn chat_completion_update_handler(
    request_context: StatelessGatewayRequest,
    Path(completion_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateChatCompletionRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ChatCompletionsUpdate(&completion_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream chat completion update");
        }
    }

    local_chat_completion_update_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &completion_id,
        request.metadata.unwrap_or(serde_json::json!({})),
    )
}

async fn chat_completion_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(completion_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ChatCompletionsDelete(&completion_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream chat completion delete");
        }
    }

    local_chat_completion_delete_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &completion_id,
    )
}

async fn chat_completion_messages_list_handler(
    request_context: StatelessGatewayRequest,
    Path(completion_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ChatCompletionsMessagesList(&completion_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream chat completion messages",
            );
        }
    }

    local_chat_completion_messages_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &completion_id,
    )
}


async fn chat_completions_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateChatCompletionRequest>,
) -> Response {
    let options = ProviderRequestOptions::default();
    let commercial_admission = match begin_gateway_commercial_admission(
        &state,
        request_context.context(),
        GatewayCommercialAdmissionSpec {
            quoted_amount: 0.10,
        },
    )
    .await
    {
        Ok(GatewayCommercialAdmissionDecision::Canonical(admission)) => Some(admission),
        Ok(GatewayCommercialAdmissionDecision::LegacyQuota) => {
            match enforce_project_quota(state.store.as_ref(), request_context.project_id(), 100)
                .await
            {
                Ok(Some(response)) => return response,
                Ok(None) => {}
                Err(_) => {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to evaluate quota",
                    )
                        .into_response();
                }
            }
            None
        }
        Err(response) => return response,
    };

    if request.stream.unwrap_or(false) {
        match relay_chat_completion_stream_from_store_with_execution_context(
            state.store.as_ref(),
            &state.secret_manager,
            request_context.tenant_id(),
            request_context.project_id(),
            &request,
            &options,
        )
        .await
        {
            Ok(execution) => {
                let usage_context = execution.usage_context;
                if let Some(response) = execution.response {
                    if let Some(admission) = commercial_admission.as_ref() {
                        if let Err(response) =
                            capture_gateway_commercial_admission(&state, admission).await
                        {
                            return response;
                        }
                    }
                    let usage_result = record_gateway_usage_for_project_with_context(
                        state.store.as_ref(),
                        request_context.tenant_id(),
                        request_context.project_id(),
                        "chat_completion",
                        &request.model,
                        100,
                        0.10,
                        usage_context.as_ref(),
                    )
                    .await;
                    if usage_result.is_err() {
                        return (
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            "failed to record usage",
                        )
                            .into_response();
                    }

                    return upstream_passthrough_response(response);
                }
            }
            Err(_) => {
                if let Some(admission) = commercial_admission.as_ref() {
                    if let Err(response) =
                        release_gateway_commercial_admission(&state, admission).await
                    {
                        return response;
                    }
                }
                return bad_gateway_openai_response(
                    "failed to relay upstream chat completion stream",
                );
            }
        }
    } else {
        match relay_chat_completion_from_store_with_execution_context(
            state.store.as_ref(),
            &state.secret_manager,
            request_context.tenant_id(),
            request_context.project_id(),
            &request,
            &options,
        )
        .await
        {
            Ok(execution) => {
                let usage_context = execution.usage_context;
                if let Some(response) = execution.response {
                    if let Some(admission) = commercial_admission.as_ref() {
                        if let Err(response) =
                            capture_gateway_commercial_admission(&state, admission).await
                        {
                            return response;
                        }
                    }
                    let token_usage = extract_token_usage_metrics(&response);
                    let usage_result =
                        record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context(
                        state.store.as_ref(),
                        request_context.tenant_id(),
                        request_context.project_id(),
                        "chat_completion",
                        &request.model,
                        &request.model,
                        100,
                        0.10,
                        token_usage,
                        response_usage_id_or_single_data_item_id(&response),
                        usage_context.as_ref(),
                    )
                    .await;
                    if usage_result.is_err() {
                        return (
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            "failed to record usage",
                        )
                            .into_response();
                    }

                    return Json(response).into_response();
                }
            }
            Err(_) => {
                if let Some(admission) = commercial_admission.as_ref() {
                    if let Err(response) =
                        release_gateway_commercial_admission(&state, admission).await
                    {
                        return response;
                    }
                }
                return bad_gateway_openai_response("failed to relay upstream chat completion");
            }
        }
    }

    let local_chat_completion = match local_chat_completion_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if let Some(admission) = commercial_admission.as_ref() {
        if let Err(response) = capture_gateway_commercial_admission(&state, admission).await {
            return response;
        }
    }

    let usage_result = record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "chat_completion",
        &request.model,
        100,
        0.10,
    )
    .await;
    if usage_result.is_err() {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    if request.stream.unwrap_or(false) {
        local_chat_completion_stream_body_response()
    } else {
        Json(local_chat_completion).into_response()
    }
}

async fn anthropic_messages_handler(
    request_context: StatelessGatewayRequest,
    headers: HeaderMap,
    ExtractJson(payload): ExtractJson<Value>,
) -> Response {
    let request = match anthropic_request_to_chat_completion(&payload) {
        Ok(request) => request,
        Err(error) => return anthropic_invalid_request_response(error.to_string()),
    };
    let options = anthropic_request_options(&headers);

    if request.stream.unwrap_or(false) {
        match relay_stateless_stream_request_with_options(
            &request_context,
            ProviderRequest::ChatCompletionsStream(&request),
            &options,
        )
        .await
        {
            Ok(Some(response)) => {
                return upstream_passthrough_response(anthropic_stream_from_openai(response));
            }
            Ok(None) => {
                return match local_anthropic_stream_result(
                    request_context.tenant_id(),
                    request_context.project_id(),
                    &request.model,
                ) {
                    Ok(response) | Err(response) => response,
                };
            }
            Err(_) => {
                return anthropic_bad_gateway_response(
                    "failed to relay upstream anthropic message stream",
                );
            }
        }
    }

    match relay_stateless_json_request_with_options(
        &request_context,
        ProviderRequest::ChatCompletions(&request),
        &options,
    )
    .await
    {
        Ok(Some(response)) => Json(openai_chat_response_to_anthropic(&response)).into_response(),
        Ok(None) => match local_anthropic_chat_completion_result(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        ) {
            Ok(response) => Json(response).into_response(),
            Err(response) => response,
        },
        Err(_) => anthropic_bad_gateway_response("failed to relay upstream anthropic message"),
    }
}

async fn anthropic_count_tokens_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(payload): ExtractJson<Value>,
) -> Response {
    let request = match anthropic_count_tokens_request(&payload) {
        Ok(request) => request,
        Err(error) => return anthropic_invalid_request_response(error.to_string()),
    };

    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ResponsesInputTokens(&request),
    )
    .await
    {
        Ok(Some(response)) => Json(openai_count_tokens_to_anthropic(&response)).into_response(),
        Ok(None) => local_anthropic_count_tokens_response(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        ),
        Err(_) => anthropic_bad_gateway_response(
            "failed to relay upstream anthropic count tokens request",
        ),
    }
}


async fn chat_completions_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Response {
    match relay_list_chat_completions_from_store(
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
                "chat_completion",
                "chat_completions",
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
            return bad_gateway_openai_response("failed to relay upstream chat completion list");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "chat_completion",
        "chat_completions",
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
        list_chat_completions(request_context.tenant_id(), request_context.project_id())
            .expect("chat completions"),
    )
    .into_response()
}

async fn chat_completion_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(completion_id): Path<String>,
) -> Response {
    match relay_get_chat_completion_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &completion_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "chat_completion",
                &completion_id,
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
                "failed to relay upstream chat completion retrieve",
            );
        }
    }

    let local_response = match local_chat_completion_retrieve_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &completion_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "chat_completion",
        &completion_id,
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

async fn chat_completion_update_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(completion_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateChatCompletionRequest>,
) -> Response {
    match relay_update_chat_completion_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &completion_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "chat_completion",
                &completion_id,
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
            return bad_gateway_openai_response("failed to relay upstream chat completion update");
        }
    }

    let local_response = match local_chat_completion_update_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &completion_id,
        request.metadata.unwrap_or(serde_json::json!({})),
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "chat_completion",
        &completion_id,
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

async fn chat_completion_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(completion_id): Path<String>,
) -> Response {
    match relay_delete_chat_completion_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &completion_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "chat_completion",
                &completion_id,
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
            return bad_gateway_openai_response("failed to relay upstream chat completion delete");
        }
    }

    let local_response = match local_chat_completion_delete_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &completion_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "chat_completion",
        &completion_id,
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

async fn chat_completion_messages_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(completion_id): Path<String>,
) -> Response {
    match relay_list_chat_completion_messages_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &completion_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "chat_completion",
                &completion_id,
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
                "failed to relay upstream chat completion messages",
            );
        }
    }

    let local_response = match local_chat_completion_messages_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &completion_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "chat_completion",
        &completion_id,
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

