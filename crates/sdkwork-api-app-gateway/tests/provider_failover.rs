use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use sdkwork_api_app_credential::{
    persist_credential_with_secret_and_manager, CredentialSecretManager,
};
use sdkwork_api_app_gateway::relay_chat_completion_from_store;
use sdkwork_api_app_routing::persist_routing_policy;
use sdkwork_api_contract_openai::chat_completions::{
    ChatMessageInput, CreateChatCompletionRequest,
};
use sdkwork_api_domain_catalog::{Channel, ModelCatalogEntry, ProxyProvider};
use sdkwork_api_domain_routing::RoutingPolicy;
use sdkwork_api_extension_core::{ExtensionInstallation, ExtensionInstance, ExtensionRuntime};
use sdkwork_api_observability::{
    annotate_current_http_metrics, with_current_http_metrics_registry, HttpMetricsRegistry,
};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
struct UpstreamState {
    authorization: Arc<Mutex<Option<String>>>,
    request_count: Arc<Mutex<usize>>,
}

#[derive(Clone)]
struct UpstreamServer {
    address: String,
    state: UpstreamState,
}

#[tokio::test]
async fn relay_chat_completion_fails_over_and_records_provider_health_evidence() {
    let primary = spawn_failing_upstream().await;
    let secondary = spawn_success_upstream("chatcmpl_secondary").await;

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
                "Primary Provider",
            )
            .with_extension_id("sdkwork.provider.openai.official"),
        )
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                "provider-secondary",
                "openai",
                "openai",
                "http://127.0.0.1:1",
                "Secondary Provider",
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
        .insert_model(&ModelCatalogEntry::new("gpt-4.1", "provider-secondary"))
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
        "provider-secondary",
        "cred-secondary",
        "sk-secondary",
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
                "provider-secondary",
                "openai-builtin",
                "sdkwork.provider.openai.official",
            )
            .with_enabled(true)
            .with_base_url(secondary.address.clone())
            .with_config(json!({})),
        )
        .await
        .unwrap();

    let policy = RoutingPolicy::new("policy-failover", "chat_completion", "gpt-4.1")
        .with_priority(100)
        .with_upstream_retry_max_attempts(1)
        .with_ordered_provider_ids(vec![
            "provider-primary".to_owned(),
            "provider-secondary".to_owned(),
        ]);
    persist_routing_policy(&store, &policy).await.unwrap();

    let first_response = relay_chat_completion_from_store(
        &store,
        &secret_manager,
        "tenant-1",
        "project-1",
        &chat_request("gpt-4.1"),
    )
    .await
    .unwrap()
    .expect("gateway response");

    assert_eq!(first_response["id"], "chatcmpl_secondary");
    assert_eq!(*primary.state.request_count.lock().unwrap(), 1);
    assert_eq!(*secondary.state.request_count.lock().unwrap(), 1);
    assert_eq!(
        primary.state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-primary")
    );
    assert_eq!(
        secondary.state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-secondary")
    );

    let snapshots = store.list_provider_health_snapshots().await.unwrap();
    let primary_snapshot = snapshots
        .iter()
        .find(|snapshot| snapshot.provider_id == "provider-primary")
        .expect("primary health snapshot");
    assert!(primary_snapshot.running);
    assert!(!primary_snapshot.healthy);
    assert!(primary_snapshot
        .message
        .as_deref()
        .unwrap_or_default()
        .contains("502"));

    let second_response = relay_chat_completion_from_store(
        &store,
        &secret_manager,
        "tenant-1",
        "project-1",
        &chat_request("gpt-4.1"),
    )
    .await
    .unwrap()
    .expect("gateway response");

    assert_eq!(second_response["id"], "chatcmpl_secondary");
    assert_eq!(
        *primary.state.request_count.lock().unwrap(),
        1,
        "persisted unhealthy snapshot should suppress the primary provider on the next route"
    );
    assert_eq!(*secondary.state.request_count.lock().unwrap(), 2);
}

