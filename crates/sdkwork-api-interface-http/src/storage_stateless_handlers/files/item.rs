use super::*;

fn local_file_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_file",
        "Requested file was not found.",
    )
}

pub(crate) async fn file_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(file_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::FilesRetrieve(&file_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream file retrieve");
        }
    }

    let response = match get_file(
        request_context.tenant_id(),
        request_context.project_id(),
        &file_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_file_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn file_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(file_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::FilesDelete(&file_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream file delete");
        }
    }

    let response = match delete_file(
        request_context.tenant_id(),
        request_context.project_id(),
        &file_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_file_error_response(error),
    };

    Json(response).into_response()
}
