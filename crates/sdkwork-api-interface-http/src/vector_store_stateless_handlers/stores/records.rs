use super::*;
use sdkwork_api_contract_openai::vector_stores::{
    DeleteVectorStoreResponse, ListVectorStoresResponse, VectorStoreObject,
};

fn local_vector_store_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_vector_store_request",
        "Requested vector store was not found.",
    )
}

fn vector_store_missing(vector_store_id: &str) -> bool {
    vector_store_id.trim().is_empty() || vector_store_id.ends_with("_missing")
}

fn local_vector_store_placeholder(vector_store_id: &str, name: &str) -> VectorStoreObject {
    VectorStoreObject::new(vector_store_id, name)
}

fn local_vector_stores_placeholder() -> ListVectorStoresResponse {
    ListVectorStoresResponse::new(vec![VectorStoreObject::new("vs_1", "kb-main")])
}

fn local_vector_store_update_name(request: &UpdateVectorStoreRequest) -> Result<&str, Response> {
    request.name.as_deref().ok_or_else(|| {
        invalid_request_openai_response(
            "Vector store name is required for local fallback updates.",
            "invalid_vector_store_request",
        )
    })
}

fn local_vector_store_create_result(
    request: &CreateVectorStoreRequest,
) -> std::result::Result<VectorStoreObject, Response> {
    if request.name.trim().is_empty() {
        return Err(local_gateway_invalid_or_bad_gateway_response(
            anyhow::anyhow!("Vector store name is required."),
            "invalid_vector_store_request",
        ));
    }

    Ok(local_vector_store_placeholder("vs_1", &request.name))
}

fn local_vector_stores_list_result() -> ListVectorStoresResponse {
    local_vector_stores_placeholder()
}

fn local_vector_store_retrieve_result(
    vector_store_id: &str,
) -> std::result::Result<VectorStoreObject, Response> {
    if vector_store_missing(vector_store_id) {
        return Err(local_vector_store_error_response(anyhow::anyhow!(
            "vector store not found"
        )));
    }

    Ok(local_vector_store_placeholder(vector_store_id, "kb-main"))
}

fn local_vector_store_update_result(
    vector_store_id: &str,
    request: &UpdateVectorStoreRequest,
) -> std::result::Result<VectorStoreObject, Response> {
    if vector_store_missing(vector_store_id) {
        return Err(local_vector_store_error_response(anyhow::anyhow!(
            "vector store not found"
        )));
    }

    let name = local_vector_store_update_name(request)?;
    if name.trim().is_empty() {
        return Err(local_gateway_invalid_or_bad_gateway_response(
            anyhow::anyhow!("Vector store name is required."),
            "invalid_vector_store_request",
        ));
    }

    Ok(local_vector_store_placeholder(vector_store_id, name))
}

fn local_vector_store_delete_result(
    vector_store_id: &str,
) -> std::result::Result<DeleteVectorStoreResponse, Response> {
    if vector_store_missing(vector_store_id) {
        return Err(local_vector_store_error_response(anyhow::anyhow!(
            "vector store not found"
        )));
    }

    Ok(DeleteVectorStoreResponse::deleted(vector_store_id))
}

pub(crate) async fn vector_stores_list_handler(
    request_context: StatelessGatewayRequest,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::VectorStoresList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector stores list");
        }
    }
    let response = local_vector_stores_list_result();

    Json(response).into_response()
}

pub(crate) async fn vector_stores_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateVectorStoreRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::VectorStores(&request))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store");
        }
    }
    let response = match local_vector_store_create_result(&request) {
        Ok(response) => response,
        Err(response) => return response,
    };

    Json(response).into_response()
}

pub(crate) async fn vector_store_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(vector_store_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoresRetrieve(&vector_store_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store retrieve");
        }
    }
    let response = match local_vector_store_retrieve_result(&vector_store_id) {
        Ok(response) => response,
        Err(response) => return response,
    };

    Json(response).into_response()
}

pub(crate) async fn vector_store_update_handler(
    request_context: StatelessGatewayRequest,
    Path(vector_store_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateVectorStoreRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoresUpdate(&vector_store_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store update");
        }
    }
    let response = match local_vector_store_update_result(&vector_store_id, &request) {
        Ok(response) => response,
        Err(response) => return response,
    };

    Json(response).into_response()
}

pub(crate) async fn vector_store_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(vector_store_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoresDelete(&vector_store_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store delete");
        }
    }
    let response = match local_vector_store_delete_result(&vector_store_id) {
        Ok(response) => response,
        Err(response) => return response,
    };

    Json(response).into_response()
}
