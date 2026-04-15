use super::*;
use anyhow::anyhow;
use sdkwork_api_domain_catalog::{
    derive_provider_protocol_kind, normalize_provider_default_plugin_family,
    normalize_provider_protocol_kind,
};

const DEFAULT_STATELESS_TENANT_ID: &str = "sdkwork-stateless";
const DEFAULT_STATELESS_PROJECT_ID: &str = "sdkwork-stateless-default";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatelessGatewayUpstream {
    runtime_key: String,
    protocol_kind: String,
    base_url: String,
    api_key: String,
}

impl StatelessGatewayUpstream {
    pub fn new(
        runtime_key: impl Into<String>,
        base_url: impl Into<String>,
        api_key: impl Into<String>,
    ) -> Self {
        let runtime_key = runtime_key.into();
        Self::new_with_protocol_kind(
            runtime_key.clone(),
            derive_provider_protocol_kind(&runtime_key),
            base_url,
            api_key,
        )
    }

    pub fn new_with_protocol_kind(
        runtime_key: impl Into<String>,
        protocol_kind: impl Into<String>,
        base_url: impl Into<String>,
        api_key: impl Into<String>,
    ) -> Self {
        let runtime_key = runtime_key.into();
        Self {
            protocol_kind: normalize_provider_protocol_kind(protocol_kind.into(), &runtime_key),
            runtime_key,
            base_url: base_url.into(),
            api_key: api_key.into(),
        }
    }

    pub fn from_adapter_kind(
        adapter_kind: impl Into<String>,
        base_url: impl Into<String>,
        api_key: impl Into<String>,
    ) -> Self {
        Self::new(adapter_kind, base_url, api_key)
    }

    pub fn from_default_plugin_family(
        default_plugin_family: impl AsRef<str>,
        base_url: impl Into<String>,
        api_key: impl Into<String>,
    ) -> anyhow::Result<Self> {
        let Some(normalized) =
            normalize_provider_default_plugin_family(default_plugin_family.as_ref())
        else {
            return Err(anyhow!("unsupported default_plugin_family"));
        };

        match normalized {
            "openrouter" | "ollama" => Ok(Self::new(normalized, base_url, api_key)),
            _ => Err(anyhow!("unsupported default_plugin_family")),
        }
    }

    pub fn runtime_key(&self) -> &str {
        &self.runtime_key
    }

    pub fn protocol_kind(&self) -> &str {
        &self.protocol_kind
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn api_key(&self) -> &str {
        &self.api_key
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatelessGatewayConfig {
    tenant_id: String,
    project_id: String,
    upstream: Option<StatelessGatewayUpstream>,
}

impl Default for StatelessGatewayConfig {
    fn default() -> Self {
        Self {
            tenant_id: DEFAULT_STATELESS_TENANT_ID.to_owned(),
            project_id: DEFAULT_STATELESS_PROJECT_ID.to_owned(),
            upstream: None,
        }
    }
}

impl StatelessGatewayConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_identity(
        mut self,
        tenant_id: impl Into<String>,
        project_id: impl Into<String>,
    ) -> Self {
        self.tenant_id = tenant_id.into();
        self.project_id = project_id.into();
        self
    }

    pub fn with_upstream(mut self, upstream: StatelessGatewayUpstream) -> Self {
        self.upstream = Some(upstream);
        self
    }

    pub fn try_with_default_plugin_upstream(
        self,
        default_plugin_family: impl AsRef<str>,
        base_url: impl Into<String>,
        api_key: impl Into<String>,
    ) -> anyhow::Result<Self> {
        Ok(
            self.with_upstream(StatelessGatewayUpstream::from_default_plugin_family(
                default_plugin_family,
                base_url,
                api_key,
            )?),
        )
    }

    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    pub fn project_id(&self) -> &str {
        &self.project_id
    }

    pub fn upstream(&self) -> Option<&StatelessGatewayUpstream> {
        self.upstream.as_ref()
    }

    pub(crate) fn into_context(self) -> StatelessGatewayContext {
        StatelessGatewayContext {
            tenant_id: Arc::from(self.tenant_id),
            project_id: Arc::from(self.project_id),
            upstream: self.upstream.map(Arc::new),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct StatelessGatewayContext {
    pub(crate) tenant_id: Arc<str>,
    pub(crate) project_id: Arc<str>,
    pub(crate) upstream: Option<Arc<StatelessGatewayUpstream>>,
}
