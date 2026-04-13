use super::*;

async fn load_portal_user_record(
    store: &dyn AdminStore,
    user_id: &str,
) -> PortalResult<PortalUserRecord> {
    store
        .find_portal_user_by_id(user_id)
        .await
        .map_err(PortalIdentityError::from)?
        .ok_or_else(|| PortalIdentityError::NotFound("portal user not found".to_owned()))
}

async fn find_portal_scoped_api_key(
    store: &dyn AdminStore,
    user_id: &str,
    hashed_key: &str,
) -> PortalResult<Option<GatewayApiKeyRecord>> {
    let user = load_portal_user_record(store, user_id).await?;
    let Some(record) = store
        .find_gateway_api_key(hashed_key)
        .await
        .map_err(PortalIdentityError::from)?
    else {
        return Ok(None);
    };

    if record.tenant_id != user.workspace_tenant_id
        || record.project_id != user.workspace_project_id
    {
        return Ok(None);
    }

    Ok(Some(record))
}

async fn find_portal_scoped_api_key_group(
    store: &dyn AdminStore,
    user_id: &str,
    group_id: &str,
) -> PortalResult<Option<ApiKeyGroupRecord>> {
    let user = load_portal_user_record(store, user_id).await?;
    let Some(record) = store
        .find_api_key_group(group_id)
        .await
        .map_err(PortalIdentityError::from)?
    else {
        return Ok(None);
    };

    if record.tenant_id != user.workspace_tenant_id
        || record.project_id != user.workspace_project_id
    {
        return Ok(None);
    }

    Ok(Some(record))
}

pub async fn list_portal_api_keys(
    store: &dyn AdminStore,
    user_id: &str,
) -> PortalResult<Vec<GatewayApiKeyRecord>> {
    let user = load_portal_user_record(store, user_id).await?;

    let keys = store
        .list_gateway_api_keys()
        .await
        .map_err(PortalIdentityError::from)?;
    Ok(keys
        .into_iter()
        .filter(|key| {
            key.tenant_id == user.workspace_tenant_id && key.project_id == user.workspace_project_id
        })
        .collect())
}

pub async fn list_portal_api_key_groups(
    store: &dyn AdminStore,
    user_id: &str,
) -> PortalResult<Vec<ApiKeyGroupRecord>> {
    let user = load_portal_user_record(store, user_id).await?;
    let groups = store
        .list_api_key_groups()
        .await
        .map_err(PortalIdentityError::from)?;
    Ok(groups
        .into_iter()
        .filter(|group| {
            group.tenant_id == user.workspace_tenant_id
                && group.project_id == user.workspace_project_id
        })
        .collect())
}

pub async fn create_portal_api_key_group(
    store: &dyn AdminStore,
    user_id: &str,
    input: PortalApiKeyGroupInput,
) -> PortalResult<ApiKeyGroupRecord> {
    let user = load_portal_user_record(store, user_id).await?;
    create_api_key_group(
        store,
        ApiKeyGroupInput {
            tenant_id: user.workspace_tenant_id,
            project_id: user.workspace_project_id,
            environment: input.environment,
            name: input.name,
            slug: input.slug,
            description: input.description,
            color: input.color,
            default_capability_scope: input.default_capability_scope,
            default_routing_profile_id: input.default_routing_profile_id,
            default_accounting_mode: input.default_accounting_mode,
        },
    )
    .await
    .map_err(map_admin_identity_error_to_portal)
}

pub async fn update_portal_api_key_group(
    store: &dyn AdminStore,
    user_id: &str,
    group_id: &str,
    input: PortalApiKeyGroupInput,
) -> PortalResult<Option<ApiKeyGroupRecord>> {
    let Some(_existing) = find_portal_scoped_api_key_group(store, user_id, group_id).await? else {
        return Ok(None);
    };
    let user = load_portal_user_record(store, user_id).await?;
    update_api_key_group(
        store,
        group_id,
        ApiKeyGroupInput {
            tenant_id: user.workspace_tenant_id,
            project_id: user.workspace_project_id,
            environment: input.environment,
            name: input.name,
            slug: input.slug,
            description: input.description,
            color: input.color,
            default_capability_scope: input.default_capability_scope,
            default_routing_profile_id: input.default_routing_profile_id,
            default_accounting_mode: input.default_accounting_mode,
        },
    )
    .await
    .map_err(map_admin_identity_error_to_portal)
}

