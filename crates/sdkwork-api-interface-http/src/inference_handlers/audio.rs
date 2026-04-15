use super::*;
use crate::gateway_commercial::{
    begin_gateway_commercial_admission, capture_gateway_commercial_admission,
    enforce_project_quota,
    record_gateway_usage_for_project as record_gateway_usage_for_project_commercial,
    record_gateway_usage_for_project_with_route_key_and_reference_id,
    release_gateway_commercial_admission,
    response_usage_id_or_single_data_item_id as commercial_response_usage_id_or_single_data_item_id,
    GatewayCommercialAdmissionDecision, GatewayCommercialAdmissionSpec,
};
use sdkwork_api_contract_openai::audio::{TranscriptionObject, TranslationObject};

const LOCAL_TRANSCRIPTION_BACKEND_UNSUPPORTED: &str =
    "Local transcription fallback is not supported without a transcription backend.";
const LOCAL_TRANSLATION_BACKEND_UNSUPPORTED: &str =
    "Local translation fallback is not supported without a translation backend.";
const LOCAL_TRANSCRIPTION_PLACEHOLDER_TEXT: &str = "sdkwork.local transcription unavailable";
const LOCAL_TRANSLATION_PLACEHOLDER_TEXT: &str = "sdkwork.local translation unavailable";

fn local_audio_error_response(error: anyhow::Error) -> Response {
    local_gateway_invalid_or_bad_gateway_response(error, "invalid_audio_request")
}

fn local_audio_response_build_error_response() -> Response {
    bad_gateway_openai_response("failed to build local audio response")
}

fn local_audio_response_decode_error_response() -> Response {
    bad_gateway_openai_response("failed to decode local speech audio")
}

fn response_text(response: &Value) -> Option<&str> {
    response
        .get("text")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
}

fn local_transcription_result(
    tenant_id: &str,
    project_id: &str,
    request: &CreateTranscriptionRequest,
) -> anyhow::Result<TranscriptionObject> {
    match create_transcription(tenant_id, project_id, &request.model) {
        Ok(response) => Ok(response),
        Err(error)
            if error
                .to_string()
                .contains(LOCAL_TRANSCRIPTION_BACKEND_UNSUPPORTED) =>
        {
            Ok(TranscriptionObject::new(
                LOCAL_TRANSCRIPTION_PLACEHOLDER_TEXT,
            ))
        }
        Err(error) => Err(error),
    }
}

fn local_translation_result(
    tenant_id: &str,
    project_id: &str,
    request: &CreateTranslationRequest,
) -> anyhow::Result<TranslationObject> {
    match create_translation(tenant_id, project_id, &request.model) {
        Ok(response) => Ok(response),
        Err(error)
            if error
                .to_string()
                .contains(LOCAL_TRANSLATION_BACKEND_UNSUPPORTED) =>
        {
            Ok(TranslationObject::new(LOCAL_TRANSLATION_PLACEHOLDER_TEXT))
        }
        Err(error) => Err(error),
    }
}

fn build_local_speech_response(
    stream_as_sse: bool,
    speech_format: &str,
    speech_audio_base64: &str,
) -> Result<Response, Response> {
    if stream_as_sse {
        let delta = serde_json::json!({
            "type":"response.output_audio.delta",
            "delta": speech_audio_base64,
            "format": speech_format,
        })
        .to_string();
        let done = serde_json::json!({
            "type":"response.completed"
        })
        .to_string();
        let body = format!("{}{}", SseFrame::data(&delta), SseFrame::data(&done));
        return Ok(([(header::CONTENT_TYPE, "text/event-stream")], body).into_response());
    }

    let bytes = match STANDARD.decode(speech_audio_base64.as_bytes()) {
        Ok(bytes) => bytes,
        Err(_) => return Err(local_audio_response_decode_error_response()),
    };

    match Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            match speech_format {
                "mp3" => "audio/mpeg",
                "opus" => "audio/opus",
                "aac" => "audio/aac",
                "flac" => "audio/flac",
                "pcm" => "audio/pcm",
                _ => "audio/wav",
            },
        )
        .body(Body::from(bytes))
    {
        Ok(response) => Ok(response),
        Err(_) => Err(local_audio_response_build_error_response()),
    }
}

