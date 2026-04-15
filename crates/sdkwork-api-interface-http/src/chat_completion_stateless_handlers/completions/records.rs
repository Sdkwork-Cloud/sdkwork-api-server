#![allow(clippy::result_large_err)]

use super::*;

fn local_chat_completion_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_chat_completion_request",
        "Requested chat completion was not found.",
    )
}

fn local_chat_completion_list_error_response() -> Response {
    invalid_request_openai_response(
        "Local chat completion listing fallback is not supported without an upstream provider.",
        "invalid_chat_completion_request",
    )
}

fn local_chat_completion_update_metadata(
    request: &UpdateChatCompletionRequest,
) -> Result<Value, Response> {
    request.metadata.clone().ok_or_else(|| {
        invalid_request_openai_response(
            "Chat completion metadata is required for local fallback updates.",
            "invalid_chat_completion_request",
        )
    })
}

pub(crate) async fn chat_completions_list_handler(
    request_context: StatelessGatewayRequest,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::ChatCompletionsList).await
    {
        Ok(Some(response)) => Json(response).into_response(),
        Ok(None) => {
            let response = match list_chat_completions(
                request_context.tenant_id(),
                request_context.project_id(),
            ) {
                Ok(response) => response,
                Err(_) => return local_chat_completion_list_error_response(),
            };

            Json(response).into_response()
        }
        Err(_) => bad_gateway_openai_response("failed to relay upstream chat completion list"),
    }
}

pub(crate) async fn chat_completion_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(completion_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ChatCompletionsRetrieve(&completion_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response(
                "failed to relay upstream chat completion retrieve",
            );
        }
    }

    let response = match get_chat_completion(
        request_context.tenant_id(),
        request_context.project_id(),
        &completion_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_chat_completion_not_found_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn chat_completion_update_handler(
    request_context: StatelessGatewayRequest,
    Path(completion_id): Path<String>,
    ExtractJson(request): ExtractJson<UpdateChatCompletionRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ChatCompletionsUpdate(&completion_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream chat completion update");
        }
    }

    let response = match update_chat_completion(
        request_context.tenant_id(),
        request_context.project_id(),
        &completion_id,
        match local_chat_completion_update_metadata(&request) {
            Ok(metadata) => metadata,
            Err(response) => return response,
        },
    ) {
        Ok(response) => response,
        Err(error) => return local_chat_completion_not_found_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn chat_completion_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(completion_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ChatCompletionsDelete(&completion_id),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream chat completion delete");
        }
    }

    let response = match delete_chat_completion(
        request_context.tenant_id(),
        request_context.project_id(),
        &completion_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_chat_completion_not_found_response(error),
    };

    Json(response).into_response()
}
