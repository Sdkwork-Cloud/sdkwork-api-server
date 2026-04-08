use super::*;

#[tokio::test]
async fn route_simulation_geo_affinity_prefers_matching_region() {
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
                "provider-eu-west",
                "openai",
                "openai",
                "https://eu-west.example/v1",
                "EU West Provider",
            )
            .with_extension_id("sdkwork.provider.openai.official"),
        )
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                "provider-us-east",
                "openai",
                "openai",
                "https://us-east.example/v1",
                "US East Provider",
            )
            .with_extension_id("sdkwork.provider.openai.official"),
        )
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new("gpt-4.1", "provider-eu-west"))
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new("gpt-4.1", "provider-us-east"))
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
                "provider-eu-west",
                "builtin-openai",
                "sdkwork.provider.openai.official",
            )
            .with_enabled(true)
            .with_config(serde_json::json!({
                "routing": {
                    "region": "eu-west"
                }
            })),
        )
        .await
        .unwrap();
    store
        .insert_extension_instance(
            &ExtensionInstance::new(
                "provider-us-east",
                "builtin-openai",
                "sdkwork.provider.openai.official",
            )
            .with_enabled(true)
            .with_config(serde_json::json!({
                "region": "us-east"
            })),
        )
        .await
        .unwrap();

    let policy = RoutingPolicy::new("policy-geo-affinity", "chat_completion", "gpt-4.1")
        .with_priority(100)
        .with_strategy(RoutingStrategy::GeoAffinity)
        .with_ordered_provider_ids(vec![
            "provider-eu-west".to_owned(),
            "provider-us-east".to_owned(),
        ]);
    persist_routing_policy(&store, &policy).await.unwrap();

    let decision = simulate_route_with_store_context(
        &store,
        "chat_completion",
        "gpt-4.1",
        Some("us-east"),
        None,
    )
    .await
    .unwrap();

    assert_eq!(decision.selected_provider_id, "provider-us-east");
    assert_eq!(decision.strategy.as_deref(), Some("geo_affinity"));
    assert_eq!(decision.requested_region.as_deref(), Some("us-east"));
    assert_eq!(decision.assessments.len(), 2);
    assert_eq!(decision.assessments[0].provider_id, "provider-eu-west");
    assert_eq!(decision.assessments[0].region.as_deref(), Some("eu-west"));
    assert_eq!(decision.assessments[0].region_match, Some(false));
    assert_eq!(decision.assessments[1].provider_id, "provider-us-east");
    assert_eq!(decision.assessments[1].region.as_deref(), Some("us-east"));
    assert_eq!(decision.assessments[1].region_match, Some(true));
}

#[serial(routing_runtime)]
#[tokio::test]
async fn route_simulation_geo_affinity_degrades_to_top_ranked_candidate_when_no_region_matches() {
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
                "provider-primary",
                "openai",
                "openai",
                "https://primary.example/v1",
                "Primary Provider",
            )
            .with_extension_id("sdkwork.provider.openai.official"),
        )
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                "provider-secondary",
                "openai",
                "openai",
                "https://secondary.example/v1",
                "Secondary Provider",
            )
            .with_extension_id("sdkwork.provider.openai.official"),
        )
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new("gpt-4.1", "provider-primary"))
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new("gpt-4.1", "provider-secondary"))
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
                "provider-primary",
                "builtin-openai",
                "sdkwork.provider.openai.official",
            )
            .with_enabled(true)
            .with_config(serde_json::json!({
                "region": "eu-west"
            })),
        )
        .await
        .unwrap();
    store
        .insert_extension_instance(
            &ExtensionInstance::new(
                "provider-secondary",
                "builtin-openai",
                "sdkwork.provider.openai.official",
            )
            .with_enabled(true)
            .with_config(serde_json::json!({
                "routing": {
                    "region": "ap-southeast"
                }
            })),
        )
        .await
        .unwrap();

    let policy = RoutingPolicy::new("policy-geo-degraded", "chat_completion", "gpt-4.1")
        .with_priority(100)
        .with_strategy(RoutingStrategy::GeoAffinity)
        .with_ordered_provider_ids(vec![
            "provider-primary".to_owned(),
            "provider-secondary".to_owned(),
        ]);
    persist_routing_policy(&store, &policy).await.unwrap();

    let decision = simulate_route_with_store_context(
        &store,
        "chat_completion",
        "gpt-4.1",
        Some("us-east"),
        None,
    )
    .await
    .unwrap();

    assert_eq!(decision.selected_provider_id, "provider-primary");
    assert_eq!(decision.requested_region.as_deref(), Some("us-east"));
    assert_eq!(decision.strategy.as_deref(), Some("geo_affinity"));
    assert!(decision
        .selection_reason
        .as_deref()
        .unwrap()
        .contains("no candidate matched"));
    assert!(decision
        .fallback_reason
        .as_deref()
        .unwrap()
        .contains("no candidate matched requested region us-east"));
    assert!(decision
        .assessments
        .iter()
        .all(|assessment| assessment.region_match == Some(false)));
}

