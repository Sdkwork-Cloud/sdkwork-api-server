fn local_anthropic_count_tokens_response(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> Response {
    match count_response_input_tokens(tenant_id, project_id, model) {
        Ok(response) => match serde_json::to_value(response) {
            Ok(value) => Json(openai_count_tokens_to_anthropic(&value)).into_response(),
            Err(error) => anthropic_bad_gateway_response(error.to_string()),
        },
        Err(error) => {
            let message = error.to_string();
            if message.to_ascii_lowercase().contains("required") {
                return anthropic_invalid_request_response(message);
            }

            anthropic_bad_gateway_response(message)
        }
    }
}

fn local_anthropic_invalid_model_response(error: anyhow::Error) -> Response {
    let message = error.to_string();
    if local_gateway_error_is_invalid_request(&message) {
        return anthropic_invalid_request_response(message);
    }

    anthropic_bad_gateway_response(message)
}

fn local_anthropic_chat_completion_result(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> std::result::Result<Value, Response> {
    match local_chat_completion_gateway_result(tenant_id, project_id, model) {
        Ok(response) => match serde_json::to_value(response) {
            Ok(value) => Ok(openai_chat_response_to_anthropic(&value)),
            Err(error) => Err(anthropic_bad_gateway_response(error.to_string())),
        },
        Err(error) => Err(local_anthropic_invalid_model_response(error)),
    }
}

fn local_anthropic_stream_result(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> std::result::Result<Response, Response> {
    match local_chat_completion_gateway_result(tenant_id, project_id, model) {
        Ok(_) => Ok(local_anthropic_stream_body_response(model)),
        Err(error) => Err(local_anthropic_invalid_model_response(error)),
    }
}

fn local_anthropic_stream_body_response(model: &str) -> Response {
    let body = format!(
        concat!(
            "event: message_start\n",
            "data: {{\"type\":\"message_start\",\"message\":{{\"id\":\"msg_stream\",\"type\":\"message\",\"role\":\"assistant\",\"model\":\"{model}\",\"content\":[],\"stop_reason\":null,\"stop_sequence\":null,\"usage\":{{\"input_tokens\":0,\"output_tokens\":0}}}}}}\n\n",
            "event: content_block_start\n",
            "data: {{\"type\":\"content_block_start\",\"index\":0,\"content_block\":{{\"type\":\"text\",\"text\":\"\"}}}}\n\n",
            "event: content_block_delta\n",
            "data: {{\"type\":\"content_block_delta\",\"index\":0,\"delta\":{{\"type\":\"text_delta\",\"text\":\"Hello\"}}}}\n\n",
            "event: content_block_stop\n",
            "data: {{\"type\":\"content_block_stop\",\"index\":0}}\n\n",
            "event: message_delta\n",
            "data: {{\"type\":\"message_delta\",\"delta\":{{\"stop_reason\":\"end_turn\",\"stop_sequence\":null}},\"usage\":{{\"output_tokens\":0}}}}\n\n",
            "event: message_stop\n",
            "data: {{\"type\":\"message_stop\"}}\n\n"
        ),
        model = model,
    );
    ([(header::CONTENT_TYPE, "text/event-stream")], body).into_response()
}

fn local_gemini_invalid_model_response(error: anyhow::Error) -> Response {
    let message = error.to_string();
    if local_gateway_error_is_invalid_request(&message) {
        return gemini_invalid_request_response(message);
    }

    gemini_bad_gateway_response(message)
}

fn local_gemini_chat_completion_result(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> std::result::Result<Value, Response> {
    match local_chat_completion_gateway_result(tenant_id, project_id, model) {
        Ok(response) => match serde_json::to_value(response) {
            Ok(value) => Ok(openai_chat_response_to_gemini(&value)),
            Err(error) => Err(gemini_bad_gateway_response(error.to_string())),
        },
        Err(error) => Err(local_gemini_invalid_model_response(error)),
    }
}

fn local_gemini_stream_result(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> std::result::Result<Response, Response> {
    match local_chat_completion_gateway_result(tenant_id, project_id, model) {
        Ok(_) => Ok(local_gemini_stream_body_response()),
        Err(error) => Err(local_gemini_invalid_model_response(error)),
    }
}

fn local_gemini_stream_body_response() -> Response {
    let body = concat!(
        "data: {\"candidates\":[{\"content\":{\"role\":\"model\",\"parts\":[{\"text\":\"Hello\"}]},\"finishReason\":\"STOP\"}]}\n\n"
    );
    ([(header::CONTENT_TYPE, "text/event-stream")], body).into_response()
}

fn local_gemini_count_tokens_result(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> std::result::Result<Value, Response> {
    match count_response_input_tokens(tenant_id, project_id, model) {
        Ok(response) => match serde_json::to_value(response) {
            Ok(value) => Ok(openai_count_tokens_to_gemini(&value)),
            Err(error) => Err(gemini_bad_gateway_response(error.to_string())),
        },
        Err(error) => {
            let message = error.to_string();
            if local_gateway_error_is_invalid_request(&message) {
                return Err(gemini_invalid_request_response(message));
            }

            Err(gemini_bad_gateway_response(message))
        }
    }
}

async fn completions_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateCompletionRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Completions(&request))
        .await
    {
        Ok(Some(response)) => Json(response).into_response(),
        Ok(None) => local_completion_response(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        ),
        Err(_) => bad_gateway_openai_response("failed to relay upstream completion"),
    }
}

async fn embeddings_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateEmbeddingRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Embeddings(&request))
        .await
    {
        Ok(Some(response)) => Json(response).into_response(),
        Ok(None) => local_embedding_response(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        ),
        Err(_) => bad_gateway_openai_response("failed to relay upstream embedding"),
    }
}

async fn moderations_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateModerationRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Moderations(&request))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {
            return local_moderation_response(
                request_context.tenant_id(),
                request_context.project_id(),
                &request.model,
            );
        }
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream moderation");
        }
    }
}

