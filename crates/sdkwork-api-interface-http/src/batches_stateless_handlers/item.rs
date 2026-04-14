use super::*;

fn local_batch_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_batch_request",
        "Requested batch was not found.",
    )
}

pub(crate) async fn batch_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(batch_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::BatchesRetrieve(&batch_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream batch retrieve");
        }
    }

    let response = match get_batch(
        request_context.tenant_id(),
        request_context.project_id(),
        &batch_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_batch_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn batch_cancel_handler(
    request_context: StatelessGatewayRequest,
    Path(batch_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::BatchesCancel(&batch_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream batch cancel");
        }
    }

    let response = match cancel_batch(
        request_context.tenant_id(),
        request_context.project_id(),
        &batch_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_batch_error_response(error),
    };

    Json(response).into_response()
}
