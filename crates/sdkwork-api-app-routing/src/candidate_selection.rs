use super::*;

pub(crate) struct CandidateSelection {
    pub(crate) selected_index: usize,
    pub(crate) strategy: String,
    pub(crate) selection_seed: Option<u64>,
    pub(crate) selection_reason: String,
    pub(crate) fallback_reason: Option<String>,
    pub(crate) provider_health_recovery_probe: Option<ProviderHealthRecoveryProbe>,
    pub(crate) slo_applied: bool,
    pub(crate) slo_degraded: bool,
}

struct RecoveryProbeEvaluation {
    selection: Option<CandidateSelection>,
    provider_health_recovery_probe: ProviderHealthRecoveryProbe,
}

impl From<RoutingStrategyExecutionResult> for CandidateSelection {
    fn from(value: RoutingStrategyExecutionResult) -> Self {
        Self {
            selected_index: value.selected_index,
            strategy: value.strategy,
            selection_seed: value.selection_seed,
            selection_reason: value.selection_reason,
            fallback_reason: value.fallback_reason,
            provider_health_recovery_probe: None,
            slo_applied: value.slo_applied,
            slo_degraded: value.slo_degraded,
        }
    }
}

pub(crate) async fn select_candidate(
    assessments: &mut [RoutingCandidateAssessment],
    matched_policy: Option<&RoutingPolicy>,
    requested_region: Option<&str>,
    provided_selection_seed: Option<u64>,
    recovery_probe_provider_id: Option<&str>,
    recovery_probe_lock_store: Option<&dyn DistributedLockStore>,
) -> CandidateSelection {
    let mut provider_health_recovery_probe = None;
    if let Some(evaluation) = select_recovery_probe_candidate(
        assessments,
        matched_policy,
        provided_selection_seed,
        recovery_probe_provider_id,
        recovery_probe_lock_store,
    )
    .await
    {
        if let Some(selection) = evaluation.selection {
            return selection;
        }
        provider_health_recovery_probe = Some(evaluation.provider_health_recovery_probe);
    }

    let routing_strategy = matched_policy
        .map(|policy| policy.strategy)
        .unwrap_or(RoutingStrategy::DeterministicPriority);
    let registry = builtin_routing_strategy_registry();
    let plugin = registry.resolve(routing_strategy).unwrap_or_else(|| {
        registry
            .resolve(RoutingStrategy::DeterministicPriority)
            .expect("builtin deterministic-priority routing plugin must exist")
    });

    let mut selection: CandidateSelection = plugin
        .execute(RoutingStrategyExecutionInput {
            assessments,
            matched_policy,
            requested_region,
            provided_selection_seed,
        })
        .into();
    selection.provider_health_recovery_probe = provider_health_recovery_probe;
    selection
}

