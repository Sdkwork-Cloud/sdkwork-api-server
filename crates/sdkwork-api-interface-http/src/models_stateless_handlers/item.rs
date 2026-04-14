use super::*;

fn local_model_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_model",
        "Requested model was not found.",
    )
}

pub(crate) async fn model_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(model_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::ModelsRetrieve(&model_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream model");
        }
    }

    let response = match get_model(
        request_context.tenant_id(),
        request_context.project_id(),
        &model_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_model_not_found_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn model_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(model_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::ModelsDelete(&model_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream model delete");
        }
    }

    let response = match delete_model(
        request_context.tenant_id(),
        request_context.project_id(),
        &model_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_model_not_found_response(error),
    };

    Json(response).into_response()
}
