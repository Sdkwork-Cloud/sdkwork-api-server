use super::*;

pub(crate) async fn select_gateway_route(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: Option<&str>,
    capability: &str,
    route_key: &str,
) -> Result<RoutingDecision> {
    let requested_region = current_request_routing_region();
    let api_key_group_id = current_request_api_key_group_id();
    let recovery_probe_lock_store = routing_recovery_probe_lock_store();
    let decision = select_route_with_store_context(
        store,
        capability,
        route_key,
        RouteSelectionContext::new(RoutingDecisionSource::Gateway)
            .with_tenant_id_option(Some(tenant_id))
            .with_project_id_option(project_id)
            .with_api_key_group_id_option(api_key_group_id.as_deref())
            .with_requested_region_option(requested_region.as_deref())
            .with_recovery_probe_lock_store_option(Some(recovery_probe_lock_store.as_ref())),
    )
    .await?;
    record_gateway_recovery_probe_from_decision(&decision);
    cache_routing_decision(
        tenant_id,
        project_id,
        api_key_group_id.as_deref(),
        capability,
        route_key,
        requested_region.as_deref(),
        &decision,
    )
    .await;
    Ok(decision)
}

pub async fn planned_execution_provider_id_for_route(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    route_key: &str,
) -> Result<String> {
    Ok(planned_execution_usage_context_for_route(
        store, tenant_id, project_id, capability, route_key,
    )
    .await?
    .provider_id)
}

pub async fn planned_execution_usage_context_for_route(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    route_key: &str,
) -> Result<PlannedExecutionUsageContext> {
    let api_key_group_id = current_request_api_key_group_id();
    let decision = planned_execution_decision_for_route_without_log_with_selection_seed(
        store,
        tenant_id,
        project_id,
        capability,
        route_key,
        None,
    )
    .await?;
    gateway_usage_context_for_decision_provider(
        store,
        tenant_id,
        &decision,
        &decision.selected_provider_id,
        api_key_group_id,
        decision.fallback_reason.clone(),
    )
    .await
}

pub async fn planned_execution_provider_context_for_route_without_log(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    route_key: &str,
) -> Result<Option<PlannedExecutionProviderContext>> {
    planned_execution_provider_context_for_route_without_log_with_selection_seed(
        store,
        secret_manager,
        tenant_id,
        project_id,
        capability,
        route_key,
        None,
    )
    .await
}

pub async fn planned_execution_provider_context_for_route_without_log_with_selection_seed(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    route_key: &str,
    selection_seed: Option<u64>,
) -> Result<Option<PlannedExecutionProviderContext>> {
    let requested_region = current_request_routing_region();
    let api_key_group_id = current_request_api_key_group_id();
    let recovery_probe_lock_store = routing_recovery_probe_lock_store();
    let decision = simulate_route_with_store_selection_context(
        store,
        capability,
        route_key,
        RouteSelectionContext::new(RoutingDecisionSource::Gateway)
            .with_tenant_id_option(Some(tenant_id))
            .with_project_id_option(Some(project_id))
            .with_api_key_group_id_option(api_key_group_id.as_deref())
            .with_requested_region_option(requested_region.as_deref())
            .with_selection_seed_option(selection_seed)
            .with_recovery_probe_lock_store_option(Some(recovery_probe_lock_store.as_ref())),
    )
    .await?;
    record_gateway_recovery_probe_from_decision(&decision);

    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };
    let failover_enabled =
        crate::gateway_execution_context::gateway_execution_failover_enabled_for_decision(
            store, &decision,
        )
        .await?;
    let mut first_resolution_error = None;
    if let Some(descriptor) = resolve_provider_execution_descriptor_for_failover_candidate(
        store,
        secret_manager,
        tenant_id,
        &provider,
        decision.requested_region.as_deref(),
        failover_enabled,
        &mut first_resolution_error,
    )
    .await?
    {
        return build_planned_execution_provider_context(
            store,
            tenant_id,
            api_key_group_id,
            decision,
            provider,
            descriptor,
        )
        .await
        .map(Some);
    } else if !failover_enabled {
        return Ok(None);
    }

    for candidate_provider_id in decision
        .candidate_ids
        .iter()
        .filter(|provider_id| provider_id.as_str() != decision.selected_provider_id)
    {
        let Some(candidate_provider) = store.find_provider(candidate_provider_id).await? else {
            continue;
        };
        let Some(candidate_descriptor) =
            resolve_provider_execution_descriptor_for_failover_candidate(
                store,
                secret_manager,
                tenant_id,
                &candidate_provider,
                decision.requested_region.as_deref(),
                true,
                &mut first_resolution_error,
            )
            .await?
        else {
            continue;
        };
        let fallback_decision =
            planned_execution_failover_decision(&decision, &candidate_provider.id);
        return build_planned_execution_provider_context(
            store,
            tenant_id,
            api_key_group_id,
            fallback_decision,
            candidate_provider,
            candidate_descriptor,
        )
        .await
        .map(Some);
    }

    if let Some(error) = first_resolution_error {
        return Err(error);
    }

    Ok(None)
}

