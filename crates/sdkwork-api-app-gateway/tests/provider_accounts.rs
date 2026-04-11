use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use sdkwork_api_app_credential::{
    persist_credential_with_secret_and_manager, CredentialSecretManager,
};
use sdkwork_api_app_gateway::{
    planned_execution_provider_context_for_route_without_log,
    relay_chat_completion_from_planned_execution_context_with_options,
    relay_chat_completion_from_store_with_execution_context, with_request_routing_region,
    relay_count_response_input_tokens_from_planned_execution_context,
    relay_response_from_store_with_execution_context,
};
use sdkwork_api_app_routing::persist_routing_policy;
use sdkwork_api_contract_openai::chat_completions::{
    ChatMessageInput, CreateChatCompletionRequest,
};
use sdkwork_api_contract_openai::responses::{
    CountResponseInputTokensRequest, CreateResponseRequest,
};
use sdkwork_api_domain_catalog::{Channel, ProviderAccountRecord, ProxyProvider};
use sdkwork_api_domain_routing::RoutingPolicy;
use sdkwork_api_extension_core::{ExtensionInstallation, ExtensionInstance, ExtensionRuntime};
use sdkwork_api_provider_core::ProviderRequestOptions;
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};
use serde_json::{json, Value};
use serial_test::serial;
use std::sync::{Arc, Mutex};

#[tokio::test]
async fn planned_execution_uses_region_preferred_provider_account_binding() {
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
                "https://api.openai.com/v1",
                "OpenAI Official",
            )
            .with_extension_id("sdkwork.provider.openai.official"),
        )
        .await
        .unwrap();
    store
        .insert_model(&sdkwork_api_domain_catalog::ModelCatalogEntry::new(
            "gpt-4.1",
            "provider-openai-official",
        ))
        .await
        .unwrap();

    persist_credential_with_secret_and_manager(
        &store,
        &secret_manager,
        "tenant-1",
        "provider-openai-official",
        "cred-default",
        "sk-default",
    )
    .await
    .unwrap();
    persist_credential_with_secret_and_manager(
        &store,
        &secret_manager,
        "tenant-1",
        "provider-openai-official",
        "cred-us-east",
        "sk-us-east",
    )
    .await
    .unwrap();

    store
        .insert_extension_installation(
            &ExtensionInstallation::new(
                "install-openai-builtin",
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
                "instance-openai-default",
                "install-openai-builtin",
                "sdkwork.provider.openai.official",
            )
            .with_enabled(true)
            .with_base_url("https://default.example.com/v1")
            .with_credential_ref("cred-default")
            .with_config(json!({})),
        )
        .await
        .unwrap();
    store
        .insert_extension_instance(
            &ExtensionInstance::new(
                "instance-openai-us-east",
                "install-openai-builtin",
                "sdkwork.provider.openai.official",
            )
            .with_enabled(true)
            .with_base_url("https://us-east.example.com/v1")
            .with_credential_ref("cred-us-east")
            .with_config(json!({})),
        )
        .await
        .unwrap();

    store
        .upsert_provider_account(
            &ProviderAccountRecord::new(
                "acct-openai-default",
                "provider-openai-official",
                "OpenAI Default",
                "api_key",
                "instance-openai-default",
            )
            .with_priority(50)
            .with_weight(10)
            .with_enabled(true),
        )
        .await
        .unwrap();
    store
        .upsert_provider_account(
            &ProviderAccountRecord::new(
                "acct-openai-us-east",
                "provider-openai-official",
                "OpenAI US East",
                "api_key",
                "instance-openai-us-east",
            )
            .with_region("us-east")
            .with_priority(100)
            .with_weight(20)
            .with_enabled(true),
        )
        .await
        .unwrap();

    persist_routing_policy(
        &store,
        &RoutingPolicy::new("policy-gpt-4-1", "chat_completion", "gpt-4.1")
            .with_priority(100)
            .with_default_provider_id("provider-openai-official"),
    )
    .await
    .unwrap();

    let planned = with_request_routing_region(
        Some("us-east".to_owned()),
        planned_execution_provider_context_for_route_without_log(
            &store,
            &secret_manager,
            "tenant-1",
            "project-1",
            "chat_completion",
            "gpt-4.1",
        ),
    )
    .await
    .unwrap()
    .unwrap();

    assert_eq!(planned.provider.id, "provider-openai-official");
    assert_eq!(planned.api_key, "sk-us-east");
    assert_eq!(
        planned.execution.provider_account_id.as_deref(),
        Some("acct-openai-us-east")
    );
    assert_eq!(
        planned.execution.execution_instance_id.as_deref(),
        Some("instance-openai-us-east")
    );
    assert_eq!(planned.execution.base_url, "https://us-east.example.com/v1");
}

