use super::*;

fn local_thread_run_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_thread_run_request",
        "Requested thread was not found.",
    )
}

pub(crate) async fn thread_and_run_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateThreadAndRunRequest>,
) -> Response {
    match relay_thread_and_run_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let Some(usage_model) = response_usage_id_or_single_data_item_id(&response) else {
                return bad_gateway_openai_response(
                    "upstream thread and run response missing usage id",
                );
            };
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                "threads/runs",
                usage_model,
                25,
                0.025,
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
            return bad_gateway_openai_response("failed to relay upstream thread and run");
        }
    }

    let response = match create_thread_and_run(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.assistant_id,
        request.model.as_deref(),
    ) {
        Ok(response) => response,
        Err(error) => return local_thread_run_error_response(error),
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        "threads/runs",
        response.id.as_str(),
        25,
        0.025,
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

pub(crate) async fn thread_runs_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(thread_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateRunRequest>,
) -> Response {
    match relay_thread_run_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let Some(run_id) = response.get("id").and_then(Value::as_str) else {
                return bad_gateway_openai_response("upstream thread run response missing id");
            };
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &thread_id,
                run_id,
                25,
                0.025,
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
            return bad_gateway_openai_response("failed to relay upstream thread run");
        }
    }

    let response = match create_thread_run(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &request.assistant_id,
        request.model.as_deref(),
    ) {
        Ok(response) => response,
        Err(error) => return local_thread_run_error_response(error),
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &thread_id,
        response.id.as_str(),
        25,
        0.025,
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
