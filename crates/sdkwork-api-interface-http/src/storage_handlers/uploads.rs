use super::*;

fn local_upload_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_upload_request",
        "Requested upload was not found.",
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

    let response = match create_upload(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => response,
        Err(error) => return local_upload_error_response(error),
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

    let response = match create_upload_part(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => response,
        Err(error) => return local_upload_error_response(error),
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

    let response = match complete_upload(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => response,
        Err(error) => return local_upload_error_response(error),
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

    let response = match cancel_upload(
        request_context.tenant_id(),
        request_context.project_id(),
        &upload_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_upload_error_response(error),
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
