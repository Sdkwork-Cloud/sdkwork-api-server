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
