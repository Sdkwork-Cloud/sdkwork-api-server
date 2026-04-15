use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use sdkwork_api_domain_routing::{
    RoutingCandidateAssessment, RoutingCandidateHealth, RoutingPolicy, RoutingStrategy,
};

const DEFAULT_WEIGHT: u64 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoutingStrategyPluginMetadata {
    pub plugin_id: &'static str,
    pub strategy: RoutingStrategy,
}

pub struct RoutingStrategyExecutionInput<'a> {
    pub assessments: &'a mut [RoutingCandidateAssessment],
    pub matched_policy: Option<&'a RoutingPolicy>,
    pub requested_region: Option<&'a str>,
    pub provided_selection_seed: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingStrategyExecutionResult {
    pub selected_index: usize,
    pub strategy: String,
    pub selection_seed: Option<u64>,
    pub selection_reason: String,
    pub fallback_reason: Option<String>,
    pub slo_applied: bool,
    pub slo_degraded: bool,
}

impl RoutingStrategyExecutionResult {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        selected_index: usize,
        strategy: impl Into<String>,
        selection_seed: Option<u64>,
        selection_reason: impl Into<String>,
        fallback_reason: Option<String>,
        slo_applied: bool,
        slo_degraded: bool,
    ) -> Self {
        Self {
            selected_index,
            strategy: strategy.into(),
            selection_seed,
            selection_reason: selection_reason.into(),
            fallback_reason,
            slo_applied,
            slo_degraded,
        }
    }
}

pub trait RoutingStrategyPlugin: Send + Sync {
    fn metadata(&self) -> RoutingStrategyPluginMetadata;

    fn execute(
        &self,
        input: RoutingStrategyExecutionInput<'_>,
    ) -> RoutingStrategyExecutionResult;
}

#[derive(Default)]
pub struct RoutingStrategyPluginRegistry {
    plugins: HashMap<&'static str, Arc<dyn RoutingStrategyPlugin>>,
}

impl RoutingStrategyPluginRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<P>(&mut self, plugin: P) -> Option<Arc<dyn RoutingStrategyPlugin>>
    where
        P: RoutingStrategyPlugin + 'static,
    {
        self.register_arc(Arc::new(plugin))
    }

    pub fn register_arc(
        &mut self,
        plugin: Arc<dyn RoutingStrategyPlugin>,
    ) -> Option<Arc<dyn RoutingStrategyPlugin>> {
        let metadata = plugin.metadata();
        self.plugins.insert(metadata.strategy.as_str(), plugin)
    }

    pub fn resolve(
        &self,
        strategy: RoutingStrategy,
    ) -> Option<Arc<dyn RoutingStrategyPlugin>> {
        self.plugins.get(strategy.as_str()).cloned()
    }
}

pub fn builtin_routing_strategy_registry() -> RoutingStrategyPluginRegistry {
    let mut registry = RoutingStrategyPluginRegistry::new();
    registry.register(DeterministicPriorityPlugin);
    registry.register(WeightedRandomPlugin);
    registry.register(SloAwarePlugin);
    registry.register(GeoAffinityPlugin);
    registry
}

struct DeterministicPriorityPlugin;

impl RoutingStrategyPlugin for DeterministicPriorityPlugin {
    fn metadata(&self) -> RoutingStrategyPluginMetadata {
        RoutingStrategyPluginMetadata {
            plugin_id: "sdkwork.routing.strategy.deterministic_priority",
            strategy: RoutingStrategy::DeterministicPriority,
        }
    }

    fn execute(
        &self,
        input: RoutingStrategyExecutionInput<'_>,
    ) -> RoutingStrategyExecutionResult {
        let selected_index = 0;
        let selection_reason = input
            .assessments
            .get_mut(selected_index)
            .map(|assessment| {
                assessment.reasons.push(
                    "selected as the first healthy available provider in configured priority order"
                        .to_owned(),
                );
                selected_assessment_reason(assessment)
            })
            .unwrap_or_else(|| {
                "selected the first healthy available candidate in configured priority order"
                    .to_owned()
            });

        RoutingStrategyExecutionResult::new(
            selected_index,
            RoutingStrategy::DeterministicPriority.as_str(),
            None,
            selection_reason,
            None,
            false,
            false,
        )
    }
}

struct WeightedRandomPlugin;

impl RoutingStrategyPlugin for WeightedRandomPlugin {
    fn metadata(&self) -> RoutingStrategyPluginMetadata {
        RoutingStrategyPluginMetadata {
            plugin_id: "sdkwork.routing.strategy.weighted_random",
            strategy: RoutingStrategy::WeightedRandom,
        }
    }

