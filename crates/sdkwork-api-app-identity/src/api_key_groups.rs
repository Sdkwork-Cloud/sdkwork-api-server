use super::*;

pub async fn create_api_key_group(
    store: &dyn AdminStore,
    input: ApiKeyGroupInput,
) -> AdminResult<ApiKeyGroupRecord> {
    let normalized_tenant_id = require_trimmed_input(&input.tenant_id, "tenant_id")?;
    let normalized_project_id = require_trimmed_input(&input.project_id, "project_id")?;
    let normalized_environment = require_trimmed_input(&input.environment, "environment")?;
    let normalized_name = normalize_api_key_group_name(&input.name)?;
    let normalized_slug = normalize_api_key_group_slug(&normalized_name, input.slug.as_deref())?;
    let normalized_description =
        normalize_api_key_group_optional_value(input.description.as_deref());
    let normalized_color = normalize_api_key_group_optional_value(input.color.as_deref());
    let normalized_default_capability_scope =
        normalize_api_key_group_optional_value(input.default_capability_scope.as_deref());
    let normalized_default_routing_profile_id = validate_default_routing_profile_binding(
        store,
        normalized_tenant_id,
        normalized_project_id,
        input.default_routing_profile_id.as_deref(),
    )
    .await?;
    let normalized_default_accounting_mode =
        validate_default_accounting_mode_binding(input.default_accounting_mode.as_deref())?;

    ensure_api_key_group_slug_available(
        store,
        None,
        normalized_tenant_id,
        normalized_project_id,
        normalized_environment,
        &normalized_slug,
    )
    .await?;

    let now = now_epoch_millis().map_err(AdminIdentityError::from)?;
    let record = ApiKeyGroupRecord::new(
        generate_entity_id("api_key_group").map_err(AdminIdentityError::from)?,
        normalized_tenant_id,
        normalized_project_id,
        normalized_environment,
        normalized_name,
        normalized_slug,
    )
    .with_description_option(normalized_description)
    .with_color_option(normalized_color)
    .with_default_capability_scope_option(normalized_default_capability_scope)
    .with_default_routing_profile_id_option(normalized_default_routing_profile_id)
    .with_default_accounting_mode_option(normalized_default_accounting_mode)
    .with_created_at_ms(now)
    .with_updated_at_ms(now);

    store
        .insert_api_key_group(&record)
        .await
        .map_err(AdminIdentityError::from)
}

pub async fn list_api_key_groups(store: &dyn AdminStore) -> Result<Vec<ApiKeyGroupRecord>> {
    store.list_api_key_groups().await
}

pub async fn set_api_key_group_active(
    store: &dyn AdminStore,
    group_id: &str,
    active: bool,
) -> AdminResult<Option<ApiKeyGroupRecord>> {
    let Some(existing) = store
        .find_api_key_group(group_id)
        .await
        .map_err(AdminIdentityError::from)?
    else {
        return Ok(None);
    };

    let updated = ApiKeyGroupRecord {
        active,
        updated_at_ms: now_epoch_millis().map_err(AdminIdentityError::from)?,
        ..existing
    };

    let saved = store
        .insert_api_key_group(&updated)
        .await
        .map_err(AdminIdentityError::from)?;
    Ok(Some(saved))
}

pub async fn update_api_key_group(
    store: &dyn AdminStore,
    group_id: &str,
    input: ApiKeyGroupInput,
) -> AdminResult<Option<ApiKeyGroupRecord>> {
    let normalized_tenant_id = require_trimmed_input(&input.tenant_id, "tenant_id")?;
    let normalized_project_id = require_trimmed_input(&input.project_id, "project_id")?;
    let normalized_environment = require_trimmed_input(&input.environment, "environment")?;
    let normalized_name = normalize_api_key_group_name(&input.name)?;
    let normalized_slug = normalize_api_key_group_slug(&normalized_name, input.slug.as_deref())?;
    let normalized_description =
        normalize_api_key_group_optional_value(input.description.as_deref());
    let normalized_color = normalize_api_key_group_optional_value(input.color.as_deref());
    let normalized_default_capability_scope =
        normalize_api_key_group_optional_value(input.default_capability_scope.as_deref());
    let normalized_default_routing_profile_id = validate_default_routing_profile_binding(
        store,
        normalized_tenant_id,
        normalized_project_id,
        input.default_routing_profile_id.as_deref(),
    )
    .await?;
    let normalized_default_accounting_mode =
        validate_default_accounting_mode_binding(input.default_accounting_mode.as_deref())?;

    ensure_api_key_group_slug_available(
        store,
        Some(group_id),
        normalized_tenant_id,
        normalized_project_id,
        normalized_environment,
        &normalized_slug,
    )
    .await?;

    let Some(existing) = store
        .find_api_key_group(group_id)
        .await
        .map_err(AdminIdentityError::from)?
    else {
        return Ok(None);
    };

    let updated = ApiKeyGroupRecord {
        tenant_id: normalized_tenant_id.to_owned(),
        project_id: normalized_project_id.to_owned(),
        environment: normalized_environment.to_owned(),
        name: normalized_name,
        slug: normalized_slug,
        description: normalized_description,
        color: normalized_color,
        default_capability_scope: normalized_default_capability_scope,
        default_routing_profile_id: normalized_default_routing_profile_id,
        default_accounting_mode: normalized_default_accounting_mode,
        updated_at_ms: now_epoch_millis().map_err(AdminIdentityError::from)?,
        ..existing
    };

    let saved = store
        .insert_api_key_group(&updated)
        .await
        .map_err(AdminIdentityError::from)?;
    Ok(Some(saved))
}

pub async fn delete_api_key_group(store: &dyn AdminStore, group_id: &str) -> AdminResult<bool> {
    let Some(_existing) = store
        .find_api_key_group(group_id)
        .await
        .map_err(AdminIdentityError::from)?
    else {
        return Ok(false);
    };

    let has_bound_keys = store
        .list_gateway_api_keys()
        .await
        .map_err(AdminIdentityError::from)?
        .iter()
        .any(|key| key.api_key_group_id.as_deref() == Some(group_id));
    if has_bound_keys {
        return Err(AdminIdentityError::InvalidInput(
            "api key group has bound api keys".to_owned(),
        ));
    }

    store
        .delete_api_key_group(group_id)
        .await
        .map_err(AdminIdentityError::from)
}

