#![allow(clippy::too_many_arguments)]

use super::*;
use crate::gateway_commercial::{
    begin_gateway_commercial_admission, capture_gateway_commercial_admission,
    enforce_project_quota,
    record_gateway_usage_for_project_with_route_key_and_tokens_reference_and_media_with_context,
    release_gateway_commercial_admission,
    response_usage_id_or_single_data_item_id as commercial_response_usage_id_or_single_data_item_id,
    BillingMediaMetrics, GatewayCommercialAdmissionDecision, GatewayCommercialAdmissionSpec,
};
use axum::http::HeaderMap;
use sdkwork_api_app_gateway::PlannedExecutionUsageContext;
use sdkwork_api_contract_openai::videos::{VideoObject, VideosResponse};

fn video_missing(video_id: &str) -> bool {
    video_id.trim().is_empty() || video_id.ends_with("_missing")
}

fn local_video_variant(video_id: &str, suffix: &str) -> VideoObject {
    let id = format!("{video_id}_{suffix}");
    VideoObject::new(id.clone(), format!("https://example.com/{id}.mp4"))
}

fn local_video_transform_result(
    video_id: &str,
    prompt: &str,
    suffix: &str,
) -> std::result::Result<VideosResponse, Response> {
    if video_missing(video_id) {
        return Err(local_gateway_error_response(
            anyhow::anyhow!("video not found"),
            "Requested video was not found.",
        ));
    }
    if prompt.trim().is_empty() {
        return Err(local_gateway_invalid_or_bad_gateway_response(
            anyhow::anyhow!("Video prompt is required."),
            "invalid_video_request",
        ));
    }

    Ok(VideosResponse::new(vec![local_video_variant(
        video_id, suffix,
    )]))
}

fn local_video_edits_result(
    request: &EditVideoRequest,
) -> std::result::Result<VideosResponse, Response> {
    local_video_transform_result(&request.video_id, &request.prompt, "edited")
}

fn local_video_extensions_result(
    request: &ExtendVideoRequest,
) -> std::result::Result<VideosResponse, Response> {
    if let Some(video_id) = request.video_id.as_deref() {
        return local_video_transform_result(video_id, &request.prompt, "extended");
    }
    if request.prompt.trim().is_empty() {
        return Err(local_gateway_invalid_or_bad_gateway_response(
            anyhow::anyhow!("Video prompt is required."),
            "invalid_video_request",
        ));
    }

    Ok(VideosResponse::new(vec![local_video_variant(
        "video_1", "extended",
    )]))
}

async fn record_video_transform_usage_with_reference(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    route_key: &str,
    usage_model: &str,
    units: u64,
    amount: f64,
    reference_id: Option<&str>,
    usage_context_override: Option<&PlannedExecutionUsageContext>,
) -> anyhow::Result<()> {
    record_gateway_usage_for_project_with_route_key_and_tokens_reference_and_media_with_context(
        store,
        tenant_id,
        project_id,
        "videos",
        route_key,
        usage_model,
        units,
        amount,
        None,
        reference_id,
        BillingMediaMetrics::default(),
        usage_context_override,
    )
    .await
}