    fn execute(
        &self,
        input: RoutingStrategyExecutionInput<'_>,
    ) -> RoutingStrategyExecutionResult {
        let selection_seed = input
            .provided_selection_seed
            .unwrap_or_else(generate_selection_seed);
        let (selected_index, selection_reason, fallback_reason) =
            select_weighted_candidate(input.assessments, selection_seed);

        RoutingStrategyExecutionResult::new(
            selected_index,
            RoutingStrategy::WeightedRandom.as_str(),
            Some(selection_seed),
            selection_reason,
            fallback_reason,
            false,
            false,
        )
    }
}

struct GeoAffinityPlugin;

impl RoutingStrategyPlugin for GeoAffinityPlugin {
    fn metadata(&self) -> RoutingStrategyPluginMetadata {
        RoutingStrategyPluginMetadata {
            plugin_id: "sdkwork.routing.strategy.geo_affinity",
            strategy: RoutingStrategy::GeoAffinity,
        }
    }

    fn execute(
        &self,
        input: RoutingStrategyExecutionInput<'_>,
    ) -> RoutingStrategyExecutionResult {
        select_geo_affinity_candidate(input.assessments, input.requested_region)
    }
}

struct SloAwarePlugin;

impl RoutingStrategyPlugin for SloAwarePlugin {
    fn metadata(&self) -> RoutingStrategyPluginMetadata {
        RoutingStrategyPluginMetadata {
            plugin_id: "sdkwork.routing.strategy.slo_aware",
            strategy: RoutingStrategy::SloAware,
        }
    }

    fn execute(
        &self,
        input: RoutingStrategyExecutionInput<'_>,
    ) -> RoutingStrategyExecutionResult {
        select_slo_aware_candidate(input.assessments, input.matched_policy)
    }
}

fn select_geo_affinity_candidate(
    assessments: &mut [RoutingCandidateAssessment],
    requested_region: Option<&str>,
) -> RoutingStrategyExecutionResult {
    let Some(requested_region) = requested_region else {
        let selected_index = 0;
        let selection_reason = assessments
            .get_mut(selected_index)
            .map(|assessment| {
                assessment.reasons.push(
                    "selected as the top-ranked candidate because no requested region was provided"
                        .to_owned(),
                );
                format!(
                    "selected {} as the top-ranked candidate because no requested region was provided",
                    assessment.provider_id
                )
            })
            .unwrap_or_else(|| "geo-affinity routing had no assessed candidates".to_owned());

        return RoutingStrategyExecutionResult::new(
            selected_index,
            RoutingStrategy::GeoAffinity.as_str(),
            None,
            selection_reason,
            Some("no requested region was provided".to_owned()),
            false,
            false,
        );
    };

    let has_healthy_available_candidate = assessments
        .iter()
        .any(|assessment| assessment.available && assessment.health == RoutingCandidateHealth::Healthy);

    let mut eligible_indices = Vec::new();
    let mut matching_indices = Vec::new();
    for (index, assessment) in assessments.iter_mut().enumerate() {
        let region_match = assessment.region.as_deref() == Some(requested_region);
        assessment.region_match = Some(region_match);

        if !assessment.available {
            assessment.reasons.push(
                "excluded from geo-affinity selection because candidate is unavailable".to_owned(),
            );
            continue;
        }
        if has_healthy_available_candidate && assessment.health == RoutingCandidateHealth::Unhealthy
        {
            assessment.reasons.push(
                "excluded from geo-affinity selection because a healthy candidate is available"
                    .to_owned(),
            );
            continue;
        }

        eligible_indices.push(index);
        if region_match {
            assessment.reasons.push(format!(
                "eligible for geo-affinity selection because region matches requested region {requested_region}"
            ));
            matching_indices.push(index);
        } else if let Some(region) = assessment.region.as_deref() {
            assessment.reasons.push(format!(
                "excluded from geo-affinity selection because region {region} does not match requested region {requested_region}"
            ));
        } else {
            assessment.reasons.push(format!(
                "excluded from geo-affinity selection because candidate region is unknown for requested region {requested_region}"
            ));
        }
    }

    if let Some(&selected_index) = matching_indices.first() {
        let provider_id = assessments
            .get(selected_index)
            .map(|assessment| assessment.provider_id.clone())
            .unwrap_or_default();
        if let Some(assessment) = assessments.get_mut(selected_index) {
            assessment.reasons.push(format!(
                "selected as the top-ranked candidate matching requested region {requested_region}"
            ));
        }

        RoutingStrategyExecutionResult::new(
            selected_index,
            RoutingStrategy::GeoAffinity.as_str(),
            None,
            format!(
                "selected {provider_id} as the top-ranked candidate matching requested region {requested_region}"
            ),
            None,
            false,
            false,
        )
    } else if let Some(&selected_index) = eligible_indices.first() {
        let provider_id = assessments
            .get(selected_index)
            .map(|assessment| assessment.provider_id.clone())
            .unwrap_or_default();
        if let Some(assessment) = assessments.get_mut(selected_index) {
            assessment.reasons.push(format!(
                "selected as the top-ranked fallback because no candidate matched requested region {requested_region}"
            ));
        }

        RoutingStrategyExecutionResult::new(
            selected_index,
            RoutingStrategy::GeoAffinity.as_str(),
            None,
            format!(
                "selected {provider_id} as the top-ranked fallback because no candidate matched requested region {requested_region}"
            ),
            Some(format!(
                "no candidate matched requested region {requested_region}"
            )),
            false,
            false,
        )
    } else {
        let selected_index = 0;
        let selection_reason = assessments
            .get_mut(selected_index)
            .map(|assessment| {
                assessment.reasons.push(format!(
                    "selected as the top-ranked fallback because geo-affinity had no eligible candidate for requested region {requested_region}"
                ));
                format!(
                    "selected {} because geo-affinity had no eligible candidate for requested region {requested_region}",
                    assessment.provider_id
                )
            })
            .unwrap_or_else(|| {
                format!(
                    "geo-affinity routing had no eligible candidate for requested region {requested_region}"
                )
            });

        RoutingStrategyExecutionResult::new(
            selected_index,
            RoutingStrategy::GeoAffinity.as_str(),
            None,
            selection_reason,
            Some(format!(
                "geo-affinity had no eligible candidate for requested region {requested_region}"
            )),
            false,
            false,
        )
    }
}

