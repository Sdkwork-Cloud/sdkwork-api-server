use super::*;

const DEFAULT_STATELESS_TENANT_ID: &str = "sdkwork-stateless";
const DEFAULT_STATELESS_PROJECT_ID: &str = "sdkwork-stateless-default";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatelessGatewayUpstream {
    runtime_key: String,
    base_url: String,
    api_key: String,
}

impl StatelessGatewayUpstream {
    pub fn new(
        runtime_key: impl Into<String>,
        base_url: impl Into<String>,
        api_key: impl Into<String>,
    ) -> Self {
        Self {
            runtime_key: runtime_key.into(),
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

    pub fn runtime_key(&self) -> &str {
        &self.runtime_key
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
