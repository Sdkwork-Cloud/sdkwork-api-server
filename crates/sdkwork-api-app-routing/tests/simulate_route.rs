use sdkwork_api_app_routing::{persist_routing_policy, simulate_route, simulate_route_with_store};
use sdkwork_api_domain_catalog::{Channel, ModelCatalogEntry, ProxyProvider};
use sdkwork_api_domain_routing::RoutingPolicy;
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[test]
fn route_simulation_prefers_healthy_low_cost_provider() {
    let decision = simulate_route("chat_completion", "gpt-4.1").unwrap();
    assert_eq!(decision.selected_provider_id, "provider-openai-official");
}

#[tokio::test]
async fn route_simulation_uses_catalog_model_candidates() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    store
        .insert_model(&ModelCatalogEntry::new("gpt-4.1", "provider-openrouter"))
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new(
            "gpt-4.1",
            "provider-openai-official",
        ))
        .await
        .unwrap();

    let decision = simulate_route_with_store(&store, "chat_completion", "gpt-4.1")
        .await
        .unwrap();

    assert_eq!(decision.selected_provider_id, "provider-openai-official");
    assert_eq!(decision.candidate_ids.len(), 2);
}

#[tokio::test]
async fn route_simulation_prefers_policy_provider_order_over_lexicographic_sort() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    store
        .insert_model(&ModelCatalogEntry::new("gpt-4.1", "provider-openrouter"))
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new(
            "gpt-4.1",
            "provider-openai-official",
        ))
        .await
        .unwrap();

    let policy = RoutingPolicy::new("policy-gpt-4-1", "chat_completion", "gpt-4.1")
        .with_priority(100)
        .with_ordered_provider_ids(vec![
            "provider-openrouter".to_owned(),
            "provider-openai-official".to_owned(),
        ]);
    persist_routing_policy(&store, &policy).await.unwrap();

    let decision = simulate_route_with_store(&store, "chat_completion", "gpt-4.1")
        .await
        .unwrap();

    assert_eq!(decision.selected_provider_id, "provider-openrouter");
    assert_eq!(
        decision.candidate_ids,
        vec![
            "provider-openrouter".to_owned(),
            "provider-openai-official".to_owned(),
        ]
    );
    assert_eq!(
        decision.matched_policy_id.as_deref(),
        Some("policy-gpt-4-1")
    );
}

#[tokio::test]
async fn route_simulation_can_use_policy_without_catalog_model_candidates() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    store
        .insert_channel(&Channel::new("openai", "OpenAI"))
        .await
        .unwrap();
    store
        .insert_provider(&ProxyProvider::new(
            "provider-openrouter",
            "openai",
            "openai",
            "https://openrouter.ai/api/v1",
            "OpenRouter",
        ))
        .await
        .unwrap();

    let policy = RoutingPolicy::new("policy-response-read", "responses", "resp_*")
        .with_priority(50)
        .with_ordered_provider_ids(vec!["provider-openrouter".to_owned()]);
    persist_routing_policy(&store, &policy).await.unwrap();

    let decision = simulate_route_with_store(&store, "responses", "resp_123")
        .await
        .unwrap();

    assert_eq!(decision.selected_provider_id, "provider-openrouter");
    assert_eq!(
        decision.candidate_ids,
        vec!["provider-openrouter".to_owned()]
    );
    assert_eq!(
        decision.matched_policy_id.as_deref(),
        Some("policy-response-read")
    );
}
