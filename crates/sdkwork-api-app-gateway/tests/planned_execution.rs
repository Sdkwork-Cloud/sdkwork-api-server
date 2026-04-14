use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use sdkwork_api_app_credential::{
    persist_credential_with_secret_and_manager, CredentialSecretManager,
};
use sdkwork_api_app_gateway::{
    planned_execution_provider_context_for_route_without_log,
    planned_execution_provider_context_for_route_without_log_with_selection_seed,
    relay_chat_completion_from_planned_execution_context_with_options,
};
use sdkwork_api_app_routing::persist_routing_policy;
use sdkwork_api_contract_openai::chat_completions::{
    ChatMessageInput, CreateChatCompletionRequest,
};
use sdkwork_api_domain_catalog::{Channel, ModelCatalogEntry, ProxyProvider};
use sdkwork_api_domain_routing::{RoutingPolicy, RoutingStrategy};
use sdkwork_api_extension_core::{ExtensionInstallation, ExtensionInstance, ExtensionRuntime};
use sdkwork_api_provider_core::ProviderRequestOptions;
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};
use serde_json::{json, Value};
use serial_test::serial;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct UpstreamState {
    response_id: String,
    authorization: Arc<Mutex<Option<String>>>,
}

#[derive(Clone)]
struct UpstreamServer {
    address: String,
    state: UpstreamState,
}

#[tokio::test]
#[serial]
async fn planned_chat_execution_reuses_selected_provider_and_persists_one_decision_log() {
    let light = spawn_upstream("chatcmpl_light").await;
    let heavy = spawn_upstream("chatcmpl_heavy").await;

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
                "provider-light",
                "openai",
                "openai",
                "http://127.0.0.1:1",
                "Light Provider",
            )
            .with_extension_id("sdkwork.provider.openai.official"),
        )
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                "provider-heavy",
                "openai",
                "openai",
                "http://127.0.0.1:1",
                "Heavy Provider",
            )
            .with_extension_id("sdkwork.provider.openai.official"),
        )
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new("gpt-4.1", "provider-light"))
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new("gpt-4.1", "provider-heavy"))
        .await
        .unwrap();
    persist_credential_with_secret_and_manager(
        &store,
        &secret_manager,
        "tenant-1",
        "provider-light",
        "cred-light",
        "sk-light",
    )
    .await
    .unwrap();
    persist_credential_with_secret_and_manager(
        &store,
        &secret_manager,
        "tenant-1",
        "provider-heavy",
        "cred-heavy",
        "sk-heavy",
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
                "provider-light",
                "openai-builtin",
                "sdkwork.provider.openai.official",
            )
            .with_enabled(true)
            .with_base_url(light.address.clone())
            .with_config(json!({
                "routing": {
                    "weight": 10
                }
            })),
        )
        .await
        .unwrap();
    store
        .insert_extension_instance(
            &ExtensionInstance::new(
                "provider-heavy",
                "openai-builtin",
                "sdkwork.provider.openai.official",
            )
            .with_enabled(true)
            .with_base_url(heavy.address.clone())
            .with_config(json!({
                "routing": {
                    "weight": 90
                }
            })),
        )
        .await
        .unwrap();

    let policy = RoutingPolicy::new("policy-weighted", "chat_completion", "gpt-4.1")
        .with_priority(100)
        .with_strategy(RoutingStrategy::WeightedRandom)
        .with_ordered_provider_ids(vec![
            "provider-light".to_owned(),
            "provider-heavy".to_owned(),
        ]);
    persist_routing_policy(&store, &policy).await.unwrap();

    let planned = planned_execution_provider_context_for_route_without_log_with_selection_seed(
        &store,
        &secret_manager,
        "tenant-1",
        "project-1",
        "chat_completion",
        "gpt-4.1",
        Some(15),
    )
    .await
    .unwrap()
    .expect("planned provider context");

    assert_eq!(planned.decision.selected_provider_id, "provider-heavy");
    assert_eq!(planned.usage_context.provider_id, "provider-heavy");
    assert!(store.list_routing_decision_logs().await.unwrap().is_empty());

    let response = relay_chat_completion_from_planned_execution_context_with_options(
        &store,
        &secret_manager,
        "tenant-1",
        "project-1",
        &chat_request("gpt-4.1"),
        &ProviderRequestOptions::default(),
        &planned,
    )
    .await
    .unwrap()
    .response
    .expect("planned response");

    assert_eq!(response["id"], "chatcmpl_heavy");
    assert!(light.state.authorization.lock().unwrap().is_none());
    assert_eq!(
        heavy.state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-heavy")
    );

    let logs = store.list_routing_decision_logs().await.unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].selected_provider_id, "provider-heavy");
    assert_eq!(logs[0].selection_seed, Some(15));
}

