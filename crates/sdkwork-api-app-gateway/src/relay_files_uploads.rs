use super::*;

pub async fn relay_file_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateFileRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "files",
        &request.purpose,
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
        ProviderRequest::Files(request),
    )
    .await
}

pub async fn relay_list_files_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "files", "files").await?;
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
        ProviderRequest::FilesList,
    )
    .await
}

pub async fn relay_get_file_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    file_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "files", file_id).await?;
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
        ProviderRequest::FilesRetrieve(file_id),
    )
    .await
}

pub async fn relay_delete_file_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    file_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "files", file_id).await?;
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
        ProviderRequest::FilesDelete(file_id),
    )
    .await
}

pub async fn relay_file_content_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    file_id: &str,
) -> Result<Option<ProviderStreamOutput>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "files", file_id).await?;
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
        ProviderRequest::FilesContent(file_id),
    )
    .await
}

pub async fn relay_upload_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateUploadRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "uploads",
        &request.purpose,
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
        ProviderRequest::Uploads(request),
    )
    .await
}

pub async fn relay_upload_part_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &AddUploadPartRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "uploads",
        &request.upload_id,
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
        ProviderRequest::UploadParts(request),
    )
    .await
}

pub async fn relay_complete_upload_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CompleteUploadRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "uploads",
        &request.upload_id,
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
        ProviderRequest::UploadComplete(request),
    )
    .await
}

pub async fn relay_cancel_upload_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    upload_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "uploads", upload_id).await?;
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
        ProviderRequest::UploadCancel(upload_id),
    )
    .await
}

pub fn create_file(
    _tenant_id: &str,
    _project_id: &str,
    request: &CreateFileRequest,
) -> Result<FileObject> {
    if request.purpose.trim().is_empty() {
        bail!("File purpose is required.");
    }
    if request.filename.trim().is_empty() {
        bail!("File filename is required.");
    }
    bail!("Local file fallback is not supported without an upstream provider.")
}

pub fn list_files(_tenant_id: &str, _project_id: &str) -> Result<ListFilesResponse> {
    bail!("Local file listing fallback is not supported without an upstream provider.")
}

fn ensure_local_file_exists(file_id: &str) -> Result<()> {
    if !local_object_id_matches(file_id, "file") {
        bail!("file not found");
    }

    Ok(())
}

pub fn get_file(_tenant_id: &str, _project_id: &str, file_id: &str) -> Result<FileObject> {
    ensure_local_file_exists(file_id)?;
    bail!("file not found")
}

pub fn delete_file(
    _tenant_id: &str,
    _project_id: &str,
    file_id: &str,
) -> Result<DeleteFileResponse> {
    ensure_local_file_exists(file_id)?;
    bail!("file not found")
}

pub fn file_content(_tenant_id: &str, _project_id: &str, _file_id: &str) -> Result<Vec<u8>> {
    ensure_local_file_exists(_file_id)?;
    bail!("file not found")
}

pub fn create_upload(
    _tenant_id: &str,
    _project_id: &str,
    request: &CreateUploadRequest,
) -> Result<UploadObject> {
    if request.purpose.trim().is_empty() {
        bail!("Upload purpose is required.");
    }
    if request.filename.trim().is_empty() {
        bail!("Upload filename is required.");
    }
    if request.mime_type.trim().is_empty() {
        bail!("Upload MIME type is required.");
    }

    bail!("Local upload fallback is not supported without an upstream provider.")
}

pub fn create_upload_part(
    _tenant_id: &str,
    _project_id: &str,
    request: &AddUploadPartRequest,
) -> Result<UploadPartObject> {
    ensure_local_upload_exists(&request.upload_id)?;
    bail!("Persisted local upload part state is required for local part creation.")
}

fn ensure_local_upload_exists(upload_id: &str) -> Result<()> {
    if !local_object_id_matches(upload_id, "upload") {
        bail!("upload not found");
    }

    Ok(())
}

pub fn complete_upload(
    _tenant_id: &str,
    _project_id: &str,
    request: &CompleteUploadRequest,
) -> Result<UploadObject> {
    ensure_local_upload_exists(&request.upload_id)?;
    bail!("upload not found")
}

pub fn cancel_upload(_tenant_id: &str, _project_id: &str, upload_id: &str) -> Result<UploadObject> {
    ensure_local_upload_exists(upload_id)?;
    bail!("upload not found")
}
