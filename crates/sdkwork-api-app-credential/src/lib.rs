use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use sdkwork_api_domain_credential::UpstreamCredential;
use sdkwork_api_secret_core::{
    decrypt, encrypt, master_key_id, CredentialSecretRef, SecretBackendKind, SecretEnvelope,
};
use sdkwork_api_secret_keyring::{KeyringBackend, KeyringSecretStore, OsKeyringBackend};
use sdkwork_api_secret_local::LocalEncryptedFileSecretStore;
use sdkwork_api_storage_core::AdminStore;

pub fn service_name() -> &'static str {
    "credential-service"
}

#[derive(Debug, Clone)]
pub struct CredentialSecretManager {
    default_backend: SecretBackendKind,
    master_key: String,
    current_master_key_id: String,
    legacy_master_keys_by_id: HashMap<String, String>,
    fallback_master_keys: Vec<String>,
    secret_local_file: PathBuf,
    secret_keyring_service: String,
    keyring_backend: Arc<dyn KeyringBackend>,
}

impl CredentialSecretManager {
    pub fn new(
        default_backend: SecretBackendKind,
        master_key: impl Into<String>,
        secret_local_file: impl Into<PathBuf>,
        secret_keyring_service: impl Into<String>,
    ) -> Self {
        Self::new_with_legacy_master_keys(
            default_backend,
            master_key,
            Vec::new(),
            secret_local_file,
            secret_keyring_service,
        )
    }

    pub fn new_with_legacy_master_keys(
        default_backend: SecretBackendKind,
        master_key: impl Into<String>,
        legacy_master_keys: Vec<String>,
        secret_local_file: impl Into<PathBuf>,
        secret_keyring_service: impl Into<String>,
    ) -> Self {
        Self::new_with_legacy_master_keys_and_keyring_backend(
            default_backend,
            master_key,
            legacy_master_keys,
            secret_local_file,
            secret_keyring_service,
            Arc::new(OsKeyringBackend),
        )
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
        Self::new_with_legacy_master_keys_and_keyring_backend(
            SecretBackendKind::OsKeyring,
            master_key,
            Vec::new(),
            "sdkwork-api-secrets.json",
            service_name,
            backend,
        )
    }

    fn new_with_legacy_master_keys_and_keyring_backend(
        default_backend: SecretBackendKind,
        master_key: impl Into<String>,
        legacy_master_keys: Vec<String>,
        secret_local_file: impl Into<PathBuf>,
        secret_keyring_service: impl Into<String>,
        keyring_backend: Arc<dyn KeyringBackend>,
    ) -> Self {
        let master_key = master_key.into();
        let current_master_key_id = master_key_id(&master_key);
        let mut legacy_master_keys_by_id = HashMap::new();
        let mut fallback_master_keys = vec![master_key.clone()];

        for legacy_master_key in legacy_master_keys {
            let legacy_master_key_id = master_key_id(&legacy_master_key);
            if legacy_master_key_id == current_master_key_id
                || legacy_master_keys_by_id.contains_key(&legacy_master_key_id)
            {
                continue;
            }

            fallback_master_keys.push(legacy_master_key.clone());
            legacy_master_keys_by_id.insert(legacy_master_key_id, legacy_master_key);
        }

        Self {
            default_backend,
            master_key,
            current_master_key_id,
            legacy_master_keys_by_id,
            fallback_master_keys,
            secret_local_file: secret_local_file.into(),
            secret_keyring_service: secret_keyring_service.into(),
            keyring_backend,
        }
    }

    pub fn default_backend(&self) -> SecretBackendKind {
        self.default_backend
    }

    pub fn current_master_key_id(&self) -> &str {
        &self.current_master_key_id
    }

    fn secret_ref(&self, credential: &UpstreamCredential) -> CredentialSecretRef {
        CredentialSecretRef::new(
            credential.tenant_id.clone(),
            credential.provider_id.clone(),
            credential.key_reference.clone(),
        )
    }

    fn credential_with_current_metadata(
        &self,
        credential: &UpstreamCredential,
    ) -> UpstreamCredential {
        let (secret_local_file, secret_keyring_service) = match self.default_backend {
            SecretBackendKind::DatabaseEncrypted => (None, None),
            SecretBackendKind::LocalEncryptedFile => (
                Some(self.secret_local_file.to_string_lossy().into_owned()),
                None,
            ),
            SecretBackendKind::OsKeyring => (None, Some(self.secret_keyring_service.clone())),
        };

        UpstreamCredential {
            tenant_id: credential.tenant_id.clone(),
            provider_id: credential.provider_id.clone(),
            key_reference: credential.key_reference.clone(),
            secret_backend: self.default_backend.as_str().to_owned(),
            secret_local_file,
            secret_keyring_service,
            secret_master_key_id: Some(self.current_master_key_id.clone()),
        }
    }

