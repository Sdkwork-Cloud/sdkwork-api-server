use super::*;

#[tokio::test]
async fn relay_uses_persisted_extension_instance_base_url_override() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/health", get(upstream_health_handler))
        .route("/v1/chat/completions", post(upstream_chat_handler))
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });
    wait_for_health(&format!("http://{address}")).await;

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
                "provider-openai-official",
                "openai",
                "custom-openai",
                "http://127.0.0.1:1",
                "OpenAI Official",
            )
            .with_extension_id("sdkwork.provider.openai.official"),
        )
        .await
        .unwrap();
    store
        .insert_model(
            &ModelCatalogEntry::new("gpt-4.1", "provider-openai-official").with_streaming(true),
        )
        .await
        .unwrap();
    persist_credential_with_secret_and_manager(
        &store,
        &secret_manager,
        "tenant-1",
        "provider-openai-official",
        "cred-openai",
        "sk-upstream-openai",
    )
    .await
    .unwrap();
    store
        .insert_extension_installation(
            &ExtensionInstallation::new(
                "openai-builtin",
                "sdkwork.provider.openai.official",
                ExtensionRuntime::Builtin,
            )
            .with_enabled(true)
            .with_config(json!({})),
        )
        .await
        .unwrap();
    store
        .insert_extension_instance(
            &ExtensionInstance::new(
                "provider-openai-official",
                "openai-builtin",
                "sdkwork.provider.openai.official",
            )
            .with_enabled(true)
            .with_base_url(format!("http://{address}"))
            .with_config(json!({})),
        )
        .await
        .unwrap();

    let request = chat_request("gpt-4.1");
    let response = relay_chat_completion_from_store(
        &store,
        &secret_manager,
        "tenant-1",
        "project-1",
        &request,
    )
    .await
    .unwrap()
    .expect("upstream response");

    assert_eq!(response["id"], "chatcmpl_upstream");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
}

#[serial(extension_env)]
#[tokio::test]
async fn disabled_extension_instance_prevents_upstream_relay() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/health", get(upstream_health_handler))
        .route("/v1/chat/completions", post(upstream_chat_handler))
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });
    wait_for_health(&format!("http://{address}")).await;

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
                "provider-openai-official",
                "openai",
                "custom-openai",
                format!("http://{address}"),
                "OpenAI Official",
            )
            .with_extension_id("sdkwork.provider.openai.official"),
        )
        .await
        .unwrap();
    store
        .insert_model(
            &ModelCatalogEntry::new("gpt-4.1", "provider-openai-official").with_streaming(true),
        )
        .await
        .unwrap();
    persist_credential_with_secret_and_manager(
        &store,
        &secret_manager,
        "tenant-1",
        "provider-openai-official",
        "cred-openai",
        "sk-upstream-openai",
    )
    .await
    .unwrap();
    store
        .insert_extension_installation(
            &ExtensionInstallation::new(
                "openai-builtin",
                "sdkwork.provider.openai.official",
                ExtensionRuntime::Builtin,
            )
            .with_enabled(true)
            .with_config(json!({})),
        )
        .await
        .unwrap();
    store
        .insert_extension_instance(
            &ExtensionInstance::new(
                "provider-openai-official",
                "openai-builtin",
                "sdkwork.provider.openai.official",
            )
            .with_enabled(false)
            .with_base_url(format!("http://{address}"))
            .with_config(json!({})),
        )
        .await
        .unwrap();

    let request = chat_request("gpt-4.1");
    let response = relay_chat_completion_from_store(
        &store,
        &secret_manager,
        "tenant-1",
        "project-1",
        &request,
    )
    .await
    .unwrap();

    assert!(response.is_none());
    assert!(upstream_state.authorization.lock().unwrap().is_none());
}

