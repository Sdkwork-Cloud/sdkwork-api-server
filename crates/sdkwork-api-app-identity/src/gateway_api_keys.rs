use super::*;

pub struct CreateGatewayApiKey;

impl CreateGatewayApiKey {
    pub fn execute(
        tenant_id: &str,
        project_id: &str,
        environment: &str,
    ) -> Result<CreatedGatewayApiKey> {
        Self::execute_with_metadata(
            tenant_id,
            project_id,
            environment,
            &default_gateway_api_key_label(environment),
            None,
        )
    }

    pub fn execute_with_metadata(
        tenant_id: &str,
        project_id: &str,
        environment: &str,
        label: &str,
        expires_at_ms: Option<u64>,
    ) -> Result<CreatedGatewayApiKey> {
        Self::execute_with_optional_plaintext(
            tenant_id,
            project_id,
            environment,
            label,
            expires_at_ms,
            None,
            None,
        )
    }

    pub fn execute_with_optional_plaintext(
        tenant_id: &str,
        project_id: &str,
        environment: &str,
        label: &str,
        expires_at_ms: Option<u64>,
        plaintext_key: Option<&str>,
        notes: Option<&str>,
    ) -> Result<CreatedGatewayApiKey> {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| anyhow!("system clock error"))?
            .as_nanos();
        let created_at_ms = now_epoch_millis()?;
        let normalized_label = normalize_gateway_api_key_label(label);
        let normalized_notes = normalize_gateway_api_key_notes(notes);
        let plaintext = plaintext_key
            .map(normalize_gateway_api_key_plaintext)
            .transpose()
            .map_err(|message| anyhow!(message))?
            .unwrap_or_else(|| format!("skw_{environment}_{nonce:x}"));
        let hashed = hash_gateway_api_key(&plaintext);
        Ok(CreatedGatewayApiKey {
            plaintext,
            hashed,
            tenant_id: tenant_id.to_owned(),
            project_id: project_id.to_owned(),
            environment: environment.to_owned(),
            api_key_group_id: None,
            label: normalized_label,
            notes: normalized_notes,
            created_at_ms,
            expires_at_ms,
        })
    }
}

pub struct PersistGatewayApiKeyInput<'a> {
    pub tenant_id: &'a str,
    pub project_id: &'a str,
    pub environment: &'a str,
    pub label: &'a str,
    pub expires_at_ms: Option<u64>,
    pub plaintext_key: Option<&'a str>,
    pub notes: Option<&'a str>,
    pub api_key_group_id: Option<&'a str>,
}

pub struct UpdateGatewayApiKeyMetadataInput<'a> {
    pub hashed_key: &'a str,
    pub tenant_id: &'a str,
    pub project_id: &'a str,
    pub environment: &'a str,
    pub label: &'a str,
    pub expires_at_ms: Option<u64>,
    pub notes: Option<&'a str>,
    pub api_key_group_id: Option<&'a str>,
}

fn redact_gateway_api_key_record(mut record: GatewayApiKeyRecord) -> GatewayApiKeyRecord {
    record.raw_key = None;
    record
}

