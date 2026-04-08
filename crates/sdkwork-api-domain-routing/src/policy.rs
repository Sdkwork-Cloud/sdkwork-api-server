use super::*;

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
    #[serde(default = "default_enabled")]
    pub execution_failover_enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upstream_retry_max_attempts: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upstream_retry_base_delay_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upstream_retry_max_delay_ms: Option<u64>,
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
            execution_failover_enabled: true,
            upstream_retry_max_attempts: None,
            upstream_retry_base_delay_ms: None,
            upstream_retry_max_delay_ms: None,
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

    pub fn with_execution_failover_enabled(mut self, execution_failover_enabled: bool) -> Self {
        self.execution_failover_enabled = execution_failover_enabled;
        self
    }

    pub fn with_upstream_retry_max_attempts(mut self, upstream_retry_max_attempts: u32) -> Self {
        self.upstream_retry_max_attempts = Some(upstream_retry_max_attempts);
        self
    }

    pub fn with_upstream_retry_max_attempts_option(
        mut self,
        upstream_retry_max_attempts: Option<u32>,
    ) -> Self {
        self.upstream_retry_max_attempts = upstream_retry_max_attempts;
        self
    }

    pub fn with_upstream_retry_base_delay_ms(mut self, upstream_retry_base_delay_ms: u64) -> Self {
        self.upstream_retry_base_delay_ms = Some(upstream_retry_base_delay_ms);
        self
    }

    pub fn with_upstream_retry_base_delay_ms_option(
        mut self,
        upstream_retry_base_delay_ms: Option<u64>,
    ) -> Self {
        self.upstream_retry_base_delay_ms = upstream_retry_base_delay_ms;
        self
    }

    pub fn with_upstream_retry_max_delay_ms(mut self, upstream_retry_max_delay_ms: u64) -> Self {
        self.upstream_retry_max_delay_ms = Some(upstream_retry_max_delay_ms);
        self
    }

    pub fn with_upstream_retry_max_delay_ms_option(
        mut self,
        upstream_retry_max_delay_ms: Option<u64>,
    ) -> Self {
        self.upstream_retry_max_delay_ms = upstream_retry_max_delay_ms;
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
