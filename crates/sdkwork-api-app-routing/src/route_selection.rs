use super::*;

const POLICY_CANDIDATE_UNAVAILABLE_FALLBACK_REASON: &str = "policy_candidate_unavailable";
const NO_CATALOG_OR_POLICY_CANDIDATES_FALLBACK_REASON: &str =
    "no catalog-backed or policy-backed candidates were available";

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
        context.tenant_id,
        context.project_id,
        context.api_key_group_id,
        normalize_region_option(context.requested_region),
        context.selection_seed,
        context.recovery_probe_lock_store,
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
    .with_api_key_group_id_option(context.api_key_group_id.map(ToOwned::to_owned))
    .with_matched_policy_id_option(decision.matched_policy_id.clone())
    .with_applied_routing_profile_id_option(decision.applied_routing_profile_id.clone())
    .with_compiled_routing_snapshot_id_option(decision.compiled_routing_snapshot_id.clone())
    .with_selection_seed_option(context.selection_seed.or(decision.selection_seed))
    .with_selection_reason_option(decision.selection_reason.clone())
    .with_fallback_reason_option(decision.fallback_reason.clone())
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
    tenant_id: Option<&str>,
    project_id: Option<&str>,
    api_key_group_id: Option<&str>,
    requested_region: Option<String>,
    selection_seed: Option<u64>,
    recovery_probe_lock_store: Option<&dyn DistributedLockStore>,
) -> Result<RoutingDecision> {
    let project_preferences = match project_id {
        Some(project_id) => store.find_project_routing_preferences(project_id).await?,
        None => None,
    };
    let applied_routing_profile =
        load_group_routing_profile(store, tenant_id, project_id, api_key_group_id).await?;
    let applied_routing_profile_id = applied_routing_profile
        .as_ref()
        .map(|profile| profile.profile_id.clone());
    let requested_region = requested_region
        .or_else(|| preferred_region_from_routing_profile(applied_routing_profile.as_ref()))
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
        applied_routing_profile.as_ref(),
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
    let policy_candidate_unavailable = effective_policy.as_ref().and_then(|policy| {
        let available_provider_ids = if model_candidate_ids.is_empty() {
            provider_map.keys().cloned().collect::<HashSet<_>>()
        } else {
            model_candidate_ids.iter().cloned().collect::<HashSet<_>>()
        };
        policy_candidate_unavailable_fallback_reason(policy, &available_provider_ids)
    });

    let candidate_ids = if model_candidate_ids.is_empty() {
        if let Some(policy) = effective_policy.as_ref() {
            let available_provider_ids = provider_map.keys().cloned().collect::<HashSet<_>>();
            let candidate_ids = policy
                .declared_provider_ids()
                .into_iter()
                .filter(|provider_id| available_provider_ids.contains(provider_id))
                .collect::<Vec<_>>();

            if candidate_ids.is_empty() {
                let compiled_snapshot_id = persist_compiled_routing_snapshot(
                    store,
                    capability,
                    model,
                    tenant_id,
                    project_id,
                    api_key_group_id,
                    matched_policy,
                    project_preferences.as_ref(),
                    applied_routing_profile_id.as_deref(),
                    effective_policy.as_ref(),
                    requested_region.as_deref(),
                    STRATEGY_STATIC_FALLBACK,
                    &[],
                )
                .await?;
                return Ok(simulate_route(capability, model)?
                    .with_applied_routing_profile_id_option(applied_routing_profile_id.clone())
                    .with_compiled_routing_snapshot_id(compiled_snapshot_id)
                    .with_fallback_reason_option(merged_fallback_reason(
                        Some(NO_CATALOG_OR_POLICY_CANDIDATES_FALLBACK_REASON),
                        policy_candidate_unavailable,
                    ))
                    .with_requested_region_option(requested_region.clone()));
            }

            candidate_ids
        } else {
            let compiled_snapshot_id = persist_compiled_routing_snapshot(
                store,
                capability,
                model,
                tenant_id,
                project_id,
                api_key_group_id,
                matched_policy,
                project_preferences.as_ref(),
                applied_routing_profile_id.as_deref(),
                effective_policy.as_ref(),
                requested_region.as_deref(),
                STRATEGY_STATIC_FALLBACK,
                &[],
            )
            .await?;
            return Ok(simulate_route(capability, model)?
                .with_applied_routing_profile_id_option(applied_routing_profile_id.clone())
                .with_compiled_routing_snapshot_id(compiled_snapshot_id)
                .with_fallback_reason(NO_CATALOG_OR_POLICY_CANDIDATES_FALLBACK_REASON)
                .with_requested_region_option(requested_region.clone()));
        }
    } else {
        match effective_policy.as_ref() {
            Some(policy) => policy.rank_candidates(&model_candidate_ids),
            None => model_candidate_ids,
        }
    };

    let compiled_routing_snapshot_id = persist_compiled_routing_snapshot(
        store,
        capability,
        model,
        tenant_id,
        project_id,
        api_key_group_id,
        matched_policy,
        project_preferences.as_ref(),
        applied_routing_profile_id.as_deref(),
        effective_policy.as_ref(),
        requested_region.as_deref(),
        resolved_compiled_snapshot_strategy(effective_policy.as_ref()),
        &candidate_ids,
    )
    .await?;

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
    let assessment_time_ms = current_time_millis();
    let recovery_probe_provider_id =
        recovery_probe_provider_id(&candidate_ids, &latest_provider_health, assessment_time_ms);

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
                assessment_time_ms,
            )
        })
        .collect::<Vec<_>>();
    assessments.sort_by(compare_assessments);

    let selection = select_candidate(
        &mut assessments,
        effective_policy.as_ref(),
        requested_region.as_deref(),
        selection_seed,
        recovery_probe_provider_id.as_deref(),
        recovery_probe_lock_store,
    );
    let selection = selection.await;
    let CandidateSelection {
        selected_index,
        strategy,
        selection_seed,
        selection_reason,
        fallback_reason,
        provider_health_recovery_probe,
        slo_applied,
        slo_degraded,
    } = selection;
    let selected = assessments
        .get(selected_index)
        .map(|assessment| assessment.provider_id.clone())
        .unwrap_or_else(|| candidate_ids[0].clone());

    let ranked_candidate_ids = assessments
        .iter()
        .map(|assessment| assessment.provider_id.clone())
        .collect::<Vec<_>>();
    let fallback_reason = merged_fallback_reason(
        fallback_reason.as_deref(),
        policy_candidate_unavailable,
    );

    let mut decision = RoutingDecision::new(selected, ranked_candidate_ids)
        .with_applied_routing_profile_id_option(applied_routing_profile_id)
        .with_compiled_routing_snapshot_id(compiled_routing_snapshot_id)
        .with_strategy(strategy)
        .with_selection_reason(selection_reason)
        .with_fallback_reason_option(fallback_reason)
        .with_requested_region_option(requested_region)
        .with_provider_health_recovery_probe_option(provider_health_recovery_probe)
        .with_slo_state(slo_applied, slo_degraded)
        .with_assessments(assessments);
    if let Some(selection_seed) = selection_seed {
        decision = decision.with_selection_seed(selection_seed);
    }
    Ok(match matched_policy {
        Some(policy) => decision.with_matched_policy_id(policy.policy_id.clone()),
        None => decision,
    })
}

