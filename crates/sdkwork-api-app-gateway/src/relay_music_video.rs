use super::*;

pub async fn relay_music_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    request: &CreateMusicRequest,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(project_id), "music", &request.model).await?;
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
        ProviderRequest::Music(request),
    )
    .await
}

pub async fn relay_list_music_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(project_id), "music", "music").await?;
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
        ProviderRequest::MusicList,
    )
    .await
}

pub async fn relay_get_music_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    music_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(project_id), "music", music_id).await?;
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
        ProviderRequest::MusicRetrieve(music_id),
    )
    .await
}

pub async fn relay_delete_music_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    music_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(project_id), "music", music_id).await?;
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
        ProviderRequest::MusicDelete(music_id),
    )
    .await
}

pub async fn relay_music_content_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    music_id: &str,
) -> Result<Option<ProviderStreamOutput>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(project_id), "music", music_id).await?;
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
        ProviderRequest::MusicContent(music_id),
    )
    .await
}

pub async fn relay_music_lyrics_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    request: &CreateMusicLyricsRequest,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(project_id), "music", "lyrics").await?;
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
        ProviderRequest::MusicLyrics(request),
    )
    .await
}

pub async fn relay_video_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateVideoRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "videos",
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
        ProviderRequest::Videos(request),
    )
    .await
}

pub async fn relay_list_videos_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "videos", "videos").await?;
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
        ProviderRequest::VideosList,
    )
    .await
}

pub async fn relay_get_video_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    video_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "videos", video_id).await?;
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
        ProviderRequest::VideosRetrieve(video_id),
    )
    .await
}

pub async fn relay_get_video_from_store_with_provider_id(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    provider_id: &str,
    video_id: &str,
) -> Result<Option<Value>> {
    let Some(provider) = store.find_provider(provider_id).await? else {
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
        ProviderRequest::VideosRetrieve(video_id),
    )
    .await
}

pub async fn relay_delete_video_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    video_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "videos", video_id).await?;
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
        ProviderRequest::VideosDelete(video_id),
    )
    .await
}

pub async fn relay_video_content_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    video_id: &str,
) -> Result<Option<ProviderStreamOutput>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "videos", video_id).await?;
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
        ProviderRequest::VideosContent(video_id),
    )
    .await
}

pub async fn relay_remix_video_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    video_id: &str,
    request: &RemixVideoRequest,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "videos", video_id).await?;
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
        ProviderRequest::VideosRemix(video_id, request),
    )
    .await
}

pub async fn relay_create_video_character_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    request: &CreateVideoCharacterRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(project_id),
        "videos",
        &request.video_id,
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
        ProviderRequest::VideoCharactersCreate(request),
    )
    .await
}

pub async fn relay_list_video_characters_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    video_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "videos", video_id).await?;
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
        ProviderRequest::VideoCharactersList(video_id),
    )
    .await
}

pub async fn relay_get_video_character_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    video_id: &str,
    character_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "videos", video_id).await?;
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
        ProviderRequest::VideoCharactersRetrieve(video_id, character_id),
    )
    .await
}

pub async fn relay_update_video_character_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    video_id: &str,
    character_id: &str,
    request: &UpdateVideoCharacterRequest,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "videos", video_id).await?;
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
        ProviderRequest::VideoCharactersUpdate(video_id, character_id, request),
    )
    .await
}

pub async fn relay_get_video_character_canonical_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    character_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(project_id), "videos", character_id).await?;
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
        ProviderRequest::VideoCharactersCanonicalRetrieve(character_id),
    )
    .await
}

pub async fn relay_edit_video_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    request: &EditVideoRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(project_id),
        "videos",
        &request.video_id,
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
        ProviderRequest::VideosEdits(request),
    )
    .await
}

pub async fn relay_extensions_video_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    request: &ExtendVideoRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(project_id),
        "videos",
        request.video_id.as_deref().unwrap_or("videos"),
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
        ProviderRequest::VideosExtensions(request),
    )
    .await
}

pub async fn relay_extend_video_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    video_id: &str,
    request: &ExtendVideoRequest,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "videos", video_id).await?;
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
        ProviderRequest::VideosExtend(video_id, request),
    )
    .await
}

pub fn create_music(
    _tenant_id: &str,
    _project_id: &str,
    request: &CreateMusicRequest,
) -> Result<MusicTracksResponse> {
    if request.model.trim().is_empty() {
        bail!("Music model is required.");
    }
    if request.prompt.trim().is_empty() {
        bail!("Music prompt is required.");
    }
    if let Some(continue_track_id) = request.continue_track_id.as_deref() {
        if !local_object_id_matches(continue_track_id, "music") {
            bail!("A local music track id is required for local continuation.");
        }
    }

    bail!("Local music fallback is not supported without an upstream provider.")
}

pub fn list_music(_tenant_id: &str, _project_id: &str) -> Result<MusicTracksResponse> {
    bail!("Local music listing fallback is not supported without an upstream provider.")
}

fn ensure_local_music_exists(music_id: &str) -> Result<()> {
    if !local_object_id_matches(music_id, "music") {
        bail!("music not found");
    }

    Ok(())
}

pub fn get_music(_tenant_id: &str, _project_id: &str, music_id: &str) -> Result<MusicObject> {
    ensure_local_music_exists(music_id)?;
    bail!("music not found")
}

pub fn delete_music(
    _tenant_id: &str,
    _project_id: &str,
    music_id: &str,
) -> Result<DeleteMusicResponse> {
    ensure_local_music_exists(music_id)?;
    bail!("music not found")
}

