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
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};
use serde_json::{json, Value};
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
async fn relay_chat_completion_honors_routing_policy_provider_order() {
    let openai = spawn_upstream("chatcmpl_openai").await;
    let openrouter = spawn_upstream("chatcmpl_openrouter").await;

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
                "openai",
                "http://127.0.0.1:1",
                "OpenAI Official",
            )
            .with_extension_id("sdkwork.provider.openai.official"),
        )
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                "provider-openrouter",
                "openai",
                "openai",
                "http://127.0.0.1:1",
                "OpenRouter",
            )
            .with_extension_id("sdkwork.provider.openai.official"),
        )
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new(
            "gpt-4.1",
            "provider-openai-official",
        ))
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new("gpt-4.1", "provider-openrouter"))
        .await
        .unwrap();
    persist_credential_with_secret_and_manager(
        &store,
        &secret_manager,
        "tenant-1",
        "provider-openai-official",
        "cred-openai",
        "sk-openai",
    )
    .await
    .unwrap();
    persist_credential_with_secret_and_manager(
        &store,
        &secret_manager,
        "tenant-1",
        "provider-openrouter",
        "cred-openrouter",
        "sk-openrouter",
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
            .with_base_url(openai.address.clone())
            .with_config(json!({})),
        )
        .await
        .unwrap();
    store
        .insert_extension_instance(
            &ExtensionInstance::new(
                "provider-openrouter",
                "openai-builtin",
                "sdkwork.provider.openai.official",
            )
            .with_enabled(true)
            .with_base_url(openrouter.address.clone())
            .with_config(json!({})),
        )
        .await
        .unwrap();

    let policy = RoutingPolicy::new("policy-gpt-4-1", "chat_completion", "gpt-4.1")
        .with_priority(100)
        .with_ordered_provider_ids(vec![
            "provider-openrouter".to_owned(),
            "provider-openai-official".to_owned(),
        ]);
    persist_routing_policy(&store, &policy).await.unwrap();

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

    assert_eq!(response["id"], "chatcmpl_openrouter");
    assert!(openai.state.authorization.lock().unwrap().is_none());
    assert_eq!(
        openrouter.state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-openrouter")
    );
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
