use anyhow::Result;
use sdkwork_api_domain_credential::UpstreamCredential;

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