async fn planned_execution_decision_for_route_without_log_with_selection_seed(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    route_key: &str,
    selection_seed: Option<u64>,
) -> Result<RoutingDecision> {
    let requested_region = current_request_routing_region();
    let api_key_group_id = current_request_api_key_group_id();

    if selection_seed.is_none() {
        if let Some(decision) = take_cached_routing_decision(
            tenant_id,
            Some(project_id),
            api_key_group_id.as_deref(),
            capability,
            route_key,
            requested_region.as_deref(),
        )
        .await
        {
            return Ok(decision);
        }
    }

    let recovery_probe_lock_store = routing_recovery_probe_lock_store();
    let decision = simulate_route_with_store_selection_context(
        store,
        capability,
        route_key,
        RouteSelectionContext::new(RoutingDecisionSource::Gateway)
            .with_tenant_id_option(Some(tenant_id))
            .with_project_id_option(Some(project_id))
            .with_api_key_group_id_option(api_key_group_id.as_deref())
            .with_requested_region_option(requested_region.as_deref())
            .with_selection_seed_option(selection_seed)
            .with_recovery_probe_lock_store_option(Some(recovery_probe_lock_store.as_ref())),
    )
    .await?;
    record_gateway_recovery_probe_from_decision(&decision);

    Ok(decision)
}

pub(crate) async fn resolve_store_relay_provider_for_decision(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    decision: &RoutingDecision,
    failover_enabled: bool,
) -> Result<Option<(RoutingDecision, ProxyProvider, ProviderExecutionDescriptor)>> {
    let Some(provider) = store.find_provider(&decision.selected_provider_id).await? else {
        return Ok(None);
    };

    let mut first_resolution_error = None;
    if let Some(descriptor) = resolve_provider_execution_descriptor_for_failover_candidate(
        store,
        secret_manager,
        tenant_id,
        &provider,
        decision.requested_region.as_deref(),
        failover_enabled,
        &mut first_resolution_error,
    )
    .await?
    {
        return Ok(Some((decision.clone(), provider, descriptor)));
    }

    if !failover_enabled {
        return Ok(None);
    }

    for candidate_provider_id in decision
        .candidate_ids
        .iter()
        .filter(|provider_id| provider_id.as_str() != decision.selected_provider_id)
    {
        let Some(candidate_provider) = store.find_provider(candidate_provider_id).await? else {
            continue;
        };
        let Some(candidate_descriptor) =
            resolve_provider_execution_descriptor_for_failover_candidate(
                store,
                secret_manager,
                tenant_id,
                &candidate_provider,
                decision.requested_region.as_deref(),
                true,
                &mut first_resolution_error,
            )
            .await?
        else {
            continue;
        };

        return Ok(Some((
            planned_execution_failover_decision(decision, &candidate_provider.id),
            candidate_provider,
            candidate_descriptor,
        )));
    }

    if let Some(error) = first_resolution_error {
        return Err(error);
    }

    Ok(None)
}