fn select_slo_aware_candidate(
    assessments: &mut [RoutingCandidateAssessment],
    matched_policy: Option<&RoutingPolicy>,
) -> RoutingStrategyExecutionResult {
    let Some(policy) = matched_policy else {
        return RoutingStrategyExecutionResult::new(
            0,
            RoutingStrategy::SloAware.as_str(),
            None,
            "selected the top-ranked candidate because no routing policy was available for SLO evaluation",
            Some("no routing policy was available for SLO evaluation".to_owned()),
            false,
            false,
        );
    };

    let slo_applied =
        policy.max_cost.is_some() || policy.max_latency_ms.is_some() || policy.require_healthy;
    if !slo_applied {
        for assessment in assessments.iter_mut() {
            assessment.slo_eligible = Some(true);
        }

        let selection_reason = assessments
            .first()
            .map(|assessment| {
                format!(
                    "selected {} as the top-ranked candidate because the slo_aware policy had no active SLO thresholds",
                    assessment.provider_id
                )
            })
            .unwrap_or_else(|| "slo_aware policy had no assessed candidates".to_owned());

        return RoutingStrategyExecutionResult::new(
            0,
            RoutingStrategy::SloAware.as_str(),
            None,
            selection_reason,
            None,
            false,
            false,
        );
    }

    let mut eligible_indices = Vec::new();
    for (index, assessment) in assessments.iter_mut().enumerate() {
        let violations = slo_violations_for_candidate(assessment, policy);
        assessment.slo_violations = violations.clone();
        assessment.slo_eligible = Some(violations.is_empty());
        if violations.is_empty() {
            assessment
                .reasons
                .push("eligible for active SLO thresholds".to_owned());
            eligible_indices.push(index);
        } else {
            assessment.reasons.push(format!(
                "excluded from SLO-aware selection because {}",
                violations.join(", ")
            ));
        }
    }

    if let Some(&selected_index) = eligible_indices.first() {
        let provider_id = assessments
            .get(selected_index)
            .map(|assessment| assessment.provider_id.clone())
            .unwrap_or_default();
        if let Some(assessment) = assessments.get_mut(selected_index) {
            assessment
                .reasons
                .push("selected as the top-ranked SLO-compliant candidate".to_owned());
        }

        RoutingStrategyExecutionResult::new(
            selected_index,
            RoutingStrategy::SloAware.as_str(),
            None,
            format!("selected {provider_id} as the top-ranked SLO-compliant candidate"),
            None,
            true,
            false,
        )
    } else {
        let selected_index = 0;
        let provider_id = assessments
            .get(selected_index)
            .map(|assessment| assessment.provider_id.clone())
            .unwrap_or_default();
        if let Some(assessment) = assessments.get_mut(selected_index) {
            assessment.reasons.push(
                "selected as a degraded fallback because no candidate satisfied the active SLO thresholds"
                    .to_owned(),
            );
        }

        RoutingStrategyExecutionResult::new(
            selected_index,
            RoutingStrategy::SloAware.as_str(),
            None,
            format!(
                "degraded to {provider_id} because no candidate satisfied the active SLO thresholds"
            ),
            Some("no candidate satisfied the active SLO thresholds".to_owned()),
            true,
            true,
        )
    }
}

