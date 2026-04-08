use super::*;

#[tokio::test]
async fn postgres_store_round_trips_slo_policy_fields_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);

    let policy = RoutingPolicy::new("policy-slo", "chat_completion", "gpt-4.1")
        .with_strategy(RoutingStrategy::SloAware)
        .with_max_cost(0.35)
        .with_max_latency_ms(300)
        .with_require_healthy(true)
        .with_ordered_provider_ids(vec!["provider-openrouter".to_owned()]);

    store.insert_routing_policy(&policy).await.unwrap();

    let stored = store
        .list_routing_policies()
        .await
        .unwrap()
        .into_iter()
        .find(|entry| entry.policy_id == "policy-slo")
        .expect("routing policy");
    assert_eq!(stored.strategy, RoutingStrategy::SloAware);
    assert_eq!(stored.max_cost, Some(0.35));
    assert_eq!(stored.max_latency_ms, Some(300));
    assert!(stored.require_healthy);
}

#[tokio::test]
async fn postgres_store_persists_routing_decision_logs_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);

    let log = RoutingDecisionLog::new(
        "decision-postgres",
        RoutingDecisionSource::Gateway,
        "chat_completion",
        "gpt-4.1",
        "provider-openai-official",
        "slo_aware",
        1234,
    )
    .with_tenant_id("tenant-1")
    .with_project_id("project-1")
    .with_selection_reason(
        "selected provider-openai-official as the top-ranked SLO-compliant candidate",
    )
    .with_slo_state(true, false)
    .with_assessments(vec![RoutingCandidateAssessment::new(
        "provider-openai-official",
    )
    .with_slo_eligible(true)]);

    store.insert_routing_decision_log(&log).await.unwrap();

    let logs = store.list_routing_decision_logs().await.unwrap();
    assert!(logs.iter().any(|entry| entry == &log));
}

#[tokio::test]
async fn postgres_store_round_trips_requested_region_in_routing_decision_logs_when_url_is_provided()
{
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);

    let log = RoutingDecisionLog::new(
        "decision-postgres-region",
        RoutingDecisionSource::AdminSimulation,
        "chat_completion",
        "gpt-4.1",
        "provider-us-east",
        "geo_affinity",
        4321,
    )
    .with_requested_region("us-east")
    .with_assessments(vec![RoutingCandidateAssessment::new("provider-us-east")
        .with_region("us-east")
        .with_region_match(true)]);

    store.insert_routing_decision_log(&log).await.unwrap();

    let logs = store.list_routing_decision_logs().await.unwrap();
    assert!(logs.iter().any(|entry| entry == &log));
}

#[tokio::test]
async fn postgres_store_persists_provider_health_snapshots_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);

    let snapshot = ProviderHealthSnapshot::new(
        "provider-openai-official",
        "sdkwork.provider.openai.official",
        "builtin",
        1234,
    )
    .with_running(true)
    .with_healthy(true)
    .with_message("healthy");

    store
        .insert_provider_health_snapshot(&snapshot)
        .await
        .unwrap();

    let snapshots = store.list_provider_health_snapshots().await.unwrap();
    assert!(snapshots.iter().any(|entry| entry == &snapshot));
}

#[tokio::test]
async fn postgres_store_replaces_provider_health_snapshot_for_same_provider_runtime_and_instance() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);

    let original = ProviderHealthSnapshot::new(
        "provider-openai-official",
        "sdkwork.provider.openai.official",
        "builtin",
        1_000,
    )
    .with_running(true)
    .with_healthy(false)
    .with_message("first failure");
    let replacement = ProviderHealthSnapshot::new(
        "provider-openai-official",
        "sdkwork.provider.openai.official",
        "builtin",
        2_000,
    )
    .with_running(true)
    .with_healthy(true)
    .with_message("recovered");

    store
        .insert_provider_health_snapshot(&original)
        .await
        .unwrap();
    store
        .insert_provider_health_snapshot(&replacement)
        .await
        .unwrap();

    let snapshots = store.list_provider_health_snapshots().await.unwrap();
    assert_eq!(snapshots, vec![replacement]);
}

async fn postgres_store_persists_quota_policies_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);

    let policy = QuotaPolicy::new("quota-project-1", "project-1", 1_000).with_enabled(true);

    store.insert_quota_policy(&policy).await.unwrap();

    let policies = store.list_quota_policies().await.unwrap();
    assert!(policies.iter().any(|entry| entry == &policy));
}

#[tokio::test]
async fn postgres_store_persists_billing_events_when_url_is_provided() {
    let Some(database_url) = std::env::var("SDKWORK_TEST_POSTGRES_URL").ok() else {
        return;
    };

    let pool = run_migrations(&database_url).await.unwrap();
    let store = PostgresAdminStore::new(pool);

    let event = BillingEventRecord::new(
        "evt-postgres-1",
        "tenant-1",
        "project-1",
        "responses",
        "gpt-4.1",
        "gpt-4.1",
        "provider-openrouter",
        BillingAccountingMode::PlatformCredit,
        1_717_171_717,
    )
    .with_api_key_group_id("group-blue")
    .with_operation("responses.create", "multimodal")
    .with_request_facts(
        Some("key-live"),
        Some("openai"),
        Some("resp_123"),
        Some(850),
    )
    .with_units(240)
    .with_token_usage(120, 80, 200)
    .with_cache_token_usage(30, 10)
    .with_media_usage(2, 3.5, 0.0, 12.0)
    .with_financials(0.42, 0.89)
    .with_routing_evidence(
        Some("route-profile-1"),
        Some("snapshot-1"),
        Some("latency_guardrail"),
    );

    store.insert_billing_event(&event).await.unwrap();

    let events = store.list_billing_events().await.unwrap();
    assert!(events.iter().any(|entry| entry == &event));
}
