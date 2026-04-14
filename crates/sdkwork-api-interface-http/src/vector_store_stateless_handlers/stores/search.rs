use super::*;

fn local_vector_store_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_vector_store_request",
        "Requested vector store was not found.",
    )
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
    let response = match search_vector_store(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &request.query,
    ) {
        Ok(response) => response,
        Err(error) => return local_vector_store_error_response(error),
    };

    Json(response).into_response()
}
