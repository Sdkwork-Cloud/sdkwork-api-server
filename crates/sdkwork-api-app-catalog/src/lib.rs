use anyhow::Result;
use sdkwork_api_domain_catalog::{Channel, ModelCatalogEntry, ProxyProvider};
use sdkwork_api_storage_sqlite::SqliteAdminStore;

pub fn service_name() -> &'static str {
    "catalog-service"
}

pub fn create_channel(id: &str, name: &str) -> Result<Channel> {
    Ok(Channel::new(id, name))
}

pub fn create_provider(id: &str, channel_id: &str, display_name: &str) -> Result<ProxyProvider> {
    Ok(ProxyProvider::new(id, channel_id, display_name))
}

pub async fn persist_channel(store: &SqliteAdminStore, id: &str, name: &str) -> Result<Channel> {
    let channel = create_channel(id, name)?;
    store.insert_channel(&channel).await
}

pub async fn list_channels(store: &SqliteAdminStore) -> Result<Vec<Channel>> {
    store.list_channels().await
}

pub async fn persist_provider(
    store: &SqliteAdminStore,
    id: &str,
    channel_id: &str,
    display_name: &str,
) -> Result<ProxyProvider> {
    let provider = create_provider(id, channel_id, display_name)?;
    store.insert_provider(&provider).await
}

pub async fn list_providers(store: &SqliteAdminStore) -> Result<Vec<ProxyProvider>> {
    store.list_providers().await
}

pub fn create_model(external_name: &str, provider_id: &str) -> Result<ModelCatalogEntry> {
    Ok(ModelCatalogEntry::new(external_name, provider_id))
}

pub async fn persist_model(
    store: &SqliteAdminStore,
    external_name: &str,
    provider_id: &str,
) -> Result<ModelCatalogEntry> {
    let model = create_model(external_name, provider_id)?;
    store.insert_model(&model).await
}

pub async fn list_model_entries(store: &SqliteAdminStore) -> Result<Vec<ModelCatalogEntry>> {
    store.list_models().await
}
