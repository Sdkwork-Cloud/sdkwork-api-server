use anyhow::{anyhow, Result};
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rand_core::OsRng;
use sdkwork_api_domain_identity::{
    AdminUserProfile, AdminUserRecord, GatewayApiKeyRecord, PortalUserProfile, PortalUserRecord,
};
use sdkwork_api_domain_tenant::{Project, Tenant};
use sdkwork_api_storage_core::AdminStore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

const ADMIN_JWT_ISSUER: &str = "sdkwork-admin";
const ADMIN_JWT_AUDIENCE: &str = "sdkwork-admin-ui";
const ADMIN_JWT_TTL_SECS: u64 = 60 * 60 * 12;

const PORTAL_JWT_ISSUER: &str = "sdkwork-portal";
const PORTAL_JWT_AUDIENCE: &str = "sdkwork-public-portal";
const PORTAL_JWT_TTL_SECS: u64 = 60 * 60 * 12;

pub const DEFAULT_ADMIN_EMAIL: &str = "admin@sdkwork.local";
pub const DEFAULT_ADMIN_PASSWORD: &str = "ChangeMe123!";
pub const DEFAULT_ADMIN_DISPLAY_NAME: &str = "Admin Operator";

pub const DEFAULT_PORTAL_EMAIL: &str = "portal@sdkwork.local";
pub const DEFAULT_PORTAL_PASSWORD: &str = "ChangeMe123!";
pub const DEFAULT_PORTAL_DISPLAY_NAME: &str = "Portal Demo";

const DEFAULT_ADMIN_USER_ID: &str = "admin_local_default";
const DEFAULT_PORTAL_USER_ID: &str = "user_local_demo";
const DEFAULT_PORTAL_TENANT_ID: &str = "tenant_local_demo";
const DEFAULT_PORTAL_PROJECT_ID: &str = "project_local_demo";

type AdminResult<T> = std::result::Result<T, AdminIdentityError>;
type PortalResult<T> = std::result::Result<T, PortalIdentityError>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub iss: String,
    pub aud: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortalClaims {
    pub sub: String,
    pub iss: String,
    pub aud: String,
    pub exp: usize,
    pub iat: usize,
    pub email: String,
    pub workspace_tenant_id: String,
    pub workspace_project_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GatewayRequestContext {
    pub tenant_id: String,
    pub project_id: String,
    pub environment: String,
}

impl GatewayRequestContext {
    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    pub fn project_id(&self) -> &str {
        &self.project_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreatedGatewayApiKey {
    pub plaintext: String,
    pub hashed: String,
    pub tenant_id: String,
    pub project_id: String,
    pub environment: String,
    pub label: String,
    pub notes: Option<String>,
    pub created_at_ms: u64,
    pub expires_at_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortalWorkspaceScope {
    pub tenant_id: String,
    pub project_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortalAuthSession {
    pub token: String,
    pub user: PortalUserProfile,
    pub workspace: PortalWorkspaceScope,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdminAuthSession {
    pub token: String,
    pub user: AdminUserProfile,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortalWorkspaceSummary {
    pub user: PortalUserProfile,
    pub tenant: Tenant,
    pub project: Project,
}

#[derive(Debug)]
pub enum PortalIdentityError {
    InvalidInput(String),
    DuplicateEmail,
    Protected(String),
    InvalidCredentials,
    InactiveUser,
    NotFound(String),
    Storage(anyhow::Error),
}

#[derive(Debug)]
pub enum AdminIdentityError {
    InvalidInput(String),
    DuplicateEmail,
    Protected(String),
    InvalidCredentials,
    InactiveUser,
    NotFound(String),
    Storage(anyhow::Error),
}

impl std::fmt::Display for PortalIdentityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput(message) => write!(f, "{message}"),
            Self::DuplicateEmail => write!(f, "portal user already exists"),
            Self::Protected(message) => write!(f, "{message}"),
            Self::InvalidCredentials => write!(f, "invalid email or password"),
            Self::InactiveUser => write!(f, "portal user is inactive"),
            Self::NotFound(message) => write!(f, "{message}"),
            Self::Storage(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for PortalIdentityError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error.as_ref()),
            _ => None,
        }
    }
}

impl From<anyhow::Error> for PortalIdentityError {
    fn from(value: anyhow::Error) -> Self {
        Self::Storage(value)
    }
}

impl std::fmt::Display for AdminIdentityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput(message) => write!(f, "{message}"),
            Self::DuplicateEmail => write!(f, "admin user already exists"),
            Self::Protected(message) => write!(f, "{message}"),
            Self::InvalidCredentials => write!(f, "invalid email or password"),
            Self::InactiveUser => write!(f, "admin user is inactive"),
            Self::NotFound(message) => write!(f, "{message}"),
            Self::Storage(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for AdminIdentityError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error.as_ref()),
            _ => None,
        }
    }
}

impl From<anyhow::Error> for AdminIdentityError {
    fn from(value: anyhow::Error) -> Self {
        Self::Storage(value)
    }
}

pub fn hash_gateway_api_key(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn issue_jwt(subject: &str, signing_secret: &str) -> Result<String> {
    let issued_at = now_epoch_secs()?;
    let claims = Claims {
        sub: subject.to_owned(),
        iss: ADMIN_JWT_ISSUER.to_owned(),
        aud: ADMIN_JWT_AUDIENCE.to_owned(),
        exp: (issued_at + ADMIN_JWT_TTL_SECS) as usize,
        iat: issued_at as usize,
    };
    Ok(encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(signing_secret.as_bytes()),
    )?)
}

pub fn verify_jwt(token: &str, signing_secret: &str) -> Result<Claims> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_audience(&[ADMIN_JWT_AUDIENCE]);
    validation.set_issuer(&[ADMIN_JWT_ISSUER]);
    Ok(decode::<Claims>(
        token,
        &DecodingKey::from_secret(signing_secret.as_bytes()),
        &validation,
    )?
    .claims)
}

