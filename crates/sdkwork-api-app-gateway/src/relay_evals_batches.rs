use super::*;

pub async fn relay_eval_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateEvalRequest,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "evals", &request.name).await?;
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
        ProviderRequest::Evals(request),
    )
    .await
}

pub async fn relay_list_evals_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "evals", "evals").await?;
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
        ProviderRequest::EvalsList,
    )
    .await
}

pub async fn relay_get_eval_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    eval_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "evals", eval_id).await?;
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
        ProviderRequest::EvalsRetrieve(eval_id),
    )
    .await
}

pub async fn relay_update_eval_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    eval_id: &str,
    request: &UpdateEvalRequest,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "evals", eval_id).await?;
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
        ProviderRequest::EvalsUpdate(eval_id, request),
    )
    .await
}

pub async fn relay_delete_eval_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    eval_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "evals", eval_id).await?;
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
        ProviderRequest::EvalsDelete(eval_id),
    )
    .await
}

pub async fn relay_list_eval_runs_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    eval_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "evals", eval_id).await?;
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
        ProviderRequest::EvalRunsList(eval_id),
    )
    .await
}

pub async fn relay_eval_run_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    eval_id: &str,
    request: &CreateEvalRunRequest,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "evals", eval_id).await?;
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
        ProviderRequest::EvalRuns(eval_id, request),
    )
    .await
}

pub async fn relay_get_eval_run_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    eval_id: &str,
    run_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "evals", eval_id).await?;
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
        ProviderRequest::EvalRunsRetrieve(eval_id, run_id),
    )
    .await
}

pub async fn relay_delete_eval_run_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    eval_id: &str,
    run_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(project_id), "evals", eval_id).await?;
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
        ProviderRequest::EvalRunsDelete(eval_id, run_id),
    )
    .await
}

pub async fn relay_cancel_eval_run_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    eval_id: &str,
    run_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(project_id), "evals", eval_id).await?;
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
        ProviderRequest::EvalRunsCancel(eval_id, run_id),
    )
    .await
}

pub async fn relay_list_eval_run_output_items_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    eval_id: &str,
    run_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(project_id), "evals", eval_id).await?;
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
        ProviderRequest::EvalRunOutputItemsList(eval_id, run_id),
    )
    .await
}

pub async fn relay_get_eval_run_output_item_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    eval_id: &str,
    run_id: &str,
    output_item_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(project_id), "evals", eval_id).await?;
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
        ProviderRequest::EvalRunOutputItemsRetrieve(eval_id, run_id, output_item_id),
    )
    .await
}

pub async fn relay_batch_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateBatchRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "batches",
        &request.endpoint,
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
        ProviderRequest::Batches(request),
    )
    .await
}

pub async fn relay_list_batches_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "batches", "batches").await?;
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
        ProviderRequest::BatchesList,
    )
    .await
}

pub async fn relay_get_batch_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    batch_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "batches", batch_id).await?;
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
        ProviderRequest::BatchesRetrieve(batch_id),
    )
    .await
}

pub async fn relay_cancel_batch_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    batch_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "batches", batch_id).await?;
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
        ProviderRequest::BatchesCancel(batch_id),
    )
    .await
}

pub fn create_eval(
    _tenant_id: &str,
    _project_id: &str,
    request: &CreateEvalRequest,
) -> Result<EvalObject> {
    if request.name.trim().is_empty() {
        bail!("Eval name is required.");
    }
    if request.data_source_config.r#type != "file" {
        bail!("Only file-based local eval fallback is supported.");
    }
    if !local_object_id_matches(&request.data_source_config.file_id, "file") {
        bail!("A local file id is required for local eval fallback.");
    }

    bail!("Local eval fallback is not supported without an upstream provider.")
}

pub fn list_evals(_tenant_id: &str, _project_id: &str) -> Result<ListEvalsResponse> {
    bail!("Local eval listing fallback is not supported without an upstream provider.")
}

fn ensure_local_eval_exists(eval_id: &str) -> Result<()> {
    if !local_object_id_matches(eval_id, "eval") {
        bail!("eval not found");
    }

    Ok(())
}

fn ensure_local_eval_run_reference(eval_id: &str, run_id: &str) -> Result<()> {
    ensure_local_eval_exists(eval_id)?;
    if !local_object_id_matches(run_id, "run") {
        bail!("eval run not found");
    }

    Ok(())
}

fn ensure_local_eval_run_output_item_reference(
    eval_id: &str,
    run_id: &str,
    output_item_id: &str,
) -> Result<()> {
    ensure_local_eval_run_reference(eval_id, run_id)?;
    if !local_object_id_matches(output_item_id, "output_item") {
        bail!("eval run output item not found");
    }

    Ok(())
}

