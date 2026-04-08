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

pub fn create_eval(_tenant_id: &str, _project_id: &str, name: &str) -> Result<EvalObject> {
    Ok(EvalObject::new("eval_1", name))
}

pub fn list_evals(_tenant_id: &str, _project_id: &str) -> Result<ListEvalsResponse> {
    Ok(ListEvalsResponse::new(vec![EvalObject::new(
        "eval_1",
        "qa-benchmark",
    )]))
}

fn ensure_local_eval_exists(eval_id: &str) -> Result<()> {
    if eval_id != "eval_1" {
        bail!("eval not found");
    }

    Ok(())
}

fn ensure_local_eval_run_exists(eval_id: &str, run_id: &str) -> Result<()> {
    ensure_local_eval_exists(eval_id)?;
    if run_id != "run_1" {
        bail!("eval run not found");
    }

    Ok(())
}

fn ensure_local_eval_run_output_item_exists(
    eval_id: &str,
    run_id: &str,
    output_item_id: &str,
) -> Result<()> {
    ensure_local_eval_run_exists(eval_id, run_id)?;
    if output_item_id != "output_item_1" {
        bail!("eval run output item not found");
    }

    Ok(())
}

pub fn get_eval(_tenant_id: &str, _project_id: &str, eval_id: &str) -> Result<EvalObject> {
    ensure_local_eval_exists(eval_id)?;
    Ok(EvalObject::new(eval_id, "qa-benchmark"))
}

pub fn update_eval(
    _tenant_id: &str,
    _project_id: &str,
    eval_id: &str,
    request: &UpdateEvalRequest,
) -> Result<EvalObject> {
    ensure_local_eval_exists(eval_id)?;
    Ok(EvalObject::new(
        eval_id,
        request.name.as_deref().unwrap_or("qa-benchmark"),
    ))
}

pub fn delete_eval(
    _tenant_id: &str,
    _project_id: &str,
    eval_id: &str,
) -> Result<DeleteEvalResponse> {
    ensure_local_eval_exists(eval_id)?;
    Ok(DeleteEvalResponse::deleted(eval_id))
}

pub fn list_eval_runs(
    _tenant_id: &str,
    _project_id: &str,
    eval_id: &str,
) -> Result<ListEvalRunsResponse> {
    ensure_local_eval_exists(eval_id)?;
    Ok(ListEvalRunsResponse::new(vec![EvalRunObject::completed(
        "run_1",
    )]))
}

pub fn create_eval_run(
    _tenant_id: &str,
    _project_id: &str,
    eval_id: &str,
    _request: &CreateEvalRunRequest,
) -> Result<EvalRunObject> {
    ensure_local_eval_exists(eval_id)?;
    Ok(EvalRunObject::queued("run_1"))
}

pub fn get_eval_run(
    _tenant_id: &str,
    _project_id: &str,
    eval_id: &str,
    run_id: &str,
) -> Result<EvalRunObject> {
    ensure_local_eval_run_exists(eval_id, run_id)?;
    Ok(EvalRunObject::completed(run_id))
}

pub fn delete_eval_run(
    _tenant_id: &str,
    _project_id: &str,
    eval_id: &str,
    run_id: &str,
) -> Result<DeleteEvalRunResponse> {
    ensure_local_eval_run_exists(eval_id, run_id)?;
    Ok(DeleteEvalRunResponse::deleted(run_id))
}

pub fn cancel_eval_run(
    _tenant_id: &str,
    _project_id: &str,
    eval_id: &str,
    run_id: &str,
) -> Result<EvalRunObject> {
    ensure_local_eval_run_exists(eval_id, run_id)?;
    Ok(EvalRunObject {
        id: run_id.to_owned(),
        object: "eval.run",
        status: "cancelled",
    })
}

pub fn list_eval_run_output_items(
    _tenant_id: &str,
    _project_id: &str,
    eval_id: &str,
    run_id: &str,
) -> Result<ListEvalRunOutputItemsResponse> {
    ensure_local_eval_run_exists(eval_id, run_id)?;
    Ok(ListEvalRunOutputItemsResponse::new(vec![
        EvalRunOutputItemObject::passed("output_item_1"),
    ]))
}

pub fn get_eval_run_output_item(
    _tenant_id: &str,
    _project_id: &str,
    eval_id: &str,
    run_id: &str,
    output_item_id: &str,
) -> Result<EvalRunOutputItemObject> {
    ensure_local_eval_run_output_item_exists(eval_id, run_id, output_item_id)?;
    Ok(EvalRunOutputItemObject::passed(output_item_id))
}

pub fn create_batch(
    _tenant_id: &str,
    _project_id: &str,
    endpoint: &str,
    input_file_id: &str,
) -> Result<BatchObject> {
    Ok(BatchObject::new("batch_1", endpoint, input_file_id))
}

pub fn list_batches(_tenant_id: &str, _project_id: &str) -> Result<ListBatchesResponse> {
    Ok(ListBatchesResponse::new(vec![BatchObject::new(
        "batch_1",
        "/v1/responses",
        "file_1",
    )]))
}

fn ensure_local_batch_exists(batch_id: &str) -> Result<()> {
    if batch_id != "batch_1" {
        bail!("batch not found");
    }

    Ok(())
}

pub fn get_batch(_tenant_id: &str, _project_id: &str, batch_id: &str) -> Result<BatchObject> {
    ensure_local_batch_exists(batch_id)?;
    Ok(BatchObject::new(batch_id, "/v1/responses", "file_1"))
}

pub fn cancel_batch(_tenant_id: &str, _project_id: &str, batch_id: &str) -> Result<BatchObject> {
    ensure_local_batch_exists(batch_id)?;
    Ok(BatchObject::cancelled(batch_id, "/v1/responses", "file_1"))
}
