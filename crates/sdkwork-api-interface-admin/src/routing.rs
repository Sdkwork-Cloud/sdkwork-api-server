use super::*;

pub(crate) async fn list_provider_health_snapshots_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<ProviderHealthSnapshot>>, StatusCode> {
    list_provider_health_snapshots(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn simulate_routing_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<RoutingSimulationRequest>,
) -> Result<Json<RoutingSimulationResponse>, StatusCode> {
    let decision = select_route_with_store_context(
        state.store.as_ref(),
        &request.capability,
        &request.model,
        RouteSelectionContext::new(RoutingDecisionSource::AdminSimulation)
            .with_tenant_id_option(request.tenant_id.as_deref())
            .with_project_id_option(request.project_id.as_deref())
            .with_api_key_group_id_option(request.api_key_group_id.as_deref())
            .with_requested_region_option(request.requested_region.as_deref())
            .with_selection_seed_option(request.selection_seed),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let (selected_candidate, rejected_candidates) =
        split_routing_assessments(&decision.selected_provider_id, &decision.assessments);
    Ok(Json(RoutingSimulationResponse {
        selected_provider_id: decision.selected_provider_id,
        candidate_ids: decision.candidate_ids,
        matched_policy_id: decision.matched_policy_id,
        applied_routing_profile_id: decision.applied_routing_profile_id,
        compiled_routing_snapshot_id: decision.compiled_routing_snapshot_id,
        strategy: decision.strategy,
        selection_seed: decision.selection_seed,
        selection_reason: decision.selection_reason,
        fallback_reason: decision.fallback_reason,
        requested_region: decision.requested_region,
        slo_applied: decision.slo_applied,
        slo_degraded: decision.slo_degraded,
        selected_candidate,
        rejected_candidates,
        assessments: decision.assessments,
    }))
}

fn split_routing_assessments(
    selected_provider_id: &str,
    assessments: &[RoutingCandidateAssessment],
) -> (
    Option<RoutingCandidateAssessment>,
    Vec<RoutingCandidateAssessment>,
) {
    let mut selected_candidate = None;
    let mut rejected_candidates = Vec::new();
    for assessment in assessments {
        if assessment.provider_id == selected_provider_id && selected_candidate.is_none() {
            selected_candidate = Some(assessment.clone());
        } else {
            rejected_candidates.push(assessment.clone());
        }
    }
    (selected_candidate, rejected_candidates)
}

pub(crate) async fn list_compiled_routing_snapshots_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<CompiledRoutingSnapshotRecord>>, StatusCode> {
    list_compiled_routing_snapshots(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn list_routing_decision_logs_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<RoutingDecisionLog>>, StatusCode> {
    list_routing_decision_logs(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn list_routing_policies_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<RoutingPolicy>>, StatusCode> {
    list_routing_policies(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn list_routing_profiles_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<RoutingProfileRecord>>, StatusCode> {
    list_routing_profiles(state.store.as_ref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(crate) async fn create_routing_policy_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateRoutingPolicyRequest>,
) -> Result<(StatusCode, Json<RoutingPolicy>), StatusCode> {
    let policy = create_routing_policy(CreateRoutingPolicyInput {
        policy_id: &request.policy_id,
        capability: &request.capability,
        model_pattern: &request.model_pattern,
        enabled: request.enabled,
        priority: request.priority,
        strategy: request.strategy,
        ordered_provider_ids: &request.ordered_provider_ids,
        default_provider_id: request.default_provider_id.as_deref(),
        max_cost: request.max_cost,
        max_latency_ms: request.max_latency_ms,
        require_healthy: request.require_healthy,
        execution_failover_enabled: request.execution_failover_enabled,
        upstream_retry_max_attempts: request.upstream_retry_max_attempts,
        upstream_retry_base_delay_ms: request.upstream_retry_base_delay_ms,
        upstream_retry_max_delay_ms: request.upstream_retry_max_delay_ms,
    })
    .map_err(|_| StatusCode::BAD_REQUEST)?;
    let policy = persist_routing_policy(state.store.as_ref(), &policy)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(policy)))
}

pub(crate) async fn create_routing_profile_handler(
    _claims: AuthenticatedAdminClaims,
    State(state): State<AdminApiState>,
    Json(request): Json<CreateRoutingProfileRequest>,
) -> Result<(StatusCode, Json<RoutingProfileRecord>), StatusCode> {
    let profile = create_routing_profile(CreateRoutingProfileInput {
        profile_id: &request.profile_id,
        tenant_id: &request.tenant_id,
        project_id: &request.project_id,
        name: &request.name,
        slug: &request.slug,
        description: request.description.as_deref(),
        active: request.active,
        strategy: request.strategy,
        ordered_provider_ids: &request.ordered_provider_ids,
        default_provider_id: request.default_provider_id.as_deref(),
        max_cost: request.max_cost,
        max_latency_ms: request.max_latency_ms,
        require_healthy: request.require_healthy,
        preferred_region: request.preferred_region.as_deref(),
    })
    .map_err(|_| StatusCode::BAD_REQUEST)?;
    let profile = persist_routing_profile(state.store.as_ref(), &profile)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(profile)))
}