pub async fn set_portal_api_key_group_active(
    store: &dyn AdminStore,
    user_id: &str,
    group_id: &str,
    active: bool,
) -> PortalResult<Option<ApiKeyGroupRecord>> {
    let Some(_existing) = find_portal_scoped_api_key_group(store, user_id, group_id).await? else {
        return Ok(None);
    };
    set_api_key_group_active(store, group_id, active)
        .await
        .map_err(map_admin_identity_error_to_portal)
}

pub async fn delete_portal_api_key_group(
    store: &dyn AdminStore,
    user_id: &str,
    group_id: &str,
) -> PortalResult<bool> {
    let Some(_existing) = find_portal_scoped_api_key_group(store, user_id, group_id).await? else {
        return Ok(false);
    };
    delete_api_key_group(store, group_id)
        .await
        .map_err(map_admin_identity_error_to_portal)
}

pub async fn set_portal_api_key_active(
    store: &dyn AdminStore,
    user_id: &str,
    hashed_key: &str,
    active: bool,
) -> PortalResult<Option<GatewayApiKeyRecord>> {
    let Some(existing) = find_portal_scoped_api_key(store, user_id, hashed_key).await? else {
        return Ok(None);
    };

    let updated = GatewayApiKeyRecord { active, ..existing };
    let saved = store
        .insert_gateway_api_key(&updated)
        .await
        .map_err(PortalIdentityError::from)?;
    Ok(Some(saved))
}

pub async fn delete_portal_api_key(
    store: &dyn AdminStore,
    user_id: &str,
    hashed_key: &str,
) -> PortalResult<bool> {
    let Some(existing) = find_portal_scoped_api_key(store, user_id, hashed_key).await? else {
        return Ok(false);
    };

    store
        .delete_gateway_api_key(&existing.hashed_key)
        .await
        .map_err(PortalIdentityError::from)
}

pub async fn create_portal_api_key(
    store: &dyn AdminStore,
    user_id: &str,
    environment: &str,
) -> PortalResult<CreatedGatewayApiKey> {
    let default_label = default_gateway_api_key_label(environment);
    create_portal_api_key_with_metadata(
        store,
        CreatePortalApiKeyInput {
            user_id,
            environment,
            label: &default_label,
            expires_at_ms: None,
            plaintext_key: None,
            notes: None,
            api_key_group_id: None,
        },
    )
    .await
}

pub struct CreatePortalApiKeyInput<'a> {
    pub user_id: &'a str,
    pub environment: &'a str,
    pub label: &'a str,
    pub expires_at_ms: Option<u64>,
    pub plaintext_key: Option<&'a str>,
    pub notes: Option<&'a str>,
    pub api_key_group_id: Option<&'a str>,
}

pub async fn create_portal_api_key_with_metadata(
    store: &dyn AdminStore,
    input: CreatePortalApiKeyInput<'_>,
) -> PortalResult<CreatedGatewayApiKey> {
    let environment = input.environment.trim();
    if environment.is_empty() {
        return Err(PortalIdentityError::InvalidInput(
            "environment is required".to_owned(),
        ));
    }
    validate_gateway_api_key_metadata(input.label, input.notes, input.expires_at_ms)
        .map_err(PortalIdentityError::InvalidInput)?;
    let user = load_portal_user_record(store, input.user_id).await?;

    persist_gateway_api_key_with_metadata(
        store,
        PersistGatewayApiKeyInput {
            tenant_id: &user.workspace_tenant_id,
            project_id: &user.workspace_project_id,
            environment,
            label: input.label,
            expires_at_ms: input.expires_at_ms,
            plaintext_key: input.plaintext_key,
            notes: input.notes,
            api_key_group_id: input.api_key_group_id,
        },
    )
    .await
    .map_err(PortalIdentityError::from)
}

