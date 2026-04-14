use super::*;

pub async fn relay_thread_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateThreadRequest,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        "threads",
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::Threads(request),
    )
    .await
}

pub async fn relay_get_thread_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadsRetrieve(thread_id),
    )
    .await
}

pub async fn relay_update_thread_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    request: &UpdateThreadRequest,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadsUpdate(thread_id, request),
    )
    .await
}

pub async fn relay_delete_thread_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadsDelete(thread_id),
    )
    .await
}

pub async fn relay_thread_messages_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    request: &CreateThreadMessageRequest,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadMessages(thread_id, request),
    )
    .await
}

pub async fn relay_list_thread_messages_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadMessagesList(thread_id),
    )
    .await
}

pub async fn relay_get_thread_message_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    message_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadMessagesRetrieve(thread_id, message_id),
    )
    .await
}

pub async fn relay_update_thread_message_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    message_id: &str,
    request: &UpdateThreadMessageRequest,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadMessagesUpdate(thread_id, message_id, request),
    )
    .await
}

pub async fn relay_delete_thread_message_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    message_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadMessagesDelete(thread_id, message_id),
    )
    .await
}

pub async fn relay_thread_run_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    request: &CreateRunRequest,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadRuns(thread_id, request),
    )
    .await
}

pub async fn relay_list_thread_runs_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadRunsList(thread_id),
    )
    .await
}

pub async fn relay_get_thread_run_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadRunsRetrieve(thread_id, run_id),
    )
    .await
}

pub async fn relay_update_thread_run_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
    request: &UpdateRunRequest,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadRunsUpdate(thread_id, run_id, request),
    )
    .await
}

pub async fn relay_cancel_thread_run_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadRunsCancel(thread_id, run_id),
    )
    .await
}

pub async fn relay_submit_thread_run_tool_outputs_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
    request: &SubmitToolOutputsRunRequest,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadRunsSubmitToolOutputs(thread_id, run_id, request),
    )
    .await
}

pub async fn relay_list_thread_run_steps_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadRunStepsList(thread_id, run_id),
    )
    .await
}

pub async fn relay_get_thread_run_step_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
    step_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        thread_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadRunStepsRetrieve(thread_id, run_id, step_id),
    )
    .await
}

pub async fn relay_thread_and_run_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateThreadAndRunRequest,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "assistants",
        "threads/runs",
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ThreadsRuns(request),
    )
    .await
}

pub fn create_thread(_tenant_id: &str, _project_id: &str) -> Result<ThreadObject> {
    bail!("Local thread fallback is not supported without an upstream provider.")
}

fn ensure_local_thread_exists(thread_id: &str) -> Result<()> {
    if !local_object_id_matches(thread_id, "thread") {
        bail!("thread not found");
    }

    Ok(())
}

fn ensure_local_thread_message_exists(thread_id: &str, message_id: &str) -> Result<()> {
    ensure_local_thread_exists(thread_id)?;
    if !local_object_id_matches(message_id, "msg") {
        bail!("thread message not found");
    }

    Ok(())
}

fn ensure_local_thread_run_exists(thread_id: &str, run_id: &str) -> Result<()> {
    ensure_local_thread_exists(thread_id)?;
    if !local_object_id_matches(run_id, "run") {
        bail!("run not found");
    }

    Ok(())
}

pub fn get_thread(_tenant_id: &str, _project_id: &str, thread_id: &str) -> Result<ThreadObject> {
    ensure_local_thread_exists(thread_id)?;
    bail!("thread not found")
}

pub fn update_thread(_tenant_id: &str, _project_id: &str, thread_id: &str) -> Result<ThreadObject> {
    ensure_local_thread_exists(thread_id)?;
    bail!("thread not found")
}

pub fn delete_thread(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
) -> Result<DeleteThreadResponse> {
    ensure_local_thread_exists(thread_id)?;
    bail!("thread not found")
}

pub fn create_thread_message(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    role: &str,
    text: &str,
) -> Result<ThreadMessageObject> {
    ensure_local_thread_exists(thread_id)?;
    if role.trim().is_empty() {
        bail!("Thread message role is required.");
    }
    if text.trim().is_empty() {
        bail!("Thread message text is required.");
    }

    bail!("Persisted local thread message state is required for local message creation.")
}

pub fn list_thread_messages(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
) -> Result<ListThreadMessagesResponse> {
    ensure_local_thread_exists(thread_id)?;
    bail!("Persisted local thread message state is required for local message listing.")
}

pub fn get_thread_message(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    message_id: &str,
) -> Result<ThreadMessageObject> {
    ensure_local_thread_message_exists(thread_id, message_id)?;
    bail!("thread message not found")
}

pub fn update_thread_message(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    message_id: &str,
) -> Result<ThreadMessageObject> {
    ensure_local_thread_message_exists(thread_id, message_id)?;
    bail!("thread message not found")
}

pub fn delete_thread_message(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    message_id: &str,
) -> Result<DeleteThreadMessageResponse> {
    ensure_local_thread_message_exists(thread_id, message_id)?;
    bail!("thread message not found")
}

pub fn create_thread_run(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    assistant_id: &str,
    model: Option<&str>,
) -> Result<RunObject> {
    ensure_local_thread_exists(thread_id)?;
    if assistant_id.trim().is_empty() {
        bail!("Thread run assistant_id is required.");
    }
    let Some(model) = model.filter(|value| !value.trim().is_empty()) else {
        bail!("Thread run model is required for local fallback.");
    };

    let _ = (assistant_id, model);
    bail!("Persisted local thread run state is required for local run creation.")
}

pub fn create_thread_and_run(
    _tenant_id: &str,
    _project_id: &str,
    assistant_id: &str,
    model: Option<&str>,
) -> Result<RunObject> {
    if assistant_id.trim().is_empty() {
        bail!("Thread and run assistant_id is required.");
    }
    let Some(model) = model.filter(|value| !value.trim().is_empty()) else {
        bail!("Thread and run model is required for local fallback.");
    };

    let _ = (assistant_id, model);
    bail!("Local thread and run fallback is not supported without an upstream provider.")
}

pub fn list_thread_runs(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
) -> Result<ListRunsResponse> {
    ensure_local_thread_exists(thread_id)?;
    bail!("Persisted local thread run state is required for local run listing.")
}

pub fn get_thread_run(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> Result<RunObject> {
    ensure_local_thread_run_exists(thread_id, run_id)?;
    bail!("run not found")
}

pub fn update_thread_run(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> Result<RunObject> {
    ensure_local_thread_run_exists(thread_id, run_id)?;
    bail!("run not found")
}

pub fn cancel_thread_run(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> Result<RunObject> {
    ensure_local_thread_run_exists(thread_id, run_id)?;
    bail!("run not found")
}

pub fn submit_thread_run_tool_outputs(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
    _tool_outputs: Vec<(&str, &str)>,
) -> Result<RunObject> {
    ensure_local_thread_run_exists(thread_id, run_id)?;
    bail!("run not found")
}

pub fn list_thread_run_steps(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> Result<ListRunStepsResponse> {
    ensure_local_thread_run_exists(thread_id, run_id)?;
    bail!("Persisted local thread run step state is required for local run step listing.")
}

pub fn get_thread_run_step(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
    step_id: &str,
) -> Result<RunStepObject> {
    ensure_local_thread_run_exists(thread_id, run_id)?;
    let _ = step_id;
    bail!("run step not found")
}
