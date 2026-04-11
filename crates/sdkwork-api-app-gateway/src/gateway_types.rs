use sdkwork_api_domain_catalog::ProxyProvider;
use sdkwork_api_domain_routing::RoutingDecision;
use sdkwork_api_extension_core::ExtensionRuntime;

#[derive(Debug, Clone, PartialEq)]
pub struct PlannedExecutionUsageContext {
    pub provider_id: String,
    pub channel_id: Option<String>,
    pub api_key_group_id: Option<String>,
    pub applied_routing_profile_id: Option<String>,
    pub compiled_routing_snapshot_id: Option<String>,
    pub fallback_reason: Option<String>,
    pub latency_ms: Option<u64>,
    pub reference_amount: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlannedExecutionRuntimeContext {
    pub provider_account_id: Option<String>,
    pub execution_instance_id: Option<String>,
    pub runtime_key: String,
    pub base_url: String,
    pub runtime: ExtensionRuntime,
    pub local_fallback: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlannedExecutionProviderContext {
    pub decision: RoutingDecision,
    pub provider: ProxyProvider,
    pub api_key: String,
    pub usage_context: PlannedExecutionUsageContext,
    pub execution: PlannedExecutionRuntimeContext,
}

pub struct GatewayExecutionResult<T> {
    pub response: Option<T>,
    pub usage_context: Option<PlannedExecutionUsageContext>,
}

impl<T> GatewayExecutionResult<T> {
    pub(crate) fn new(
        response: Option<T>,
        usage_context: Option<PlannedExecutionUsageContext>,
    ) -> Self {
        Self {
            response,
            usage_context,
        }
    }
}
