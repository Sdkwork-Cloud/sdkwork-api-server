use super::*;

fn local_webhook_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_webhook_request",
        "Requested webhook was not found.",
    )
}

pub(crate) async fn webhooks_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateWebhookRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Webhooks(&request)).await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream webhook");
        }
    }

    let response = match create_webhook(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.url,
        &request.events,
    ) {
        Ok(response) => response,
        Err(error) => return local_webhook_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn webhooks_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::WebhooksList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream webhooks list");
        }
    }

    let response = match list_webhooks(request_context.tenant_id(), request_context.project_id()) {
        Ok(response) => response,
        Err(error) => return local_webhook_error_response(error),
    };

    Json(response).into_response()
}
