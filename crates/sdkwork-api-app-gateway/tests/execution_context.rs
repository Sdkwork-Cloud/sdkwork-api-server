use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use sdkwork_api_app_credential::{
    persist_credential_with_secret_and_manager, CredentialSecretManager,
};
use sdkwork_api_app_gateway::{
    configure_gateway_provider_max_in_flight_limit, relay_chat_completion_from_store_with_options,
};
use sdkwork_api_app_routing::persist_routing_policy;
use sdkwork_api_contract_openai::chat_completions::{
    ChatMessageInput, CreateChatCompletionRequest,
};
use sdkwork_api_domain_catalog::{Channel, ModelCatalogEntry, ProxyProvider};
use sdkwork_api_domain_routing::RoutingPolicy;
use sdkwork_api_extension_core::{ExtensionInstallation, ExtensionInstance, ExtensionRuntime};
use sdkwork_api_observability::HttpMetricsRegistry;
use sdkwork_api_provider_core::ProviderRequestOptions;
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};
use serde_json::{json, Value};
use serial_test::serial;
use tokio::sync::Notify;

#[derive(Clone)]
struct UpstreamState {
    response_id: String,
    delay_ms: u64,
    attempts: Arc<AtomicUsize>,
    started: Option<Arc<Notify>>,
    release: Option<Arc<Notify>>,
}

#[derive(Clone)]
struct UpstreamServer {
    address: String,
    state: UpstreamState,
}

#[tokio::test]
#[serial]
async fn relay_chat_completion_times_out_primary_and_fails_over_to_backup() {
    let primary = spawn_upstream("chatcmpl_slow_primary", 200).await;
    let backup = spawn_upstream("chatcmpl_fast_backup", 0).await;

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
                "provider-primary",
                "openai",
                "openai",
                "http://127.0.0.1:1",
                "Primary",
            )
            .with_extension_id("sdkwork.provider.openai.official"),
        )
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                "provider-backup",
                "openai",
                "openai",
                "http://127.0.0.1:1",
                "Backup",
            )
            .with_extension_id("sdkwork.provider.openai.official"),
        )
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new("gpt-4.1", "provider-primary"))
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new("gpt-4.1", "provider-backup"))
        .await
        .unwrap();
    persist_credential_with_secret_and_manager(
        &store,
        &secret_manager,
        "tenant-1",
        "provider-primary",
        "cred-primary",
        "sk-primary",
    )
    .await
    .unwrap();
    persist_credential_with_secret_and_manager(
        &store,
        &secret_manager,
        "tenant-1",
        "provider-backup",
        "cred-backup",
        "sk-backup",
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
                "provider-primary",
                "openai-builtin",
                "sdkwork.provider.openai.official",
            )
            .with_enabled(true)
            .with_base_url(primary.address.clone())
            .with_config(json!({})),
        )
        .await
        .unwrap();
    store
        .insert_extension_instance(
            &ExtensionInstance::new(
                "provider-backup",
                "openai-builtin",
                "sdkwork.provider.openai.official",
            )
            .with_enabled(true)
            .with_base_url(backup.address.clone())
            .with_config(json!({})),
        )
        .await
        .unwrap();

    let policy = RoutingPolicy::new("policy-gpt-4-1", "chat_completion", "gpt-4.1")
        .with_priority(100)
        .with_ordered_provider_ids(vec![
            "provider-primary".to_owned(),
            "provider-backup".to_owned(),
        ])
        .with_upstream_retry_max_attempts(1);
    persist_routing_policy(&store, &policy).await.unwrap();

    let options = ProviderRequestOptions::new()
        .with_request_timeout_ms(50)
        .with_request_trace_id("trace-timeout");
    let response = relay_chat_completion_from_store_with_options(
        &store,
        &secret_manager,
        "tenant-1",
        "project-1",
        &chat_request("gpt-4.1"),
        &options,
    )
    .await
    .unwrap()
    .expect("backup response");

    let metrics = HttpMetricsRegistry::new("gateway").render_prometheus();

    assert_eq!(response["id"], "chatcmpl_fast_backup");
    assert_eq!(primary.state.attempts.load(Ordering::SeqCst), 1);
    assert_eq!(backup.state.attempts.load(Ordering::SeqCst), 1);
    assert!(metrics.contains(
        "sdkwork_gateway_execution_context_failures_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-primary\",reason=\"request_timeout\"}"
    ));
}