#[serial(extension_env)]
#[tokio::test]
async fn discovered_connector_extension_can_relay_through_supported_protocol() {
    let extension_root = temp_extension_root("discovered-connector");
    let package_dir = extension_root.join("sdkwork-provider-custom-openai");
    fs::create_dir_all(&package_dir).unwrap();
    fs::write(
        package_dir.join("sdkwork-extension.toml"),
        discovered_connector_manifest(),
    )
    .unwrap();
    let _guard = extension_env_guard(&extension_root);

    let upstream_state = UpstreamCaptureState::default();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let upstream_thread = thread::spawn({
        let upstream_state = upstream_state.clone();
        move || serve_connector_compatible_upstream(listener, upstream_state, 2)
    });
    wait_for_health(&format!("http://{address}")).await;

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
                "provider-custom-openai",
                "openai",
                "custom-openai",
                "http://127.0.0.1:1",
                "Custom OpenAI",
            )
            .with_extension_id("sdkwork.provider.custom-openai"),
        )
        .await
        .unwrap();
    store
        .insert_model(
            &ModelCatalogEntry::new("gpt-4.1", "provider-custom-openai").with_streaming(true),
        )
        .await
        .unwrap();
    persist_credential_with_secret_and_manager(
        &store,
        &secret_manager,
        "tenant-1",
        "provider-custom-openai",
        "cred-custom-openai",
        "sk-upstream-openai",
    )
    .await
    .unwrap();
    store
        .insert_extension_installation(
            &ExtensionInstallation::new(
                "custom-openai-installation",
                "sdkwork.provider.custom-openai",
                ExtensionRuntime::Connector,
            )
            .with_enabled(true)
            .with_entrypoint("bin/sdkwork-provider-custom-openai")
            .with_config(json!({})),
        )
        .await
        .unwrap();
    store
        .insert_extension_instance(
            &ExtensionInstance::new(
                "provider-custom-openai",
                "custom-openai-installation",
                "sdkwork.provider.custom-openai",
            )
            .with_enabled(true)
            .with_base_url(format!("http://{address}"))
            .with_config(json!({})),
        )
        .await
        .unwrap();

    let request = chat_request("gpt-4.1");
    let response = relay_chat_completion_from_store(
        &store,
        &secret_manager,
        "tenant-1",
        "project-1",
        &request,
    )
    .await
    .unwrap()
    .expect("upstream response");

    assert_eq!(response["id"], "chatcmpl_upstream");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );

    upstream_thread.join().unwrap();
    cleanup_dir(&extension_root);
}

#[serial(extension_env)]
#[tokio::test]
async fn extension_host_cache_reuses_configured_host_for_stable_policy() {
    let extension_root = temp_extension_root("configured-host-cache");
    let package_dir = extension_root.join("sdkwork-provider-custom-openai");
    fs::create_dir_all(&package_dir).unwrap();
    fs::write(
        package_dir.join("sdkwork-extension.toml"),
        discovered_connector_manifest(),
    )
    .unwrap();
    let _guard = extension_env_guard(&extension_root);

    let upstream_state = UpstreamCaptureState::default();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let upstream_thread = thread::spawn({
        let upstream_state = upstream_state.clone();
        move || serve_connector_compatible_upstream(listener, upstream_state, 2)
    });
    wait_for_health(&format!("http://{address}")).await;

    let request = chat_request("gpt-4.1");
    let first_response = execute_json_provider_request_with_runtime(
        "sdkwork.provider.custom-openai",
        format!("http://{address}"),
        "sk-upstream-openai",
        ProviderRequest::ChatCompletions(&request),
    )
    .await
    .unwrap()
    .expect("first cached response");

    assert_eq!(first_response["id"], "chatcmpl_upstream");

    cleanup_dir(&package_dir);

    let second_response = execute_json_provider_request_with_runtime(
        "sdkwork.provider.custom-openai",
        format!("http://{address}"),
        "sk-upstream-openai",
        ProviderRequest::ChatCompletions(&request),
    )
    .await
    .unwrap()
    .expect("second cached response");

    assert_eq!(second_response["id"], "chatcmpl_upstream");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );

    upstream_thread.join().unwrap();
    cleanup_dir(&extension_root);
}

