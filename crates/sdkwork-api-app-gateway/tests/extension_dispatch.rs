use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use sdkwork_api_app_credential::{
    persist_credential_with_secret_and_manager, CredentialSecretManager,
};
use sdkwork_api_app_gateway::{builtin_extension_host, relay_chat_completion_from_store};
use sdkwork_api_contract_openai::chat_completions::{
    ChatMessageInput, CreateChatCompletionRequest,
};
use sdkwork_api_domain_catalog::{Channel, ModelCatalogEntry, ProxyProvider};
use sdkwork_api_extension_core::{ExtensionInstallation, ExtensionInstance, ExtensionRuntime};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};

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

#[tokio::test]
async fn relay_uses_persisted_extension_instance_base_url_override() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/chat/completions", post(upstream_chat_handler))
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
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

#[tokio::test]
async fn disabled_extension_instance_prevents_upstream_relay() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/chat/completions", post(upstream_chat_handler))
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
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

fn chat_request(model: &str) -> CreateChatCompletionRequest {
    CreateChatCompletionRequest {
        model: model.to_owned(),
        messages: vec![ChatMessageInput {
            role: "user".to_owned(),
            content: Value::String("hello".to_owned()),
        }],
        stream: None,
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
