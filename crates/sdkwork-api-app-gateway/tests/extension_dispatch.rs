use axum::extract::State;
use axum::routing::{get, post};
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
    }
}

struct ExtensionEnvGuard {
    previous_paths: Option<String>,
    previous_connector: Option<String>,
    previous_native: Option<String>,
    previous_connector_signature: Option<String>,
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
    }
}

fn restore_env_var(key: &str, value: Option<&str>) {
    match value {
        Some(value) => std::env::set_var(key, value),
        None => std::env::remove_var(key),
    }
}
