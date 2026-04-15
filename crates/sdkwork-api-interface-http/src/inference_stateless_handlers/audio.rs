use super::*;
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

pub(crate) async fn transcriptions_handler(
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

    let response = match local_transcription_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => response,
        Err(error) => return local_audio_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn translations_handler(
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

    let response = match local_translation_result(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => response,
        Err(error) => return local_audio_error_response(error),
    };

    Json(response).into_response()
}

pub(crate) async fn audio_speech_handler(
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

pub(crate) async fn audio_voices_handler(request_context: StatelessGatewayRequest) -> Response {
    match relay_stateless_json_request(&request_context, ProviderRequest::AudioVoicesList).await {
        Ok(Some(response)) => return Json(response).into_response(),
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

    Json(response).into_response()
}

pub(crate) async fn audio_voice_consents_handler(
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

    let response = match create_audio_voice_consent(
        request_context.tenant_id(),
        request_context.project_id(),
        &request,
    ) {
        Ok(response) => response,
        Err(error) => return local_audio_error_response(error),
    };

    Json(response).into_response()
}
