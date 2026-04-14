use super::*;

pub async fn relay_completion_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateCompletionRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "completions",
        &request.model,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::Completions(request),
    )
    .await
}

pub async fn relay_embedding_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateEmbeddingRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "embeddings",
        &request.model,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::Embeddings(request),
    )
    .await
}

pub async fn relay_moderation_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateModerationRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "moderations",
        &request.model,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::Moderations(request),
    )
    .await
}

pub async fn relay_image_generation_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateImageRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "images",
        &request.model,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ImagesGenerations(request),
    )
    .await
}

pub async fn relay_image_edit_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateImageEditRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "images",
        request.model_or_default(),
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ImagesEdits(request),
    )
    .await
}

pub async fn relay_image_variation_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateImageVariationRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "images",
        request.model_or_default(),
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::ImagesVariations(request),
    )
    .await
}

pub async fn relay_transcription_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateTranscriptionRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "audio_transcriptions",
        &request.model,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::AudioTranscriptions(request),
    )
    .await
}

pub async fn relay_translation_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateTranslationRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "audio_translations",
        &request.model,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::AudioTranslations(request),
    )
    .await
}

pub async fn relay_speech_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateSpeechRequest,
) -> Result<Option<ProviderStreamOutput>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "audio_speech",
        &request.model,
    )
    .await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_stream_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::AudioSpeech(request),
    )
    .await
}

pub async fn relay_audio_voices_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "audio", "voices").await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::AudioVoicesList,
    )
    .await
}

pub async fn relay_audio_voice_consent_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateVoiceConsentRequest,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "audio", &request.voice).await?;
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let Some(api_key) =
        resolve_provider_secret_with_manager(store, secret_manager, tenant_id, &provider.id)
            .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request_for_provider(
        store,
        &provider,
        &api_key,
        ProviderRequest::AudioVoiceConsents(request),
    )
    .await
}

pub fn create_completion(
    _tenant_id: &str,
    _project_id: &str,
    model: &str,
) -> Result<CompletionObject> {
    if model.trim().is_empty() {
        bail!("Completion model is required.");
    }

    bail!("Local completion fallback is not supported without a text generation backend.")
}

pub fn create_embedding(
    _tenant_id: &str,
    _project_id: &str,
    model: &str,
) -> Result<CreateEmbeddingResponse> {
    if model.trim().is_empty() {
        bail!("Embedding model is required.");
    }

    bail!("Local embedding fallback is not supported without an embedding backend.")
}

pub fn create_moderation(
    _tenant_id: &str,
    _project_id: &str,
    model: &str,
) -> Result<ModerationResponse> {
    if model.trim().is_empty() {
        bail!("Moderation model is required.");
    }

    bail!("Local moderation fallback is not supported without a moderation backend.")
}

pub fn create_image_generation(
    _tenant_id: &str,
    _project_id: &str,
    model: &str,
) -> Result<ImagesResponse> {
    if model.trim().is_empty() {
        bail!("Image generation model is required.");
    }

    bail!("Local image generation fallback is not supported without an image backend.")
}

pub fn create_image_edit(
    _tenant_id: &str,
    _project_id: &str,
    request: &CreateImageEditRequest,
) -> Result<ImagesResponse> {
    if request.prompt.trim().is_empty() {
        bail!("Image edit prompt is required.");
    }

    bail!("Local image edit fallback is not supported without an image backend.")
}

pub fn create_image_variation(
    _tenant_id: &str,
    _project_id: &str,
    request: &CreateImageVariationRequest,
) -> Result<ImagesResponse> {
    if request.image.bytes.is_empty() {
        bail!("Image input is required.");
    }

    bail!("Local image variation fallback is not supported without an image backend.")
}

pub fn create_transcription(
    _tenant_id: &str,
    _project_id: &str,
    model: &str,
) -> Result<TranscriptionObject> {
    if model.trim().is_empty() {
        bail!("Transcription model is required.");
    }

    bail!("Local transcription fallback is not supported without a transcription backend.")
}

pub fn create_translation(
    _tenant_id: &str,
    _project_id: &str,
    model: &str,
) -> Result<TranslationObject> {
    if model.trim().is_empty() {
        bail!("Translation model is required.");
    }

    bail!("Local translation fallback is not supported without a translation backend.")
}

pub fn list_audio_voices(_tenant_id: &str, _project_id: &str) -> Result<ListVoicesResponse> {
    bail!("Local audio voice listing fallback is not supported without an upstream provider.")
}

pub fn create_audio_voice_consent(
    _tenant_id: &str,
    _project_id: &str,
    request: &CreateVoiceConsentRequest,
) -> Result<VoiceConsentObject> {
    if request.voice.trim().is_empty() {
        bail!("Voice id is required.");
    }
    if request.name.trim().is_empty() {
        bail!("Voice consent name is required.");
    }
    if request.consent_text.trim().is_empty() {
        bail!("Voice consent text is required.");
    }

    bail!("Local voice consent fallback is not supported without a consent backend.")
}

pub fn create_speech_response(
    _tenant_id: &str,
    _project_id: &str,
    request: &CreateSpeechRequest,
) -> Result<SpeechResponse> {
    if request.model.trim().is_empty() {
        bail!("Speech model is required.");
    }
    if request.voice.trim().is_empty() {
        bail!("Speech voice is required.");
    }
    if request.input.trim().is_empty() {
        bail!("Speech input is required.");
    }

    normalize_local_speech_format(request.response_format.as_deref().unwrap_or("wav"))?;
    bail!("Local speech fallback is not supported without a speech synthesis backend.")
}
