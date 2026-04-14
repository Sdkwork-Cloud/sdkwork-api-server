async fn files_handler(request_context: StatelessGatewayRequest, multipart: Multipart) -> Response {
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

            local_file_create_response(
                request_context.tenant_id(),
                request_context.project_id(),
                &request,
            )
        }
        Err(response) => response,
    }
}

async fn files_list_handler(request_context: StatelessGatewayRequest) -> Response {
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

async fn file_retrieve_handler(
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

    local_file_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &file_id,
    )
}

async fn file_delete_handler(
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

    local_file_delete_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &file_id,
    )
}

async fn file_content_handler(
    request_context: StatelessGatewayRequest,
    Path(file_id): Path<String>,
) -> Response {
    match relay_stateless_stream_request(&request_context, ProviderRequest::FilesContent(&file_id))
        .await
    {
        Ok(Some(response)) => return upstream_passthrough_response(response),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream file content");
        }
    }

    local_file_content_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &file_id,
    )
}


fn local_file_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(error, "invalid_file", "Requested file was not found.")
}

fn local_file_content_response(tenant_id: &str, project_id: &str, file_id: &str) -> Response {
    match file_content(tenant_id, project_id, file_id) {
        Ok(bytes) => match Response::builder()
            .status(axum::http::StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/jsonl")
            .body(Body::from(bytes))
        {
            Ok(response) => response,
            Err(_) => bad_gateway_openai_response("failed to process local file content fallback"),
        },
        Err(error) => local_file_error_response(error),
    }
}

fn local_file_not_found_response(error: anyhow::Error) -> Response {
    local_file_error_response(error)
}

fn local_file_create_error_response(error: anyhow::Error) -> Response {
    local_file_error_response(error)
}

fn local_file_create_result(
    tenant_id: &str,
    project_id: &str,
    request: &CreateFileRequest,
) -> std::result::Result<FileObject, Response> {
    create_file(tenant_id, project_id, request).map_err(local_file_create_error_response)
}

fn local_file_create_response(
    tenant_id: &str,
    project_id: &str,
    request: &CreateFileRequest,
) -> Response {
    match local_file_create_result(tenant_id, project_id, request) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_file_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    file_id: &str,
) -> std::result::Result<FileObject, Response> {
    get_file(tenant_id, project_id, file_id).map_err(local_file_not_found_response)
}

fn local_file_retrieve_response(tenant_id: &str, project_id: &str, file_id: &str) -> Response {
    match local_file_retrieve_result(tenant_id, project_id, file_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_file_delete_result(
    tenant_id: &str,
    project_id: &str,
    file_id: &str,
) -> std::result::Result<DeleteFileResponse, Response> {
    delete_file(tenant_id, project_id, file_id).map_err(local_file_not_found_response)
}

fn local_file_delete_response(tenant_id: &str, project_id: &str, file_id: &str) -> Response {
    match local_file_delete_result(tenant_id, project_id, file_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}


async fn files_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    multipart: Multipart,
) -> Response {
    let request = match parse_file_request(multipart).await {
        Ok(request) => request,
        Err(response) => return response,
    };

    match relay_file_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let Some(file_id) = response.get("id").and_then(Value::as_str) else {
                return bad_gateway_openai_response("upstream file response missing id");
            };
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "files",
                &request.purpose,
                file_id,
                5,
                0.005,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream file");
        }
    }

    let response = match local_file_create_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "files",
        &request.purpose,
        response.id.as_str(),
        5,
        0.005,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn files_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Response {
    match relay_list_files_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "files",
                "list",
                1,
                0.001,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream files list");
        }
    }

    let response = match list_files(request_context.tenant_id(), request_context.project_id()) {
        Ok(response) => response,
        Err(error) => return local_file_error_response(error),
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "files",
        "list",
        1,
        0.001,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn file_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(file_id): Path<String>,
) -> Response {
    match relay_get_file_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &file_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "files",
                &file_id,
                1,
                0.001,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream file retrieve");
        }
    }

    let response = match local_file_retrieve_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &file_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "files",
        &file_id,
        1,
        0.001,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn file_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(file_id): Path<String>,
) -> Response {
    match relay_delete_file_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &file_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "files",
                &file_id,
                1,
                0.001,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream file delete");
        }
    }

    let response = match local_file_delete_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &file_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "files",
        &file_id,
        1,
        0.001,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

async fn file_content_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(file_id): Path<String>,
) -> Response {
    match relay_file_content_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &file_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "files",
                &file_id,
                1,
                0.001,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return upstream_passthrough_response(response);
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream file content");
        }
    }

    let response = local_file_content_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &file_id,
    );
    if !response.status().is_success() {
        return response;
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "files",
        &file_id,
        1,
        0.001,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    response
}


async fn parse_file_request(mut multipart: Multipart) -> Result<CreateFileRequest, Response> {
    let mut purpose = None;
    let mut filename = None;
    let mut bytes = None;
    let mut content_type = None;

    while let Some(field) = multipart.next_field().await.map_err(bad_multipart)? {
        match field.name() {
            Some("purpose") => {
                purpose = Some(field.text().await.map_err(bad_multipart)?);
            }
            Some("file") => {
                filename = field.file_name().map(ToOwned::to_owned);
                content_type = field.content_type().map(ToOwned::to_owned);
                bytes = Some(field.bytes().await.map_err(bad_multipart)?.to_vec());
            }
            _ => {}
        }
    }

    let mut request = CreateFileRequest::new(
        purpose.ok_or_else(missing_multipart_field)?,
        filename.ok_or_else(missing_multipart_field)?,
        bytes.ok_or_else(missing_multipart_field)?,
    );
    if let Some(content_type) = content_type {
        request = request.with_content_type(content_type);
    }
    Ok(request)
}