pub(crate) async fn video_remix_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    headers: HeaderMap,
    Path(video_id): Path<String>,
    ExtractJson(request): ExtractJson<RemixVideoRequest>,
) -> Response {
    let pending_plan = match plan_pending_video_commercial_workflow(
        &state,
        request_context.context(),
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
        60,
    )
    .await
    {
        Ok(plan) => plan,
        Err(response) => return response,
    };
    let commercial_admission = if pending_plan.model_price.is_some() {
        pending_plan.admission.clone()
    } else {
        match begin_gateway_commercial_admission(
            &state,
            request_context.context(),
            GatewayCommercialAdmissionSpec {
                quoted_amount: 0.06,
            },
        )
        .await
        {
            Ok(GatewayCommercialAdmissionDecision::Canonical(admission)) => Some(admission),
            Ok(GatewayCommercialAdmissionDecision::LegacyQuota) => {
                match enforce_project_quota(state.store.as_ref(), request_context.project_id(), 60)
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
        }
    };

    match relay_remix_video_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if video_response_is_processing(&response) {
                if let Err(response) = persist_pending_video_meter_fact(
                    &state,
                    request_context.context(),
                    &headers,
                    &response,
                    &pending_plan,
                )
                .await
                {
                    if let Some(admission) = commercial_admission.as_ref() {
                        if let Err(release_response) =
                            release_gateway_commercial_admission(&state, admission).await
                        {
                            return release_response;
                        }
                    }
                    return response;
                }
                return Json(response).into_response();
            }
            let Some(usage_model) = commercial_response_usage_id_or_single_data_item_id(&response)
            else {
                if let Some(admission) = commercial_admission.as_ref() {
                    if let Err(response) =
                        release_gateway_commercial_admission(&state, admission).await
                    {
                        return response;
                    }
                }
                return bad_gateway_openai_response(
                    "upstream video remix response missing usage id",
                );
            };
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = capture_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            if record_video_transform_usage_with_reference(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                &video_id,
                usage_model,
                60,
                0.06,
                commercial_response_usage_id_or_single_data_item_id(&response),
                pending_plan.usage_context.as_ref(),
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
            return bad_gateway_openai_response("failed to relay upstream video remix");
        }
    }

    let response = match local_video_transform_result(&video_id, &request.prompt, "remix") {
        Ok(response) => response,
        Err(response) => {
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = release_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            return response;
        }
    };
    let [item] = response.data.as_slice() else {
        if let Some(admission) = commercial_admission.as_ref() {
            if let Err(response) = release_gateway_commercial_admission(&state, admission).await {
                return response;
            }
        }
        return bad_gateway_openai_response("failed to process local video fallback");
    };
    let usage_model = item.id.as_str();

    if let Some(admission) = commercial_admission.as_ref() {
        if let Err(response) = capture_gateway_commercial_admission(&state, admission).await {
            return response;
        }
    }

    if record_video_transform_usage_with_reference(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
        usage_model,
        60,
        0.06,
        Some(usage_model),
        pending_plan.usage_context.as_ref(),
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

pub(crate) async fn video_extend_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    headers: HeaderMap,
    Path(video_id): Path<String>,
    ExtractJson(request): ExtractJson<ExtendVideoRequest>,
) -> Response {
    let pending_plan = match plan_pending_video_commercial_workflow(
        &state,
        request_context.context(),
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
        60,
    )
    .await
    {
        Ok(plan) => plan,
        Err(response) => return response,
    };
    let commercial_admission = if pending_plan.model_price.is_some() {
        pending_plan.admission.clone()
    } else {
        match begin_gateway_commercial_admission(
            &state,
            request_context.context(),
            GatewayCommercialAdmissionSpec {
                quoted_amount: 0.06,
            },
        )
        .await
        {
            Ok(GatewayCommercialAdmissionDecision::Canonical(admission)) => Some(admission),
            Ok(GatewayCommercialAdmissionDecision::LegacyQuota) => {
                match enforce_project_quota(state.store.as_ref(), request_context.project_id(), 60)
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
        }
    };

    match relay_extend_video_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if video_response_is_processing(&response) {
                if let Err(response) = persist_pending_video_meter_fact(
                    &state,
                    request_context.context(),
                    &headers,
                    &response,
                    &pending_plan,
                )
                .await
                {
                    if let Some(admission) = commercial_admission.as_ref() {
                        if let Err(release_response) =
                            release_gateway_commercial_admission(&state, admission).await
                        {
                            return release_response;
                        }
                    }
                    return response;
                }
                return Json(response).into_response();
            }
            let Some(usage_model) = commercial_response_usage_id_or_single_data_item_id(&response)
            else {
                if let Some(admission) = commercial_admission.as_ref() {
                    if let Err(response) =
                        release_gateway_commercial_admission(&state, admission).await
                    {
                        return response;
                    }
                }
                return bad_gateway_openai_response(
                    "upstream video extend response missing usage id",
                );
            };
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = capture_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            if record_video_transform_usage_with_reference(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                &video_id,
                usage_model,
                60,
                0.06,
                commercial_response_usage_id_or_single_data_item_id(&response),
                pending_plan.usage_context.as_ref(),
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
            return bad_gateway_openai_response("failed to relay upstream video extend");
        }
    }

    let response = match local_video_transform_result(&video_id, &request.prompt, "extended") {
        Ok(response) => response,
        Err(response) => {
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = release_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            return response;
        }
    };
    let [item] = response.data.as_slice() else {
        if let Some(admission) = commercial_admission.as_ref() {
            if let Err(response) = release_gateway_commercial_admission(&state, admission).await {
                return response;
            }
        }
        return bad_gateway_openai_response("failed to process local video fallback");
    };
    let usage_model = item.id.as_str();

    if let Some(admission) = commercial_admission.as_ref() {
        if let Err(response) = capture_gateway_commercial_admission(&state, admission).await {
            return response;
        }
    }

    if record_video_transform_usage_with_reference(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
        usage_model,
        60,
        0.06,
        Some(usage_model),
        pending_plan.usage_context.as_ref(),
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

pub(crate) async fn video_edits_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    headers: HeaderMap,
    ExtractJson(request): ExtractJson<EditVideoRequest>,
) -> Response {
    let pending_plan = match plan_pending_video_commercial_workflow(
        &state,
        request_context.context(),
        request_context.tenant_id(),
        request_context.project_id(),
        &request.video_id,
        80,
    )
    .await
    {
        Ok(plan) => plan,
        Err(response) => return response,
    };
    let commercial_admission = if pending_plan.model_price.is_some() {
        pending_plan.admission.clone()
    } else {
        match begin_gateway_commercial_admission(
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
        }
    };

    match sdkwork_api_app_gateway::relay_edit_video_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if video_response_is_processing(&response) {
                if let Err(response) = persist_pending_video_meter_fact(
                    &state,
                    request_context.context(),
                    &headers,
                    &response,
                    &pending_plan,
                )
                .await
                {
                    if let Some(admission) = commercial_admission.as_ref() {
                        if let Err(release_response) =
                            release_gateway_commercial_admission(&state, admission).await
                        {
                            return release_response;
                        }
                    }
                    return response;
                }
                return Json(response).into_response();
            }
            let Some(usage_model) = commercial_response_usage_id_or_single_data_item_id(&response)
            else {
                if let Some(admission) = commercial_admission.as_ref() {
                    if let Err(response) =
                        release_gateway_commercial_admission(&state, admission).await
                    {
                        return response;
                    }
                }
                return bad_gateway_openai_response(
                    "upstream video edit response missing usage id",
                );
            };
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = capture_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            if record_video_transform_usage_with_reference(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                &request.video_id,
                usage_model,
                80,
                0.08,
                commercial_response_usage_id_or_single_data_item_id(&response),
                pending_plan.usage_context.as_ref(),
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
            return bad_gateway_openai_response("failed to relay upstream video edits");
        }
    }

    let response = match local_video_edits_result(&request) {
        Ok(response) => response,
        Err(response) => {
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = release_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            return response;
        }
    };
    let [item] = response.data.as_slice() else {
        if let Some(admission) = commercial_admission.as_ref() {
            if let Err(response) = release_gateway_commercial_admission(&state, admission).await {
                return response;
            }
        }
        return bad_gateway_openai_response("failed to process local video fallback");
    };
    let usage_model = item.id.as_str();

    if let Some(admission) = commercial_admission.as_ref() {
        if let Err(response) = capture_gateway_commercial_admission(&state, admission).await {
            return response;
        }
    }

    if record_video_transform_usage_with_reference(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        &request.video_id,
        usage_model,
        80,
        0.08,
        Some(usage_model),
        pending_plan.usage_context.as_ref(),
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

pub(crate) async fn video_extensions_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    headers: HeaderMap,
    ExtractJson(request): ExtractJson<ExtendVideoRequest>,
) -> Response {
    let pending_plan = match plan_pending_video_commercial_workflow(
        &state,
        request_context.context(),
        request_context.tenant_id(),
        request_context.project_id(),
        request.video_id.as_deref().unwrap_or("videos"),
        80,
    )
    .await
    {
        Ok(plan) => plan,
        Err(response) => return response,
    };
    let commercial_admission = if pending_plan.model_price.is_some() {
        pending_plan.admission.clone()
    } else {
        match begin_gateway_commercial_admission(
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
        }
    };

    match sdkwork_api_app_gateway::relay_extensions_video_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if video_response_is_processing(&response) {
                if let Err(response) = persist_pending_video_meter_fact(
                    &state,
                    request_context.context(),
                    &headers,
                    &response,
                    &pending_plan,
                )
                .await
                {
                    if let Some(admission) = commercial_admission.as_ref() {
                        if let Err(release_response) =
                            release_gateway_commercial_admission(&state, admission).await
                        {
                            return release_response;
                        }
                    }
                    return response;
                }
                return Json(response).into_response();
            }
            let route_key = request.video_id.as_deref().unwrap_or("videos");
            let Some(usage_model) = commercial_response_usage_id_or_single_data_item_id(&response)
            else {
                if let Some(admission) = commercial_admission.as_ref() {
                    if let Err(response) =
                        release_gateway_commercial_admission(&state, admission).await
                    {
                        return response;
                    }
                }
                return bad_gateway_openai_response(
                    "upstream video extensions response missing usage id",
                );
            };
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = capture_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            if record_video_transform_usage_with_reference(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                route_key,
                usage_model,
                80,
                0.08,
                commercial_response_usage_id_or_single_data_item_id(&response),
                pending_plan.usage_context.as_ref(),
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
            return bad_gateway_openai_response("failed to relay upstream video extensions");
        }
    }

    let response = match local_video_extensions_result(&request) {
        Ok(response) => response,
        Err(response) => {
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = release_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            return response;
        }
    };
    let route_key = request.video_id.as_deref().unwrap_or("videos");
    let [item] = response.data.as_slice() else {
        if let Some(admission) = commercial_admission.as_ref() {
            if let Err(response) = release_gateway_commercial_admission(&state, admission).await {
                return response;
            }
        }
        return bad_gateway_openai_response("failed to process local video fallback");
    };
    let usage_model = item.id.as_str();

    if let Some(admission) = commercial_admission.as_ref() {
        if let Err(response) = capture_gateway_commercial_admission(&state, admission).await {
            return response;
        }
    }

    if record_video_transform_usage_with_reference(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        route_key,
        usage_model,
        80,
        0.08,
        Some(usage_model),
        pending_plan.usage_context.as_ref(),
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