    fn local_file_store_for(
        &self,
        credential: &UpstreamCredential,
    ) -> LocalEncryptedFileSecretStore {
        let path = credential
            .secret_local_file
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| self.secret_local_file.clone());
        LocalEncryptedFileSecretStore::new(path)
    }

    fn keyring_store_for(&self, credential: &UpstreamCredential) -> KeyringSecretStore {
        let service_name = credential
            .secret_keyring_service
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(self.secret_keyring_service.as_str());
        KeyringSecretStore::with_backend(service_name.to_owned(), self.keyring_backend.clone())
    }

    fn decrypt_envelope(
        &self,
        credential: &UpstreamCredential,
        envelope: &SecretEnvelope,
    ) -> Result<String> {
        if let Some(secret_master_key_id) = credential
            .secret_master_key_id
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            let master_key = if secret_master_key_id == self.current_master_key_id {
                Some(self.master_key.as_str())
            } else {
                self.legacy_master_keys_by_id
                    .get(secret_master_key_id)
                    .map(String::as_str)
            }
            .ok_or_else(|| anyhow!("credential master key is not configured"))?;

            return decrypt(master_key, envelope);
        }

        for master_key in &self.fallback_master_keys {
            if let Ok(secret) = decrypt(master_key, envelope) {
                return Ok(secret);
            }
        }

        Err(anyhow!(
            "credential secret could not be decrypted with configured master keys"
        ))
    }

    async fn persist_secret(
        &self,
        store: &dyn AdminStore,
        credential: &UpstreamCredential,
        secret_value: &str,
    ) -> Result<UpstreamCredential> {
        let credential = self.credential_with_current_metadata(credential);
        let envelope = encrypt(&self.master_key, secret_value)?;
        let secret_ref = self.secret_ref(&credential);

        match self.default_backend {
            SecretBackendKind::DatabaseEncrypted => {
                store
                    .insert_encrypted_credential(&credential, &envelope)
                    .await
            }
            SecretBackendKind::LocalEncryptedFile => {
                self.local_file_store_for(&credential)
                    .store_envelope(&secret_ref, &envelope)?;
                store.insert_credential(&credential).await
            }
            SecretBackendKind::OsKeyring => {
                self.keyring_store_for(&credential)
                    .store_envelope(&secret_ref, &envelope)?;
                store.insert_credential(&credential).await
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
            SecretBackendKind::LocalEncryptedFile => self
                .local_file_store_for(credential)
                .load_envelope(&secret_ref)?,
            SecretBackendKind::OsKeyring => self
                .keyring_store_for(credential)
                .load_envelope(&secret_ref)?,
        }
        .ok_or_else(|| anyhow!("credential secret not found"))?;

        self.decrypt_envelope(credential, &envelope)
    }

    fn remove_secret(&self, credential: &UpstreamCredential) -> Result<()> {
        let secret_ref = self.secret_ref(credential);
        let backend = SecretBackendKind::parse(&credential.secret_backend)?;

        match backend {
            SecretBackendKind::DatabaseEncrypted => Ok(()),
            SecretBackendKind::LocalEncryptedFile => {
                self.local_file_store_for(credential)
                    .delete_envelope(&secret_ref)?;
                Ok(())
            }
            SecretBackendKind::OsKeyring => {
                self.keyring_store_for(credential)
                    .delete_envelope(&secret_ref)?;
                Ok(())
            }
        }
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

#[async_trait]
pub trait CredentialInventoryStore: Send + Sync {
    async fn list_credentials_for_tenant(&self, tenant_id: &str)
        -> Result<Vec<UpstreamCredential>>;

    async fn list_credentials_for_provider(
        &self,
        provider_id: &str,
    ) -> Result<Vec<UpstreamCredential>>;
}

#[async_trait]
impl<T> CredentialInventoryStore for T
where
    T: AdminStore + ?Sized,
{
    async fn list_credentials_for_tenant(
        &self,
        tenant_id: &str,
    ) -> Result<Vec<UpstreamCredential>> {
        AdminStore::list_credentials_for_tenant(self, tenant_id).await
    }

    async fn list_credentials_for_provider(
        &self,
        provider_id: &str,
    ) -> Result<Vec<UpstreamCredential>> {
        AdminStore::list_credentials_for_provider(self, provider_id).await
    }
}

pub async fn delete_credential_with_manager<T>(
    store: &T,
    manager: &CredentialSecretManager,
    tenant_id: &str,
    provider_id: &str,
    key_reference: &str,
) -> Result<bool>
where
    T: AdminStore + ?Sized,
{
    let Some(credential) = store
        .find_credential(tenant_id, provider_id, key_reference)
        .await?
    else {
        return Ok(false);
    };

    delete_credential_record_with_manager(store, manager, &credential)
        .await
        .map(|_| true)
}

pub async fn delete_provider_credentials_with_manager<T>(
    store: &T,
    manager: &CredentialSecretManager,
    provider_id: &str,
) -> Result<()>
where
    T: AdminStore + CredentialInventoryStore + ?Sized,
{
    for credential in list_provider_credentials(store, provider_id).await? {
        delete_credential_record_with_manager(store, manager, &credential).await?;
    }

    Ok(())
}

pub async fn delete_tenant_credentials_with_manager<T>(
    store: &T,
    manager: &CredentialSecretManager,
    tenant_id: &str,
) -> Result<()>
where
    T: AdminStore + CredentialInventoryStore + ?Sized,
{
    for credential in list_tenant_credentials(store, tenant_id).await? {
        delete_credential_record_with_manager(store, manager, &credential).await?;
    }

    Ok(())
}

async fn delete_credential_record_with_manager<T>(
    store: &T,
    manager: &CredentialSecretManager,
    credential: &UpstreamCredential,
) -> Result<()>
where
    T: AdminStore + ?Sized,
{
    manager.remove_secret(credential)?;
    store
        .delete_credential(
            &credential.tenant_id,
            &credential.provider_id,
            &credential.key_reference,
        )
        .await?;
    Ok(())
}

async fn list_tenant_credentials<T>(store: &T, tenant_id: &str) -> Result<Vec<UpstreamCredential>>
where
    T: CredentialInventoryStore + ?Sized,
{
    store.list_credentials_for_tenant(tenant_id).await
}

async fn list_provider_credentials<T>(
    store: &T,
    provider_id: &str,
) -> Result<Vec<UpstreamCredential>>
where
    T: CredentialInventoryStore + ?Sized,
{
    store.list_credentials_for_provider(provider_id).await
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

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Mutex;

    #[tokio::test]
    async fn list_tenant_credentials_reads_only_tenant_scope() {
        let store = RecordingCredentialInventoryStore::new(vec![
            UpstreamCredential::new("tenant-1", "provider-a", "key-a"),
            UpstreamCredential::new("tenant-1", "provider-b", "key-b"),
            UpstreamCredential::new("tenant-2", "provider-a", "key-c"),
        ]);

        let credentials = list_tenant_credentials(&store, "tenant-1").await.unwrap();

        assert_eq!(credentials.len(), 2);
        assert_eq!(
            store.last_tenant_id.lock().unwrap().as_deref(),
            Some("tenant-1")
        );
    }

    #[tokio::test]
    async fn list_provider_credentials_reads_only_provider_scope() {
        let store = RecordingCredentialInventoryStore::new(vec![
            UpstreamCredential::new("tenant-1", "provider-a", "key-a"),
            UpstreamCredential::new("tenant-1", "provider-b", "key-b"),
            UpstreamCredential::new("tenant-2", "provider-a", "key-c"),
        ]);

        let credentials = list_provider_credentials(&store, "provider-a")
            .await
            .unwrap();

        assert_eq!(credentials.len(), 2);
        assert_eq!(
            store.last_provider_id.lock().unwrap().as_deref(),
            Some("provider-a")
        );
    }

    struct RecordingCredentialInventoryStore {
        credentials: Vec<UpstreamCredential>,
        last_tenant_id: Mutex<Option<String>>,
        last_provider_id: Mutex<Option<String>>,
    }

    impl RecordingCredentialInventoryStore {
        fn new(credentials: Vec<UpstreamCredential>) -> Self {
            Self {
                credentials,
                last_tenant_id: Mutex::new(None),
                last_provider_id: Mutex::new(None),
            }
        }
    }

    #[async_trait]
    impl CredentialInventoryStore for RecordingCredentialInventoryStore {
        async fn list_credentials_for_tenant(
            &self,
            tenant_id: &str,
        ) -> Result<Vec<UpstreamCredential>> {
            *self.last_tenant_id.lock().unwrap() = Some(tenant_id.to_owned());
            Ok(self
                .credentials
                .iter()
                .filter(|credential| credential.tenant_id == tenant_id)
                .cloned()
                .collect())
        }

        async fn list_credentials_for_provider(
            &self,
            provider_id: &str,
        ) -> Result<Vec<UpstreamCredential>> {
            *self.last_provider_id.lock().unwrap() = Some(provider_id.to_owned());
            Ok(self
                .credentials
                .iter()
                .filter(|credential| credential.provider_id == provider_id)
                .cloned()
                .collect())
        }
    }
}