#[serial(routing_runtime)]
#[tokio::test]
async fn route_simulation_geo_affinity_keeps_health_precedence_over_matching_unhealthy_region() {
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
                "provider-match-unhealthy",
                "openai",
                "openai",
                "https://match-unhealthy.example/v1",
                "Matching Unhealthy Provider",
            )
            .with_extension_id("sdkwork.provider.snapshot.match"),
        )
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                "provider-healthy-backup",
                "openai",
                "openai",
                "https://healthy-backup.example/v1",
                "Healthy Backup Provider",
            )
            .with_extension_id("sdkwork.provider.snapshot.backup"),
        )
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new(
            "gpt-4.1",
            "provider-match-unhealthy",
        ))
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new(
            "gpt-4.1",
            "provider-healthy-backup",
        ))
        .await
        .unwrap();
    let observed_at_ms = observed_at_now_ms();
    store
        .insert_provider_health_snapshot(
            &ProviderHealthSnapshot::new(
                "provider-match-unhealthy",
                "sdkwork.provider.snapshot.match",
                "builtin",
                observed_at_ms.saturating_sub(1_000),
            )
            .with_running(true)
            .with_healthy(false),
        )
        .await
        .unwrap();
    store
        .insert_provider_health_snapshot(
            &ProviderHealthSnapshot::new(
                "provider-healthy-backup",
                "sdkwork.provider.snapshot.backup",
                "builtin",
                observed_at_ms,
            )
            .with_running(true)
            .with_healthy(true),
        )
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
                "provider-match-unhealthy",
                "builtin-openai",
                "sdkwork.provider.openai.official",
            )
            .with_enabled(true)
            .with_config(serde_json::json!({
                "region": "us-east"
            })),
        )
        .await
        .unwrap();
    store
        .insert_extension_instance(
            &ExtensionInstance::new(
                "provider-healthy-backup",
                "builtin-openai",
                "sdkwork.provider.openai.official",
            )
            .with_enabled(true)
            .with_config(serde_json::json!({
                "region": "eu-west"
            })),
        )
        .await
        .unwrap();

    let policy = RoutingPolicy::new("policy-geo-health", "chat_completion", "gpt-4.1")
        .with_priority(100)
        .with_strategy(RoutingStrategy::GeoAffinity)
        .with_ordered_provider_ids(vec![
            "provider-match-unhealthy".to_owned(),
            "provider-healthy-backup".to_owned(),
        ]);
    persist_routing_policy(&store, &policy).await.unwrap();

    let decision = simulate_route_with_store_context(
        &store,
        "chat_completion",
        "gpt-4.1",
        Some("us-east"),
        None,
    )
    .await
    .unwrap();

    assert_eq!(decision.selected_provider_id, "provider-healthy-backup");
    assert_eq!(decision.requested_region.as_deref(), Some("us-east"));
    assert_eq!(
        decision.assessments[0].provider_id,
        "provider-healthy-backup"
    );
    assert_eq!(
        decision.assessments[1].provider_id,
        "provider-match-unhealthy"
    );
    assert_eq!(decision.assessments[1].region_match, Some(true));
    assert!(decision.assessments[1]
        .reasons
        .iter()
        .any(|reason| reason.contains("healthy candidate is available")));
}

