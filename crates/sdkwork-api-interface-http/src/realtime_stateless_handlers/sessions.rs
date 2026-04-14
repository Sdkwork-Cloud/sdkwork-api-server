use super::*;

fn local_realtime_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_bad_gateway_response(error, "invalid_model")
}

pub(crate) async fn realtime_sessions_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateRealtimeSessionRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::RealtimeSessions(&request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream realtime session");
        }
    }

    let response = match create_realtime_session(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
    ) {
        Ok(response) => response,
        Err(error) => return local_realtime_error_response(error),
    };

    Json(response).into_response()
}
