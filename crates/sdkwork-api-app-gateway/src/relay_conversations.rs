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
    Ok(ConversationObject::new("conv_1"))
}

pub fn list_conversations(
    _tenant_id: &str,
    _project_id: &str,
) -> Result<ListConversationsResponse> {
    Ok(ListConversationsResponse::new(vec![
        ConversationObject::new("conv_1"),
    ]))
}

fn ensure_local_conversation_exists(conversation_id: &str) -> Result<()> {
    if conversation_id != "conv_1" {
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
    Ok(ConversationObject::new(conversation_id))
}

pub fn update_conversation(
    _tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
    metadata: Value,
) -> Result<ConversationObject> {
    ensure_local_conversation_exists(conversation_id)?;
    Ok(ConversationObject::with_metadata(conversation_id, metadata))
}

pub fn delete_conversation(
    _tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
) -> Result<DeleteConversationResponse> {
    ensure_local_conversation_exists(conversation_id)?;
    Ok(DeleteConversationResponse::deleted(conversation_id))
}

pub fn create_conversation_items(
    _tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
) -> Result<ListConversationItemsResponse> {
    ensure_local_conversation_exists(conversation_id)?;
    Ok(ListConversationItemsResponse::new(vec![
        ConversationItemObject::message("item_1", "assistant", "hello"),
    ]))
}

pub fn list_conversation_items(
    _tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
) -> Result<ListConversationItemsResponse> {
    ensure_local_conversation_exists(conversation_id)?;
    Ok(ListConversationItemsResponse::new(vec![
        ConversationItemObject::message("item_1", "assistant", "hello"),
    ]))
}

fn ensure_local_conversation_item_exists(conversation_id: &str, item_id: &str) -> Result<()> {
    ensure_local_conversation_exists(conversation_id)?;
    if item_id != "item_1" {
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
    Ok(ConversationItemObject::message(
        item_id,
        "assistant",
        "hello",
    ))
}

pub fn delete_conversation_item(
    _tenant_id: &str,
    _project_id: &str,
    conversation_id: &str,
    item_id: &str,
) -> Result<DeleteConversationItemResponse> {
    ensure_local_conversation_item_exists(conversation_id, item_id)?;
    Ok(DeleteConversationItemResponse::deleted(item_id))
}
