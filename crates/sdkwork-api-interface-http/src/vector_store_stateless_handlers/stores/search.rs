use super::*;
use sdkwork_api_contract_openai::vector_stores::SearchVectorStoreResponse;

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

fn local_vector_store_search_placeholder(query: &str) -> SearchVectorStoreResponse {
    SearchVectorStoreResponse::sample(query)
}

fn local_vector_store_search_result(
    vector_store_id: &str,
    query: &str,
) -> std::result::Result<SearchVectorStoreResponse, Response> {
    if vector_store_missing(vector_store_id) {
        return Err(local_vector_store_error_response(anyhow::anyhow!(
            "vector store not found"
        )));
    }
    if query.trim().is_empty() {
        return Err(local_gateway_invalid_or_bad_gateway_response(
            anyhow::anyhow!("Vector store search query is required."),
            "invalid_vector_store_request",
        ));
    }

    Ok(local_vector_store_search_placeholder(query))
}

pub(crate) async fn vector_store_search_handler(
    request_context: StatelessGatewayRequest,
    Path(vector_store_id): Path<String>,
    ExtractJson(request): ExtractJson<SearchVectorStoreRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VectorStoresSearch(&vector_store_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream vector store search");
        }
    }
    let response = match local_vector_store_search_result(&vector_store_id, &request.query) {
        Ok(response) => response,
        Err(response) => return response,
    };

    Json(response).into_response()
}