async fn resolve_provider_execution_descriptor_for_failover_candidate(
    store: &dyn AdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    provider: &ProxyProvider,
    requested_region: Option<&str>,
    suppress_errors_for_failover: bool,
    first_resolution_error: &mut Option<anyhow::Error>,
) -> Result<Option<ProviderExecutionDescriptor>> {
    match provider_execution_descriptor_for_provider_account_context(
        store,
        secret_manager,
        tenant_id,
        provider,
        requested_region,
    )
    .await
    {
        Ok(descriptor) => Ok(descriptor),
        Err(error) if suppress_errors_for_failover => {
            if first_resolution_error.is_none() {
                *first_resolution_error = Some(error);
            }
            Ok(None)
        }
        Err(error) => Err(error),
    }
}

async fn build_planned_execution_provider_context(
    store: &dyn AdminStore,
    tenant_id: &str,
    api_key_group_id: Option<String>,
    decision: RoutingDecision,
    provider: ProxyProvider,
    descriptor: ProviderExecutionDescriptor,
) -> Result<PlannedExecutionProviderContext> {
    let execution_target = ProviderExecutionTarget {
        provider_id: descriptor.provider_id.clone(),
        provider_account_id: descriptor.provider_account_id.clone(),
        execution_instance_id: descriptor.execution_instance_id.clone(),
        runtime_key: descriptor.runtime_key.clone(),
        base_url: descriptor.base_url.clone(),
        runtime: descriptor.runtime.clone(),
        local_fallback: descriptor.local_fallback,
    };
    let usage_context = gateway_usage_context_for_decision_provider_with_target(
        store,
        tenant_id,
        &decision,
        &provider.id,
        api_key_group_id,
        decision.fallback_reason.clone(),
        Some(&execution_target),
    )
    .await?;

    Ok(PlannedExecutionProviderContext {
        decision,
        provider,
        api_key: descriptor.api_key,
        usage_context,
        execution: PlannedExecutionRuntimeContext {
            provider_account_id: execution_target.provider_account_id,
            execution_instance_id: execution_target.execution_instance_id,
            runtime_key: execution_target.runtime_key,
            base_url: execution_target.base_url,
            runtime: execution_target.runtime,
            local_fallback: execution_target.local_fallback,
        },
    })
}

fn planned_execution_failover_decision(
    decision: &RoutingDecision,
    provider_id: &str,
) -> RoutingDecision {
    if decision.selected_provider_id == provider_id {
        return decision.clone();
    }

    let mut fallback_decision = decision.clone();
    fallback_decision.selected_provider_id = provider_id.to_owned();
    fallback_decision.fallback_reason =
        gateway_execution_failover_fallback_reason(decision.fallback_reason.as_deref());
    fallback_decision
}

pub async fn persist_planned_execution_decision_log(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    route_key: &str,
    decision: &RoutingDecision,
) -> Result<()> {
    let created_at_ms = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis() as u64;
    let requested_region =
        current_request_routing_region().or_else(|| decision.requested_region.clone());
    let api_key_group_id = current_request_api_key_group_id();
    let log = RoutingDecisionLog::new(
        gateway_execution_decision_id(&decision.selected_provider_id, created_at_ms),
        RoutingDecisionSource::Gateway,
        capability,
        route_key,
        decision.selected_provider_id.clone(),
        decision
            .strategy
            .clone()
            .unwrap_or_else(|| "deterministic_priority".to_owned()),
        created_at_ms,
    )
    .with_tenant_id_option(Some(tenant_id.to_owned()))
    .with_project_id_option(Some(project_id.to_owned()))
    .with_api_key_group_id_option(api_key_group_id)
    .with_matched_policy_id_option(decision.matched_policy_id.clone())
    .with_applied_routing_profile_id_option(decision.applied_routing_profile_id.clone())
    .with_compiled_routing_snapshot_id_option(decision.compiled_routing_snapshot_id.clone())
    .with_selection_seed_option(decision.selection_seed)
    .with_selection_reason_option(decision.selection_reason.clone())
    .with_fallback_reason_option(decision.fallback_reason.clone())
    .with_requested_region_option(requested_region)
    .with_slo_state(decision.slo_applied, decision.slo_degraded)
    .with_assessments(decision.assessments.clone());
    store.insert_routing_decision_log(&log).await?;
    Ok(())
}

