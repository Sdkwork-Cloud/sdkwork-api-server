use super::*;

pub(crate) async fn ensure_default_admin_user(store: &dyn AdminStore) -> AdminResult<AdminUserRecord> {
    if let Some(user) = store
        .find_admin_user_by_email(DEFAULT_ADMIN_EMAIL)
        .await
        .map_err(AdminIdentityError::from)?
    {
        return Ok(user);
    }

    let created_at_ms = now_epoch_millis().map_err(AdminIdentityError::from)?;
    let (password_salt, password_hash) =
        hash_identity_password(DEFAULT_ADMIN_PASSWORD, "admin password")
            .map_err(AdminIdentityError::from)?;
    let user = AdminUserRecord::new(
        DEFAULT_ADMIN_USER_ID,
        DEFAULT_ADMIN_EMAIL,
        DEFAULT_ADMIN_DISPLAY_NAME,
        password_salt,
        password_hash,
        true,
        created_at_ms,
    );

    match store.insert_admin_user(&user).await {
        Ok(saved) => Ok(saved),
        Err(error) => {
            if looks_like_duplicate_error(&error) {
                store
                    .find_admin_user_by_email(DEFAULT_ADMIN_EMAIL)
                    .await
                    .map_err(AdminIdentityError::from)?
                    .ok_or_else(|| AdminIdentityError::Storage(error))
            } else {
                Err(AdminIdentityError::Storage(error))
            }
        }
    }
}

pub(crate) async fn ensure_default_portal_user(store: &dyn AdminStore) -> PortalResult<PortalUserRecord> {
    if let Some(user) = store
        .find_portal_user_by_email(DEFAULT_PORTAL_EMAIL)
        .await
        .map_err(PortalIdentityError::from)?
    {
        return Ok(user);
    }

    let tenant = Tenant::new(DEFAULT_PORTAL_TENANT_ID, "Portal Demo Workspace");
    let project = Project::new(
        DEFAULT_PORTAL_TENANT_ID,
        DEFAULT_PORTAL_PROJECT_ID,
        "default",
    );
    store
        .insert_tenant(&tenant)
        .await
        .map_err(PortalIdentityError::from)?;
    store
        .insert_project(&project)
        .await
        .map_err(PortalIdentityError::from)?;

    let created_at_ms = now_epoch_millis().map_err(PortalIdentityError::from)?;
    let (password_salt, password_hash) =
        hash_identity_password(DEFAULT_PORTAL_PASSWORD, "portal password")
            .map_err(PortalIdentityError::from)?;
    let user = PortalUserRecord::new(
        DEFAULT_PORTAL_USER_ID,
        DEFAULT_PORTAL_EMAIL,
        DEFAULT_PORTAL_DISPLAY_NAME,
        password_salt,
        password_hash,
        DEFAULT_PORTAL_TENANT_ID,
        DEFAULT_PORTAL_PROJECT_ID,
        true,
        created_at_ms,
    );

    match store.insert_portal_user(&user).await {
        Ok(saved) => Ok(saved),
        Err(error) => {
            if looks_like_duplicate_error(&error) {
                store
                    .find_portal_user_by_email(DEFAULT_PORTAL_EMAIL)
                    .await
                    .map_err(PortalIdentityError::from)?
                    .ok_or_else(|| PortalIdentityError::Storage(error))
            } else {
                Err(map_portal_store_error(error))
            }
        }
    }
}

pub(crate) fn admin_session_from_user(
    user: &AdminUserRecord,
    signing_secret: &str,
) -> AdminResult<AdminAuthSession> {
    let token = issue_jwt(&user.id, signing_secret).map_err(AdminIdentityError::from)?;
    Ok(AdminAuthSession {
        token,
        user: AdminUserProfile::from(user),
    })
}

pub(crate) fn portal_session_from_user(
    user: &PortalUserRecord,
    signing_secret: &str,
) -> PortalResult<PortalAuthSession> {
    let token = issue_portal_jwt(user, signing_secret)?;
    Ok(PortalAuthSession {
        token,
        user: PortalUserProfile::from(user),
        workspace: PortalWorkspaceScope {
            tenant_id: user.workspace_tenant_id.clone(),
            project_id: user.workspace_project_id.clone(),
        },
    })
}

