use super::*;

fn local_batch_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_batch_request",
        "Requested batch was not found.",
    )
}

pub(crate) async fn batches_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateBatchRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Batches(&request)).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream batch");
        }
    }

    let response = match create_batch(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => response,
        Err(error) => return local_batch_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn batches_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::BatchesList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream batches list");
        }
    }

    let response = match list_batches(request_context.tenant_id(), request_context.project_id()) {
        Ok(response) => response,
        Err(error) => return local_batch_error_response(error),
    };

    Json(response).into_response()
}
