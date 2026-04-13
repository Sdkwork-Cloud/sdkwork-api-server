use anyhow::{anyhow, Result};
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rand_core::OsRng;
use sdkwork_api_domain_billing::BillingAccountingMode;
use sdkwork_api_domain_identity::{
    AdminUserProfile, AdminUserRecord, AdminUserRole, ApiKeyGroupRecord, GatewayApiKeyRecord,
    GatewayAuthSubject, PortalUserProfile, PortalUserRecord, PortalWorkspaceMembershipRecord,
};
use sdkwork_api_domain_tenant::{Project, Tenant};
use sdkwork_api_storage_core::{AdminStore, IdentityKernelStore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::str::FromStr;
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
    pub api_key_hash: String,
    pub api_key_group_id: Option<String>,
}

const CANONICAL_GATEWAY_COMPAT_ENVIRONMENT: &str = "live";

impl GatewayRequestContext {
    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    pub fn project_id(&self) -> &str {
        &self.project_id
    }

    pub fn api_key_hash(&self) -> &str {
        &self.api_key_hash
    }

    pub fn api_key_group_id(&self) -> Option<&str> {
        self.api_key_group_id.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreatedGatewayApiKey {
    pub plaintext: String,
    pub hashed: String,
    pub tenant_id: String,
    pub project_id: String,
    pub environment: String,
    pub api_key_group_id: Option<String>,
    pub label: String,
    pub notes: Option<String>,
    pub created_at_ms: u64,
    pub expires_at_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiKeyGroupInput {
    pub tenant_id: String,
    pub project_id: String,
    pub environment: String,
    pub name: String,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub default_capability_scope: Option<String>,
    #[serde(default)]
    pub default_routing_profile_id: Option<String>,
    #[serde(default)]
    pub default_accounting_mode: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortalApiKeyGroupInput {
    pub environment: String,
    pub name: String,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub default_capability_scope: Option<String>,
    #[serde(default)]
    pub default_routing_profile_id: Option<String>,
    #[serde(default)]
    pub default_accounting_mode: Option<String>,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortalWorkspaceMembershipSummary {
    pub membership_id: String,
    pub role: String,
    pub current: bool,
    pub created_at_ms: u64,
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

pub fn gateway_api_key_prefix(value: &str) -> String {
    value.chars().take(16).collect()
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
    login_admin_user_with_bootstrap(store, email, password, signing_secret, true).await
}

pub async fn login_admin_user_with_bootstrap(
    store: &dyn AdminStore,
    email: &str,
    password: &str,
    signing_secret: &str,
    allow_default_bootstrap: bool,
) -> AdminResult<AdminAuthSession> {
    validate_login_password(password).map_err(AdminIdentityError::InvalidInput)?;
    let _ = ensure_default_admin_user_if_allowed(store, allow_default_bootstrap).await?;

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
    list_admin_user_profiles_with_bootstrap(store, true).await
}

pub async fn list_admin_user_profiles_with_bootstrap(
    store: &dyn AdminStore,
    allow_default_bootstrap: bool,
) -> AdminResult<Vec<AdminUserProfile>> {
    let _ = ensure_default_admin_user_if_allowed(store, allow_default_bootstrap).await?;
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
    role: Option<&str>,
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
    let role = resolve_admin_user_role(role, existing_by_id.as_ref())
        .map_err(AdminIdentityError::InvalidInput)?;
    let record = AdminUserRecord::new(
        target_id,
        normalized_email,
        display_name.trim(),
        password_salt,
        password_hash,
        role,
        active,
        created_at_ms,
    );
    let saved = store
        .insert_admin_user(&record)
        .await
        .map_err(map_admin_store_error)?;
    Ok(AdminUserProfile::from(&saved))
}

fn resolve_admin_user_role(
    requested_role: Option<&str>,
    existing_user: Option<&AdminUserRecord>,
) -> std::result::Result<AdminUserRole, String> {
    match normalize_optional_value(requested_role) {
        Some(role) => AdminUserRole::from_str(role),
        None => existing_user
            .map(|user| user.role)
            .or(Some(AdminUserRole::PlatformOperator))
            .ok_or_else(|| "admin role is required".to_owned()),
    }
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
    delete_admin_user_with_bootstrap(store, user_id, true).await
}

pub async fn delete_admin_user_with_bootstrap(
    store: &dyn AdminStore,
    user_id: &str,
    allow_default_bootstrap: bool,
) -> AdminResult<bool> {
    let _ = ensure_default_admin_user_if_allowed(store, allow_default_bootstrap).await?;

    let Some(user) = store
        .find_admin_user_by_id(user_id)
        .await
        .map_err(AdminIdentityError::from)?
    else {
        return Ok(false);
    };

    if allow_default_bootstrap
        && (user.id == DEFAULT_ADMIN_USER_ID || user.email == DEFAULT_ADMIN_EMAIL)
    {
        return Err(AdminIdentityError::Protected(
            "default bootstrap admin cannot be deleted".to_owned(),
        ));
    }

    store
        .delete_admin_user(&user.id)
        .await
        .map_err(AdminIdentityError::from)
}

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
    api_key_group_id: Option<&str>,
) -> Result<CreatedGatewayApiKey> {
    validate_gateway_api_key_metadata(label, notes, expires_at_ms)
        .map_err(|message| anyhow!(message))?;
    let validated_group_id = validate_gateway_api_key_group_assignment(
        store,
        tenant_id,
        project_id,
        environment,
        api_key_group_id,
    )
    .await?;
    let mut created = CreateGatewayApiKey::execute_with_optional_plaintext(
        tenant_id,
        project_id,
        environment,
        label,
        expires_at_ms,
        plaintext_key,
        notes,
    )?;
    created.api_key_group_id = validated_group_id.clone();
    let record =
        GatewayApiKeyRecord::new(tenant_id, project_id, environment, created.hashed.clone())
            .with_key_prefix(gateway_api_key_prefix(&created.plaintext))
            .with_api_key_group_id_option(validated_group_id)
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

pub async fn update_gateway_api_key_metadata(
    store: &dyn AdminStore,
    hashed_key: &str,
    tenant_id: &str,
    project_id: &str,
    environment: &str,
    label: &str,
    expires_at_ms: Option<u64>,
    notes: Option<&str>,
    api_key_group_id: Option<&str>,
) -> AdminResult<Option<GatewayApiKeyRecord>> {
    let normalized_tenant_id = tenant_id.trim();
    if normalized_tenant_id.is_empty() {
        return Err(AdminIdentityError::InvalidInput(
            "tenant_id is required".to_owned(),
        ));
    }

    let normalized_project_id = project_id.trim();
    if normalized_project_id.is_empty() {
        return Err(AdminIdentityError::InvalidInput(
            "project_id is required".to_owned(),
        ));
    }

    let normalized_environment = environment.trim();
    if normalized_environment.is_empty() {
        return Err(AdminIdentityError::InvalidInput(
            "environment is required".to_owned(),
        ));
    }

    validate_gateway_api_key_metadata(label, notes, expires_at_ms)
        .map_err(AdminIdentityError::InvalidInput)?;
    let validated_group_id = validate_gateway_api_key_group_assignment(
        store,
        normalized_tenant_id,
        normalized_project_id,
        normalized_environment,
        api_key_group_id,
    )
    .await
    .map_err(|error| AdminIdentityError::InvalidInput(error.to_string()))?;

    let Some(existing) = store
        .find_gateway_api_key(hashed_key)
        .await
        .map_err(AdminIdentityError::from)?
    else {
        return Ok(None);
    };

    let updated = GatewayApiKeyRecord {
        tenant_id: normalized_tenant_id.to_owned(),
        project_id: normalized_project_id.to_owned(),
        environment: normalized_environment.to_owned(),
        api_key_group_id: validated_group_id,
        label: normalize_gateway_api_key_label(label),
        notes: normalize_gateway_api_key_notes(notes),
        expires_at_ms,
        ..existing
    };

    let saved = store
        .insert_gateway_api_key(&updated)
        .await
        .map_err(AdminIdentityError::from)?;
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
        api_key_hash: hashed_key,
        api_key_group_id: record.api_key_group_id,
    }))
}

pub async fn resolve_gateway_request_context_from_auth_subject(
    store: &dyn AdminStore,
    subject: &GatewayAuthSubject,
) -> Result<Option<GatewayRequestContext>> {
    let tenant_id = subject.tenant_id.to_string();
    let Some(_tenant) = store.find_tenant(&tenant_id).await? else {
        return Ok(None);
    };

    let projects = store.list_projects().await?;
    let Some(project) = select_gateway_compat_project(&projects, &tenant_id) else {
        return Ok(None);
    };

    Ok(Some(GatewayRequestContext {
        tenant_id,
        project_id: project.id.clone(),
        environment: CANONICAL_GATEWAY_COMPAT_ENVIRONMENT.to_owned(),
        api_key_hash: subject
            .api_key_hash
            .clone()
            .unwrap_or_else(|| subject.request_principal.clone()),
        api_key_group_id: None,
    }))
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

fn select_gateway_compat_project<'a>(
    projects: &'a [Project],
    tenant_id: &str,
) -> Option<&'a Project> {
    let candidates: Vec<&Project> = projects
        .iter()
        .filter(|project| project.tenant_id == tenant_id)
        .collect();
    match candidates.as_slice() {
        [] => None,
        [project] => Some(*project),
        _ => {
            let default_candidates: Vec<&Project> = candidates
                .into_iter()
                .filter(|project| {
                    project.id.eq_ignore_ascii_case("default")
                        || project.name.eq_ignore_ascii_case("default")
                })
                .collect();
            match default_candidates.as_slice() {
                [project] => Some(*project),
                _ => None,
            }
        }
    }
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
    let user = ensure_portal_workspace_membership(
        store,
        &user,
        &user.workspace_tenant_id,
        &user.workspace_project_id,
        "owner",
    )
    .await?;
    portal_session_from_user(&user, signing_secret)
}

pub async fn login_portal_user(
    store: &dyn AdminStore,
    email: &str,
    password: &str,
    signing_secret: &str,
) -> PortalResult<PortalAuthSession> {
    login_portal_user_with_bootstrap(store, email, password, signing_secret, true).await
}

pub async fn login_portal_user_with_bootstrap(
    store: &dyn AdminStore,
    email: &str,
    password: &str,
    signing_secret: &str,
    allow_default_bootstrap: bool,
) -> PortalResult<PortalAuthSession> {
    validate_login_password(password).map_err(PortalIdentityError::InvalidInput)?;
    let _ = ensure_default_portal_user_if_allowed(store, allow_default_bootstrap).await?;

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
    list_portal_user_profiles_with_bootstrap(store, true).await
}

pub async fn list_portal_user_profiles_with_bootstrap(
    store: &dyn AdminStore,
    allow_default_bootstrap: bool,
) -> PortalResult<Vec<PortalUserProfile>> {
    let _ = ensure_default_portal_user_if_allowed(store, allow_default_bootstrap).await?;
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
    let saved = ensure_portal_workspace_membership(
        store,
        &saved,
        workspace_tenant_id.trim(),
        workspace_project_id.trim(),
        "member",
    )
    .await?;
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
    delete_portal_user_with_bootstrap(store, user_id, true).await
}

pub async fn delete_portal_user_with_bootstrap(
    store: &dyn AdminStore,
    user_id: &str,
    allow_default_bootstrap: bool,
) -> PortalResult<bool> {
    let _ = ensure_default_portal_user_if_allowed(store, allow_default_bootstrap).await?;

    let Some(user) = store
        .find_portal_user_by_id(user_id)
        .await
        .map_err(PortalIdentityError::from)?
    else {
        return Ok(false);
    };

    if allow_default_bootstrap
        && (user.id == DEFAULT_PORTAL_USER_ID || user.email == DEFAULT_PORTAL_EMAIL)
    {
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
    let user = ensure_portal_workspace_membership_seeded(store, user).await?;

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

pub async fn list_portal_workspace_memberships(
    store: &dyn AdminStore,
    user_id: &str,
) -> PortalResult<Vec<PortalWorkspaceMembershipSummary>> {
    let user = load_portal_user_record(store, user_id).await?;
    let memberships = load_portal_workspace_memberships_for_user(store, &user).await?;
    let mut summaries = Vec::with_capacity(memberships.len());
    for membership in memberships {
        let tenant = store
            .find_tenant(&membership.tenant_id)
            .await
            .map_err(PortalIdentityError::from)?
            .ok_or_else(|| {
                PortalIdentityError::NotFound("portal workspace tenant not found".to_owned())
            })?;
        let project = store
            .find_project(&membership.project_id)
            .await
            .map_err(PortalIdentityError::from)?
            .ok_or_else(|| {
                PortalIdentityError::NotFound("portal workspace project not found".to_owned())
            })?;
        summaries.push(PortalWorkspaceMembershipSummary {
            membership_id: membership.membership_id,
            role: membership.role,
            current: membership.tenant_id == user.workspace_tenant_id
                && membership.project_id == user.workspace_project_id,
            created_at_ms: membership.created_at_ms,
            tenant,
            project,
        });
    }
    summaries.sort_by(|left, right| {
        right
            .current
            .cmp(&left.current)
            .then_with(|| left.tenant.id.cmp(&right.tenant.id))
            .then_with(|| left.project.id.cmp(&right.project.id))
            .then_with(|| left.membership_id.cmp(&right.membership_id))
    });
    Ok(summaries)
}

pub async fn select_portal_workspace(
    store: &dyn AdminStore,
    user_id: &str,
    workspace_tenant_id: &str,
    workspace_project_id: &str,
    signing_secret: &str,
) -> PortalResult<PortalAuthSession> {
    validate_workspace_scope(store, workspace_tenant_id, workspace_project_id).await?;
    let user = load_portal_user_record(store, user_id).await?;
    let _ = load_portal_workspace_memberships_for_user(store, &user).await?;
    let workspace_tenant_id = workspace_tenant_id.trim();
    let workspace_project_id = workspace_project_id.trim();
    if store
        .find_portal_workspace_membership(&user.id, workspace_tenant_id, workspace_project_id)
        .await
        .map_err(PortalIdentityError::from)?
        .is_none()
    {
        return Err(PortalIdentityError::NotFound(
            "portal workspace membership not found".to_owned(),
        ));
    }

    let updated = PortalUserRecord::new(
        user.id,
        user.email,
        user.display_name,
        user.password_salt,
        user.password_hash,
        workspace_tenant_id,
        workspace_project_id,
        user.active,
        user.created_at_ms,
    );
    let saved = store
        .insert_portal_user(&updated)
        .await
        .map_err(PortalIdentityError::from)?;
    portal_session_from_user(&saved, signing_secret)
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

async fn load_portal_workspace_memberships_for_user(
    store: &dyn AdminStore,
    user: &PortalUserRecord,
) -> PortalResult<Vec<PortalWorkspaceMembershipRecord>> {
    let memberships = store
        .list_portal_workspace_memberships_for_user(&user.id)
        .await
        .map_err(PortalIdentityError::from)?;
    if !memberships.is_empty() {
        return Ok(memberships);
    }

    let _ = ensure_portal_workspace_membership(
        store,
        user,
        &user.workspace_tenant_id,
        &user.workspace_project_id,
        "owner",
    )
    .await?;
    store
        .list_portal_workspace_memberships_for_user(&user.id)
        .await
        .map_err(PortalIdentityError::from)
}

async fn ensure_portal_workspace_membership_seeded(
    store: &dyn AdminStore,
    user: PortalUserRecord,
) -> PortalResult<PortalUserRecord> {
    let _ = load_portal_workspace_memberships_for_user(store, &user).await?;
    Ok(user)
}

async fn ensure_portal_workspace_membership(
    store: &dyn AdminStore,
    user: &PortalUserRecord,
    tenant_id: &str,
    project_id: &str,
    role: &str,
) -> PortalResult<PortalUserRecord> {
    let tenant_id = tenant_id.trim();
    let project_id = project_id.trim();
    if store
        .find_portal_workspace_membership(&user.id, tenant_id, project_id)
        .await
        .map_err(PortalIdentityError::from)?
        .is_none()
    {
        let membership = PortalWorkspaceMembershipRecord::new(
            generate_entity_id("membership").map_err(PortalIdentityError::from)?,
            &user.id,
            tenant_id,
            project_id,
            user.created_at_ms,
        )
        .with_role(role);
        match store.insert_portal_workspace_membership(&membership).await {
            Ok(_) => {}
            Err(error) => {
                if !looks_like_duplicate_error(&error) {
                    return Err(PortalIdentityError::Storage(error));
                }
            }
        }
    }
    Ok(user.clone())
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
    create_portal_api_key_with_metadata(
        store,
        user_id,
        environment,
        &default_gateway_api_key_label(environment),
        None,
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
    api_key_group_id: Option<&str>,
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
        api_key_group_id,
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
        AdminUserRole::SuperAdmin,
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

async fn ensure_default_admin_user_if_allowed(
    store: &dyn AdminStore,
    allow_default_bootstrap: bool,
) -> AdminResult<Option<AdminUserRecord>> {
    if allow_default_bootstrap {
        ensure_default_admin_user(store).await.map(Some)
    } else {
        Ok(None)
    }
}

async fn ensure_default_portal_user(store: &dyn AdminStore) -> PortalResult<PortalUserRecord> {
    if let Some(user) = store
        .find_portal_user_by_email(DEFAULT_PORTAL_EMAIL)
        .await
        .map_err(PortalIdentityError::from)?
    {
        return ensure_portal_workspace_membership(
            store,
            &user,
            &user.workspace_tenant_id,
            &user.workspace_project_id,
            "owner",
        )
        .await;
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
        Ok(saved) => {
            ensure_portal_workspace_membership(
                store,
                &saved,
                &saved.workspace_tenant_id,
                &saved.workspace_project_id,
                "owner",
            )
            .await
        }
        Err(error) => {
            if looks_like_duplicate_error(&error) {
                let user = store
                    .find_portal_user_by_email(DEFAULT_PORTAL_EMAIL)
                    .await
                    .map_err(PortalIdentityError::from)?
                    .ok_or_else(|| PortalIdentityError::Storage(error))?;
                ensure_portal_workspace_membership(
                    store,
                    &user,
                    &user.workspace_tenant_id,
                    &user.workspace_project_id,
                    "owner",
                )
                .await
            } else {
                Err(map_portal_store_error(error))
            }
        }
    }
}

async fn ensure_default_portal_user_if_allowed(
    store: &dyn AdminStore,
    allow_default_bootstrap: bool,
) -> PortalResult<Option<PortalUserRecord>> {
    if allow_default_bootstrap {
        ensure_default_portal_user(store).await.map(Some)
    } else {
        Ok(None)
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

fn require_trimmed_input<'a>(value: &'a str, field_name: &str) -> AdminResult<&'a str> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(AdminIdentityError::InvalidInput(format!(
            "{field_name} is required"
        )));
    }
    Ok(normalized)
}

fn normalize_api_key_group_name(name: &str) -> AdminResult<String> {
    let normalized = name.trim();
    if normalized.is_empty() {
        return Err(AdminIdentityError::InvalidInput(
            "name is required".to_owned(),
        ));
    }
    Ok(normalized.to_owned())
}

fn normalize_api_key_group_slug(name: &str, slug: Option<&str>) -> AdminResult<String> {
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

fn normalize_api_key_group_optional_value(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn validate_default_accounting_mode_binding(
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

async fn ensure_api_key_group_slug_available(
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

async fn validate_default_routing_profile_binding(
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

async fn validate_gateway_api_key_group_assignment(
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

fn map_admin_identity_error_to_portal(error: AdminIdentityError) -> PortalIdentityError {
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
