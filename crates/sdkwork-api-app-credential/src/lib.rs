use anyhow::Result;
use sdkwork_api_domain_credential::UpstreamCredential;
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

pub async fn list_credentials(store: &SqliteAdminStore) -> Result<Vec<UpstreamCredential>> {
    store.list_credentials().await
}