pub fn music_content(_tenant_id: &str, _project_id: &str, music_id: &str) -> Result<Vec<u8>> {
    ensure_local_music_exists(music_id)?;
    bail!("music not found")
}

pub fn create_music_lyrics(
    _tenant_id: &str,
    _project_id: &str,
    request: &CreateMusicLyricsRequest,
) -> Result<MusicLyricsObject> {
    if request.prompt.trim().is_empty() {
        bail!("Music lyrics prompt is required.");
    }

    bail!("Local music lyrics fallback is not supported without a lyrics generation backend.")
}

pub fn create_video(
    _tenant_id: &str,
    _project_id: &str,
    model: &str,
    prompt: &str,
) -> Result<VideosResponse> {
    if model.trim().is_empty() {
        bail!("Video model is required.");
    }
    if prompt.trim().is_empty() {
        bail!("Video prompt is required.");
    }

    bail!("Local video fallback is not supported without a persisted asset store.")
}

pub fn list_videos(_tenant_id: &str, _project_id: &str) -> Result<VideosResponse> {
    bail!("Local video listing fallback is not supported without an upstream provider.")
}

pub fn get_video(_tenant_id: &str, _project_id: &str, video_id: &str) -> Result<VideoObject> {
    ensure_local_video_exists(video_id)?;
    bail!("video not found")
}

fn ensure_local_video_exists(video_id: &str) -> Result<()> {
    if !local_object_id_matches(video_id, "video") {
        bail!("video not found");
    }

    Ok(())
}

fn ensure_local_video_character_exists(video_id: &str, character_id: &str) -> Result<()> {
    ensure_local_video_exists(video_id)?;
    if !local_object_id_matches(character_id, "char") {
        bail!("video character not found");
    }

    Ok(())
}

pub fn delete_video(
    _tenant_id: &str,
    _project_id: &str,
    video_id: &str,
) -> Result<DeleteVideoResponse> {
    ensure_local_video_exists(video_id)?;
    bail!("video not found")
}

pub fn video_content(_tenant_id: &str, _project_id: &str, video_id: &str) -> Result<Vec<u8>> {
    ensure_local_video_exists(video_id)?;
    bail!("video not found")
}

pub fn remix_video(
    _tenant_id: &str,
    _project_id: &str,
    video_id: &str,
    prompt: &str,
) -> Result<VideosResponse> {
    ensure_local_video_exists(video_id)?;
    if prompt.trim().is_empty() {
        bail!("Video prompt is required.");
    }

    bail!("Local video fallback is not supported without a persisted asset store.")
}

pub fn create_video_character(
    _tenant_id: &str,
    _project_id: &str,
    request: &CreateVideoCharacterRequest,
) -> Result<VideoCharacterObject> {
    ensure_local_video_exists(&request.video_id)?;
    if request.name.trim().is_empty() {
        bail!("Video character name is required.");
    }

    bail!("Persisted local video character state is required for local character creation.")
}

pub fn list_video_characters(
    _tenant_id: &str,
    _project_id: &str,
    video_id: &str,
) -> Result<VideoCharactersResponse> {
    ensure_local_video_exists(video_id)?;
    bail!("Persisted local video character state is required for local character listing.")
}

pub fn get_video_character(
    _tenant_id: &str,
    _project_id: &str,
    video_id: &str,
    character_id: &str,
) -> Result<VideoCharacterObject> {
    ensure_local_video_character_exists(video_id, character_id)?;
    bail!("video character not found")
}

pub fn get_video_character_canonical(
    _tenant_id: &str,
    _project_id: &str,
    character_id: &str,
) -> Result<VideoCharacterObject> {
    if !local_object_id_matches(character_id, "char") {
        bail!("video character not found");
    }

    bail!("video character not found")
}

pub fn update_video_character(
    _tenant_id: &str,
    _project_id: &str,
    video_id: &str,
    character_id: &str,
    request: &UpdateVideoCharacterRequest,
) -> Result<VideoCharacterObject> {
    ensure_local_video_character_exists(video_id, character_id)?;
    let Some(name) = request
        .name
        .as_deref()
        .filter(|name| !name.trim().is_empty())
    else {
        bail!("Video character name is required.");
    };

    let _ = name;
    bail!("video character not found")
}

pub fn extend_video(
    _tenant_id: &str,
    _project_id: &str,
    video_id: &str,
    prompt: &str,
) -> Result<VideosResponse> {
    ensure_local_video_exists(video_id)?;
    if prompt.trim().is_empty() {
        bail!("Video prompt is required.");
    }

    bail!("Local video fallback is not supported without a persisted asset store.")
}

pub fn edit_video(
    _tenant_id: &str,
    _project_id: &str,
    request: &EditVideoRequest,
) -> Result<VideosResponse> {
    ensure_local_video_exists(&request.video_id)?;
    if request.prompt.trim().is_empty() {
        bail!("Video prompt is required.");
    }

    bail!("Local video fallback is not supported without a persisted asset store.")
}

pub fn extensions_video(
    _tenant_id: &str,
    _project_id: &str,
    request: &ExtendVideoRequest,
) -> Result<VideosResponse> {
    if let Some(video_id) = request.video_id.as_deref() {
        ensure_local_video_exists(video_id)?;
    }
    if request.prompt.trim().is_empty() {
        bail!("Video prompt is required.");
    }

    bail!("Local video fallback is not supported without a persisted asset store.")
}
