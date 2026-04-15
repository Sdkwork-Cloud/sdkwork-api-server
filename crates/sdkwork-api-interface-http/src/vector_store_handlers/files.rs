use super::*;
use sdkwork_api_contract_openai::vector_stores::{
    DeleteVectorStoreFileResponse, ListVectorStoreFilesResponse, VectorStoreFileObject,
};

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

fn vector_store_missing(vector_store_id: &str) -> bool {
    vector_store_id.trim().is_empty() || vector_store_id.ends_with("_missing")
}

fn file_missing(file_id: &str) -> bool {
    file_id.trim().is_empty() || file_id.ends_with("_missing")
}

fn local_vector_store_file_placeholder(file_id: &str) -> VectorStoreFileObject {
    VectorStoreFileObject::new(file_id)
}

fn local_vector_store_files_placeholder() -> ListVectorStoreFilesResponse {
    ListVectorStoreFilesResponse::new(vec![VectorStoreFileObject::new("file_1")])
}

fn local_vector_store_file_create_result(
    vector_store_id: &str,
    request: &CreateVectorStoreFileRequest,
) -> std::result::Result<VectorStoreFileObject, Response> {
    if vector_store_missing(vector_store_id) {
        return Err(local_vector_store_files_error_response(anyhow::anyhow!(
            "vector store not found"
        )));
    }
    if file_missing(&request.file_id) {
        return Err(local_vector_store_file_error_response(anyhow::anyhow!(
            "vector store file not found"
        )));
    }

    Ok(local_vector_store_file_placeholder(&request.file_id))
}

fn local_vector_store_files_list_result(
    vector_store_id: &str,
) -> std::result::Result<ListVectorStoreFilesResponse, Response> {
    if vector_store_missing(vector_store_id) {
        return Err(local_vector_store_files_error_response(anyhow::anyhow!(
            "vector store not found"
        )));
    }

    Ok(local_vector_store_files_placeholder())
}

fn local_vector_store_file_retrieve_result(
    vector_store_id: &str,
    file_id: &str,
) -> std::result::Result<VectorStoreFileObject, Response> {
    if vector_store_missing(vector_store_id) || file_missing(file_id) {
        return Err(local_vector_store_file_error_response(anyhow::anyhow!(
            "vector store file not found"
        )));
    }

    Ok(local_vector_store_file_placeholder(file_id))
}

fn local_vector_store_file_delete_result(
    vector_store_id: &str,
    file_id: &str,
) -> std::result::Result<DeleteVectorStoreFileResponse, Response> {
    if vector_store_missing(vector_store_id) || file_missing(file_id) {
        return Err(local_vector_store_file_error_response(anyhow::anyhow!(
            "vector store file not found"
        )));
    }

    Ok(DeleteVectorStoreFileResponse::deleted(file_id))
}

pub(crate) async fn vector_store_files_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(vector_store_id): Path<String>,
    ExtractJson(request): ExtractJson<CreateVectorStoreFileRequest>,
) -> Response {
    match relay_vector_store_file_from_store(
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
            let Some(usage_model) = response_usage_id_or_single_data_item_id(&response) else {
                return bad_gateway_openai_response(
                    "upstream vector store file response missing usage id",
                );
            };
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "vector_store_files",
                &vector_store_id,
                usage_model,
                25,
                0.025,
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
            return bad_gateway_openai_response("failed to relay upstream vector store file");
        }
    }
    let response = match local_vector_store_file_create_result(&vector_store_id, &request) {
        Ok(response) => response,
        Err(response) => return response,
    };
    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_store_files",
        &vector_store_id,
        response.id.as_str(),
        25,
        0.025,
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

pub(crate) async fn vector_store_files_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(vector_store_id): Path<String>,
) -> Response {
    match relay_list_vector_store_files_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "vector_store_files",
                &vector_store_id,
                15,
                0.015,
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
            return bad_gateway_openai_response("failed to relay upstream vector store files list");
        }
    }
    let response = match local_vector_store_files_list_result(&vector_store_id) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_store_files",
        &vector_store_id,
        15,
        0.015,
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

pub(crate) async fn vector_store_file_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((vector_store_id, file_id)): Path<(String, String)>,
) -> Response {
    match relay_get_vector_store_file_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &file_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "vector_store_files",
                &vector_store_id,
                &file_id,
                15,
                0.015,
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
            return bad_gateway_openai_response(
                "failed to relay upstream vector store file retrieve",
            );
        }
    }
    let response = match local_vector_store_file_retrieve_result(&vector_store_id, &file_id) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_store_files",
        &vector_store_id,
        &file_id,
        15,
        0.015,
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

pub(crate) async fn vector_store_file_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path((vector_store_id, file_id)): Path<(String, String)>,
) -> Response {
    match relay_delete_vector_store_file_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &vector_store_id,
        &file_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "vector_store_files",
                &vector_store_id,
                &file_id,
                15,
                0.015,
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
            return bad_gateway_openai_response(
                "failed to relay upstream vector store file delete",
            );
        }
    }
    let response = match local_vector_store_file_delete_result(&vector_store_id, &file_id) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "vector_store_files",
        &vector_store_id,
        &file_id,
        15,
        0.015,
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
