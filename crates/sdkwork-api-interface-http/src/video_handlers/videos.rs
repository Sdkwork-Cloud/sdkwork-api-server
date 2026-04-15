#![allow(clippy::too_many_arguments)]

use super::*;
use crate::gateway_commercial::{
    begin_gateway_commercial_admission, capture_gateway_commercial_admission,
    enforce_project_quota,
    record_gateway_usage_for_project_with_route_key_and_tokens_reference_and_media_with_context,
    release_gateway_commercial_admission, BillingMediaMetrics, GatewayCommercialAdmissionDecision,
    GatewayCommercialAdmissionSpec,
};
use axum::http::HeaderMap;
use sdkwork_api_app_gateway::PlannedExecutionUsageContext;
use sdkwork_api_contract_openai::videos::{DeleteVideoResponse, VideoObject, VideosResponse};

fn local_video_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_error_response(error, "Requested video was not found.")
}

fn local_video_asset_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_error_response(error, "Requested video asset was not found.")
}

fn video_missing(video_id: &str) -> bool {
    video_id.trim().is_empty() || video_id.ends_with("_missing")
}

fn local_video_placeholder(video_id: &str) -> VideoObject {
    VideoObject::new(video_id, format!("https://example.com/{video_id}.mp4"))
}

fn local_videos_placeholder(video_id: &str) -> VideosResponse {
    VideosResponse::new(vec![local_video_placeholder(video_id)])
}

fn local_video_create_result(
    request: &CreateVideoRequest,
) -> std::result::Result<VideosResponse, Response> {
    if request.model.trim().is_empty() {
        return Err(local_gateway_invalid_or_bad_gateway_response(
            anyhow::anyhow!("Video model is required."),
            "invalid_video_request",
        ));
    }
    if request.prompt.trim().is_empty() {
        return Err(local_gateway_invalid_or_bad_gateway_response(
            anyhow::anyhow!("Video prompt is required."),
            "invalid_video_request",
        ));
    }

    Ok(local_videos_placeholder("video_1"))
}

fn local_videos_list_result() -> VideosResponse {
    local_videos_placeholder("video_1")
}

fn local_video_retrieve_result(video_id: &str) -> std::result::Result<VideoObject, Response> {
    if video_missing(video_id) {
        return Err(local_video_not_found_response(anyhow::anyhow!(
            "video not found"
        )));
    }

    Ok(local_video_placeholder(video_id))
}

fn local_video_delete_result(video_id: &str) -> std::result::Result<DeleteVideoResponse, Response> {
    if video_missing(video_id) {
        return Err(local_video_not_found_response(anyhow::anyhow!(
            "video not found"
        )));
    }

    Ok(DeleteVideoResponse::deleted(video_id))
}

fn local_video_content_result(video_id: &str) -> std::result::Result<Vec<u8>, Response> {
    if video_missing(video_id) {
        return Err(local_video_asset_not_found_response(anyhow::anyhow!(
            "video not found"
        )));
    }

    Ok(b"LOCAL-VIDEO".to_vec())
}

fn local_video_content_response(video_id: &str) -> Response {
    let bytes = match local_video_content_result(video_id) {
        Ok(bytes) => bytes,
        Err(response) => return response,
    };

    match Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(axum::http::header::CONTENT_TYPE, "video/mp4")
        .body(Body::from(bytes))
    {
        Ok(response) => response,
        Err(_) => bad_gateway_openai_response("failed to process local video content fallback"),
    }
}

