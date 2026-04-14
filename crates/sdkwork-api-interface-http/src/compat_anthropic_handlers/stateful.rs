use super::*;
use crate::gateway_commercial::{
    begin_gateway_commercial_admission, capture_gateway_commercial_admission,
    extract_token_usage_metrics as extract_commercial_token_usage_metrics,
    record_gateway_usage_for_project as record_gateway_usage_for_project_commercial,
    record_gateway_usage_for_project_with_context,
    record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context,
    release_gateway_commercial_admission,
    response_usage_id_or_single_data_item_id as commercial_response_usage_id_or_single_data_item_id,
    GatewayCommercialAdmissionDecision, GatewayCommercialAdmissionSpec,
};
use sdkwork_api_app_gateway::{
    relay_chat_completion_from_store_with_execution_context,
    relay_chat_completion_stream_from_store_with_execution_context,
};

fn local_anthropic_error_response(error: anyhow::Error) -> Response {
    let message = error.to_string();
    if local_gateway_error_is_invalid_request(&message) {
        return anthropic_invalid_request_response(message);
    }

    anthropic_bad_gateway_response("failed to process local anthropic fallback")
}

fn local_anthropic_encoding_error_response() -> Response {
    anthropic_bad_gateway_response("failed to encode local anthropic response")
}

pub(crate) async fn anthropic_messages_with_state_handler(
    request_context: CompatAuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    headers: HeaderMap,
    ExtractJson(payload): ExtractJson<Value>,
) -> Response {
    let request = match anthropic_request_to_chat_completion(&payload) {
        Ok(request) => request,
        Err(error) => return anthropic_invalid_request_response(error.to_string()),
    };
    if request.model.trim().is_empty() {
        return anthropic_invalid_request_response("Chat completion model is required.");
    }
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

        if let Some(admission) = commercial_admission.as_ref() {
            if let Err(response) = release_gateway_commercial_admission(&state, admission).await {
                return response;
            }
        }

        return anthropic_invalid_request_response(
            "Local anthropic message streaming fallback is not supported without an upstream provider.",
        );
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
                if record_gateway_usage_for_project_with_route_key_and_tokens_and_reference_with_context(
                    state.store.as_ref(),
                    request_context.tenant_id(),
                    request_context.project_id(),
                    "chat_completion",
                    &request.model,
                    &request.model,
                    100,
                    0.10,
                    extract_commercial_token_usage_metrics(&response),
                    commercial_response_usage_id_or_single_data_item_id(&response),
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

    let local_chat_completion = match create_chat_completion(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
    ) {
        Ok(response) => response,
        Err(error) => {
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = release_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            return local_anthropic_error_response(error);
        }
    };
    let local_response = match serde_json::to_value(local_chat_completion) {
        Ok(response) => response,
        Err(_) => {
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = release_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            return local_anthropic_encoding_error_response();
        }
    };

    if let Some(admission) = commercial_admission.as_ref() {
        if let Err(response) = capture_gateway_commercial_admission(&state, admission).await {
            return response;
        }
    }

    if record_gateway_usage_for_project_commercial(
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

    Json(openai_chat_response_to_anthropic(&local_response)).into_response()
}

pub(crate) async fn anthropic_count_tokens_with_state_handler(
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
        Ok(None) => {
            let local_response = match count_response_input_tokens(
                request_context.tenant_id(),
                request_context.project_id(),
                &request.model,
            ) {
                Ok(response) => response,
                Err(error) => return local_anthropic_error_response(error),
            };
            match serde_json::to_value(local_response) {
                Ok(response) => Json(openai_count_tokens_to_anthropic(&response)).into_response(),
                Err(_) => local_anthropic_encoding_error_response(),
            }
        }
        Err(_) => anthropic_bad_gateway_response(
            "failed to relay upstream anthropic count tokens request",
        ),
    }
}
