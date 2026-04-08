async fn realtime_sessions_handler(
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

    Json(
        create_realtime_session(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
        )
        .expect("realtime session"),
    )
    .into_response()
}


async fn realtime_sessions_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateRealtimeSessionRequest>,
) -> Response {
    match relay_realtime_session_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let realtime_session_id = response
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(request.model.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "realtime_sessions",
                &request.model,
                realtime_session_id,
                30,
                0.03,
            )
            .await
            .is_err()
            {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to record usage",
                )
                    .into_response();
            }

            return Json(response).into_response();
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream realtime session");
        }
    }

    let response = create_realtime_session(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
    )
    .expect("realtime");

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "realtime_sessions",
        &request.model,
        response.id.as_str(),
        30,
        0.03,
    )
    .await
    .is_err()
    {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to record usage",
        )
            .into_response();
    }

    Json(response).into_response()
}

