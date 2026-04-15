use super::*;
use sdkwork_api_contract_openai::uploads::{UploadObject, UploadPartObject};

const LOCAL_UPLOAD_BACKEND_UNSUPPORTED: &str =
    "Local upload fallback is not supported without an upstream provider.";
const LOCAL_UPLOAD_PART_PERSISTENCE_REQUIRED: &str =
    "Persisted local upload part state is required for local part creation.";
const LOCAL_UPLOAD_NOT_FOUND: &str = "upload not found";

fn local_upload_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_upload_request",
        "Requested upload was not found.",
    )
}

fn local_upload_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_upload_request",
        "Requested upload session was not found.",
    )
}

fn response_upload_id(response: &Value) -> Option<&str> {
    response
        .get("id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|id| !id.is_empty())
}

fn response_upload_status(response: &Value) -> Option<&str> {
    response
        .get("status")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|status| !status.is_empty())
}

fn upload_missing(upload_id: &str) -> bool {
    upload_id.trim().is_empty() || upload_id.ends_with("_missing")
}

fn local_upload_placeholder(request: &CreateUploadRequest) -> UploadObject {
    UploadObject::with_details(
        "upload_1",
        request.filename.clone(),
        request.purpose.clone(),
        request.mime_type.clone(),
        request.bytes,
        vec![],
    )
}

fn local_upload_part_placeholder(upload_id: &str) -> UploadPartObject {
    UploadPartObject::new("part_1", upload_id)
}

fn local_completed_upload_placeholder(request: &CompleteUploadRequest) -> UploadObject {
    UploadObject::completed(
        request.upload_id.clone(),
        "input.jsonl",
        "batch",
        "application/octet-stream",
        0,
        request.part_ids.clone(),
    )
}

fn local_cancelled_upload_placeholder(upload_id: &str) -> UploadObject {
    UploadObject::cancelled(
        upload_id,
        "input.jsonl",
        "batch",
        "application/octet-stream",
        0,
        vec![],
    )
}

fn local_upload_result(
    tenant_id: &str,
    project_id: &str,
    request: &CreateUploadRequest,
) -> std::result::Result<UploadObject, Response> {
    match create_upload(tenant_id, project_id, request) {
        Ok(response) => Ok(response),
        Err(error) if error.to_string().contains(LOCAL_UPLOAD_BACKEND_UNSUPPORTED) => {
            Ok(local_upload_placeholder(request))
        }
        Err(error) => Err(local_upload_error_response(error)),
    }
}

fn local_upload_part_result(
    tenant_id: &str,
    project_id: &str,
    request: &AddUploadPartRequest,
) -> std::result::Result<UploadPartObject, Response> {
    if upload_missing(&request.upload_id) {
        return Err(local_upload_not_found_response(anyhow::anyhow!(
            LOCAL_UPLOAD_NOT_FOUND
        )));
    }

    match create_upload_part(tenant_id, project_id, request) {
        Ok(response) => Ok(response),
        Err(error)
            if error
                .to_string()
                .contains(LOCAL_UPLOAD_PART_PERSISTENCE_REQUIRED)
                || error.to_string().contains(LOCAL_UPLOAD_NOT_FOUND) =>
        {
            Ok(local_upload_part_placeholder(&request.upload_id))
        }
        Err(error) => Err(local_upload_error_response(error)),
    }
}

fn local_upload_complete_result(
    tenant_id: &str,
    project_id: &str,
    request: &CompleteUploadRequest,
) -> std::result::Result<UploadObject, Response> {
    if upload_missing(&request.upload_id) {
        return Err(local_upload_not_found_response(anyhow::anyhow!(
            LOCAL_UPLOAD_NOT_FOUND
        )));
    }

    match complete_upload(tenant_id, project_id, request) {
        Ok(response) => Ok(response),
        Err(error) if error.to_string().contains(LOCAL_UPLOAD_NOT_FOUND) => {
            Ok(local_completed_upload_placeholder(request))
        }
        Err(error) => Err(local_upload_error_response(error)),
    }
}

fn local_upload_cancel_result(
    tenant_id: &str,
    project_id: &str,
    upload_id: &str,
) -> std::result::Result<UploadObject, Response> {
    if upload_missing(upload_id) {
        return Err(local_upload_not_found_response(anyhow::anyhow!(
            LOCAL_UPLOAD_NOT_FOUND
        )));
    }

    match cancel_upload(tenant_id, project_id, upload_id) {
        Ok(response) => Ok(response),
        Err(error) if error.to_string().contains(LOCAL_UPLOAD_NOT_FOUND) => {
            Ok(local_cancelled_upload_placeholder(upload_id))
        }
        Err(error) => Err(local_upload_error_response(error)),
    }
}

pub(crate) async fn uploads_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateUploadRequest>,
) -> Response {
    match relay_upload_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let Some(upload_id) = response_upload_id(&response) else {
                return bad_gateway_openai_response("upstream upload response missing id");
            };
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "uploads",
                &request.purpose,
                upload_id,
                8,
                0.008,
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
            return bad_gateway_openai_response("failed to relay upstream upload");
        }
    }

    let response = match local_upload_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "uploads",
        &request.purpose,
        response.id.as_str(),
        8,
        0.008,
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

pub(crate) async fn upload_parts_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(upload_id): Path<String>,
    multipart: Multipart,
) -> Response {
    let request = match parse_upload_part_request(upload_id, multipart).await {
        Ok(request) => request,
        Err(response) => return response,
    };

    match relay_upload_part_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let Some(part_id) = response_upload_id(&response) else {
                return bad_gateway_openai_response("upstream upload part response missing id");
            };
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "uploads",
                &request.upload_id,
                part_id,
                4,
                0.004,
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
            return bad_gateway_openai_response("failed to relay upstream upload part");
        }
    }

    let response = match local_upload_part_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "uploads",
        &request.upload_id,
        response.id.as_str(),
        4,
        0.004,
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

pub(crate) async fn upload_complete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(upload_id): Path<String>,
    ExtractJson(mut request): ExtractJson<CompleteUploadRequest>,
) -> Response {
    request.upload_id = upload_id;

    match relay_complete_upload_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if response_upload_id(&response).is_none()
                || response_upload_status(&response) != Some("completed")
            {
                return bad_gateway_openai_response(
                    "upstream upload completion response missing id or completed status",
                );
            }
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "uploads",
                &request.upload_id,
                4,
                0.004,
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
            return bad_gateway_openai_response("failed to relay upstream upload completion");
        }
    }

    let response = match local_upload_complete_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };
    if response.id.trim().is_empty() || response.status != "completed" {
        return bad_gateway_openai_response("failed to process local upload completion fallback");
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "uploads",
        &request.upload_id,
        4,
        0.004,
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

pub(crate) async fn upload_cancel_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(upload_id): Path<String>,
) -> Response {
    match relay_cancel_upload_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &upload_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if response_upload_id(&response).is_none()
                || response_upload_status(&response) != Some("cancelled")
            {
                return bad_gateway_openai_response(
                    "upstream upload cancellation response missing id or cancelled status",
                );
            }
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "uploads",
                &upload_id,
                4,
                0.004,
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
            return bad_gateway_openai_response("failed to relay upstream upload cancellation");
        }
    }

    let response = match local_upload_cancel_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &upload_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };
    if response.id.trim().is_empty() || response.status != "cancelled" {
        return bad_gateway_openai_response("failed to process local upload cancellation fallback");
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "uploads",
        &upload_id,
        4,
        0.004,
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
