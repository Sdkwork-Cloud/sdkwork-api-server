use super::*;

pub(crate) async fn file_content_handler(
    request_context: StatelessGatewayRequest,
    Path(file_id): Path<String>,
) -> Response {
    match relay_stateless_stream_request(&request_context, ProviderRequest::FilesContent(&file_id))
        .await
    {
        Ok(Some(response)) => return upstream_passthrough_response(response),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream file content");
        }
    }

    local_file_content_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &file_id,
    )
}
