#[cfg(windows)]
use std::fs;
#[cfg(windows)]
use std::net::TcpListener;
#[cfg(windows)]
use std::path::{Path, PathBuf};
#[cfg(windows)]
use std::time::{SystemTime, UNIX_EPOCH};

use sdkwork_api_app_routing::RouteSelectionContext;
use sdkwork_api_app_routing::{
    persist_routing_policy, select_route_with_store, select_route_with_store_context,
    simulate_route, simulate_route_with_store, simulate_route_with_store_context,
    simulate_route_with_store_seeded,
};
use sdkwork_api_domain_catalog::{Channel, ModelCatalogEntry, ProxyProvider};
use sdkwork_api_domain_routing::{
    ProjectRoutingPreferences, ProviderHealthSnapshot, RoutingDecisionSource, RoutingPolicy,
    RoutingStrategy,
};
use sdkwork_api_extension_core::{ExtensionInstallation, ExtensionInstance, ExtensionRuntime};
#[cfg(windows)]
use sdkwork_api_extension_host::{
    ensure_connector_runtime_started, load_native_dynamic_provider_adapter, ExtensionLoadPlan,
};
use sdkwork_api_extension_host::{
    shutdown_all_connector_runtimes, shutdown_all_native_dynamic_runtimes,
};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};
use serial_test::serial;

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

#[serial(routing_runtime)]
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
    store
        .insert_provider_health_snapshot(
            &ProviderHealthSnapshot::new(
                "provider-match-unhealthy",
                "sdkwork.provider.snapshot.match",
                "builtin",
                100,
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
                200,
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

    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

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

    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

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
            .with_requested_region_option(Some("us-east")),
    )
    .await
    .unwrap();

    assert_eq!(decision.requested_region.as_deref(), Some("us-east"));

    let logs = store.list_routing_decision_logs().await.unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].requested_region.as_deref(), Some("us-east"));
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
async fn route_simulation_falls_back_to_persisted_provider_health_snapshot() {
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
                "provider-unhealthy-snapshot",
                "openai",
                "openai",
                "https://unhealthy.example/v1",
                "Unhealthy Snapshot Provider",
            )
            .with_extension_id("sdkwork.provider.snapshot.unhealthy"),
        )
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                "provider-healthy-snapshot",
                "openai",
                "openai",
                "https://healthy.example/v1",
                "Healthy Snapshot Provider",
            )
            .with_extension_id("sdkwork.provider.snapshot.healthy"),
        )
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new(
            "gpt-4.1",
            "provider-unhealthy-snapshot",
        ))
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new(
            "gpt-4.1",
            "provider-healthy-snapshot",
        ))
        .await
        .unwrap();
    store
        .insert_provider_health_snapshot(
            &ProviderHealthSnapshot::new(
                "provider-unhealthy-snapshot",
                "sdkwork.provider.snapshot.unhealthy",
                "builtin",
                100,
            )
            .with_running(true)
            .with_healthy(false)
            .with_message("persisted unhealthy"),
        )
        .await
        .unwrap();
    store
        .insert_provider_health_snapshot(
            &ProviderHealthSnapshot::new(
                "provider-healthy-snapshot",
                "sdkwork.provider.snapshot.healthy",
                "builtin",
                200,
            )
            .with_running(true)
            .with_healthy(true)
            .with_message("persisted healthy"),
        )
        .await
        .unwrap();

    let policy = RoutingPolicy::new("policy-snapshot", "chat_completion", "gpt-4.1")
        .with_priority(100)
        .with_ordered_provider_ids(vec![
            "provider-unhealthy-snapshot".to_owned(),
            "provider-healthy-snapshot".to_owned(),
        ]);
    persist_routing_policy(&store, &policy).await.unwrap();

    let decision = simulate_route_with_store(&store, "chat_completion", "gpt-4.1")
        .await
        .unwrap();

    assert_eq!(decision.selected_provider_id, "provider-healthy-snapshot");
    assert_eq!(
        decision.assessments[0].provider_id,
        "provider-healthy-snapshot"
    );
    assert_eq!(
        decision.assessments[0].health,
        sdkwork_api_domain_routing::RoutingCandidateHealth::Healthy
    );
}