pub fn verify_portal_jwt(token: &str, signing_secret: &str) -> Result<PortalClaims> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_audience(&[PORTAL_JWT_AUDIENCE]);
    validation.set_issuer(&[PORTAL_JWT_ISSUER]);
    Ok(decode::<PortalClaims>(
        token,
        &DecodingKey::from_secret(signing_secret.as_bytes()),
        &validation,
    )?
    .claims)
}

pub async fn login_admin_user(
    store: &dyn AdminStore,
    email: &str,
    password: &str,
    signing_secret: &str,
) -> AdminResult<AdminAuthSession> {
    validate_login_password(password).map_err(AdminIdentityError::InvalidInput)?;
    let _ = ensure_default_admin_user(store).await?;

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
    let _ = ensure_default_admin_user(store).await?;
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
    let record = AdminUserRecord::new(
        target_id,
        normalized_email,
        display_name.trim(),
        password_salt,
        password_hash,
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
    let _ = ensure_default_admin_user(store).await?;

    let Some(user) = store
        .find_admin_user_by_id(user_id)
        .await
        .map_err(AdminIdentityError::from)?
    else {
        return Ok(false);
    };

    if user.id == DEFAULT_ADMIN_USER_ID || user.email == DEFAULT_ADMIN_EMAIL {
        return Err(AdminIdentityError::Protected(
            "default bootstrap admin cannot be deleted".to_owned(),
        ));
    }

    store
        .delete_admin_user(&user.id)
        .await
        .map_err(AdminIdentityError::from)
}

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
            label: normalized_label,
            notes: normalized_notes,
            created_at_ms,
            expires_at_ms,
        })
    }
}

pub async fn persist_gateway_api_key(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    environment: &str,
) -> Result<CreatedGatewayApiKey> {
    persist_gateway_api_key_with_metadata(
        store,
        tenant_id,
        project_id,
        environment,
        &default_gateway_api_key_label(environment),
        None,
        None,
        None,
    )
    .await
}

pub async fn persist_gateway_api_key_with_metadata(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    environment: &str,
    label: &str,
    expires_at_ms: Option<u64>,
    plaintext_key: Option<&str>,
    notes: Option<&str>,
) -> Result<CreatedGatewayApiKey> {
    validate_gateway_api_key_metadata(label, notes, expires_at_ms)
        .map_err(|message| anyhow!(message))?;
    let created = CreateGatewayApiKey::execute_with_optional_plaintext(
        tenant_id,
        project_id,
        environment,
        label,
        expires_at_ms,
        plaintext_key,
        notes,
    )?;
    let record =
        GatewayApiKeyRecord::new(tenant_id, project_id, environment, created.hashed.clone())
            .with_label(created.label.clone())
            .with_notes_option(created.notes.clone())
            .with_created_at_ms(created.created_at_ms)
            .with_expires_at_ms_option(created.expires_at_ms);
    store.insert_gateway_api_key(&record).await?;
    Ok(created)
}

pub async fn list_gateway_api_keys(store: &dyn AdminStore) -> Result<Vec<GatewayApiKeyRecord>> {
    store.list_gateway_api_keys().await
}

pub async fn set_gateway_api_key_active(
    store: &dyn AdminStore,
    hashed_key: &str,
    active: bool,
) -> Result<Option<GatewayApiKeyRecord>> {
    let Some(existing) = store.find_gateway_api_key(hashed_key).await? else {
        return Ok(None);
    };

    let updated = GatewayApiKeyRecord { active, ..existing };

    let saved = store.insert_gateway_api_key(&updated).await?;
    Ok(Some(saved))
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
        ..record.clone()
    };
    let _ = store.insert_gateway_api_key(&updated).await?;

    Ok(Some(GatewayRequestContext {
        tenant_id: record.tenant_id,
        project_id: record.project_id,
        environment: record.environment,
    }))
}

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

