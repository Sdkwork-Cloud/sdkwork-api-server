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
    Ok(FileObject::with_bytes(
        "file_1",
        &request.filename,
        &request.purpose,
        request.bytes.len() as u64,
    ))
}

pub fn list_files(_tenant_id: &str, _project_id: &str) -> Result<ListFilesResponse> {
    Ok(ListFilesResponse::new(vec![FileObject::with_bytes(
        "file_1",
        "train.jsonl",
        "fine-tune",
        2,
    )]))
}

fn ensure_local_file_exists(file_id: &str) -> Result<()> {
    if file_id != "file_1" {
        bail!("file not found");
    }

    Ok(())
}

pub fn get_file(_tenant_id: &str, _project_id: &str, file_id: &str) -> Result<FileObject> {
    ensure_local_file_exists(file_id)?;
    Ok(FileObject::with_bytes(
        file_id,
        "train.jsonl",
        "fine-tune",
        2,
    ))
}

pub fn delete_file(
    _tenant_id: &str,
    _project_id: &str,
    file_id: &str,
) -> Result<DeleteFileResponse> {
    ensure_local_file_exists(file_id)?;
    Ok(DeleteFileResponse::deleted(file_id))
}

pub fn file_content(_tenant_id: &str, _project_id: &str, _file_id: &str) -> Result<Vec<u8>> {
    ensure_local_file_exists(_file_id)?;
    Ok(b"{}".to_vec())
}

pub fn create_upload(
    _tenant_id: &str,
    _project_id: &str,
    request: &CreateUploadRequest,
) -> Result<UploadObject> {
    Ok(UploadObject::with_details(
        "upload_1",
        &request.filename,
        &request.purpose,
        &request.mime_type,
        request.bytes,
        vec![],
    ))
}

pub fn create_upload_part(
    _tenant_id: &str,
    _project_id: &str,
    request: &AddUploadPartRequest,
) -> Result<UploadPartObject> {
    ensure_local_upload_exists(&request.upload_id)?;
    Ok(UploadPartObject::new("part_1", &request.upload_id))
}

fn ensure_local_upload_exists(upload_id: &str) -> Result<()> {
    if upload_id != "upload_1" {
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
    Ok(UploadObject::completed(
        &request.upload_id,
        "input.jsonl",
        "batch",
        "application/jsonl",
        0,
        request.part_ids.clone(),
    ))
}

pub fn cancel_upload(_tenant_id: &str, _project_id: &str, upload_id: &str) -> Result<UploadObject> {
    ensure_local_upload_exists(upload_id)?;
    Ok(UploadObject::cancelled(
        upload_id,
        "input.jsonl",
        "batch",
        "application/jsonl",
        0,
        vec![],
    ))
}
