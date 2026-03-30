use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{ensure, Result};
use sdkwork_api_app_extension::{
    list_extension_runtime_statuses, matching_runtime_statuses_for_provider,
    ExtensionRuntimeStatusRecord,
};
use sdkwork_api_domain_catalog::ProxyProvider;
use sdkwork_api_domain_routing::{
    select_policy, ProjectRoutingPreferences, ProviderHealthSnapshot, RoutingCandidateAssessment,
    RoutingCandidateHealth, RoutingDecision, RoutingDecisionLog, RoutingDecisionSource,
    RoutingPolicy, RoutingStrategy,
};
use sdkwork_api_extension_core::{ExtensionInstallation, ExtensionInstance};
use sdkwork_api_storage_core::AdminStore;
use serde_json::Value;

const DEFAULT_WEIGHT: u64 = 100;
const STRATEGY_STATIC_FALLBACK: &str = "static_fallback";

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
}

#[derive(Debug, Clone, Copy)]
pub struct RouteSelectionContext<'a> {
    pub decision_source: RoutingDecisionSource,
    pub tenant_id: Option<&'a str>,
    pub project_id: Option<&'a str>,
    pub requested_region: Option<&'a str>,
    pub selection_seed: Option<u64>,
}

impl<'a> RouteSelectionContext<'a> {
    pub fn new(decision_source: RoutingDecisionSource) -> Self {
        Self {
            decision_source,
            tenant_id: None,
            project_id: None,
            requested_region: None,
            selection_seed: None,
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

    pub fn with_requested_region_option(mut self, requested_region: Option<&'a str>) -> Self {
        self.requested_region = requested_region;
        self
    }

    pub fn with_selection_seed_option(mut self, selection_seed: Option<u64>) -> Self {
        self.selection_seed = selection_seed;
        self
    }
}

pub fn simulate_route(_capability: &str, _model: &str) -> Result<RoutingDecision> {
    Ok(RoutingDecision::new(
        "provider-openai-official",
        vec!["provider-openai-official".into()],
    )
    .with_strategy(STRATEGY_STATIC_FALLBACK)
    .with_selection_reason(
        "used static fallback because no catalog-backed or policy-backed candidates were available",
    ))
}

pub fn create_routing_policy(input: CreateRoutingPolicyInput<'_>) -> Result<RoutingPolicy> {
    ensure!(
        !input.policy_id.trim().is_empty(),
        "policy_id must not be empty"
    );
    ensure!(
        !input.capability.trim().is_empty(),
        "capability must not be empty"
    );
    ensure!(
        !input.model_pattern.trim().is_empty(),
        "model_pattern must not be empty"
    );

    let policy = RoutingPolicy::new(input.policy_id, input.capability, input.model_pattern)
        .with_enabled(input.enabled)
        .with_priority(input.priority)
        .with_strategy(
            input
                .strategy
                .unwrap_or(RoutingStrategy::DeterministicPriority),
        )
        .with_ordered_provider_ids(input.ordered_provider_ids.to_vec())
        .with_max_cost_option(input.max_cost)
        .with_max_latency_ms_option(input.max_latency_ms)
        .with_require_healthy(input.require_healthy);

    Ok(match input.default_provider_id {
        Some(default_provider_id) if !default_provider_id.trim().is_empty() => {
            policy.with_default_provider_id(default_provider_id)
        }
        _ => policy,
    })
}

pub async fn persist_routing_policy(
    store: &dyn AdminStore,
    policy: &RoutingPolicy,
) -> Result<RoutingPolicy> {
    store.insert_routing_policy(policy).await
}

pub async fn list_routing_policies(store: &dyn AdminStore) -> Result<Vec<RoutingPolicy>> {
    store.list_routing_policies().await
}

pub async fn list_routing_decision_logs(store: &dyn AdminStore) -> Result<Vec<RoutingDecisionLog>> {
    store.list_routing_decision_logs().await
}

pub async fn simulate_route_with_store(
    store: &dyn AdminStore,
    capability: &str,
    model: &str,
) -> Result<RoutingDecision> {
    simulate_route_with_store_context(store, capability, model, None, None).await
}

pub async fn simulate_route_with_store_seeded(
    store: &dyn AdminStore,
    capability: &str,
    model: &str,
    selection_seed: u64,
) -> Result<RoutingDecision> {
    simulate_route_with_store_context(store, capability, model, None, Some(selection_seed)).await
}

pub async fn simulate_route_with_store_context(
    store: &dyn AdminStore,
    capability: &str,
    model: &str,
    requested_region: Option<&str>,
    selection_seed: Option<u64>,
) -> Result<RoutingDecision> {
    simulate_route_with_store_selection_context(
        store,
        capability,
        model,
        RouteSelectionContext::new(RoutingDecisionSource::AdminSimulation)
            .with_requested_region_option(requested_region)
            .with_selection_seed_option(selection_seed),
    )
    .await
}

pub async fn simulate_route_with_store_selection_context(
    store: &dyn AdminStore,
    capability: &str,
    model: &str,
    context: RouteSelectionContext<'_>,
) -> Result<RoutingDecision> {
    simulate_route_with_store_inner(
        store,
        capability,
        model,
        context.project_id,
        normalize_region_option(context.requested_region),
        context.selection_seed,
    )
    .await
}

pub async fn select_route_with_store(
    store: &dyn AdminStore,
    capability: &str,
    route_key: &str,
    decision_source: RoutingDecisionSource,
    tenant_id: Option<&str>,
    project_id: Option<&str>,
    selection_seed: Option<u64>,
) -> Result<RoutingDecision> {
    select_route_with_store_context(
        store,
        capability,
        route_key,
        RouteSelectionContext::new(decision_source)
            .with_tenant_id_option(tenant_id)
            .with_project_id_option(project_id)
            .with_selection_seed_option(selection_seed),
    )
    .await
}

pub async fn select_route_with_store_context(
    store: &dyn AdminStore,
    capability: &str,
    route_key: &str,
    context: RouteSelectionContext<'_>,
) -> Result<RoutingDecision> {
    let decision =
        simulate_route_with_store_selection_context(store, capability, route_key, context).await?;
    let created_at_ms = current_time_millis();
    let log = RoutingDecisionLog::new(
        generate_decision_id(created_at_ms),
        context.decision_source,
        capability,
        route_key,
        decision.selected_provider_id.clone(),
        decision
            .strategy
            .clone()
            .unwrap_or_else(|| STRATEGY_STATIC_FALLBACK.to_owned()),
        created_at_ms,
    )
    .with_tenant_id_option(context.tenant_id.map(ToOwned::to_owned))
    .with_project_id_option(context.project_id.map(ToOwned::to_owned))
    .with_matched_policy_id_option(decision.matched_policy_id.clone())
    .with_selection_seed_option(context.selection_seed.or(decision.selection_seed))
    .with_selection_reason_option(decision.selection_reason.clone())
    .with_requested_region_option(decision.requested_region.clone())
    .with_slo_state(decision.slo_applied, decision.slo_degraded)
    .with_assessments(decision.assessments.clone());
    store.insert_routing_decision_log(&log).await?;
    Ok(decision)
}

async fn simulate_route_with_store_inner(
    store: &dyn AdminStore,
    capability: &str,
    model: &str,
    project_id: Option<&str>,
    requested_region: Option<String>,
    selection_seed: Option<u64>,
) -> Result<RoutingDecision> {
    let project_preferences = match project_id {
        Some(project_id) => store.find_project_routing_preferences(project_id).await?,
        None => None,
    };
    let requested_region = requested_region
        .or_else(|| preferred_region_from_preferences(project_preferences.as_ref()));

    let mut model_candidate_ids: Vec<String> = store
        .list_models_for_external_name(model)
        .await?
        .into_iter()
        .map(|entry| entry.provider_id)
        .collect();

    model_candidate_ids.sort();
    model_candidate_ids.dedup();

    let policies = store.list_routing_policies().await?;
    let matched_policy = select_policy(&policies, capability, model);
    let effective_policy = effective_routing_policy(
        matched_policy,
        project_preferences.as_ref(),
        capability,
        model,
    );

    let providers = if model_candidate_ids.is_empty() {
        store.list_providers().await?
    } else {
        store.list_providers_for_model(model).await?
    };
    let provider_map = providers
        .into_iter()
        .map(|provider| (provider.id.clone(), provider))
        .collect::<HashMap<_, _>>();

    let candidate_ids = if model_candidate_ids.is_empty() {
        if let Some(policy) = effective_policy.as_ref() {
            let available_provider_ids = provider_map.keys().cloned().collect::<HashSet<_>>();
            let candidate_ids = policy
                .declared_provider_ids()
                .into_iter()
                .filter(|provider_id| available_provider_ids.contains(provider_id))
                .collect::<Vec<_>>();

            if candidate_ids.is_empty() {
                return Ok(simulate_route(capability, model)?
                    .with_requested_region_option(requested_region.clone()));
            }

            candidate_ids
        } else {
            return Ok(simulate_route(capability, model)?
                .with_requested_region_option(requested_region.clone()));
        }
    } else {
        match effective_policy.as_ref() {
            Some(policy) => policy.rank_candidates(&model_candidate_ids),
            None => model_candidate_ids,
        }
    };

    let installations = store.list_extension_installations().await?;
    let installations_by_id = installations
        .into_iter()
        .map(|installation| (installation.installation_id.clone(), installation))
        .collect::<HashMap<_, _>>();
    let instances = store.list_extension_instances().await?;
    let instances_by_provider_id = instances
        .into_iter()
        .map(|instance| (instance.instance_id.clone(), instance))
        .collect::<HashMap<_, _>>();
    let runtime_statuses = list_extension_runtime_statuses()?;
    let latest_provider_health =
        latest_provider_health_snapshots(store.list_provider_health_snapshots().await?);

    let mut assessments = candidate_ids
        .iter()
        .enumerate()
        .map(|(policy_rank, provider_id)| {
            assess_candidate(
                provider_id,
                policy_rank,
                &provider_map,
                &instances_by_provider_id,
                &installations_by_id,
                &runtime_statuses,
                latest_provider_health.get(provider_id),
            )
        })
        .collect::<Vec<_>>();
    assessments.sort_by(compare_assessments);

    let selection = select_candidate(
        &mut assessments,
        effective_policy.as_ref(),
        requested_region.as_deref(),
        selection_seed,
    );
    let selected = assessments
        .get(selection.selected_index)
        .map(|assessment| assessment.provider_id.clone())
        .unwrap_or_else(|| candidate_ids[0].clone());

    let ranked_candidate_ids = assessments
        .iter()
        .map(|assessment| assessment.provider_id.clone())
        .collect::<Vec<_>>();

    let mut decision = RoutingDecision::new(selected, ranked_candidate_ids)
        .with_strategy(selection.strategy)
        .with_selection_reason(selection.selection_reason)
        .with_requested_region_option(requested_region)
        .with_slo_state(selection.slo_applied, selection.slo_degraded)
        .with_assessments(assessments);
    if let Some(selection_seed) = selection.selection_seed {
        decision = decision.with_selection_seed(selection_seed);
    }
    Ok(match matched_policy {
        Some(policy) => decision.with_matched_policy_id(policy.policy_id.clone()),
        None => decision,
    })
}

fn preferred_region_from_preferences(
    preferences: Option<&ProjectRoutingPreferences>,
) -> Option<String> {
    preferences.and_then(|preferences| preferences.preferred_region.clone())
}

fn effective_routing_policy(
    matched_policy: Option<&RoutingPolicy>,
    preferences: Option<&ProjectRoutingPreferences>,
    capability: &str,
    model: &str,
) -> Option<RoutingPolicy> {
    let Some(preferences) = preferences else {
        return matched_policy.cloned();
    };

    let base_policy = matched_policy
        .cloned()
        .unwrap_or_else(|| RoutingPolicy::new("project-routing-preferences", capability, model));
    let base_ordered_provider_ids = base_policy.ordered_provider_ids.clone();
    let base_default_provider_id = base_policy.default_provider_id.clone();
    let base_max_cost = base_policy.max_cost;
    let base_max_latency_ms = base_policy.max_latency_ms;
    let base_require_healthy = base_policy.require_healthy;

    Some(
        base_policy
            .with_strategy(preferences.strategy)
            .with_ordered_provider_ids(if preferences.ordered_provider_ids.is_empty() {
                base_ordered_provider_ids
            } else {
                preferences.ordered_provider_ids.clone()
            })
            .with_default_provider_id_option(
                preferences
                    .default_provider_id
                    .clone()
                    .or(base_default_provider_id),
            )
            .with_max_cost_option(combine_optional_f64(base_max_cost, preferences.max_cost))
            .with_max_latency_ms_option(combine_optional_u64(
                base_max_latency_ms,
                preferences.max_latency_ms,
            ))
            .with_require_healthy(base_require_healthy || preferences.require_healthy),
    )
}

fn combine_optional_f64(base: Option<f64>, overlay: Option<f64>) -> Option<f64> {
    match (base, overlay) {
        (Some(base), Some(overlay)) => Some(base.min(overlay)),
        (Some(base), None) => Some(base),
        (None, Some(overlay)) => Some(overlay),
        (None, None) => None,
    }
}

fn combine_optional_u64(base: Option<u64>, overlay: Option<u64>) -> Option<u64> {
    match (base, overlay) {
        (Some(base), Some(overlay)) => Some(base.min(overlay)),
        (Some(base), None) => Some(base),
        (None, Some(overlay)) => Some(overlay),
        (None, None) => None,
    }
}

struct CandidateSelection {
    selected_index: usize,
    strategy: String,
    selection_seed: Option<u64>,
    selection_reason: String,
    slo_applied: bool,
    slo_degraded: bool,
}

fn select_candidate(
    assessments: &mut [RoutingCandidateAssessment],
    matched_policy: Option<&RoutingPolicy>,
    requested_region: Option<&str>,
    provided_selection_seed: Option<u64>,
) -> CandidateSelection {
    let routing_strategy = matched_policy
        .map(|policy| policy.strategy)
        .unwrap_or(RoutingStrategy::DeterministicPriority);
    match routing_strategy {
        RoutingStrategy::DeterministicPriority => {
            let selected_index = 0;
            let selection_reason = assessments
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
            CandidateSelection {
                selected_index,
                strategy: RoutingStrategy::DeterministicPriority.as_str().to_owned(),
                selection_seed: None,
                selection_reason,
                slo_applied: false,
                slo_degraded: false,
            }
        }
        RoutingStrategy::WeightedRandom => {
            let selection_seed = provided_selection_seed.unwrap_or_else(generate_selection_seed);
            let (selected_index, selection_reason) =
                select_weighted_candidate(assessments, selection_seed);
            CandidateSelection {
                selected_index,
                strategy: RoutingStrategy::WeightedRandom.as_str().to_owned(),
                selection_seed: Some(selection_seed),
                selection_reason,
                slo_applied: false,
                slo_degraded: false,
            }
        }
        RoutingStrategy::SloAware => select_slo_aware_candidate(assessments, matched_policy),
        RoutingStrategy::GeoAffinity => {
            select_geo_affinity_candidate(assessments, requested_region)
        }
    }
}

fn select_geo_affinity_candidate(
    assessments: &mut [RoutingCandidateAssessment],
    requested_region: Option<&str>,
) -> CandidateSelection {
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
        return CandidateSelection {
            selected_index,
            strategy: RoutingStrategy::GeoAffinity.as_str().to_owned(),
            selection_seed: None,
            selection_reason,
            slo_applied: false,
            slo_degraded: false,
        };
    };

    let has_healthy_available_candidate = assessments.iter().any(|assessment| {
        assessment.available && assessment.health == RoutingCandidateHealth::Healthy
    });

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
        CandidateSelection {
            selected_index,
            strategy: RoutingStrategy::GeoAffinity.as_str().to_owned(),
            selection_seed: None,
            selection_reason: format!(
                "selected {provider_id} as the top-ranked candidate matching requested region {requested_region}"
            ),
            slo_applied: false,
            slo_degraded: false,
        }
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
        CandidateSelection {
            selected_index,
            strategy: RoutingStrategy::GeoAffinity.as_str().to_owned(),
            selection_seed: None,
            selection_reason: format!(
                "selected {provider_id} as the top-ranked fallback because no candidate matched requested region {requested_region}"
            ),
            slo_applied: false,
            slo_degraded: false,
        }
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
        CandidateSelection {
            selected_index,
            strategy: RoutingStrategy::GeoAffinity.as_str().to_owned(),
            selection_seed: None,
            selection_reason,
            slo_applied: false,
            slo_degraded: false,
        }
    }
}

