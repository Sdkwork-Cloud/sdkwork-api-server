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
    Ok(MusicTracksResponse::new(vec![MusicObject::new("music_1")
        .with_status("completed")
        .with_model(&request.model)
        .with_title(
            request
                .title
                .clone()
                .unwrap_or_else(|| "SDKWork Track".to_owned()),
        )
        .with_audio_url("https://example.com/music.mp3")
        .with_lyrics(
            request
                .lyrics
                .clone()
                .unwrap_or_else(|| "We rise with the skyline".to_owned()),
        )
        .with_duration_seconds(
            request.duration_seconds.unwrap_or(123.0),
        )]))
}

pub fn list_music(_tenant_id: &str, _project_id: &str) -> Result<MusicTracksResponse> {
    Ok(MusicTracksResponse::new(vec![MusicObject::new("music_1")
        .with_status("completed")
        .with_model("suno-v4")
        .with_title("SDKWork Track")
        .with_audio_url("https://example.com/music.mp3")
        .with_duration_seconds(123.0)]))
}

fn ensure_local_music_exists(music_id: &str) -> Result<()> {
    if music_id != "music_1" {
        bail!("music not found");
    }

    Ok(())
}

pub fn get_music(_tenant_id: &str, _project_id: &str, music_id: &str) -> Result<MusicObject> {
    ensure_local_music_exists(music_id)?;
    Ok(MusicObject::new(music_id)
        .with_status("completed")
        .with_model("suno-v4")
        .with_title("SDKWork Track")
        .with_audio_url("https://example.com/music.mp3")
        .with_duration_seconds(123.0))
}

pub fn delete_music(
    _tenant_id: &str,
    _project_id: &str,
    music_id: &str,
) -> Result<DeleteMusicResponse> {
    ensure_local_music_exists(music_id)?;
    Ok(DeleteMusicResponse::deleted(music_id))
}

pub fn music_content(_tenant_id: &str, _project_id: &str, music_id: &str) -> Result<Vec<u8>> {
    ensure_local_music_exists(music_id)?;
    Ok(vec![
        0x49, 0x44, 0x33, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x21,
    ])
}

pub fn create_music_lyrics(
    _tenant_id: &str,
    _project_id: &str,
    request: &CreateMusicLyricsRequest,
) -> Result<MusicLyricsObject> {
    Ok(
        MusicLyricsObject::new("lyrics_1", "completed", &request.prompt).with_title(
            request
                .title
                .clone()
                .unwrap_or_else(|| "SDKWork Lyrics".to_owned()),
        ),
    )
}

pub fn create_video(
    _tenant_id: &str,
    _project_id: &str,
    _model: &str,
    _prompt: &str,
) -> Result<VideosResponse> {
    Ok(VideosResponse::new(vec![VideoObject::new(
        "video_1",
        "https://example.com/video.mp4",
    )]))
}

pub fn list_videos(_tenant_id: &str, _project_id: &str) -> Result<VideosResponse> {
    Ok(VideosResponse::new(vec![VideoObject::new(
        "video_1",
        "https://example.com/video.mp4",
    )]))
}

pub fn get_video(_tenant_id: &str, _project_id: &str, video_id: &str) -> Result<VideoObject> {
    ensure_local_video_exists(video_id)?;
    Ok(VideoObject::new(video_id, "https://example.com/video.mp4"))
}

fn ensure_local_video_exists(video_id: &str) -> Result<()> {
    if video_id != "video_1" {
        bail!("video not found");
    }

    Ok(())
}

pub fn delete_video(
    _tenant_id: &str,
    _project_id: &str,
    video_id: &str,
) -> Result<DeleteVideoResponse> {
    ensure_local_video_exists(video_id)?;
    Ok(DeleteVideoResponse::deleted(video_id))
}

pub fn video_content(_tenant_id: &str, _project_id: &str, video_id: &str) -> Result<Vec<u8>> {
    ensure_local_video_exists(video_id)?;
    Ok(b"VIDEO".to_vec())
}

pub fn remix_video(
    _tenant_id: &str,
    _project_id: &str,
    _video_id: &str,
    _prompt: &str,
) -> Result<VideosResponse> {
    Ok(VideosResponse::new(vec![VideoObject::new(
        "video_1_remix",
        "https://example.com/video-remix.mp4",
    )]))
}

pub fn create_video_character(
    _tenant_id: &str,
    _project_id: &str,
    request: &CreateVideoCharacterRequest,
) -> Result<VideoCharacterObject> {
    Ok(VideoCharacterObject::new("char_1", &request.name))
}

pub fn list_video_characters(
    _tenant_id: &str,
    _project_id: &str,
    _video_id: &str,
) -> Result<VideoCharactersResponse> {
    Ok(VideoCharactersResponse::new(vec![
        VideoCharacterObject::new("char_1", "Hero"),
    ]))
}

pub fn get_video_character(
    _tenant_id: &str,
    _project_id: &str,
    _video_id: &str,
    character_id: &str,
) -> Result<VideoCharacterObject> {
    Ok(VideoCharacterObject::new(character_id, "Hero"))
}

pub fn get_video_character_canonical(
    _tenant_id: &str,
    _project_id: &str,
    character_id: &str,
) -> Result<VideoCharacterObject> {
    Ok(VideoCharacterObject::new(character_id, "Hero"))
}

pub fn update_video_character(
    _tenant_id: &str,
    _project_id: &str,
    _video_id: &str,
    character_id: &str,
    request: &UpdateVideoCharacterRequest,
) -> Result<VideoCharacterObject> {
    Ok(VideoCharacterObject::new(
        character_id,
        request.name.as_deref().unwrap_or("Hero"),
    ))
}

pub fn extend_video(
    _tenant_id: &str,
    _project_id: &str,
    _video_id: &str,
    _prompt: &str,
) -> Result<VideosResponse> {
    Ok(VideosResponse::new(vec![VideoObject::new(
        "video_1_extended",
        "https://example.com/video-extended.mp4",
    )]))
}

pub fn edit_video(
    _tenant_id: &str,
    _project_id: &str,
    _request: &EditVideoRequest,
) -> Result<VideosResponse> {
    Ok(VideosResponse::new(vec![VideoObject::new(
        "video_1_edited",
        "https://example.com/video-edited.mp4",
    )]))
}

pub fn extensions_video(
    _tenant_id: &str,
    _project_id: &str,
    _request: &ExtendVideoRequest,
) -> Result<VideosResponse> {
    Ok(VideosResponse::new(vec![VideoObject::new(
        "video_1_extended",
        "https://example.com/video-extended.mp4",
    )]))
}
