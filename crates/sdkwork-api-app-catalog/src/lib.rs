use anyhow::{anyhow, Result};
use sdkwork_api_domain_catalog::{
    Channel, ChannelModelRecord, ModelCapability, ModelCatalogEntry, ModelPriceRecord,
    ProviderChannelBinding, ProxyProvider,
};
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

pub async fn delete_channel(store: &dyn AdminStore, channel_id: &str) -> Result<bool> {
    let channel_id = channel_id.trim();
    if channel_id.is_empty() {
        return Err(anyhow!("channel id is required"));
    }
    store.delete_channel(channel_id).await
}

pub fn create_provider_with_config(
    id: &str,
    channel_id: &str,
    adapter_kind: &str,
    base_url: &str,
    display_name: &str,
) -> Result<ProxyProvider> {
    create_provider_with_extension_id(id, channel_id, adapter_kind, None, base_url, display_name)
}

pub fn create_provider_with_extension_id(
    id: &str,
    channel_id: &str,
    adapter_kind: &str,
    extension_id: Option<&str>,
    base_url: &str,
    display_name: &str,
) -> Result<ProxyProvider> {
    let provider = ProxyProvider::new(id, channel_id, adapter_kind, base_url, display_name);
    Ok(match extension_id {
        Some(extension_id) => provider.with_extension_id(extension_id),
        None => provider,
    })
}

pub fn create_provider_with_bindings(
    id: &str,
    channel_id: &str,
    adapter_kind: &str,
    base_url: &str,
    display_name: &str,
    channel_bindings: &[ProviderChannelBinding],
) -> Result<ProxyProvider> {
    create_provider_with_bindings_and_extension_id(
        id,
        channel_id,
        adapter_kind,
        None,
        base_url,
        display_name,
        channel_bindings,
    )
}

pub fn create_provider_with_bindings_and_extension_id(
    id: &str,
    channel_id: &str,
    adapter_kind: &str,
    extension_id: Option<&str>,
    base_url: &str,
    display_name: &str,
    channel_bindings: &[ProviderChannelBinding],
) -> Result<ProxyProvider> {
    let mut provider = create_provider_with_extension_id(
        id,
        channel_id,
        adapter_kind,
        extension_id,
        base_url,
        display_name,
    )?;
    for binding in channel_bindings {
        provider = provider.with_channel_binding(binding.clone());
    }
    Ok(provider)
}

#[derive(Debug, Clone, Copy)]
pub struct PersistProviderWithBindingsRequest<'a> {
    pub id: &'a str,
    pub channel_id: &'a str,
    pub adapter_kind: &'a str,
    pub extension_id: Option<&'a str>,
    pub base_url: &'a str,
    pub display_name: &'a str,
    pub channel_bindings: &'a [ProviderChannelBinding],
}

pub async fn persist_provider(
    store: &dyn AdminStore,
    id: &str,
    channel_id: &str,
    adapter_kind: &str,
    base_url: &str,
    display_name: &str,
) -> Result<ProxyProvider> {
    let provider = create_provider_with_extension_id(
        id,
        channel_id,
        adapter_kind,
        None,
        base_url,
        display_name,
    )?;
    store.insert_provider(&provider).await
}

pub async fn persist_provider_with_bindings(
    store: &dyn AdminStore,
    id: &str,
    channel_id: &str,
    adapter_kind: &str,
    base_url: &str,
    display_name: &str,
    channel_bindings: &[ProviderChannelBinding],
) -> Result<ProxyProvider> {
    persist_provider_with_bindings_and_extension_id(
        store,
        PersistProviderWithBindingsRequest {
            id,
            channel_id,
            adapter_kind,
            extension_id: None,
            base_url,
            display_name,
            channel_bindings,
        },
    )
    .await
}

