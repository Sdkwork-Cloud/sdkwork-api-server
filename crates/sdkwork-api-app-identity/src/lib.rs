use anyhow::{anyhow, Result};
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rand_core::OsRng;
use sdkwork_api_domain_identity::{GatewayApiKeyRecord, PortalUserProfile, PortalUserRecord};
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
pub struct PortalWorkspaceSummary {
    pub user: PortalUserProfile,
    pub tenant: Tenant,
    pub project: Project,
}

#[derive(Debug)]
pub enum PortalIdentityError {
    InvalidInput(String),
    DuplicateEmail,
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

pub struct CreateGatewayApiKey;

impl CreateGatewayApiKey {
    pub fn execute(
        tenant_id: &str,
        project_id: &str,
        environment: &str,
    ) -> Result<CreatedGatewayApiKey> {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| anyhow!("system clock error"))?
            .as_nanos();
        let plaintext = format!("skw_{environment}_{nonce:x}");
        let hashed = hash_gateway_api_key(&plaintext);
        Ok(CreatedGatewayApiKey {
            plaintext,
            hashed,
            tenant_id: tenant_id.to_owned(),
            project_id: project_id.to_owned(),
            environment: environment.to_owned(),
        })
    }
}

pub async fn persist_gateway_api_key(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    environment: &str,
) -> Result<CreatedGatewayApiKey> {
    let created = CreateGatewayApiKey::execute(tenant_id, project_id, environment)?;
    let record =
        GatewayApiKeyRecord::new(tenant_id, project_id, environment, created.hashed.clone());
    store.insert_gateway_api_key(&record).await?;
    Ok(created)
}

pub async fn list_gateway_api_keys(store: &dyn AdminStore) -> Result<Vec<GatewayApiKeyRecord>> {
    store.list_gateway_api_keys().await
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
    let (password_salt, password_hash) = hash_portal_password(password)?;

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
    if password.trim().is_empty() {
        return Err(PortalIdentityError::InvalidInput(
            "password is required".to_owned(),
        ));
    }

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

    if !verify_portal_password(password, &user.password_hash)? {
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

pub async fn list_portal_api_keys(
    store: &dyn AdminStore,
    user_id: &str,
) -> PortalResult<Vec<GatewayApiKeyRecord>> {
    let Some(user) = store
        .find_portal_user_by_id(user_id)
        .await
        .map_err(PortalIdentityError::from)?
    else {
        return Err(PortalIdentityError::NotFound(
            "portal user not found".to_owned(),
        ));
    };

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

pub async fn create_portal_api_key(
    store: &dyn AdminStore,
    user_id: &str,
    environment: &str,
) -> PortalResult<CreatedGatewayApiKey> {
    let environment = environment.trim();
    if environment.is_empty() {
        return Err(PortalIdentityError::InvalidInput(
            "environment is required".to_owned(),
        ));
    }

    let Some(user) = store
        .find_portal_user_by_id(user_id)
        .await
        .map_err(PortalIdentityError::from)?
    else {
        return Err(PortalIdentityError::NotFound(
            "portal user not found".to_owned(),
        ));
    };

    persist_gateway_api_key(
        store,
        &user.workspace_tenant_id,
        &user.workspace_project_id,
        environment,
    )
    .await
    .map_err(PortalIdentityError::from)
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
    let normalized_email = normalize_email(email);
    if normalized_email.is_empty() || !normalized_email.contains('@') {
        return Err(PortalIdentityError::InvalidInput(
            "email must be a valid address".to_owned(),
        ));
    }
    if password.len() < 8 {
        return Err(PortalIdentityError::InvalidInput(
            "password must be at least 8 characters".to_owned(),
        ));
    }
    if display_name.trim().is_empty() {
        return Err(PortalIdentityError::InvalidInput(
            "display_name is required".to_owned(),
        ));
    }
    Ok(())
}

fn normalize_email(email: &str) -> String {
    email.trim().to_ascii_lowercase()
}

fn generate_entity_id(prefix: &str) -> Result<String> {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| anyhow!("system clock error"))?
        .as_nanos();
    Ok(format!("{prefix}_{nonce:x}"))
}

fn hash_portal_password(password: &str) -> PortalResult<(String, String)> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|error| PortalIdentityError::Storage(anyhow!("password hash failed: {error}")))?
        .to_string();
    Ok((salt.to_string(), hash))
}

fn verify_portal_password(password: &str, password_hash: &str) -> PortalResult<bool> {
    let parsed_hash = PasswordHash::new(password_hash).map_err(|error| {
        PortalIdentityError::Storage(anyhow!("password hash parse failed: {error}"))
    })?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

fn map_portal_store_error(error: anyhow::Error) -> PortalIdentityError {
    let lowered = error.to_string().to_ascii_lowercase();
    if lowered.contains("unique")
        || lowered.contains("duplicate")
        || lowered.contains("identity_users.email")
    {
        PortalIdentityError::DuplicateEmail
    } else {
        PortalIdentityError::Storage(error)
    }
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
