use sdkwork_api_domain_routing::{
    ProviderHealthSnapshot, RoutingCandidateAssessment, RoutingCandidateHealth, RoutingDecision,
    RoutingPolicy, RoutingStrategy,
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
        .with_strategy("runtime_aware_deterministic")
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

    assert_eq!(
        decision.strategy.as_deref(),
        Some("runtime_aware_deterministic")
    );
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