#[cfg(windows)]
#[serial(routing_runtime)]
#[tokio::test]
async fn route_simulation_demotes_unhealthy_runtime_backed_provider() {
    shutdown_all_connector_runtimes().unwrap();
    shutdown_all_native_dynamic_runtimes().unwrap();

    let root = temp_extension_root("routing-unhealthy-connector");
    fs::create_dir_all(&root).unwrap();
    let port = free_port();
    let degrade_file = root.join("degrade.flag");
    fs::write(
        root.join("connector.ps1"),
        unstable_connector_script_body(port, &degrade_file),
    )
    .unwrap();

    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    store
        .insert_channel(&Channel::new("openai", "OpenAI"))
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                "provider-unhealthy",
                "openai",
                "openai",
                format!("http://127.0.0.1:{port}"),
                "Unhealthy Connector",
            )
            .with_extension_id("sdkwork.provider.connector.unhealthy"),
        )
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                "provider-healthy",
                "openai",
                "openai",
                "https://healthy.example/v1",
                "Healthy Native",
            )
            .with_extension_id("sdkwork.provider.native.mock"),
        )
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new("gpt-4.1", "provider-unhealthy"))
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new("gpt-4.1", "provider-healthy"))
        .await
        .unwrap();
    store
        .insert_extension_installation(
            &ExtensionInstallation::new(
                "connector-installation",
                "sdkwork.provider.connector.unhealthy",
                ExtensionRuntime::Connector,
            )
            .with_enabled(true)
            .with_entrypoint("powershell.exe")
            .with_config(serde_json::json!({
                "command_args": [
                    "-NoProfile",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-File",
                    "connector.ps1"
                ],
                "health_path": "/health",
                "startup_timeout_ms": 4000,
                "startup_poll_interval_ms": 50
            })),
        )
        .await
        .unwrap();
    store
        .insert_extension_instance(
            &ExtensionInstance::new(
                "provider-unhealthy",
                "connector-installation",
                "sdkwork.provider.connector.unhealthy",
            )
            .with_enabled(true)
            .with_base_url(format!("http://127.0.0.1:{port}")),
        )
        .await
        .unwrap();
    store
        .insert_extension_installation(
            &ExtensionInstallation::new(
                "native-installation",
                "sdkwork.provider.native.mock",
                ExtensionRuntime::NativeDynamic,
            )
            .with_enabled(true),
        )
        .await
        .unwrap();
    store
        .insert_extension_instance(
            &ExtensionInstance::new(
                "provider-healthy",
                "native-installation",
                "sdkwork.provider.native.mock",
            )
            .with_enabled(true)
            .with_config(serde_json::json!({ "routing": { "cost": 0.40 } })),
        )
        .await
        .unwrap();

    ensure_connector_runtime_started(
        &ExtensionLoadPlan {
            instance_id: "provider-unhealthy".to_owned(),
            installation_id: "connector-installation".to_owned(),
            extension_id: "sdkwork.provider.connector.unhealthy".to_owned(),
            enabled: true,
            runtime: ExtensionRuntime::Connector,
            display_name: "Unhealthy Connector".to_owned(),
            entrypoint: Some("powershell.exe".to_owned()),
            base_url: Some(format!("http://127.0.0.1:{port}")),
            credential_ref: None,
            config_schema: None,
            credential_schema: None,
            package_root: Some(root.clone()),
            config: serde_json::json!({
                "command_args": [
                    "-NoProfile",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-File",
                    "connector.ps1"
                ],
                "health_path": "/health",
                "startup_timeout_ms": 4000,
                "startup_poll_interval_ms": 50
            }),
        },
        &format!("http://127.0.0.1:{port}"),
    )
    .unwrap();
    let native_library = native_dynamic_fixture_library_path();
    let _adapter =
        load_native_dynamic_provider_adapter(&native_library, "https://healthy.example/v1")
            .unwrap();
    fs::write(&degrade_file, "degraded").unwrap();

    let policy = RoutingPolicy::new("policy-health", "chat_completion", "gpt-4.1")
        .with_priority(100)
        .with_ordered_provider_ids(vec![
            "provider-unhealthy".to_owned(),
            "provider-healthy".to_owned(),
        ]);
    persist_routing_policy(&store, &policy).await.unwrap();

    let decision = simulate_route_with_store(&store, "chat_completion", "gpt-4.1")
        .await
        .unwrap();

    assert_eq!(decision.selected_provider_id, "provider-healthy");
    assert_eq!(decision.assessments[0].provider_id, "provider-healthy");
    assert_eq!(
        decision.assessments[0].health,
        sdkwork_api_domain_routing::RoutingCandidateHealth::Healthy
    );
    assert_eq!(decision.assessments[1].provider_id, "provider-unhealthy");
    assert_eq!(
        decision.assessments[1].health,
        sdkwork_api_domain_routing::RoutingCandidateHealth::Unhealthy
    );

    shutdown_all_connector_runtimes().unwrap();
    shutdown_all_native_dynamic_runtimes().unwrap();
    cleanup_dir(&root);
}

