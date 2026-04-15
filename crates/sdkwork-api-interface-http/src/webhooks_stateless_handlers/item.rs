use super::*;

fn local_webhook_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_webhook_request",
        "Requested webhook was not found.",
    )
}

fn local_webhook_update_url(request: &UpdateWebhookRequest) -> Result<&str, Response> {
    request.url.as_deref().ok_or_else(|| {
        invalid_request_openai_response(
            "Webhook url is required for local fallback updates.",
            "invalid_webhook_request",
        )
    })
}

pub(crate) async fn webhook_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(webhook_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::WebhooksRetrieve(&webhook_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream webhook retrieve");
        }
    }

    let response = match get_webhook(
        request_context.tenant_id(),
        request_context.project_id(),
        &webhook_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_webhook_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn webhook_update_handler(
    request_context: StatelessGatewayRequest,
    Path(webhook_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateWebhookRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::WebhooksUpdate(&webhook_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream webhook update");
        }
    }

    let webhook_url = match local_webhook_update_url(&request) {
        Ok(url) => url,
        Err(response) => return response,
    };
    let response = match update_webhook(
        request_context.tenant_id(),
        request_context.project_id(),
        &webhook_id,
        webhook_url,
    ) {
        Ok(response) => response,
        Err(error) => return local_webhook_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn webhook_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(webhook_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::WebhooksDelete(&webhook_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream webhook delete");
        }
    }

    let response = match delete_webhook(
        request_context.tenant_id(),
        request_context.project_id(),
        &webhook_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_webhook_error_response(error),
    };

    Json(response).into_response()
}