#[tokio::test]
#[serial]
async fn planned_chat_execution_fails_over_when_selected_provider_lacks_tenant_credential() {
    let primary = spawn_upstream("chatcmpl_primary_missing_credential").await;
    let backup = spawn_upstream("chatcmpl_backup_with_credential").await;

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
                "provider-backup",
                "openai",
                "openai",
                "http://127.0.0.1:1",
                "Backup Provider",
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

    let policy = RoutingPolicy::new("policy-priority", "chat_completion", "gpt-4.1")
        .with_priority(100)
        .with_ordered_provider_ids(vec![
            "provider-primary".to_owned(),
            "provider-backup".to_owned(),
        ]);
    persist_routing_policy(&store, &policy).await.unwrap();

    let planned = planned_execution_provider_context_for_route_without_log(
        &store,
        &secret_manager,
        "tenant-1",
        "project-1",
        "chat_completion",
        "gpt-4.1",
    )
    .await
    .unwrap()
    .expect("planned provider context");

    assert_eq!(planned.provider.id, "provider-backup");
    assert_eq!(planned.decision.selected_provider_id, "provider-backup");
    assert_eq!(planned.usage_context.provider_id, "provider-backup");
    assert_eq!(
        planned.decision.fallback_reason.as_deref(),
        Some("gateway_execution_failover")
    );

    let response = relay_chat_completion_from_planned_execution_context_with_options(
        &store,
        &secret_manager,
        "tenant-1",
        "project-1",
        &chat_request("gpt-4.1"),
        &ProviderRequestOptions::default(),
        &planned,
    )
    .await
    .unwrap()
    .response
    .expect("backup response");

    assert_eq!(response["id"], "chatcmpl_backup_with_credential");
    assert!(primary.state.authorization.lock().unwrap().is_none());
    assert_eq!(
        backup.state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-backup")
    );

    let logs = store.list_routing_decision_logs().await.unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].selected_provider_id, "provider-backup");
    assert_eq!(
        logs[0].fallback_reason.as_deref(),
        Some("gateway_execution_failover")
    );
}

#[tokio::test]
#[serial]
async fn planned_chat_execution_fails_over_when_selected_provider_is_missing() {
    let backup = spawn_upstream("chatcmpl_backup_when_primary_missing").await;

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
                "provider-backup",
                "openai",
                "openai",
                "http://127.0.0.1:1",
                "Backup Provider",
            )
            .with_extension_id("sdkwork.provider.openai.official"),
        )
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

    let policy = RoutingPolicy::new(
        "policy-priority-missing-provider",
        "chat_completion",
        "gpt-4.1",
    )
    .with_priority(100)
    .with_ordered_provider_ids(vec![
        "provider-primary-missing".to_owned(),
        "provider-backup".to_owned(),
    ]);
    persist_routing_policy(&store, &policy).await.unwrap();

    let planned = planned_execution_provider_context_for_route_without_log(
        &store,
        &secret_manager,
        "tenant-1",
        "project-1",
        "chat_completion",
        "gpt-4.1",
    )
    .await
    .unwrap()
    .expect("planned provider context");

    assert_eq!(planned.provider.id, "provider-backup");
    assert_eq!(planned.decision.selected_provider_id, "provider-backup");
    assert_eq!(planned.usage_context.provider_id, "provider-backup");
    assert_eq!(
        planned.decision.fallback_reason.as_deref(),
        Some("policy_candidate_unavailable")
    );

    let response = relay_chat_completion_from_planned_execution_context_with_options(
        &store,
        &secret_manager,
        "tenant-1",
        "project-1",
        &chat_request("gpt-4.1"),
        &ProviderRequestOptions::default(),
        &planned,
    )
    .await
    .unwrap()
    .response
    .expect("backup response");

    assert_eq!(response["id"], "chatcmpl_backup_when_primary_missing");
    assert_eq!(
        backup.state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-backup")
    );

    let logs = store.list_routing_decision_logs().await.unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].selected_provider_id, "provider-backup");
    assert_eq!(
        logs[0].fallback_reason.as_deref(),
        Some("policy_candidate_unavailable")
    );
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

async fn spawn_upstream(response_id: &str) -> UpstreamServer {
    let state = UpstreamState {
        response_id: response_id.to_owned(),
        authorization: Arc::new(Mutex::new(None)),
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

async fn upstream_chat_handler(
    State(state): State<UpstreamState>,
    headers: axum::http::HeaderMap,
) -> Json<Value> {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    Json(json!({
        "id": state.response_id,
        "object": "chat.completion",
        "model": "gpt-4.1",
        "choices": []
    }))
}
