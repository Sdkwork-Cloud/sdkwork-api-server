use sdkwork_api_domain_routing::RoutingDecision;

#[test]
fn decision_retains_candidate_ids() {
    let decision =
        RoutingDecision::new("provider-a", vec!["provider-a".into(), "provider-b".into()]);
    assert_eq!(decision.candidate_ids.len(), 2);
}
