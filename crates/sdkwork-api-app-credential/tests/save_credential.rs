use sdkwork_api_app_credential::{
    persist_official_provider_config_with_secret_and_manager,
    persist_credential_with_secret, persist_credential_with_secret_and_manager,
    resolve_credential_secret, resolve_provider_secret_with_fallback_and_manager,
    resolve_provider_secret_with_manager, save_credential,
    CredentialSecretManager,
};
use sdkwork_api_secret_core::{master_key_id, SecretBackendKind};
use sdkwork_api_secret_keyring::KeyringBackend;
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Default)]
struct MemoryKeyringBackend {
    entries: Mutex<HashMap<(String, String), String>>,
}

impl KeyringBackend for MemoryKeyringBackend {
    fn set_password(&self, service: &str, username: &str, secret: &str) -> anyhow::Result<()> {
        self.entries
            .lock()
            .unwrap()
            .insert((service.to_owned(), username.to_owned()), secret.to_owned());
        Ok(())
    }

    fn get_password(&self, service: &str, username: &str) -> anyhow::Result<Option<String>> {
        Ok(self
            .entries
            .lock()
            .unwrap()
            .get(&(service.to_owned(), username.to_owned()))
            .cloned())
    }

    fn delete_password(&self, service: &str, username: &str) -> anyhow::Result<bool> {
        Ok(self
            .entries
            .lock()
            .unwrap()
            .remove(&(service.to_owned(), username.to_owned()))
            .is_some())
    }
}

#[test]
fn saves_upstream_credential_binding() {
    let credential =
        save_credential("tenant-1", "provider-openai-official", "cred-openai").unwrap();
    assert_eq!(credential.provider_id, "provider-openai-official");
}

#[tokio::test]
async fn persists_encrypted_credential_and_resolves_plaintext_secret() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let credential = persist_credential_with_secret(
        &store,
        "local-dev-master-key",
        "tenant-1",
        "provider-openai-official",
        "cred-openai",
        "sk-upstream-openai",
    )
    .await
    .unwrap();

    assert_eq!(credential.key_reference, "cred-openai");

    let secret = resolve_credential_secret(
        &store,
        "local-dev-master-key",
        "tenant-1",
        "provider-openai-official",
        "cred-openai",
    )
    .await
    .unwrap();

    assert_eq!(secret, "sk-upstream-openai");
}

#[tokio::test]
async fn local_file_backend_persists_binding_and_resolves_plaintext_secret() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "sdkwork-api-credential-{}-{unique}.json",
        std::process::id()
    ));
    let manager = CredentialSecretManager::local_encrypted_file("local-dev-master-key", &path);

    let credential = persist_credential_with_secret_and_manager(
        &store,
        &manager,
        "tenant-1",
        "provider-openai-official",
        "cred-openai",
        "sk-upstream-openai",
    )
    .await
    .unwrap();

    assert_eq!(credential.secret_backend, "local_encrypted_file");

    let secret = resolve_provider_secret_with_manager(
        &store,
        &manager,
        "tenant-1",
        "provider-openai-official",
    )
    .await
    .unwrap();

    assert_eq!(secret.as_deref(), Some("sk-upstream-openai"));
    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn keyring_backend_persists_binding_and_resolves_plaintext_secret() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let manager = CredentialSecretManager::os_keyring_with_backend(
        "local-dev-master-key",
        "sdkwork-api-server-test",
        Arc::new(MemoryKeyringBackend::default()),
    );

    let credential = persist_credential_with_secret_and_manager(
        &store,
        &manager,
        "tenant-1",
        "provider-openai-official",
        "cred-openai",
        "sk-upstream-openai",
    )
    .await
    .unwrap();

    assert_eq!(credential.secret_backend, "os_keyring");

    let secret = resolve_provider_secret_with_manager(
        &store,
        &manager,
        "tenant-1",
        "provider-openai-official",
    )
    .await
    .unwrap();

    assert_eq!(secret.as_deref(), Some("sk-upstream-openai"));
}

