use super::*;

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
