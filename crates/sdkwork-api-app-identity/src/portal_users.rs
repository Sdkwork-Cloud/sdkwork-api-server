use super::*;

pub async fn register_portal_user(
    store: &dyn AdminStore,
    email: &str,
    password: &str,
    display_name: &str,
    signing_secret: &str,
) -> PortalResult<PortalAuthSession> {
    validate_registration_input(email, password, display_name)?;
    let normalized_email = normalize_email(email);

    if store
        .find_portal_user_by_email(&normalized_email)
        .await
        .map_err(PortalIdentityError::from)?
        .is_some()
    {
        return Err(PortalIdentityError::DuplicateEmail);
    }

    let tenant_id = generate_entity_id("tenant")?;
    let project_id = generate_entity_id("project")?;
    let user_id = generate_entity_id("user")?;
    let created_at_ms = now_epoch_millis()?;
    let (password_salt, password_hash) =
        hash_identity_password(password, "portal password").map_err(PortalIdentityError::from)?;

    let tenant = Tenant::new(&tenant_id, format!("{display_name} Workspace"));
    let project = Project::new(&tenant_id, &project_id, "default");
    store
        .insert_tenant(&tenant)
        .await
        .map_err(PortalIdentityError::from)?;
    store
        .insert_project(&project)
        .await
        .map_err(PortalIdentityError::from)?;

    let user = PortalUserRecord::new(
        &user_id,
        &normalized_email,
        display_name.trim(),
        password_salt,
        password_hash,
        &tenant_id,
        &project_id,
        true,
        created_at_ms,
    );
    let user = store
        .insert_portal_user(&user)
        .await
        .map_err(map_portal_store_error)?;
    portal_session_from_user(&user, signing_secret)
}

pub async fn login_portal_user(
    store: &dyn AdminStore,
    email: &str,
    password: &str,
    signing_secret: &str,
) -> PortalResult<PortalAuthSession> {
    validate_login_password(password).map_err(PortalIdentityError::InvalidInput)?;
    let _ = ensure_default_portal_user(store).await?;

    let normalized_email = normalize_email(email);
    let Some(user) = store
        .find_portal_user_by_email(&normalized_email)
        .await
        .map_err(PortalIdentityError::from)?
    else {
        return Err(PortalIdentityError::InvalidCredentials);
    };

    if !user.active {
        return Err(PortalIdentityError::InactiveUser);
    }

    if !verify_password_hash(password, &user.password_hash, "portal password")
        .map_err(PortalIdentityError::from)?
    {
        return Err(PortalIdentityError::InvalidCredentials);
    }

    portal_session_from_user(&user, signing_secret)
}

pub async fn load_portal_user_profile(
    store: &dyn AdminStore,
    user_id: &str,
) -> PortalResult<Option<PortalUserProfile>> {
    store
        .find_portal_user_by_id(user_id)
        .await
        .map(|maybe_user| maybe_user.map(|user| PortalUserProfile::from(&user)))
        .map_err(PortalIdentityError::from)
}

pub async fn list_portal_user_profiles(
    store: &dyn AdminStore,
) -> PortalResult<Vec<PortalUserProfile>> {
    let _ = ensure_default_portal_user(store).await?;
    store
        .list_portal_users()
        .await
        .map(|users| users.iter().map(PortalUserProfile::from).collect())
        .map_err(PortalIdentityError::from)
}

pub struct UpsertPortalUserInput<'a> {
    pub user_id: Option<&'a str>,
    pub email: &'a str,
    pub display_name: &'a str,
    pub password: Option<&'a str>,
    pub workspace_tenant_id: &'a str,
    pub workspace_project_id: &'a str,
    pub active: bool,
}

pub async fn upsert_portal_user(
    store: &dyn AdminStore,
    input: UpsertPortalUserInput<'_>,
) -> PortalResult<PortalUserProfile> {
    validate_identity_profile_input(input.email, input.display_name)
        .map_err(PortalIdentityError::InvalidInput)?;
    validate_workspace_scope(store, input.workspace_tenant_id, input.workspace_project_id).await?;

    let normalized_email = normalize_email(input.email);
    let requested_id = normalize_optional_value(input.user_id);
    let existing_by_id = match requested_id {
        Some(id) => store
            .find_portal_user_by_id(id)
            .await
            .map_err(PortalIdentityError::from)?,
        None => None,
    };

    if let Some(existing) = store
        .find_portal_user_by_email(&normalized_email)
        .await
        .map_err(PortalIdentityError::from)?
    {
        let matches_target = existing_by_id
            .as_ref()
            .map(|current| current.id == existing.id)
            .unwrap_or(false)
            || requested_id.is_some_and(|id| id == existing.id);
        if !matches_target {
            return Err(PortalIdentityError::DuplicateEmail);
        }
    }

    let target_id = match existing_by_id
        .as_ref()
        .map(|user| user.id.clone())
        .or_else(|| requested_id.map(ToOwned::to_owned))
    {
        Some(id) => id,
        None => generate_entity_id("user").map_err(PortalIdentityError::from)?,
    };

    let (password_salt, password_hash) =
        match input.password.map(str::trim).filter(|value| !value.is_empty()) {
            Some(next_password) => {
                validate_password_strength(next_password)
                    .map_err(PortalIdentityError::InvalidInput)?;
                hash_identity_password(next_password, "portal password")
                    .map_err(PortalIdentityError::from)?
            }
            None => {
                let Some(existing) = existing_by_id.as_ref() else {
                    return Err(PortalIdentityError::InvalidInput(
                        "password is required for new portal users".to_owned(),
                    ));
                };
                (
                    existing.password_salt.clone(),
                    existing.password_hash.clone(),
                )
            }
        };

    let created_at_ms = existing_by_id
        .as_ref()
        .map(|user| user.created_at_ms)
        .unwrap_or(now_epoch_millis().map_err(PortalIdentityError::from)?);
    let record = PortalUserRecord::new(
        target_id,
        normalized_email,
        input.display_name.trim(),
        password_salt,
        password_hash,
        input.workspace_tenant_id.trim(),
        input.workspace_project_id.trim(),
        input.active,
        created_at_ms,
    );
    let saved = store
        .insert_portal_user(&record)
        .await
        .map_err(map_portal_store_error)?;
    Ok(PortalUserProfile::from(&saved))
}

