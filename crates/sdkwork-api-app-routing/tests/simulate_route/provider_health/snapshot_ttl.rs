use super::*;

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
    let observed_at_ms = observed_at_now_ms();
    store
        .insert_provider_health_snapshot(
            &ProviderHealthSnapshot::new(
                "provider-unhealthy-snapshot",
                "sdkwork.provider.snapshot.unhealthy",
                "builtin",
                observed_at_ms.saturating_sub(1_000),
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
                observed_at_ms,
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

#[serial(routing_runtime)]
#[tokio::test]
async fn route_simulation_ignores_stale_persisted_provider_health_snapshot() {
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
                "provider-stale-unhealthy",
                "openai",
                "openai",
                "https://stale-unhealthy.example/v1",
                "Stale Unhealthy Provider",
            )
            .with_extension_id("sdkwork.provider.snapshot.stale"),
        )
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                "provider-fresh-backup",
                "openai",
                "openai",
                "https://fresh-backup.example/v1",
                "Fresh Backup Provider",
            )
            .with_extension_id("sdkwork.provider.snapshot.fresh"),
        )
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new(
            "gpt-4.1",
            "provider-stale-unhealthy",
        ))
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new("gpt-4.1", "provider-fresh-backup"))
        .await
        .unwrap();

    let observed_at_ms = observed_at_now_ms();
    store
        .insert_provider_health_snapshot(
            &ProviderHealthSnapshot::new(
                "provider-stale-unhealthy",
                "sdkwork.provider.snapshot.stale",
                "builtin",
                observed_at_ms.saturating_sub(300_000),
            )
            .with_running(true)
            .with_healthy(false)
            .with_message("stale unhealthy"),
        )
        .await
        .unwrap();
    store
        .insert_provider_health_snapshot(
            &ProviderHealthSnapshot::new(
                "provider-fresh-backup",
                "sdkwork.provider.snapshot.fresh",
                "builtin",
                observed_at_ms.saturating_sub(300_000),
            )
            .with_running(true)
            .with_healthy(true)
            .with_message("stale healthy"),
        )
        .await
        .unwrap();

    let policy = RoutingPolicy::new("policy-stale-snapshot", "chat_completion", "gpt-4.1")
        .with_priority(100)
        .with_ordered_provider_ids(vec![
            "provider-stale-unhealthy".to_owned(),
            "provider-fresh-backup".to_owned(),
        ]);
    persist_routing_policy(&store, &policy).await.unwrap();

    let decision = simulate_route_with_store(&store, "chat_completion", "gpt-4.1")
        .await
        .unwrap();

    assert_eq!(decision.selected_provider_id, "provider-stale-unhealthy");
    assert_eq!(
        decision.assessments[0].provider_id,
        "provider-stale-unhealthy"
    );
    assert_eq!(
        decision.assessments[0].health,
        sdkwork_api_domain_routing::RoutingCandidateHealth::Unknown
    );
}

