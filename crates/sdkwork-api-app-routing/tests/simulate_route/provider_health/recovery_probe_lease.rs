use super::*;

#[serial(routing_runtime)]
#[tokio::test]
async fn route_simulation_selects_stale_primary_for_recovery_probe_when_probe_lease_is_available() {
    shutdown_all_connector_runtimes().unwrap();
    shutdown_all_native_dynamic_runtimes().unwrap();
    let _ttl = ScopedEnvVar::set(PROVIDER_HEALTH_TTL_ENV, "60000");
    let _probe = ScopedEnvVar::set(PROVIDER_HEALTH_RECOVERY_PROBE_PERCENT_ENV, "100");
    let _lease_ttl = ScopedEnvVar::set(PROVIDER_HEALTH_RECOVERY_PROBE_LOCK_TTL_ENV, "30000");

    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    store
        .insert_channel(&Channel::new("openai", "OpenAI"))
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                "provider-recovery-lease-primary",
                "openai",
                "openai",
                "https://recovery-lease-primary.example/v1",
                "Recovery Lease Primary",
            )
            .with_extension_id("sdkwork.provider.snapshot.recovery.lease.primary"),
        )
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                "provider-recovery-lease-backup",
                "openai",
                "openai",
                "https://recovery-lease-backup.example/v1",
                "Recovery Lease Backup",
            )
            .with_extension_id("sdkwork.provider.snapshot.recovery.lease.backup"),
        )
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new(
            "gpt-4.1",
            "provider-recovery-lease-primary",
        ))
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new(
            "gpt-4.1",
            "provider-recovery-lease-backup",
        ))
        .await
        .unwrap();

    let observed_at_ms = observed_at_now_ms();
    store
        .insert_provider_health_snapshot(
            &ProviderHealthSnapshot::new(
                "provider-recovery-lease-primary",
                "sdkwork.provider.snapshot.recovery.lease.primary",
                "builtin",
                observed_at_ms.saturating_sub(300_000),
            )
            .with_running(true)
            .with_healthy(false)
            .with_message("primary is stale unhealthy and needs a controlled probe"),
        )
        .await
        .unwrap();
    store
        .insert_provider_health_snapshot(
            &ProviderHealthSnapshot::new(
                "provider-recovery-lease-backup",
                "sdkwork.provider.snapshot.recovery.lease.backup",
                "builtin",
                observed_at_ms,
            )
            .with_running(true)
            .with_healthy(true)
            .with_message("backup is healthy"),
        )
        .await
        .unwrap();

    let policy = RoutingPolicy::new("policy-recovery-probe-lease", "chat_completion", "gpt-4.1")
        .with_priority(100)
        .with_ordered_provider_ids(vec![
            "provider-recovery-lease-primary".to_owned(),
            "provider-recovery-lease-backup".to_owned(),
        ]);
    persist_routing_policy(&store, &policy).await.unwrap();

    let lease_store = MemoryCacheStore::default();
    let decision = simulate_route_with_store_selection_context(
        &store,
        "chat_completion",
        "gpt-4.1",
        RouteSelectionContext::new(RoutingDecisionSource::Gateway)
            .with_selection_seed_option(Some(42))
            .with_recovery_probe_lock_store_option(Some(&lease_store)),
    )
    .await
    .unwrap();

    assert_eq!(
        decision.selected_provider_id,
        "provider-recovery-lease-primary"
    );
    assert_eq!(decision.selection_seed, Some(42));
    assert_eq!(
        decision.fallback_reason.as_deref(),
        Some("provider_health_recovery_probe")
    );
    assert_eq!(
        decision
            .provider_health_recovery_probe
            .as_ref()
            .map(|probe| (probe.provider_id.as_str(), probe.outcome.as_str())),
        Some(("provider-recovery-lease-primary", "selected"))
    );
    assert!(!lease_store
        .try_acquire_lock(
            "provider-health-recovery-probe:provider-recovery-lease-primary",
            "second-owner",
            30_000,
        )
        .await
        .unwrap());
}

#[serial(routing_runtime)]
#[tokio::test]
async fn route_simulation_keeps_backup_when_recovery_probe_lease_is_already_held() {
    shutdown_all_connector_runtimes().unwrap();
    shutdown_all_native_dynamic_runtimes().unwrap();
    let _ttl = ScopedEnvVar::set(PROVIDER_HEALTH_TTL_ENV, "60000");
    let _probe = ScopedEnvVar::set(PROVIDER_HEALTH_RECOVERY_PROBE_PERCENT_ENV, "100");
    let _lease_ttl = ScopedEnvVar::set(PROVIDER_HEALTH_RECOVERY_PROBE_LOCK_TTL_ENV, "30000");

    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);

    store
        .insert_channel(&Channel::new("openai", "OpenAI"))
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                "provider-recovery-held-primary",
                "openai",
                "openai",
                "https://recovery-held-primary.example/v1",
                "Recovery Held Primary",
            )
            .with_extension_id("sdkwork.provider.snapshot.recovery.held.primary"),
        )
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                "provider-recovery-held-backup",
                "openai",
                "openai",
                "https://recovery-held-backup.example/v1",
                "Recovery Held Backup",
            )
            .with_extension_id("sdkwork.provider.snapshot.recovery.held.backup"),
        )
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new(
            "gpt-4.1",
            "provider-recovery-held-primary",
        ))
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new(
            "gpt-4.1",
            "provider-recovery-held-backup",
        ))
        .await
        .unwrap();

    let observed_at_ms = observed_at_now_ms();
    store
        .insert_provider_health_snapshot(
            &ProviderHealthSnapshot::new(
                "provider-recovery-held-primary",
                "sdkwork.provider.snapshot.recovery.held.primary",
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
                "provider-recovery-held-backup",
                "sdkwork.provider.snapshot.recovery.held.backup",
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
        "policy-recovery-probe-lease-held",
        "chat_completion",
        "gpt-4.1",
    )
    .with_priority(100)
    .with_ordered_provider_ids(vec![
        "provider-recovery-held-primary".to_owned(),
        "provider-recovery-held-backup".to_owned(),
    ]);
    persist_routing_policy(&store, &policy).await.unwrap();

    let lease_store = MemoryCacheStore::default();
    assert!(lease_store
        .try_acquire_lock(
            "provider-health-recovery-probe:provider-recovery-held-primary",
            "existing-owner",
            30_000,
        )
        .await
        .unwrap());

    let decision = simulate_route_with_store_selection_context(
        &store,
        "chat_completion",
        "gpt-4.1",
        RouteSelectionContext::new(RoutingDecisionSource::Gateway)
            .with_selection_seed_option(Some(42))
            .with_recovery_probe_lock_store_option(Some(&lease_store)),
    )
    .await
    .unwrap();

    assert_eq!(
        decision.selected_provider_id,
        "provider-recovery-held-backup"
    );
    assert_ne!(
        decision.fallback_reason.as_deref(),
        Some("provider_health_recovery_probe")
    );
    assert_eq!(
        decision
            .provider_health_recovery_probe
            .as_ref()
            .map(|probe| (probe.provider_id.as_str(), probe.outcome.as_str())),
        Some(("provider-recovery-held-primary", "lease_contended"))
    );
    assert!(decision
        .assessments
        .iter()
        .find(|assessment| assessment.provider_id == "provider-recovery-held-primary")
        .unwrap()
        .reasons
        .iter()
        .any(|reason| reason.contains("lease")));
}
