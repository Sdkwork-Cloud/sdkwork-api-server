use super::*;

fn local_inference_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_bad_gateway_response(error, "invalid_model")
}

pub(crate) async fn completions_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateCompletionRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Completions(&request))
        .await
    {
        Ok(Some(response)) => Json(response).into_response(),
        Ok(None) => {
            let response = match create_completion(
                request_context.tenant_id(),
                request_context.project_id(),
                &request.model,
            ) {
                Ok(response) => response,
                Err(error) => return local_inference_error_response(error),
            };
            Json(response).into_response()
        }
        Err(_) => bad_gateway_openai_response("failed to relay upstream completion"),
    }
}

pub(crate) async fn embeddings_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateEmbeddingRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Embeddings(&request))
        .await
    {
        Ok(Some(response)) => Json(response).into_response(),
        Ok(None) => {
            let response = match create_embedding(
                request_context.tenant_id(),
                request_context.project_id(),
                &request.model,
            ) {
                Ok(response) => response,
                Err(error) => return local_inference_error_response(error),
            };
            Json(response).into_response()
        }
        Err(_) => bad_gateway_openai_response("failed to relay upstream embedding"),
    }
}

pub(crate) async fn moderations_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateModerationRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Moderations(&request))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream moderation");
        }
    }

    let response = match create_moderation(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
    ) {
        Ok(response) => response,
        Err(error) => return local_inference_error_response(error),
    };

    Json(response).into_response()
}