fn select_slo_aware_candidate(
    assessments: &mut [RoutingCandidateAssessment],
    matched_policy: Option<&RoutingPolicy>,
) -> CandidateSelection {
    let Some(policy) = matched_policy else {
        return CandidateSelection {
            selected_index: 0,
            strategy: RoutingStrategy::SloAware.as_str().to_owned(),
            selection_seed: None,
            selection_reason: "selected the top-ranked candidate because no routing policy was available for SLO evaluation".to_owned(),
            slo_applied: false,
            slo_degraded: false,
        };
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
        return CandidateSelection {
            selected_index: 0,
            strategy: RoutingStrategy::SloAware.as_str().to_owned(),
            selection_seed: None,
            selection_reason,
            slo_applied: false,
            slo_degraded: false,
        };
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
        CandidateSelection {
            selected_index,
            strategy: RoutingStrategy::SloAware.as_str().to_owned(),
            selection_seed: None,
            selection_reason: format!(
                "selected {provider_id} as the top-ranked SLO-compliant candidate"
            ),
            slo_applied: true,
            slo_degraded: false,
        }
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
        CandidateSelection {
            selected_index,
            strategy: RoutingStrategy::SloAware.as_str().to_owned(),
            selection_seed: None,
            selection_reason: format!(
                "degraded to {provider_id} because no candidate satisfied the active SLO thresholds"
            ),
            slo_applied: true,
            slo_degraded: true,
        }
    }
}

