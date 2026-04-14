use super::*;

fn local_fine_tuning_job_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_fine_tuning_request",
        "Requested fine tuning job was not found.",
    )
}

pub(crate) async fn fine_tuning_job_events_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match relay_list_fine_tuning_job_events_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "fine_tuning",
                &fine_tuning_job_id,
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
            return bad_gateway_openai_response("failed to relay upstream fine tuning job events");
        }
    }

    let response = match list_fine_tuning_job_events(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_fine_tuning_job_error_response(error),
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "fine_tuning",
        &fine_tuning_job_id,
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

pub(crate) async fn fine_tuning_job_checkpoints_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(fine_tuning_job_id): Path<String>,
) -> Response {
    match relay_list_fine_tuning_job_checkpoints_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "fine_tuning",
                &fine_tuning_job_id,
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
                "failed to relay upstream fine tuning job checkpoints",
            );
        }
    }

    let response = match list_fine_tuning_job_checkpoints(
        request_context.tenant_id(),
        request_context.project_id(),
        &fine_tuning_job_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_fine_tuning_job_error_response(error),
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "fine_tuning",
        &fine_tuning_job_id,
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