#[tokio::test]
async fn relay_chat_completion_records_provider_telemetry_for_retry_and_failover() {
    let primary = spawn_failing_upstream().await;
    let secondary = spawn_success_upstream("chatcmpl_secondary").await;

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
                "Primary Provider",
            )
            .with_extension_id("sdkwork.provider.openai.official"),
        )
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                "provider-secondary",
                "openai",
                "openai",
                "http://127.0.0.1:1",
                "Secondary Provider",
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
        .insert_model(&ModelCatalogEntry::new("gpt-4.1", "provider-secondary"))
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
        "provider-secondary",
        "cred-secondary",
        "sk-secondary",
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
                "provider-secondary",
                "openai-builtin",
                "sdkwork.provider.openai.official",
            )
            .with_enabled(true)
            .with_base_url(secondary.address.clone())
            .with_config(json!({})),
        )
        .await
        .unwrap();

    let policy = RoutingPolicy::new("policy-failover", "chat_completion", "gpt-4.1")
        .with_priority(100)
        .with_upstream_retry_max_attempts(1)
        .with_ordered_provider_ids(vec![
            "provider-primary".to_owned(),
            "provider-secondary".to_owned(),
        ]);
    persist_routing_policy(&store, &policy).await.unwrap();

    let telemetry = Arc::new(HttpMetricsRegistry::new("gateway"));
    with_current_http_metrics_registry(telemetry.clone(), async {
        annotate_current_http_metrics(|dimensions| {
            dimensions.route = Some("/v1/chat/completions".to_owned());
            dimensions.tenant = Some("tenant-1".to_owned());
            dimensions.model = Some("gpt-4.1".to_owned());
            dimensions.billing_mode = Some("canonical_account".to_owned());
        });

        let response = relay_chat_completion_from_store(
            &store,
            &secret_manager,
            "tenant-1",
            "project-1",
            &chat_request("gpt-4.1"),
        )
        .await
        .unwrap()
        .expect("gateway response");

        assert_eq!(response["id"], "chatcmpl_secondary");
    })
    .await;

    let output = telemetry.render_prometheus();
    assert!(output.contains(
        "sdkwork_provider_execution_total{service=\"gateway\",route=\"/v1/chat/completions\",tenant=\"tenant-1\",model=\"gpt-4.1\",provider=\"provider-primary\",billing_mode=\"canonical_account\",retry_outcome=\"will_failover\",failover_outcome=\"activated\",result=\"retryable_failure\"} 1"
    ));
    assert!(output.contains(
        "sdkwork_provider_execution_total{service=\"gateway\",route=\"/v1/chat/completions\",tenant=\"tenant-1\",model=\"gpt-4.1\",provider=\"provider-secondary\",billing_mode=\"canonical_account\",retry_outcome=\"none\",failover_outcome=\"activated\",result=\"succeeded\"} 1"
    ));
    assert!(output.contains(
        "sdkwork_commercial_events_total{service=\"gateway\",event_kind=\"failover_activation\",route=\"/v1/chat/completions\",tenant=\"tenant-1\",provider=\"provider-primary\",model=\"gpt-4.1\",payment_outcome=\"none\",result=\"activated\"} 1"
    ));
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

async fn spawn_success_upstream(response_id: &str) -> UpstreamServer {
    let state = UpstreamState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    let app = Router::new()
        .route("/v1/chat/completions", post(success_chat_handler))
        .with_state((state.clone(), response_id.to_owned()));

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    UpstreamServer { address, state }
}

async fn spawn_failing_upstream() -> UpstreamServer {
    let state = UpstreamState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    let app = Router::new()
        .route("/v1/chat/completions", post(failing_chat_handler))
        .with_state(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    UpstreamServer { address, state }
}

async fn success_chat_handler(
    State((state, response_id)): State<(UpstreamState, String)>,
    headers: axum::http::HeaderMap,
) -> Json<Value> {
    *state.request_count.lock().unwrap() += 1;
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    Json(json!({
        "id": response_id,
        "object": "chat.completion",
        "model": "gpt-4.1",
        "choices": []
    }))
}

async fn failing_chat_handler(
    State(state): State<UpstreamState>,
    headers: axum::http::HeaderMap,
) -> (axum::http::StatusCode, Json<Value>) {
    *state.request_count.lock().unwrap() += 1;
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    (
        axum::http::StatusCode::BAD_GATEWAY,
        Json(json!({
            "error": {
                "message": "temporary upstream outage"
            }
        })),
    )
}
