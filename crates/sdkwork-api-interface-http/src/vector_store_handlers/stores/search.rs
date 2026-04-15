use super::*;
use sdkwork_api_contract_openai::vector_stores::SearchVectorStoreResponse;

fn local_vector_store_search_error_response(error: anyhow::Error) -> Response {
    let message = error.to_string();
    if local_gateway_error_is_invalid_request(&message) {
        return invalid_request_openai_response(message, "invalid_vector_store_request");
    }

    local_gateway_error_response(error, "Requested vector store was not found.")
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
        return Err(local_vector_store_search_error_response(anyhow::anyhow!(
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

pub(crate) async fn vector_store_search_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(vector_store_id): Path<String>,
    ExtractJson(request): ExtractJson<SearchVectorStoreRequest>,
) -> Response {
    match relay_search_vector_store_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "vector_store_search",
                &vector_store_id,
                20,
                0.02,
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
            return bad_gateway_openai_response("failed to relay upstream vector store search");
        }
    }

    let response = match local_vector_store_search_result(&vector_store_id, &request.query) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_store_search",
        &vector_store_id,
        20,
        0.02,
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