async fn image_generations_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateImageRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ImagesGenerations(&request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {
            return local_image_generation_response(
                request_context.tenant_id(),
                request_context.project_id(),
                &request.model,
            );
        }
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream image generation");
        }
    }
}

async fn image_edits_handler(
    request_context: StatelessGatewayRequest,
    multipart: Multipart,
) -> Response {
    match parse_image_edit_request(multipart).await {
        Ok(request) => {
            match relay_stateless_json_request(
                &request_context,
                ProviderRequest::ImagesEdits(&request),
            )
            .await
            {
                Ok(Some(response)) => return Json(response).into_response(),
                Ok(None) => {}
                Err(_) => {
                    return bad_gateway_openai_response("failed to relay upstream image edit");
                }
            }

            Json(
                create_image_edit(
                    request_context.tenant_id(),
                    request_context.project_id(),
                    &request,
                )
                .expect("image edit"),
            )
            .into_response()
        }
        Err(response) => response,
    }
}

async fn image_variations_handler(
    request_context: StatelessGatewayRequest,
    multipart: Multipart,
) -> Response {
    match parse_image_variation_request(multipart).await {
        Ok(request) => {
            match relay_stateless_json_request(
                &request_context,
                ProviderRequest::ImagesVariations(&request),
            )
            .await
            {
                Ok(Some(response)) => return Json(response).into_response(),
                Ok(None) => {}
                Err(_) => {
                    return bad_gateway_openai_response("failed to relay upstream image variation");
                }
            }

            Json(
                create_image_variation(
                    request_context.tenant_id(),
                    request_context.project_id(),
                    &request,
                )
                .expect("image variation"),
            )
            .into_response()
        }
        Err(response) => response,
    }
}


