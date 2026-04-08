use super::*;

#[tokio::test]
async fn native_dynamic_extension_can_relay_through_loaded_library() {
    let extension_root = temp_extension_root("native-dynamic");
    let package_dir = extension_root.join("sdkwork-provider-native-mock");
    fs::create_dir_all(&package_dir).unwrap();

    let signing_key = SigningKey::from_bytes(&[9_u8; 32]);
    let public_key = STANDARD.encode(signing_key.verifying_key().to_bytes());
    let library_path = native_dynamic_fixture_library_path();
    let manifest = native_dynamic_manifest(&library_path);
    let signature = sign_native_dynamic_package(&package_dir, &manifest, &signing_key);
    let manifest = manifest.with_trust(ExtensionTrustDeclaration::signed(
        "sdkwork",
        ExtensionSignature::new(
            ExtensionSignatureAlgorithm::Ed25519,
            public_key.clone(),
            signature,
        ),
    ));
    fs::write(
        package_dir.join("sdkwork-extension.toml"),
        toml::to_string(&manifest).unwrap(),
    )
    .unwrap();

    let _guard = native_dynamic_env_guard(&extension_root, &public_key);

    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let secret_manager = CredentialSecretManager::database_encrypted("local-dev-master-key");

    store
        .insert_channel(&Channel::new("openai", "OpenAI"))
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                "provider-native-mock",
                "openai",
                "native-dynamic",
                "https://native-dynamic.invalid/v1",
                "Native Mock",
            )
            .with_extension_id(FIXTURE_EXTENSION_ID),
        )
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new("gpt-4.1", "provider-native-mock"))
        .await
        .unwrap();
    persist_credential_with_secret_and_manager(
        &store,
        &secret_manager,
        "tenant-1",
        "provider-native-mock",
        "cred-native-mock",
        "sk-native",
    )
    .await
    .unwrap();
    store
        .insert_extension_installation(
            &ExtensionInstallation::new(
                "native-mock-installation",
                FIXTURE_EXTENSION_ID,
                ExtensionRuntime::NativeDynamic,
            )
            .with_enabled(true)
            .with_entrypoint(library_path.to_string_lossy())
            .with_config(json!({})),
        )
        .await
        .unwrap();
    store
        .insert_extension_instance(
            &ExtensionInstance::new(
                "provider-native-mock",
                "native-mock-installation",
                FIXTURE_EXTENSION_ID,
            )
            .with_enabled(true)
            .with_base_url("https://native-dynamic.invalid/v1")
            .with_config(json!({})),
        )
        .await
        .unwrap();

    let response = relay_chat_completion_from_store(
        &store,
        &secret_manager,
        "tenant-1",
        "project-1",
        &chat_request("gpt-4.1"),
    )
    .await
    .unwrap()
    .expect("native dynamic response");

    assert_eq!(response["id"], "chatcmpl_native_dynamic");

    cleanup_dir(&extension_root);
}

