use super::*;

#[serial(routing_runtime)]
#[tokio::test]
async fn route_simulation_selects_stale_primary_for_recovery_probe_by_default() {
    shutdown_all_connector_runtimes().unwrap();
    shutdown_all_native_dynamic_runtimes().unwrap();
    let _ttl = ScopedEnvVar::set(PROVIDER_HEALTH_TTL_ENV, "60000");
    let _probe = ScopedEnvVar::set(PROVIDER_HEALTH_RECOVERY_PROBE_PERCENT_ENV, "");

    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    store
        .insert_channel(&Channel::new("openai", "OpenAI"))
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                "provider-default-recovery-primary",
                "openai",
                "openai",
                "https://default-recovery-primary.example/v1",
                "Default Recovery Primary",
            )
            .with_extension_id("sdkwork.provider.snapshot.default.recovery.primary"),
        )
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                "provider-default-recovery-backup",
                "openai",
                "openai",
                "https://default-recovery-backup.example/v1",
                "Default Recovery Backup",
            )
            .with_extension_id("sdkwork.provider.snapshot.default.recovery.backup"),
        )
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new(
            "gpt-4.1",
            "provider-default-recovery-primary",
        ))
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new(
            "gpt-4.1",
            "provider-default-recovery-backup",
        ))
        .await
        .unwrap();

    let observed_at_ms = observed_at_now_ms();
    store
        .insert_provider_health_snapshot(
            &ProviderHealthSnapshot::new(
                "provider-default-recovery-primary",
                "sdkwork.provider.snapshot.default.recovery.primary",
                "builtin",
                observed_at_ms.saturating_sub(300_000),
            )
            .with_running(true)
            .with_healthy(false)
            .with_message("primary is stale unhealthy"),
        )
        .await
        .unwrap();
    store
        .insert_provider_health_snapshot(
            &ProviderHealthSnapshot::new(
                "provider-default-recovery-backup",
                "sdkwork.provider.snapshot.default.recovery.backup",
                "builtin",
                observed_at_ms,
            )
            .with_running(true)
            .with_healthy(true)
            .with_message("backup is healthy"),
        )
        .await
        .unwrap();

    let policy = RoutingPolicy::new(
        "policy-default-recovery-probe",
        "chat_completion",
        "gpt-4.1",
    )
    .with_priority(100)
    .with_ordered_provider_ids(vec![
        "provider-default-recovery-primary".to_owned(),
        "provider-default-recovery-backup".to_owned(),
    ]);
    persist_routing_policy(&store, &policy).await.unwrap();

    let decision = simulate_route_with_store_seeded(&store, "chat_completion", "gpt-4.1", 3)
        .await
        .unwrap();

    assert_eq!(
        decision.selected_provider_id,
        "provider-default-recovery-primary"
    );
    assert_eq!(
        decision.fallback_reason.as_deref(),
        Some("provider_health_recovery_probe")
    );
}

#[serial(routing_runtime)]
#[tokio::test]
async fn route_simulation_selects_stale_primary_for_recovery_probe_when_probe_cohort_enabled() {
    shutdown_all_connector_runtimes().unwrap();
    shutdown_all_native_dynamic_runtimes().unwrap();
    let _ttl = ScopedEnvVar::set(PROVIDER_HEALTH_TTL_ENV, "60000");
    let _probe = ScopedEnvVar::set(PROVIDER_HEALTH_RECOVERY_PROBE_PERCENT_ENV, "100");

    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    store
        .insert_channel(&Channel::new("openai", "OpenAI"))
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                "provider-recovery-primary",
                "openai",
                "openai",
                "https://recovery-primary.example/v1",
                "Recovery Primary",
            )
            .with_extension_id("sdkwork.provider.snapshot.recovery.primary"),
        )
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                "provider-recovery-backup",
                "openai",
                "openai",
                "https://recovery-backup.example/v1",
                "Recovery Backup",
            )
            .with_extension_id("sdkwork.provider.snapshot.recovery.backup"),
        )
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new(
            "gpt-4.1",
            "provider-recovery-primary",
        ))
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new(
            "gpt-4.1",
            "provider-recovery-backup",
        ))
        .await
        .unwrap();

    let observed_at_ms = observed_at_now_ms();
    store
        .insert_provider_health_snapshot(
            &ProviderHealthSnapshot::new(
                "provider-recovery-primary",
                "sdkwork.provider.snapshot.recovery.primary",
                "builtin",
                observed_at_ms.saturating_sub(300_000),
            )
            .with_running(true)
            .with_healthy(false)
            .with_message("primary was unhealthy but snapshot is now stale"),
        )
        .await
        .unwrap();
    store
        .insert_provider_health_snapshot(
            &ProviderHealthSnapshot::new(
                "provider-recovery-backup",
                "sdkwork.provider.snapshot.recovery.backup",
                "builtin",
                observed_at_ms,
            )
            .with_running(true)
            .with_healthy(true)
            .with_message("backup remains healthy"),
        )
        .await
        .unwrap();

    let policy = RoutingPolicy::new("policy-recovery-probe", "chat_completion", "gpt-4.1")
        .with_priority(100)
        .with_ordered_provider_ids(vec![
            "provider-recovery-primary".to_owned(),
            "provider-recovery-backup".to_owned(),
        ]);
    persist_routing_policy(&store, &policy).await.unwrap();

    let decision = simulate_route_with_store_seeded(&store, "chat_completion", "gpt-4.1", 42)
        .await
        .unwrap();

    assert_eq!(decision.selected_provider_id, "provider-recovery-primary");
    assert_eq!(decision.selection_seed, Some(42));
    assert_eq!(
        decision.fallback_reason.as_deref(),
        Some("provider_health_recovery_probe")
    );
    assert!(decision
        .selection_reason
        .as_deref()
        .unwrap()
        .contains("recovery probe"));
}

