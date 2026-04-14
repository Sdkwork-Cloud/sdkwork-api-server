use super::*;

fn local_thread_run_step_error_response(error: anyhow::Error) -> Response {
    let message = error.to_string();
    if local_gateway_error_is_invalid_request(&message) {
        return invalid_request_openai_response(message, "invalid_thread_run_request");
    }
    if message.to_ascii_lowercase().contains("run step not found") {
        return local_gateway_error_response(error, "Requested thread run step was not found.");
    }
    if message.to_ascii_lowercase().contains("run not found") {
        return local_gateway_error_response(error, "Requested thread run was not found.");
    }

    local_gateway_error_response(error, "Requested thread was not found.")
}

pub(crate) async fn thread_run_steps_list_handler(
    request_context: StatelessGatewayRequest,
    Path((thread_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadRunStepsList(&thread_id, &run_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread run steps list");
        }
    }

    let response = match list_thread_run_steps(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_thread_run_step_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn thread_run_step_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((thread_id, run_id, step_id)): Path<(String, String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadRunStepsRetrieve(&thread_id, &run_id, &step_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream thread run step retrieve",
            );
        }
    }

    let response = match get_thread_run_step(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
        &step_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_thread_run_step_error_response(error),
    };

    Json(response).into_response()
}
