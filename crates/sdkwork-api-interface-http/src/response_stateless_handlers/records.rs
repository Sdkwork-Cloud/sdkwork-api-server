use super::*;

fn local_response_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_response_request",
        "Requested response was not found.",
    )
}

pub(crate) async fn response_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(response_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ResponsesRetrieve(&response_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream response retrieve");
        }
    }

    let response = match get_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &response_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_response_not_found_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn response_input_items_list_handler(
    request_context: StatelessGatewayRequest,
    Path(response_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ResponsesInputItemsList(&response_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream response input items");
        }
    }

    let response = match list_response_input_items(
        request_context.tenant_id(),
        request_context.project_id(),
        &response_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_response_not_found_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn response_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(response_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ResponsesDelete(&response_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream response delete");
        }
    }

    let response = match delete_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &response_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_response_not_found_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn response_cancel_handler(
    request_context: StatelessGatewayRequest,
    Path(response_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ResponsesCancel(&response_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream response cancel");
        }
    }

    let response = match cancel_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &response_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_response_not_found_response(error),
    };

    Json(response).into_response()
}