async fn select_recovery_probe_candidate(
    assessments: &mut [RoutingCandidateAssessment],
    matched_policy: Option<&RoutingPolicy>,
    provided_selection_seed: Option<u64>,
    recovery_probe_provider_id: Option<&str>,
    recovery_probe_lock_store: Option<&dyn DistributedLockStore>,
) -> Option<RecoveryProbeEvaluation> {
    let recovery_probe_provider_id = recovery_probe_provider_id?;
    let routing_strategy = matched_policy
        .map(|policy| policy.strategy)
        .unwrap_or(RoutingStrategy::DeterministicPriority);
    if routing_strategy != RoutingStrategy::DeterministicPriority {
        return None;
    }

    let recovery_probe_percent = provider_health_recovery_probe_percent();
    if recovery_probe_percent == 0 {
        return None;
    }

    let selected_assessment = assessments.first()?;
    if !selected_assessment.available
        || selected_assessment.health != RoutingCandidateHealth::Healthy
    {
        return None;
    }
    if selected_assessment.provider_id == recovery_probe_provider_id {
        return None;
    }
    let displaced_provider_id = selected_assessment.provider_id.clone();

    let recovery_probe_index = assessments
        .iter()
        .position(|assessment| assessment.provider_id == recovery_probe_provider_id)?;
    let recovery_assessment = assessments.get(recovery_probe_index)?;
    if !recovery_assessment.available
        || recovery_assessment.health != RoutingCandidateHealth::Unknown
    {
        return None;
    }

    let selection_seed = provided_selection_seed.unwrap_or_else(generate_selection_seed);
    if selection_seed % 100 >= u64::from(recovery_probe_percent) {
        return None;
    }

    let selected_provider_id = recovery_assessment.provider_id.clone();
    let selected_probe = ProviderHealthRecoveryProbe::new(
        selected_provider_id.clone(),
        ProviderHealthRecoveryProbeOutcome::Selected,
    );
    let mut lease_reason_suffix = String::new();
    if let Some(lock_store) = recovery_probe_lock_store {
        let lock_ttl_ms = provider_health_recovery_probe_lock_ttl_ms();
        if lock_ttl_ms > 0 {
            let lock_scope = provider_health_recovery_probe_lock_scope(&selected_provider_id);
            let lock_owner = provider_health_recovery_probe_lock_owner(selection_seed);
            match lock_store
                .try_acquire_lock(&lock_scope, &lock_owner, lock_ttl_ms)
                .await
            {
                Ok(true) => {
                    lease_reason_suffix =
                        format!(" and the recovery probe lease was acquired for {lock_ttl_ms}ms");
                }
                Ok(false) => {
                    if let Some(assessment) = assessments.get_mut(recovery_probe_index) {
                        assessment.reasons.push(format!(
                            "skipped provider health recovery probe because lease scope {lock_scope} is already held by another request"
                        ));
                    }
                    return Some(RecoveryProbeEvaluation {
                        selection: None,
                        provider_health_recovery_probe: ProviderHealthRecoveryProbe::new(
                            selected_provider_id,
                            ProviderHealthRecoveryProbeOutcome::LeaseContended,
                        ),
                    });
                }
                Err(error) => {
                    if let Some(assessment) = assessments.get_mut(recovery_probe_index) {
                        assessment.reasons.push(format!(
                            "skipped provider health recovery probe because lease acquisition for scope {lock_scope} failed: {error}"
                        ));
                    }
                    return Some(RecoveryProbeEvaluation {
                        selection: None,
                        provider_health_recovery_probe: ProviderHealthRecoveryProbe::new(
                            selected_provider_id,
                            ProviderHealthRecoveryProbeOutcome::LeaseError,
                        ),
                    });
                }
            }
        }
    }
    if let Some(assessment) = assessments.get_mut(recovery_probe_index) {
        assessment.reasons.push(format!(
            "selected for provider health recovery probe because the stale-unhealthy primary is due for revalidation and selection seed {selection_seed} matched the {recovery_probe_percent}% recovery cohort{lease_reason_suffix}"
        ));
    }

    Some(RecoveryProbeEvaluation {
        provider_health_recovery_probe: selected_probe.clone(),
        selection: Some(CandidateSelection {
            selected_index: recovery_probe_index,
            strategy: RoutingStrategy::DeterministicPriority.as_str().to_owned(),
            selection_seed: Some(selection_seed),
            selection_reason: format!(
                "selected {selected_provider_id} for provider health recovery probe because {displaced_provider_id} would otherwise keep serving traffic while the stale-unhealthy primary remains unrevalidated and selection seed {selection_seed} matched the {recovery_probe_percent}% recovery cohort{lease_reason_suffix}"
            ),
            fallback_reason: Some(PROVIDER_HEALTH_RECOVERY_PROBE_FALLBACK_REASON.to_owned()),
            provider_health_recovery_probe: Some(selected_probe),
            slo_applied: false,
            slo_degraded: false,
        }),
    })
}

