use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpstreamCredential {
    pub tenant_id: String,
    pub provider_id: String,
    pub key_reference: String,
    pub secret_backend: String,
    #[serde(default)]
    pub secret_local_file: Option<String>,
    #[serde(default)]
    pub secret_keyring_service: Option<String>,
    #[serde(default)]
    pub secret_master_key_id: Option<String>,
}

impl UpstreamCredential {
    pub fn new(
        tenant_id: impl Into<String>,
        provider_id: impl Into<String>,
        key_reference: impl Into<String>,
    ) -> Self {
        Self::with_secret_backend(tenant_id, provider_id, key_reference, "database_encrypted")
    }

    pub fn with_secret_backend(
        tenant_id: impl Into<String>,
        provider_id: impl Into<String>,
        key_reference: impl Into<String>,
        secret_backend: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            provider_id: provider_id.into(),
            key_reference: key_reference.into(),
            secret_backend: secret_backend.into(),
            secret_local_file: None,
            secret_keyring_service: None,
            secret_master_key_id: None,
        }
    }

    pub fn with_secret_metadata(
        mut self,
        secret_local_file: Option<String>,
        secret_keyring_service: Option<String>,
        secret_master_key_id: Option<String>,
    ) -> Self {
        self.secret_local_file = secret_local_file;
        self.secret_keyring_service = secret_keyring_service;
        self.secret_master_key_id = secret_master_key_id;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OfficialProviderConfig {
    pub provider_id: String,
    pub key_reference: String,
    pub base_url: String,
    pub enabled: bool,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl OfficialProviderConfig {
    pub fn new(
        provider_id: impl Into<String>,
        key_reference: impl Into<String>,
        base_url: impl Into<String>,
        enabled: bool,
    ) -> Self {
        Self {
            provider_id: provider_id.into(),
            key_reference: key_reference.into(),
            base_url: base_url.into(),
            enabled,
            created_at_ms: 0,
            updated_at_ms: 0,
        }
    }

    pub fn with_timestamps(mut self, created_at_ms: u64, updated_at_ms: u64) -> Self {
        self.created_at_ms = created_at_ms;
        self.updated_at_ms = updated_at_ms;
        self
    }
}
