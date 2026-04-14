use super::*;

fn local_container_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_container_request",
        "Requested container was not found.",
    )
}

pub(crate) async fn containers_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateContainerRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Containers(&request))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream container");
        }
    }

    let response = match sdkwork_api_app_gateway::create_container(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => response,
        Err(error) => return local_container_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn containers_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::ContainersList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream containers list");
        }
    }

    let response = match sdkwork_api_app_gateway::list_containers(
        request_context.tenant_id(),
        request_context.project_id(),
    ) {
        Ok(response) => response,
        Err(error) => return local_container_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn container_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(container_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ContainersRetrieve(&container_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream container retrieve");
        }
    }

    let response = match sdkwork_api_app_gateway::get_container(
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_container_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn container_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(container_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ContainersDelete(&container_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream container delete");
        }
    }

    let response = match sdkwork_api_app_gateway::delete_container(
        request_context.tenant_id(),
        request_context.project_id(),
        &container_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_container_error_response(error),
    };

    Json(response).into_response()
}
