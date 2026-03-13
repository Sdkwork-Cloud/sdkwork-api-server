use anyhow::Result;
use sdkwork_api_domain_credential::UpstreamCredential;
use sdkwork_api_secret_core::{decrypt, encrypt};
use sdkwork_api_storage_sqlite::SqliteAdminStore;

pub fn service_name() -> &'static str {
    "credential-service"
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

pub async fn persist_credential(
    store: &SqliteAdminStore,
    tenant_id: &str,
    provider_id: &str,
    key_reference: &str,
) -> Result<UpstreamCredential> {
    let credential = save_credential(tenant_id, provider_id, key_reference)?;
    store.insert_credential(&credential).await
}

pub async fn persist_credential_with_secret(
    store: &SqliteAdminStore,
    master_key: &str,
    tenant_id: &str,
    provider_id: &str,
    key_reference: &str,
    secret_value: &str,
) -> Result<UpstreamCredential> {
    let credential = save_credential(tenant_id, provider_id, key_reference)?;
    let envelope = encrypt(master_key, secret_value)?;
    store
        .insert_encrypted_credential(&credential, &envelope)
        .await
}

pub async fn list_credentials(store: &SqliteAdminStore) -> Result<Vec<UpstreamCredential>> {
    store.list_credentials().await
}

pub async fn resolve_credential_secret(
    store: &SqliteAdminStore,
    master_key: &str,
    tenant_id: &str,
    provider_id: &str,
    key_reference: &str,
) -> Result<String> {
    let envelope = store
        .find_credential_envelope(tenant_id, provider_id, key_reference)
        .await?
        .ok_or_else(|| anyhow::anyhow!("credential secret not found"))?;
    decrypt(master_key, &envelope)
}
