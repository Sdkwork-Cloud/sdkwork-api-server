use super::*;
use crate::gateway_commercial::{
    begin_gateway_commercial_admission, capture_gateway_commercial_admission,
    enforce_project_quota, music_billing_amount, music_billing_units, music_seconds_from_response,
    record_gateway_usage_for_project_with_media_and_reference_id,
    record_gateway_usage_for_project_with_route_key_and_reference_id,
    release_gateway_commercial_admission,
    response_usage_id_or_single_data_item_id as commercial_response_usage_id_or_single_data_item_id,
    BillingMediaMetrics, GatewayCommercialAdmissionDecision, GatewayCommercialAdmissionSpec,
};

fn local_music_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_bad_gateway_response(error, "invalid_music_request")
}

fn local_music_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_error_response(error, "Requested music was not found.")
}

fn validated_requested_music_seconds(request: &CreateMusicRequest) -> Result<f64, Response> {
    let Some(duration_seconds) = request.duration_seconds else {
        return Err(invalid_request_openai_response(
            "duration_seconds is required for music create requests.",
            "invalid_music_request",
        ));
    };

    if !duration_seconds.is_finite() || duration_seconds <= 0.0 {
        return Err(invalid_request_openai_response(
            "duration_seconds must be a positive finite number.",
            "invalid_music_request",
        ));
    }

    Ok(duration_seconds)
}

pub(crate) async fn music_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateMusicRequest>,
) -> Response {
    let requested_music_seconds = match validated_requested_music_seconds(&request) {
        Ok(duration_seconds) => duration_seconds,
        Err(response) => return response,
    };
    let commercial_admission = match begin_gateway_commercial_admission(
        &state,
        request_context.context(),
        GatewayCommercialAdmissionSpec {
            quoted_amount: music_billing_amount(requested_music_seconds),
        },
    )
    .await
    {
        Ok(GatewayCommercialAdmissionDecision::Canonical(admission)) => Some(admission),
        Ok(GatewayCommercialAdmissionDecision::LegacyQuota) => {
            match enforce_project_quota(
                state.store.as_ref(),
                request_context.project_id(),
                music_billing_units(requested_music_seconds),
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

    match relay_music_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let Some(usage_model) = commercial_response_usage_id_or_single_data_item_id(&response)
            else {
                if let Some(admission) = commercial_admission.as_ref() {
                    if let Err(response) =
                        release_gateway_commercial_admission(&state, admission).await
                    {
                        return response;
                    }
                }
                return bad_gateway_openai_response("upstream music response missing usage id");
            };
            let music_seconds = {
                let response_music_seconds = music_seconds_from_response(&response);
                if response_music_seconds > 0.0 {
                    response_music_seconds
                } else {
                    requested_music_seconds
                }
            };

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
                "music",
                &request.model,
                music_billing_units(music_seconds),
                music_billing_amount(music_seconds),
                BillingMediaMetrics {
                    music_seconds,
                    ..BillingMediaMetrics::default()
                },
                Some(usage_model),
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
            return bad_gateway_openai_response("failed to relay upstream music create");
        }
    }

    let response = match create_music(
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
            return local_music_error_response(error);
        }
    };
    let usage_model = match response.data.as_slice() {
        [track] => track.id.as_str(),
        _ => request.model.as_str(),
    };
    let music_seconds = match response.data.as_slice() {
        [track] => track.duration_seconds.unwrap_or(requested_music_seconds),
        _ => requested_music_seconds,
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
        "music",
        &request.model,
        music_billing_units(music_seconds),
        music_billing_amount(music_seconds),
        BillingMediaMetrics {
            music_seconds,
            ..BillingMediaMetrics::default()
        },
        Some(usage_model),
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

pub(crate) async fn music_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Response {
    match relay_list_music_from_store(
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
                "music",
                "music",
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

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream music list");
        }
    }

    let response = match list_music(request_context.tenant_id(), request_context.project_id()) {
        Ok(response) => response,
        Err(error) => return local_music_error_response(error),
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "music",
        "music",
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

    Json(response).into_response()
}

pub(crate) async fn music_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(music_id): Path<String>,
) -> Response {
    match relay_get_music_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &music_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "music",
                &music_id,
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

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream music retrieve");
        }
    }

    let response = match get_music(
        request_context.tenant_id(),
        request_context.project_id(),
        &music_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_music_not_found_response(error),
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "music",
        &music_id,
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

    Json(response).into_response()
}

pub(crate) async fn music_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(music_id): Path<String>,
) -> Response {
    match relay_delete_music_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &music_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "music",
                &music_id,
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

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream music delete");
        }
    }

    let response = match delete_music(
        request_context.tenant_id(),
        request_context.project_id(),
        &music_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_music_not_found_response(error),
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "music",
        &music_id,
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

    Json(response).into_response()
}

pub(crate) async fn music_content_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(music_id): Path<String>,
) -> Response {
    match relay_music_content_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &music_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "music",
                &music_id,
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

            return upstream_passthrough_response(response);
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream music content");
        }
    }

    let response = local_music_content_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &music_id,
    );
    if !response.status().is_success() {
        return response;
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "music",
        &music_id,
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

    response
}

pub(crate) async fn music_lyrics_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateMusicLyricsRequest>,
) -> Response {
    match relay_music_lyrics_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let Some(reference_id) = commercial_response_usage_id_or_single_data_item_id(&response)
            else {
                return bad_gateway_openai_response("upstream music lyrics response missing id");
            };
            if record_gateway_usage_for_project_with_route_key_and_reference_id(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "music",
                "lyrics",
                reference_id,
                20,
                0.02,
                Some(reference_id),
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
            return bad_gateway_openai_response("failed to relay upstream music lyrics");
        }
    }

    let response = match create_music_lyrics(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => response,
        Err(error) => return local_music_error_response(error),
    };

    if record_gateway_usage_for_project_with_route_key_and_reference_id(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "music",
        "lyrics",
        response.id.as_str(),
        20,
        0.02,
        Some(response.id.as_str()),
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