#[serial(routing_runtime)]
#[tokio::test]
async fn route_simulation_slo_aware_prefers_eligible_candidate_over_cheaper_violating_candidate() {
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
                "provider-cheap-violating",
                "openai",
                "openai",
                "https://cheap-violating.example/v1",
                "Cheap Violating Provider",
            )
            .with_extension_id("sdkwork.provider.openai.official"),
        )
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                "provider-compliant",
                "openai",
                "openai",
                "https://compliant.example/v1",
                "Compliant Provider",
            )
            .with_extension_id("sdkwork.provider.openai.official"),
        )
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new(
            "gpt-4.1",
            "provider-cheap-violating",
        ))
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new("gpt-4.1", "provider-compliant"))
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
                "provider-cheap-violating",
                "builtin-openai",
                "sdkwork.provider.openai.official",
            )
            .with_enabled(true)
            .with_config(serde_json::json!({
                "routing": {
                    "cost": 0.10,
                    "latency_ms": 450
                }
            })),
        )
        .await
        .unwrap();
    store
        .insert_extension_instance(
            &ExtensionInstance::new(
                "provider-compliant",
                "builtin-openai",
                "sdkwork.provider.openai.official",
            )
            .with_enabled(true)
            .with_config(serde_json::json!({
                "routing": {
                    "cost": 0.40,
                    "latency_ms": 120
                }
            })),
        )
        .await
        .unwrap();

    let policy = RoutingPolicy::new("policy-slo", "chat_completion", "gpt-4.1")
        .with_priority(100)
        .with_strategy(RoutingStrategy::SloAware)
        .with_max_cost(0.50)
        .with_max_latency_ms(200)
        .with_ordered_provider_ids(vec![
            "provider-cheap-violating".to_owned(),
            "provider-compliant".to_owned(),
        ]);
    persist_routing_policy(&store, &policy).await.unwrap();

    let decision = simulate_route_with_store(&store, "chat_completion", "gpt-4.1")
        .await
        .unwrap();

    assert_eq!(decision.selected_provider_id, "provider-compliant");
    assert_eq!(decision.strategy.as_deref(), Some("slo_aware"));
    assert!(decision
        .selection_reason
        .as_deref()
        .unwrap()
        .contains("SLO-compliant"));
    assert_eq!(decision.assessments.len(), 2);
    assert_eq!(
        decision.assessments[0].provider_id,
        "provider-cheap-violating"
    );
    assert_eq!(decision.assessments[0].slo_eligible, Some(false));
    assert_eq!(decision.assessments[1].provider_id, "provider-compliant");
    assert_eq!(decision.assessments[1].slo_eligible, Some(true));
}

#[serial(routing_runtime)]
#[tokio::test]
async fn route_simulation_slo_aware_degrades_to_top_ranked_candidate_when_no_candidate_meets_slo() {
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
                "provider-top-ranked",
                "openai",
                "openai",
                "https://top-ranked.example/v1",
                "Top Ranked Provider",
            )
            .with_extension_id("sdkwork.provider.openai.official"),
        )
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                "provider-second-ranked",
                "openai",
                "openai",
                "https://second-ranked.example/v1",
                "Second Ranked Provider",
            )
            .with_extension_id("sdkwork.provider.openai.official"),
        )
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new("gpt-4.1", "provider-top-ranked"))
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new("gpt-4.1", "provider-second-ranked"))
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
                "provider-top-ranked",
                "builtin-openai",
                "sdkwork.provider.openai.official",
            )
            .with_enabled(true)
            .with_config(serde_json::json!({
                "routing": {
                    "cost": 0.10,
                    "latency_ms": 550
                }
            })),
        )
        .await
        .unwrap();
    store
        .insert_extension_instance(
            &ExtensionInstance::new(
                "provider-second-ranked",
                "builtin-openai",
                "sdkwork.provider.openai.official",
            )
            .with_enabled(true)
            .with_config(serde_json::json!({
                "routing": {
                    "cost": 0.30,
                    "latency_ms": 600
                }
            })),
        )
        .await
        .unwrap();

    let policy = RoutingPolicy::new("policy-slo-degraded", "chat_completion", "gpt-4.1")
        .with_priority(100)
        .with_strategy(RoutingStrategy::SloAware)
        .with_max_latency_ms(200)
        .with_ordered_provider_ids(vec![
            "provider-top-ranked".to_owned(),
            "provider-second-ranked".to_owned(),
        ]);
    persist_routing_policy(&store, &policy).await.unwrap();

    let decision = simulate_route_with_store(&store, "chat_completion", "gpt-4.1")
        .await
        .unwrap();

    assert_eq!(decision.selected_provider_id, "provider-top-ranked");
    assert_eq!(decision.strategy.as_deref(), Some("slo_aware"));
    assert!(decision
        .selection_reason
        .as_deref()
        .unwrap()
        .contains("degraded"));
    assert!(decision
        .assessments
        .iter()
        .all(|assessment| assessment.slo_eligible == Some(false)));
}

