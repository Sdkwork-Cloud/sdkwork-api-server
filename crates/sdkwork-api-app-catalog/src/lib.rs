use anyhow::Result;
use sdkwork_api_domain_catalog::ProxyProvider;

pub fn service_name() -> &'static str {
    "catalog-service"
}

pub fn create_provider(id: &str, channel_id: &str, display_name: &str) -> Result<ProxyProvider> {
    Ok(ProxyProvider::new(id, channel_id, display_name))
}
