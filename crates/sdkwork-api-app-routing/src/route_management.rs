use super::*;

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
        .with_require_healthy(input.require_healthy)
        .with_execution_failover_enabled(input.execution_failover_enabled)
        .with_upstream_retry_max_attempts_option(input.upstream_retry_max_attempts)
        .with_upstream_retry_base_delay_ms_option(input.upstream_retry_base_delay_ms)
        .with_upstream_retry_max_delay_ms_option(input.upstream_retry_max_delay_ms);

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

pub fn create_routing_profile(
    input: CreateRoutingProfileInput<'_>,
) -> Result<RoutingProfileRecord> {
    ensure!(
        !input.profile_id.trim().is_empty(),
        "profile_id must not be empty"
    );
    ensure!(
        !input.tenant_id.trim().is_empty(),
        "tenant_id must not be empty"
    );
    ensure!(
        !input.project_id.trim().is_empty(),
        "project_id must not be empty"
    );
    ensure!(!input.name.trim().is_empty(), "name must not be empty");
    ensure!(!input.slug.trim().is_empty(), "slug must not be empty");

    let now = current_time_millis();
    let profile = RoutingProfileRecord::new(
        input.profile_id.trim(),
        input.tenant_id.trim(),
        input.project_id.trim(),
        input.name.trim(),
        input.slug.trim(),
    )
    .with_description_option(normalize_optional_text(input.description))
    .with_active(input.active)
    .with_strategy(
        input
            .strategy
            .unwrap_or(RoutingStrategy::DeterministicPriority),
    )
    .with_ordered_provider_ids(input.ordered_provider_ids.to_vec())
    .with_default_provider_id_option(normalize_optional_text(input.default_provider_id))
    .with_max_cost_option(input.max_cost)
    .with_max_latency_ms_option(input.max_latency_ms)
    .with_require_healthy(input.require_healthy)
    .with_preferred_region_option(normalize_optional_text(input.preferred_region))
    .with_created_at_ms(now)
    .with_updated_at_ms(now);

    Ok(profile)
}

pub async fn persist_routing_profile(
    store: &dyn AdminStore,
    profile: &RoutingProfileRecord,
) -> Result<RoutingProfileRecord> {
    store.insert_routing_profile(profile).await
}

pub async fn list_routing_profiles(store: &dyn AdminStore) -> Result<Vec<RoutingProfileRecord>> {
    store.list_routing_profiles().await
}

pub async fn list_compiled_routing_snapshots(
    store: &dyn AdminStore,
) -> Result<Vec<CompiledRoutingSnapshotRecord>> {
    store.list_compiled_routing_snapshots().await
}

pub async fn list_routing_decision_logs(store: &dyn AdminStore) -> Result<Vec<RoutingDecisionLog>> {
    store.list_routing_decision_logs().await
}