pub(crate) fn issue_portal_jwt(user: &PortalUserRecord, signing_secret: &str) -> PortalResult<String> {
    ensure_jsonwebtoken_provider();
    let issued_at = now_epoch_secs().map_err(PortalIdentityError::from)?;
    let claims = PortalClaims {
        sub: user.id.clone(),
        iss: PORTAL_JWT_ISSUER.to_owned(),
        aud: PORTAL_JWT_AUDIENCE.to_owned(),
        exp: (issued_at + PORTAL_JWT_TTL_SECS) as usize,
        iat: issued_at as usize,
        email: user.email.clone(),
        workspace_tenant_id: user.workspace_tenant_id.clone(),
        workspace_project_id: user.workspace_project_id.clone(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(signing_secret.as_bytes()),
    )
    .map_err(|error| PortalIdentityError::Storage(error.into()))
}

pub(crate) fn validate_registration_input(
    email: &str,
    password: &str,
    display_name: &str,
) -> PortalResult<()> {
    validate_identity_profile_input(email, display_name)
        .map_err(PortalIdentityError::InvalidInput)?;
    validate_password_strength(password).map_err(PortalIdentityError::InvalidInput)?;
    Ok(())
}

pub(crate) fn validate_identity_profile_input(
    email: &str,
    display_name: &str,
) -> std::result::Result<(), String> {
    let normalized_email = normalize_email(email);
    if normalized_email.is_empty() || !normalized_email.contains('@') {
        return Err("email must be a valid address".to_owned());
    }
    if display_name.trim().is_empty() {
        return Err("display_name is required".to_owned());
    }
    Ok(())
}

pub(crate) fn normalize_email(email: &str) -> String {
    email.trim().to_ascii_lowercase()
}

pub(crate) fn normalize_optional_value(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

pub(crate) fn generate_entity_id(prefix: &str) -> Result<String> {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| anyhow!("system clock error"))?
        .as_nanos();
    Ok(format!("{prefix}_{nonce:x}"))
}

pub(crate) fn hash_identity_password(password: &str, context: &str) -> Result<(String, String)> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|error| anyhow!("{context} hash failed: {error}"))?
        .to_string();
    Ok((salt.to_string(), hash))
}

pub(crate) fn verify_password_hash(password: &str, password_hash: &str, context: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(password_hash)
        .map_err(|error| anyhow!("{context} hash parse failed: {error}"))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub(crate) fn validate_login_password(password: &str) -> std::result::Result<(), String> {
    if password.trim().is_empty() {
        return Err("password is required".to_owned());
    }
    Ok(())
}

pub(crate) fn validate_current_password(password: &str) -> std::result::Result<(), String> {
    if password.trim().is_empty() {
        return Err("current_password is required".to_owned());
    }
    Ok(())
}

pub(crate) fn validate_password_strength(password: &str) -> std::result::Result<(), String> {
    if password.chars().count() < 12 {
        return Err("password must be at least 12 characters".to_owned());
    }
    if password.chars().any(char::is_whitespace) {
        return Err("password must not contain whitespace".to_owned());
    }
    if !password.chars().any(|ch| ch.is_ascii_uppercase()) {
        return Err("password must include an uppercase letter".to_owned());
    }
    if !password.chars().any(|ch| ch.is_ascii_lowercase()) {
        return Err("password must include a lowercase letter".to_owned());
    }
    if !password.chars().any(|ch| ch.is_ascii_digit()) {
        return Err("password must include a number".to_owned());
    }
    if !password.chars().any(|ch| ch.is_ascii_punctuation()) {
        return Err("password must include a special character".to_owned());
    }
    Ok(())
}

pub(crate) fn require_trimmed_input<'a>(value: &'a str, field_name: &str) -> AdminResult<&'a str> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(AdminIdentityError::InvalidInput(format!(
            "{field_name} is required"
        )));
    }
    Ok(normalized)
}

pub(crate) fn normalize_api_key_group_name(name: &str) -> AdminResult<String> {
    let normalized = name.trim();
    if normalized.is_empty() {
        return Err(AdminIdentityError::InvalidInput(
            "name is required".to_owned(),
        ));
    }
    Ok(normalized.to_owned())
}

pub(crate) fn normalize_api_key_group_slug(name: &str, slug: Option<&str>) -> AdminResult<String> {
    let source = normalize_optional_value(slug).unwrap_or(name);
    let mut normalized = String::new();
    let mut previous_was_dash = false;

    for ch in source.chars() {
        if ch.is_ascii_alphanumeric() {
            normalized.push(ch.to_ascii_lowercase());
            previous_was_dash = false;
        } else if !normalized.is_empty() && !previous_was_dash {
            normalized.push('-');
            previous_was_dash = true;
        }
    }

    while normalized.ends_with('-') {
        normalized.pop();
    }

    if normalized.is_empty() {
        return Err(AdminIdentityError::InvalidInput(
            "slug is required".to_owned(),
        ));
    }

    Ok(normalized)
}

