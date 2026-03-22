use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use ed25519_dalek::SigningKey;
use sdkwork_api_app_credential::{
    persist_credential_with_secret_and_manager, CredentialSecretManager,
};
use sdkwork_api_app_gateway::{
    builtin_extension_host, execute_json_provider_request_with_runtime,
    relay_chat_completion_from_store, reload_configured_extension_host,
    reload_extension_host_with_scope, start_configured_extension_hot_reload_supervision,
    ConfiguredExtensionHostReloadScope,
};
use sdkwork_api_contract_openai::chat_completions::{
    ChatMessageInput, CreateChatCompletionRequest,
};
use sdkwork_api_domain_catalog::{Channel, ModelCatalogEntry, ProxyProvider};
use sdkwork_api_ext_provider_native_mock::FIXTURE_EXTENSION_ID;
use sdkwork_api_extension_core::{
    ExtensionInstallation, ExtensionInstance, ExtensionRuntime, ExtensionSignature,
    ExtensionSignatureAlgorithm, ExtensionTrustDeclaration,
};
use sdkwork_api_extension_host::{
    load_native_dynamic_provider_adapter, shutdown_all_native_dynamic_runtimes,
};
use sdkwork_api_provider_core::ProviderRequest;
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};
use serde_json::{json, Value};
use serial_test::serial;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{sleep, Duration};

#[serial(extension_env)]
#[test]
fn builtin_host_registers_current_provider_extensions() {
    let host = builtin_extension_host();

    assert!(host.manifest("sdkwork.provider.openai.official").is_some());
    assert!(host.manifest("sdkwork.provider.openrouter").is_some());
    assert!(host.manifest("sdkwork.provider.ollama").is_some());

    assert!(host
        .resolve_provider("openai", "http://localhost")
        .is_some());
    assert!(host
        .resolve_provider("openrouter", "http://localhost")
        .is_some());
    assert!(host
        .resolve_provider("ollama", "http://localhost")
        .is_some());
}

#[serial(extension_env)]
#[test]
fn builtin_host_resolves_provider_by_extension_id() {
    let host = builtin_extension_host();

    assert!(host
        .resolve_provider("sdkwork.provider.openai.official", "http://localhost")
        .is_some());
    assert!(host
        .resolve_provider("sdkwork.provider.openrouter", "http://localhost")
        .is_some());
    assert!(host
        .resolve_provider("sdkwork.provider.ollama", "http://localhost")
        .is_some());
}

#[derive(Clone, Default)]
struct UpstreamCaptureState {
    authorization: Arc<Mutex<Option<String>>>,
}

#[serial(extension_env)]
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

#[serial(extension_env)]
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

fn chat_request(model: &str) -> CreateChatCompletionRequest {
    CreateChatCompletionRequest {
        model: model.to_owned(),
        messages: vec![ChatMessageInput {
            role: "user".to_owned(),
            content: Value::String("hello".to_owned()),
            extra: serde_json::Map::new(),
        }],
        stream: None,
        extra: serde_json::Map::new(),
    }
}

async fn upstream_chat_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> Json<Value> {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    Json(json!({
        "id": "chatcmpl_upstream",
        "object": "chat.completion",
        "model": "gpt-4.1",
        "choices": []
    }))
}

async fn upstream_health_handler() -> Json<Value> {
    Json(json!({
        "status": "ok"
    }))
}

