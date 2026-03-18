use sdkwork_api_domain_routing::{
    ProviderHealthSnapshot, RoutingCandidateAssessment, RoutingCandidateHealth, RoutingDecision,
    RoutingDecisionLog, RoutingDecisionSource, RoutingPolicy, RoutingStrategy,
};

#[test]
fn decision_retains_candidate_ids() {
    let decision =
        RoutingDecision::new("provider-a", vec!["provider-a".into(), "provider-b".into()]);
    assert_eq!(decision.candidate_ids.len(), 2);
    assert!(decision.matched_policy_id.is_none());
}

#[test]
fn decision_can_include_strategy_reason_and_assessments() {
    let decision = RoutingDecision::new("provider-a", vec!["provider-a".into()])
        .with_strategy("deterministic_priority")
        .with_selection_seed(42)
        .with_selection_reason("selected the top-ranked healthy candidate")
        .with_assessments(vec![RoutingCandidateAssessment::new("provider-a")
            .with_available(true)
            .with_health(RoutingCandidateHealth::Healthy)
            .with_policy_rank(0)
            .with_weight(100)
            .with_cost(0.25)
            .with_latency_ms(120)
            .with_reason("healthy runtime")]);

    assert_eq!(decision.strategy.as_deref(), Some("deterministic_priority"));
    assert_eq!(
        decision.selection_reason.as_deref(),
        Some("selected the top-ranked healthy candidate")
    );
    assert_eq!(decision.selection_seed, Some(42));
    assert_eq!(decision.assessments.len(), 1);
    assert_eq!(
        decision.assessments[0].health,
        RoutingCandidateHealth::Healthy
    );
    assert_eq!(decision.assessments[0].weight, Some(100));
}

#[test]
fn policy_matches_exact_and_wildcard_model_patterns() {
    let exact = RoutingPolicy::new("policy-exact", "chat_completion", "gpt-4.1");
    assert_eq!(exact.strategy, RoutingStrategy::DeterministicPriority);
    assert!(exact.matches("chat_completion", "gpt-4.1"));
    assert!(!exact.matches("chat_completion", "gpt-4.1-mini"));
    assert!(!exact.matches("responses", "gpt-4.1"));

    let wildcard = RoutingPolicy::new("policy-wildcard", "chat_completion", "gpt-4*");
    assert!(wildcard.matches("chat_completion", "gpt-4.1"));
    assert!(wildcard.matches("chat_completion", "gpt-4o-mini"));
    assert!(!wildcard.matches("chat_completion", "text-embedding-3-large"));
}

#[test]
fn policy_ranks_providers_using_explicit_order_and_default() {
    let policy = RoutingPolicy::new("policy-rank", "chat_completion", "gpt-4.1")
        .with_priority(100)
        .with_ordered_provider_ids(vec![
            "provider-openrouter".to_owned(),
            "provider-openai-official".to_owned(),
        ])
        .with_default_provider_id("provider-openai-official");

    let ranked = policy.rank_candidates(&[
        "provider-openai-official".to_owned(),
        "provider-azure".to_owned(),
        "provider-openrouter".to_owned(),
    ]);

    assert_eq!(
        ranked,
        vec![
            "provider-openrouter".to_owned(),
            "provider-openai-official".to_owned(),
            "provider-azure".to_owned(),
        ]
    );
}

#[test]
fn policy_can_switch_to_weighted_random_strategy() {
    let policy = RoutingPolicy::new("policy-weighted", "chat_completion", "gpt-4.1")
        .with_strategy(RoutingStrategy::WeightedRandom);

    assert_eq!(policy.strategy, RoutingStrategy::WeightedRandom);
}

#[test]
fn policy_can_switch_to_slo_aware_strategy_with_thresholds() {
    let policy = RoutingPolicy::new("policy-slo", "chat_completion", "gpt-4.1")
        .with_strategy(RoutingStrategy::SloAware)
        .with_max_cost(0.25)
        .with_max_latency_ms(200)
        .with_require_healthy(true);

    assert_eq!(policy.strategy, RoutingStrategy::SloAware);
    assert_eq!(policy.max_cost, Some(0.25));
    assert_eq!(policy.max_latency_ms, Some(200));
    assert!(policy.require_healthy);
}

#[test]
fn assessment_can_record_slo_eligibility_and_violations() {
    let assessment = RoutingCandidateAssessment::new("provider-a")
        .with_slo_eligible(false)
        .with_slo_violation("latency 450ms exceeds max_latency_ms 200")
        .with_slo_violation("cost 0.4 exceeds max_cost 0.25");

    assert_eq!(assessment.slo_eligible, Some(false));
    assert_eq!(assessment.slo_violations.len(), 2);
    assert!(assessment.slo_violations[0].contains("latency 450ms"));
}

#[test]
fn routing_decision_log_captures_auditable_selection_context() {
    let assessment = RoutingCandidateAssessment::new("provider-a")
        .with_available(true)
        .with_health(RoutingCandidateHealth::Healthy)
        .with_slo_eligible(true);
    let log = RoutingDecisionLog::new(
        "decision-1",
        RoutingDecisionSource::Gateway,
        "chat_completion",
        "gpt-4.1",
        "provider-a",
        "slo_aware",
        1_710_000_000_000,
    )
    .with_tenant_id("tenant-1")
    .with_project_id("project-1")
    .with_matched_policy_id("policy-slo")
    .with_selection_seed(42)
    .with_selection_reason("selected provider-a as the top-ranked SLO-compliant candidate")
    .with_slo_state(true, false)
    .with_assessments(vec![assessment]);

    assert_eq!(log.decision_source, RoutingDecisionSource::Gateway);
    assert_eq!(log.tenant_id.as_deref(), Some("tenant-1"));
    assert_eq!(log.project_id.as_deref(), Some("project-1"));
    assert_eq!(log.matched_policy_id.as_deref(), Some("policy-slo"));
    assert_eq!(log.strategy, "slo_aware");
    assert!(log.slo_applied);
    assert!(!log.slo_degraded);
    assert_eq!(log.assessments.len(), 1);
}

#[test]
fn provider_health_snapshot_captures_runtime_observation() {
    let snapshot = ProviderHealthSnapshot::new(
        "provider-openai-official",
        "sdkwork.provider.openai.official",
        "builtin",
        1_710_000_000_000,
    )
    .with_instance_id("provider-openai-official")
    .with_running(true)
    .with_healthy(true)
    .with_message("healthy");

    assert_eq!(snapshot.provider_id, "provider-openai-official");
    assert_eq!(
        snapshot.instance_id.as_deref(),
        Some("provider-openai-official")
    );
    assert!(snapshot.running);
    assert!(snapshot.healthy);
    assert_eq!(snapshot.message.as_deref(), Some("healthy"));
}