#[tokio::test]
#[serial]
async fn relay_chat_completion_fails_over_when_primary_provider_is_locally_overloaded() {
    let started = Arc::new(Notify::new());
    let release = Arc::new(Notify::new());
    let primary =
        spawn_blocking_upstream("chatcmpl_primary_busy", started.clone(), release.clone()).await;
    let backup = spawn_upstream("chatcmpl_backup_overflow", 0).await;

    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let secret_manager = CredentialSecretManager::database_encrypted("local-dev-master-key");
    seed_openai_failover_fixture(&store, &secret_manager, &primary.address, &backup.address).await;

    let reset_guard = GatewayInFlightLimitGuard::set(Some(1));
    let store_for_first = store.clone();
    let secret_manager_for_first = secret_manager.clone();
    let first_request = tokio::spawn({
        let request = chat_request("gpt-4.1");
        async move {
            relay_chat_completion_from_store_with_options(
                &store_for_first,
                &secret_manager_for_first,
                "tenant-1",
                "project-1",
                &request,
                &ProviderRequestOptions::new().with_request_trace_id("trace-primary"),
            )
            .await
        }
    });

    started.notified().await;

    let overflow_response = relay_chat_completion_from_store_with_options(
        &store,
        &secret_manager,
        "tenant-1",
        "project-1",
        &chat_request("gpt-4.1"),
        &ProviderRequestOptions::new().with_request_trace_id("trace-overflow"),
    )
    .await
    .unwrap()
    .expect("overflow response");

    let metrics = HttpMetricsRegistry::new("gateway").render_prometheus();

    assert_eq!(overflow_response["id"], "chatcmpl_backup_overflow");
    assert_eq!(primary.state.attempts.load(Ordering::SeqCst), 1);
    assert_eq!(backup.state.attempts.load(Ordering::SeqCst), 1);
    assert!(metrics.contains(
        "sdkwork_gateway_execution_context_failures_total{service=\"gateway\",capability=\"chat_completion\",provider=\"provider-primary\",reason=\"provider_overloaded\"}"
    ));
    let snapshots_after_overload = store.list_provider_health_snapshots().await.unwrap();
    assert!(
        snapshots_after_overload
            .iter()
            .all(|snapshot| snapshot.provider_id != "provider-primary"),
        "local overload should not persist an unhealthy primary snapshot before the primary request completes"
    );

    release.notify_waiters();
    let first_response = first_request
        .await
        .unwrap()
        .unwrap()
        .expect("primary response");
    assert_eq!(first_response["id"], "chatcmpl_primary_busy");

    drop(reset_guard);
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

async fn spawn_upstream(response_id: &str, delay_ms: u64) -> UpstreamServer {
    let state = UpstreamState {
        response_id: response_id.to_owned(),
        delay_ms,
        attempts: Arc::new(AtomicUsize::new(0)),
        started: None,
        release: None,
    };
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    let app = Router::new()
        .route("/v1/chat/completions", post(upstream_chat_handler))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    UpstreamServer { address, state }
}

async fn spawn_blocking_upstream(
    response_id: &str,
    started: Arc<Notify>,
    release: Arc<Notify>,
) -> UpstreamServer {
    let state = UpstreamState {
        response_id: response_id.to_owned(),
        delay_ms: 0,
        attempts: Arc::new(AtomicUsize::new(0)),
        started: Some(started),
        release: Some(release),
    };
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    let app = Router::new()
        .route("/v1/chat/completions", post(upstream_chat_handler))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    UpstreamServer { address, state }
}

async fn seed_openai_failover_fixture(
    store: &SqliteAdminStore,
    secret_manager: &CredentialSecretManager,
    primary_base_url: &str,
    backup_base_url: &str,
) {
    store
        .insert_channel(&Channel::new("openai", "OpenAI"))
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                "provider-primary",
                "openai",
                "openai",
                "http://127.0.0.1:1",
                "Primary",
            )
            .with_extension_id("sdkwork.provider.openai.official"),
        )
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                "provider-backup",
                "openai",
                "openai",
                "http://127.0.0.1:1",
                "Backup",
            )
            .with_extension_id("sdkwork.provider.openai.official"),
        )
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new("gpt-4.1", "provider-primary"))
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new("gpt-4.1", "provider-backup"))
        .await
        .unwrap();
    persist_credential_with_secret_and_manager(
        store,
        secret_manager,
        "tenant-1",
        "provider-primary",
        "cred-primary",
        "sk-primary",
    )
    .await
    .unwrap();
    persist_credential_with_secret_and_manager(
        store,
        secret_manager,
        "tenant-1",
        "provider-backup",
        "cred-backup",
        "sk-backup",
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
                "provider-primary",
                "openai-builtin",
                "sdkwork.provider.openai.official",
            )
            .with_enabled(true)
            .with_base_url(primary_base_url.to_owned())
            .with_config(json!({})),
        )
        .await
        .unwrap();
    store
        .insert_extension_instance(
            &ExtensionInstance::new(
                "provider-backup",
                "openai-builtin",
                "sdkwork.provider.openai.official",
            )
            .with_enabled(true)
            .with_base_url(backup_base_url.to_owned())
            .with_config(json!({})),
        )
        .await
        .unwrap();

    let policy = RoutingPolicy::new("policy-gpt-4-1", "chat_completion", "gpt-4.1")
        .with_priority(100)
        .with_ordered_provider_ids(vec![
            "provider-primary".to_owned(),
            "provider-backup".to_owned(),
        ])
        .with_upstream_retry_max_attempts(1);
    persist_routing_policy(store, &policy).await.unwrap();
}

async fn upstream_chat_handler(State(state): State<UpstreamState>) -> Json<Value> {
    state.attempts.fetch_add(1, Ordering::SeqCst);
    if let Some(started) = state.started.as_ref() {
        started.notify_waiters();
    }
    if state.delay_ms > 0 {
        tokio::time::sleep(Duration::from_millis(state.delay_ms)).await;
    }
    if let Some(release) = state.release.as_ref() {
        release.notified().await;
    }

    Json(json!({
        "id": state.response_id,
        "object": "chat.completion",
        "model": "gpt-4.1",
        "choices": []
    }))
}

struct GatewayInFlightLimitGuard;

impl GatewayInFlightLimitGuard {
    fn set(limit: Option<usize>) -> Self {
        configure_gateway_provider_max_in_flight_limit(limit);
        Self
    }
}

impl Drop for GatewayInFlightLimitGuard {
    fn drop(&mut self) {
        configure_gateway_provider_max_in_flight_limit(None);
    }
}
