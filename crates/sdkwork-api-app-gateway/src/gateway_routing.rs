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
    let requested_region = current_request_routing_region();
    let api_key_group_id = current_request_api_key_group_id();
    let decision = match take_cached_routing_decision(
        tenant_id,
        Some(project_id),
        api_key_group_id.as_deref(),
        capability,
        route_key,
        requested_region.as_deref(),
    )
    .await
    {
        Some(decision) => decision,
        None => {
            let recovery_probe_lock_store = routing_recovery_probe_lock_store();
            let decision = select_route_with_store_context(
                store,
                capability,
                route_key,
                RouteSelectionContext::new(RoutingDecisionSource::Gateway)
                    .with_tenant_id_option(Some(tenant_id))
                    .with_project_id_option(Some(project_id))
                    .with_api_key_group_id_option(api_key_group_id.as_deref())
                    .with_requested_region_option(requested_region.as_deref())
                    .with_recovery_probe_lock_store_option(Some(
                        recovery_probe_lock_store.as_ref(),
                    )),
            )
            .await?;
            record_gateway_recovery_probe_from_decision(&decision);
            decision
        }
    };
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

pub(crate) async fn gateway_usage_context_for_decision_provider(
    store: &dyn AdminStore,
    tenant_id: &str,
    decision: &RoutingDecision,
    provider_id: &str,
    api_key_group_id: Option<String>,
    fallback_reason: Option<String>,
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

    let target = provider_execution_target_for_provider(store, &provider).await?;
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
        .is_some();

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
