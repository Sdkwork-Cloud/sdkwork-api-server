use super::*;

pub async fn relay_fine_tuning_job_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateFineTuningJobRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "fine_tuning",
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
        ProviderRequest::FineTuningJobs(request),
    )
    .await
}

pub async fn relay_list_fine_tuning_jobs_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "fine_tuning", "jobs").await?;
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
        ProviderRequest::FineTuningJobsList,
    )
    .await
}

pub async fn relay_get_fine_tuning_job_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    job_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "fine_tuning", job_id).await?;
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
        ProviderRequest::FineTuningJobsRetrieve(job_id),
    )
    .await
}

pub async fn relay_cancel_fine_tuning_job_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    job_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "fine_tuning", job_id).await?;
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
        ProviderRequest::FineTuningJobsCancel(job_id),
    )
    .await
}

pub async fn relay_list_fine_tuning_job_events_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    job_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "fine_tuning", job_id).await?;
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
        ProviderRequest::FineTuningJobsEvents(job_id),
    )
    .await
}

pub async fn relay_list_fine_tuning_job_checkpoints_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    job_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "fine_tuning", job_id).await?;
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
        ProviderRequest::FineTuningJobsCheckpoints(job_id),
    )
    .await
}

pub async fn relay_pause_fine_tuning_job_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    job_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(project_id), "fine_tuning", job_id).await?;
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
        ProviderRequest::FineTuningJobsPause(job_id),
    )
    .await
}

pub async fn relay_resume_fine_tuning_job_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    job_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(project_id), "fine_tuning", job_id).await?;
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
        ProviderRequest::FineTuningJobsResume(job_id),
    )
    .await
}

pub async fn relay_fine_tuning_checkpoint_permissions_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    checkpoint_id: &str,
    request: &CreateFineTuningCheckpointPermissionsRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(project_id),
        "fine_tuning",
        checkpoint_id,
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
        ProviderRequest::FineTuningCheckpointPermissions(checkpoint_id, request),
    )
    .await
}

pub async fn relay_list_fine_tuning_checkpoint_permissions_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    checkpoint_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(project_id),
        "fine_tuning",
        checkpoint_id,
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
        ProviderRequest::FineTuningCheckpointPermissionsList(checkpoint_id),
    )
    .await
}

pub async fn relay_delete_fine_tuning_checkpoint_permission_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    checkpoint_id: &str,
    permission_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(project_id),
        "fine_tuning",
        checkpoint_id,
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
        ProviderRequest::FineTuningCheckpointPermissionsDelete(checkpoint_id, permission_id),
    )
    .await
}

pub fn create_fine_tuning_job(
    _tenant_id: &str,
    _project_id: &str,
    request: &CreateFineTuningJobRequest,
) -> Result<FineTuningJobObject> {
    if request.model.trim().is_empty() {
        bail!("Fine tuning model is required.");
    }
    if !local_object_id_matches(&request.training_file, "file") {
        bail!("A local training file id is required for local fine tuning fallback.");
    }

    bail!("Local fine-tuning job fallback is not supported without an upstream provider.")
}

fn ensure_local_fine_tuning_job_exists(job_id: &str) -> Result<()> {
    if !local_object_id_matches(job_id, "ftjob") {
        bail!("fine tuning job not found");
    }

    Ok(())
}

fn ensure_local_fine_tuning_checkpoint_exists(checkpoint_id: &str) -> Result<()> {
    if !local_object_id_matches(checkpoint_id, "ftckpt") {
        bail!("fine tuning checkpoint not found");
    }

    Ok(())
}

fn ensure_local_fine_tuning_checkpoint_permission_exists(
    checkpoint_id: &str,
    permission_id: &str,
) -> Result<()> {
    ensure_local_fine_tuning_checkpoint_exists(checkpoint_id)?;
    if !local_object_id_matches(permission_id, "perm") {
        bail!("fine tuning checkpoint permission not found");
    }

    Ok(())
}

pub fn list_fine_tuning_jobs(
    _tenant_id: &str,
    _project_id: &str,
) -> Result<ListFineTuningJobsResponse> {
    bail!("Local fine-tuning job listing fallback is not supported without an upstream provider.")
}

pub fn get_fine_tuning_job(
    _tenant_id: &str,
    _project_id: &str,
    job_id: &str,
) -> Result<FineTuningJobObject> {
    ensure_local_fine_tuning_job_exists(job_id)?;
    bail!("fine tuning job not found")
}

pub fn cancel_fine_tuning_job(
    _tenant_id: &str,
    _project_id: &str,
    job_id: &str,
) -> Result<FineTuningJobObject> {
    ensure_local_fine_tuning_job_exists(job_id)?;
    bail!("fine tuning job not found")
}

pub fn list_fine_tuning_job_events(
    _tenant_id: &str,
    _project_id: &str,
    job_id: &str,
) -> Result<ListFineTuningJobEventsResponse> {
    ensure_local_fine_tuning_job_exists(job_id)?;
    bail!("Persisted local fine tuning job event state is required for local event listing.")
}

pub fn list_fine_tuning_job_checkpoints(
    _tenant_id: &str,
    _project_id: &str,
    job_id: &str,
) -> Result<ListFineTuningJobCheckpointsResponse> {
    ensure_local_fine_tuning_job_exists(job_id)?;
    bail!("Persisted local fine tuning checkpoint state is required for local checkpoint listing.")
}

pub fn pause_fine_tuning_job(
    _tenant_id: &str,
    _project_id: &str,
    job_id: &str,
) -> Result<FineTuningJobObject> {
    ensure_local_fine_tuning_job_exists(job_id)?;
    bail!("fine tuning job not found")
}

pub fn resume_fine_tuning_job(
    _tenant_id: &str,
    _project_id: &str,
    job_id: &str,
) -> Result<FineTuningJobObject> {
    ensure_local_fine_tuning_job_exists(job_id)?;
    bail!("fine tuning job not found")
}

pub fn create_fine_tuning_checkpoint_permissions(
    _tenant_id: &str,
    _project_id: &str,
    checkpoint_id: &str,
    request: &CreateFineTuningCheckpointPermissionsRequest,
) -> Result<ListFineTuningCheckpointPermissionsResponse> {
    ensure_local_fine_tuning_checkpoint_exists(checkpoint_id)?;
    let project_ids = request
        .project_ids
        .iter()
        .map(|project_id| project_id.trim())
        .filter(|project_id| !project_id.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if project_ids.is_empty() {
        bail!("At least one project id is required.");
    }

    let _ = project_ids;
    bail!("Persisted local fine tuning checkpoint permission state is required for local permission creation.")
}

pub fn list_fine_tuning_checkpoint_permissions(
    _tenant_id: &str,
    _project_id: &str,
    checkpoint_id: &str,
) -> Result<ListFineTuningCheckpointPermissionsResponse> {
    ensure_local_fine_tuning_checkpoint_exists(checkpoint_id)?;
    bail!("Persisted local fine tuning checkpoint permission state is required for local permission listing.")
}

pub fn delete_fine_tuning_checkpoint_permission(
    _tenant_id: &str,
    _project_id: &str,
    checkpoint_id: &str,
    permission_id: &str,
) -> Result<DeleteFineTuningCheckpointPermissionResponse> {
    ensure_local_fine_tuning_checkpoint_permission_exists(checkpoint_id, permission_id)?;
    bail!("fine tuning checkpoint permission not found")
}