#[cfg(windows)]
fn temp_extension_root(suffix: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    path.push(format!("sdkwork-app-routing-{suffix}-{millis}"));
    path
}

#[cfg(windows)]
fn cleanup_dir(path: &Path) {
    let _ = fs::remove_dir_all(path);
}

#[cfg(windows)]
fn unstable_connector_script_body(port: u16, degrade_file: &Path) -> String {
    format!(
        r#"
$degradeFile = '{}'
$listener = [System.Net.Sockets.TcpListener]::new([System.Net.IPAddress]::Parse("127.0.0.1"), {port})
$listener.Start()
while ($true) {{
    $client = $listener.AcceptTcpClient()
    $stream = $client.GetStream()
    $reader = New-Object System.IO.StreamReader($stream, [System.Text.Encoding]::ASCII, $false, 1024, $true)
    $requestLine = $reader.ReadLine()
    while ($true) {{
        $line = $reader.ReadLine()
        if ([string]::IsNullOrEmpty($line)) {{
            break
        }}
    }}

    if ($requestLine.StartsWith('GET /health')) {{
        if (Test-Path $degradeFile) {{
            $status = 'HTTP/1.1 503 Service Unavailable'
            $body = '{{"status":"degraded"}}'
        }} else {{
            $status = 'HTTP/1.1 200 OK'
            $body = '{{"status":"ok"}}'
        }}
    }} else {{
        $status = 'HTTP/1.1 404 Not Found'
        $body = '{{"error":"not_found"}}'
    }}

    $bodyBytes = [System.Text.Encoding]::UTF8.GetBytes($body)
    $writer = New-Object System.IO.StreamWriter($stream, [System.Text.Encoding]::ASCII, 1024, $true)
    $writer.NewLine = "`r`n"
    $writer.WriteLine($status)
    $writer.WriteLine('Content-Type: application/json')
    $writer.WriteLine(('Content-Length: ' + $bodyBytes.Length))
    $writer.WriteLine('Connection: close')
    $writer.WriteLine()
    $writer.Flush()
    $stream.Write($bodyBytes, 0, $bodyBytes.Length)
    $stream.Flush()
    $client.Close()
}}
"#,
        degrade_file.display().to_string().replace('\\', "\\\\")
    )
}

#[cfg(windows)]
fn native_dynamic_fixture_library_path() -> PathBuf {
    let current_exe = std::env::current_exe().expect("current exe");
    let directory = current_exe.parent().expect("exe dir");
    let prefix = if cfg!(windows) {
        "sdkwork_api_ext_provider_native_mock"
    } else {
        "libsdkwork_api_ext_provider_native_mock"
    };
    let extension = if cfg!(windows) {
        "dll"
    } else if cfg!(target_os = "macos") {
        "dylib"
    } else {
        "so"
    };

    std::fs::read_dir(directory)
        .expect("deps dir")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| {
            path.extension().and_then(|value| value.to_str()) == Some(extension)
                && path
                    .file_stem()
                    .and_then(|value| value.to_str())
                    .is_some_and(|stem| stem.starts_with(prefix))
        })
        .expect("native dynamic fixture library")
}

#[cfg(windows)]
fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}
