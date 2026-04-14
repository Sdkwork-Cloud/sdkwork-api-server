use super::*;

fn local_vector_store_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_vector_store_request",
        "Requested vector store was not found.",
    )
}

fn local_vector_store_update_name<'a>(
    request: &'a UpdateVectorStoreRequest,
) -> Result<&'a str, Response> {
    request.name.as_deref().ok_or_else(|| {
        invalid_request_openai_response(
            "Vector store name is required for local fallback updates.",
            "invalid_vector_store_request",
        )
    })
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
    let response =
        match list_vector_stores(request_context.tenant_id(), request_context.project_id()) {
            Ok(response) => response,
            Err(error) => return local_vector_store_error_response(error),
        };

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
    let response = match create_vector_store(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.name,
    ) {
        Ok(response) => response,
        Err(error) => return local_vector_store_error_response(error),
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
    let response = match get_vector_store(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_vector_store_error_response(error),
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
    let response = match update_vector_store(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        match local_vector_store_update_name(&request) {
            Ok(name) => name,
            Err(response) => return response,
        },
    ) {
        Ok(response) => response,
        Err(error) => return local_vector_store_error_response(error),
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
    let response = match delete_vector_store(
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_vector_store_error_response(error),
    };

    Json(response).into_response()
}
