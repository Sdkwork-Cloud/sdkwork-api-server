use super::*;
use crate::gateway_commercial::{
    begin_gateway_commercial_admission, capture_gateway_commercial_admission,
    extract_token_usage_metrics as extract_commercial_token_usage_metrics,
    record_gateway_usage_for_project as record_gateway_usage_for_project_commercial,
    record_gateway_usage_for_project_with_route_key_and_reference_id,
    record_gateway_usage_for_project_with_route_key_and_tokens_and_reference,
    release_gateway_commercial_admission,
    response_usage_id_or_single_data_item_id as commercial_response_usage_id_or_single_data_item_id,
    GatewayCommercialAdmissionDecision, GatewayCommercialAdmissionSpec,
};

fn local_inference_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_bad_gateway_response(error, "invalid_model")
}

fn response_id(response: &Value) -> Option<&str> {
    response
        .get("id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|id| !id.is_empty())
}

fn response_has_non_empty_array(response: &Value, field: &str) -> bool {
    response
        .get(field)
        .and_then(Value::as_array)
        .is_some_and(|items| !items.is_empty())
}

fn completion_response_has_output(response: &Value) -> bool {
    response_id(response).is_some() && response_has_non_empty_array(response, "choices")
}

fn embedding_response_has_output(response: &Value) -> bool {
    response_has_non_empty_array(response, "data")
}

fn moderation_response_has_output(response: &Value) -> bool {
    response_id(response).is_some() && response_has_non_empty_array(response, "results")
}

pub(crate) async fn completions_with_state_handler(
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
            if !completion_response_has_output(&response) {
                if let Some(admission) = commercial_admission.as_ref() {
                    if let Err(response) =
                        release_gateway_commercial_admission(&state, admission).await
                    {
                        return response;
                    }
                }
                return bad_gateway_openai_response(
                    "upstream completion response missing id or choices",
                );
            }
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
                commercial_response_usage_id_or_single_data_item_id(&response),
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

    let local_completion = match create_completion(
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
            return local_inference_error_response(error);
        }
    };
    if local_completion.id.trim().is_empty() || local_completion.choices.is_empty() {
        if let Some(admission) = commercial_admission.as_ref() {
            if let Err(response) = release_gateway_commercial_admission(&state, admission).await {
                return response;
            }
        }
        return bad_gateway_openai_response("failed to process local completion fallback");
    }

    if let Some(admission) = commercial_admission.as_ref() {
        if let Err(response) = capture_gateway_commercial_admission(&state, admission).await {
            return response;
        }
    }

    if record_gateway_usage_for_project_commercial(
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

pub(crate) async fn embeddings_with_state_handler(
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
            if !embedding_response_has_output(&response) {
                if let Some(admission) = commercial_admission.as_ref() {
                    if let Err(response) =
                        release_gateway_commercial_admission(&state, admission).await
                    {
                        return response;
                    }
                }
                return bad_gateway_openai_response(
                    "upstream embedding response missing embedding data",
                );
            }
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = capture_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            if record_gateway_usage_for_project_with_route_key_and_tokens_and_reference(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "embeddings",
                &request.model,
                &request.model,
                10,
                0.01,
                extract_commercial_token_usage_metrics(&response),
                commercial_response_usage_id_or_single_data_item_id(&response),
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

    let local_embedding = match create_embedding(
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
            return local_inference_error_response(error);
        }
    };
    if local_embedding.data.is_empty() {
        if let Some(admission) = commercial_admission.as_ref() {
            if let Err(response) = release_gateway_commercial_admission(&state, admission).await {
                return response;
            }
        }
        return bad_gateway_openai_response("failed to process local embedding fallback");
    }

    if let Some(admission) = commercial_admission.as_ref() {
        if let Err(response) = capture_gateway_commercial_admission(&state, admission).await {
            return response;
        }
    }

    if record_gateway_usage_for_project_commercial(
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

pub(crate) async fn moderations_with_state_handler(
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
            if !moderation_response_has_output(&response) {
                return bad_gateway_openai_response(
                    "upstream moderation response missing id or results",
                );
            }
            if record_gateway_usage_for_project_with_route_key_and_reference_id(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "moderations",
                &request.model,
                &request.model,
                1,
                0.001,
                commercial_response_usage_id_or_single_data_item_id(&response),
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

    let local_moderation = match create_moderation(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
    ) {
        Ok(response) => response,
        Err(error) => return local_inference_error_response(error),
    };
    if local_moderation.id.trim().is_empty() || local_moderation.results.is_empty() {
        return bad_gateway_openai_response("failed to process local moderation fallback");
    }

    if record_gateway_usage_for_project_commercial(
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
