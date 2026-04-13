use super::*;

pub(crate) type AdminResult<T> = std::result::Result<T, AdminIdentityError>;
pub(crate) type PortalResult<T> = std::result::Result<T, PortalIdentityError>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Claims {
    pub sub: String,
    pub role: AdminUserRole,
    pub iss: String,
    pub aud: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_tenant_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_organization_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_user_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_api_key_id: Option<u64>,
}

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

    pub fn with_canonical_subject(
        mut self,
        tenant_id: u64,
        organization_id: u64,
        user_id: u64,
        api_key_id: Option<u64>,
    ) -> Self {
        self.canonical_tenant_id = Some(tenant_id);
        self.canonical_organization_id = Some(organization_id);
        self.canonical_user_id = Some(user_id);
        self.canonical_api_key_id = api_key_id;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct PortalWorkspaceScope {
    pub tenant_id: String,
    pub project_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
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