#[serial(extension_env)]
#[tokio::test]
async fn configured_extension_host_reload_preserves_previous_host_when_rebuild_fails() {
    shutdown_all_native_dynamic_runtimes().unwrap();

    let extension_root = temp_extension_root("configured-host-reload-failure");
    let package_dir = extension_root.join("sdkwork-provider-native-mock");
    fs::create_dir_all(&package_dir).unwrap();
    let signing_key = SigningKey::from_bytes(&[10_u8; 32]);
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

    let first_response = relay_chat_completion_from_store(
        &store,
        &secret_manager,
        "tenant-1",
        "project-1",
        &chat_request("gpt-4.1"),
    )
    .await
    .unwrap()
    .expect("first native dynamic response");

    assert_eq!(first_response["id"], "chatcmpl_native_dynamic");

    fs::write(
        package_dir.join("sdkwork-extension.toml"),
        "not valid toml = [",
    )
    .unwrap();

    let error = reload_configured_extension_host().expect_err("reload should fail");
    assert!(
        error.to_string().contains("parse")
            || error.to_string().contains("manifest")
            || error.to_string().contains("toml"),
        "unexpected reload error: {error}"
    );

    let second_response = relay_chat_completion_from_store(
        &store,
        &secret_manager,
        "tenant-1",
        "project-1",
        &chat_request("gpt-4.1"),
    )
    .await
    .unwrap()
    .expect("second native dynamic response");

    assert_eq!(second_response["id"], "chatcmpl_native_dynamic");

    shutdown_all_native_dynamic_runtimes().unwrap();
    cleanup_dir(&extension_root);
}

#[serial(extension_env)]
#[tokio::test]
async fn unsigned_discovered_connector_extension_is_blocked_when_signature_is_required() {
    let extension_root = temp_extension_root("unsigned-connector-blocked");
    let package_dir = extension_root.join("sdkwork-provider-custom-openai");
    fs::create_dir_all(&package_dir).unwrap();
    fs::write(
        package_dir.join("sdkwork-extension.toml"),
        discovered_connector_manifest(),
    )
    .unwrap();
    let _guard = extension_env_guard_with_signature_requirement(&extension_root, true);

    let upstream_state = UpstreamCaptureState::default();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let _upstream_thread = thread::spawn({
        let upstream_state = upstream_state.clone();
        move || serve_connector_compatible_upstream(listener, upstream_state, 1)
    });
    wait_for_health(&format!("http://{address}")).await;

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
                "provider-custom-openai",
                "openai",
                "custom-openai",
                format!("http://{address}"),
                "Custom OpenAI",
            )
            .with_extension_id("sdkwork.provider.custom-openai"),
        )
        .await
        .unwrap();
    store
        .insert_model(
            &ModelCatalogEntry::new("gpt-4.1", "provider-custom-openai").with_streaming(true),
        )
        .await
        .unwrap();
    persist_credential_with_secret_and_manager(
        &store,
        &secret_manager,
        "tenant-1",
        "provider-custom-openai",
        "cred-custom-openai",
        "sk-upstream-openai",
    )
    .await
    .unwrap();
    store
        .insert_extension_installation(
            &ExtensionInstallation::new(
                "custom-openai-installation",
                "sdkwork.provider.custom-openai",
                ExtensionRuntime::Connector,
            )
            .with_enabled(true)
            .with_entrypoint("bin/sdkwork-provider-custom-openai")
            .with_config(json!({})),
        )
        .await
        .unwrap();
    store
        .insert_extension_instance(
            &ExtensionInstance::new(
                "provider-custom-openai",
                "custom-openai-installation",
                "sdkwork.provider.custom-openai",
            )
            .with_enabled(true)
            .with_base_url(format!("http://{address}"))
            .with_config(json!({})),
        )
        .await
        .unwrap();

    let request = chat_request("gpt-4.1");
    let response = relay_chat_completion_from_store(
        &store,
        &secret_manager,
        "tenant-1",
        "project-1",
        &request,
    )
    .await
    .unwrap();

    assert!(response.is_none());
    assert!(upstream_state.authorization.lock().unwrap().is_none());

    cleanup_dir(&extension_root);
}
