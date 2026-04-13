use super::*;

pub async fn relay_vector_store_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateVectorStoreRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_stores",
        &request.name,
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
        ProviderRequest::VectorStores(request),
    )
    .await
}

pub async fn relay_list_vector_stores_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_stores",
        "vector_stores",
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
        ProviderRequest::VectorStoresList,
    )
    .await
}

pub async fn relay_get_vector_store_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_stores",
        vector_store_id,
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
        ProviderRequest::VectorStoresRetrieve(vector_store_id),
    )
    .await
}

pub async fn relay_update_vector_store_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    request: &UpdateVectorStoreRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_stores",
        vector_store_id,
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
        ProviderRequest::VectorStoresUpdate(vector_store_id, request),
    )
    .await
}

pub async fn relay_delete_vector_store_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_stores",
        vector_store_id,
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
        ProviderRequest::VectorStoresDelete(vector_store_id),
    )
    .await
}

pub async fn relay_search_vector_store_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    request: &SearchVectorStoreRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_store_search",
        vector_store_id,
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
        ProviderRequest::VectorStoresSearch(vector_store_id, request),
    )
    .await
}

pub async fn relay_vector_store_file_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    request: &CreateVectorStoreFileRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_store_files",
        vector_store_id,
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
        ProviderRequest::VectorStoreFiles(vector_store_id, request),
    )
    .await
}

pub async fn relay_list_vector_store_files_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_store_files",
        vector_store_id,
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
        ProviderRequest::VectorStoreFilesList(vector_store_id),
    )
    .await
}

pub async fn relay_get_vector_store_file_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    file_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_store_files",
        vector_store_id,
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
        ProviderRequest::VectorStoreFilesRetrieve(vector_store_id, file_id),
    )
    .await
}

pub async fn relay_delete_vector_store_file_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    file_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_store_files",
        vector_store_id,
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
        ProviderRequest::VectorStoreFilesDelete(vector_store_id, file_id),
    )
    .await
}

pub async fn relay_vector_store_file_batch_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    request: &CreateVectorStoreFileBatchRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_store_file_batches",
        vector_store_id,
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
        ProviderRequest::VectorStoreFileBatches(vector_store_id, request),
    )
    .await
}

pub async fn relay_get_vector_store_file_batch_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    batch_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_store_file_batches",
        vector_store_id,
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
        ProviderRequest::VectorStoreFileBatchesRetrieve(vector_store_id, batch_id),
    )
    .await
}

pub async fn relay_cancel_vector_store_file_batch_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    batch_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_store_file_batches",
        vector_store_id,
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
        ProviderRequest::VectorStoreFileBatchesCancel(vector_store_id, batch_id),
    )
    .await
}

pub async fn relay_list_vector_store_file_batch_files_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    batch_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "vector_store_file_batches",
        vector_store_id,
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
        ProviderRequest::VectorStoreFileBatchesListFiles(vector_store_id, batch_id),
    )
    .await
}

pub fn create_vector_store(
    _tenant_id: &str,
    _project_id: &str,
    name: &str,
) -> Result<VectorStoreObject> {
    Ok(VectorStoreObject::new("vs_1", name))
}

pub fn list_vector_stores(_tenant_id: &str, _project_id: &str) -> Result<ListVectorStoresResponse> {
    Ok(ListVectorStoresResponse::new(vec![VectorStoreObject::new(
        "vs_1", "kb-main",
    )]))
}

fn ensure_local_vector_store_exists(vector_store_id: &str) -> Result<()> {
    if vector_store_id != "vs_1" {
        bail!("vector store not found");
    }

    Ok(())
}

pub fn get_vector_store(
    _tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
) -> Result<VectorStoreObject> {
    ensure_local_vector_store_exists(vector_store_id)?;
    Ok(VectorStoreObject::new(vector_store_id, "kb-main"))
}

pub fn update_vector_store(
    _tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    name: &str,
) -> Result<VectorStoreObject> {
    ensure_local_vector_store_exists(vector_store_id)?;
    Ok(VectorStoreObject::new(vector_store_id, name))
}

pub fn delete_vector_store(
    _tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
) -> Result<DeleteVectorStoreResponse> {
    ensure_local_vector_store_exists(vector_store_id)?;
    Ok(DeleteVectorStoreResponse::deleted(vector_store_id))
}

pub fn search_vector_store(
    _tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    query: &str,
) -> Result<SearchVectorStoreResponse> {
    ensure_local_vector_store_exists(vector_store_id)?;
    Ok(SearchVectorStoreResponse::sample(query))
}

fn ensure_local_vector_store_file_exists(vector_store_id: &str, file_id: &str) -> Result<()> {
    if vector_store_id != "vs_1" || file_id != "file_1" {
        bail!("vector store file not found");
    }

    Ok(())
}

fn ensure_local_vector_store_file_batch_exists(
    vector_store_id: &str,
    batch_id: &str,
) -> Result<()> {
    if vector_store_id != "vs_1" || batch_id != "vsfb_1" {
        bail!("vector store file batch not found");
    }

    Ok(())
}

pub fn create_vector_store_file(
    _tenant_id: &str,
    _project_id: &str,
    _vector_store_id: &str,
    file_id: &str,
) -> Result<VectorStoreFileObject> {
    Ok(VectorStoreFileObject::new(file_id))
}

pub fn list_vector_store_files(
    _tenant_id: &str,
    _project_id: &str,
    _vector_store_id: &str,
) -> Result<ListVectorStoreFilesResponse> {
    Ok(ListVectorStoreFilesResponse::new(vec![
        VectorStoreFileObject::new("file_1"),
    ]))
}

pub fn get_vector_store_file(
    _tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    file_id: &str,
) -> Result<VectorStoreFileObject> {
    ensure_local_vector_store_file_exists(vector_store_id, file_id)?;
    Ok(VectorStoreFileObject::new(file_id))
}

pub fn delete_vector_store_file(
    _tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    file_id: &str,
) -> Result<DeleteVectorStoreFileResponse> {
    ensure_local_vector_store_file_exists(vector_store_id, file_id)?;
    Ok(DeleteVectorStoreFileResponse::deleted(file_id))
}

pub fn create_vector_store_file_batch<T: AsRef<str>>(
    _tenant_id: &str,
    _project_id: &str,
    _vector_store_id: &str,
    file_ids: &[T],
) -> Result<VectorStoreFileBatchObject> {
    let _ = file_ids.first().map(AsRef::as_ref);
    Ok(VectorStoreFileBatchObject::new("vsfb_1"))
}

pub fn get_vector_store_file_batch(
    _tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    batch_id: &str,
) -> Result<VectorStoreFileBatchObject> {
    ensure_local_vector_store_file_batch_exists(vector_store_id, batch_id)?;
    Ok(VectorStoreFileBatchObject::new(batch_id))
}

pub fn cancel_vector_store_file_batch(
    _tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    batch_id: &str,
) -> Result<VectorStoreFileBatchObject> {
    ensure_local_vector_store_file_batch_exists(vector_store_id, batch_id)?;
    Ok(VectorStoreFileBatchObject::cancelled(batch_id))
}

pub fn list_vector_store_file_batch_files(
    _tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    batch_id: &str,
) -> Result<ListVectorStoreFilesResponse> {
    ensure_local_vector_store_file_batch_exists(vector_store_id, batch_id)?;
    Ok(ListVectorStoreFilesResponse::new(vec![
        VectorStoreFileObject::new("file_1"),
    ]))
}
