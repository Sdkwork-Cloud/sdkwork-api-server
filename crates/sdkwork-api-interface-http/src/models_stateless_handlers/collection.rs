use super::*;

fn local_models_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_bad_gateway_response(error, "invalid_model")
}

pub(crate) async fn list_models_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::ModelsList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream model list");
        }
    }

    let response = match list_models(request_context.tenant_id(), request_context.project_id()) {
        Ok(response) => response,
        Err(error) => return local_models_error_response(error),
    };

    Json(response).into_response()
}
