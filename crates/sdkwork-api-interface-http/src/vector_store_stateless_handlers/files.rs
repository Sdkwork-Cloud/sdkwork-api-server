use super::*;

fn local_vector_store_files_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_vector_store_request",
        "Requested vector store was not found.",
    )
}

fn local_vector_store_file_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_vector_store_request",
        "Requested vector store file was not found.",
    )
}

pub(crate) async fn vector_store_files_handler(
    request_context: StatelessGatewayRequest,
    Path(vector_store_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateVectorStoreFileRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoreFiles(&vector_store_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store file");
        }
    }
    let response = match create_vector_store_file(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &request.file_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_vector_store_file_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn vector_store_files_list_handler(
    request_context: StatelessGatewayRequest,
    Path(vector_store_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoreFilesList(&vector_store_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store files list");
        }
    }
    let response = match list_vector_store_files(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_vector_store_files_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn vector_store_file_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((vector_store_id, file_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoreFilesRetrieve(&vector_store_id, &file_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream vector store file retrieve",
            );
        }
    }
    let response = match get_vector_store_file(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &file_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_vector_store_file_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn vector_store_file_delete_handler(
    request_context: StatelessGatewayRequest,
    Path((vector_store_id, file_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoreFilesDelete(&vector_store_id, &file_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream vector store file delete",
            );
        }
    }
    let response = match delete_vector_store_file(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &file_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_vector_store_file_error_response(error),
    };

    Json(response).into_response()
}
