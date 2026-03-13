use sdkwork_api_domain_routing::RoutingPolicy;
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[tokio::test]
async fn sqlite_store_persists_routing_policies_with_provider_order() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let policy = RoutingPolicy::new("policy-gpt-4-1", "chat_completion", "gpt-4.1")
        .with_priority(100)
        .with_ordered_provider_ids(vec![
            "provider-openrouter".to_owned(),
            "provider-openai-official".to_owned(),
        ])
        .with_default_provider_id("provider-openai-official");

    store.insert_routing_policy(&policy).await.unwrap();

    let policies = store.list_routing_policies().await.unwrap();
    assert_eq!(policies, vec![policy]);
}