fn serve_connector_compatible_upstream(
    listener: std::net::TcpListener,
    state: UpstreamCaptureState,
    expected_requests: usize,
) {
    for _ in 0..expected_requests {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buffer = [0_u8; 4096];
        let bytes_read = stream.read(&mut buffer).unwrap();
        let request = String::from_utf8_lossy(&buffer[..bytes_read]);
        let request_line = request.lines().next().unwrap_or_default();
        let authorization = request
            .lines()
            .find_map(|line| {
                line.strip_prefix("authorization: ")
                    .or_else(|| line.strip_prefix("Authorization: "))
            })
            .map(str::trim)
            .map(ToOwned::to_owned);

        let (status, body) = if request_line.starts_with("GET /health") {
            ("HTTP/1.1 200 OK", r#"{"status":"ok"}"#.to_owned())
        } else if request_line.starts_with("POST /v1/chat/completions") {
            *state.authorization.lock().unwrap() = authorization;
            (
                "HTTP/1.1 200 OK",
                r#"{"id":"chatcmpl_upstream","object":"chat.completion","model":"gpt-4.1","choices":[]}"#
                    .to_owned(),
            )
        } else {
            (
                "HTTP/1.1 404 Not Found",
                r#"{"error":"not_found"}"#.to_owned(),
            )
        };

        let response = format!(
            "{status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }
}

fn discovered_connector_manifest() -> &'static str {
    r#"
api_version = "sdkwork.extension/v1"
id = "sdkwork.provider.custom-openai"
kind = "provider"
version = "0.1.0"
display_name = "Custom OpenAI"
runtime = "connector"
protocol = "openai"
entrypoint = "bin/sdkwork-provider-custom-openai"
channel_bindings = ["sdkwork.channel.openai"]
permissions = ["network_outbound", "spawn_process"]

[health]
path = "/health"
interval_secs = 30

[[capabilities]]
operation = "chat.completions.create"
compatibility = "relay"
"#
}

fn temp_extension_root(suffix: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    path.push(format!("sdkwork-app-gateway-{suffix}-{millis}"));
    path
}

fn cleanup_dir(path: &Path) {
    let _ = fs::remove_dir_all(path);
}

async fn wait_for_health(base_url: &str) {
    let health_url = format!("{}/health", base_url.trim_end_matches('/'));
    for _ in 0..20 {
        if let Ok(response) = reqwest::get(&health_url).await {
            if response.status().is_success() {
                return;
            }
        }
        sleep(Duration::from_millis(25)).await;
    }

    panic!("health endpoint did not become ready: {health_url}");
}

async fn wait_for_lifecycle_log(path: &Path, expected: &[&str]) {
    for _ in 0..120 {
        if read_log_lines(path)
            == expected
                .iter()
                .map(|line| (*line).to_owned())
                .collect::<Vec<_>>()
        {
            return;
        }
        sleep(Duration::from_millis(25)).await;
    }

    panic!(
        "lifecycle log did not reach expected state: {}",
        path.display()
    );
}

async fn wait_for_log_line(path: &Path, expected: &str) {
    for _ in 0..120 {
        if read_log_lines(path).iter().any(|line| line == expected) {
            return;
        }
        sleep(Duration::from_millis(25)).await;
    }

    panic!("log did not contain {expected}: {}", path.display());
}

fn read_log_lines(path: &Path) -> Vec<String> {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .map(ToOwned::to_owned)
        .collect()
}

fn extension_env_guard(path: &Path) -> ExtensionEnvGuard {
    extension_env_guard_with_signature_requirement(path, false)
}

fn extension_env_guard_with_signature_requirement(
    path: &Path,
    require_connector_signature: bool,
) -> ExtensionEnvGuard {
    let previous_paths = std::env::var("SDKWORK_EXTENSION_PATHS").ok();
    let previous_connector = std::env::var("SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS").ok();
    let previous_native = std::env::var("SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS").ok();
    let previous_connector_signature =
        std::env::var("SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS").ok();

    let joined_paths = std::env::join_paths([path]).unwrap();
    std::env::set_var("SDKWORK_EXTENSION_PATHS", joined_paths);
    std::env::set_var("SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS", "true");
    std::env::set_var(
        "SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS",
        "false",
    );
    std::env::set_var(
        "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS",
        if require_connector_signature {
            "true"
        } else {
            "false"
        },
    );

    ExtensionEnvGuard {
        previous_paths,
        previous_connector,
        previous_native,
        previous_connector_signature,
        previous_native_signature: None,
        previous_trusted_signers: None,
    }
}

struct ExtensionEnvGuard {
    previous_paths: Option<String>,
    previous_connector: Option<String>,
    previous_native: Option<String>,
    previous_connector_signature: Option<String>,
    previous_native_signature: Option<String>,
    previous_trusted_signers: Option<String>,
}

impl Drop for ExtensionEnvGuard {
    fn drop(&mut self) {
        restore_env_var("SDKWORK_EXTENSION_PATHS", self.previous_paths.as_deref());
        restore_env_var(
            "SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS",
            self.previous_connector.as_deref(),
        );
        restore_env_var(
            "SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS",
            self.previous_native.as_deref(),
        );
        restore_env_var(
            "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS",
            self.previous_connector_signature.as_deref(),
        );
        restore_env_var(
            "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS",
            self.previous_native_signature.as_deref(),
        );
        restore_env_var(
            "SDKWORK_EXTENSION_TRUSTED_SIGNERS",
            self.previous_trusted_signers.as_deref(),
        );
    }
}

fn restore_env_var(key: &str, value: Option<&str>) {
    match value {
        Some(value) => std::env::set_var(key, value),
        None => std::env::remove_var(key),
    }
}

fn native_dynamic_env_guard(path: &Path, public_key: &str) -> ExtensionEnvGuard {
    let previous_paths = std::env::var("SDKWORK_EXTENSION_PATHS").ok();
    let previous_connector = std::env::var("SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS").ok();
    let previous_native = std::env::var("SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS").ok();
    let previous_connector_signature =
        std::env::var("SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS").ok();
    let previous_native_signature =
        std::env::var("SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS").ok();
    let previous_trusted_signers = std::env::var("SDKWORK_EXTENSION_TRUSTED_SIGNERS").ok();

    let joined_paths = std::env::join_paths([path]).unwrap();
    std::env::set_var("SDKWORK_EXTENSION_PATHS", joined_paths);
    std::env::set_var("SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS", "false");
    std::env::set_var("SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS", "true");
    std::env::set_var(
        "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS",
        "false",
    );
    std::env::set_var(
        "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS",
        "true",
    );
    std::env::set_var(
        "SDKWORK_EXTENSION_TRUSTED_SIGNERS",
        format!("sdkwork={public_key}"),
    );

    ExtensionEnvGuard {
        previous_paths,
        previous_connector,
        previous_native,
        previous_connector_signature,
        previous_native_signature,
        previous_trusted_signers,
    }
}

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

fn native_dynamic_manifest(library_path: &Path) -> sdkwork_api_extension_core::ExtensionManifest {
    sdkwork_api_extension_core::ExtensionManifest::new(
        FIXTURE_EXTENSION_ID,
        sdkwork_api_extension_core::ExtensionKind::Provider,
        "0.1.0",
        ExtensionRuntime::NativeDynamic,
    )
    .with_display_name("Native Mock")
    .with_protocol(sdkwork_api_extension_core::ExtensionProtocol::OpenAi)
    .with_entrypoint(library_path.to_string_lossy())
    .with_channel_binding("sdkwork.channel.openai")
    .with_permission(sdkwork_api_extension_core::ExtensionPermission::NetworkOutbound)
    .with_capability(sdkwork_api_extension_core::CapabilityDescriptor::new(
        "chat.completions.create",
        sdkwork_api_extension_core::CompatibilityLevel::Native,
    ))
    .with_capability(sdkwork_api_extension_core::CapabilityDescriptor::new(
        "chat.completions.stream",
        sdkwork_api_extension_core::CompatibilityLevel::Native,
    ))
    .with_capability(sdkwork_api_extension_core::CapabilityDescriptor::new(
        "responses.create",
        sdkwork_api_extension_core::CompatibilityLevel::Native,
    ))
    .with_capability(sdkwork_api_extension_core::CapabilityDescriptor::new(
        "responses.stream",
        sdkwork_api_extension_core::CompatibilityLevel::Native,
    ))
    .with_capability(sdkwork_api_extension_core::CapabilityDescriptor::new(
        "audio.speech.create",
        sdkwork_api_extension_core::CompatibilityLevel::Native,
    ))
    .with_capability(sdkwork_api_extension_core::CapabilityDescriptor::new(
        "files.content",
        sdkwork_api_extension_core::CompatibilityLevel::Native,
    ))
    .with_capability(sdkwork_api_extension_core::CapabilityDescriptor::new(
        "videos.content",
        sdkwork_api_extension_core::CompatibilityLevel::Native,
    ))
}

fn sign_native_dynamic_package(
    package_dir: &Path,
    manifest: &sdkwork_api_extension_core::ExtensionManifest,
    signing_key: &SigningKey,
) -> String {
    use ed25519_dalek::Signer;
    use sha2::{Digest, Sha256};

    #[derive(serde::Serialize)]
    struct PackageSignaturePayload<'a> {
        manifest: &'a sdkwork_api_extension_core::ExtensionManifest,
        files: Vec<PackageFileDigest>,
    }

    #[derive(serde::Serialize)]
    struct PackageFileDigest {
        path: String,
        sha256: String,
    }

    let files = std::fs::read_dir(package_dir)
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name().and_then(|value| value.to_str()) != Some("sdkwork-extension.toml")
        })
        .map(|path| PackageFileDigest {
            path: path
                .strip_prefix(package_dir)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/"),
            sha256: format!("{:x}", Sha256::digest(std::fs::read(&path).unwrap())),
        })
        .collect::<Vec<_>>();

    let payload = serde_json::to_vec(&PackageSignaturePayload { manifest, files }).unwrap();
    let signature = signing_key.sign(&payload);
    STANDARD.encode(signature.to_bytes())
}

