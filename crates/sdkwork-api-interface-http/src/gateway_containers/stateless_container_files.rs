async fn container_files_handler(
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

    match local_container_file_create_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
        &request,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

async fn container_files_list_handler(
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

    local_container_files_list_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
    )
}

async fn container_file_retrieve_handler(
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

    local_container_file_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
        &file_id,
    )
}

async fn container_file_delete_handler(
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

    local_container_file_delete_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
        &file_id,
    )
}

async fn container_file_content_handler(
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
