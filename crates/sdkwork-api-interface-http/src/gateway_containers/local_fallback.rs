fn local_container_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_error_response(error, "Requested container was not found.")
}

fn local_container_file_not_found_response(error: anyhow::Error) -> Response {
    if error
        .to_string()
        .to_ascii_lowercase()
        .contains("container file not found")
    {
        return not_found_openai_response("Requested container file was not found.");
    }

    local_container_not_found_response(error)
}

fn local_container_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
) -> std::result::Result<ContainerObject, Response> {
    sdkwork_api_app_gateway::get_container(tenant_id, project_id, container_id)
        .map_err(local_container_not_found_response)
}

fn local_container_retrieve_response(
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
) -> Response {
    match local_container_retrieve_result(tenant_id, project_id, container_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_container_delete_result(
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
) -> std::result::Result<DeleteContainerResponse, Response> {
    sdkwork_api_app_gateway::delete_container(tenant_id, project_id, container_id)
        .map_err(local_container_not_found_response)
}

fn local_container_delete_response(
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
) -> Response {
    match local_container_delete_result(tenant_id, project_id, container_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_container_file_create_result(
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
    request: &CreateContainerFileRequest,
) -> std::result::Result<ContainerFileObject, Response> {
    sdkwork_api_app_gateway::create_container_file(tenant_id, project_id, container_id, request)
        .map_err(local_container_not_found_response)
}

fn local_container_files_list_result(
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
) -> std::result::Result<ListContainerFilesResponse, Response> {
    sdkwork_api_app_gateway::list_container_files(tenant_id, project_id, container_id)
        .map_err(local_container_not_found_response)
}

fn local_container_files_list_response(
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
) -> Response {
    match local_container_files_list_result(tenant_id, project_id, container_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_container_file_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
    file_id: &str,
) -> std::result::Result<ContainerFileObject, Response> {
    sdkwork_api_app_gateway::get_container_file(tenant_id, project_id, container_id, file_id)
        .map_err(local_container_file_not_found_response)
}

fn local_container_file_retrieve_response(
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
    file_id: &str,
) -> Response {
    match local_container_file_retrieve_result(tenant_id, project_id, container_id, file_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_container_file_delete_result(
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
    file_id: &str,
) -> std::result::Result<DeleteContainerFileResponse, Response> {
    sdkwork_api_app_gateway::delete_container_file(tenant_id, project_id, container_id, file_id)
        .map_err(local_container_file_not_found_response)
}

fn local_container_file_delete_response(
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
    file_id: &str,
) -> Response {
    match local_container_file_delete_result(tenant_id, project_id, container_id, file_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_container_file_content_result(
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
    file_id: &str,
) -> std::result::Result<Vec<u8>, Response> {
    sdkwork_api_app_gateway::container_file_content(tenant_id, project_id, container_id, file_id)
        .map_err(local_container_file_not_found_response)
}

fn local_container_file_content_response(
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
    file_id: &str,
) -> Response {
    let bytes =
        match local_container_file_content_result(tenant_id, project_id, container_id, file_id) {
            Ok(bytes) => bytes,
            Err(response) => return response,
        };
    Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .body(Body::from(bytes))
        .expect("valid local container file content response")
}
