use std::collections::BTreeSet;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingCandidateHealth {
    Healthy,
    Unhealthy,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub selected_provider_id: String,
    pub candidate_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matched_policy_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strategy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selection_seed: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selection_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_region: Option<String>,
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
            strategy: None,
            selection_seed: None,
            selection_reason: None,
            requested_region: None,
            slo_applied: false,
            slo_degraded: false,
            assessments: Vec::new(),
        }
    }

    pub fn with_matched_policy_id(mut self, matched_policy_id: impl Into<String>) -> Self {
        self.matched_policy_id = Some(matched_policy_id.into());
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoutingDecisionLog {
    pub decision_id: String,
    pub decision_source: RoutingDecisionSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    pub capability: String,
    pub route_key: String,
    pub selected_provider_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matched_policy_id: Option<String>,
    pub strategy: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selection_seed: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selection_reason: Option<String>,
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
            capability: capability.into(),
            route_key: route_key.into(),
            selected_provider_id: selected_provider_id.into(),
            matched_policy_id: None,
            strategy: strategy.into(),
            selection_seed: None,
            selection_reason: None,
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

    pub fn with_matched_policy_id(mut self, matched_policy_id: impl Into<String>) -> Self {
        self.matched_policy_id = Some(matched_policy_id.into());
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderHealthSnapshot {
    pub provider_id: String,
    pub extension_id: String,
    pub runtime: String,
    pub observed_at_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    #[serde(default)]
    pub running: bool,
    #[serde(default)]
    pub healthy: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl ProviderHealthSnapshot {
    pub fn new(
        provider_id: impl Into<String>,
        extension_id: impl Into<String>,
        runtime: impl Into<String>,
        observed_at_ms: u64,
    ) -> Self {
        Self {
            provider_id: provider_id.into(),
            extension_id: extension_id.into(),
            runtime: runtime.into(),
            observed_at_ms,
            instance_id: None,
            running: false,
            healthy: false,
            message: None,
        }
    }

    pub fn with_instance_id(mut self, instance_id: impl Into<String>) -> Self {
        self.instance_id = Some(instance_id.into());
        self
    }

    pub fn with_instance_id_option(mut self, instance_id: Option<String>) -> Self {
        self.instance_id = instance_id;
        self
    }

    pub fn with_running(mut self, running: bool) -> Self {
        self.running = running;
        self
    }

    pub fn with_healthy(mut self, healthy: bool) -> Self {
        self.healthy = healthy;
        self
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    pub fn with_message_option(mut self, message: Option<String>) -> Self {
        self.message = message;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoutingPolicy {
    pub policy_id: String,
    pub capability: String,
    pub model_pattern: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub priority: i32,
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
}

impl RoutingPolicy {
    pub fn new(
        policy_id: impl Into<String>,
        capability: impl Into<String>,
        model_pattern: impl Into<String>,
    ) -> Self {
        Self {
            policy_id: policy_id.into(),
            capability: capability.into(),
            model_pattern: model_pattern.into(),
            enabled: true,
            priority: 0,
            strategy: RoutingStrategy::DeterministicPriority,
            ordered_provider_ids: Vec::new(),
            default_provider_id: None,
            max_cost: None,
            max_latency_ms: None,
            require_healthy: false,
        }
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
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

    pub fn matches(&self, capability: &str, model: &str) -> bool {
        self.capability == capability && glob_matches(&self.model_pattern, model)
    }

    pub fn rank_candidates(&self, available_provider_ids: &[String]) -> Vec<String> {
        let mut remaining = BTreeSet::new();
        for provider_id in available_provider_ids {
            remaining.insert(provider_id.clone());
        }

        let mut ranked = Vec::with_capacity(remaining.len());
        for provider_id in &self.ordered_provider_ids {
            if remaining.remove(provider_id) {
                ranked.push(provider_id.clone());
            }
        }

        if let Some(default_provider_id) = &self.default_provider_id {
            if remaining.remove(default_provider_id) {
                ranked.push(default_provider_id.clone());
            }
        }

        ranked.extend(remaining);
        ranked
    }

    pub fn declared_provider_ids(&self) -> Vec<String> {
        let mut declared = self.ordered_provider_ids.clone();
        if let Some(default_provider_id) = &self.default_provider_id {
            if !declared
                .iter()
                .any(|provider_id| provider_id == default_provider_id)
            {
                declared.push(default_provider_id.clone());
            }
        }
        declared
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

pub fn select_policy<'a>(
    policies: &'a [RoutingPolicy],
    capability: &str,
    model: &str,
) -> Option<&'a RoutingPolicy> {
    let mut matching = policies
        .iter()
        .filter(|policy| policy.enabled && policy.matches(capability, model))
        .collect::<Vec<_>>();
    matching.sort_by(|left, right| {
        right
            .priority
            .cmp(&left.priority)
            .then_with(|| left.policy_id.cmp(&right.policy_id))
    });
    matching.into_iter().next()
}

fn default_enabled() -> bool {
    true
}

fn is_false(value: &bool) -> bool {
    !*value
}

fn dedup_preserving_order(values: Vec<String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut deduped = Vec::with_capacity(values.len());
    for value in values {
        if seen.insert(value.clone()) {
            deduped.push(value);
        }
    }
    deduped
}

fn glob_matches(pattern: &str, input: &str) -> bool {
    glob_matches_bytes(pattern.as_bytes(), input.as_bytes())
}

fn glob_matches_bytes(pattern: &[u8], input: &[u8]) -> bool {
    if pattern.is_empty() {
        return input.is_empty();
    }

    match pattern[0] {
        b'*' => {
            glob_matches_bytes(&pattern[1..], input)
                || (!input.is_empty() && glob_matches_bytes(pattern, &input[1..]))
        }
        byte => {
            !input.is_empty() && byte == input[0] && glob_matches_bytes(&pattern[1..], &input[1..])
        }
    }
}
