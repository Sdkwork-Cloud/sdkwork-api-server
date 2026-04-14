use super::*;

fn local_vector_store_file_batch_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_vector_store_request",
        "Requested vector store file batch was not found.",
    )
}

pub(crate) async fn vector_store_file_batches_handler(
    request_context: StatelessGatewayRequest,
    Path(vector_store_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateVectorStoreFileBatchRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoreFileBatches(&vector_store_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store file batch");
        }
    }
    let response = match create_vector_store_file_batch(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &request.file_ids,
    ) {
        Ok(response) => response,
        Err(error) => return local_vector_store_file_batch_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn vector_store_file_batch_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path((vector_store_id, batch_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoreFileBatchesRetrieve(&vector_store_id, &batch_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream vector store file batch retrieve",
            );
        }
    }
    let response = match get_vector_store_file_batch(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &batch_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_vector_store_file_batch_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn vector_store_file_batch_cancel_handler(
    request_context: StatelessGatewayRequest,
    Path((vector_store_id, batch_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoreFileBatchesCancel(&vector_store_id, &batch_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream vector store file batch cancel",
            );
        }
    }
    let response = match cancel_vector_store_file_batch(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &batch_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_vector_store_file_batch_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn vector_store_file_batch_files_handler(
    request_context: StatelessGatewayRequest,
    Path((vector_store_id, batch_id)): Path<(String, String)>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoreFileBatchesListFiles(&vector_store_id, &batch_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream vector store file batch files",
            );
        }
    }
    let response = match list_vector_store_file_batch_files(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &batch_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_vector_store_file_batch_error_response(error),
    };

    Json(response).into_response()
}
