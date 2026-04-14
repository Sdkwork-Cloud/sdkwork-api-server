use super::*;

fn local_image_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_bad_gateway_response(error, "invalid_image_request")
}

pub(crate) async fn image_generations_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateImageRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::ImagesGenerations(&request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream image generation");
        }
    }

    let response = match create_image_generation(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
    ) {
        Ok(response) => response,
        Err(error) => return local_image_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn image_edits_handler(
    request_context: StatelessGatewayRequest,
    multipart: Multipart,
) -> Response {
    match parse_image_edit_request(multipart).await {
        Ok(request) => {
            match relay_stateless_json_request(
                &request_context,
                ProviderRequest::ImagesEdits(&request),
            )
            .await
            {
                Ok(Some(response)) => return Json(response).into_response(),
                Ok(None) => {}
                Err(_) => {
                    return bad_gateway_openai_response("failed to relay upstream image edit");
                }
            }

            let response = match create_image_edit(
                request_context.tenant_id(),
                request_context.project_id(),
                &request,
            ) {
                Ok(response) => response,
                Err(error) => return local_image_error_response(error),
            };

            Json(response).into_response()
        }
        Err(response) => response,
    }
}

pub(crate) async fn image_variations_handler(
    request_context: StatelessGatewayRequest,
    multipart: Multipart,
) -> Response {
    match parse_image_variation_request(multipart).await {
        Ok(request) => {
            match relay_stateless_json_request(
                &request_context,
                ProviderRequest::ImagesVariations(&request),
            )
            .await
            {
                Ok(Some(response)) => return Json(response).into_response(),
                Ok(None) => {}
                Err(_) => {
                    return bad_gateway_openai_response("failed to relay upstream image variation");
                }
            }

            let response = match create_image_variation(
                request_context.tenant_id(),
                request_context.project_id(),
                &request,
            ) {
                Ok(response) => response,
                Err(error) => return local_image_error_response(error),
            };

            Json(response).into_response()
        }
        Err(response) => response,
    }
}