pub async fn set_portal_user_active(
    store: &dyn AdminStore,
    user_id: &str,
    active: bool,
) -> PortalResult<PortalUserProfile> {
    let Some(user) = store
        .find_portal_user_by_id(user_id)
        .await
        .map_err(PortalIdentityError::from)?
    else {
        return Err(PortalIdentityError::NotFound(
            "portal user not found".to_owned(),
        ));
    };

    let updated = PortalUserRecord::new(
        user.id,
        user.email,
        user.display_name,
        user.password_salt,
        user.password_hash,
        user.workspace_tenant_id,
        user.workspace_project_id,
        active,
        user.created_at_ms,
    );
    let saved = store
        .insert_portal_user(&updated)
        .await
        .map_err(PortalIdentityError::from)?;
    Ok(PortalUserProfile::from(&saved))
}

pub async fn reset_portal_user_password(
    store: &dyn AdminStore,
    user_id: &str,
    new_password: &str,
) -> PortalResult<PortalUserProfile> {
    validate_password_strength(new_password).map_err(PortalIdentityError::InvalidInput)?;

    let Some(user) = store
        .find_portal_user_by_id(user_id)
        .await
        .map_err(PortalIdentityError::from)?
    else {
        return Err(PortalIdentityError::NotFound(
            "portal user not found".to_owned(),
        ));
    };

    let (password_salt, password_hash) = hash_identity_password(new_password, "portal password")
        .map_err(PortalIdentityError::from)?;
    let updated = PortalUserRecord::new(
        user.id,
        user.email,
        user.display_name,
        password_salt,
        password_hash,
        user.workspace_tenant_id,
        user.workspace_project_id,
        user.active,
        user.created_at_ms,
    );
    let saved = store
        .insert_portal_user(&updated)
        .await
        .map_err(PortalIdentityError::from)?;
    Ok(PortalUserProfile::from(&saved))
}

pub async fn delete_portal_user(store: &dyn AdminStore, user_id: &str) -> PortalResult<bool> {
    let _ = ensure_default_portal_user(store).await?;

    let Some(user) = store
        .find_portal_user_by_id(user_id)
        .await
        .map_err(PortalIdentityError::from)?
    else {
        return Ok(false);
    };

    if user.id == DEFAULT_PORTAL_USER_ID || user.email == DEFAULT_PORTAL_EMAIL {
        return Err(PortalIdentityError::Protected(
            "default demo portal user cannot be deleted".to_owned(),
        ));
    }

    store
        .delete_portal_user(&user.id)
        .await
        .map_err(PortalIdentityError::from)
}

pub async fn load_portal_workspace_summary(
    store: &dyn AdminStore,
    user_id: &str,
) -> PortalResult<Option<PortalWorkspaceSummary>> {
    let Some(user) = store
        .find_portal_user_by_id(user_id)
        .await
        .map_err(PortalIdentityError::from)?
    else {
        return Ok(None);
    };

    let Some(tenant) = store
        .find_tenant(&user.workspace_tenant_id)
        .await
        .map_err(PortalIdentityError::from)?
    else {
        return Err(PortalIdentityError::NotFound(
            "portal workspace tenant not found".to_owned(),
        ));
    };

    let Some(project) = store
        .find_project(&user.workspace_project_id)
        .await
        .map_err(PortalIdentityError::from)?
    else {
        return Err(PortalIdentityError::NotFound(
            "portal workspace project not found".to_owned(),
        ));
    };

    Ok(Some(PortalWorkspaceSummary {
        user: PortalUserProfile::from(&user),
        tenant,
        project,
    }))
}

pub async fn change_portal_password(
    store: &dyn AdminStore,
    user_id: &str,
    current_password: &str,
    new_password: &str,
) -> PortalResult<PortalUserProfile> {
    validate_current_password(current_password).map_err(PortalIdentityError::InvalidInput)?;
    validate_password_strength(new_password).map_err(PortalIdentityError::InvalidInput)?;

    let Some(user) = store
        .find_portal_user_by_id(user_id)
        .await
        .map_err(PortalIdentityError::from)?
    else {
        return Err(PortalIdentityError::NotFound(
            "portal user not found".to_owned(),
        ));
    };

    if !user.active {
        return Err(PortalIdentityError::InactiveUser);
    }

    if !verify_password_hash(current_password, &user.password_hash, "portal password")
        .map_err(PortalIdentityError::from)?
    {
        return Err(PortalIdentityError::InvalidCredentials);
    }

    let (password_salt, password_hash) = hash_identity_password(new_password, "portal password")
        .map_err(PortalIdentityError::from)?;
    let updated = PortalUserRecord::new(
        user.id,
        user.email,
        user.display_name,
        password_salt,
        password_hash,
        user.workspace_tenant_id,
        user.workspace_project_id,
        user.active,
        user.created_at_ms,
    );
    let saved = store
        .insert_portal_user(&updated)
        .await
        .map_err(PortalIdentityError::from)?;
    Ok(PortalUserProfile::from(&saved))
}

