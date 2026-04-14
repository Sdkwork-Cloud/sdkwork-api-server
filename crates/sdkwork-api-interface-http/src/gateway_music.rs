fn local_music_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_bad_gateway_response(error, "invalid_music_request")
}

async fn music_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateMusicRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::Music(&request)).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream music create");
        }
    }

    match create_music(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(error) => local_music_error_response(error),
    }
}

async fn music_list_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::MusicList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream music list");
        }
    }

    let response = match list_music(request_context.tenant_id(), request_context.project_id()) {
        Ok(response) => response,
        Err(error) => return local_music_error_response(error),
    };

    Json(response).into_response()
}

async fn music_retrieve_handler(
    request_context: StatelessGatewayRequest,
    Path(music_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::MusicRetrieve(&music_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream music retrieve");
        }
    }

    local_music_retrieve_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &music_id,
    )
}

async fn music_delete_handler(
    request_context: StatelessGatewayRequest,
    Path(music_id): Path<String>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::MusicDelete(&music_id))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream music delete");
        }
    }

    local_music_delete_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &music_id,
    )
}

async fn music_content_handler(
    request_context: StatelessGatewayRequest,
    Path(music_id): Path<String>,
) -> Response {
    match relay_stateless_stream_request(&request_context, ProviderRequest::MusicContent(&music_id))
        .await
    {
        Ok(Some(response)) => return upstream_passthrough_response(response),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream music content");
        }
    }

    local_music_content_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &music_id,
    )
}

async fn music_lyrics_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateMusicLyricsRequest>,
) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::MusicLyrics(&request))
        .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream music lyrics");
        }
    }

    match create_music_lyrics(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(error) => local_music_error_response(error),
    }
}


fn local_music_not_found_response(error: anyhow::Error) -> Response {
    local_gateway_error_response(error, "Requested music was not found.")
}

fn local_music_retrieve_result(
    tenant_id: &str,
    project_id: &str,
    music_id: &str,
) -> std::result::Result<MusicObject, Response> {
    get_music(tenant_id, project_id, music_id).map_err(local_music_not_found_response)
}

fn local_music_retrieve_response(tenant_id: &str, project_id: &str, music_id: &str) -> Response {
    match local_music_retrieve_result(tenant_id, project_id, music_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_music_delete_result(
    tenant_id: &str,
    project_id: &str,
    music_id: &str,
) -> std::result::Result<DeleteMusicResponse, Response> {
    delete_music(tenant_id, project_id, music_id).map_err(local_music_not_found_response)
}

fn local_music_delete_response(tenant_id: &str, project_id: &str, music_id: &str) -> Response {
    match local_music_delete_result(tenant_id, project_id, music_id) {
        Ok(response) => Json(response).into_response(),
        Err(response) => response,
    }
}

fn local_music_content_result(
    tenant_id: &str,
    project_id: &str,
    music_id: &str,
) -> std::result::Result<Vec<u8>, Response> {
    music_content(tenant_id, project_id, music_id).map_err(|error| {
        local_gateway_error_response(error, "Requested music asset was not found.")
    })
}

fn local_music_content_response(tenant_id: &str, project_id: &str, music_id: &str) -> Response {
    let bytes = match local_music_content_result(tenant_id, project_id, music_id) {
        Ok(bytes) => bytes,
        Err(response) => return response,
    };
    match Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(header::CONTENT_TYPE, "audio/mpeg")
        .body(Body::from(bytes))
    {
        Ok(response) => response,
        Err(_) => bad_gateway_openai_response("failed to process local music content fallback"),
    }
}


async fn music_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateMusicRequest>,
) -> Response {
    match relay_music_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let Some(usage_model) = response_usage_id_or_single_data_item_id(&response) else {
                return bad_gateway_openai_response("upstream music response missing usage id");
            };
            let music_seconds = request
                .duration_seconds
                .unwrap_or_else(|| music_seconds_from_response(&response));
            if record_gateway_usage_for_project_with_media_and_reference_id(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "music",
                &request.model,
                music_billing_units(music_seconds),
                music_billing_amount(music_seconds),
                BillingMediaMetrics {
                    music_seconds,
                    ..BillingMediaMetrics::default()
                },
                Some(usage_model),
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
            return bad_gateway_openai_response("failed to relay upstream music create");
        }
    }

    let response = match create_music(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => response,
        Err(error) => return local_music_error_response(error),
    };
    let usage_model = match response.data.as_slice() {
        [track] => track.id.as_str(),
        _ => request.model.as_str(),
    };
    let music_seconds = match response.data.as_slice() {
        [track] => track
            .duration_seconds
            .unwrap_or(request.duration_seconds.unwrap_or(0.0)),
        _ => request.duration_seconds.unwrap_or(0.0),
    };

    if record_gateway_usage_for_project_with_media_and_reference_id(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "music",
        &request.model,
        music_billing_units(music_seconds),
        music_billing_amount(music_seconds),
        BillingMediaMetrics {
            music_seconds,
            ..BillingMediaMetrics::default()
        },
        Some(usage_model),
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

async fn music_list_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Response {
    match relay_list_music_from_store(
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
                "music",
                "music",
                10,
                0.01,
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
            return bad_gateway_openai_response("failed to relay upstream music list");
        }
    }

    let response = match list_music(request_context.tenant_id(), request_context.project_id()) {
        Ok(response) => response,
        Err(error) => return local_music_error_response(error),
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "music",
        "music",
        10,
        0.01,
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

async fn music_retrieve_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(music_id): Path<String>,
) -> Response {
    match relay_get_music_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &music_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "music",
                &music_id,
                10,
                0.01,
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
            return bad_gateway_openai_response("failed to relay upstream music retrieve");
        }
    }

    let response = match local_music_retrieve_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &music_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "music",
        &music_id,
        10,
        0.01,
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

async fn music_delete_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(music_id): Path<String>,
) -> Response {
    match relay_delete_music_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &music_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "music",
                &music_id,
                10,
                0.01,
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
            return bad_gateway_openai_response("failed to relay upstream music delete");
        }
    }

    let response = match local_music_delete_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &music_id,
    ) {
        Ok(response) => response,
        Err(response) => return response,
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "music",
        &music_id,
        10,
        0.01,
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

async fn music_content_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    Path(music_id): Path<String>,
) -> Response {
    match relay_music_content_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &music_id,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "music",
                &music_id,
                10,
                0.01,
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
            return bad_gateway_openai_response("failed to relay upstream music content");
        }
    }

    let response = local_music_content_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &music_id,
    );
    if !response.status().is_success() {
        return response;
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "music",
        &music_id,
        10,
        0.01,
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

    response
}

async fn music_lyrics_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateMusicLyricsRequest>,
) -> Response {
    match relay_music_lyrics_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let Some(usage_model) = response.get("id").and_then(Value::as_str) else {
                return bad_gateway_openai_response("upstream music lyrics response missing id");
            };
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "music",
                "lyrics",
                usage_model,
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
            return bad_gateway_openai_response("failed to relay upstream music lyrics");
        }
    }

    let response = match create_music_lyrics(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => response,
        Err(error) => return local_music_error_response(error),
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "music",
        "lyrics",
        &response.id,
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