pub async fn persist_provider_with_bindings_and_extension_id(
    store: &dyn AdminStore,
    request: PersistProviderWithBindingsRequest<'_>,
) -> Result<ProxyProvider> {
    let provider = create_provider_with_bindings_and_extension_id(
        request.id,
        request.channel_id,
        request.adapter_kind,
        request.extension_id,
        request.base_url,
        request.display_name,
        request.channel_bindings,
    )?;
    store.insert_provider(&provider).await
}

pub async fn list_providers(store: &dyn AdminStore) -> Result<Vec<ProxyProvider>> {
    store.list_providers().await
}

pub async fn delete_provider(store: &dyn AdminStore, provider_id: &str) -> Result<bool> {
    let provider_id = provider_id.trim();
    if provider_id.is_empty() {
        return Err(anyhow!("provider id is required"));
    }
    store.delete_provider(provider_id).await
}

pub fn create_model(external_name: &str, provider_id: &str) -> Result<ModelCatalogEntry> {
    Ok(ModelCatalogEntry::new(external_name, provider_id))
}

pub fn create_channel_model(
    channel_id: &str,
    model_id: &str,
    model_display_name: &str,
) -> Result<ChannelModelRecord> {
    Ok(ChannelModelRecord::new(
        channel_id,
        model_id,
        model_display_name,
    ))
}

#[allow(clippy::too_many_arguments)]
pub fn create_channel_model_with_metadata(
    channel_id: &str,
    model_id: &str,
    model_display_name: &str,
    capabilities: &[ModelCapability],
    streaming: bool,
    context_window: Option<u64>,
    description: Option<&str>,
) -> Result<ChannelModelRecord> {
    let mut model = create_channel_model(channel_id, model_id, model_display_name)?
        .with_streaming(streaming)
        .with_context_window_option(context_window)
        .with_description_option(description.map(str::to_owned));
    for capability in capabilities {
        model = model.with_capability(capability.clone());
    }
    Ok(model)
}

pub fn create_model_price(
    channel_id: &str,
    model_id: &str,
    proxy_provider_id: &str,
) -> Result<ModelPriceRecord> {
    Ok(ModelPriceRecord::new(
        channel_id,
        model_id,
        proxy_provider_id,
    ))
}

#[allow(clippy::too_many_arguments)]
pub fn create_model_price_with_rates(
    channel_id: &str,
    model_id: &str,
    proxy_provider_id: &str,
    currency_code: &str,
    price_unit: &str,
    input_price: f64,
    output_price: f64,
    cache_read_price: f64,
    cache_write_price: f64,
    request_price: f64,
    is_active: bool,
) -> Result<ModelPriceRecord> {
    Ok(create_model_price(channel_id, model_id, proxy_provider_id)?
        .with_currency_code(currency_code)
        .with_price_unit(price_unit)
        .with_input_price(input_price)
        .with_output_price(output_price)
        .with_cache_read_price(cache_read_price)
        .with_cache_write_price(cache_write_price)
        .with_request_price(request_price)
        .with_active(is_active))
}

pub fn create_model_with_metadata(
    external_name: &str,
    provider_id: &str,
    capabilities: &[ModelCapability],
    streaming: bool,
    context_window: Option<u64>,
) -> Result<ModelCatalogEntry> {
    let mut model = create_model(external_name, provider_id)?.with_streaming(streaming);
    for capability in capabilities {
        model = model.with_capability(capability.clone());
    }
    if let Some(context_window) = context_window {
        model = model.with_context_window(context_window);
    }
    Ok(model)
}

pub async fn persist_model(
    store: &dyn AdminStore,
    external_name: &str,
    provider_id: &str,
) -> Result<ModelCatalogEntry> {
    persist_model_with_metadata(store, external_name, provider_id, &[], false, None).await
}

