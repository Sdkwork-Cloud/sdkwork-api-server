use super::*;

pub async fn login_admin_user(
    store: &dyn AdminStore,
    email: &str,
    password: &str,
    signing_secret: &str,
) -> AdminResult<AdminAuthSession> {
    validate_login_password(password).map_err(AdminIdentityError::InvalidInput)?;

    let normalized_email = normalize_email(email);
    let Some(user) = store
        .find_admin_user_by_email(&normalized_email)
        .await
        .map_err(AdminIdentityError::from)?
    else {
        return Err(AdminIdentityError::InvalidCredentials);
    };

    if !user.active {
        return Err(AdminIdentityError::InactiveUser);
    }

    if !verify_password_hash(password, &user.password_hash, "admin password")
        .map_err(AdminIdentityError::from)?
    {
        return Err(AdminIdentityError::InvalidCredentials);
    }

    admin_session_from_user(&user, signing_secret)
}

pub async fn load_admin_user_profile(
    store: &dyn AdminStore,
    user_id: &str,
) -> AdminResult<Option<AdminUserProfile>> {
    store
        .find_admin_user_by_id(user_id)
        .await
        .map(|maybe_user| maybe_user.map(|user| AdminUserProfile::from(&user)))
        .map_err(AdminIdentityError::from)
}

pub async fn list_admin_user_profiles(
    store: &dyn AdminStore,
) -> AdminResult<Vec<AdminUserProfile>> {
    store
        .list_admin_users()
        .await
        .map(|users| users.iter().map(AdminUserProfile::from).collect())
        .map_err(AdminIdentityError::from)
}

pub async fn upsert_admin_user(
    store: &dyn AdminStore,
    user_id: Option<&str>,
    email: &str,
    display_name: &str,
    password: Option<&str>,
    role: Option<AdminUserRole>,
    active: bool,
) -> AdminResult<AdminUserProfile> {
    validate_identity_profile_input(email, display_name)
        .map_err(AdminIdentityError::InvalidInput)?;

    let normalized_email = normalize_email(email);
    let requested_id = normalize_optional_value(user_id);
    let existing_by_id = match requested_id {
        Some(id) => store
            .find_admin_user_by_id(id)
            .await
            .map_err(AdminIdentityError::from)?,
        None => None,
    };

    if let Some(existing) = store
        .find_admin_user_by_email(&normalized_email)
        .await
        .map_err(AdminIdentityError::from)?
    {
        let matches_target = existing_by_id
            .as_ref()
            .map(|current| current.id == existing.id)
            .unwrap_or(false)
            || requested_id.is_some_and(|id| id == existing.id);
        if !matches_target {
            return Err(AdminIdentityError::DuplicateEmail);
        }
    }

    let target_id = match existing_by_id
        .as_ref()
        .map(|user| user.id.clone())
        .or_else(|| requested_id.map(ToOwned::to_owned))
    {
        Some(id) => id,
        None => generate_entity_id("admin").map_err(AdminIdentityError::from)?,
    };

    let (password_salt, password_hash) =
        match password.map(str::trim).filter(|value| !value.is_empty()) {
            Some(next_password) => {
                validate_password_strength(next_password)
                    .map_err(AdminIdentityError::InvalidInput)?;
                hash_identity_password(next_password, "admin password")
                    .map_err(AdminIdentityError::from)?
            }
            None => {
                let Some(existing) = existing_by_id.as_ref() else {
                    return Err(AdminIdentityError::InvalidInput(
                        "password is required for new admin users".to_owned(),
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
        .unwrap_or(now_epoch_millis().map_err(AdminIdentityError::from)?);
    let resolved_role = role
        .or_else(|| existing_by_id.as_ref().map(|user| user.role))
        .unwrap_or_default();
    let record = AdminUserRecord::new(
        target_id,
        normalized_email,
        display_name.trim(),
        password_salt,
        password_hash,
        resolved_role,
        active,
        created_at_ms,
    );
    let saved = store
        .insert_admin_user(&record)
        .await
        .map_err(map_admin_store_error)?;
    Ok(AdminUserProfile::from(&saved))
}

pub async fn set_admin_user_active(
    store: &dyn AdminStore,
    user_id: &str,
    active: bool,
) -> AdminResult<AdminUserProfile> {
    let Some(user) = store
        .find_admin_user_by_id(user_id)
        .await
        .map_err(AdminIdentityError::from)?
    else {
        return Err(AdminIdentityError::NotFound(
            "admin user not found".to_owned(),
        ));
    };

    let updated = AdminUserRecord::new(
        user.id,
        user.email,
        user.display_name,
        user.password_salt,
        user.password_hash,
        user.role,
        active,
        user.created_at_ms,
    );
    let saved = store
        .insert_admin_user(&updated)
        .await
        .map_err(AdminIdentityError::from)?;
    Ok(AdminUserProfile::from(&saved))
}

pub async fn reset_admin_user_password(
    store: &dyn AdminStore,
    user_id: &str,
    new_password: &str,
) -> AdminResult<AdminUserProfile> {
    validate_password_strength(new_password).map_err(AdminIdentityError::InvalidInput)?;

    let Some(user) = store
        .find_admin_user_by_id(user_id)
        .await
        .map_err(AdminIdentityError::from)?
    else {
        return Err(AdminIdentityError::NotFound(
            "admin user not found".to_owned(),
        ));
    };

    let (password_salt, password_hash) =
        hash_identity_password(new_password, "admin password").map_err(AdminIdentityError::from)?;
    let updated = AdminUserRecord::new(
        user.id,
        user.email,
        user.display_name,
        password_salt,
        password_hash,
        user.role,
        user.active,
        user.created_at_ms,
    );
    let saved = store
        .insert_admin_user(&updated)
        .await
        .map_err(AdminIdentityError::from)?;
    Ok(AdminUserProfile::from(&saved))
}

pub async fn change_admin_password(
    store: &dyn AdminStore,
    user_id: &str,
    current_password: &str,
    new_password: &str,
) -> AdminResult<AdminUserProfile> {
    validate_current_password(current_password).map_err(AdminIdentityError::InvalidInput)?;
    validate_password_strength(new_password).map_err(AdminIdentityError::InvalidInput)?;

    let Some(user) = store
        .find_admin_user_by_id(user_id)
        .await
        .map_err(AdminIdentityError::from)?
    else {
        return Err(AdminIdentityError::NotFound(
            "admin user not found".to_owned(),
        ));
    };

    if !user.active {
        return Err(AdminIdentityError::InactiveUser);
    }

    if !verify_password_hash(current_password, &user.password_hash, "admin password")
        .map_err(AdminIdentityError::from)?
    {
        return Err(AdminIdentityError::InvalidCredentials);
    }

    let (password_salt, password_hash) =
        hash_identity_password(new_password, "admin password").map_err(AdminIdentityError::from)?;
    let updated = AdminUserRecord::new(
        user.id,
        user.email,
        user.display_name,
        password_salt,
        password_hash,
        user.role,
        user.active,
        user.created_at_ms,
    );
    let saved = store
        .insert_admin_user(&updated)
        .await
        .map_err(AdminIdentityError::from)?;
    Ok(AdminUserProfile::from(&saved))
}

pub async fn delete_admin_user(store: &dyn AdminStore, user_id: &str) -> AdminResult<bool> {
    let Some(user) = store
        .find_admin_user_by_id(user_id)
        .await
        .map_err(AdminIdentityError::from)?
    else {
        return Ok(false);
    };

    store
        .delete_admin_user(&user.id)
        .await
        .map_err(AdminIdentityError::from)
}