pub(crate) async fn transcriptions_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateTranscriptionRequest>,
) -> Response {
    let commercial_admission = match begin_gateway_commercial_admission(
        &state,
        request_context.context(),
        GatewayCommercialAdmissionSpec {
            quoted_amount: 0.025,
        },
    )
    .await
    {
        Ok(GatewayCommercialAdmissionDecision::Canonical(admission)) => Some(admission),
        Ok(GatewayCommercialAdmissionDecision::LegacyQuota) => {
            match enforce_project_quota(state.store.as_ref(), request_context.project_id(), 25)
                .await
            {
                Ok(Some(response)) => return response,
                Ok(None) => {}
                Err(_) => {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to evaluate quota",
                    )
                        .into_response();
                }
            }
            None
        }
        Err(response) => return response,
    };

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
            if response_text(&response).is_none() {
                if let Some(admission) = commercial_admission.as_ref() {
                    if let Err(response) =
                        release_gateway_commercial_admission(&state, admission).await
                    {
                        return response;
                    }
                }
                return bad_gateway_openai_response("upstream transcription response missing text");
            }
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = capture_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            if record_gateway_usage_for_project_with_route_key_and_reference_id(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "audio_transcriptions",
                &request.model,
                &request.model,
                25,
                0.025,
                commercial_response_usage_id_or_single_data_item_id(&response),
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
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = release_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            return bad_gateway_openai_response("failed to relay upstream transcription");
        }
    }

    let response = match local_transcription_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => response,
        Err(error) => {
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = release_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            return local_audio_error_response(error);
        }
    };
    if response.text.trim().is_empty() {
        if let Some(admission) = commercial_admission.as_ref() {
            if let Err(response) = release_gateway_commercial_admission(&state, admission).await {
                return response;
            }
        }
        return bad_gateway_openai_response("failed to process local transcription fallback");
    }

    if let Some(admission) = commercial_admission.as_ref() {
        if let Err(response) = capture_gateway_commercial_admission(&state, admission).await {
            return response;
        }
    }

    if record_gateway_usage_for_project_commercial(
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

pub(crate) async fn translations_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateTranslationRequest>,
) -> Response {
    let commercial_admission = match begin_gateway_commercial_admission(
        &state,
        request_context.context(),
        GatewayCommercialAdmissionSpec {
            quoted_amount: 0.025,
        },
    )
    .await
    {
        Ok(GatewayCommercialAdmissionDecision::Canonical(admission)) => Some(admission),
        Ok(GatewayCommercialAdmissionDecision::LegacyQuota) => {
            match enforce_project_quota(state.store.as_ref(), request_context.project_id(), 25)
                .await
            {
                Ok(Some(response)) => return response,
                Ok(None) => {}
                Err(_) => {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to evaluate quota",
                    )
                        .into_response();
                }
            }
            None
        }
        Err(response) => return response,
    };

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
            if response_text(&response).is_none() {
                if let Some(admission) = commercial_admission.as_ref() {
                    if let Err(response) =
                        release_gateway_commercial_admission(&state, admission).await
                    {
                        return response;
                    }
                }
                return bad_gateway_openai_response("upstream translation response missing text");
            }
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = capture_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            if record_gateway_usage_for_project_with_route_key_and_reference_id(
                state.store.as_ref(),
                request_context.tenant_id(),
                request_context.project_id(),
                "audio_translations",
                &request.model,
                &request.model,
                25,
                0.025,
                commercial_response_usage_id_or_single_data_item_id(&response),
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
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = release_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            return bad_gateway_openai_response("failed to relay upstream translation");
        }
    }

    let response = match local_translation_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => response,
        Err(error) => {
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = release_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            return local_audio_error_response(error);
        }
    };
    if response.text.trim().is_empty() {
        if let Some(admission) = commercial_admission.as_ref() {
            if let Err(response) = release_gateway_commercial_admission(&state, admission).await {
                return response;
            }
        }
        return bad_gateway_openai_response("failed to process local translation fallback");
    }

    if let Some(admission) = commercial_admission.as_ref() {
        if let Err(response) = capture_gateway_commercial_admission(&state, admission).await {
            return response;
        }
    }

    if record_gateway_usage_for_project_commercial(
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

pub(crate) async fn audio_speech_with_state_handler(
    request_context: AuthenticatedGatewayRequest,
    State(state): State<GatewayApiState>,
    ExtractJson(request): ExtractJson<CreateSpeechRequest>,
) -> Response {
    let commercial_admission = match begin_gateway_commercial_admission(
        &state,
        request_context.context(),
        GatewayCommercialAdmissionSpec {
            quoted_amount: 0.025,
        },
    )
    .await
    {
        Ok(GatewayCommercialAdmissionDecision::Canonical(admission)) => Some(admission),
        Ok(GatewayCommercialAdmissionDecision::LegacyQuota) => {
            match enforce_project_quota(state.store.as_ref(), request_context.project_id(), 25)
                .await
            {
                Ok(Some(response)) => return response,
                Ok(None) => {}
                Err(_) => {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to evaluate quota",
                    )
                        .into_response();
                }
            }
            None
        }
        Err(response) => return response,
    };

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
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = capture_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            if record_gateway_usage_for_project_commercial(
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
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = release_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            return bad_gateway_openai_response("failed to relay upstream speech");
        }
    }

    let speech = match create_speech_response(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(speech) => speech,
        Err(error) => {
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(response) = release_gateway_commercial_admission(&state, admission).await
                {
                    return response;
                }
            }
            return local_audio_error_response(error);
        }
    };

    let speech_response = match build_local_speech_response(
        request.stream_format.as_deref() == Some("sse"),
        speech.format.as_str(),
        speech.audio_base64.as_str(),
    ) {
        Ok(response) => response,
        Err(response) => {
            if let Some(admission) = commercial_admission.as_ref() {
                if let Err(release_response) =
                    release_gateway_commercial_admission(&state, admission).await
                {
                    return release_response;
                }
            }
            return response;
        }
    };

    if let Some(admission) = commercial_admission.as_ref() {
        if let Err(response) = capture_gateway_commercial_admission(&state, admission).await {
            return response;
        }
    }

    if record_gateway_usage_for_project_commercial(
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

    speech_response
}

pub(crate) async fn audio_voices_with_state_handler(
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
            if record_gateway_usage_for_project_commercial(
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

    let response =
        match list_audio_voices(request_context.tenant_id(), request_context.project_id()) {
            Ok(response) => response,
            Err(error) => return local_audio_error_response(error),
        };

    if record_gateway_usage_for_project_commercial(
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

pub(crate) async fn audio_voice_consents_with_state_handler(
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
                return bad_gateway_openai_response("upstream voice consent response missing id");
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