pub async fn upsert_portal_user(
    store: &dyn AdminStore,
    user_id: Option<&str>,
    email: &str,
    display_name: &str,
    password: Option<&str>,
    workspace_tenant_id: &str,
    workspace_project_id: &str,
    active: bool,
) -> PortalResult<PortalUserProfile> {
    validate_identity_profile_input(email, display_name)
        .map_err(PortalIdentityError::InvalidInput)?;
    validate_workspace_scope(store, workspace_tenant_id, workspace_project_id).await?;

    let normalized_email = normalize_email(email);
    let requested_id = normalize_optional_value(user_id);
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
        match password.map(str::trim).filter(|value| !value.is_empty()) {
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
        display_name.trim(),
        password_salt,
        password_hash,
        workspace_tenant_id.trim(),
        workspace_project_id.trim(),
        active,
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
    create_portal_api_key_with_metadata(
        store,
        user_id,
        environment,
        &default_gateway_api_key_label(environment),
        None,
        None,
        None,
    )
    .await
}

pub async fn create_portal_api_key_with_metadata(
    store: &dyn AdminStore,
    user_id: &str,
    environment: &str,
    label: &str,
    expires_at_ms: Option<u64>,
    plaintext_key: Option<&str>,
    notes: Option<&str>,
) -> PortalResult<CreatedGatewayApiKey> {
    let environment = environment.trim();
    if environment.is_empty() {
        return Err(PortalIdentityError::InvalidInput(
            "environment is required".to_owned(),
        ));
    }
    validate_gateway_api_key_metadata(label, notes, expires_at_ms)
        .map_err(PortalIdentityError::InvalidInput)?;
    let user = load_portal_user_record(store, user_id).await?;

    persist_gateway_api_key_with_metadata(
        store,
        &user.workspace_tenant_id,
        &user.workspace_project_id,
        environment,
        label,
        expires_at_ms,
        plaintext_key,
        notes,
    )
    .await
    .map_err(PortalIdentityError::from)
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

async fn ensure_default_admin_user(store: &dyn AdminStore) -> AdminResult<AdminUserRecord> {
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

async fn ensure_default_portal_user(store: &dyn AdminStore) -> PortalResult<PortalUserRecord> {
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

fn admin_session_from_user(
    user: &AdminUserRecord,
    signing_secret: &str,
) -> AdminResult<AdminAuthSession> {
    let token = issue_jwt(&user.id, signing_secret).map_err(AdminIdentityError::from)?;
    Ok(AdminAuthSession {
        token,
        user: AdminUserProfile::from(user),
    })
}

fn portal_session_from_user(
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

fn issue_portal_jwt(user: &PortalUserRecord, signing_secret: &str) -> PortalResult<String> {
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

fn validate_registration_input(
    email: &str,
    password: &str,
    display_name: &str,
) -> PortalResult<()> {
    validate_identity_profile_input(email, display_name)
        .map_err(PortalIdentityError::InvalidInput)?;
    validate_password_strength(password).map_err(PortalIdentityError::InvalidInput)?;
    Ok(())
}

fn validate_identity_profile_input(
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

fn normalize_email(email: &str) -> String {
    email.trim().to_ascii_lowercase()
}

fn normalize_optional_value(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

fn generate_entity_id(prefix: &str) -> Result<String> {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| anyhow!("system clock error"))?
        .as_nanos();
    Ok(format!("{prefix}_{nonce:x}"))
}

fn hash_identity_password(password: &str, context: &str) -> Result<(String, String)> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|error| anyhow!("{context} hash failed: {error}"))?
        .to_string();
    Ok((salt.to_string(), hash))
}

fn verify_password_hash(password: &str, password_hash: &str, context: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(password_hash)
        .map_err(|error| anyhow!("{context} hash parse failed: {error}"))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

fn validate_login_password(password: &str) -> std::result::Result<(), String> {
    if password.trim().is_empty() {
        return Err("password is required".to_owned());
    }
    Ok(())
}

fn validate_current_password(password: &str) -> std::result::Result<(), String> {
    if password.trim().is_empty() {
        return Err("current_password is required".to_owned());
    }
    Ok(())
}

fn validate_password_strength(password: &str) -> std::result::Result<(), String> {
    if password.len() < 8 {
        return Err("password must be at least 8 characters".to_owned());
    }
    Ok(())
}

fn default_gateway_api_key_label(environment: &str) -> String {
    format!("{} gateway key", environment.trim())
}

fn normalize_gateway_api_key_label(label: &str) -> String {
    let trimmed = label.trim();
    if trimmed.is_empty() {
        "Gateway key".to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn normalize_gateway_api_key_plaintext(value: &str) -> std::result::Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("api_key is required when custom key mode is selected".to_owned());
    }

    Ok(trimmed.to_owned())
}

fn normalize_gateway_api_key_notes(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn validate_gateway_api_key_metadata(
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

fn map_portal_store_error(error: anyhow::Error) -> PortalIdentityError {
    if looks_like_duplicate_error(&error) {
        PortalIdentityError::DuplicateEmail
    } else {
        PortalIdentityError::Storage(error)
    }
}

fn map_admin_store_error(error: anyhow::Error) -> AdminIdentityError {
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

async fn validate_workspace_scope(
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

fn now_epoch_secs() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| anyhow!("system clock error"))?
        .as_secs())
}

fn now_epoch_millis() -> Result<u64> {
    Ok(u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| anyhow!("system clock error"))?
            .as_millis(),
    )?)
}