pub fn get_eval(_tenant_id: &str, _project_id: &str, eval_id: &str) -> Result<EvalObject> {
    ensure_local_eval_exists(eval_id)?;
    bail!("eval not found")
}

pub fn update_eval(
    _tenant_id: &str,
    _project_id: &str,
    eval_id: &str,
    request: &UpdateEvalRequest,
) -> Result<EvalObject> {
    ensure_local_eval_exists(eval_id)?;
    let Some(name) = request
        .name
        .as_deref()
        .filter(|name| !name.trim().is_empty())
    else {
        bail!("Eval name is required.");
    };

    let _ = name;
    bail!("eval not found")
}

pub fn delete_eval(
    _tenant_id: &str,
    _project_id: &str,
    eval_id: &str,
) -> Result<DeleteEvalResponse> {
    ensure_local_eval_exists(eval_id)?;
    bail!("eval not found")
}

pub fn list_eval_runs(
    _tenant_id: &str,
    _project_id: &str,
    eval_id: &str,
) -> Result<ListEvalRunsResponse> {
    ensure_local_eval_exists(eval_id)?;
    bail!("Persisted local eval run state is required for local run listing.")
}

pub fn create_eval_run(
    _tenant_id: &str,
    _project_id: &str,
    eval_id: &str,
    request: &CreateEvalRunRequest,
) -> Result<EvalRunObject> {
    ensure_local_eval_exists(eval_id)?;
    if request.name.trim().is_empty() {
        bail!("Eval run name is required.");
    }

    bail!("Persisted local eval run state is required for local run creation.")
}

pub fn get_eval_run(
    _tenant_id: &str,
    _project_id: &str,
    eval_id: &str,
    run_id: &str,
) -> Result<EvalRunObject> {
    ensure_local_eval_run_reference(eval_id, run_id)?;
    bail!("eval run not found")
}

pub fn delete_eval_run(
    _tenant_id: &str,
    _project_id: &str,
    eval_id: &str,
    run_id: &str,
) -> Result<DeleteEvalRunResponse> {
    ensure_local_eval_run_reference(eval_id, run_id)?;
    bail!("eval run not found")
}

pub fn cancel_eval_run(
    _tenant_id: &str,
    _project_id: &str,
    eval_id: &str,
    run_id: &str,
) -> Result<EvalRunObject> {
    ensure_local_eval_run_reference(eval_id, run_id)?;
    bail!("eval run not found")
}

pub fn list_eval_run_output_items(
    _tenant_id: &str,
    _project_id: &str,
    eval_id: &str,
    run_id: &str,
) -> Result<ListEvalRunOutputItemsResponse> {
    ensure_local_eval_run_reference(eval_id, run_id)?;
    bail!("Persisted local eval run output item state is required for local output item listing.")
}

pub fn get_eval_run_output_item(
    _tenant_id: &str,
    _project_id: &str,
    eval_id: &str,
    run_id: &str,
    output_item_id: &str,
) -> Result<EvalRunOutputItemObject> {
    ensure_local_eval_run_output_item_reference(eval_id, run_id, output_item_id)?;
    bail!("eval run output item not found")
}

pub fn create_batch(
    _tenant_id: &str,
    _project_id: &str,
    request: &CreateBatchRequest,
) -> Result<BatchObject> {
    if request.endpoint.trim().is_empty() {
        bail!("Batch endpoint is required.");
    }
    if request.completion_window.trim().is_empty() {
        bail!("Batch completion window is required.");
    }
    if !local_object_id_matches(&request.input_file_id, "file") {
        bail!("A local file id is required for local batch fallback.");
    }

    bail!("Local batch fallback is not supported without an upstream provider.")
}

pub fn list_batches(_tenant_id: &str, _project_id: &str) -> Result<ListBatchesResponse> {
    bail!("Local batch listing fallback is not supported without an upstream provider.")
}

fn ensure_local_batch_exists(batch_id: &str) -> Result<()> {
    if !local_object_id_matches(batch_id, "batch") {
        bail!("batch not found");
    }

    Ok(())
}

pub fn get_batch(_tenant_id: &str, _project_id: &str, batch_id: &str) -> Result<BatchObject> {
    ensure_local_batch_exists(batch_id)?;
    bail!("batch not found")
}

pub fn cancel_batch(_tenant_id: &str, _project_id: &str, batch_id: &str) -> Result<BatchObject> {
    ensure_local_batch_exists(batch_id)?;
    bail!("batch not found")
}
