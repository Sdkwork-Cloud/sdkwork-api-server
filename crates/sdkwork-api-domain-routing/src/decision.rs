use super::*;


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RoutingCandidateHealth {
    Healthy,
    Unhealthy,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RoutingStrategy {
    #[default]
    DeterministicPriority,
    WeightedRandom,
    SloAware,
    GeoAffinity,
}

impl RoutingStrategy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DeterministicPriority => "deterministic_priority",
            Self::WeightedRandom => "weighted_random",
            Self::SloAware => "slo_aware",
            Self::GeoAffinity => "geo_affinity",
        }
    }
}

impl FromStr for RoutingStrategy {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "deterministic_priority" => Ok(Self::DeterministicPriority),
            "weighted_random" => Ok(Self::WeightedRandom),
            "slo_aware" => Ok(Self::SloAware),
            "geo_affinity" => Ok(Self::GeoAffinity),
            _ => Err(format!("unsupported routing strategy: {value}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct RoutingCandidateAssessment {
    pub provider_id: String,
    pub available: bool,
    pub health: RoutingCandidateHealth,
    pub policy_rank: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region_match: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slo_eligible: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub slo_violations: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reasons: Vec<String>,
}

impl RoutingCandidateAssessment {
    pub fn new(provider_id: impl Into<String>) -> Self {
        Self {
            provider_id: provider_id.into(),
            available: true,
            health: RoutingCandidateHealth::Unknown,
            policy_rank: 0,
            weight: None,
            cost: None,
            latency_ms: None,
            region: None,
            region_match: None,
            slo_eligible: None,
            slo_violations: Vec::new(),
            reasons: Vec::new(),
        }
    }

    pub fn with_available(mut self, available: bool) -> Self {
        self.available = available;
        self
    }

    pub fn with_health(mut self, health: RoutingCandidateHealth) -> Self {
        self.health = health;
        self
    }

    pub fn with_policy_rank(mut self, policy_rank: usize) -> Self {
        self.policy_rank = policy_rank;
        self
    }

    pub fn with_weight(mut self, weight: u64) -> Self {
        self.weight = Some(weight);
        self
    }

    pub fn with_cost(mut self, cost: f64) -> Self {
        self.cost = Some(cost);
        self
    }

    pub fn with_latency_ms(mut self, latency_ms: u64) -> Self {
        self.latency_ms = Some(latency_ms);
        self
    }

    pub fn with_region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(region.into());
        self
    }

    pub fn with_region_option(mut self, region: Option<String>) -> Self {
        self.region = region;
        self
    }

    pub fn with_region_match(mut self, region_match: bool) -> Self {
        self.region_match = Some(region_match);
        self
    }

    pub fn with_region_match_option(mut self, region_match: Option<bool>) -> Self {
        self.region_match = region_match;
        self
    }

    pub fn with_slo_eligible(mut self, slo_eligible: bool) -> Self {
        self.slo_eligible = Some(slo_eligible);
        self
    }

    pub fn with_slo_violation(mut self, slo_violation: impl Into<String>) -> Self {
        self.slo_violations.push(slo_violation.into());
        self
    }

    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reasons.push(reason.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct RoutingDecision {
    pub selected_provider_id: String,
    pub candidate_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matched_policy_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applied_routing_profile_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compiled_routing_snapshot_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strategy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selection_seed: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selection_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_region: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_health_recovery_probe: Option<ProviderHealthRecoveryProbe>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub slo_applied: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub slo_degraded: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub assessments: Vec<RoutingCandidateAssessment>,
}

impl RoutingDecision {
    pub fn new(selected_provider_id: impl Into<String>, candidate_ids: Vec<String>) -> Self {
        Self {
            selected_provider_id: selected_provider_id.into(),
            candidate_ids,
            matched_policy_id: None,
            applied_routing_profile_id: None,
            compiled_routing_snapshot_id: None,
            strategy: None,
            selection_seed: None,
            selection_reason: None,
            fallback_reason: None,
            requested_region: None,
            provider_health_recovery_probe: None,
            slo_applied: false,
            slo_degraded: false,
            assessments: Vec::new(),
        }
    }

    pub fn with_matched_policy_id(mut self, matched_policy_id: impl Into<String>) -> Self {
        self.matched_policy_id = Some(matched_policy_id.into());
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

    pub fn with_compiled_routing_snapshot_id(
        mut self,
        compiled_routing_snapshot_id: impl Into<String>,
    ) -> Self {
        self.compiled_routing_snapshot_id = Some(compiled_routing_snapshot_id.into());
        self
    }

    pub fn with_compiled_routing_snapshot_id_option(
        mut self,
        compiled_routing_snapshot_id: Option<String>,
    ) -> Self {
        self.compiled_routing_snapshot_id = compiled_routing_snapshot_id;
        self
    }

    pub fn with_strategy(mut self, strategy: impl Into<String>) -> Self {
        self.strategy = Some(strategy.into());
        self
    }

    pub fn with_selection_seed(mut self, selection_seed: u64) -> Self {
        self.selection_seed = Some(selection_seed);
        self
    }

    pub fn with_selection_reason(mut self, selection_reason: impl Into<String>) -> Self {
        self.selection_reason = Some(selection_reason.into());
        self
    }

    pub fn with_fallback_reason(mut self, fallback_reason: impl Into<String>) -> Self {
        self.fallback_reason = Some(fallback_reason.into());
        self
    }

    pub fn with_fallback_reason_option(mut self, fallback_reason: Option<String>) -> Self {
        self.fallback_reason = fallback_reason;
        self
    }

    pub fn with_requested_region(mut self, requested_region: impl Into<String>) -> Self {
        self.requested_region = Some(requested_region.into());
        self
    }

    pub fn with_requested_region_option(mut self, requested_region: Option<String>) -> Self {
        self.requested_region = requested_region;
        self
    }

    pub fn with_provider_health_recovery_probe(
        mut self,
        provider_health_recovery_probe: ProviderHealthRecoveryProbe,
    ) -> Self {
        self.provider_health_recovery_probe = Some(provider_health_recovery_probe);
        self
    }

    pub fn with_provider_health_recovery_probe_option(
        mut self,
        provider_health_recovery_probe: Option<ProviderHealthRecoveryProbe>,
    ) -> Self {
        self.provider_health_recovery_probe = provider_health_recovery_probe;
        self
    }

    pub fn with_slo_state(mut self, slo_applied: bool, slo_degraded: bool) -> Self {
        self.slo_applied = slo_applied;
        self.slo_degraded = slo_degraded;
        self
    }

    pub fn with_assessments(mut self, assessments: Vec<RoutingCandidateAssessment>) -> Self {
        self.assessments = assessments;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProviderHealthRecoveryProbeOutcome {
    Selected,
    LeaseContended,
    LeaseError,
}

impl ProviderHealthRecoveryProbeOutcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Selected => "selected",
            Self::LeaseContended => "lease_contended",
            Self::LeaseError => "lease_error",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ProviderHealthRecoveryProbe {
    pub provider_id: String,
    pub outcome: ProviderHealthRecoveryProbeOutcome,
}

impl ProviderHealthRecoveryProbe {
    pub fn new(
        provider_id: impl Into<String>,
        outcome: ProviderHealthRecoveryProbeOutcome,
    ) -> Self {
        Self {
            provider_id: provider_id.into(),
            outcome,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RoutingDecisionSource {
    Gateway,
    AdminSimulation,
    PortalSimulation,
}

impl RoutingDecisionSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Gateway => "gateway",
            Self::AdminSimulation => "admin_simulation",
            Self::PortalSimulation => "portal_simulation",
        }
    }
}

impl FromStr for RoutingDecisionSource {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "gateway" => Ok(Self::Gateway),
            "admin_simulation" => Ok(Self::AdminSimulation),
            "portal_simulation" => Ok(Self::PortalSimulation),
            _ => Err(format!("unsupported routing decision source: {value}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct RoutingDecisionLog {
    pub decision_id: String,
    pub decision_source: RoutingDecisionSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_group_id: Option<String>,
    pub capability: String,
    pub route_key: String,
    pub selected_provider_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matched_policy_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applied_routing_profile_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compiled_routing_snapshot_id: Option<String>,
    pub strategy: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selection_seed: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selection_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_region: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub slo_applied: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub slo_degraded: bool,
    pub created_at_ms: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub assessments: Vec<RoutingCandidateAssessment>,
}

impl RoutingDecisionLog {
    pub fn new(
        decision_id: impl Into<String>,
        decision_source: RoutingDecisionSource,
        capability: impl Into<String>,
        route_key: impl Into<String>,
        selected_provider_id: impl Into<String>,
        strategy: impl Into<String>,
        created_at_ms: u64,
    ) -> Self {
        Self {
            decision_id: decision_id.into(),
            decision_source,
            tenant_id: None,
            project_id: None,
            api_key_group_id: None,
            capability: capability.into(),
            route_key: route_key.into(),
            selected_provider_id: selected_provider_id.into(),
            matched_policy_id: None,
            applied_routing_profile_id: None,
            compiled_routing_snapshot_id: None,
            strategy: strategy.into(),
            selection_seed: None,
            selection_reason: None,
            fallback_reason: None,
            requested_region: None,
            slo_applied: false,
            slo_degraded: false,
            created_at_ms,
            assessments: Vec::new(),
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

    pub fn with_compiled_routing_snapshot_id(
        mut self,
        compiled_routing_snapshot_id: impl Into<String>,
    ) -> Self {
        self.compiled_routing_snapshot_id = Some(compiled_routing_snapshot_id.into());
        self
    }

    pub fn with_compiled_routing_snapshot_id_option(
        mut self,
        compiled_routing_snapshot_id: Option<String>,
    ) -> Self {
        self.compiled_routing_snapshot_id = compiled_routing_snapshot_id;
        self
    }

    pub fn with_matched_policy_id_option(mut self, matched_policy_id: Option<String>) -> Self {
        self.matched_policy_id = matched_policy_id;
        self
    }

    pub fn with_selection_seed(mut self, selection_seed: u64) -> Self {
        self.selection_seed = Some(selection_seed);
        self
    }

    pub fn with_selection_seed_option(mut self, selection_seed: Option<u64>) -> Self {
        self.selection_seed = selection_seed;
        self
    }

    pub fn with_selection_reason(mut self, selection_reason: impl Into<String>) -> Self {
        self.selection_reason = Some(selection_reason.into());
        self
    }

    pub fn with_selection_reason_option(mut self, selection_reason: Option<String>) -> Self {
        self.selection_reason = selection_reason;
        self
    }

    pub fn with_fallback_reason(mut self, fallback_reason: impl Into<String>) -> Self {
        self.fallback_reason = Some(fallback_reason.into());
        self
    }

    pub fn with_fallback_reason_option(mut self, fallback_reason: Option<String>) -> Self {
        self.fallback_reason = fallback_reason;
        self
    }

    pub fn with_requested_region(mut self, requested_region: impl Into<String>) -> Self {
        self.requested_region = Some(requested_region.into());
        self
    }

    pub fn with_requested_region_option(mut self, requested_region: Option<String>) -> Self {
        self.requested_region = requested_region;
        self
    }

    pub fn with_slo_state(mut self, slo_applied: bool, slo_degraded: bool) -> Self {
        self.slo_applied = slo_applied;
        self.slo_degraded = slo_degraded;
        self
    }

    pub fn with_assessments(mut self, assessments: Vec<RoutingCandidateAssessment>) -> Self {
        self.assessments = assessments;
        self
    }
}
