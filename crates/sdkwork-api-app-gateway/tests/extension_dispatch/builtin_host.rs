use super::*;
use sdkwork_api_extension_core::ExtensionProtocol;

#[test]
fn builtin_host_registers_current_provider_extensions() {
    let host = builtin_extension_host();

    assert!(host.manifest("sdkwork.provider.openai.official").is_some());
    assert!(host.manifest("sdkwork.provider.xai").is_some());
    assert!(host.manifest("sdkwork.provider.deepseek").is_some());
    assert!(host.manifest("sdkwork.provider.qwen").is_some());
    assert!(host.manifest("sdkwork.provider.doubao").is_some());
    assert!(host.manifest("sdkwork.provider.hunyuan").is_some());
    assert!(host.manifest("sdkwork.provider.moonshot").is_some());
    assert!(host.manifest("sdkwork.provider.zhipu").is_some());
    assert!(host.manifest("sdkwork.provider.mistral").is_some());
    assert!(host.manifest("sdkwork.provider.cohere").is_some());
    assert!(host.manifest("sdkwork.provider.openrouter").is_some());
    assert!(host.manifest("sdkwork.provider.siliconflow").is_some());
    assert!(host.manifest("sdkwork.provider.ollama").is_some());

    assert!(host
        .resolve_provider("openai", "http://localhost")
        .is_some());
    assert!(host
        .resolve_provider("openrouter", "http://localhost")
        .is_some());
    assert!(host
        .resolve_provider("siliconflow", "http://localhost")
        .is_some());
    assert!(host
        .resolve_provider("ollama", "http://localhost")
        .is_some());
}

#[test]
fn builtin_host_manifests_expose_protocol_capabilities() {
    let host = builtin_extension_host();

    assert_eq!(
        host.manifest("sdkwork.provider.openai.official")
            .and_then(|manifest| manifest.protocol_capability()),
        Some(ExtensionProtocol::OpenAi)
    );
    assert_eq!(
        host.manifest("sdkwork.provider.openrouter")
            .and_then(|manifest| manifest.protocol_capability()),
        Some(ExtensionProtocol::OpenAi)
    );
    assert_eq!(
        host.manifest("sdkwork.provider.siliconflow")
            .and_then(|manifest| manifest.protocol_capability()),
        Some(ExtensionProtocol::OpenAi)
    );
    assert_eq!(
        host.manifest("sdkwork.provider.deepseek")
            .and_then(|manifest| manifest.protocol_capability()),
        Some(ExtensionProtocol::OpenAi)
    );
    assert_eq!(
        host.manifest("sdkwork.provider.ollama")
            .and_then(|manifest| manifest.protocol_capability()),
        Some(ExtensionProtocol::Custom)
    );
}

#[serial(extension_env)]
#[test]
fn builtin_host_resolves_provider_by_extension_id() {
    let host = builtin_extension_host();

    assert!(host
        .resolve_provider("sdkwork.provider.openai.official", "http://localhost")
        .is_some());
    assert!(host
        .resolve_provider("sdkwork.provider.xai", "http://localhost")
        .is_some());
    assert!(host
        .resolve_provider("sdkwork.provider.deepseek", "http://localhost")
        .is_some());
    assert!(host
        .resolve_provider("sdkwork.provider.qwen", "http://localhost")
        .is_some());
    assert!(host
        .resolve_provider("sdkwork.provider.doubao", "http://localhost")
        .is_some());
    assert!(host
        .resolve_provider("sdkwork.provider.hunyuan", "http://localhost")
        .is_some());
    assert!(host
        .resolve_provider("sdkwork.provider.moonshot", "http://localhost")
        .is_some());
    assert!(host
        .resolve_provider("sdkwork.provider.zhipu", "http://localhost")
        .is_some());
    assert!(host
        .resolve_provider("sdkwork.provider.mistral", "http://localhost")
        .is_some());
    assert!(host
        .resolve_provider("sdkwork.provider.cohere", "http://localhost")
        .is_some());
    assert!(host
        .resolve_provider("sdkwork.provider.openrouter", "http://localhost")
        .is_some());
    assert!(host
        .resolve_provider("sdkwork.provider.siliconflow", "http://localhost")
        .is_some());
    assert!(host
        .resolve_provider("sdkwork.provider.ollama", "http://localhost")
        .is_some());
}

#[serial(extension_env)]
#[tokio::test]
async fn missing_extension_id_falls_back_to_protocol_default_plugin() {
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
                "provider-openai-legacy",
                "openai",
                "native-dynamic",
                format!("http://{address}"),
                "Legacy OpenAI",
            )
            .with_extension_id("sdkwork.provider.missing-openai")
            .with_protocol_kind("openai"),
        )
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new("gpt-4.1", "provider-openai-legacy"))
        .await
        .unwrap();
    persist_credential_with_secret_and_manager(
        &store,
        &secret_manager,
        "tenant-1",
        "provider-openai-legacy",
        "cred-openai-legacy",
        "sk-upstream-openai",
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
    .expect("upstream response");

    assert_eq!(response["id"], "chatcmpl_upstream");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
}
