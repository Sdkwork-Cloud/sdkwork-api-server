use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use sdkwork_api_domain_credential::UpstreamCredential;
use sdkwork_api_secret_core::{decrypt, encrypt, CredentialSecretRef, SecretBackendKind};
use sdkwork_api_secret_keyring::{KeyringBackend, KeyringSecretStore};
use sdkwork_api_secret_local::LocalEncryptedFileSecretStore;
use sdkwork_api_storage_core::AdminStore;

pub fn service_name() -> &'static str {
    "credential-service"
}

#[derive(Debug, Clone)]
pub struct CredentialSecretManager {
    default_backend: SecretBackendKind,
    master_key: String,
    local_file_store: LocalEncryptedFileSecretStore,
    keyring_store: KeyringSecretStore,
}

impl CredentialSecretManager {
    pub fn new(
        default_backend: SecretBackendKind,
        master_key: impl Into<String>,
        secret_local_file: impl Into<PathBuf>,
        secret_keyring_service: impl Into<String>,
    ) -> Self {
        Self {
            default_backend,
            master_key: master_key.into(),
            local_file_store: LocalEncryptedFileSecretStore::new(secret_local_file),
            keyring_store: KeyringSecretStore::new(secret_keyring_service),
        }
    }

    pub fn database_encrypted(master_key: impl Into<String>) -> Self {
        Self::new(
            SecretBackendKind::DatabaseEncrypted,
            master_key,
            "sdkwork-api-secrets.json",
            "sdkwork-api-server",
        )
    }

    pub fn local_encrypted_file(master_key: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        Self::new(
            SecretBackendKind::LocalEncryptedFile,
            master_key,
            path,
            "sdkwork-api-server",
        )
    }

    pub fn os_keyring(master_key: impl Into<String>, service_name: impl Into<String>) -> Self {
        Self::new(
            SecretBackendKind::OsKeyring,
            master_key,
            "sdkwork-api-secrets.json",
            service_name,
        )
    }

    pub fn os_keyring_with_backend(
        master_key: impl Into<String>,
        service_name: impl Into<String>,
        backend: Arc<dyn KeyringBackend>,
    ) -> Self {
        Self {
            default_backend: SecretBackendKind::OsKeyring,
            master_key: master_key.into(),
            local_file_store: LocalEncryptedFileSecretStore::new("sdkwork-api-secrets.json"),
            keyring_store: KeyringSecretStore::with_backend(service_name, backend),
        }
    }

    pub fn default_backend(&self) -> SecretBackendKind {
        self.default_backend
    }

    fn secret_ref(&self, credential: &UpstreamCredential) -> CredentialSecretRef {
        CredentialSecretRef::new(
            credential.tenant_id.clone(),
            credential.provider_id.clone(),
            credential.key_reference.clone(),
        )
    }

    async fn persist_secret(
        &self,
        store: &dyn AdminStore,
        credential: &UpstreamCredential,
        secret_value: &str,
    ) -> Result<UpstreamCredential> {
        let envelope = encrypt(&self.master_key, secret_value)?;
        let secret_ref = self.secret_ref(credential);

        match self.default_backend {
            SecretBackendKind::DatabaseEncrypted => {
                store
                    .insert_encrypted_credential(credential, &envelope)
                    .await
            }
            SecretBackendKind::LocalEncryptedFile => {
                self.local_file_store
                    .store_envelope(&secret_ref, &envelope)?;
                store.insert_credential(credential).await
            }
            SecretBackendKind::OsKeyring => {
                self.keyring_store.store_envelope(&secret_ref, &envelope)?;
                store.insert_credential(credential).await
            }
        }
    }

    async fn resolve_secret(
        &self,
        store: &dyn AdminStore,
        credential: &UpstreamCredential,
    ) -> Result<String> {
        let secret_ref = self.secret_ref(credential);
        let backend = SecretBackendKind::parse(&credential.secret_backend)?;

        let envelope = match backend {
            SecretBackendKind::DatabaseEncrypted => {
                store
                    .find_credential_envelope(
                        &credential.tenant_id,
                        &credential.provider_id,
                        &credential.key_reference,
                    )
                    .await?
            }
            SecretBackendKind::LocalEncryptedFile => {
                self.local_file_store.load_envelope(&secret_ref)?
            }
            SecretBackendKind::OsKeyring => self.keyring_store.load_envelope(&secret_ref)?,
        }
        .ok_or_else(|| anyhow!("credential secret not found"))?;

        decrypt(&self.master_key, &envelope)
    }
}

