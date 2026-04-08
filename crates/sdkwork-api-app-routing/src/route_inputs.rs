use super::*;

pub fn service_name() -> &'static str {
    "routing-service"
}

pub struct CreateRoutingPolicyInput<'a> {
    pub policy_id: &'a str,
    pub capability: &'a str,
    pub model_pattern: &'a str,
    pub enabled: bool,
    pub priority: i32,
    pub strategy: Option<RoutingStrategy>,
    pub ordered_provider_ids: &'a [String],
    pub default_provider_id: Option<&'a str>,
    pub max_cost: Option<f64>,
    pub max_latency_ms: Option<u64>,
    pub require_healthy: bool,
    pub execution_failover_enabled: bool,
    pub upstream_retry_max_attempts: Option<u32>,
    pub upstream_retry_base_delay_ms: Option<u64>,
    pub upstream_retry_max_delay_ms: Option<u64>,
}

pub struct CreateRoutingProfileInput<'a> {
    pub profile_id: &'a str,
    pub tenant_id: &'a str,
    pub project_id: &'a str,
    pub name: &'a str,
    pub slug: &'a str,
    pub description: Option<&'a str>,
    pub active: bool,
    pub strategy: Option<RoutingStrategy>,
    pub ordered_provider_ids: &'a [String],
    pub default_provider_id: Option<&'a str>,
    pub max_cost: Option<f64>,
    pub max_latency_ms: Option<u64>,
    pub require_healthy: bool,
    pub preferred_region: Option<&'a str>,
}

#[derive(Clone, Copy)]
pub struct RouteSelectionContext<'a> {
    pub decision_source: RoutingDecisionSource,
    pub tenant_id: Option<&'a str>,
    pub project_id: Option<&'a str>,
    pub api_key_group_id: Option<&'a str>,
    pub requested_region: Option<&'a str>,
    pub selection_seed: Option<u64>,
    pub recovery_probe_lock_store: Option<&'a dyn DistributedLockStore>,
}

impl std::fmt::Debug for RouteSelectionContext<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RouteSelectionContext")
            .field("decision_source", &self.decision_source)
            .field("tenant_id", &self.tenant_id)
            .field("project_id", &self.project_id)
            .field("api_key_group_id", &self.api_key_group_id)
            .field("requested_region", &self.requested_region)
            .field("selection_seed", &self.selection_seed)
            .field(
                "recovery_probe_lock_store_configured",
                &self.recovery_probe_lock_store.is_some(),
            )
            .finish()
    }
}

impl<'a> RouteSelectionContext<'a> {
    pub fn new(decision_source: RoutingDecisionSource) -> Self {
        Self {
            decision_source,
            tenant_id: None,
            project_id: None,
            api_key_group_id: None,
            requested_region: None,
            selection_seed: None,
            recovery_probe_lock_store: None,
        }
    }

    pub fn with_tenant_id_option(mut self, tenant_id: Option<&'a str>) -> Self {
        self.tenant_id = tenant_id;
        self
    }

    pub fn with_project_id_option(mut self, project_id: Option<&'a str>) -> Self {
        self.project_id = project_id;
        self
    }

    pub fn with_api_key_group_id_option(mut self, api_key_group_id: Option<&'a str>) -> Self {
        self.api_key_group_id = api_key_group_id;
        self
    }

    pub fn with_requested_region_option(mut self, requested_region: Option<&'a str>) -> Self {
        self.requested_region = requested_region;
        self
    }

    pub fn with_selection_seed_option(mut self, selection_seed: Option<u64>) -> Self {
        self.selection_seed = selection_seed;
        self
    }

    pub fn with_recovery_probe_lock_store_option(
        mut self,
        recovery_probe_lock_store: Option<&'a dyn DistributedLockStore>,
    ) -> Self {
        self.recovery_probe_lock_store = recovery_probe_lock_store;
        self
    }
}

