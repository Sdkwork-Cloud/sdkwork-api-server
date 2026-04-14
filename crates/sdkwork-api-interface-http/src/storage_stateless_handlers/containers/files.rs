use super::*;

fn local_container_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_container_request",
        "Requested container was not found.",
    )
}

fn local_container_file_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_container_request",
        "Requested container file was not found.",
    )
}

pub(crate) async fn container_files_handler(
    request_context: StatelessGatewayRequest,
    Path(container_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateContainerFileRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ContainerFiles(&container_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream container file");
        }
    }

    let response = match sdkwork_api_app_gateway::create_container_file(
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
        &request,
    ) {
        Ok(response) => response,
        Err(error) => return local_container_file_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn container_files_list_handler(
    request_context: StatelessGatewayRequest,
    Path(container_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ContainerFilesList(&container_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream container files list");
        }
    }

    let response = match sdkwork_api_app_gateway::list_container_files(
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_container_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn container_file_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((container_id, file_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ContainerFilesRetrieve(&container_id, &file_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream container file retrieve");
        }
    }

    let response = match sdkwork_api_app_gateway::get_container_file(
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
        &file_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_container_file_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn container_file_delete_handler(
    request_context: StatelessGatewayRequest,
    Path((container_id, file_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ContainerFilesDelete(&container_id, &file_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream container file delete");
        }
    }

    let response = match sdkwork_api_app_gateway::delete_container_file(
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
        &file_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_container_file_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn container_file_content_handler(
    request_context: StatelessGatewayRequest,
    Path((container_id, file_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_stream_request(
        &request_context,
        ProviderRequest::ContainerFilesContent(&container_id, &file_id),
    )
    .await
    {
        Ok(Some(response)) => return upstream_passthrough_response(response),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream container file content");
        }
    }

    local_container_file_content_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
        &file_id,
    )
}