#[tokio::test]
async fn planned_execution_skips_disabled_provider_account_binding() {
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
                "https://api.openai.com/v1",
                "OpenAI Official",
            )
            .with_extension_id("sdkwork.provider.openai.official"),
        )
        .await
        .unwrap();
    store
        .insert_model(&sdkwork_api_domain_catalog::ModelCatalogEntry::new(
            "gpt-4.1",
            "provider-openai-official",
        ))
        .await
        .unwrap();

    persist_credential_with_secret_and_manager(
        &store,
        &secret_manager,
        "tenant-1",
        "provider-openai-official",
        "cred-default",
        "sk-default",
    )
    .await
    .unwrap();
    persist_credential_with_secret_and_manager(
        &store,
        &secret_manager,
        "tenant-1",
        "provider-openai-official",
        "cred-us-east",
        "sk-us-east",
    )
    .await
    .unwrap();

    store
        .insert_extension_installation(
            &ExtensionInstallation::new(
                "install-openai-builtin",
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
                "instance-openai-default",
                "install-openai-builtin",
                "sdkwork.provider.openai.official",
            )
            .with_enabled(true)
            .with_base_url("https://default.example.com/v1")
            .with_credential_ref("cred-default")
            .with_config(json!({})),
        )
        .await
        .unwrap();
    store
        .insert_extension_instance(
            &ExtensionInstance::new(
                "instance-openai-us-east",
                "install-openai-builtin",
                "sdkwork.provider.openai.official",
            )
            .with_enabled(true)
            .with_base_url("https://us-east.example.com/v1")
            .with_credential_ref("cred-us-east")
            .with_config(json!({})),
        )
        .await
        .unwrap();

    store
        .upsert_provider_account(
            &ProviderAccountRecord::new(
                "acct-openai-default",
                "provider-openai-official",
                "OpenAI Default",
                "api_key",
                "instance-openai-default",
            )
            .with_priority(50)
            .with_weight(10)
            .with_enabled(true),
        )
        .await
        .unwrap();
    store
        .upsert_provider_account(
            &ProviderAccountRecord::new(
                "acct-openai-us-east",
                "provider-openai-official",
                "OpenAI US East",
                "api_key",
                "instance-openai-us-east",
            )
            .with_region("us-east")
            .with_priority(100)
            .with_weight(20)
            .with_enabled(false),
        )
        .await
        .unwrap();

    persist_routing_policy(
        &store,
        &RoutingPolicy::new("policy-gpt-4-1", "chat_completion", "gpt-4.1")
            .with_priority(100)
            .with_default_provider_id("provider-openai-official"),
    )
    .await
    .unwrap();

    let planned = with_request_routing_region(
        Some("us-east".to_owned()),
        planned_execution_provider_context_for_route_without_log(
            &store,
            &secret_manager,
            "tenant-1",
            "project-1",
            "chat_completion",
            "gpt-4.1",
        ),
    )
    .await
    .unwrap()
    .unwrap();

    assert_eq!(
        planned.execution.provider_account_id.as_deref(),
        Some("acct-openai-default")
    );
    assert_eq!(
        planned.execution.execution_instance_id.as_deref(),
        Some("instance-openai-default")
    );
    assert_eq!(planned.api_key, "sk-default");
    assert_eq!(planned.execution.base_url, "https://default.example.com/v1");
}

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
async fn planned_chat_execution_uses_selected_provider_account_runtime_binding() {
    let default_upstream = spawn_upstream("chatcmpl_default_account").await;
    let us_east_upstream = spawn_upstream("chatcmpl_us_east_account").await;

    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let secret_manager = CredentialSecretManager::database_encrypted("local-dev-master-key");

    seed_provider_account_gateway_fixture(
        &store,
        &secret_manager,
        &default_upstream.address,
        &us_east_upstream.address,
    )
    .await;

    let planned = with_request_routing_region(
        Some("us-east".to_owned()),
        planned_execution_provider_context_for_route_without_log(
            &store,
            &secret_manager,
            "tenant-1",
            "project-planned-relay",
            "chat_completion",
            "gpt-4.1",
        ),
    )
    .await
    .unwrap()
    .unwrap();

    let response = relay_chat_completion_from_planned_execution_context_with_options(
        &store,
        &secret_manager,
        "tenant-1",
        "project-planned-relay",
        &chat_request("gpt-4.1"),
        &ProviderRequestOptions::default(),
        &planned,
    )
    .await
    .unwrap()
    .response
    .expect("planned relay response");

    assert_eq!(response["id"], "chatcmpl_us_east_account");
    assert!(default_upstream.state.authorization.lock().unwrap().is_none());
    assert_eq!(
        us_east_upstream
            .state
            .authorization
            .lock()
            .unwrap()
            .as_deref(),
        Some("Bearer sk-us-east")
    );
}