#[serial(extension_env)]
#[test]
fn configured_extension_host_reload_rebuilds_native_dynamic_runtimes() {
    shutdown_all_native_dynamic_runtimes().unwrap();

    let log_guard = NativeDynamicLifecycleLogGuard::new();
    let extension_root = temp_extension_root("native-dynamic-reload");
    let package_dir = extension_root.join("sdkwork-provider-native-mock");
    fs::create_dir_all(&package_dir).unwrap();

    let signing_key = SigningKey::from_bytes(&[11_u8; 32]);
    let public_key = STANDARD.encode(signing_key.verifying_key().to_bytes());
    let library_path = native_dynamic_fixture_library_path();
    let manifest = native_dynamic_manifest(&library_path);
    let signature = sign_native_dynamic_package(&package_dir, &manifest, &signing_key);
    let manifest = manifest.with_trust(ExtensionTrustDeclaration::signed(
        "sdkwork",
        ExtensionSignature::new(
            ExtensionSignatureAlgorithm::Ed25519,
            public_key.clone(),
            signature,
        ),
    ));
    let manifest_text = toml::to_string(&manifest).unwrap();
    fs::write(package_dir.join("sdkwork-extension.toml"), &manifest_text).unwrap();

    let _guard = native_dynamic_env_guard(&extension_root, &public_key);
    let direct_policy = ExtensionDiscoveryPolicy::new(vec![extension_root.clone()])
        .with_connector_extensions(false)
        .with_native_dynamic_extensions(true)
        .with_required_signatures_for_native_dynamic_extensions(true)
        .with_trusted_signer("sdkwork", &public_key);

    let packages = discover_extension_packages(&direct_policy).unwrap();
    assert_eq!(packages.len(), 1);

    let first = reload_configured_extension_host().unwrap();
    assert_eq!(first.discovered_package_count, 1);
    assert_eq!(first.loadable_package_count, 1);
    assert_eq!(
        std::fs::read_to_string(log_guard.path())
            .unwrap()
            .lines()
            .collect::<Vec<_>>(),
        vec!["init"]
    );

    let second = reload_configured_extension_host().unwrap();
    assert_eq!(second.discovered_package_count, 1);
    assert_eq!(second.loadable_package_count, 1);
    assert_eq!(
        std::fs::read_to_string(log_guard.path())
            .unwrap()
            .lines()
            .collect::<Vec<_>>(),
        vec!["init", "shutdown", "init"]
    );

    shutdown_all_native_dynamic_runtimes().unwrap();
    cleanup_dir(&extension_root);
}

#[serial(extension_env)]
#[test]
fn configured_extension_host_targeted_reload_does_not_reinitialize_unrelated_native_dynamic_runtime(
) {
    shutdown_all_native_dynamic_runtimes().unwrap();

    let log_guard = NativeDynamicLifecycleLogGuard::new();
    let extension_root = temp_extension_root("native-dynamic-targeted-reload-unrelated");
    let package_dir = extension_root.join("sdkwork-provider-native-mock");
    fs::create_dir_all(&package_dir).unwrap();

    let signing_key = SigningKey::from_bytes(&[13_u8; 32]);
    let public_key = STANDARD.encode(signing_key.verifying_key().to_bytes());
    let library_path = native_dynamic_fixture_library_path();
    let manifest = native_dynamic_manifest(&library_path);
    let signature = sign_native_dynamic_package(&package_dir, &manifest, &signing_key);
    let manifest = manifest.with_trust(ExtensionTrustDeclaration::signed(
        "sdkwork",
        ExtensionSignature::new(
            ExtensionSignatureAlgorithm::Ed25519,
            public_key.clone(),
            signature,
        ),
    ));
    let manifest_text = toml::to_string(&manifest).unwrap();
    fs::write(package_dir.join("sdkwork-extension.toml"), &manifest_text).unwrap();

    let _guard = native_dynamic_env_guard(&extension_root, &public_key);

    let first = reload_configured_extension_host().unwrap();
    assert_eq!(first.discovered_package_count, 1);
    assert_eq!(first.loadable_package_count, 1);
    assert_eq!(
        std::fs::read_to_string(log_guard.path())
            .unwrap()
            .lines()
            .collect::<Vec<_>>(),
        vec!["init"]
    );

    let second = reload_extension_host_with_scope(&ConfiguredExtensionHostReloadScope::Extension {
        extension_id: "sdkwork.provider.openai.official".to_owned(),
    })
    .unwrap();
    assert_eq!(second.discovered_package_count, 1);
    assert_eq!(second.loadable_package_count, 1);
    assert_eq!(
        std::fs::read_to_string(log_guard.path())
            .unwrap()
            .lines()
            .collect::<Vec<_>>(),
        vec!["init"]
    );

    shutdown_all_native_dynamic_runtimes().unwrap();
    cleanup_dir(&extension_root);
}

