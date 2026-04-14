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
    if name.trim().is_empty() {
        bail!("Vector store name is required.");
    }

    let _ = name;
    bail!("Local vector store fallback is not supported without an upstream provider.")
}

pub fn list_vector_stores(_tenant_id: &str, _project_id: &str) -> Result<ListVectorStoresResponse> {
    bail!("Local vector store listing fallback is not supported without an upstream provider.")
}

fn ensure_local_vector_store_exists(vector_store_id: &str) -> Result<()> {
    if !local_object_id_matches(vector_store_id, "vs") {
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
    bail!("vector store not found")
}

pub fn update_vector_store(
    _tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    name: &str,
) -> Result<VectorStoreObject> {
    ensure_local_vector_store_exists(vector_store_id)?;
    if name.trim().is_empty() {
        bail!("Vector store name is required.");
    }

    let _ = name;
    bail!("vector store not found")
}

pub fn delete_vector_store(
    _tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
) -> Result<DeleteVectorStoreResponse> {
    ensure_local_vector_store_exists(vector_store_id)?;
    bail!("vector store not found")
}

pub fn search_vector_store(
    _tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    query: &str,
) -> Result<SearchVectorStoreResponse> {
    ensure_local_vector_store_exists(vector_store_id)?;
    if query.trim().is_empty() {
        bail!("Vector store search query is required.");
    }

    bail!("Persisted local vector store index state is required for local search.")
}

fn ensure_local_vector_store_file_exists(vector_store_id: &str, file_id: &str) -> Result<()> {
    ensure_local_vector_store_exists(vector_store_id)?;
    if !local_object_id_matches(file_id, "file") {
        bail!("vector store file not found");
    }

    Ok(())
}

fn ensure_local_vector_store_file_batch_exists(
    vector_store_id: &str,
    batch_id: &str,
) -> Result<()> {
    ensure_local_vector_store_exists(vector_store_id)?;
    if !local_object_id_matches(batch_id, "vsfb") {
        bail!("vector store file batch not found");
    }

    Ok(())
}

pub fn create_vector_store_file(
    _tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    file_id: &str,
) -> Result<VectorStoreFileObject> {
    ensure_local_vector_store_exists(vector_store_id)?;
    if !local_object_id_matches(file_id, "file") {
        bail!("A local file id is required for local vector store fallback.");
    }

    bail!("Persisted local vector store file state is required for local file attachment.")
}

pub fn list_vector_store_files(
    _tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
) -> Result<ListVectorStoreFilesResponse> {
    ensure_local_vector_store_exists(vector_store_id)?;
    bail!("Persisted local vector store file state is required for local file listing.")
}

pub fn get_vector_store_file(
    _tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    file_id: &str,
) -> Result<VectorStoreFileObject> {
    ensure_local_vector_store_file_exists(vector_store_id, file_id)?;
    bail!("vector store file not found")
}

pub fn delete_vector_store_file(
    _tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    file_id: &str,
) -> Result<DeleteVectorStoreFileResponse> {
    ensure_local_vector_store_file_exists(vector_store_id, file_id)?;
    bail!("vector store file not found")
}

pub fn create_vector_store_file_batch<T: AsRef<str>>(
    _tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    file_ids: &[T],
) -> Result<VectorStoreFileBatchObject> {
    ensure_local_vector_store_exists(vector_store_id)?;
    if file_ids.is_empty() {
        bail!("At least one local file id is required for local vector store fallback.");
    }
    if file_ids
        .iter()
        .map(AsRef::as_ref)
        .any(|file_id| !local_object_id_matches(file_id, "file"))
    {
        bail!("A local file id is required for local vector store fallback.");
    }

    bail!("Persisted local vector store file batch state is required for local batch creation.")
}

pub fn get_vector_store_file_batch(
    _tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    batch_id: &str,
) -> Result<VectorStoreFileBatchObject> {
    ensure_local_vector_store_file_batch_exists(vector_store_id, batch_id)?;
    bail!("vector store file batch not found")
}

pub fn cancel_vector_store_file_batch(
    _tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    batch_id: &str,
) -> Result<VectorStoreFileBatchObject> {
    ensure_local_vector_store_file_batch_exists(vector_store_id, batch_id)?;
    bail!("vector store file batch not found")
}

pub fn list_vector_store_file_batch_files(
    _tenant_id: &str,
    _project_id: &str,
    vector_store_id: &str,
    batch_id: &str,
) -> Result<ListVectorStoreFilesResponse> {
    ensure_local_vector_store_file_batch_exists(vector_store_id, batch_id)?;
    bail!("Persisted local vector store file batch state is required for local batch file listing.")
}