pub(crate) fn normalize_api_key_group_optional_value(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

pub(crate) fn validate_default_accounting_mode_binding(
    default_accounting_mode: Option<&str>,
) -> AdminResult<Option<String>> {
    let Some(default_accounting_mode) =
        normalize_api_key_group_optional_value(default_accounting_mode)
    else {
        return Ok(None);
    };

    let normalized = default_accounting_mode.to_ascii_lowercase();
    let parsed = BillingAccountingMode::from_str(&normalized).map_err(|_| {
        AdminIdentityError::InvalidInput(
            "default_accounting_mode must be one of: platform_credit, byok, passthrough".to_owned(),
        )
    })?;

    Ok(Some(parsed.as_str().to_owned()))
}

pub(crate) async fn ensure_api_key_group_slug_available(
    store: &dyn AdminStore,
    current_group_id: Option<&str>,
    tenant_id: &str,
    project_id: &str,
    environment: &str,
    slug: &str,
) -> AdminResult<()> {
    let groups = store
        .list_api_key_groups()
        .await
        .map_err(AdminIdentityError::from)?;
    let duplicate_exists = groups.iter().any(|group| {
        group.tenant_id == tenant_id
            && group.project_id == project_id
            && group.environment == environment
            && group.slug == slug
            && current_group_id != Some(group.group_id.as_str())
    });
    if duplicate_exists {
        return Err(AdminIdentityError::InvalidInput(
            "api key group slug already exists".to_owned(),
        ));
    }
    Ok(())
}

pub(crate) async fn validate_default_routing_profile_binding(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    default_routing_profile_id: Option<&str>,
) -> AdminResult<Option<String>> {
    let Some(profile_id) = normalize_api_key_group_optional_value(default_routing_profile_id)
    else {
        return Ok(None);
    };

    let Some(profile) = store
        .find_routing_profile(&profile_id)
        .await
        .map_err(AdminIdentityError::from)?
    else {
        return Err(AdminIdentityError::InvalidInput(
            "routing profile not found".to_owned(),
        ));
    };

    if profile.tenant_id != tenant_id {
        return Err(AdminIdentityError::InvalidInput(
            "routing profile tenant does not match".to_owned(),
        ));
    }

    if profile.project_id != project_id {
        return Err(AdminIdentityError::InvalidInput(
            "routing profile project does not match".to_owned(),
        ));
    }

    if !profile.active {
        return Err(AdminIdentityError::InvalidInput(
            "routing profile is inactive".to_owned(),
        ));
    }

    Ok(Some(profile.profile_id))
}

pub(crate) async fn validate_gateway_api_key_group_assignment(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    environment: &str,
    api_key_group_id: Option<&str>,
) -> Result<Option<String>> {
    let Some(group_id) = normalize_optional_value(api_key_group_id) else {
        return Ok(None);
    };

    let Some(group) = store.find_api_key_group(group_id).await? else {
        return Err(anyhow!("api key group not found"));
    };

    validate_gateway_api_key_group_record(&group, tenant_id, project_id, environment)
        .map_err(anyhow::Error::msg)?;

    Ok(Some(group.group_id))
}

fn validate_gateway_api_key_group_record(
    group: &ApiKeyGroupRecord,
    tenant_id: &str,
    project_id: &str,
    environment: &str,
) -> std::result::Result<(), String> {
    if group.tenant_id != tenant_id {
        return Err("api key group tenant does not match".to_owned());
    }

    if group.project_id != project_id {
        return Err("api key group project does not match".to_owned());
    }

    if group.environment != environment {
        return Err("api key group environment does not match".to_owned());
    }

    if !group.active {
        return Err("api key group is inactive".to_owned());
    }

    Ok(())
}

pub(crate) fn default_gateway_api_key_label(environment: &str) -> String {
    format!("{} gateway key", environment.trim())
}

pub(crate) fn stable_gateway_principal_id(scope: &str, parts: &[&str]) -> u64 {
    let mut hasher = Sha256::new();
    hasher.update(b"sdkwork.gateway-commercial-principal.v1");
    hasher.update([0x1f]);
    hasher.update(scope.as_bytes());
    for part in parts {
        hasher.update([0x1f]);
        hasher.update(part.as_bytes());
    }
    let digest = hasher.finalize();
    let mut bytes = [0_u8; 8];
    bytes.copy_from_slice(&digest[..8]);
    (u64::from_be_bytes(bytes) & 0x3fff_ffff_ffff_ffff) | (1_u64 << 62)
}

pub(crate) fn normalize_gateway_api_key_label(label: &str) -> String {
    let trimmed = label.trim();
    if trimmed.is_empty() {
        "Gateway key".to_owned()
    } else {
        trimmed.to_owned()
    }
}

pub(crate) fn normalize_gateway_api_key_plaintext(value: &str) -> std::result::Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("api_key is required when custom key mode is selected".to_owned());
    }

    Ok(trimmed.to_owned())
}

