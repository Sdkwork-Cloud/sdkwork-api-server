use super::*;

pub async fn relay_container_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    request: &CreateContainerRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(project_id),
        "containers",
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
        ProviderRequest::Containers(request),
    )
    .await
}

pub async fn relay_list_containers_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(project_id),
        "containers",
        "containers",
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
        ProviderRequest::ContainersList,
    )
    .await
}

pub async fn relay_get_container_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(project_id),
        "containers",
        container_id,
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
        ProviderRequest::ContainersRetrieve(container_id),
    )
    .await
}

pub async fn relay_delete_container_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(project_id),
        "containers",
        container_id,
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
        ProviderRequest::ContainersDelete(container_id),
    )
    .await
}

pub async fn relay_container_file_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
    request: &CreateContainerFileRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(project_id),
        "containers",
        container_id,
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
        ProviderRequest::ContainerFiles(container_id, request),
    )
    .await
}

pub async fn relay_list_container_files_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(project_id),
        "containers",
        container_id,
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
        ProviderRequest::ContainerFilesList(container_id),
    )
    .await
}

pub async fn relay_get_container_file_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
    file_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(project_id),
        "containers",
        container_id,
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
        ProviderRequest::ContainerFilesRetrieve(container_id, file_id),
    )
    .await
}

pub async fn relay_delete_container_file_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
    file_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(project_id),
        "containers",
        container_id,
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
        ProviderRequest::ContainerFilesDelete(container_id, file_id),
    )
    .await
}

pub async fn relay_container_file_content_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    container_id: &str,
    file_id: &str,
) -> Result<Option<ProviderStreamOutput>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(project_id),
        "containers",
        container_id,
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
        ProviderRequest::ContainerFilesContent(container_id, file_id),
    )
    .await
}

pub fn create_container(
    _tenant_id: &str,
    _project_id: &str,
    request: &CreateContainerRequest,
) -> Result<ContainerObject> {
    Ok(ContainerObject::new("container_1", &request.name))
}

pub fn list_containers(_tenant_id: &str, _project_id: &str) -> Result<ListContainersResponse> {
    Ok(ListContainersResponse::new(vec![ContainerObject::new(
        "container_1",
        "ci-container",
    )]))
}

fn ensure_local_container_exists(container_id: &str) -> Result<()> {
    if container_id != "container_1" {
        bail!("container not found");
    }

    Ok(())
}

fn ensure_local_container_file_exists(container_id: &str, file_id: &str) -> Result<()> {
    ensure_local_container_exists(container_id)?;
    if file_id != "file_1" {
        bail!("container file not found");
    }

    Ok(())
}

pub fn get_container(
    _tenant_id: &str,
    _project_id: &str,
    container_id: &str,
) -> Result<ContainerObject> {
    ensure_local_container_exists(container_id)?;
    Ok(ContainerObject::new(container_id, "ci-container"))
}

pub fn delete_container(
    _tenant_id: &str,
    _project_id: &str,
    container_id: &str,
) -> Result<DeleteContainerResponse> {
    ensure_local_container_exists(container_id)?;
    Ok(DeleteContainerResponse::deleted(container_id))
}

pub fn create_container_file(
    _tenant_id: &str,
    _project_id: &str,
    container_id: &str,
    request: &CreateContainerFileRequest,
) -> Result<ContainerFileObject> {
    ensure_local_container_exists(container_id)?;
    Ok(ContainerFileObject::new(&request.file_id, container_id))
}

pub fn list_container_files(
    _tenant_id: &str,
    _project_id: &str,
    container_id: &str,
) -> Result<ListContainerFilesResponse> {
    ensure_local_container_exists(container_id)?;
    Ok(ListContainerFilesResponse::new(vec![
        ContainerFileObject::new("file_1", container_id),
    ]))
}

pub fn get_container_file(
    _tenant_id: &str,
    _project_id: &str,
    container_id: &str,
    file_id: &str,
) -> Result<ContainerFileObject> {
    ensure_local_container_file_exists(container_id, file_id)?;
    Ok(ContainerFileObject::new(file_id, container_id))
}

pub fn delete_container_file(
    _tenant_id: &str,
    _project_id: &str,
    container_id: &str,
    file_id: &str,
) -> Result<DeleteContainerFileResponse> {
    ensure_local_container_file_exists(container_id, file_id)?;
    Ok(DeleteContainerFileResponse::deleted(file_id))
}

pub fn container_file_content(
    _tenant_id: &str,
    _project_id: &str,
    container_id: &str,
    file_id: &str,
) -> Result<Vec<u8>> {
    ensure_local_container_file_exists(container_id, file_id)?;
    Ok(b"CONTAINER-FILE".to_vec())
}