#[tokio::test]
async fn resolves_historical_credentials_after_secret_manager_reconfiguration() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let initial_path = std::env::temp_dir().join(format!(
        "sdkwork-api-credential-initial-{}-{unique}.json",
        std::process::id()
    ));
    let rotated_path = std::env::temp_dir().join(format!(
        "sdkwork-api-credential-rotated-{}-{unique}.json",
        std::process::id()
    ));
    let initial_manager =
        CredentialSecretManager::local_encrypted_file("initial-master-key", &initial_path);

    let credential = persist_credential_with_secret_and_manager(
        &store,
        &initial_manager,
        "tenant-1",
        "provider-openai-official",
        "cred-openai",
        "sk-upstream-openai",
    )
    .await
    .unwrap();

    assert_eq!(
        credential.secret_local_file.as_deref(),
        Some(initial_path.to_string_lossy().as_ref())
    );
    assert_eq!(
        credential.secret_master_key_id.as_deref(),
        Some(master_key_id("initial-master-key").as_str())
    );

    let rotated_manager = CredentialSecretManager::new_with_legacy_master_keys(
        SecretBackendKind::LocalEncryptedFile,
        "rotated-master-key",
        vec!["initial-master-key".to_owned()],
        &rotated_path,
        "sdkwork-api-server",
    );

    let secret = resolve_provider_secret_with_manager(
        &store,
        &rotated_manager,
        "tenant-1",
        "provider-openai-official",
    )
    .await
    .unwrap();

    assert_eq!(secret.as_deref(), Some("sk-upstream-openai"));
    let _ = std::fs::remove_file(initial_path);
    let _ = std::fs::remove_file(rotated_path);
}

#[tokio::test]
async fn official_provider_secret_is_used_as_fallback_when_tenant_secret_is_missing() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let manager = CredentialSecretManager::database_encrypted("local-dev-master-key");

    persist_official_provider_config_with_secret_and_manager(
        &store,
        &manager,
        "provider-openai-official",
        "https://api.openai.com/v1",
        true,
        "sk-platform-openai",
    )
    .await
    .unwrap();

    let secret = resolve_provider_secret_with_fallback_and_manager(
        &store,
        &manager,
        "tenant-1",
        "provider-openai-official",
    )
    .await
    .unwrap();

    assert_eq!(secret.as_deref(), Some("sk-platform-openai"));
}

#[tokio::test]
async fn tenant_secret_wins_over_official_provider_fallback_secret() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let manager = CredentialSecretManager::database_encrypted("local-dev-master-key");

    persist_official_provider_config_with_secret_and_manager(
        &store,
        &manager,
        "provider-openai-official",
        "https://api.openai.com/v1",
        true,
        "sk-platform-openai",
    )
    .await
    .unwrap();
    persist_credential_with_secret_and_manager(
        &store,
        &manager,
        "tenant-1",
        "provider-openai-official",
        "cred-openai",
        "sk-tenant-openai",
    )
    .await
    .unwrap();

    let secret = resolve_provider_secret_with_fallback_and_manager(
        &store,
        &manager,
        "tenant-1",
        "provider-openai-official",
    )
    .await
    .unwrap();

    assert_eq!(secret.as_deref(), Some("sk-tenant-openai"));
}

#[tokio::test]
async fn disabled_official_provider_secret_does_not_count_as_fallback() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let manager = CredentialSecretManager::database_encrypted("local-dev-master-key");

    persist_official_provider_config_with_secret_and_manager(
        &store,
        &manager,
        "provider-openai-official",
        "https://api.openai.com/v1",
        false,
        "sk-platform-openai",
    )
    .await
    .unwrap();

    let secret = resolve_provider_secret_with_fallback_and_manager(
        &store,
        &manager,
        "tenant-1",
        "provider-openai-official",
    )
    .await
    .unwrap();

    assert!(secret.is_none());
}
