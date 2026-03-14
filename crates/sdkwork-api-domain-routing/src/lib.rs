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
}

impl RoutingStrategy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DeterministicPriority => "deterministic_priority",
            Self::WeightedRandom => "weighted_random",
        }
    }
}

impl FromStr for RoutingStrategy {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "deterministic_priority" => Ok(Self::DeterministicPriority),
            "weighted_random" => Ok(Self::WeightedRandom),
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