async fn anthropic_messages_with_state_handler(
    request_context: CompatAuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    headers: HeaderMap,
    ExtractJson(payload): ExtractJson<Value>,
) -> Response {
    let request = match anthropic_request_to_chat_completion(&payload) {
        Ok(request) => request,
        Err(error) => return anthropic_invalid_request_response(error.to_string()),
    };
    let options = anthropic_request_options(&headers);

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
                    if record_gateway_usage_for_project_with_context(
                        state.store.as_ref(),
                        request_context.tenant_id(),
                        request_context.project_id(),
                        "chat_completion",
                        &request.model,
                        100,
                        0.10,
                        usage_context.as_ref(),
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

                    return upstream_passthrough_response(anthropic_stream_from_openai(response));
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
                return anthropic_bad_gateway_response(
                    "failed to relay upstream anthropic message stream",
                );
            }
        }

        let local_response = match local_anthropic_stream_result(
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

        if record_gateway_usage_for_project(
            state.store.as_ref(),
            request_context.tenant_id(),
            request_context.project_id(),
            "chat_completion",
            &request.model,
            100,
            0.10,
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

        return local_response;
    }

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
                if record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context(
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
                .await
                .is_err()
                {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to record usage",
                    )
                        .into_response();
                }

                return Json(openai_chat_response_to_anthropic(&response)).into_response();
            }
        }
        Err(_) => {
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = release_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            return anthropic_bad_gateway_response("failed to relay upstream anthropic message");
        }
    }

    let local_response = match local_anthropic_chat_completion_result(
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

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "chat_completion",
        &request.model,
        100,
        0.10,
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

async fn anthropic_count_tokens_with_state_handler(
    request_context: CompatAuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(payload): ExtractJson<Value>,
) -> Response {
    let request = match anthropic_count_tokens_request(&payload) {
        Ok(request) => request,
        Err(error) => return anthropic_invalid_request_response(error.to_string()),
    };

    match relay_count_response_input_tokens_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
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

async fn gemini_models_compat_handler(
    request_context: StatelessGatewayRequest,
    Path(tail): Path<String>,
    ExtractJson(payload): ExtractJson<Value>,
) -> Response {
    let Some((model, action)) = parse_gemini_compat_tail(&tail) else {
        return gemini_invalid_request_response("unsupported gemini compatibility route");
    };

    match action {
        GeminiCompatAction::GenerateContent => {
            let request = match gemini_request_to_chat_completion(&model, &payload) {
                Ok(request) => request,
                Err(error) => return gemini_invalid_request_response(error.to_string()),
            };

            match relay_stateless_json_request(
                &request_context,
                ProviderRequest::ChatCompletions(&request),
            )
            .await
            {
                Ok(Some(response)) => {
                    Json(openai_chat_response_to_gemini(&response)).into_response()
                }
                Ok(None) => match local_gemini_chat_completion_result(
                    request_context.tenant_id(),
                    request_context.project_id(),
                    &request.model,
                ) {
                    Ok(response) => Json(response).into_response(),
                    Err(response) => response,
                },
                Err(_) => gemini_bad_gateway_response(
                    "failed to relay upstream gemini generateContent request",
                ),
            }
        }
        GeminiCompatAction::StreamGenerateContent => {
            let mut request = match gemini_request_to_chat_completion(&model, &payload) {
                Ok(request) => request,
                Err(error) => return gemini_invalid_request_response(error.to_string()),
            };
            request.stream = Some(true);

            match relay_stateless_stream_request(
                &request_context,
                ProviderRequest::ChatCompletionsStream(&request),
            )
            .await
            {
                Ok(Some(response)) => {
                    upstream_passthrough_response(gemini_stream_from_openai(response))
                }
                Ok(None) => match local_gemini_stream_result(
                    request_context.tenant_id(),
                    request_context.project_id(),
                    &request.model,
                ) {
                    Ok(response) | Err(response) => response,
                },
                Err(_) => gemini_bad_gateway_response(
                    "failed to relay upstream gemini streamGenerateContent request",
                ),
            }
        }
        GeminiCompatAction::CountTokens => {
            let request = gemini_count_tokens_request(&model, &payload);
            match relay_stateless_json_request(
                &request_context,
                ProviderRequest::ResponsesInputTokens(&request),
            )
            .await
            {
                Ok(Some(response)) => {
                    Json(openai_count_tokens_to_gemini(&response)).into_response()
                }
                Ok(None) => match local_gemini_count_tokens_result(
                    request_context.tenant_id(),
                    request_context.project_id(),
                    &request.model,
                ) {
                    Ok(response) => Json(response).into_response(),
                    Err(response) => response,
                },
                Err(_) => gemini_bad_gateway_response(
                    "failed to relay upstream gemini countTokens request",
                ),
            }
        }
    }
}

async fn gemini_models_compat_with_state_handler(
    request_context: CompatAuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(tail): Path<String>,
    ExtractJson(payload): ExtractJson<Value>,
) -> Response {
    let Some((model, action)) = parse_gemini_compat_tail(&tail) else {
        return gemini_invalid_request_response("unsupported gemini compatibility route");
    };

    match action {
        GeminiCompatAction::GenerateContent => {
            let request = match gemini_request_to_chat_completion(&model, &payload) {
                Ok(request) => request,
                Err(error) => return gemini_invalid_request_response(error.to_string()),
            };

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
                    match enforce_project_quota(
                        state.store.as_ref(),
                        request_context.project_id(),
                        100,
                    )
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

            match relay_chat_completion_from_store_with_execution_context(
                state.store.as_ref(),
                &state.secret_manager,
                request_context.tenant_id(),
                request_context.project_id(),
                &request,
                &ProviderRequestOptions::default(),
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
                        if record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context(
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
                        .await
                        .is_err()
                        {
                            return (
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                "failed to record usage",
                            )
                                .into_response();
                        }

                        return Json(openai_chat_response_to_gemini(&response)).into_response();
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
                    return gemini_bad_gateway_response(
                        "failed to relay upstream gemini generateContent request",
                    );
                }
            }

            let local_response = match local_gemini_chat_completion_result(
                request_context.tenant_id(),
                request_context.project_id(),
                &request.model,
            ) {
                Ok(response) => response,
                Err(response) => {
                    if let Some(admission) = commercial_admission.as_ref() {
                        if let Err(release_response) =
                            release_gateway_commercial_admission(&state, admission).await
                        {
                            return release_response;
                        }
                    }
                    return response;
                }
            };

            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = capture_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }

            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "chat_completion",
                &request.model,
                100,
                0.10,
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
        GeminiCompatAction::StreamGenerateContent => {
            let mut request = match gemini_request_to_chat_completion(&model, &payload) {
                Ok(request) => request,
                Err(error) => return gemini_invalid_request_response(error.to_string()),
            };
            request.stream = Some(true);

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
                    match enforce_project_quota(
                        state.store.as_ref(),
                        request_context.project_id(),
                        100,
                    )
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

            match relay_chat_completion_stream_from_store_with_execution_context(
                state.store.as_ref(),
                &state.secret_manager,
                request_context.tenant_id(),
                request_context.project_id(),
                &request,
                &ProviderRequestOptions::default(),
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
                        if record_gateway_usage_for_project_with_context(
                            state.store.as_ref(),
                            request_context.tenant_id(),
                            request_context.project_id(),
                            "chat_completion",
                            &request.model,
                            100,
                            0.10,
                            usage_context.as_ref(),
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

                        return upstream_passthrough_response(gemini_stream_from_openai(response));
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
                    return gemini_bad_gateway_response(
                        "failed to relay upstream gemini streamGenerateContent request",
                    );
                }
            }

            let local_response = match local_gemini_stream_result(
                request_context.tenant_id(),
                request_context.project_id(),
                &request.model,
            ) {
                Ok(response) => response,
                Err(response) => {
                    if let Some(admission) = commercial_admission.as_ref() {
                        if let Err(release_response) =
                            release_gateway_commercial_admission(&state, admission).await
                        {
                            return release_response;
                        }
                    }
                    return response;
                }
            };

            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = capture_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }

            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "chat_completion",
                &request.model,
                100,
                0.10,
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

            local_response
        }
        GeminiCompatAction::CountTokens => {
            let request = gemini_count_tokens_request(&model, &payload);

            match relay_count_response_input_tokens_from_store(
                state.store.as_ref(),
                &state.secret_manager,
                request_context.tenant_id(),
                request_context.project_id(),
                &request,
            )
            .await
            {
                Ok(Some(response)) => Json(openai_count_tokens_to_gemini(&response)).into_response(),
                Ok(None) => match local_gemini_count_tokens_result(
                    request_context.tenant_id(),
                    request_context.project_id(),
                    &request.model,
                ) {
                    Ok(response) => Json(response).into_response(),
                    Err(response) => response,
                },
                Err(_) => gemini_bad_gateway_response(
                    "failed to relay upstream gemini countTokens request",
                ),
            }
        }
    }
}

