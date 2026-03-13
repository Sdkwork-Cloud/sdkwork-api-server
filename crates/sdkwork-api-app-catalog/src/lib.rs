use anyhow::Result;
use sdkwork_api_domain_catalog::{Channel, ModelCatalogEntry, ProxyProvider};
use sdkwork_api_storage_core::AdminStore;

pub fn service_name() -> &'static str {
    "catalog-service"
}

pub fn create_channel(id: &str, name: &str) -> Result<Channel> {
    Ok(Channel::new(id, name))
}

pub fn create_provider(id: &str, channel_id: &str, display_name: &str) -> Result<ProxyProvider> {
    Ok(ProxyProvider::new(
        id,
        channel_id,
        channel_id,
        "http://localhost",
        display_name,
    ))
}

pub async fn persist_channel(store: &dyn AdminStore, id: &str, name: &str) -> Result<Channel> {
    let channel = create_channel(id, name)?;
    store.insert_channel(&channel).await
}

pub async fn list_channels(store: &dyn AdminStore) -> Result<Vec<Channel>> {
    store.list_channels().await
}

pub fn create_provider_with_config(
    id: &str,
    channel_id: &str,
    adapter_kind: &str,
    base_url: &str,
    display_name: &str,
) -> Result<ProxyProvider> {
    Ok(ProxyProvider::new(
        id,
        channel_id,
        adapter_kind,
        base_url,
        display_name,
    ))
}

pub async fn persist_provider(
    store: &dyn AdminStore,
    id: &str,
    channel_id: &str,
    adapter_kind: &str,
    base_url: &str,
    display_name: &str,
) -> Result<ProxyProvider> {
    let provider =
        create_provider_with_config(id, channel_id, adapter_kind, base_url, display_name)?;
    store.insert_provider(&provider).await
}

pub async fn list_providers(store: &dyn AdminStore) -> Result<Vec<ProxyProvider>> {
    store.list_providers().await
}

pub fn create_model(external_name: &str, provider_id: &str) -> Result<ModelCatalogEntry> {
    Ok(ModelCatalogEntry::new(external_name, provider_id))
}

pub async fn persist_model(
    store: &dyn AdminStore,
    external_name: &str,
    provider_id: &str,
) -> Result<ModelCatalogEntry> {
    let model = create_model(external_name, provider_id)?;
    store.insert_model(&model).await
}

pub async fn list_model_entries(store: &dyn AdminStore) -> Result<Vec<ModelCatalogEntry>> {
    store.list_models().await
}
