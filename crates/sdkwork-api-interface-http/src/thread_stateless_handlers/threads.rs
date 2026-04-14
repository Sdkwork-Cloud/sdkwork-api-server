use super::*;

fn local_thread_error_response(error: anyhow::Error) -> Response {
    let message = error.to_string();
    if local_gateway_error_is_invalid_request(&message) {
        return invalid_request_openai_response(message, "invalid_thread_request");
    }

    local_gateway_error_response(error, "Requested thread was not found.")
}

pub(crate) async fn threads_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateThreadRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Threads(&request)).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread");
        }
    }

    let response = match create_thread(request_context.tenant_id(), request_context.project_id()) {
        Ok(response) => response,
        Err(error) => return local_thread_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn thread_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(thread_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadsRetrieve(&thread_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread retrieve");
        }
    }

    let response = match get_thread(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
    ) {
        Ok(response) => response,
        Err(error) => {
            return local_gateway_error_response(error, "Requested thread was not found.")
        }
    };

    Json(response).into_response()
}

pub(crate) async fn thread_update_handler(
    request_context: StatelessGatewayRequest,
    Path(thread_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateThreadRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ThreadsUpdate(&thread_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread update");
        }
    }

    let response = match update_thread(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
    ) {
        Ok(response) => response,
        Err(error) => {
            return local_gateway_error_response(error, "Requested thread was not found.")
        }
    };

    Json(response).into_response()
}

pub(crate) async fn thread_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(thread_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::ThreadsDelete(&thread_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream thread delete");
        }
    }

    let response = match delete_thread(
        request_context.tenant_id(),
        request_context.project_id(),
        &thread_id,
    ) {
        Ok(response) => response,
        Err(error) => {
            return local_gateway_error_response(error, "Requested thread was not found.")
        }
    };

    Json(response).into_response()
}