#[serial(routing_runtime)]
#[tokio::test]
async fn select_route_with_store_persists_routing_decision_log() {
    shutdown_all_connector_runtimes().unwrap();
    shutdown_all_native_dynamic_runtimes().unwrap();

    let store = create_store_with_openai_channel().await;
    insert_openai_provider(
        &store,
        "provider-openai-official",
        "https://api.openai.com/v1",
        "OpenAI Official",
    )
    .await;

    store
        .insert_model(&ModelCatalogEntry::new(
            "gpt-4.1",
            "provider-openai-official",
        ))
        .await
        .unwrap();

    let decision = select_route_with_store(
        &store,
        "chat_completion",
        "gpt-4.1",
        RoutingDecisionSource::Gateway,
        Some("tenant-1"),
        Some("project-1"),
        None,
    )
    .await
    .unwrap();

    assert_eq!(decision.selected_provider_id, "provider-openai-official");

    let logs = store.list_routing_decision_logs().await.unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].decision_source, RoutingDecisionSource::Gateway);
    assert_eq!(logs[0].tenant_id.as_deref(), Some("tenant-1"));
    assert_eq!(logs[0].project_id.as_deref(), Some("project-1"));
    assert_eq!(logs[0].route_key, "gpt-4.1");
}

#[serial(routing_runtime)]
#[tokio::test]
async fn select_route_with_store_context_persists_requested_region_in_routing_decision_log() {
    shutdown_all_connector_runtimes().unwrap();
    shutdown_all_native_dynamic_runtimes().unwrap();

    let store = create_store_with_openai_channel().await;
    insert_openai_provider(
        &store,
        "provider-openai-official",
        "https://api.openai.com/v1",
        "OpenAI Official",
    )
    .await;

    store
        .insert_model(&ModelCatalogEntry::new(
            "gpt-4.1",
            "provider-openai-official",
        ))
        .await
        .unwrap();

    let decision = select_route_with_store_context(
        &store,
        "chat_completion",
        "gpt-4.1",
        RouteSelectionContext::new(RoutingDecisionSource::Gateway)
            .with_tenant_id_option(Some("tenant-1"))
            .with_project_id_option(Some("project-1"))
            .with_api_key_group_id_option(Some("group-live"))
            .with_requested_region_option(Some("us-east")),
    )
    .await
    .unwrap();

    assert_eq!(decision.requested_region.as_deref(), Some("us-east"));

    let logs = store.list_routing_decision_logs().await.unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].requested_region.as_deref(), Some("us-east"));
    assert_eq!(logs[0].api_key_group_id.as_deref(), Some("group-live"));
}

#[serial(routing_runtime)]
#[tokio::test]
async fn select_route_with_store_context_applies_project_routing_preferences_over_global_policy() {
    shutdown_all_connector_runtimes().unwrap();
    shutdown_all_native_dynamic_runtimes().unwrap();

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
            "https://openrouter.example/v1",
            "OpenRouter",
        ))
        .await
        .unwrap();
    store
        .insert_provider(&ProxyProvider::new(
            "provider-openai-official",
            "openai",
            "openai",
            "https://openai.example/v1",
            "OpenAI Official",
        ))
        .await
        .unwrap();
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

    let global_policy = RoutingPolicy::new("policy-global", "chat_completion", "gpt-4.1")
        .with_priority(100)
        .with_ordered_provider_ids(vec![
            "provider-openai-official".to_owned(),
            "provider-openrouter".to_owned(),
        ]);
    persist_routing_policy(&store, &global_policy)
        .await
        .unwrap();

    let preferences = ProjectRoutingPreferences::new("project-1")
        .with_preset_id("reliability")
        .with_strategy(RoutingStrategy::SloAware)
        .with_ordered_provider_ids(vec![
            "provider-openrouter".to_owned(),
            "provider-openai-official".to_owned(),
        ])
        .with_default_provider_id("provider-openai-official")
        .with_max_cost(0.30)
        .with_max_latency_ms(250)
        .with_require_healthy(true)
        .with_preferred_region("us-east")
        .with_updated_at_ms(123);
    store
        .insert_project_routing_preferences(&preferences)
        .await
        .unwrap();

    let decision = select_route_with_store_context(
        &store,
        "chat_completion",
        "gpt-4.1",
        RouteSelectionContext::new(RoutingDecisionSource::Gateway)
            .with_project_id_option(Some("project-1")),
    )
    .await
    .unwrap();

    assert_eq!(decision.selected_provider_id, "provider-openrouter");
    assert_eq!(decision.requested_region.as_deref(), Some("us-east"));
    assert_eq!(decision.strategy.as_deref(), Some("slo_aware"));
}