pub(crate) async fn gateway_usage_context_for_decision_provider(
    store: &dyn AdminStore,
    tenant_id: &str,
    decision: &RoutingDecision,
    provider_id: &str,
    api_key_group_id: Option<String>,
    fallback_reason: Option<String>,
) -> Result<PlannedExecutionUsageContext> {
    gateway_usage_context_for_decision_provider_with_target(
        store,
        tenant_id,
        decision,
        provider_id,
        api_key_group_id,
        fallback_reason,
        None,
    )
    .await
}

async fn gateway_usage_context_for_decision_provider_with_target(
    store: &dyn AdminStore,
    tenant_id: &str,
    decision: &RoutingDecision,
    provider_id: &str,
    api_key_group_id: Option<String>,
    fallback_reason: Option<String>,
    execution_target: Option<&ProviderExecutionTarget>,
) -> Result<PlannedExecutionUsageContext> {
    let selected_assessment = decision
        .assessments
        .iter()
        .find(|assessment| assessment.provider_id == provider_id);
    let Some(provider) = store.find_provider(provider_id).await? else {
        return Ok(PlannedExecutionUsageContext {
            provider_id: LOCAL_PROVIDER_ID.to_owned(),
            channel_id: None,
            api_key_group_id,
            applied_routing_profile_id: decision.applied_routing_profile_id.clone(),
            compiled_routing_snapshot_id: decision.compiled_routing_snapshot_id.clone(),
            fallback_reason,
            latency_ms: selected_assessment.and_then(|assessment| assessment.latency_ms),
            reference_amount: selected_assessment.and_then(|assessment| assessment.cost),
        });
    };

    let target = match execution_target {
        Some(target) => target.clone(),
        None => provider_execution_target_for_provider(store, &provider).await?,
    };
    if target.local_fallback {
        return Ok(PlannedExecutionUsageContext {
            provider_id: target.provider_id,
            channel_id: Some(provider.channel_id),
            api_key_group_id,
            applied_routing_profile_id: decision.applied_routing_profile_id.clone(),
            compiled_routing_snapshot_id: decision.compiled_routing_snapshot_id.clone(),
            fallback_reason,
            latency_ms: selected_assessment.and_then(|assessment| assessment.latency_ms),
            reference_amount: selected_assessment.and_then(|assessment| assessment.cost),
        });
    }

    let has_credential = store
        .find_provider_credential(tenant_id, &provider.id)
        .await?
        .is_some()
        || official_provider_secret_configured(store, &provider.id).await?;

    if has_credential {
        Ok(PlannedExecutionUsageContext {
            provider_id: target.provider_id,
            channel_id: Some(provider.channel_id),
            api_key_group_id,
            applied_routing_profile_id: decision.applied_routing_profile_id.clone(),
            compiled_routing_snapshot_id: decision.compiled_routing_snapshot_id.clone(),
            fallback_reason,
            latency_ms: selected_assessment.and_then(|assessment| assessment.latency_ms),
            reference_amount: selected_assessment.and_then(|assessment| assessment.cost),
        })
    } else {
        Ok(PlannedExecutionUsageContext {
            provider_id: LOCAL_PROVIDER_ID.to_owned(),
            channel_id: Some(provider.channel_id),
            api_key_group_id,
            applied_routing_profile_id: decision.applied_routing_profile_id.clone(),
            compiled_routing_snapshot_id: decision.compiled_routing_snapshot_id.clone(),
            fallback_reason,
            latency_ms: selected_assessment.and_then(|assessment| assessment.latency_ms),
            reference_amount: selected_assessment.and_then(|assessment| assessment.cost),
        })
    }
}

pub(crate) fn gateway_execution_failover_fallback_reason(existing: Option<&str>) -> Option<String> {
    match existing {
        Some(existing)
            if existing
                .split(';')
                .any(|value| value == "gateway_execution_failover") =>
        {
            Some(existing.to_owned())
        }
        Some(existing) => Some(format!("{existing};gateway_execution_failover")),
        None => Some("gateway_execution_failover".to_owned()),
    }
}

