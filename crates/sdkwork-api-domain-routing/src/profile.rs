use super::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct RoutingProfileRecord {
    pub profile_id: String,
    pub tenant_id: String,
    pub project_id: String,
    pub name: String,
    pub slug: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub active: bool,
    #[serde(default)]
    pub strategy: RoutingStrategy,
    #[serde(default)]
    pub ordered_provider_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_provider_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_cost: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_latency_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub require_healthy: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preferred_region: Option<String>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl RoutingProfileRecord {
    pub fn new(
        profile_id: impl Into<String>,
        tenant_id: impl Into<String>,
        project_id: impl Into<String>,
        name: impl Into<String>,
        slug: impl Into<String>,
    ) -> Self {
        Self {
            profile_id: profile_id.into(),
            tenant_id: tenant_id.into(),
            project_id: project_id.into(),
            name: name.into(),
            slug: slug.into(),
            description: None,
            active: true,
            strategy: RoutingStrategy::DeterministicPriority,
            ordered_provider_ids: Vec::new(),
            default_provider_id: None,
            max_cost: None,
            max_latency_ms: None,
            require_healthy: false,
            preferred_region: None,
            created_at_ms: 0,
            updated_at_ms: 0,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_description_option(mut self, description: Option<String>) -> Self {
        self.description = description;
        self
    }

    pub fn with_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn with_strategy(mut self, strategy: RoutingStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    pub fn with_ordered_provider_ids(mut self, ordered_provider_ids: Vec<String>) -> Self {
        self.ordered_provider_ids = dedup_preserving_order(ordered_provider_ids);
        self
    }

    pub fn with_default_provider_id(mut self, default_provider_id: impl Into<String>) -> Self {
        self.default_provider_id = Some(default_provider_id.into());
        self
    }

    pub fn with_default_provider_id_option(mut self, default_provider_id: Option<String>) -> Self {
        self.default_provider_id = default_provider_id;
        self
    }

    pub fn with_max_cost(mut self, max_cost: f64) -> Self {
        self.max_cost = Some(max_cost);
        self
    }

    pub fn with_max_cost_option(mut self, max_cost: Option<f64>) -> Self {
        self.max_cost = max_cost;
        self
    }

    pub fn with_max_latency_ms(mut self, max_latency_ms: u64) -> Self {
        self.max_latency_ms = Some(max_latency_ms);
        self
    }

    pub fn with_max_latency_ms_option(mut self, max_latency_ms: Option<u64>) -> Self {
        self.max_latency_ms = max_latency_ms;
        self
    }

    pub fn with_require_healthy(mut self, require_healthy: bool) -> Self {
        self.require_healthy = require_healthy;
        self
    }

    pub fn with_preferred_region(mut self, preferred_region: impl Into<String>) -> Self {
        self.preferred_region = Some(preferred_region.into());
        self
    }

    pub fn with_preferred_region_option(mut self, preferred_region: Option<String>) -> Self {
        self.preferred_region = preferred_region;
        self
    }

    pub fn with_created_at_ms(mut self, created_at_ms: u64) -> Self {
        self.created_at_ms = created_at_ms;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct CompiledRoutingSnapshotRecord {
    pub snapshot_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_group_id: Option<String>,
    pub capability: String,
    pub route_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matched_policy_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_routing_preferences_project_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applied_routing_profile_id: Option<String>,
    pub strategy: String,
    #[serde(default)]
    pub ordered_provider_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_provider_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_cost: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_latency_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub require_healthy: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preferred_region: Option<String>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

impl CompiledRoutingSnapshotRecord {
    pub fn new(
        snapshot_id: impl Into<String>,
        capability: impl Into<String>,
        route_key: impl Into<String>,
    ) -> Self {
        Self {
            snapshot_id: snapshot_id.into(),
            tenant_id: None,
            project_id: None,
            api_key_group_id: None,
            capability: capability.into(),
            route_key: route_key.into(),
            matched_policy_id: None,
            project_routing_preferences_project_id: None,
            applied_routing_profile_id: None,
            strategy: String::new(),
            ordered_provider_ids: Vec::new(),
            default_provider_id: None,
            max_cost: None,
            max_latency_ms: None,
            require_healthy: false,
            preferred_region: None,
            created_at_ms: 0,
            updated_at_ms: 0,
        }
    }

    pub fn with_tenant_id(mut self, tenant_id: impl Into<String>) -> Self {
        self.tenant_id = Some(tenant_id.into());
        self
    }

    pub fn with_tenant_id_option(mut self, tenant_id: Option<String>) -> Self {
        self.tenant_id = tenant_id;
        self
    }

    pub fn with_project_id(mut self, project_id: impl Into<String>) -> Self {
        self.project_id = Some(project_id.into());
        self
    }

    pub fn with_project_id_option(mut self, project_id: Option<String>) -> Self {
        self.project_id = project_id;
        self
    }

    pub fn with_api_key_group_id(mut self, api_key_group_id: impl Into<String>) -> Self {
        self.api_key_group_id = Some(api_key_group_id.into());
        self
    }

    pub fn with_api_key_group_id_option(mut self, api_key_group_id: Option<String>) -> Self {
        self.api_key_group_id = api_key_group_id;
        self
    }

    pub fn with_matched_policy_id(mut self, matched_policy_id: impl Into<String>) -> Self {
        self.matched_policy_id = Some(matched_policy_id.into());
        self
    }

    pub fn with_matched_policy_id_option(mut self, matched_policy_id: Option<String>) -> Self {
        self.matched_policy_id = matched_policy_id;
        self
    }

    pub fn with_project_routing_preferences_project_id(
        mut self,
        project_id: impl Into<String>,
    ) -> Self {
        self.project_routing_preferences_project_id = Some(project_id.into());
        self
    }

    pub fn with_project_routing_preferences_project_id_option(
        mut self,
        project_id: Option<String>,
    ) -> Self {
        self.project_routing_preferences_project_id = project_id;
        self
    }

    pub fn with_applied_routing_profile_id(
        mut self,
        applied_routing_profile_id: impl Into<String>,
    ) -> Self {
        self.applied_routing_profile_id = Some(applied_routing_profile_id.into());
        self
    }

    pub fn with_applied_routing_profile_id_option(
        mut self,
        applied_routing_profile_id: Option<String>,
    ) -> Self {
        self.applied_routing_profile_id = applied_routing_profile_id;
        self
    }

    pub fn with_strategy(mut self, strategy: impl Into<String>) -> Self {
        self.strategy = strategy.into();
        self
    }

    pub fn with_ordered_provider_ids(mut self, ordered_provider_ids: Vec<String>) -> Self {
        self.ordered_provider_ids = dedup_preserving_order(ordered_provider_ids);
        self
    }

    pub fn with_default_provider_id(mut self, default_provider_id: impl Into<String>) -> Self {
        self.default_provider_id = Some(default_provider_id.into());
        self
    }

    pub fn with_default_provider_id_option(mut self, default_provider_id: Option<String>) -> Self {
        self.default_provider_id = default_provider_id;
        self
    }

    pub fn with_max_cost(mut self, max_cost: f64) -> Self {
        self.max_cost = Some(max_cost);
        self
    }

    pub fn with_max_cost_option(mut self, max_cost: Option<f64>) -> Self {
        self.max_cost = max_cost;
        self
    }

    pub fn with_max_latency_ms(mut self, max_latency_ms: u64) -> Self {
        self.max_latency_ms = Some(max_latency_ms);
        self
    }

    pub fn with_max_latency_ms_option(mut self, max_latency_ms: Option<u64>) -> Self {
        self.max_latency_ms = max_latency_ms;
        self
    }

    pub fn with_require_healthy(mut self, require_healthy: bool) -> Self {
        self.require_healthy = require_healthy;
        self
    }

    pub fn with_preferred_region(mut self, preferred_region: impl Into<String>) -> Self {
        self.preferred_region = Some(preferred_region.into());
        self
    }

    pub fn with_preferred_region_option(mut self, preferred_region: Option<String>) -> Self {
        self.preferred_region = preferred_region;
        self
    }

    pub fn with_created_at_ms(mut self, created_at_ms: u64) -> Self {
        self.created_at_ms = created_at_ms;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct ProjectRoutingPreferences {
    pub project_id: String,
    #[serde(default)]
    pub preset_id: String,
    #[serde(default)]
    pub strategy: RoutingStrategy,
    #[serde(default)]
    pub ordered_provider_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_provider_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_cost: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_latency_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub require_healthy: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preferred_region: Option<String>,
    #[serde(default)]
    pub updated_at_ms: u64,
}

impl ProjectRoutingPreferences {
    pub fn new(project_id: impl Into<String>) -> Self {
        Self {
            project_id: project_id.into(),
            preset_id: String::new(),
            strategy: RoutingStrategy::DeterministicPriority,
            ordered_provider_ids: Vec::new(),
            default_provider_id: None,
            max_cost: None,
            max_latency_ms: None,
            require_healthy: false,
            preferred_region: None,
            updated_at_ms: 0,
        }
    }

    pub fn with_preset_id(mut self, preset_id: impl Into<String>) -> Self {
        self.preset_id = preset_id.into();
        self
    }

    pub fn with_strategy(mut self, strategy: RoutingStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    pub fn with_ordered_provider_ids(mut self, ordered_provider_ids: Vec<String>) -> Self {
        self.ordered_provider_ids = dedup_preserving_order(ordered_provider_ids);
        self
    }

    pub fn with_default_provider_id(mut self, default_provider_id: impl Into<String>) -> Self {
        self.default_provider_id = Some(default_provider_id.into());
        self
    }

    pub fn with_default_provider_id_option(mut self, default_provider_id: Option<String>) -> Self {
        self.default_provider_id = default_provider_id;
        self
    }

    pub fn with_max_cost(mut self, max_cost: f64) -> Self {
        self.max_cost = Some(max_cost);
        self
    }

    pub fn with_max_cost_option(mut self, max_cost: Option<f64>) -> Self {
        self.max_cost = max_cost;
        self
    }

    pub fn with_max_latency_ms(mut self, max_latency_ms: u64) -> Self {
        self.max_latency_ms = Some(max_latency_ms);
        self
    }

    pub fn with_max_latency_ms_option(mut self, max_latency_ms: Option<u64>) -> Self {
        self.max_latency_ms = max_latency_ms;
        self
    }

    pub fn with_require_healthy(mut self, require_healthy: bool) -> Self {
        self.require_healthy = require_healthy;
        self
    }

    pub fn with_preferred_region(mut self, preferred_region: impl Into<String>) -> Self {
        self.preferred_region = Some(preferred_region.into());
        self
    }

    pub fn with_preferred_region_option(mut self, preferred_region: Option<String>) -> Self {
        self.preferred_region = preferred_region;
        self
    }

    pub fn with_updated_at_ms(mut self, updated_at_ms: u64) -> Self {
        self.updated_at_ms = updated_at_ms;
        self
    }
}