#[tokio::test]
#[serial]
async fn direct_chat_relay_uses_selected_provider_account_runtime_binding() {
    let default_upstream = spawn_upstream("chatcmpl_default_account").await;
    let us_east_upstream = spawn_upstream("chatcmpl_us_east_account").await;

    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let secret_manager = CredentialSecretManager::database_encrypted("local-dev-master-key");

    seed_provider_account_gateway_fixture(
        &store,
        &secret_manager,
        &default_upstream.address,
        &us_east_upstream.address,
    )
    .await;

    let result = with_request_routing_region(
        Some("us-east".to_owned()),
        relay_chat_completion_from_store_with_execution_context(
            &store,
            &secret_manager,
            "tenant-1",
            "project-direct-relay",
            &chat_request("gpt-4.1"),
            &ProviderRequestOptions::default(),
        ),
    )
    .await
    .unwrap();

    let response = result.response.expect("direct relay response");
    assert_eq!(response["id"], "chatcmpl_us_east_account");
    assert!(default_upstream.state.authorization.lock().unwrap().is_none());
    assert_eq!(
        us_east_upstream
            .state
            .authorization
            .lock()
            .unwrap()
            .as_deref(),
        Some("Bearer sk-us-east")
    );
}

#[tokio::test]
#[serial]
async fn direct_response_relay_uses_selected_provider_account_runtime_binding() {
    let default_upstream = spawn_upstream("resp_default_account").await;
    let us_east_upstream = spawn_upstream("resp_us_east_account").await;

    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let secret_manager = CredentialSecretManager::database_encrypted("local-dev-master-key");

    seed_provider_account_gateway_fixture(
        &store,
        &secret_manager,
        &default_upstream.address,
        &us_east_upstream.address,
    )
    .await;

    let result = with_request_routing_region(
        Some("us-east".to_owned()),
        relay_response_from_store_with_execution_context(
            &store,
            &secret_manager,
            "tenant-1",
            "project-response-relay",
            &response_request("gpt-4.1"),
        ),
    )
    .await
    .unwrap();

    let response = result.response.expect("direct response relay");
    assert_eq!(response["id"], "resp_us_east_account");
    assert!(default_upstream.state.authorization.lock().unwrap().is_none());
    assert_eq!(
        us_east_upstream
            .state
            .authorization
            .lock()
            .unwrap()
            .as_deref(),
        Some("Bearer sk-us-east")
    );
}

#[tokio::test]
#[serial]
async fn planned_response_input_tokens_uses_selected_provider_account_runtime_binding() {
    let default_upstream = spawn_upstream("resp_default_account").await;
    let us_east_upstream = spawn_upstream("resp_us_east_account").await;

    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    let secret_manager = CredentialSecretManager::database_encrypted("local-dev-master-key");

    seed_provider_account_gateway_fixture(
        &store,
        &secret_manager,
        &default_upstream.address,
        &us_east_upstream.address,
    )
    .await;

    let planned = with_request_routing_region(
        Some("us-east".to_owned()),
        planned_execution_provider_context_for_route_without_log(
            &store,
            &secret_manager,
            "tenant-1",
            "project-response-planned",
            "responses",
            "gpt-4.1",
        ),
    )
    .await
    .unwrap()
    .unwrap();

    let response = relay_count_response_input_tokens_from_planned_execution_context(
        &store,
        "tenant-1",
        "project-response-planned",
        &response_input_tokens_request("gpt-4.1"),
        &planned,
    )
    .await
    .unwrap()
    .expect("planned response input tokens");

    assert_eq!(response["object"], "response.input_tokens");
    assert_eq!(response["input_tokens"], 21);
    assert!(default_upstream.state.authorization.lock().unwrap().is_none());
    assert_eq!(
        us_east_upstream
            .state
            .authorization
            .lock()
            .unwrap()
            .as_deref(),
        Some("Bearer sk-us-east")
    );
}