pub fn save_credential(
    tenant_id: &str,
    provider_id: &str,
    key_reference: &str,
) -> Result<UpstreamCredential> {
    Ok(UpstreamCredential::new(
        tenant_id,
        provider_id,
        key_reference,
    ))
}

pub fn save_credential_with_backend(
    tenant_id: &str,
    provider_id: &str,
    key_reference: &str,
    secret_backend: SecretBackendKind,
) -> Result<UpstreamCredential> {
    Ok(UpstreamCredential::with_secret_backend(
        tenant_id,
        provider_id,
        key_reference,
        secret_backend.as_str(),
    ))
}

pub async fn persist_credential(
    store: &dyn AdminStore,
    tenant_id: &str,
    provider_id: &str,
    key_reference: &str,
) -> Result<UpstreamCredential> {
    let credential = save_credential(tenant_id, provider_id, key_reference)?;
    store.insert_credential(&credential).await
}

pub async fn persist_credential_with_secret(
    store: &dyn AdminStore,
    master_key: &str,
    tenant_id: &str,
    provider_id: &str,
    key_reference: &str,
    secret_value: &str,
) -> Result<UpstreamCredential> {
    let manager = CredentialSecretManager::database_encrypted(master_key);
    persist_credential_with_secret_and_manager(
        store,
        &manager,
        tenant_id,
        provider_id,
        key_reference,
        secret_value,
    )
    .await
}

pub async fn persist_credential_with_secret_and_manager(
    store: &dyn AdminStore,
    manager: &CredentialSecretManager,
    tenant_id: &str,
    provider_id: &str,
    key_reference: &str,
    secret_value: &str,
) -> Result<UpstreamCredential> {
    let credential = save_credential_with_backend(
        tenant_id,
        provider_id,
        key_reference,
        manager.default_backend(),
    )?;
    manager
        .persist_secret(store, &credential, secret_value)
        .await
}

pub async fn list_credentials(store: &dyn AdminStore) -> Result<Vec<UpstreamCredential>> {
    store.list_credentials().await
}

pub async fn resolve_credential_secret(
    store: &dyn AdminStore,
    master_key: &str,
    tenant_id: &str,
    provider_id: &str,
    key_reference: &str,
) -> Result<String> {
    let manager = CredentialSecretManager::database_encrypted(master_key);
    resolve_credential_secret_with_manager(store, &manager, tenant_id, provider_id, key_reference)
        .await
}

pub async fn resolve_credential_secret_with_manager(
    store: &dyn AdminStore,
    manager: &CredentialSecretManager,
    tenant_id: &str,
    provider_id: &str,
    key_reference: &str,
) -> Result<String> {
    let credential = store
        .find_credential(tenant_id, provider_id, key_reference)
        .await?
        .ok_or_else(|| anyhow!("credential secret not found"))?;
    manager.resolve_secret(store, &credential).await
}

pub async fn resolve_provider_secret(
    store: &dyn AdminStore,
    master_key: &str,
    tenant_id: &str,
    provider_id: &str,
) -> Result<Option<String>> {
    let manager = CredentialSecretManager::database_encrypted(master_key);
    resolve_provider_secret_with_manager(store, &manager, tenant_id, provider_id).await
}

pub async fn resolve_provider_secret_with_manager(
    store: &dyn AdminStore,
    manager: &CredentialSecretManager,
    tenant_id: &str,
    provider_id: &str,
) -> Result<Option<String>> {
    let Some(credential) = store
        .find_provider_credential(tenant_id, provider_id)
        .await?
    else {
        return Ok(None);
    };

    let secret = manager.resolve_secret(store, &credential).await?;
    Ok(Some(secret))
}
