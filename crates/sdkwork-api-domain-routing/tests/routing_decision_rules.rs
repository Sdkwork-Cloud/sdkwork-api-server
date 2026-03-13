use sdkwork_api_domain_routing::{RoutingDecision, RoutingPolicy};

#[test]
fn decision_retains_candidate_ids() {
    let decision =
        RoutingDecision::new("provider-a", vec!["provider-a".into(), "provider-b".into()]);
    assert_eq!(decision.candidate_ids.len(), 2);
    assert!(decision.matched_policy_id.is_none());
}

#[test]
fn policy_matches_exact_and_wildcard_model_patterns() {
    let exact = RoutingPolicy::new("policy-exact", "chat_completion", "gpt-4.1");
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
