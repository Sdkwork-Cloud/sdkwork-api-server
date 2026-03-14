use sdkwork_api_domain_routing::{ProviderHealthSnapshot, RoutingPolicy, RoutingStrategy};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[tokio::test]
async fn sqlite_store_persists_routing_policies_with_provider_order() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let policy = RoutingPolicy::new("policy-gpt-4-1", "chat_completion", "gpt-4.1")
        .with_priority(100)
        .with_strategy(RoutingStrategy::WeightedRandom)
        .with_ordered_provider_ids(vec![
            "provider-openrouter".to_owned(),
            "provider-openai-official".to_owned(),
        ])
        .with_default_provider_id("provider-openai-official");

    store.insert_routing_policy(&policy).await.unwrap();

    let policies = store.list_routing_policies().await.unwrap();
    assert_eq!(policies, vec![policy]);
    assert_eq!(policies[0].strategy, RoutingStrategy::WeightedRandom);
}

#[tokio::test]
async fn sqlite_store_persists_provider_health_snapshots_newest_first() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let older = ProviderHealthSnapshot::new("provider-a", "sdkwork.provider.a", "builtin", 100)
        .with_healthy(false)
        .with_running(true);
    let newer = ProviderHealthSnapshot::new("provider-a", "sdkwork.provider.a", "builtin", 200)
        .with_healthy(true)
        .with_running(true)
        .with_message("recovered");

    store.insert_provider_health_snapshot(&older).await.unwrap();
    store.insert_provider_health_snapshot(&newer).await.unwrap();

    let snapshots = store.list_provider_health_snapshots().await.unwrap();
    assert_eq!(snapshots.len(), 2);
    assert_eq!(snapshots[0].observed_at_ms, 200);
    assert_eq!(snapshots[0].message.as_deref(), Some("recovered"));
}
