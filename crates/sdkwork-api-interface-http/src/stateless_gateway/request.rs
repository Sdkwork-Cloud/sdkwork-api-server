use super::config::StatelessGatewayContext;
use super::*;

#[derive(Clone, Debug)]
pub(crate) struct StatelessGatewayRequest(StatelessGatewayContext);

impl StatelessGatewayRequest {
    pub(crate) fn tenant_id(&self) -> &str {
        &self.0.tenant_id
    }

    pub(crate) fn project_id(&self) -> &str {
        &self.0.project_id
    }

    pub(crate) fn upstream(&self) -> Option<&StatelessGatewayUpstream> {
        self.0.upstream.as_deref()
    }
}

impl FromRequestParts<StatelessGatewayContext> for StatelessGatewayRequest {
    type Rejection = StatusCode;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &StatelessGatewayContext,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self(state.clone()))
    }
}