pub async fn persist_gateway_api_key(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    environment: &str,
) -> Result<CreatedGatewayApiKey> {
    let default_label = default_gateway_api_key_label(environment);
    persist_gateway_api_key_with_metadata(
        store,
        PersistGatewayApiKeyInput {
            tenant_id,
            project_id,
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

pub async fn persist_gateway_api_key_with_metadata(
    store: &dyn AdminStore,
    input: PersistGatewayApiKeyInput<'_>,
) -> Result<CreatedGatewayApiKey> {
    validate_gateway_api_key_metadata(input.label, input.notes, input.expires_at_ms)
        .map_err(|message| anyhow!(message))?;
    let validated_group_id = validate_gateway_api_key_group_assignment(
        store,
        input.tenant_id,
        input.project_id,
        input.environment,
        input.api_key_group_id,
    )
    .await?;
    let mut created = CreateGatewayApiKey::execute_with_optional_plaintext(
        input.tenant_id,
        input.project_id,
        input.environment,
        input.label,
        input.expires_at_ms,
        input.plaintext_key,
        input.notes,
    )?;
    created.api_key_group_id = validated_group_id.clone();
    let record = GatewayApiKeyRecord::new(
        input.tenant_id,
        input.project_id,
        input.environment,
        created.hashed.clone(),
    )
    .with_api_key_group_id_option(validated_group_id)
    .with_label(created.label.clone())
    .with_notes_option(created.notes.clone())
    .with_created_at_ms(created.created_at_ms)
    .with_expires_at_ms_option(created.expires_at_ms);
    store.insert_gateway_api_key(&record).await?;
    Ok(created)
}

pub async fn list_gateway_api_keys(store: &dyn AdminStore) -> Result<Vec<GatewayApiKeyRecord>> {
    store
        .list_gateway_api_keys()
        .await
        .map(|records| records.into_iter().map(redact_gateway_api_key_record).collect())
}

pub async fn set_gateway_api_key_active(
    store: &dyn AdminStore,
    hashed_key: &str,
    active: bool,
) -> Result<Option<GatewayApiKeyRecord>> {
    let Some(existing) = store.find_gateway_api_key(hashed_key).await? else {
        return Ok(None);
    };

    let updated = GatewayApiKeyRecord {
        active,
        raw_key: None,
        ..existing
    };

    let saved = store.insert_gateway_api_key(&updated).await?;
    Ok(Some(redact_gateway_api_key_record(saved)))
}

pub async fn update_gateway_api_key_metadata(
    store: &dyn AdminStore,
    input: UpdateGatewayApiKeyMetadataInput<'_>,
) -> AdminResult<Option<GatewayApiKeyRecord>> {
    let normalized_tenant_id = input.tenant_id.trim();
    if normalized_tenant_id.is_empty() {
        return Err(AdminIdentityError::InvalidInput(
            "tenant_id is required".to_owned(),
        ));
    }

    let normalized_project_id = input.project_id.trim();
    if normalized_project_id.is_empty() {
        return Err(AdminIdentityError::InvalidInput(
            "project_id is required".to_owned(),
        ));
    }

    let normalized_environment = input.environment.trim();
    if normalized_environment.is_empty() {
        return Err(AdminIdentityError::InvalidInput(
            "environment is required".to_owned(),
        ));
    }

    validate_gateway_api_key_metadata(input.label, input.notes, input.expires_at_ms)
        .map_err(AdminIdentityError::InvalidInput)?;
    let validated_group_id = validate_gateway_api_key_group_assignment(
        store,
        normalized_tenant_id,
        normalized_project_id,
        normalized_environment,
        input.api_key_group_id,
    )
    .await
    .map_err(|error| AdminIdentityError::InvalidInput(error.to_string()))?;

    let Some(existing) = store
        .find_gateway_api_key(input.hashed_key)
        .await
        .map_err(AdminIdentityError::from)?
    else {
        return Ok(None);
    };

    let updated = GatewayApiKeyRecord {
        tenant_id: normalized_tenant_id.to_owned(),
        project_id: normalized_project_id.to_owned(),
        environment: normalized_environment.to_owned(),
        raw_key: None,
        api_key_group_id: validated_group_id,
        label: normalize_gateway_api_key_label(input.label),
        notes: normalize_gateway_api_key_notes(input.notes),
        expires_at_ms: input.expires_at_ms,
        ..existing
    };

    let saved = store
        .insert_gateway_api_key(&updated)
        .await
        .map_err(AdminIdentityError::from)?;
    Ok(Some(redact_gateway_api_key_record(saved)))
}

pub async fn delete_gateway_api_key(store: &dyn AdminStore, hashed_key: &str) -> Result<bool> {
    store.delete_gateway_api_key(hashed_key).await
}

pub async fn resolve_gateway_request_context(
    store: &dyn AdminStore,
    plaintext_key: &str,
) -> Result<Option<GatewayRequestContext>> {
    let hashed_key = hash_gateway_api_key(plaintext_key);
    let Some(record) = store.find_gateway_api_key(&hashed_key).await? else {
        return Ok(None);
    };

    if !record.active {
        return Ok(None);
    }

    if record
        .expires_at_ms
        .is_some_and(|expires_at_ms| expires_at_ms <= now_epoch_millis().unwrap_or(u64::MAX))
    {
        return Ok(None);
    }

    let updated = GatewayApiKeyRecord {
        last_used_at_ms: Some(now_epoch_millis()?),
        raw_key: None,
        ..record.clone()
    };
    let _ = store.insert_gateway_api_key(&updated).await?;

    Ok(Some(GatewayRequestContext {
        tenant_id: record.tenant_id,
        project_id: record.project_id,
        environment: record.environment,
        api_key_hash: hashed_key,
        api_key_group_id: record.api_key_group_id,
        canonical_tenant_id: None,
        canonical_organization_id: None,
        canonical_user_id: None,
        canonical_api_key_id: None,
    }))
}

pub fn gateway_auth_subject_from_request_context(
    context: &GatewayRequestContext,
) -> GatewayAuthSubject {
    let subject = match (
        context.canonical_tenant_id,
        context.canonical_organization_id,
        context.canonical_user_id,
        context.canonical_api_key_id,
    ) {
        (Some(tenant_id), Some(organization_id), Some(user_id), Some(api_key_id)) => {
            GatewayAuthSubject::for_api_key(
                tenant_id,
                organization_id,
                user_id,
                api_key_id,
                context.api_key_hash().to_owned(),
            )
        }
        _ => GatewayAuthSubject::for_api_key(
            stable_gateway_principal_id("tenant", &[context.tenant_id()]),
            stable_gateway_principal_id(
                "organization",
                &[context.tenant_id(), context.project_id()],
            ),
            stable_gateway_principal_id(
                "project_principal",
                &[context.tenant_id(), context.project_id()],
            ),
            stable_gateway_principal_id("api_key", &[context.api_key_hash()]),
            context.api_key_hash().to_owned(),
        ),
    };

    subject
    .with_platform("gateway")
    .with_owner(format!(
        "project:{}:{}",
        context.tenant_id(),
        context.project_id()
    ))
}

pub async fn resolve_canonical_gateway_request_context_from_api_key<S>(
    store: &S,
    plaintext_key: &str,
) -> Result<Option<GatewayRequestContext>>
where
    S: IdentityKernelStore + ?Sized,
{
    let hashed_key = hash_gateway_api_key(plaintext_key);
    let Some(record) = store
        .find_canonical_api_key_record_by_hash(&hashed_key)
        .await?
    else {
        return Ok(None);
    };

    if record.status != "active" {
        return Ok(None);
    }

    if record
        .expires_at_ms
        .is_some_and(|expires_at_ms| expires_at_ms <= now_epoch_millis().unwrap_or(u64::MAX))
    {
        return Ok(None);
    }

    let Some(user) = store.find_identity_user_record(record.user_id).await? else {
        return Ok(None);
    };
    if user.status != "active" {
        return Ok(None);
    }

    let tenant_id = record.tenant_id.to_string();
    let Some(project) = resolve_canonical_gateway_project(store, &tenant_id).await? else {
        return Ok(None);
    };

    let now_ms = now_epoch_millis()?;
    let updated = record
        .clone()
        .with_last_used_at_ms(Some(now_ms))
        .with_updated_at_ms(now_ms);
    let _ = store.insert_canonical_api_key_record(&updated).await?;

    Ok(Some(
        GatewayRequestContext {
            tenant_id,
            project_id: project.id,
            environment: "live".to_owned(),
            api_key_hash: hashed_key,
            api_key_group_id: None,
            canonical_tenant_id: None,
            canonical_organization_id: None,
            canonical_user_id: None,
            canonical_api_key_id: None,
        }
        .with_canonical_subject(
            record.tenant_id,
            record.organization_id,
            record.user_id,
            Some(record.api_key_id),
        ),
    ))
}

async fn resolve_canonical_gateway_project<S>(
    store: &S,
    tenant_id: &str,
) -> Result<Option<Project>>
where
    S: AdminStore + ?Sized,
{
    let mut projects = store
        .list_projects()
        .await?
        .into_iter()
        .filter(|project| project.tenant_id == tenant_id)
        .collect::<Vec<_>>();

    if projects.len() == 1 {
        Ok(projects.pop())
    } else {
        Ok(None)
    }
}

pub async fn resolve_gateway_auth_subject_from_api_key<S>(
    store: &S,
    plaintext_key: &str,
) -> Result<Option<GatewayAuthSubject>>
where
    S: IdentityKernelStore + ?Sized,
{
    let hashed_key = hash_gateway_api_key(plaintext_key);
    let Some(record) = store
        .find_canonical_api_key_record_by_hash(&hashed_key)
        .await?
    else {
        return Ok(None);
    };

    if record.status != "active" {
        return Ok(None);
    }

    if record
        .expires_at_ms
        .is_some_and(|expires_at_ms| expires_at_ms <= now_epoch_millis().unwrap_or(u64::MAX))
    {
        return Ok(None);
    }

    let Some(user) = store.find_identity_user_record(record.user_id).await? else {
        return Ok(None);
    };
    if user.status != "active" {
        return Ok(None);
    }

    let updated = record
        .clone()
        .with_last_used_at_ms(Some(now_epoch_millis()?))
        .with_updated_at_ms(now_epoch_millis()?);
    let _ = store.insert_canonical_api_key_record(&updated).await?;

    Ok(Some(GatewayAuthSubject::for_api_key(
        record.tenant_id,
        record.organization_id,
        record.user_id,
        record.api_key_id,
        hashed_key,
    )))
}

