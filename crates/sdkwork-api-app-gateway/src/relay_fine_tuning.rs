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
    model: &str,
) -> Result<FineTuningJobObject> {
    Ok(FineTuningJobObject::new("ftjob_1", model))
}

fn ensure_local_fine_tuning_job_exists(job_id: &str) -> Result<()> {
    if job_id != "ftjob_1" {
        bail!("fine tuning job not found");
    }

    Ok(())
}

fn ensure_local_fine_tuning_checkpoint_exists(checkpoint_id: &str) -> Result<()> {
    if checkpoint_id != "ft:gpt-4.1-mini:checkpoint-1" {
        bail!("fine tuning checkpoint not found");
    }

    Ok(())
}

fn ensure_local_fine_tuning_checkpoint_permission_exists(
    checkpoint_id: &str,
    permission_id: &str,
) -> Result<()> {
    ensure_local_fine_tuning_checkpoint_exists(checkpoint_id)?;
    if permission_id != "perm_1" {
        bail!("fine tuning checkpoint permission not found");
    }

    Ok(())
}

pub fn list_fine_tuning_jobs(
    _tenant_id: &str,
    _project_id: &str,
) -> Result<ListFineTuningJobsResponse> {
    Ok(ListFineTuningJobsResponse::new(vec![
        FineTuningJobObject::new("ftjob_1", "gpt-4.1-mini"),
    ]))
}

pub fn get_fine_tuning_job(
    _tenant_id: &str,
    _project_id: &str,
    job_id: &str,
) -> Result<FineTuningJobObject> {
    ensure_local_fine_tuning_job_exists(job_id)?;
    Ok(FineTuningJobObject::new(job_id, "gpt-4.1-mini"))
}

pub fn cancel_fine_tuning_job(
    _tenant_id: &str,
    _project_id: &str,
    job_id: &str,
) -> Result<FineTuningJobObject> {
    ensure_local_fine_tuning_job_exists(job_id)?;
    Ok(FineTuningJobObject::cancelled(job_id, "gpt-4.1-mini"))
}

pub fn list_fine_tuning_job_events(
    _tenant_id: &str,
    _project_id: &str,
    job_id: &str,
) -> Result<ListFineTuningJobEventsResponse> {
    ensure_local_fine_tuning_job_exists(job_id)?;
    Ok(ListFineTuningJobEventsResponse::new(vec![
        FineTuningJobEventObject::new("ftevent_1", "info", "job queued"),
    ]))
}

pub fn list_fine_tuning_job_checkpoints(
    _tenant_id: &str,
    _project_id: &str,
    job_id: &str,
) -> Result<ListFineTuningJobCheckpointsResponse> {
    ensure_local_fine_tuning_job_exists(job_id)?;
    Ok(ListFineTuningJobCheckpointsResponse::new(vec![
        FineTuningJobCheckpointObject::new("ftckpt_1", "ft:gpt-4.1-mini:checkpoint-1"),
    ]))
}

pub fn pause_fine_tuning_job(
    _tenant_id: &str,
    _project_id: &str,
    job_id: &str,
) -> Result<FineTuningJobObject> {
    ensure_local_fine_tuning_job_exists(job_id)?;
    Ok(FineTuningJobObject::paused(job_id, "gpt-4.1-mini"))
}

pub fn resume_fine_tuning_job(
    _tenant_id: &str,
    _project_id: &str,
    job_id: &str,
) -> Result<FineTuningJobObject> {
    ensure_local_fine_tuning_job_exists(job_id)?;
    Ok(FineTuningJobObject::running(job_id, "gpt-4.1-mini"))
}

pub fn create_fine_tuning_checkpoint_permissions(
    _tenant_id: &str,
    _project_id: &str,
    checkpoint_id: &str,
    request: &CreateFineTuningCheckpointPermissionsRequest,
) -> Result<ListFineTuningCheckpointPermissionsResponse> {
    ensure_local_fine_tuning_checkpoint_exists(checkpoint_id)?;
    let project_id = request
        .project_ids
        .first()
        .cloned()
        .unwrap_or_else(|| "project-2".to_owned());
    Ok(ListFineTuningCheckpointPermissionsResponse::new(vec![
        FineTuningCheckpointPermissionObject::new("perm_1", project_id),
    ]))
}

pub fn list_fine_tuning_checkpoint_permissions(
    _tenant_id: &str,
    _project_id: &str,
    checkpoint_id: &str,
) -> Result<ListFineTuningCheckpointPermissionsResponse> {
    ensure_local_fine_tuning_checkpoint_exists(checkpoint_id)?;
    Ok(ListFineTuningCheckpointPermissionsResponse::new(vec![
        FineTuningCheckpointPermissionObject::new("perm_1", "project-2"),
    ]))
}

pub fn delete_fine_tuning_checkpoint_permission(
    _tenant_id: &str,
    _project_id: &str,
    checkpoint_id: &str,
    permission_id: &str,
) -> Result<DeleteFineTuningCheckpointPermissionResponse> {
    ensure_local_fine_tuning_checkpoint_permission_exists(checkpoint_id, permission_id)?;
    Ok(DeleteFineTuningCheckpointPermissionResponse::deleted(
        permission_id,
    ))
}