fn gateway_execution_decision_id(provider_id: &str, created_at_ms: u64) -> String {
    format!("route_decision:gateway_execution:{provider_id}:{created_at_ms}")
}

pub(crate) async fn persist_gateway_execution_failover_decision_log(
    store: &dyn AdminStore,
    tenant_id: &str,
    project_id: &str,
    capability: &str,
    route_key: &str,
    decision: &RoutingDecision,
    executed_provider_id: &str,
) -> Result<()> {
    let created_at_ms = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis() as u64;
    let requested_region =
        current_request_routing_region().or_else(|| decision.requested_region.clone());
    let api_key_group_id = current_request_api_key_group_id();
    let fallback_reason =
        gateway_execution_failover_fallback_reason(decision.fallback_reason.as_deref());
    let log = RoutingDecisionLog::new(
        gateway_execution_decision_id(executed_provider_id, created_at_ms),
        RoutingDecisionSource::Gateway,
        capability,
        route_key,
        executed_provider_id,
        decision
            .strategy
            .clone()
            .unwrap_or_else(|| "deterministic_priority".to_owned()),
        created_at_ms,
    )
    .with_tenant_id_option(Some(tenant_id.to_owned()))
    .with_project_id_option(Some(project_id.to_owned()))
    .with_api_key_group_id_option(api_key_group_id)
    .with_matched_policy_id_option(decision.matched_policy_id.clone())
    .with_applied_routing_profile_id_option(decision.applied_routing_profile_id.clone())
    .with_compiled_routing_snapshot_id_option(decision.compiled_routing_snapshot_id.clone())
    .with_selection_seed_option(decision.selection_seed)
    .with_selection_reason_option(decision.selection_reason.clone())
    .with_fallback_reason_option(fallback_reason)
    .with_requested_region_option(requested_region)
    .with_slo_state(decision.slo_applied, decision.slo_degraded)
    .with_assessments(decision.assessments.clone());
    store.insert_routing_decision_log(&log).await?;
    Ok(())
}

pub(crate) fn gateway_execution_observed_at_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

pub(crate) fn gateway_execution_health_message(
    capability: &str,
    provider_id: &str,
    healthy: bool,
    error: Option<&anyhow::Error>,
) -> String {
    match (healthy, error) {
        (true, _) => format!(
            "gateway execution succeeded for capability {capability} on provider {provider_id}"
        ),
        (false, Some(error)) => format!(
            "gateway execution failed for capability {capability} on provider {provider_id}: {error}"
        ),
        (false, None) => format!(
            "gateway execution failed for capability {capability} on provider {provider_id}"
        ),
    }
}

pub(crate) async fn persist_gateway_execution_health_snapshot(
    store: &dyn AdminStore,
    descriptor: &ProviderExecutionDescriptor,
    healthy: bool,
    capability: &str,
    error: Option<&anyhow::Error>,
) {
    if descriptor.local_fallback {
        return;
    }

    let observed_at_ms = gateway_execution_observed_at_ms();
    record_gateway_provider_health(
        &descriptor.provider_id,
        descriptor.runtime.as_str(),
        healthy,
        observed_at_ms,
    );

    let snapshot = ProviderHealthSnapshot::new(
        &descriptor.provider_id,
        &descriptor.runtime_key,
        descriptor.runtime.as_str(),
        observed_at_ms,
    )
    .with_running(true)
    .with_healthy(healthy)
    .with_message(gateway_execution_health_message(
        capability,
        &descriptor.provider_id,
        healthy,
        error,
    ));

    if let Err(persist_error) = store.insert_provider_health_snapshot(&snapshot).await {
        record_gateway_provider_health_persist_failure(
            &descriptor.provider_id,
            descriptor.runtime.as_str(),
        );
        eprintln!(
            "gateway execution health snapshot persistence failed for provider {}: {persist_error}",
            descriptor.provider_id
        );
    }
}