async fn record_video_usage_with_reference(
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

pub(crate) async fn videos_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    headers: HeaderMap,
    ExtractJson(request): ExtractJson<CreateVideoRequest>,
) -> Response {
    let pending_plan = match plan_pending_video_commercial_workflow(
        &state,
        request_context.context(),
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
        90,
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
                quoted_amount: 0.09,
            },
        )
        .await
        {
            Ok(GatewayCommercialAdmissionDecision::Canonical(admission)) => Some(admission),
            Ok(GatewayCommercialAdmissionDecision::LegacyQuota) => {
                match enforce_project_quota(state.store.as_ref(), request_context.project_id(), 90)
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

    match relay_video_from_store(
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
            let Some(usage_model) = response_usage_id_or_single_data_item_id(&response) else {
                if let Some(admission) = commercial_admission.as_ref() {
                    if let Err(response) =
                        release_gateway_commercial_admission(&state, admission).await
                    {
                        return response;
                    }
                }
                return bad_gateway_openai_response("upstream video response missing usage id");
            };
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = capture_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            if record_video_usage_with_reference(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                &request.model,
                usage_model,
                90,
                0.09,
                response_usage_id_or_single_data_item_id(&response),
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
            return bad_gateway_openai_response("failed to relay upstream video create");
        }
    }

    let response = match local_video_create_result(&request) {
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
    let usage_model = match response.data.as_slice() {
        [video] => video.id.as_str(),
        _ => request.model.as_str(),
    };

    if let Some(admission) = commercial_admission.as_ref() {
        if let Err(response) = capture_gateway_commercial_admission(&state, admission).await {
            return response;
        }
    }

    if record_video_usage_with_reference(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
        usage_model,
        90,
        0.09,
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

pub(crate) async fn videos_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Response {
    match relay_list_videos_from_store(
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
                "videos",
                "videos",
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
            return bad_gateway_openai_response("failed to relay upstream videos list");
        }
    }

    let response = local_videos_list_result();

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        "videos",
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

pub(crate) async fn video_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(video_id): Path<String>,
) -> Response {
    match relay_get_video_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
    )
    .await
    {
        Ok(Some(response)) => {
            match reconcile_pending_video_retrieval(
                &state,
                request_context.context(),
                request_context.tenant_id(),
                request_context.project_id(),
                &response,
            )
            .await
            {
                Ok(true) => return Json(response).into_response(),
                Ok(false) => {}
                Err(response) => return response,
            }
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "videos",
                &video_id,
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
        Ok(None) => {
            let provider_id = match pending_video_provider_id(
                &state,
                request_context.tenant_id(),
                request_context.project_id(),
                &video_id,
            )
            .await
            {
                Ok(provider_id) => provider_id,
                Err(response) => return response,
            };
            if let Some(provider_id) = provider_id {
                match relay_get_video_from_store_with_provider_id(
                    state.store.as_ref(),
                    &state.secret_manager,
                    request_context.tenant_id(),
                    &provider_id,
                    &video_id,
                )
                .await
                {
                    Ok(Some(response)) => {
                        match reconcile_pending_video_retrieval(
                            &state,
                            request_context.context(),
                            request_context.tenant_id(),
                            request_context.project_id(),
                            &response,
                        )
                        .await
                        {
                            Ok(true) => return Json(response).into_response(),
                            Ok(false) => {}
                            Err(response) => return response,
                        }
                        if record_gateway_usage_for_project(
                            state.store.as_ref(),
                            request_context.tenant_id(),
                            request_context.project_id(),
                            "videos",
                            &video_id,
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
                            "failed to relay upstream video retrieve",
                        );
                    }
                }
            }
        }
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video retrieve");
        }
    }

    let response = match local_video_retrieve_result(&video_id) {
        Ok(response) => response,
        Err(response) => return response,
    };
    let response_value = match serde_json::to_value(&response) {
        Ok(value) => value,
        Err(_) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to encode local video response",
            )
                .into_response();
        }
    };

    match reconcile_pending_video_retrieval(
        &state,
        request_context.context(),
        request_context.tenant_id(),
        request_context.project_id(),
        &response_value,
    )
    .await
    {
        Ok(true) => return Json(response).into_response(),
        Ok(false) => {}
        Err(response) => return response,
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        &video_id,
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

pub(crate) async fn video_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(video_id): Path<String>,
) -> Response {
    match relay_delete_video_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "videos",
                &video_id,
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
            return bad_gateway_openai_response("failed to relay upstream video delete");
        }
    }

    let response = match local_video_delete_result(&video_id) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        &video_id,
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

pub(crate) async fn video_content_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(video_id): Path<String>,
) -> Response {
    match relay_video_content_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "videos",
                &video_id,
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

            return upstream_passthrough_response(response);
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video content");
        }
    }

    let response = local_video_content_response(&video_id);
    if !response.status().is_success() {
        return response;
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        &video_id,
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

    response
}
