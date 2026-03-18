use sdkwork_api_domain_routing::{ProjectRoutingPreferences, RoutingStrategy};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};

#[tokio::test]
async fn sqlite_store_persists_project_routing_preferences() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let preferences = ProjectRoutingPreferences::new("project-1")
        .with_preset_id("balanced")
        .with_strategy(RoutingStrategy::SloAware)
        .with_ordered_provider_ids(vec![
            "provider-openrouter".to_owned(),
            "provider-openai-official".to_owned(),
        ])
        .with_default_provider_id("provider-openai-official")
        .with_max_cost(0.25)
        .with_max_latency_ms(200)
        .with_require_healthy(true)
        .with_preferred_region("us-east")
        .with_updated_at_ms(123);

    store
        .insert_project_routing_preferences(&preferences)
        .await
        .unwrap();

    let loaded = store
        .find_project_routing_preferences("project-1")
        .await
        .unwrap()
        .expect("preferences");

    assert_eq!(loaded, preferences);
}

#[tokio::test]
async fn sqlite_store_updates_project_routing_preferences_provider_order() {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    let initial = ProjectRoutingPreferences::new("project-1")
        .with_preset_id("balanced")
        .with_ordered_provider_ids(vec!["provider-a".to_owned()])
        .with_updated_at_ms(100);
    store
        .insert_project_routing_preferences(&initial)
        .await
        .unwrap();

    let updated = ProjectRoutingPreferences::new("project-1")
        .with_preset_id("regional")
        .with_strategy(RoutingStrategy::GeoAffinity)
        .with_ordered_provider_ids(vec!["provider-b".to_owned(), "provider-a".to_owned()])
        .with_preferred_region("eu-west")
        .with_updated_at_ms(200);
    store
        .insert_project_routing_preferences(&updated)
        .await
        .unwrap();

    let loaded = store
        .find_project_routing_preferences("project-1")
        .await
        .unwrap()
        .expect("preferences");

    assert_eq!(loaded.preset_id, "regional");
    assert_eq!(loaded.strategy, RoutingStrategy::GeoAffinity);
    assert_eq!(
        loaded.ordered_provider_ids,
        vec!["provider-b".to_owned(), "provider-a".to_owned()]
    );
    assert_eq!(loaded.preferred_region.as_deref(), Some("eu-west"));
    assert_eq!(loaded.updated_at_ms, 200);
}
