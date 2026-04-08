async fn responses_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateResponseRequest>,
) -> Response {
    let commercial_admission = match begin_gateway_commercial_admission(
        &state,
        request_context.context(),
        GatewayCommercialAdmissionSpec {
            quoted_amount: 0.12,
        },
    )
    .await
    {
        Ok(GatewayCommercialAdmissionDecision::Canonical(admission)) => Some(admission),
        Ok(GatewayCommercialAdmissionDecision::LegacyQuota) => {
            match enforce_project_quota(state.store.as_ref(), request_context.project_id(), 120)
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
        match relay_response_stream_from_store_with_execution_context(
            state.store.as_ref(),
            &state.secret_manager,
            request_context.tenant_id(),
            request_context.project_id(),
            &request,
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
                        "responses",
                        &request.model,
                        120,
                        0.12,
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
                return bad_gateway_openai_response("failed to relay upstream response stream");
            }
        }

        let _local_response = match local_response_create_result(
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
            "responses",
            &request.model,
            120,
            0.12,
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

        return local_response_stream_body_response("resp_1", &request.model);
    }

    match relay_response_from_store_with_execution_context(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
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
                    "responses",
                    &request.model,
                    &request.model,
                    120,
                    0.12,
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

                return Json(response).into_response();
            }
        }
        Err(_) => {
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = release_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            return bad_gateway_openai_response("failed to relay upstream response");
        }
    }

    let local_response = match local_response_create_result(
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
        "responses",
        &request.model,
        120,
        0.12,
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