struct NativeDynamicLifecycleLogGuard {
    path: PathBuf,
    previous: Option<String>,
}

impl NativeDynamicLifecycleLogGuard {
    fn new() -> Self {
        let mut path = std::env::temp_dir();
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("unix time")
            .as_millis();
        path.push(format!(
            "sdkwork-app-gateway-native-dynamic-lifecycle-{millis}.log"
        ));

        let previous = std::env::var("SDKWORK_NATIVE_MOCK_LIFECYCLE_LOG").ok();
        std::env::set_var("SDKWORK_NATIVE_MOCK_LIFECYCLE_LOG", &path);

        Self { path, previous }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for NativeDynamicLifecycleLogGuard {
    fn drop(&mut self) {
        restore_env_var(
            "SDKWORK_NATIVE_MOCK_LIFECYCLE_LOG",
            self.previous.as_deref(),
        );
        let _ = std::fs::remove_file(&self.path);
    }
}

struct NativeDynamicInvocationLogGuard {
    path: PathBuf,
    previous: Option<String>,
}

impl NativeDynamicInvocationLogGuard {
    fn new() -> Self {
        let mut path = std::env::temp_dir();
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("unix time")
            .as_millis();
        path.push(format!(
            "sdkwork-app-gateway-native-dynamic-invocation-{millis}.log"
        ));

        let previous = std::env::var("SDKWORK_NATIVE_MOCK_INVOCATION_LOG").ok();
        std::env::set_var("SDKWORK_NATIVE_MOCK_INVOCATION_LOG", &path);

        Self { path, previous }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for NativeDynamicInvocationLogGuard {
    fn drop(&mut self) {
        restore_env_var(
            "SDKWORK_NATIVE_MOCK_INVOCATION_LOG",
            self.previous.as_deref(),
        );
        let _ = std::fs::remove_file(&self.path);
    }
}

struct NativeDynamicMockDelayGuard {
    previous_json_delay_ms: Option<String>,
    previous_stream_delay_ms: Option<String>,
}

impl NativeDynamicMockDelayGuard {
    fn json(delay_ms: u64) -> Self {
        let previous_json_delay_ms = std::env::var("SDKWORK_NATIVE_MOCK_JSON_DELAY_MS").ok();
        let previous_stream_delay_ms = std::env::var("SDKWORK_NATIVE_MOCK_STREAM_DELAY_MS").ok();
        std::env::set_var("SDKWORK_NATIVE_MOCK_JSON_DELAY_MS", delay_ms.to_string());
        std::env::remove_var("SDKWORK_NATIVE_MOCK_STREAM_DELAY_MS");
        Self {
            previous_json_delay_ms,
            previous_stream_delay_ms,
        }
    }
}

impl Drop for NativeDynamicMockDelayGuard {
    fn drop(&mut self) {
        restore_env_var(
            "SDKWORK_NATIVE_MOCK_JSON_DELAY_MS",
            self.previous_json_delay_ms.as_deref(),
        );
        restore_env_var(
            "SDKWORK_NATIVE_MOCK_STREAM_DELAY_MS",
            self.previous_stream_delay_ms.as_deref(),
        );
    }
}

struct NativeDynamicDrainTimeoutGuard {
    previous_timeout_ms: Option<String>,
}

impl NativeDynamicDrainTimeoutGuard {
    fn new(timeout_ms: u64) -> Self {
        let previous_timeout_ms =
            std::env::var("SDKWORK_NATIVE_DYNAMIC_SHUTDOWN_DRAIN_TIMEOUT_MS").ok();
        std::env::set_var(
            "SDKWORK_NATIVE_DYNAMIC_SHUTDOWN_DRAIN_TIMEOUT_MS",
            timeout_ms.to_string(),
        );
        Self {
            previous_timeout_ms,
        }
    }
}

impl Drop for NativeDynamicDrainTimeoutGuard {
    fn drop(&mut self) {
        restore_env_var(
            "SDKWORK_NATIVE_DYNAMIC_SHUTDOWN_DRAIN_TIMEOUT_MS",
            self.previous_timeout_ms.as_deref(),
        );
    }
}