#[serial(routing_runtime)]
#[tokio::test]
async fn route_selection_applies_group_routing_profile_over_project_preferences() {
    shutdown_all_connector_runtimes().unwrap();
    shutdown_all_native_dynamic_runtimes().unwrap();

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
            "https://openrouter.example/v1",
            "OpenRouter",
        ))
        .await
        .unwrap();
    store
        .insert_provider(&ProxyProvider::new(
            "provider-openai-official",
            "openai",
            "openai",
            "https://openai.example/v1",
            "OpenAI Official",
        ))
        .await
        .unwrap();
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

    let global_policy = RoutingPolicy::new("policy-global", "chat_completion", "gpt-4.1")
        .with_priority(100)
        .with_ordered_provider_ids(vec![
            "provider-openai-official".to_owned(),
            "provider-openrouter".to_owned(),
        ]);
    persist_routing_policy(&store, &global_policy)
        .await
        .unwrap();

    let preferences = ProjectRoutingPreferences::new("project-1")
        .with_preset_id("balanced")
        .with_strategy(RoutingStrategy::DeterministicPriority)
        .with_ordered_provider_ids(vec![
            "provider-openai-official".to_owned(),
            "provider-openrouter".to_owned(),
        ])
        .with_default_provider_id("provider-openai-official")
        .with_preferred_region("eu-west")
        .with_updated_at_ms(123);
    store
        .insert_project_routing_preferences(&preferences)
        .await
        .unwrap();

    let profile = RoutingProfileRecord::new(
        "profile-priority",
        "tenant-1",
        "project-1",
        "Priority Live",
        "priority-live",
    )
    .with_strategy(RoutingStrategy::GeoAffinity)
    .with_ordered_provider_ids(vec![
        "provider-openrouter".to_owned(),
        "provider-openai-official".to_owned(),
    ])
    .with_default_provider_id("provider-openrouter")
    .with_preferred_region("us-east")
    .with_created_at_ms(100)
    .with_updated_at_ms(200);
    store.insert_routing_profile(&profile).await.unwrap();

    let group = ApiKeyGroupRecord::new(
        "group-live",
        "tenant-1",
        "project-1",
        "live",
        "Production Keys",
        "production-keys",
    )
    .with_default_routing_profile_id("profile-priority")
    .with_created_at_ms(100)
    .with_updated_at_ms(200);
    store.insert_api_key_group(&group).await.unwrap();

    let decision = select_route_with_store_context(
        &store,
        "chat_completion",
        "gpt-4.1",
        RouteSelectionContext::new(RoutingDecisionSource::Gateway)
            .with_tenant_id_option(Some("tenant-1"))
            .with_project_id_option(Some("project-1"))
            .with_api_key_group_id_option(Some("group-live")),
    )
    .await
    .unwrap();

    assert_eq!(decision.selected_provider_id, "provider-openrouter");
    assert_eq!(
        decision.applied_routing_profile_id.as_deref(),
        Some("profile-priority")
    );
    assert!(decision.compiled_routing_snapshot_id.as_deref().is_some());
    assert_eq!(decision.requested_region.as_deref(), Some("us-east"));
    assert_eq!(decision.strategy.as_deref(), Some("geo_affinity"));

    let snapshots = store.list_compiled_routing_snapshots().await.unwrap();
    assert_eq!(snapshots.len(), 1);
    assert_eq!(
        snapshots[0].snapshot_id.as_str(),
        decision.compiled_routing_snapshot_id.as_deref().unwrap()
    );
    assert_eq!(snapshots[0].tenant_id.as_deref(), Some("tenant-1"));
    assert_eq!(snapshots[0].project_id.as_deref(), Some("project-1"));
    assert_eq!(snapshots[0].api_key_group_id.as_deref(), Some("group-live"));
    assert_eq!(snapshots[0].capability, "chat_completion");
    assert_eq!(snapshots[0].route_key, "gpt-4.1");
    assert_eq!(
        snapshots[0].matched_policy_id.as_deref(),
        Some("policy-global")
    );
    assert_eq!(
        snapshots[0]
            .project_routing_preferences_project_id
            .as_deref(),
        Some("project-1")
    );
    assert_eq!(
        snapshots[0].applied_routing_profile_id.as_deref(),
        Some("profile-priority")
    );
    assert_eq!(snapshots[0].strategy, "geo_affinity");
    assert_eq!(
        snapshots[0].ordered_provider_ids,
        vec![
            "provider-openrouter".to_owned(),
            "provider-openai-official".to_owned(),
        ]
    );
    assert_eq!(snapshots[0].preferred_region.as_deref(), Some("us-east"));

    let logs = store.list_routing_decision_logs().await.unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(
        logs[0].applied_routing_profile_id.as_deref(),
        Some("profile-priority")
    );
    assert_eq!(
        logs[0].compiled_routing_snapshot_id.as_deref(),
        decision.compiled_routing_snapshot_id.as_deref()
    );
}
