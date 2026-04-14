use super::*;

fn local_file_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_file",
        "Requested file was not found.",
    )
}

pub(crate) async fn files_handler(
    request_context: StatelessGatewayRequest,
    multipart: Multipart,
) -> Response {
    match parse_file_request(multipart).await {
        Ok(request) => {
            match relay_stateless_json_request(&request_context, ProviderRequest::Files(&request))
                .await
            {
                Ok(Some(response)) => return Json(response).into_response(),
                Ok(None) => {}
                Err(_) => {
                    return bad_gateway_openai_response("failed to relay upstream file");
                }
            }

            let response = match create_file(
                request_context.tenant_id(),
                request_context.project_id(),
                &request,
            ) {
                Ok(response) => response,
                Err(error) => return local_file_error_response(error),
            };

            Json(response).into_response()
        }
        Err(response) => response,
    }
}

pub(crate) async fn files_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::FilesList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream files list");
        }
    }

    let response = match list_files(request_context.tenant_id(), request_context.project_id()) {
        Ok(response) => response,
        Err(error) => return local_file_error_response(error),
    };

    Json(response).into_response()
}