#[serial(routing_runtime)]
#[tokio::test]
async fn route_simulation_keeps_backup_when_request_is_outside_recovery_probe_cohort() {
    shutdown_all_connector_runtimes().unwrap();
    shutdown_all_native_dynamic_runtimes().unwrap();
    let _ttl = ScopedEnvVar::set(PROVIDER_HEALTH_TTL_ENV, "60000");
    let _probe = ScopedEnvVar::set(PROVIDER_HEALTH_RECOVERY_PROBE_PERCENT_ENV, "10");

    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    store
        .insert_channel(&Channel::new("openai", "OpenAI"))
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                "provider-recovery-gated-primary",
                "openai",
                "openai",
                "https://recovery-gated-primary.example/v1",
                "Recovery Gated Primary",
            )
            .with_extension_id("sdkwork.provider.snapshot.recovery.gated.primary"),
        )
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                "provider-recovery-gated-backup",
                "openai",
                "openai",
                "https://recovery-gated-backup.example/v1",
                "Recovery Gated Backup",
            )
            .with_extension_id("sdkwork.provider.snapshot.recovery.gated.backup"),
        )
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new(
            "gpt-4.1",
            "provider-recovery-gated-primary",
        ))
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new(
            "gpt-4.1",
            "provider-recovery-gated-backup",
        ))
        .await
        .unwrap();

    let observed_at_ms = observed_at_now_ms();
    store
        .insert_provider_health_snapshot(
            &ProviderHealthSnapshot::new(
                "provider-recovery-gated-primary",
                "sdkwork.provider.snapshot.recovery.gated.primary",
                "builtin",
                observed_at_ms.saturating_sub(300_000),
            )
            .with_running(true)
            .with_healthy(false)
            .with_message("primary is stale unhealthy"),
        )
        .await
        .unwrap();
    store
        .insert_provider_health_snapshot(
            &ProviderHealthSnapshot::new(
                "provider-recovery-gated-backup",
                "sdkwork.provider.snapshot.recovery.gated.backup",
                "builtin",
                observed_at_ms,
            )
            .with_running(true)
            .with_healthy(true)
            .with_message("backup is healthy"),
        )
        .await
        .unwrap();

    let policy = RoutingPolicy::new("policy-recovery-probe-gated", "chat_completion", "gpt-4.1")
        .with_priority(100)
        .with_ordered_provider_ids(vec![
            "provider-recovery-gated-primary".to_owned(),
            "provider-recovery-gated-backup".to_owned(),
        ]);
    persist_routing_policy(&store, &policy).await.unwrap();

    let decision = simulate_route_with_store_seeded(&store, "chat_completion", "gpt-4.1", 50)
        .await
        .unwrap();

    assert_eq!(
        decision.selected_provider_id,
        "provider-recovery-gated-backup"
    );
    assert_ne!(
        decision.fallback_reason.as_deref(),
        Some("provider_health_recovery_probe")
    );
}
