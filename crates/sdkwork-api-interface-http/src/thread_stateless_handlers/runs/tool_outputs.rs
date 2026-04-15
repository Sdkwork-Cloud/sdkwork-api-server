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

pub(crate) async fn thread_run_submit_tool_outputs_handler(
    request_context: StatelessGatewayRequest,
    Path((thread_id, run_id)): Path<(String, String)>,
    ExtractJson(request): ExtractJson<SubmitToolOutputsRunRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadRunsSubmitToolOutputs(&thread_id, &run_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream thread run submit tool outputs",
            );
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

    Json(response).into_response()
}