pub async fn persist_model_with_metadata(
    store: &dyn AdminStore,
    external_name: &str,
    provider_id: &str,
    capabilities: &[ModelCapability],
    streaming: bool,
    context_window: Option<u64>,
) -> Result<ModelCatalogEntry> {
    let provider = store
        .find_provider(provider_id)
        .await?
        .ok_or_else(|| anyhow!("provider_id is not registered"))?;
    let channel_model = create_channel_model_with_metadata(
        &provider.channel_id,
        external_name,
        external_name,
        capabilities,
        streaming,
        context_window,
        None,
    )?;
    store.insert_channel_model(&channel_model).await?;

    let default_price = create_model_price(&provider.channel_id, external_name, provider_id)?;
    store.insert_model_price(&default_price).await?;

    let model = create_model_with_metadata(
        external_name,
        provider_id,
        capabilities,
        streaming,
        context_window,
    )?;
    Ok(model)
}

pub async fn list_model_entries(store: &dyn AdminStore) -> Result<Vec<ModelCatalogEntry>> {
    store.list_models().await
}

#[allow(clippy::too_many_arguments)]
pub async fn persist_channel_model_with_metadata(
    store: &dyn AdminStore,
    channel_id: &str,
    model_id: &str,
    model_display_name: &str,
    capabilities: &[ModelCapability],
    streaming: bool,
    context_window: Option<u64>,
    description: Option<&str>,
) -> Result<ChannelModelRecord> {
    let record = create_channel_model_with_metadata(
        channel_id,
        model_id,
        model_display_name,
        capabilities,
        streaming,
        context_window,
        description,
    )?;
    store.insert_channel_model(&record).await
}

pub async fn list_channel_models(store: &dyn AdminStore) -> Result<Vec<ChannelModelRecord>> {
    store.list_channel_models().await
}

pub async fn delete_channel_model(
    store: &dyn AdminStore,
    channel_id: &str,
    model_id: &str,
) -> Result<bool> {
    let channel_id = channel_id.trim();
    let model_id = model_id.trim();
    if channel_id.is_empty() || model_id.is_empty() {
        return Err(anyhow!("channel_id and model_id are required"));
    }
    store.delete_channel_model(channel_id, model_id).await
}

#[allow(clippy::too_many_arguments)]
pub async fn persist_model_price_with_rates(
    store: &dyn AdminStore,
    channel_id: &str,
    model_id: &str,
    proxy_provider_id: &str,
    currency_code: &str,
    price_unit: &str,
    input_price: f64,
    output_price: f64,
    cache_read_price: f64,
    cache_write_price: f64,
    request_price: f64,
    is_active: bool,
) -> Result<ModelPriceRecord> {
    let record = create_model_price_with_rates(
        channel_id,
        model_id,
        proxy_provider_id,
        currency_code,
        price_unit,
        input_price,
        output_price,
        cache_read_price,
        cache_write_price,
        request_price,
        is_active,
    )?;
    store.insert_model_price(&record).await
}

pub async fn list_model_prices(store: &dyn AdminStore) -> Result<Vec<ModelPriceRecord>> {
    store.list_model_prices().await
}

pub async fn delete_model_price(
    store: &dyn AdminStore,
    channel_id: &str,
    model_id: &str,
    proxy_provider_id: &str,
) -> Result<bool> {
    let channel_id = channel_id.trim();
    let model_id = model_id.trim();
    let proxy_provider_id = proxy_provider_id.trim();
    if channel_id.is_empty() || model_id.is_empty() || proxy_provider_id.is_empty() {
        return Err(anyhow!(
            "channel_id, model_id, and proxy_provider_id are required"
        ));
    }
    store
        .delete_model_price(channel_id, model_id, proxy_provider_id)
        .await
}

pub async fn delete_model_variant(
    store: &dyn AdminStore,
    external_name: &str,
    provider_id: &str,
) -> Result<bool> {
    let external_name = external_name.trim();
    if external_name.is_empty() {
        return Err(anyhow!("external_name is required"));
    }
    let provider_id = provider_id.trim();
    if provider_id.is_empty() {
        return Err(anyhow!("provider_id is required"));
    }
    store.delete_model_variant(external_name, provider_id).await
}
