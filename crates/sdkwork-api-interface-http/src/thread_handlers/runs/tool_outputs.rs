use super::*;

fn local_thread_run_tool_outputs_error_response(error: anyhow::Error) -> Response {
    let message = error.to_string();
    if local_gateway_error_is_invalid_request(&message) {
        return invalid_request_openai_response(message, "invalid_thread_run_request");
    }

    if message.to_ascii_lowercase().contains("run not found") {
        return local_gateway_error_response(error, "Requested run was not found.");
    }

    local_gateway_error_response(error, "Requested thread was not found.")
}

pub(crate) async fn thread_run_submit_tool_outputs_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((thread_id, run_id)): Path<(String, String)>,
    ExtractJson(request): ExtractJson<SubmitToolOutputsRunRequest>,
) -> Response {
    match relay_submit_thread_run_tool_outputs_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "assistants",
                &thread_id,
                &run_id,
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
            return bad_gateway_openai_response("failed to relay upstream thread run tool outputs");
        }
    }

    let tool_outputs = request
        .tool_outputs
        .iter()
        .map(|output| (output.tool_call_id.as_str(), output.output.as_str()))
        .collect();
    let response = match submit_thread_run_tool_outputs(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
        tool_outputs,
    ) {
        Ok(response) => response,
        Err(error) => return local_thread_run_tool_outputs_error_response(error),
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "assistants",
        &thread_id,
        &run_id,
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
