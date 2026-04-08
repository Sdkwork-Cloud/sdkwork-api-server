async fn responses_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateResponseRequest>,
) -> Response {
    if request.stream.unwrap_or(false) {
        match relay_stateless_stream_request(
            &request_context,
            ProviderRequest::ResponsesStream(&request),
        )
        .await
        {
            Ok(Some(response)) => return upstream_passthrough_response(response),
            Ok(None) => {}
            Err(_) => {
                return bad_gateway_openai_response("failed to relay upstream response stream");
            }
        }

        return local_response_stream_response(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        );
    }

    match relay_stateless_json_request(&request_context, ProviderRequest::Responses(&request)).await
    {
        Ok(Some(response)) => Json(response).into_response(),
        Ok(None) => local_response_create_response(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        )
        .into_response(),
        Err(_) => bad_gateway_openai_response("failed to relay upstream response"),
    }
}

async fn response_input_tokens_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CountResponseInputTokensRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ResponsesInputTokens(&request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream response input tokens");
        }
    }

    local_response_input_tokens_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
    )
}

async fn response_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(response_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ResponsesRetrieve(&response_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream response retrieve");
        }
    }

    match get_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &response_id,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(error) => local_response_not_found_response(error),
    }
}

async fn response_input_items_list_handler(
    request_context: StatelessGatewayRequest,
    Path(response_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ResponsesInputItemsList(&response_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream response input items");
        }
    }

    match list_response_input_items(
        request_context.tenant_id(),
        request_context.project_id(),
        &response_id,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(error) => local_response_not_found_response(error),
    }
}

async fn response_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(response_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ResponsesDelete(&response_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream response delete");
        }
    }

    match delete_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &response_id,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(error) => local_response_not_found_response(error),
    }
}

async fn response_cancel_handler(
    request_context: StatelessGatewayRequest,
    Path(response_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ResponsesCancel(&response_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream response cancel");
        }
    }

    match cancel_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &response_id,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(error) => local_response_not_found_response(error),
    }
}

async fn response_compact_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CompactResponseRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ResponsesCompact(&request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream response compact");
        }
    }

    local_response_compact_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
    )
}