pub(crate) fn normalize_gateway_api_key_notes(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

pub(crate) fn validate_gateway_api_key_metadata(
    label: &str,
    notes: Option<&str>,
    expires_at_ms: Option<u64>,
) -> std::result::Result<(), String> {
    if label.trim().is_empty() {
        return Err("label is required".to_owned());
    }

    let _ = normalize_gateway_api_key_notes(notes);

    if let Some(expires_at_ms) = expires_at_ms {
        let now = now_epoch_millis().map_err(|error| error.to_string())?;
        if expires_at_ms <= now {
            return Err("expires_at_ms must be in the future".to_owned());
        }
    }

    Ok(())
}

pub(crate) fn map_portal_store_error(error: anyhow::Error) -> PortalIdentityError {
    if looks_like_duplicate_error(&error) {
        PortalIdentityError::DuplicateEmail
    } else {
        PortalIdentityError::Storage(error)
    }
}

pub(crate) fn map_admin_identity_error_to_portal(error: AdminIdentityError) -> PortalIdentityError {
    match error {
        AdminIdentityError::InvalidInput(message) => PortalIdentityError::InvalidInput(message),
        AdminIdentityError::DuplicateEmail => {
            PortalIdentityError::InvalidInput("duplicate identity record".to_owned())
        }
        AdminIdentityError::Protected(message) => PortalIdentityError::Protected(message),
        AdminIdentityError::InvalidCredentials => PortalIdentityError::InvalidCredentials,
        AdminIdentityError::InactiveUser => PortalIdentityError::InactiveUser,
        AdminIdentityError::NotFound(message) => PortalIdentityError::NotFound(message),
        AdminIdentityError::Storage(error) => PortalIdentityError::Storage(error),
    }
}

pub(crate) fn map_admin_store_error(error: anyhow::Error) -> AdminIdentityError {
    if looks_like_duplicate_error(&error) {
        AdminIdentityError::DuplicateEmail
    } else {
        AdminIdentityError::Storage(error)
    }
}

fn looks_like_duplicate_error(error: &anyhow::Error) -> bool {
    let lowered = error.to_string().to_ascii_lowercase();
    lowered.contains("unique")
        || lowered.contains("duplicate")
        || lowered.contains("identity_users.email")
        || lowered.contains("admin_users.email")
}

pub(crate) async fn validate_workspace_scope(
    store: &dyn AdminStore,
    workspace_tenant_id: &str,
    workspace_project_id: &str,
) -> PortalResult<()> {
    let tenant_id = workspace_tenant_id.trim();
    if tenant_id.is_empty() {
        return Err(PortalIdentityError::InvalidInput(
            "workspace_tenant_id is required".to_owned(),
        ));
    }

    let project_id = workspace_project_id.trim();
    if project_id.is_empty() {
        return Err(PortalIdentityError::InvalidInput(
            "workspace_project_id is required".to_owned(),
        ));
    }

    let Some(_tenant) = store
        .find_tenant(tenant_id)
        .await
        .map_err(PortalIdentityError::from)?
    else {
        return Err(PortalIdentityError::NotFound(
            "workspace tenant not found".to_owned(),
        ));
    };

    let Some(project) = store
        .find_project(project_id)
        .await
        .map_err(PortalIdentityError::from)?
    else {
        return Err(PortalIdentityError::NotFound(
            "workspace project not found".to_owned(),
        ));
    };

    if project.tenant_id != tenant_id {
        return Err(PortalIdentityError::InvalidInput(
            "workspace project does not belong to workspace tenant".to_owned(),
        ));
    }

    Ok(())
}

pub(crate) fn now_epoch_secs() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| anyhow!("system clock error"))?
        .as_secs())
}

pub(crate) fn now_epoch_millis() -> Result<u64> {
    Ok(u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| anyhow!("system clock error"))?
            .as_millis(),
    )?)
}

