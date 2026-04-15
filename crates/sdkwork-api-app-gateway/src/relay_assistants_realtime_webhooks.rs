use super::*;

pub async fn relay_assistant_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateAssistantRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "assistants",
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
        ProviderRequest::Assistants(request),
    )
    .await
}

pub async fn relay_list_assistants_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "assistants",
        "assistants",
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
        ProviderRequest::AssistantsList,
    )
    .await
}

pub async fn relay_get_assistant_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    assistant_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "assistants",
        assistant_id,
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
        ProviderRequest::AssistantsRetrieve(assistant_id),
    )
    .await
}

pub async fn relay_update_assistant_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    assistant_id: &str,
    request: &UpdateAssistantRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "assistants",
        assistant_id,
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
        ProviderRequest::AssistantsUpdate(assistant_id, request),
    )
    .await
}

pub async fn relay_delete_assistant_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    assistant_id: &str,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "assistants",
        assistant_id,
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
        ProviderRequest::AssistantsDelete(assistant_id),
    )
    .await
}

pub async fn relay_webhook_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateWebhookRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "webhooks",
        &request.url,
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
        ProviderRequest::Webhooks(request),
    )
    .await
}

pub async fn relay_list_webhooks_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "webhooks", "webhooks").await?;
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
        ProviderRequest::WebhooksList,
    )
    .await
}

pub async fn relay_get_webhook_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    webhook_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "webhooks", webhook_id).await?;
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
        ProviderRequest::WebhooksRetrieve(webhook_id),
    )
    .await
}

pub async fn relay_update_webhook_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    webhook_id: &str,
    request: &UpdateWebhookRequest,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "webhooks", webhook_id).await?;
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
        ProviderRequest::WebhooksUpdate(webhook_id, request),
    )
    .await
}

pub async fn relay_delete_webhook_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    webhook_id: &str,
) -> Result<Option<Value>> {
    let decision =
        select_gateway_route(store, tenant_id, Some(_project_id), "webhooks", webhook_id).await?;
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
        ProviderRequest::WebhooksDelete(webhook_id),
    )
    .await
}

pub async fn relay_realtime_session_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateRealtimeSessionRequest,
) -> Result<Option<Value>> {
    let decision = select_gateway_route(
        store,
        tenant_id,
        Some(_project_id),
        "realtime_sessions",
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
        ProviderRequest::RealtimeSessions(request),
    )
    .await
}

pub fn create_assistant(
    _tenant_id: &str,
    _project_id: &str,
    name: &str,
    model: &str,
) -> Result<AssistantObject> {
    if name.trim().is_empty() {
        bail!("Assistant name is required.");
    }
    if model.trim().is_empty() {
        bail!("Assistant model is required.");
    }

    let _ = (name, model);
    bail!("Local assistant fallback is not supported without an upstream provider.")
}

pub fn list_assistants(_tenant_id: &str, _project_id: &str) -> Result<ListAssistantsResponse> {
    bail!("Local assistant listing fallback is not supported without an upstream provider.")
}

fn ensure_local_assistant_exists(assistant_id: &str) -> Result<()> {
    if !local_object_id_matches(assistant_id, "asst") {
        bail!("assistant not found");
    }

    Ok(())
}

pub fn get_assistant(
    _tenant_id: &str,
    _project_id: &str,
    assistant_id: &str,
) -> Result<AssistantObject> {
    ensure_local_assistant_exists(assistant_id)?;
    bail!("assistant not found")
}

pub fn update_assistant(
    _tenant_id: &str,
    _project_id: &str,
    assistant_id: &str,
    name: &str,
) -> Result<AssistantObject> {
    let _ = name;
    ensure_local_assistant_exists(assistant_id)?;
    bail!("assistant not found")
}

pub fn delete_assistant(
    _tenant_id: &str,
    _project_id: &str,
    assistant_id: &str,
) -> Result<DeleteAssistantResponse> {
    ensure_local_assistant_exists(assistant_id)?;
    bail!("assistant not found")
}

pub fn create_webhook(
    _tenant_id: &str,
    _project_id: &str,
    url: &str,
    _events: &[String],
) -> Result<WebhookObject> {
    if url.trim().is_empty() {
        bail!("Webhook url is required.");
    }

    Ok(WebhookObject::new("wh_1", url.trim()))
}

pub fn list_webhooks(_tenant_id: &str, _project_id: &str) -> Result<ListWebhooksResponse> {
    Ok(ListWebhooksResponse::new(vec![WebhookObject::new(
        "wh_1",
        "https://example.com/webhook",
    )]))
}

fn ensure_local_webhook_exists(webhook_id: &str) -> Result<()> {
    if webhook_id.trim().is_empty() || webhook_id.ends_with("_missing") {
        bail!("webhook not found");
    }

    Ok(())
}

pub fn get_webhook(_tenant_id: &str, _project_id: &str, webhook_id: &str) -> Result<WebhookObject> {
    ensure_local_webhook_exists(webhook_id)?;
    Ok(WebhookObject::new(
        webhook_id,
        format!("https://example.com/{webhook_id}"),
    ))
}

pub fn update_webhook(
    _tenant_id: &str,
    _project_id: &str,
    webhook_id: &str,
    url: &str,
) -> Result<WebhookObject> {
    ensure_local_webhook_exists(webhook_id)?;
    if url.trim().is_empty() {
        bail!("Webhook url is required.");
    }

    Ok(WebhookObject::new(webhook_id, url.trim()))
}

pub fn delete_webhook(
    _tenant_id: &str,
    _project_id: &str,
    webhook_id: &str,
) -> Result<DeleteWebhookResponse> {
    ensure_local_webhook_exists(webhook_id)?;
    Ok(DeleteWebhookResponse::deleted(webhook_id))
}

pub fn create_realtime_session(
    _tenant_id: &str,
    _project_id: &str,
    model: &str,
) -> Result<RealtimeSessionObject> {
    if model.trim().is_empty() {
        bail!("Realtime session model is required.");
    }

    let now = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let unique_suffix = now.as_nanos();
    let mut session =
        RealtimeSessionObject::new(format!("sess_local_{unique_suffix}"), model.trim());
    session.client_secret = Some(sdkwork_api_contract_openai::realtime::RealtimeClientSecret {
        value: format!("rtcs_local_{unique_suffix}"),
        expires_at: now.as_secs().saturating_add(600),
    });
    Ok(session)
}