#[serial(extension_env)]
#[test]
fn configured_extension_host_targeted_reload_reinitializes_matching_native_dynamic_runtime() {
    shutdown_all_native_dynamic_runtimes().unwrap();

    let log_guard = NativeDynamicLifecycleLogGuard::new();
    let extension_root = temp_extension_root("native-dynamic-targeted-reload-matching");
    let package_dir = extension_root.join("sdkwork-provider-native-mock");
    fs::create_dir_all(&package_dir).unwrap();

    let signing_key = SigningKey::from_bytes(&[14_u8; 32]);
    let public_key = STANDARD.encode(signing_key.verifying_key().to_bytes());
    let library_path = native_dynamic_fixture_library_path();
    let manifest = native_dynamic_manifest(&library_path);
    let signature = sign_native_dynamic_package(&package_dir, &manifest, &signing_key);
    let manifest = manifest.with_trust(ExtensionTrustDeclaration::signed(
        "sdkwork",
        ExtensionSignature::new(
            ExtensionSignatureAlgorithm::Ed25519,
            public_key.clone(),
            signature,
        ),
    ));
    let manifest_text = toml::to_string(&manifest).unwrap();
    fs::write(package_dir.join("sdkwork-extension.toml"), &manifest_text).unwrap();

    let _guard = native_dynamic_env_guard(&extension_root, &public_key);

    let first = reload_configured_extension_host().unwrap();
    assert_eq!(first.discovered_package_count, 1);
    assert_eq!(first.loadable_package_count, 1);
    assert_eq!(
        std::fs::read_to_string(log_guard.path())
            .unwrap()
            .lines()
            .collect::<Vec<_>>(),
        vec!["init"]
    );

    let second = reload_extension_host_with_scope(&ConfiguredExtensionHostReloadScope::Extension {
        extension_id: FIXTURE_EXTENSION_ID.to_owned(),
    })
    .unwrap();
    assert_eq!(second.discovered_package_count, 1);
    assert_eq!(second.loadable_package_count, 1);
    assert_eq!(
        std::fs::read_to_string(log_guard.path())
            .unwrap()
            .lines()
            .collect::<Vec<_>>(),
        vec!["init", "shutdown", "init"]
    );

    shutdown_all_native_dynamic_runtimes().unwrap();
    cleanup_dir(&extension_root);
}

#[serial(extension_env)]
#[tokio::test]
async fn configured_extension_host_hot_reload_supervision_reloads_after_extension_tree_change() {
    shutdown_all_native_dynamic_runtimes().unwrap();

    let log_guard = NativeDynamicLifecycleLogGuard::new();
    let extension_root = temp_extension_root("native-dynamic-hot-reload");
    let package_dir = extension_root.join("sdkwork-provider-native-mock");
    fs::create_dir_all(&package_dir).unwrap();

    let signing_key = SigningKey::from_bytes(&[12_u8; 32]);
    let public_key = STANDARD.encode(signing_key.verifying_key().to_bytes());
    let library_path = native_dynamic_fixture_library_path();
    let manifest = native_dynamic_manifest(&library_path);
    let signature = sign_native_dynamic_package(&package_dir, &manifest, &signing_key);
    let manifest = manifest.with_trust(ExtensionTrustDeclaration::signed(
        "sdkwork",
        ExtensionSignature::new(
            ExtensionSignatureAlgorithm::Ed25519,
            public_key.clone(),
            signature,
        ),
    ));
    let manifest_text = toml::to_string(&manifest).unwrap();
    fs::write(package_dir.join("sdkwork-extension.toml"), &manifest_text).unwrap();

    let _guard = native_dynamic_env_guard(&extension_root, &public_key);

    let first = reload_configured_extension_host().unwrap();
    assert_eq!(first.discovered_package_count, 1);
    assert_eq!(first.loadable_package_count, 1);
    assert_eq!(
        std::fs::read_to_string(log_guard.path())
            .unwrap()
            .lines()
            .collect::<Vec<_>>(),
        vec!["init"]
    );

    let supervisor =
        start_configured_extension_hot_reload_supervision(1).expect("hot reload supervisor");
    fs::write(package_dir.join("sdkwork-extension.toml"), &manifest_text).unwrap();

    wait_for_lifecycle_log(log_guard.path(), &["init", "shutdown", "init"]).await;

    supervisor.abort();
    shutdown_all_native_dynamic_runtimes().unwrap();
    cleanup_dir(&extension_root);
}

