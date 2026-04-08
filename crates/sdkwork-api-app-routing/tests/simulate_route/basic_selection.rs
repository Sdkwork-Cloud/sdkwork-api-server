use super::*;

#[test]
fn route_simulation_prefers_healthy_low_cost_provider() {
    let decision = simulate_route("chat_completion", "gpt-4.1").unwrap();
    assert_eq!(decision.selected_provider_id, "provider-openai-official");
}

#[tokio::test]
async fn route_simulation_uses_catalog_model_candidates() {
    let store = create_store_with_openai_channel().await;
    insert_openai_provider(
        &store,
        "provider-openrouter",
        "https://openrouter.ai/api/v1",
        "OpenRouter",
    )
    .await;
    insert_openai_provider(
        &store,
        "provider-openai-official",
        "https://api.openai.com/v1",
        "OpenAI Official",
    )
    .await;
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
    let store = create_store_with_openai_channel().await;
    insert_openai_provider(
        &store,
        "provider-openrouter",
        "https://openrouter.ai/api/v1",
        "OpenRouter",
    )
    .await;
    insert_openai_provider(
        &store,
        "provider-openai-official",
        "https://api.openai.com/v1",
        "OpenAI Official",
    )
    .await;
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

#[serial(routing_runtime)]
#[tokio::test]
async fn route_simulation_respects_explicit_provider_order_before_lower_cost_hints() {
    shutdown_all_connector_runtimes().unwrap();
    shutdown_all_native_dynamic_runtimes().unwrap();

    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    store
        .insert_channel(&Channel::new("openai", "OpenAI"))
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                "provider-expensive",
                "openai",
                "openai",
                "https://expensive.example/v1",
                "Expensive Provider",
            )
            .with_extension_id("sdkwork.provider.openai.official"),
        )
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                "provider-cheap",
                "openai",
                "openai",
                "https://cheap.example/v1",
                "Cheap Provider",
            )
            .with_extension_id("sdkwork.provider.openai.official"),
        )
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new("gpt-4.1", "provider-expensive"))
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new("gpt-4.1", "provider-cheap"))
        .await
        .unwrap();
    store
        .insert_extension_installation(
            &ExtensionInstallation::new(
                "builtin-openai",
                "sdkwork.provider.openai.official",
                ExtensionRuntime::Builtin,
            )
            .with_enabled(true),
        )
        .await
        .unwrap();
    store
        .insert_extension_instance(
            &ExtensionInstance::new(
                "provider-expensive",
                "builtin-openai",
                "sdkwork.provider.openai.official",
            )
            .with_enabled(true)
            .with_config(serde_json::json!({
                "cost": 0.80,
                "weight": 50
            })),
        )
        .await
        .unwrap();
    store
        .insert_extension_instance(
            &ExtensionInstance::new(
                "provider-cheap",
                "builtin-openai",
                "sdkwork.provider.openai.official",
            )
            .with_enabled(true)
            .with_config(serde_json::json!({
                "routing": {
                    "cost": 0.25,
                    "latency_ms": 120
                },
                "weight": 100
            })),
        )
        .await
        .unwrap();

    let policy = RoutingPolicy::new("policy-cost-aware", "chat_completion", "gpt-4.1")
        .with_priority(100)
        .with_ordered_provider_ids(vec![
            "provider-expensive".to_owned(),
            "provider-cheap".to_owned(),
        ]);
    persist_routing_policy(&store, &policy).await.unwrap();

    let decision = simulate_route_with_store(&store, "chat_completion", "gpt-4.1")
        .await
        .unwrap();

    assert_eq!(decision.selected_provider_id, "provider-expensive");
    assert_eq!(decision.strategy.as_deref(), Some("deterministic_priority"));
    assert_eq!(decision.assessments.len(), 2);
    assert_eq!(decision.assessments[0].provider_id, "provider-expensive");
    assert_eq!(decision.assessments[1].provider_id, "provider-cheap");
    assert_eq!(decision.assessments[1].cost, Some(0.25));
}