async fn seed_provider_account_gateway_fixture(
    store: &SqliteAdminStore,
    secret_manager: &CredentialSecretManager,
    default_base_url: &str,
    us_east_base_url: &str,
) {
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
                "https://api.openai.com/v1",
                "OpenAI Official",
            )
            .with_extension_id("sdkwork.provider.openai.official"),
        )
        .await
        .unwrap();
    store
        .insert_model(&sdkwork_api_domain_catalog::ModelCatalogEntry::new(
            "gpt-4.1",
            "provider-openai-official",
        ))
        .await
        .unwrap();

    persist_credential_with_secret_and_manager(
        store,
        secret_manager,
        "tenant-1",
        "provider-openai-official",
        "cred-default",
        "sk-default",
    )
    .await
    .unwrap();
    persist_credential_with_secret_and_manager(
        store,
        secret_manager,
        "tenant-1",
        "provider-openai-official",
        "cred-us-east",
        "sk-us-east",
    )
    .await
    .unwrap();

    store
        .insert_extension_installation(
            &ExtensionInstallation::new(
                "install-openai-builtin",
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
                "install-openai-builtin",
                "sdkwork.provider.openai.official",
            )
            .with_enabled(true)
            .with_base_url(default_base_url.to_owned())
            .with_credential_ref("cred-default")
            .with_config(json!({})),
        )
        .await
        .unwrap();
    store
        .insert_extension_instance(
            &ExtensionInstance::new(
                "instance-openai-us-east",
                "install-openai-builtin",
                "sdkwork.provider.openai.official",
            )
            .with_enabled(true)
            .with_base_url(us_east_base_url.to_owned())
            .with_credential_ref("cred-us-east")
            .with_config(json!({})),
        )
        .await
        .unwrap();

    store
        .upsert_provider_account(
            &ProviderAccountRecord::new(
                "acct-openai-default",
                "provider-openai-official",
                "OpenAI Default",
                "api_key",
                "provider-openai-official",
            )
            .with_priority(50)
            .with_weight(10)
            .with_enabled(true),
        )
        .await
        .unwrap();
    store
        .upsert_provider_account(
            &ProviderAccountRecord::new(
                "acct-openai-us-east",
                "provider-openai-official",
                "OpenAI US East",
                "api_key",
                "instance-openai-us-east",
            )
            .with_region("us-east")
            .with_priority(100)
            .with_weight(20)
            .with_enabled(true),
        )
        .await
        .unwrap();

    persist_routing_policy(
        store,
        &RoutingPolicy::new("policy-gpt-4-1", "chat_completion", "gpt-4.1")
            .with_priority(100)
            .with_default_provider_id("provider-openai-official"),
    )
    .await
    .unwrap();
    persist_routing_policy(
        store,
        &RoutingPolicy::new("policy-responses-gpt-4-1", "responses", "gpt-4.1")
            .with_priority(100)
            .with_default_provider_id("provider-openai-official"),
    )
    .await
    .unwrap();
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

fn response_request(model: &str) -> CreateResponseRequest {
    CreateResponseRequest {
        model: model.to_owned(),
        input: json!([{
            "role": "user",
            "content": [{"type": "input_text", "text": "hello"}]
        }]),
        stream: None,
    }
}

fn response_input_tokens_request(model: &str) -> CountResponseInputTokensRequest {
    CountResponseInputTokensRequest {
        model: model.to_owned(),
        input: json!([{
            "role": "user",
            "content": [{"type": "input_text", "text": "hello"}]
        }]),
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
        .route("/v1/responses", post(upstream_responses_handler))
        .route(
            "/v1/responses/input_tokens",
            post(upstream_response_input_tokens_handler),
        )
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

async fn upstream_responses_handler(
    State(state): State<UpstreamState>,
    headers: axum::http::HeaderMap,
) -> Json<Value> {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    Json(json!({
        "id": state.response_id,
        "object": "response",
        "model": "gpt-4.1",
        "output": []
    }))
}

async fn upstream_response_input_tokens_handler(
    State(state): State<UpstreamState>,
    headers: axum::http::HeaderMap,
) -> Json<Value> {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    Json(json!({
        "object": "response.input_tokens",
        "input_tokens": 21
    }))
}
