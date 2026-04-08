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
    Ok(ThreadObject::new("thread_1"))
}

fn ensure_local_thread_exists(thread_id: &str) -> Result<()> {
    if thread_id != "thread_1" {
        bail!("thread not found");
    }

    Ok(())
}

fn ensure_local_thread_message_exists(thread_id: &str, message_id: &str) -> Result<()> {
    ensure_local_thread_exists(thread_id)?;
    if message_id != "msg_1" {
        bail!("thread message not found");
    }

    Ok(())
}

fn ensure_local_thread_run_exists(thread_id: &str, run_id: &str) -> Result<()> {
    ensure_local_thread_exists(thread_id)?;
    if run_id != "run_1" {
        bail!("run not found");
    }

    Ok(())
}

pub fn get_thread(_tenant_id: &str, _project_id: &str, thread_id: &str) -> Result<ThreadObject> {
    ensure_local_thread_exists(thread_id)?;
    Ok(ThreadObject::new(thread_id))
}

pub fn update_thread(_tenant_id: &str, _project_id: &str, thread_id: &str) -> Result<ThreadObject> {
    ensure_local_thread_exists(thread_id)?;
    Ok(ThreadObject::new(thread_id))
}

pub fn delete_thread(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
) -> Result<DeleteThreadResponse> {
    ensure_local_thread_exists(thread_id)?;
    Ok(DeleteThreadResponse::deleted(thread_id))
}

pub fn create_thread_message(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    role: &str,
    text: &str,
) -> Result<ThreadMessageObject> {
    ensure_local_thread_exists(thread_id)?;
    Ok(ThreadMessageObject::text("msg_1", thread_id, role, text))
}

pub fn list_thread_messages(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
) -> Result<ListThreadMessagesResponse> {
    ensure_local_thread_exists(thread_id)?;
    Ok(ListThreadMessagesResponse::new(vec![
        ThreadMessageObject::text("msg_1", thread_id, "assistant", "hello"),
    ]))
}

pub fn get_thread_message(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    message_id: &str,
) -> Result<ThreadMessageObject> {
    ensure_local_thread_message_exists(thread_id, message_id)?;
    Ok(ThreadMessageObject::text(
        message_id,
        thread_id,
        "assistant",
        "hello",
    ))
}

pub fn update_thread_message(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    message_id: &str,
) -> Result<ThreadMessageObject> {
    ensure_local_thread_message_exists(thread_id, message_id)?;
    Ok(ThreadMessageObject::text(
        message_id,
        thread_id,
        "assistant",
        "hello",
    ))
}

pub fn delete_thread_message(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    message_id: &str,
) -> Result<DeleteThreadMessageResponse> {
    ensure_local_thread_message_exists(thread_id, message_id)?;
    Ok(DeleteThreadMessageResponse::deleted(message_id))
}

pub fn create_thread_run(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    assistant_id: &str,
    model: Option<&str>,
) -> Result<RunObject> {
    ensure_local_thread_exists(thread_id)?;
    Ok(RunObject::queued(
        "run_1",
        thread_id,
        assistant_id,
        model.unwrap_or("gpt-4.1"),
    ))
}

pub fn create_thread_and_run(
    _tenant_id: &str,
    _project_id: &str,
    assistant_id: &str,
) -> Result<RunObject> {
    if assistant_id.trim().is_empty() {
        bail!("Thread and run assistant_id is required.");
    }

    Ok(RunObject::queued(
        "run_1",
        "thread_1",
        assistant_id,
        "gpt-4.1",
    ))
}

pub fn list_thread_runs(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
) -> Result<ListRunsResponse> {
    ensure_local_thread_exists(thread_id)?;
    Ok(ListRunsResponse::new(vec![RunObject::queued(
        "run_1", thread_id, "asst_1", "gpt-4.1",
    )]))
}

pub fn get_thread_run(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> Result<RunObject> {
    ensure_local_thread_run_exists(thread_id, run_id)?;
    Ok(RunObject::in_progress(
        run_id, thread_id, "asst_1", "gpt-4.1",
    ))
}

pub fn update_thread_run(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> Result<RunObject> {
    ensure_local_thread_run_exists(thread_id, run_id)?;
    Ok(RunObject::with_metadata(
        run_id,
        thread_id,
        "asst_1",
        "gpt-4.1",
        "in_progress",
        serde_json::json!({"priority":"high"}),
    ))
}

pub fn cancel_thread_run(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> Result<RunObject> {
    ensure_local_thread_run_exists(thread_id, run_id)?;
    Ok(RunObject::cancelled(run_id, thread_id, "asst_1", "gpt-4.1"))
}

pub fn submit_thread_run_tool_outputs(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
    _tool_outputs: Vec<(&str, &str)>,
) -> Result<RunObject> {
    ensure_local_thread_run_exists(thread_id, run_id)?;
    Ok(RunObject::queued(run_id, thread_id, "asst_1", "gpt-4.1"))
}

pub fn list_thread_run_steps(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
) -> Result<ListRunStepsResponse> {
    ensure_local_thread_run_exists(thread_id, run_id)?;
    Ok(ListRunStepsResponse::new(vec![
        RunStepObject::message_creation("step_1", thread_id, run_id, "asst_1", "msg_1"),
    ]))
}

pub fn get_thread_run_step(
    _tenant_id: &str,
    _project_id: &str,
    thread_id: &str,
    run_id: &str,
    step_id: &str,
) -> Result<RunStepObject> {
    ensure_local_thread_run_exists(thread_id, run_id)?;
    if step_id != "step_1" {
        bail!("run step not found");
    }
    Ok(RunStepObject::message_creation(
        step_id, thread_id, run_id, "asst_1", "msg_1",
    ))
}
