use super::*;

pub async fn relay_conversation_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    request: &CreateConversationRequest,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "responses",
        "conversations",
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::Conversations(request),
    )
    .await
}

pub async fn relay_list_conversations_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "responses",
        "conversations",
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ConversationsList,
    )
    .await
}

pub async fn relay_get_conversation_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "responses",
        conversation_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ConversationsRetrieve(conversation_id),
    )
    .await
}

pub async fn relay_update_conversation_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
    request: &UpdateConversationRequest,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "responses",
        conversation_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ConversationsUpdate(conversation_id, request),
    )
    .await
}

pub async fn relay_delete_conversation_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "responses",
        conversation_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ConversationsDelete(conversation_id),
    )
    .await
}

pub async fn relay_conversation_items_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
    request: &CreateConversationItemsRequest,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "responses",
        conversation_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ConversationItems(conversation_id, request),
    )
    .await
}

pub async fn relay_list_conversation_items_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "responses",
        conversation_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ConversationItemsList(conversation_id),
    )
    .await
}

pub async fn relay_get_conversation_item_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
    item_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "responses",
        conversation_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ConversationItemsRetrieve(conversation_id, item_id),
    )
    .await
}

pub async fn relay_delete_conversation_item_from_store(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
    item_id: &str,
) -> Result<Option<Value>> {
    let Some((adapter_kind, base_url, api_key)) = resolve_non_model_provider(
        store,
        secret_manager,
        tenant_id,
        _project_id,
        "responses",
        conversation_id,
    )
    .await?
    else {
        return Ok(None);
    };

    execute_json_provider_request(
        &adapter_kind,
        base_url,
        &api_key,
        ProviderRequest::ConversationItemsDelete(conversation_id, item_id),
    )
    .await
}

pub fn create_conversation(_tenant_id: &str, _project_id: &str) -> Result<ConversationObject> {
    bail!("Local conversation fallback is not supported without an upstream provider.")
}

pub fn list_conversations(
    _tenant_id: &str,
    _project_id: &str,
) -> Result<ListConversationsResponse> {
    bail!("Local conversation listing fallback is not supported without an upstream provider.")
}

fn ensure_local_conversation_exists(conversation_id: &str) -> Result<()> {
    if !local_object_id_matches(conversation_id, "conv") {
        bail!("conversation not found");
    }

    Ok(())
}

pub fn get_conversation(
    _tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
) -> Result<ConversationObject> {
    ensure_local_conversation_exists(conversation_id)?;
    bail!("conversation not found")
}

pub fn update_conversation(
    _tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
    metadata: Option<Value>,
) -> Result<ConversationObject> {
    ensure_local_conversation_exists(conversation_id)?;
    let Some(metadata) = metadata else {
        bail!("Conversation metadata is required for local fallback updates.");
    };

    let _ = metadata;
    bail!("conversation not found")
}

pub fn delete_conversation(
    _tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
) -> Result<DeleteConversationResponse> {
    ensure_local_conversation_exists(conversation_id)?;
    bail!("conversation not found")
}

pub fn create_conversation_items(
    _tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
) -> Result<ListConversationItemsResponse> {
    ensure_local_conversation_exists(conversation_id)?;
    bail!("Persisted local conversation item state is required for local item creation.")
}

pub fn list_conversation_items(
    _tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
) -> Result<ListConversationItemsResponse> {
    ensure_local_conversation_exists(conversation_id)?;
    bail!("Persisted local conversation item state is required for local item listing.")
}

fn ensure_local_conversation_item_exists(conversation_id: &str, item_id: &str) -> Result<()> {
    ensure_local_conversation_exists(conversation_id)?;
    if !local_object_id_matches(item_id, "item") {
        bail!("conversation item not found");
    }

    Ok(())
}

pub fn get_conversation_item(
    _tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
    item_id: &str,
) -> Result<ConversationItemObject> {
    ensure_local_conversation_item_exists(conversation_id, item_id)?;
    bail!("conversation item not found")
}

pub fn delete_conversation_item(
    _tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
    item_id: &str,
) -> Result<DeleteConversationItemResponse> {
    ensure_local_conversation_item_exists(conversation_id, item_id)?;
    bail!("conversation item not found")
}