async fn load_group_routing_profile(
    store: &dyn AdminStore,
    tenant_id: Option<&str>,
    project_id: Option<&str>,
    api_key_group_id: Option<&str>,
) -> Result<Option<RoutingProfileRecord>> {
    let Some(group_id) = api_key_group_id else {
        return Ok(None);
    };

    let Some(group) = store.find_api_key_group(group_id).await? else {
        return Ok(None);
    };

    if let Some(tenant_id) = tenant_id {
        if group.tenant_id != tenant_id {
            return Ok(None);
        }
    }

    if let Some(project_id) = project_id {
        if group.project_id != project_id {
            return Ok(None);
        }
    }

    let Some(profile_id) = group.default_routing_profile_id.as_deref() else {
        return Ok(None);
    };

    let Some(profile) = store.find_routing_profile(profile_id).await? else {
        return Ok(None);
    };

    if !profile.active
        || profile.tenant_id != group.tenant_id
        || profile.project_id != group.project_id
    {
        return Ok(None);
    }

    Ok(Some(profile))
}

fn preferred_region_from_preferences(
    preferences: Option<&ProjectRoutingPreferences>,
) -> Option<String> {
    preferences.and_then(|preferences| preferences.preferred_region.clone())
}

fn preferred_region_from_routing_profile(profile: Option<&RoutingProfileRecord>) -> Option<String> {
    profile.and_then(|profile| profile.preferred_region.clone())
}

