use super::*;

fn local_thread_run_error_response(error: anyhow::Error) -> Response {
    let message = error.to_string();
    if message.to_ascii_lowercase().contains("run not found") {
        return local_gateway_invalid_or_not_found_response(
            error,
            "invalid_thread_run_request",
            "Requested thread run was not found.",
        );
    }

    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_thread_run_request",
        "Requested thread was not found.",
    )
}

pub(crate) async fn thread_runs_list_handler(
    request_context: StatelessGatewayRequest,
    Path(thread_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadRunsList(&thread_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread runs list");
        }
    }

    let response = match list_thread_runs(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_thread_run_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn thread_run_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((thread_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadRunsRetrieve(&thread_id, &run_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread run retrieve");
        }
    }

    let response = match get_thread_run(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_thread_run_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn thread_run_update_handler(
    request_context: StatelessGatewayRequest,
    Path((thread_id, run_id)): Path<(String, String)>,
    ExtractJson(request): ExtractJson<UpdateRunRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadRunsUpdate(&thread_id, &run_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread run update");
        }
    }

    let response = match update_thread_run(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_thread_run_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn thread_run_cancel_handler(
    request_context: StatelessGatewayRequest,
    Path((thread_id, run_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadRunsCancel(&thread_id, &run_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread run cancel");
        }
    }

    let response = match cancel_thread_run(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
        &run_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_thread_run_error_response(error),
    };

    Json(response).into_response()
}
