use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayApiKey {
    pub tenant_id: String,
    pub project_id: String,
    pub environment: String,
    revoked: bool,
}

impl GatewayApiKey {
    pub fn new(
        tenant_id: impl Into<String>,
        project_id: impl Into<String>,
        environment: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            project_id: project_id.into(),
            environment: environment.into(),
            revoked: false,
        }
    }

    pub fn revoke(&mut self) {
        self.revoked = true;
    }

    pub fn is_active(&self) -> bool {
        !self.revoked
    }
}

pub trait GatewayApiKeyRepository: Send + Sync {
    fn save(&self, key: &GatewayApiKey) -> Result<(), String>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GatewayApiKeyRecord {
    pub tenant_id: String,
    pub project_id: String,
    pub environment: String,
    pub hashed_key: String,
    pub active: bool,
}

impl GatewayApiKeyRecord {
    pub fn new(
        tenant_id: impl Into<String>,
        project_id: impl Into<String>,
        environment: impl Into<String>,
        hashed_key: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            project_id: project_id.into(),
            environment: environment.into(),
            hashed_key: hashed_key.into(),
            active: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortalUserRecord {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub password_salt: String,
    pub password_hash: String,
    pub workspace_tenant_id: String,
    pub workspace_project_id: String,
    pub active: bool,
    pub created_at_ms: u64,
}

impl PortalUserRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        email: impl Into<String>,
        display_name: impl Into<String>,
        password_salt: impl Into<String>,
        password_hash: impl Into<String>,
        workspace_tenant_id: impl Into<String>,
        workspace_project_id: impl Into<String>,
        active: bool,
        created_at_ms: u64,
    ) -> Self {
        Self {
            id: id.into(),
            email: email.into(),
            display_name: display_name.into(),
            password_salt: password_salt.into(),
            password_hash: password_hash.into(),
            workspace_tenant_id: workspace_tenant_id.into(),
            workspace_project_id: workspace_project_id.into(),
            active,
            created_at_ms,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortalUserProfile {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub workspace_tenant_id: String,
    pub workspace_project_id: String,
    pub active: bool,
    pub created_at_ms: u64,
}

impl From<&PortalUserRecord> for PortalUserProfile {
    fn from(value: &PortalUserRecord) -> Self {
        Self {
            id: value.id.clone(),
            email: value.email.clone(),
            display_name: value.display_name.clone(),
            workspace_tenant_id: value.workspace_tenant_id.clone(),
            workspace_project_id: value.workspace_project_id.clone(),
            active: value.active,
            created_at_ms: value.created_at_ms,
        }
    }
}