fn effective_routing_policy(
    matched_policy: Option<&RoutingPolicy>,
    preferences: Option<&ProjectRoutingPreferences>,
    routing_profile: Option<&RoutingProfileRecord>,
    capability: &str,
    model: &str,
) -> Option<RoutingPolicy> {
    let policy =
        apply_project_routing_preferences(matched_policy.cloned(), preferences, capability, model);
    apply_routing_profile(policy, routing_profile, capability, model)
}

fn resolved_compiled_snapshot_strategy(effective_policy: Option<&RoutingPolicy>) -> &'static str {
    match effective_policy {
        Some(policy) => policy.strategy.as_str(),
        None => RoutingStrategy::DeterministicPriority.as_str(),
    }
}

async fn persist_compiled_routing_snapshot(
    store: &dyn AdminStore,
    capability: &str,
    route_key: &str,
    tenant_id: Option<&str>,
    project_id: Option<&str>,
    api_key_group_id: Option<&str>,
    matched_policy: Option<&RoutingPolicy>,
    project_preferences: Option<&ProjectRoutingPreferences>,
    applied_routing_profile_id: Option<&str>,
    effective_policy: Option<&RoutingPolicy>,
    preferred_region: Option<&str>,
    strategy: &str,
    ordered_provider_ids: &[String],
) -> Result<String> {
    let now = current_time_millis();
    let snapshot_id = build_compiled_routing_snapshot_id(
        tenant_id,
        project_id,
        api_key_group_id,
        capability,
        route_key,
    );
    let snapshot = CompiledRoutingSnapshotRecord::new(&snapshot_id, capability, route_key)
        .with_tenant_id_option(tenant_id.map(ToOwned::to_owned))
        .with_project_id_option(project_id.map(ToOwned::to_owned))
        .with_api_key_group_id_option(api_key_group_id.map(ToOwned::to_owned))
        .with_matched_policy_id_option(matched_policy.map(|policy| policy.policy_id.clone()))
        .with_project_routing_preferences_project_id_option(
            project_preferences.map(|preferences| preferences.project_id.clone()),
        )
        .with_applied_routing_profile_id_option(applied_routing_profile_id.map(ToOwned::to_owned))
        .with_strategy(strategy)
        .with_ordered_provider_ids(ordered_provider_ids.to_vec())
        .with_default_provider_id_option(
            effective_policy.and_then(|policy| policy.default_provider_id.clone()),
        )
        .with_max_cost_option(effective_policy.and_then(|policy| policy.max_cost))
        .with_max_latency_ms_option(effective_policy.and_then(|policy| policy.max_latency_ms))
        .with_require_healthy(
            effective_policy
                .map(|policy| policy.require_healthy)
                .unwrap_or(false),
        )
        .with_preferred_region_option(preferred_region.map(ToOwned::to_owned))
        .with_created_at_ms(now)
        .with_updated_at_ms(now);
    store.insert_compiled_routing_snapshot(&snapshot).await?;
    Ok(snapshot_id)
}

fn build_compiled_routing_snapshot_id(
    tenant_id: Option<&str>,
    project_id: Option<&str>,
    api_key_group_id: Option<&str>,
    capability: &str,
    route_key: &str,
) -> String {
    format!(
        "routing-snapshot-{}-{}-{}-{}-{}",
        snapshot_id_segment(tenant_id, "global"),
        snapshot_id_segment(project_id, "global"),
        snapshot_id_segment(api_key_group_id, "default"),
        sanitize_snapshot_segment(capability),
        sanitize_snapshot_segment(route_key),
    )
}

