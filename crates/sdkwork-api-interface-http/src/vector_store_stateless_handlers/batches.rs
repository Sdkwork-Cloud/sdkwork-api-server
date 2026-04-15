use super::*;
use sdkwork_api_contract_openai::vector_stores::{
    ListVectorStoreFilesResponse, VectorStoreFileBatchObject, VectorStoreFileObject,
};

fn local_vector_store_file_batch_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_vector_store_request",
        "Requested vector store file batch was not found.",
    )
}

fn vector_store_missing(vector_store_id: &str) -> bool {
    vector_store_id.trim().is_empty() || vector_store_id.ends_with("_missing")
}

fn batch_missing(batch_id: &str) -> bool {
    batch_id.trim().is_empty() || batch_id.ends_with("_missing")
}

fn local_vector_store_batch_placeholder(batch_id: &str) -> VectorStoreFileBatchObject {
    VectorStoreFileBatchObject::new(batch_id)
}

fn local_cancelled_vector_store_batch_placeholder(batch_id: &str) -> VectorStoreFileBatchObject {
    VectorStoreFileBatchObject::cancelled(batch_id)
}

fn local_vector_store_batch_files_placeholder() -> ListVectorStoreFilesResponse {
    ListVectorStoreFilesResponse::new(vec![VectorStoreFileObject::new("file_1")])
}

fn local_vector_store_file_batch_create_result(
    vector_store_id: &str,
    request: &CreateVectorStoreFileBatchRequest,
) -> std::result::Result<VectorStoreFileBatchObject, Response> {
    if vector_store_missing(vector_store_id) {
        return Err(local_vector_store_file_batch_error_response(
            anyhow::anyhow!("vector store not found"),
        ));
    }
    if request.file_ids.is_empty()
        || request
            .file_ids
            .iter()
            .any(|file_id| file_id.trim().is_empty())
    {
        return Err(local_gateway_invalid_or_bad_gateway_response(
            anyhow::anyhow!(
                "At least one local file id is required for local vector store fallback."
            ),
            "invalid_vector_store_request",
        ));
    }

    Ok(local_vector_store_batch_placeholder("vsfb_1"))
}

fn local_vector_store_file_batch_retrieve_result(
    vector_store_id: &str,
    batch_id: &str,
) -> std::result::Result<VectorStoreFileBatchObject, Response> {
    if vector_store_missing(vector_store_id) || batch_missing(batch_id) {
        return Err(local_vector_store_file_batch_error_response(
            anyhow::anyhow!("vector store file batch not found"),
        ));
    }

    Ok(local_vector_store_batch_placeholder(batch_id))
}

fn local_vector_store_file_batch_cancel_result(
    vector_store_id: &str,
    batch_id: &str,
) -> std::result::Result<VectorStoreFileBatchObject, Response> {
    if vector_store_missing(vector_store_id) || batch_missing(batch_id) {
        return Err(local_vector_store_file_batch_error_response(
            anyhow::anyhow!("vector store file batch not found"),
        ));
    }

    Ok(local_cancelled_vector_store_batch_placeholder(batch_id))
}

fn local_vector_store_file_batch_files_result(
    vector_store_id: &str,
    batch_id: &str,
) -> std::result::Result<ListVectorStoreFilesResponse, Response> {
    if vector_store_missing(vector_store_id) || batch_missing(batch_id) {
        return Err(local_vector_store_file_batch_error_response(
            anyhow::anyhow!("vector store file batch not found"),
        ));
    }

    Ok(local_vector_store_batch_files_placeholder())
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
    let response = match local_vector_store_file_batch_create_result(&vector_store_id, &request) {
        Ok(response) => response,
        Err(response) => return response,
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
    let response = match local_vector_store_file_batch_retrieve_result(&vector_store_id, &batch_id)
    {
        Ok(response) => response,
        Err(response) => return response,
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
    let response = match local_vector_store_file_batch_cancel_result(&vector_store_id, &batch_id) {
        Ok(response) => response,
        Err(response) => return response,
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
    let response = match local_vector_store_file_batch_files_result(&vector_store_id, &batch_id) {
        Ok(response) => response,
        Err(response) => return response,
    };

    Json(response).into_response()
}