fn select_weighted_candidate(
    assessments: &mut [RoutingCandidateAssessment],
    selection_seed: u64,
) -> (usize, String) {
    let has_healthy_available_candidate = assessments.iter().any(|assessment| {
        assessment.available && assessment.health == RoutingCandidateHealth::Healthy
    });

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
        return (selected_index, selection_reason);
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

fn current_time_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| u64::try_from(duration.as_millis()).unwrap_or(u64::MAX))
        .unwrap_or(0)
}

fn generate_decision_id(created_at_ms: u64) -> String {
    format!("route-dec-{}-{}", created_at_ms, generate_selection_seed())
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

fn assess_candidate(
    provider_id: &str,
    policy_rank: usize,
    provider_map: &HashMap<String, ProxyProvider>,
    instances_by_provider_id: &HashMap<String, ExtensionInstance>,
    installations_by_id: &HashMap<String, ExtensionInstallation>,
    runtime_statuses: &[ExtensionRuntimeStatusRecord],
    persisted_provider_health: Option<&ProviderHealthSnapshot>,
) -> RoutingCandidateAssessment {
    let mut assessment = RoutingCandidateAssessment::new(provider_id).with_policy_rank(policy_rank);

    let Some(provider) = provider_map.get(provider_id) else {
        return assessment
            .with_available(false)
            .with_health(RoutingCandidateHealth::Unknown)
            .with_reason("provider record is missing from the catalog");
    };

    let instance = instances_by_provider_id.get(provider_id);
    let mut available = true;

    if let Some(instance) = instance {
        match installations_by_id.get(&instance.installation_id) {
            Some(installation) => {
                if !installation.enabled {
                    available = false;
                    assessment = assessment.with_reason("extension installation is disabled");
                }
            }
            None => {
                assessment = assessment.with_reason(
                    "matching extension instance has no installation record, direct provider fallback may apply",
                );
            }
        }

        if !instance.enabled {
            available = false;
            assessment = assessment.with_reason("matching extension instance is disabled");
        }

        if let Some(weight) = routing_hint_u64(&instance.config, "weight") {
            assessment = assessment.with_weight(weight);
        }
        if let Some(cost) = routing_hint_f64(&instance.config, "cost") {
            assessment = assessment.with_cost(cost);
        }
        if let Some(latency_ms) = routing_hint_u64(&instance.config, "latency_ms") {
            assessment = assessment.with_latency_ms(latency_ms);
        }
        if let Some(region) = routing_hint_string(&instance.config, "region") {
            assessment = assessment.with_region(region);
        }
    } else {
        assessment =
            assessment.with_reason("no matching extension instance is mounted for this provider");
    }

    assessment = assessment.with_available(available);

    let matching_statuses =
        matching_runtime_statuses_for_provider(provider, instance, runtime_statuses);
    if matching_statuses.is_empty() {
        if let Some(snapshot) = persisted_provider_health {
            let health = if snapshot.healthy {
                RoutingCandidateHealth::Healthy
            } else {
                RoutingCandidateHealth::Unhealthy
            };
            assessment = assessment.with_health(health).with_reason(format!(
                "used persisted runtime health snapshot from {} at {}",
                snapshot.runtime, snapshot.observed_at_ms
            ));
            if let Some(message) = &snapshot.message {
                assessment = assessment.with_reason(format!("snapshot message = {message}"));
            }
        } else {
            assessment = assessment
                .with_health(RoutingCandidateHealth::Unknown)
                .with_reason("no runtime health signal is available");
        }
    } else if matching_statuses.iter().any(|status| status.healthy) {
        let runtime = matching_statuses[0].runtime.as_str();
        assessment = assessment
            .with_health(RoutingCandidateHealth::Healthy)
            .with_reason(format!("healthy runtime signal from {runtime}"));
    } else {
        let runtime = matching_statuses[0].runtime.as_str();
        assessment = assessment
            .with_health(RoutingCandidateHealth::Unhealthy)
            .with_reason(format!("runtime signal from {runtime} is unhealthy"));
    }

    if assessment.weight.is_none() {
        assessment = assessment.with_reason("default routing weight applies");
    }
    if let Some(cost) = assessment.cost {
        assessment = assessment.with_reason(format!("cost hint = {cost}"));
    }
    if let Some(latency_ms) = assessment.latency_ms {
        assessment = assessment.with_reason(format!("latency hint = {latency_ms}ms"));
    }
    if let Some(region) = assessment.region.clone() {
        assessment = assessment.with_reason(format!("region hint = {region}"));
    }

    assessment
}

fn latest_provider_health_snapshots(
    snapshots: Vec<ProviderHealthSnapshot>,
) -> HashMap<String, ProviderHealthSnapshot> {
    let mut latest = HashMap::new();
    for snapshot in snapshots {
        latest
            .entry(snapshot.provider_id.clone())
            .or_insert(snapshot);
    }
    latest
}

fn compare_assessments(
    left: &RoutingCandidateAssessment,
    right: &RoutingCandidateAssessment,
) -> Ordering {
    right
        .available
        .cmp(&left.available)
        .then_with(|| health_rank(&right.health).cmp(&health_rank(&left.health)))
        .then_with(|| left.policy_rank.cmp(&right.policy_rank))
        .then_with(|| compare_option_f64_asc(left.cost, right.cost))
        .then_with(|| compare_option_u64_asc(left.latency_ms, right.latency_ms))
        .then_with(|| {
            compare_option_u64_desc(
                Some(resolved_weight(left.weight)),
                Some(resolved_weight(right.weight)),
            )
        })
        .then_with(|| left.provider_id.cmp(&right.provider_id))
}

fn health_rank(health: &RoutingCandidateHealth) -> u8 {
    match health {
        RoutingCandidateHealth::Healthy => 2,
        RoutingCandidateHealth::Unknown => 1,
        RoutingCandidateHealth::Unhealthy => 0,
    }
}

fn resolved_weight(weight: Option<u64>) -> u64 {
    weight.unwrap_or(DEFAULT_WEIGHT)
}

fn compare_option_f64_asc(left: Option<f64>, right: Option<f64>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left.partial_cmp(&right).unwrap_or(Ordering::Equal),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn compare_option_u64_asc(left: Option<u64>, right: Option<u64>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left.cmp(&right),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn compare_option_u64_desc(left: Option<u64>, right: Option<u64>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => right.cmp(&left),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn routing_hint_u64(config: &Value, key: &str) -> Option<u64> {
    config
        .get("routing")
        .and_then(|routing| routing.get(key))
        .and_then(Value::as_u64)
        .or_else(|| config.get(key).and_then(Value::as_u64))
}

fn routing_hint_f64(config: &Value, key: &str) -> Option<f64> {
    config
        .get("routing")
        .and_then(|routing| routing.get(key))
        .and_then(Value::as_f64)
        .or_else(|| config.get(key).and_then(Value::as_f64))
}

fn routing_hint_string(config: &Value, key: &str) -> Option<String> {
    config
        .get("routing")
        .and_then(|routing| routing.get(key))
        .and_then(Value::as_str)
        .or_else(|| config.get(key).and_then(Value::as_str))
        .and_then(normalize_region)
}

fn normalize_region_option(region: Option<&str>) -> Option<String> {
    region.and_then(normalize_region)
}

fn normalize_region(region: &str) -> Option<String> {
    let normalized = region.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}
