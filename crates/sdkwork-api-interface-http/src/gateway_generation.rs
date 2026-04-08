fn local_completion_gateway_result(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> anyhow::Result<CompletionObject> {
    create_completion(tenant_id, project_id, model)
}

fn local_completion_result(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> std::result::Result<CompletionObject, Response> {
    local_completion_gateway_result(tenant_id, project_id, model)
        .map_err(local_response_invalid_model_response)
}

fn local_completion_response(tenant_id: &str, project_id: &str, model: &str) -> Response {
    match local_completion_result(tenant_id, project_id, model) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_embedding_gateway_result(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> anyhow::Result<CreateEmbeddingResponse> {
    create_embedding(tenant_id, project_id, model)
}

fn local_embedding_result(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> std::result::Result<CreateEmbeddingResponse, Response> {
    local_embedding_gateway_result(tenant_id, project_id, model)
        .map_err(local_response_invalid_model_response)
}

fn local_embedding_response(tenant_id: &str, project_id: &str, model: &str) -> Response {
    match local_embedding_result(tenant_id, project_id, model) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_moderation_gateway_result(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> anyhow::Result<ModerationResponse> {
    create_moderation(tenant_id, project_id, model)
}

fn local_moderation_result(
    tenant_id: &str,
    project_id: &str,
    model: &str,
) -> std::result::Result<ModerationResponse, Response> {
    local_moderation_gateway_result(tenant_id, project_id, model)
        .map_err(local_response_invalid_model_response)
}

fn local_moderation_response(tenant_id: &str, project_id: &str, model: &str) -> Response {
    match local_moderation_result(tenant_id, project_id, model) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}


async fn completions_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateCompletionRequest>,
) -> Response {
    let commercial_admission = match begin_gateway_commercial_admission(
        &state,
        request_context.context(),
        GatewayCommercialAdmissionSpec {
            quoted_amount: 0.08,
        },
    )
    .await
    {
        Ok(GatewayCommercialAdmissionDecision::Canonical(admission)) => Some(admission),
        Ok(GatewayCommercialAdmissionDecision::LegacyQuota) => {
            match enforce_project_quota(state.store.as_ref(), request_context.project_id(), 80)
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

    match relay_completion_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = capture_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            if record_gateway_usage_for_project_with_route_key_and_reference_id(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "completions",
                &request.model,
                &request.model,
                80,
                0.08,
                response_usage_id_or_single_data_item_id(&response),
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
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = release_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            return bad_gateway_openai_response("failed to relay upstream completion");
        }
    }

    if let Some(admission) = commercial_admission.as_ref() {
        if let Err(response) = capture_gateway_commercial_admission(&state, admission).await {
            return response;
        }
    }

    let local_completion = match local_completion_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "completions",
        &request.model,
        80,
        0.08,
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

    Json(local_completion).into_response()
}

async fn embeddings_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateEmbeddingRequest>,
) -> Response {
    let commercial_admission = match begin_gateway_commercial_admission(
        &state,
        request_context.context(),
        GatewayCommercialAdmissionSpec {
            quoted_amount: 0.01,
        },
    )
    .await
    {
        Ok(GatewayCommercialAdmissionDecision::Canonical(admission)) => Some(admission),
        Ok(GatewayCommercialAdmissionDecision::LegacyQuota) => {
            match enforce_project_quota(state.store.as_ref(), request_context.project_id(), 10)
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

    match relay_embedding_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = capture_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            let token_usage = extract_token_usage_metrics(&response);
            if record_gateway_usage_for_project_with_route_key_and_tokens_and_reference(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "embeddings",
                &request.model,
                &request.model,
                10,
                0.01,
                token_usage,
                response_usage_id_or_single_data_item_id(&response),
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
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = release_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            return bad_gateway_openai_response("failed to relay upstream embedding");
        }
    }

    let local_embedding = match local_embedding_result(
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
        if let Err(response) = capture_gateway_commercial_admission(&state, admission).await {
            return response;
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "embeddings",
        &request.model,
        10,
        0.01,
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

    Json(local_embedding).into_response()
}

async fn moderations_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateModerationRequest>,
) -> Response {
    match relay_moderation_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key_and_reference_id(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "moderations",
                &request.model,
                &request.model,
                1,
                0.001,
                response_usage_id_or_single_data_item_id(&response),
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
            return bad_gateway_openai_response("failed to relay upstream moderation");
        }
    }

    let local_moderation = match local_moderation_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "moderations",
        &request.model,
        1,
        0.001,
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

    Json(local_moderation).into_response()
}