#[serial(routing_runtime)]
#[tokio::test]
async fn route_simulation_demotes_disabled_provider_instance() {
    shutdown_all_connector_runtimes().unwrap();
    shutdown_all_native_dynamic_runtimes().unwrap();

    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    store
        .insert_channel(&Channel::new("openai", "OpenAI"))
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                "provider-disabled",
                "openai",
                "openai",
                "https://disabled.example/v1",
                "Disabled Provider",
            )
            .with_extension_id("sdkwork.provider.openai.official"),
        )
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                "provider-available",
                "openai",
                "openai",
                "https://available.example/v1",
                "Available Provider",
            )
            .with_extension_id("sdkwork.provider.openai.official"),
        )
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new("gpt-4.1", "provider-disabled"))
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new("gpt-4.1", "provider-available"))
        .await
        .unwrap();
    store
        .insert_extension_installation(
            &ExtensionInstallation::new(
                "builtin-openai",
                "sdkwork.provider.openai.official",
                ExtensionRuntime::Builtin,
            )
            .with_enabled(true),
        )
        .await
        .unwrap();
    store
        .insert_extension_instance(
            &ExtensionInstance::new(
                "provider-disabled",
                "builtin-openai",
                "sdkwork.provider.openai.official",
            )
            .with_enabled(false)
            .with_config(serde_json::json!({ "cost": 0.10 })),
        )
        .await
        .unwrap();
    store
        .insert_extension_instance(
            &ExtensionInstance::new(
                "provider-available",
                "builtin-openai",
                "sdkwork.provider.openai.official",
            )
            .with_enabled(true)
            .with_config(serde_json::json!({ "cost": 0.30 })),
        )
        .await
        .unwrap();

    let policy = RoutingPolicy::new("policy-disabled", "chat_completion", "gpt-4.1")
        .with_priority(100)
        .with_ordered_provider_ids(vec![
            "provider-disabled".to_owned(),
            "provider-available".to_owned(),
        ]);
    persist_routing_policy(&store, &policy).await.unwrap();

    let decision = simulate_route_with_store(&store, "chat_completion", "gpt-4.1")
        .await
        .unwrap();

    assert_eq!(decision.selected_provider_id, "provider-available");
    assert_eq!(decision.assessments[0].provider_id, "provider-available");
    assert_eq!(decision.assessments[1].provider_id, "provider-disabled");
    assert!(!decision.assessments[1].available);
}

#[serial(routing_runtime)]
#[tokio::test]
async fn route_simulation_can_use_seeded_weighted_random_strategy() {
    shutdown_all_connector_runtimes().unwrap();
    shutdown_all_native_dynamic_runtimes().unwrap();

    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    store
        .insert_channel(&Channel::new("openai", "OpenAI"))
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                "provider-light",
                "openai",
                "openai",
                "https://light.example/v1",
                "Light Provider",
            )
            .with_extension_id("sdkwork.provider.openai.official"),
        )
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                "provider-heavy",
                "openai",
                "openai",
                "https://heavy.example/v1",
                "Heavy Provider",
            )
            .with_extension_id("sdkwork.provider.openai.official"),
        )
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new("gpt-4.1", "provider-light"))
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new("gpt-4.1", "provider-heavy"))
        .await
        .unwrap();
    store
        .insert_extension_installation(
            &ExtensionInstallation::new(
                "builtin-openai",
                "sdkwork.provider.openai.official",
                ExtensionRuntime::Builtin,
            )
            .with_enabled(true),
        )
        .await
        .unwrap();
    store
        .insert_extension_instance(
            &ExtensionInstance::new(
                "provider-light",
                "builtin-openai",
                "sdkwork.provider.openai.official",
            )
            .with_enabled(true)
            .with_config(serde_json::json!({
                "routing": {
                    "weight": 10
                }
            })),
        )
        .await
        .unwrap();
    store
        .insert_extension_instance(
            &ExtensionInstance::new(
                "provider-heavy",
                "builtin-openai",
                "sdkwork.provider.openai.official",
            )
            .with_enabled(true)
            .with_config(serde_json::json!({
                "routing": {
                    "weight": 90
                }
            })),
        )
        .await
        .unwrap();

    let policy = RoutingPolicy::new("policy-weighted", "chat_completion", "gpt-4.1")
        .with_priority(100)
        .with_strategy(RoutingStrategy::WeightedRandom)
        .with_ordered_provider_ids(vec![
            "provider-light".to_owned(),
            "provider-heavy".to_owned(),
        ]);
    persist_routing_policy(&store, &policy).await.unwrap();

    let decision = simulate_route_with_store_seeded(&store, "chat_completion", "gpt-4.1", 15)
        .await
        .unwrap();

    assert_eq!(decision.selected_provider_id, "provider-heavy");
    assert_eq!(decision.strategy.as_deref(), Some("weighted_random"));
    assert_eq!(decision.selection_seed, Some(15));
}