fn snapshot_id_segment(value: Option<&str>, fallback: &str) -> String {
    value
        .map(sanitize_snapshot_segment)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| fallback.to_owned())
}

fn sanitize_snapshot_segment(value: &str) -> String {
    let mut sanitized = String::with_capacity(value.len());
    let mut last_was_dash = false;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            sanitized.push(ch.to_ascii_lowercase());
            last_was_dash = false;
        } else if !last_was_dash {
            sanitized.push('-');
            last_was_dash = true;
        }
    }
    sanitized.trim_matches('-').to_owned()
}

fn policy_candidate_unavailable_fallback_reason<'a>(
    policy: &RoutingPolicy,
    available_provider_ids: &'a HashSet<String>,
) -> Option<&'static str> {
    policy
        .declared_provider_ids()
        .into_iter()
        .any(|provider_id| !available_provider_ids.contains(&provider_id))
        .then_some(POLICY_CANDIDATE_UNAVAILABLE_FALLBACK_REASON)
}

fn merged_fallback_reason(existing: Option<&str>, additional: Option<&str>) -> Option<String> {
    match (existing, additional) {
        (Some(existing), Some(additional))
            if existing.split(';').any(|value| value == additional) =>
        {
            Some(existing.to_owned())
        }
        (Some(existing), Some(additional)) => Some(format!("{existing};{additional}")),
        (Some(existing), None) => Some(existing.to_owned()),
        (None, Some(additional)) => Some(additional.to_owned()),
        (None, None) => None,
    }
}

fn apply_project_routing_preferences(
    base_policy: Option<RoutingPolicy>,
    preferences: Option<&ProjectRoutingPreferences>,
    capability: &str,
    model: &str,
) -> Option<RoutingPolicy> {
    let Some(preferences) = preferences else {
        return base_policy;
    };

    let base_policy = base_policy
        .unwrap_or_else(|| RoutingPolicy::new("project-routing-preferences", capability, model));

    Some(
        base_policy
            .clone()
            .with_strategy(preferences.strategy)
            .with_ordered_provider_ids(if preferences.ordered_provider_ids.is_empty() {
                base_policy.ordered_provider_ids.clone()
            } else {
                preferences.ordered_provider_ids.clone()
            })
            .with_default_provider_id_option(
                preferences
                    .default_provider_id
                    .clone()
                    .or(base_policy.default_provider_id.clone()),
            )
            .with_max_cost_option(combine_optional_f64(
                base_policy.max_cost,
                preferences.max_cost,
            ))
            .with_max_latency_ms_option(combine_optional_u64(
                base_policy.max_latency_ms,
                preferences.max_latency_ms,
            ))
            .with_require_healthy(base_policy.require_healthy || preferences.require_healthy),
    )
}

fn apply_routing_profile(
    base_policy: Option<RoutingPolicy>,
    routing_profile: Option<&RoutingProfileRecord>,
    capability: &str,
    model: &str,
) -> Option<RoutingPolicy> {
    let Some(routing_profile) = routing_profile else {
        return base_policy;
    };

    let base_policy = base_policy
        .unwrap_or_else(|| RoutingPolicy::new("group-routing-profile", capability, model));

    Some(
        base_policy
            .clone()
            .with_strategy(routing_profile.strategy)
            .with_ordered_provider_ids(if routing_profile.ordered_provider_ids.is_empty() {
                base_policy.ordered_provider_ids.clone()
            } else {
                routing_profile.ordered_provider_ids.clone()
            })
            .with_default_provider_id_option(
                routing_profile
                    .default_provider_id
                    .clone()
                    .or(base_policy.default_provider_id.clone()),
            )
            .with_max_cost_option(combine_optional_f64(
                base_policy.max_cost,
                routing_profile.max_cost,
            ))
            .with_max_latency_ms_option(combine_optional_u64(
                base_policy.max_latency_ms,
                routing_profile.max_latency_ms,
            ))
            .with_require_healthy(base_policy.require_healthy || routing_profile.require_healthy),
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
