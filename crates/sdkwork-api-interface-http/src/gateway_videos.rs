async fn videos_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateVideoRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Videos(&request)).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video");
        }
    }

    Json(
        create_video(
            request_context.tenant_id(),
            request_context.project_id(),
            &request.model,
            &request.prompt,
        )
        .expect("video"),
    )
    .into_response()
}

async fn videos_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::VideosList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream videos list");
        }
    }

    Json(
        list_videos(request_context.tenant_id(), request_context.project_id())
            .expect("videos list"),
    )
    .into_response()
}

async fn video_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(video_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::VideosRetrieve(&video_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video retrieve");
        }
    }

    local_video_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
    )
}

async fn video_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(video_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::VideosDelete(&video_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video delete");
        }
    }

    local_video_delete_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
    )
}

async fn video_content_handler(
    request_context: StatelessGatewayRequest,
    Path(video_id): Path<String>,
) -> Response {
    match relay_stateless_stream_request(
        &request_context,
        ProviderRequest::VideosContent(&video_id),
    )
    .await
    {
        Ok(Some(response)) => return upstream_passthrough_response(response),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video content");
        }
    }

    local_video_content_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
    )
}

async fn video_remix_handler(
    request_context: StatelessGatewayRequest,
    Path(video_id): Path<String>,
    ExtractJson(request): ExtractJson<RemixVideoRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::VideosRemix(&video_id, &request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video remix");
        }
    }

    Json(
        remix_video(
            request_context.tenant_id(),
            request_context.project_id(),
            &video_id,
            &request.prompt,
        )
        .expect("video remix"),
    )
    .into_response()
}


fn local_video_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_error_response(error, "Requested video was not found.")
}

fn local_video_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    video_id: &str,
) -> std::result::Result<VideoObject, Response> {
    get_video(tenant_id, project_id, video_id).map_err(local_video_not_found_response)
}

fn local_video_retrieve_response(tenant_id: &str, project_id: &str, video_id: &str) -> Response {
    match local_video_retrieve_result(tenant_id, project_id, video_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_video_delete_result(
    tenant_id: &str,
    project_id: &str,
    video_id: &str,
) -> std::result::Result<DeleteVideoResponse, Response> {
    delete_video(tenant_id, project_id, video_id).map_err(local_video_not_found_response)
}

fn local_video_delete_response(tenant_id: &str, project_id: &str, video_id: &str) -> Response {
    match local_video_delete_result(tenant_id, project_id, video_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_video_content_result(
    tenant_id: &str,
    project_id: &str,
    video_id: &str,
) -> std::result::Result<Vec<u8>, Response> {
    video_content(tenant_id, project_id, video_id).map_err(|error| {
        local_gateway_error_response(error, "Requested video asset was not found.")
    })
}

fn local_video_content_response(tenant_id: &str, project_id: &str, video_id: &str) -> Response {
    let bytes = match local_video_content_result(tenant_id, project_id, video_id) {
        Ok(bytes) => bytes,
        Err(response) => return response,
    };
    Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(header::CONTENT_TYPE, "video/mp4")
        .body(Body::from(bytes))
        .expect("valid local video content response")
}


async fn videos_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateVideoRequest>,
) -> Response {
    match relay_video_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let usage_model = response_usage_id_or_single_data_item_id(&response)
                .unwrap_or(request.model.as_str());
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "videos",
                &request.model,
                usage_model,
                90,
                0.09,
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
            return bad_gateway_openai_response("failed to relay upstream video create");
        }
    }

    let response = create_video(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
        &request.prompt,
    )
    .expect("video");
    let usage_model = match response.data.as_slice() {
        [video] => video.id.as_str(),
        _ => request.model.as_str(),
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        &request.model,
        usage_model,
        90,
        0.09,
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

async fn videos_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Response {
    match relay_list_videos_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "videos",
                "videos",
                20,
                0.02,
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
            return bad_gateway_openai_response("failed to relay upstream videos list");
        }
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        "videos",
        20,
        0.02,
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

    Json(
        list_videos(request_context.tenant_id(), request_context.project_id())
            .expect("videos list"),
    )
    .into_response()
}

async fn video_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(video_id): Path<String>,
) -> Response {
    match relay_get_video_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "videos",
                &video_id,
                20,
                0.02,
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
            return bad_gateway_openai_response("failed to relay upstream video retrieve");
        }
    }

    let response = match local_video_retrieve_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        &video_id,
        20,
        0.02,
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

async fn video_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(video_id): Path<String>,
) -> Response {
    match relay_delete_video_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "videos",
                &video_id,
                20,
                0.02,
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
            return bad_gateway_openai_response("failed to relay upstream video delete");
        }
    }

    let response = match local_video_delete_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        &video_id,
        20,
        0.02,
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

async fn video_content_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(video_id): Path<String>,
) -> Response {
    match relay_video_content_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "videos",
                &video_id,
                20,
                0.02,
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

            return upstream_passthrough_response(response);
        }
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream video content");
        }
    }

    let bytes = match local_video_content_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &video_id,
    ) {
        Ok(bytes) => bytes,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "videos",
        &video_id,
        20,
        0.02,
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

    Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(header::CONTENT_TYPE, "video/mp4")
        .body(Body::from(bytes))
        .expect("valid local video content response")
}