fn select_weighted_candidate(
    assessments: &mut [RoutingCandidateAssessment],
    selection_seed: u64,
) -> (usize, String, Option<String>) {
    let has_healthy_available_candidate = assessments
        .iter()
        .any(|assessment| assessment.available && assessment.health == RoutingCandidateHealth::Healthy);

    let mut eligible = Vec::new();
    for (index, assessment) in assessments.iter_mut().enumerate() {
        let resolved_weight = resolved_weight(assessment.weight);
        if !assessment.available {
            assessment
                .reasons
                .push("excluded from weighted selection because candidate is unavailable".into());
            continue;
        }
        if has_healthy_available_candidate && assessment.health == RoutingCandidateHealth::Unhealthy
        {
            assessment.reasons.push(
                "excluded from weighted selection because a healthier candidate is available"
                    .into(),
            );
            continue;
        }
        if resolved_weight == 0 {
            assessment
                .reasons
                .push("excluded from weighted selection because resolved weight is 0".into());
            continue;
        }

        assessment.reasons.push(format!(
            "eligible for weighted selection with resolved weight = {resolved_weight}"
        ));
        eligible.push((index, resolved_weight));
    }

    if eligible.is_empty() {
        let selected_index = 0;
        let selection_reason = assessments
            .get_mut(selected_index)
            .map(|assessment| {
                assessment.reasons.push(
                    "weighted routing fell back to the top-ranked candidate because no candidate was eligible".into(),
                );
                format!(
                    "selected {} because weighted routing had no eligible candidate",
                    assessment.provider_id
                )
            })
            .unwrap_or_else(|| {
                "weighted routing had no eligible candidate and no assessed candidates".to_owned()
            });
        return (
            selected_index,
            selection_reason,
            Some("weighted routing had no eligible candidate".to_owned()),
        );
    }

    let total_weight = eligible.iter().map(|(_, weight)| *weight).sum::<u64>();
    let bucket = selection_seed % total_weight;
    let mut cumulative_weight = 0_u64;
    let mut selected_index = eligible[0].0;
    for (index, weight) in eligible {
        cumulative_weight += weight;
        if bucket < cumulative_weight {
            selected_index = index;
            break;
        }
    }

    let selected_provider_id = assessments
        .get(selected_index)
        .map(|assessment| assessment.provider_id.clone())
        .unwrap_or_default();
    if let Some(selected_assessment) = assessments.get_mut(selected_index) {
        selected_assessment.reasons.push(format!(
            "selected by weighted seed {selection_seed} across total eligible weight {total_weight}"
        ));
    }

    (
        selected_index,
        format!(
            "selected {selected_provider_id} by weighted seed {selection_seed} across total eligible weight {total_weight}"
        ),
        None,
    )
}

fn selected_assessment_reason(assessment: &RoutingCandidateAssessment) -> String {
    if assessment.reasons.is_empty() {
        format!(
            "selected {} as the top-ranked candidate",
            assessment.provider_id
        )
    } else {
        format!(
            "selected {} because {}",
            assessment.provider_id,
            assessment.reasons.join(", ")
        )
    }
}

fn generate_selection_seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| {
            let nanos = duration.as_nanos();
            (nanos ^ u128::from(std::process::id())) as u64
        })
        .unwrap_or_else(|_| u64::from(std::process::id()))
}

fn slo_violations_for_candidate(
    assessment: &RoutingCandidateAssessment,
    policy: &RoutingPolicy,
) -> Vec<String> {
    let mut violations = Vec::new();

    if !assessment.available {
        violations.push("candidate is unavailable".to_owned());
    }

    if policy.require_healthy && assessment.health != RoutingCandidateHealth::Healthy {
        violations.push("health requirement not satisfied".to_owned());
    }

    if let Some(max_cost) = policy.max_cost {
        match assessment.cost {
            Some(cost) if cost > max_cost => {
                violations.push(format!("cost {cost} exceeds max_cost {max_cost}"));
            }
            Some(_) => {}
            None => violations.push("cost evidence missing for required SLO threshold".to_owned()),
        }
    }

    if let Some(max_latency_ms) = policy.max_latency_ms {
        match assessment.latency_ms {
            Some(latency_ms) if latency_ms > max_latency_ms => {
                violations.push(format!(
                    "latency {latency_ms}ms exceeds max_latency_ms {max_latency_ms}"
                ));
            }
            Some(_) => {}
            None => {
                violations.push("latency evidence missing for required SLO threshold".to_owned())
            }
        }
    }

    violations
}

fn resolved_weight(weight: Option<u64>) -> u64 {
    weight.unwrap_or(DEFAULT_WEIGHT)
}
