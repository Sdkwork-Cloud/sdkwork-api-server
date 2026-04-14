use super::*;
use crate::gateway_commercial::{
    begin_gateway_commercial_admission, capture_gateway_commercial_admission,
    enforce_project_quota, record_gateway_usage_for_project_with_media_and_reference_id,
    release_gateway_commercial_admission,
    response_usage_id_or_single_data_item_id as commercial_response_usage_id_or_single_data_item_id,
    BillingMediaMetrics, GatewayCommercialAdmissionDecision, GatewayCommercialAdmissionSpec,
};

fn local_image_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_bad_gateway_response(error, "invalid_image_request")
}

fn response_image_count(response: &Value) -> u64 {
    response
        .get("data")
        .and_then(Value::as_array)
        .and_then(|data| u64::try_from(data.len()).ok())
        .unwrap_or(0)
}

pub(crate) async fn image_generations_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateImageRequest>,
) -> Response {
    let commercial_admission = match begin_gateway_commercial_admission(
        &state,
        request_context.context(),
        GatewayCommercialAdmissionSpec {
            quoted_amount: 0.05,
        },
    )
    .await
    {
        Ok(GatewayCommercialAdmissionDecision::Canonical(admission)) => Some(admission),
        Ok(GatewayCommercialAdmissionDecision::LegacyQuota) => {
            match enforce_project_quota(state.store.as_ref(), request_context.project_id(), 50)
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

    match relay_image_generation_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let image_count = response_image_count(&response);
            if image_count == 0 {
                if let Some(admission) = commercial_admission.as_ref() {
                    if let Err(response) =
                        release_gateway_commercial_admission(&state, admission).await
                    {
                        return response;
                    }
                }
                return bad_gateway_openai_response("upstream image response missing image data");
            }
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = capture_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            if record_gateway_usage_for_project_with_media_and_reference_id(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "images",
                &request.model,
                50,
                0.05,
                BillingMediaMetrics {
                    image_count,
                    ..BillingMediaMetrics::default()
                },
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
            return bad_gateway_openai_response("failed to relay upstream image generation");
        }
    }

    let response = match create_image_generation(
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
            return local_image_error_response(error);
        }
    };
    let Some(image_count) = u64::try_from(response.data.len())
        .ok()
        .filter(|count| *count > 0)
    else {
        if let Some(admission) = commercial_admission.as_ref() {
            if let Err(response) = release_gateway_commercial_admission(&state, admission).await {
                return response;
            }
        }
        return bad_gateway_openai_response("local image response missing image data");
    };

    if let Some(admission) = commercial_admission.as_ref() {
        if let Err(response) = capture_gateway_commercial_admission(&state, admission).await {
            return response;
        }
    }

    if record_gateway_usage_for_project_with_media_and_reference_id(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "images",
        &request.model,
        50,
        0.05,
        BillingMediaMetrics {
            image_count,
            ..BillingMediaMetrics::default()
        },
        None,
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

pub(crate) async fn image_edits_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    multipart: Multipart,
) -> Response {
    let request = match parse_image_edit_request(multipart).await {
        Ok(request) => request,
        Err(response) => return response,
    };
    let route_model = request.model_or_default().to_owned();
    let commercial_admission = match begin_gateway_commercial_admission(
        &state,
        request_context.context(),
        GatewayCommercialAdmissionSpec {
            quoted_amount: 0.05,
        },
    )
    .await
    {
        Ok(GatewayCommercialAdmissionDecision::Canonical(admission)) => Some(admission),
        Ok(GatewayCommercialAdmissionDecision::LegacyQuota) => {
            match enforce_project_quota(state.store.as_ref(), request_context.project_id(), 50)
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

    match relay_image_edit_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let image_count = response_image_count(&response);
            if image_count == 0 {
                if let Some(admission) = commercial_admission.as_ref() {
                    if let Err(response) =
                        release_gateway_commercial_admission(&state, admission).await
                    {
                        return response;
                    }
                }
                return bad_gateway_openai_response("upstream image response missing image data");
            }
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = capture_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            if record_gateway_usage_for_project_with_media_and_reference_id(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "images",
                &route_model,
                50,
                0.05,
                BillingMediaMetrics {
                    image_count,
                    ..BillingMediaMetrics::default()
                },
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
            return bad_gateway_openai_response("failed to relay upstream image edit");
        }
    }

    let response = match create_image_edit(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => response,
        Err(error) => {
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = release_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            return local_image_error_response(error);
        }
    };
    let Some(image_count) = u64::try_from(response.data.len())
        .ok()
        .filter(|count| *count > 0)
    else {
        if let Some(admission) = commercial_admission.as_ref() {
            if let Err(response) = release_gateway_commercial_admission(&state, admission).await {
                return response;
            }
        }
        return bad_gateway_openai_response("local image response missing image data");
    };

    if let Some(admission) = commercial_admission.as_ref() {
        if let Err(response) = capture_gateway_commercial_admission(&state, admission).await {
            return response;
        }
    }

    if record_gateway_usage_for_project_with_media_and_reference_id(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "images",
        &route_model,
        50,
        0.05,
        BillingMediaMetrics {
            image_count,
            ..BillingMediaMetrics::default()
        },
        None,
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

pub(crate) async fn image_variations_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    multipart: Multipart,
) -> Response {
    let request = match parse_image_variation_request(multipart).await {
        Ok(request) => request,
        Err(response) => return response,
    };
    let route_model = request.model_or_default().to_owned();
    let commercial_admission = match begin_gateway_commercial_admission(
        &state,
        request_context.context(),
        GatewayCommercialAdmissionSpec {
            quoted_amount: 0.05,
        },
    )
    .await
    {
        Ok(GatewayCommercialAdmissionDecision::Canonical(admission)) => Some(admission),
        Ok(GatewayCommercialAdmissionDecision::LegacyQuota) => {
            match enforce_project_quota(state.store.as_ref(), request_context.project_id(), 50)
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

    match relay_image_variation_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let image_count = response_image_count(&response);
            if image_count == 0 {
                if let Some(admission) = commercial_admission.as_ref() {
                    if let Err(response) =
                        release_gateway_commercial_admission(&state, admission).await
                    {
                        return response;
                    }
                }
                return bad_gateway_openai_response("upstream image response missing image data");
            }
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = capture_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            if record_gateway_usage_for_project_with_media_and_reference_id(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "images",
                &route_model,
                50,
                0.05,
                BillingMediaMetrics {
                    image_count,
                    ..BillingMediaMetrics::default()
                },
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
            return bad_gateway_openai_response("failed to relay upstream image variation");
        }
    }

    let response = match create_image_variation(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => response,
        Err(error) => {
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = release_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            return local_image_error_response(error);
        }
    };
    let Some(image_count) = u64::try_from(response.data.len())
        .ok()
        .filter(|count| *count > 0)
    else {
        if let Some(admission) = commercial_admission.as_ref() {
            if let Err(response) = release_gateway_commercial_admission(&state, admission).await {
                return response;
            }
        }
        return bad_gateway_openai_response("local image response missing image data");
    };

    if let Some(admission) = commercial_admission.as_ref() {
        if let Err(response) = capture_gateway_commercial_admission(&state, admission).await {
            return response;
        }
    }

    if record_gateway_usage_for_project_with_media_and_reference_id(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "images",
        &route_model,
        50,
        0.05,
        BillingMediaMetrics {
            image_count,
            ..BillingMediaMetrics::default()
        },
        None,
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
