use super::*;

fn local_upload_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_not_found_response(
        error,
        "invalid_upload_request",
        "Requested upload was not found.",
    )
}

pub(crate) async fn uploads_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateUploadRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Uploads(&request)).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream upload");
        }
    }

    let response = match create_upload(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => response,
        Err(error) => return local_upload_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn upload_parts_handler(
    request_context: StatelessGatewayRequest,
    Path(upload_id): Path<String>,
    multipart: Multipart,
) -> Response {
    match parse_upload_part_request(upload_id, multipart).await {
        Ok(request) => {
            match relay_stateless_json_request(
                &request_context,
                ProviderRequest::UploadParts(&request),
            )
            .await
            {
                Ok(Some(response)) => return Json(response).into_response(),
                Ok(None) => {}
                Err(_) => {
                    return bad_gateway_openai_response("failed to relay upstream upload part");
                }
            }

            let response = match create_upload_part(
                request_context.tenant_id(),
                request_context.project_id(),
                &request,
            ) {
                Ok(response) => response,
                Err(error) => return local_upload_error_response(error),
            };

            Json(response).into_response()
        }
        Err(response) => response,
    }
}

pub(crate) async fn upload_complete_handler(
    request_context: StatelessGatewayRequest,
    Path(upload_id): Path<String>,
    ExtractJson(mut request): ExtractJson<CompleteUploadRequest>,
) -> Response {
    request.upload_id = upload_id;
    match relay_stateless_json_request(&request_context, ProviderRequest::UploadComplete(&request))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream upload complete");
        }
    }

    let response = match complete_upload(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => response,
        Err(error) => return local_upload_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn upload_cancel_handler(
    request_context: StatelessGatewayRequest,
    Path(upload_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::UploadCancel(&upload_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream upload cancel");
        }
    }

    let response = match cancel_upload(
        request_context.tenant_id(),
        request_context.project_id(),
        &upload_id,
    ) {
        Ok(response) => response,
        Err(error) => return local_upload_error_response(error),
    };

    Json(response).into_response()
}
