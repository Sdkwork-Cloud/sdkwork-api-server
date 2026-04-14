fn local_audio_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_bad_gateway_response(error, "invalid_audio_request")
}

async fn transcriptions_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateTranscriptionRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::AudioTranscriptions(&request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream transcription");
        }
    }

    match create_transcription(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(error) => local_audio_error_response(error),
    }
}

async fn translations_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateTranslationRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::AudioTranslations(&request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream translation");
        }
    }

    match create_translation(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(error) => local_audio_error_response(error),
    }
}

async fn audio_speech_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateSpeechRequest>,
) -> Response {
    match relay_stateless_stream_request(&request_context, ProviderRequest::AudioSpeech(&request))
        .await
    {
        Ok(Some(response)) => return upstream_passthrough_response(response),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream audio speech");
        }
    }

    local_speech_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
}

async fn audio_voices_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::AudioVoicesList).await {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream audio voices list");
        }
    }

    let response = match list_audio_voices(request_context.tenant_id(), request_context.project_id()) {
        Ok(response) => response,
        Err(error) => return local_audio_error_response(error),
    };

    Json(response).into_response()
}

async fn audio_voice_consents_handler(
    request_context: StatelessGatewayRequest,
    ExtractJson(request): ExtractJson<CreateVoiceConsentRequest>,
) -> Response {
    match relay_stateless_json_request(
        &request_context,
        ProviderRequest::AudioVoiceConsents(&request),
    )
    .await
    {
        Ok(Some(response)) => return Json(response).into_response(),
        Ok(None) => {}
        Err(_) => {
            return bad_gateway_openai_response("failed to relay upstream audio voice consent");
        }
    }

    match create_audio_voice_consent(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => Json(response).into_response(),
        Err(error) => local_audio_error_response(error),
    }
}


async fn transcriptions_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateTranscriptionRequest>,
) -> Response {
    match relay_transcription_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "audio_transcriptions",
                &request.model,
                25,
                0.025,
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
            return bad_gateway_openai_response("failed to relay upstream transcription");
        }
    }

    let response = match create_transcription(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
    ) {
        Ok(response) => response,
        Err(error) => return local_audio_error_response(error),
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "audio_transcriptions",
        &request.model,
        25,
        0.025,
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

async fn translations_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateTranslationRequest>,
) -> Response {
    match relay_translation_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "audio_translations",
                &request.model,
                25,
                0.025,
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
            return bad_gateway_openai_response("failed to relay upstream translation");
        }
    }

    let response = match create_translation(
        request_context.tenant_id(),
        request_context.project_id(),
        &request.model,
    ) {
        Ok(response) => response,
        Err(error) => return local_audio_error_response(error),
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "audio_translations",
        &request.model,
        25,
        0.025,
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

async fn audio_speech_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateSpeechRequest>,
) -> Response {
    match relay_speech_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            if record_gateway_usage_for_project(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "audio_speech",
                &request.model,
                25,
                0.025,
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
            return bad_gateway_openai_response("failed to relay upstream speech");
        }
    }

    let response = local_speech_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    );
    if !response.status().is_success() {
        return response;
    }

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "audio_speech",
        &request.model,
        25,
        0.025,
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

async fn audio_voices_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
) -> Response {
    match relay_audio_voices_from_store(
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
                "audio",
                "voices",
                5,
                0.005,
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
            return bad_gateway_openai_response("failed to relay upstream audio voices list");
        }
    }

    let response = match list_audio_voices(request_context.tenant_id(), request_context.project_id()) {
        Ok(response) => response,
        Err(error) => return local_audio_error_response(error),
    };

    if record_gateway_usage_for_project(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "audio",
        "voices",
        5,
        0.005,
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

async fn audio_voice_consents_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateVoiceConsentRequest>,
) -> Response {
    match relay_audio_voice_consent_from_store(
        state.store.as_ref(),
        &state.secret_manager,
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    )
    .await
    {
        Ok(Some(response)) => {
            let Some(consent_id) = response.get("id").and_then(Value::as_str) else {
                return bad_gateway_openai_response(
                    "upstream audio voice consent response missing id",
                );
            };
            if record_gateway_usage_for_project_with_route_key(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "audio",
                &request.voice,
                consent_id,
                5,
                0.005,
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
            return bad_gateway_openai_response("failed to relay upstream audio voice consent");
        }
    }

    let response = match create_audio_voice_consent(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => response,
        Err(error) => return local_audio_error_response(error),
    };

    if record_gateway_usage_for_project_with_route_key(
        state.store.as_ref(),
        request_context.tenant_id(),
        request_context.project_id(),
        "audio",
        &request.voice,
        response.id.as_str(),
        5,
        0.005,
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


fn local_speech_response(
    tenant_id: &str,
    project_id: &str,
    request: &CreateSpeechRequest,
) -> Response {
    let speech = match create_speech_response(tenant_id, project_id, request) {
        Ok(speech) => speech,
        Err(error) => {
            return invalid_request_openai_response(error.to_string(), "invalid_response_format");
        }
    };
    if request.stream_format.as_deref() == Some("sse") {
        let delta = serde_json::json!({
            "type":"response.output_audio.delta",
            "delta": speech.audio_base64,
            "format": speech.format,
        })
        .to_string();
        let done = serde_json::json!({
            "type":"response.completed"
        })
        .to_string();
        let body = format!("{}{}", SseFrame::data(&delta), SseFrame::data(&done));
        return ([(header::CONTENT_TYPE, "text/event-stream")], body).into_response();
    }

    let bytes = STANDARD
        .decode(speech.audio_base64.as_bytes())
        .map_err(|_| bad_gateway_openai_response("failed to process local speech fallback"))?;

    match Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(header::CONTENT_TYPE, speech_content_type(&speech.format))
        .body(Body::from(bytes))
    {
        Ok(response) => response,
        Err(_) => bad_gateway_openai_response("failed to process local speech fallback"),
    }
}


fn speech_content_type(format: &str) -> &'static str {
    match format {
        "mp3" => "audio/mpeg",
        "opus" => "audio/opus",
        "aac" => "audio/aac",
        "flac" => "audio/flac",
        "pcm" => "audio/pcm",
        _ => "audio/wav",
    }
}