#[serial(extension_env)]
#[tokio::test]
async fn configured_extension_host_reload_fails_safely_when_native_dynamic_drain_times_out() {
    shutdown_all_native_dynamic_runtimes().unwrap();

    let lifecycle_log = NativeDynamicLifecycleLogGuard::new();
    let invocation_log = NativeDynamicInvocationLogGuard::new();
    let delay_guard = NativeDynamicMockDelayGuard::json(250);
    let timeout_guard = NativeDynamicDrainTimeoutGuard::new(25);
    let extension_root = temp_extension_root("native-dynamic-reload-timeout");
    let package_dir = extension_root.join("sdkwork-provider-native-mock");
    fs::create_dir_all(&package_dir).unwrap();

    let signing_key = SigningKey::from_bytes(&[15_u8; 32]);
    let public_key = STANDARD.encode(signing_key.verifying_key().to_bytes());
    let library_path = native_dynamic_fixture_library_path();
    let manifest = native_dynamic_manifest(&library_path);
    let signature = sign_native_dynamic_package(&package_dir, &manifest, &signing_key);
    let manifest = manifest.with_trust(ExtensionTrustDeclaration::signed(
        "sdkwork",
        ExtensionSignature::new(
            ExtensionSignatureAlgorithm::Ed25519,
            public_key.clone(),
            signature,
        ),
    ));
    let manifest_text = toml::to_string(&manifest).unwrap();
    fs::write(package_dir.join("sdkwork-extension.toml"), &manifest_text).unwrap();

    let _guard = native_dynamic_env_guard(&extension_root, &public_key);

    let first = reload_configured_extension_host().unwrap();
    assert_eq!(first.discovered_package_count, 1);
    assert_eq!(first.loadable_package_count, 1);
    assert_eq!(read_log_lines(lifecycle_log.path()), vec!["init"]);

    let adapter = load_native_dynamic_provider_adapter(&library_path, "https://example.com/v1")
        .expect("native dynamic provider adapter");
    let request = chat_request("gpt-4.1");
    let invocation = std::thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("thread runtime")
            .block_on(async move {
                let output = adapter
                    .execute("sk-native", ProviderRequest::ChatCompletions(&request))
                    .await
                    .expect("native dynamic output");
                output.into_json().expect("json output")
            })
    });

    wait_for_log_line(invocation_log.path(), "execute_json_start").await;

    let reload = tokio::task::spawn_blocking(reload_configured_extension_host);
    let error = reload
        .await
        .expect("reload join")
        .expect_err("reload should fail on drain timeout");
    assert!(
        error.to_string().contains("drain"),
        "unexpected reload error: {error}"
    );
    assert_eq!(read_log_lines(lifecycle_log.path()), vec!["init"]);

    let output = invocation.join().expect("invocation thread");
    assert_eq!(output["id"], "chatcmpl_native_dynamic");

    drop(timeout_guard);
    drop(delay_guard);

    let second = reload_configured_extension_host().unwrap();
    assert_eq!(second.discovered_package_count, 1);
    assert_eq!(second.loadable_package_count, 1);
    wait_for_lifecycle_log(lifecycle_log.path(), &["init", "shutdown", "init"]).await;

    shutdown_all_native_dynamic_runtimes().unwrap();
    cleanup_dir(&extension_root);
}
