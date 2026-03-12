#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpstreamCredential {
    pub tenant_id: String,
    pub provider_id: String,
    pub key_reference: String,
}

impl UpstreamCredential {
    pub fn new(
        tenant_id: impl Into<String>,
        provider_id: impl Into<String>,
        key_reference: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            provider_id: provider_id.into(),
            key_reference: key_reference.into(),
        }
    }
}