#[serial(routing_runtime)]
#[tokio::test]
async fn route_simulation_uses_configured_provider_health_ttl_to_keep_older_snapshot_fresh() {
    shutdown_all_connector_runtimes().unwrap();
    shutdown_all_native_dynamic_runtimes().unwrap();
    let _ttl = ScopedEnvVar::set(PROVIDER_HEALTH_TTL_ENV, "600000");

    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    store
        .insert_channel(&Channel::new("openai", "OpenAI"))
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                "provider-config-unhealthy",
                "openai",
                "openai",
                "https://config-unhealthy.example/v1",
                "Config Unhealthy Provider",
            )
            .with_extension_id("sdkwork.provider.snapshot.config.unhealthy"),
        )
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                "provider-config-healthy",
                "openai",
                "openai",
                "https://config-healthy.example/v1",
                "Config Healthy Provider",
            )
            .with_extension_id("sdkwork.provider.snapshot.config.healthy"),
        )
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new(
            "gpt-4.1",
            "provider-config-unhealthy",
        ))
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new(
            "gpt-4.1",
            "provider-config-healthy",
        ))
        .await
        .unwrap();

    let observed_at_ms = observed_at_now_ms();
    store
        .insert_provider_health_snapshot(
            &ProviderHealthSnapshot::new(
                "provider-config-unhealthy",
                "sdkwork.provider.snapshot.config.unhealthy",
                "builtin",
                observed_at_ms.saturating_sub(300_000),
            )
            .with_running(true)
            .with_healthy(false)
            .with_message("configured ttl still considers this unhealthy"),
        )
        .await
        .unwrap();
    store
        .insert_provider_health_snapshot(
            &ProviderHealthSnapshot::new(
                "provider-config-healthy",
                "sdkwork.provider.snapshot.config.healthy",
                "builtin",
                observed_at_ms.saturating_sub(300_000),
            )
            .with_running(true)
            .with_healthy(true)
            .with_message("configured ttl still considers this healthy"),
        )
        .await
        .unwrap();

    let policy = RoutingPolicy::new("policy-config-fresh", "chat_completion", "gpt-4.1")
        .with_priority(100)
        .with_ordered_provider_ids(vec![
            "provider-config-unhealthy".to_owned(),
            "provider-config-healthy".to_owned(),
        ]);
    persist_routing_policy(&store, &policy).await.unwrap();

    let decision = simulate_route_with_store(&store, "chat_completion", "gpt-4.1")
        .await
        .unwrap();

    assert_eq!(decision.selected_provider_id, "provider-config-healthy");
    assert_eq!(
        decision.assessments[0].provider_id,
        "provider-config-healthy"
    );
    assert_eq!(
        decision.assessments[0].health,
        sdkwork_api_domain_routing::RoutingCandidateHealth::Healthy
    );
}

#[serial(routing_runtime)]
#[tokio::test]
async fn route_simulation_uses_configured_provider_health_ttl_to_expire_recent_snapshot() {
    shutdown_all_connector_runtimes().unwrap();
    shutdown_all_native_dynamic_runtimes().unwrap();
    let _ttl = ScopedEnvVar::set(PROVIDER_HEALTH_TTL_ENV, "100");

    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    store
        .insert_channel(&Channel::new("openai", "OpenAI"))
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                "provider-short-ttl-primary",
                "openai",
                "openai",
                "https://short-ttl-primary.example/v1",
                "Short TTL Primary",
            )
            .with_extension_id("sdkwork.provider.snapshot.short.primary"),
        )
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                "provider-short-ttl-backup",
                "openai",
                "openai",
                "https://short-ttl-backup.example/v1",
                "Short TTL Backup",
            )
            .with_extension_id("sdkwork.provider.snapshot.short.backup"),
        )
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new(
            "gpt-4.1",
            "provider-short-ttl-primary",
        ))
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new(
            "gpt-4.1",
            "provider-short-ttl-backup",
        ))
        .await
        .unwrap();

    let observed_at_ms = observed_at_now_ms();
    store
        .insert_provider_health_snapshot(
            &ProviderHealthSnapshot::new(
                "provider-short-ttl-primary",
                "sdkwork.provider.snapshot.short.primary",
                "builtin",
                observed_at_ms.saturating_sub(1_000),
            )
            .with_running(true)
            .with_healthy(false)
            .with_message("short ttl should expire this unhealthy snapshot"),
        )
        .await
        .unwrap();
    store
        .insert_provider_health_snapshot(
            &ProviderHealthSnapshot::new(
                "provider-short-ttl-backup",
                "sdkwork.provider.snapshot.short.backup",
                "builtin",
                observed_at_ms.saturating_sub(1_000),
            )
            .with_running(true)
            .with_healthy(true)
            .with_message("short ttl should expire this healthy snapshot"),
        )
        .await
        .unwrap();

    let policy = RoutingPolicy::new("policy-short-ttl", "chat_completion", "gpt-4.1")
        .with_priority(100)
        .with_ordered_provider_ids(vec![
            "provider-short-ttl-primary".to_owned(),
            "provider-short-ttl-backup".to_owned(),
        ]);
    persist_routing_policy(&store, &policy).await.unwrap();

    let decision = simulate_route_with_store(&store, "chat_completion", "gpt-4.1")
        .await
        .unwrap();

    assert_eq!(decision.selected_provider_id, "provider-short-ttl-primary");
    assert_eq!(
        decision.assessments[0].provider_id,
        "provider-short-ttl-primary"
    );
    assert_eq!(
        decision.assessments[0].health,
        sdkwork_api_domain_routing::RoutingCandidateHealth::Unknown
    );
}
