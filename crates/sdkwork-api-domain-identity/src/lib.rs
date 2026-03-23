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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_key: Option<String>,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    pub created_at_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_used_at_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at_ms: Option<u64>,
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
            raw_key: None,
            label: String::new(),
            notes: None,
            created_at_ms: 0,
            last_used_at_ms: None,
            expires_at_ms: None,
            active: true,
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn with_raw_key(mut self, raw_key: impl Into<String>) -> Self {
        self.raw_key = Some(raw_key.into());
        self
    }

    pub fn with_raw_key_option(mut self, raw_key: Option<String>) -> Self {
        self.raw_key = raw_key;
        self
    }

    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    pub fn with_notes_option(mut self, notes: Option<String>) -> Self {
        self.notes = notes;
        self
    }

    pub fn with_created_at_ms(mut self, created_at_ms: u64) -> Self {
        self.created_at_ms = created_at_ms;
        self
    }

    pub fn with_last_used_at_ms(mut self, last_used_at_ms: u64) -> Self {
        self.last_used_at_ms = Some(last_used_at_ms);
        self
    }

    pub fn with_last_used_at_ms_option(mut self, last_used_at_ms: Option<u64>) -> Self {
        self.last_used_at_ms = last_used_at_ms;
        self
    }

    pub fn with_expires_at_ms(mut self, expires_at_ms: u64) -> Self {
        self.expires_at_ms = Some(expires_at_ms);
        self
    }

    pub fn with_expires_at_ms_option(mut self, expires_at_ms: Option<u64>) -> Self {
        self.expires_at_ms = expires_at_ms;
        self
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
pub struct AdminUserRecord {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub password_salt: String,
    pub password_hash: String,
    pub active: bool,
    pub created_at_ms: u64,
}

impl AdminUserRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        email: impl Into<String>,
        display_name: impl Into<String>,
        password_salt: impl Into<String>,
        password_hash: impl Into<String>,
        active: bool,
        created_at_ms: u64,
    ) -> Self {
        Self {
            id: id.into(),
            email: email.into(),
            display_name: display_name.into(),
            password_salt: password_salt.into(),
            password_hash: password_hash.into(),
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdminUserProfile {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub active: bool,
    pub created_at_ms: u64,
}

impl From<&AdminUserRecord> for AdminUserProfile {
    fn from(value: &AdminUserRecord) -> Self {
        Self {
            id: value.id.clone(),
            email: value.email.clone(),
            display_name: value.display_name.clone(),
            active: value.active,
            created_at_ms: value.created_at_ms,
        }
    }
}
