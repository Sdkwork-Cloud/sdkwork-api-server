use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub selected_provider_id: String,
    pub candidate_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matched_policy_id: Option<String>,
}

impl RoutingDecision {
    pub fn new(selected_provider_id: impl Into<String>, candidate_ids: Vec<String>) -> Self {
        Self {
            selected_provider_id: selected_provider_id.into(),
            candidate_ids,
            matched_policy_id: None,
        }
    }

    pub fn with_matched_policy_id(mut self, matched_policy_id: impl Into<String>) -> Self {
        self.matched_policy_id = Some(matched_policy_id.into());
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