pub(crate) fn generate_decision_id(created_at_ms: u64) -> String {
    format!("route-dec-{}-{}", created_at_ms, generate_selection_seed())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn assess_candidate(
    provider_id: &str,
    policy_rank: usize,
    provider_map: &HashMap<String, ProxyProvider>,
    instances_by_provider_id: &HashMap<String, ExtensionInstance>,
    installations_by_id: &HashMap<String, ExtensionInstallation>,
    runtime_statuses: &[ExtensionRuntimeStatusRecord],
    persisted_provider_health: Option<&ProviderHealthSnapshot>,
    assessment_time_ms: u64,
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
            if persisted_provider_health_snapshot_is_fresh(snapshot, assessment_time_ms) {
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
                    .with_reason(format!(
                        "ignored stale persisted runtime health snapshot from {} at {}",
                        snapshot.runtime, snapshot.observed_at_ms
                    ));
                if let Some(message) = &snapshot.message {
                    assessment =
                        assessment.with_reason(format!("stale snapshot message = {message}"));
                }
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

fn persisted_provider_health_snapshot_is_fresh(
    snapshot: &ProviderHealthSnapshot,
    assessment_time_ms: u64,
) -> bool {
    persisted_provider_health_snapshot_is_fresh_for_ttl(
        snapshot,
        assessment_time_ms,
        persisted_provider_health_snapshot_freshness_ttl_ms(),
    )
}

fn persisted_provider_health_snapshot_is_fresh_for_ttl(
    snapshot: &ProviderHealthSnapshot,
    assessment_time_ms: u64,
    freshness_ttl_ms: u64,
) -> bool {
    assessment_time_ms.saturating_sub(snapshot.observed_at_ms) <= freshness_ttl_ms
}

fn persisted_provider_health_snapshot_freshness_ttl_ms() -> u64 {
    let configured = std::env::var(PROVIDER_HEALTH_FRESHNESS_TTL_ENV).ok();
    persisted_provider_health_snapshot_freshness_ttl_ms_from_env(configured.as_deref())
}

fn persisted_provider_health_snapshot_freshness_ttl_ms_from_env(configured: Option<&str>) -> u64 {
    configured
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(DEFAULT_PERSISTED_PROVIDER_HEALTH_FRESHNESS_TTL_MS)
}

pub(crate) fn recovery_probe_provider_id(
    candidate_ids: &[String],
    latest_provider_health: &HashMap<String, ProviderHealthSnapshot>,
    assessment_time_ms: u64,
) -> Option<String> {
    let provider_id = candidate_ids.first()?;
    let snapshot = latest_provider_health.get(provider_id)?;
    if snapshot.healthy || persisted_provider_health_snapshot_is_fresh(snapshot, assessment_time_ms)
    {
        return None;
    }
    Some(provider_id.clone())
}

fn provider_health_recovery_probe_percent() -> u8 {
    let configured = std::env::var(PROVIDER_HEALTH_RECOVERY_PROBE_PERCENT_ENV).ok();
    provider_health_recovery_probe_percent_from_env(configured.as_deref())
}

fn provider_health_recovery_probe_lock_ttl_ms() -> u64 {
    let configured = std::env::var(PROVIDER_HEALTH_RECOVERY_PROBE_LOCK_TTL_ENV).ok();
    provider_health_recovery_probe_lock_ttl_ms_from_env(configured.as_deref())
}

fn provider_health_recovery_probe_percent_from_env(configured: Option<&str>) -> u8 {
    configured
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .and_then(|value| value.parse::<u8>().ok())
        .map(|value| value.min(100))
        .unwrap_or(DEFAULT_PROVIDER_HEALTH_RECOVERY_PROBE_PERCENT)
}

fn provider_health_recovery_probe_lock_ttl_ms_from_env(configured: Option<&str>) -> u64 {
    configured
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(DEFAULT_PROVIDER_HEALTH_RECOVERY_PROBE_LOCK_TTL_MS)
}

fn provider_health_recovery_probe_lock_scope(provider_id: &str) -> String {
    format!("provider-health-recovery-probe:{provider_id}")
}

fn provider_health_recovery_probe_lock_owner(selection_seed: u64) -> String {
    format!(
        "{}:{}:{}:{}",
        service_name(),
        std::process::id(),
        current_time_millis(),
        selection_seed
    )
}

pub(crate) fn latest_provider_health_snapshots(
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

pub(crate) fn compare_assessments(
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
