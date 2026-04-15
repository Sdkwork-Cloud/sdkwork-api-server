use super::*;

fn local_thread_run_error_response(error: anyhow::Error) -> Response {
    let message = error.to_string();
    if message.contains("assistant_id is required") {
        return invalid_request_openai_response(message, "invalid_assistant_id");
    }

    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_thread_run_request",
        "Requested thread was not found.",
    )
}

pub(crate) async fn thread_and_run_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateThreadAndRunRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::ThreadsRuns(&request))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
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

    Json(response).into_response()
}

pub(crate) async fn thread_runs_handler(
    request_context: StatelessGatewayRequest,
    Path(thread_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateRunRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadRuns(&thread_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
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

    Json(response).into_response()
}
