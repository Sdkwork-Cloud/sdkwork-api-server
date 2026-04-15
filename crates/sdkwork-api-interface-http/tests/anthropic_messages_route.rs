#![allow(clippy::too_many_arguments)]

use axum::body::Body;
use axum::extract::State;
use axum::http::{HeaderMap, Request, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use sdkwork_api_app_credential::{
    persist_credential_with_secret_and_manager, CredentialSecretManager,
};
use sdkwork_api_domain_catalog::{Channel, ModelCatalogEntry, ProxyProvider};
use sdkwork_api_ext_provider_native_mock::FIXTURE_EXTENSION_ID;
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use serde_json::Value;
use serial_test::serial;
use sqlx::SqlitePool;
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

mod support;

#[derive(Clone, Default)]
struct UpstreamCaptureState {
    authorization: Arc<Mutex<Option<String>>>,
    x_api_key: Arc<Mutex<Option<String>>>,
    anthropic_version: Arc<Mutex<Option<String>>>,
    anthropic_beta: Arc<Mutex<Option<String>>>,
    body: Arc<Mutex<Option<Value>>>,
}

#[serial]
#[tokio::test]
async fn stateless_anthropic_messages_route_translates_to_chat_completions() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/chat/completions", post(upstream_chat_handler))
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let app = sdkwork_api_interface_http::gateway_router_with_stateless_config(
        sdkwork_api_interface_http::StatelessGatewayConfig::default().with_upstream(
            sdkwork_api_interface_http::StatelessGatewayUpstream::from_adapter_kind(
                "openai",
                format!("http://{address}"),
                "sk-stateless-openai",
            ),
        ),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("anthropic-version", "2023-06-01")
                .header("anthropic-beta", "tools-2024-04-04")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "model": "gpt-4.1",
                        "system": "Follow the repo rules.",
                        "max_tokens": 512,
                        "messages": [
                            {
                                "role": "user",
                                "content": [
                                    { "type": "text", "text": "hello from claude" }
                                ]
                            }
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["type"], "message");
    assert_eq!(json["role"], "assistant");
    assert_eq!(json["model"], "gpt-4.1");
    assert_eq!(json["content"][0]["type"], "text");
    assert_eq!(json["content"][0]["text"], "Hello from upstream");
    assert_eq!(json["stop_reason"], "end_turn");
    assert_eq!(json["usage"]["input_tokens"], 11);
    assert_eq!(json["usage"]["output_tokens"], 7);

    let upstream_body = upstream_state.body.lock().unwrap().clone().unwrap();
    assert_eq!(upstream_body["model"], "gpt-4.1");
    assert_eq!(upstream_body["messages"][0]["role"], "system");
    assert_eq!(
        upstream_body["messages"][0]["content"],
        "Follow the repo rules."
    );
    assert_eq!(upstream_body["messages"][1]["role"], "user");
    assert_eq!(upstream_body["messages"][1]["content"], "hello from claude");
    assert_eq!(upstream_body["max_tokens"], 512);
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-stateless-openai")
    );
    assert_eq!(
        upstream_state.anthropic_version.lock().unwrap().as_deref(),
        Some("2023-06-01")
    );
    assert_eq!(
        upstream_state.anthropic_beta.lock().unwrap().as_deref(),
        Some("tools-2024-04-04")
    );
}

#[serial]
#[tokio::test]
async fn stateless_anthropic_messages_route_returns_invalid_request_for_missing_model() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "model": "",
                        "messages": [
                            {
                                "role": "user",
                                "content": "route by anthropic"
                            }
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = read_json(response).await;
    assert_eq!(json["type"], "error");
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(
        json["error"]["message"],
        "Chat completion model is required."
    );
}

#[serial]
#[tokio::test]
async fn stateful_anthropic_messages_route_accepts_x_api_key_and_records_usage() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/chat/completions", post(upstream_chat_handler))
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, "tenant-1", "project-1").await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(&pool, admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    create_openai_provider(&admin_app, &admin_token, &address.to_string()).await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("x-api-key", api_key.clone())
                .header("anthropic-version", "2023-06-01")
                .header("anthropic-beta", "tools-2024-04-04")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "model": "gpt-4.1",
                        "max_tokens": 256,
                        "messages": [
                            {
                                "role": "user",
                                "content": "route by anthropic"
                            }
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["content"][0]["text"], "Hello from upstream");
    assert_eq!(
        upstream_state.anthropic_version.lock().unwrap().as_deref(),
        Some("2023-06-01")
    );
    assert_eq!(
        upstream_state.anthropic_beta.lock().unwrap().as_deref(),
        Some("tools-2024-04-04")
    );

    support::assert_single_usage_record_and_decision_log(
        admin_app,
        &admin_token,
        "gpt-4.1",
        "provider-openai-official",
        "gpt-4.1",
    )
    .await;
}

#[serial]
#[tokio::test]
async fn stateless_anthropic_messages_route_passthroughs_native_anthropic_protocol() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/health", get(upstream_health_handler))
        .route("/v1/messages", post(upstream_anthropic_messages_handler))
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let app = sdkwork_api_interface_http::gateway_router_with_stateless_config(
        sdkwork_api_interface_http::StatelessGatewayConfig::default().with_upstream(
            sdkwork_api_interface_http::StatelessGatewayUpstream::from_adapter_kind(
                "anthropic",
                format!("http://{address}"),
                "sk-stateless-anthropic",
            ),
        ),
    );

    let payload = serde_json::json!({
        "model": "claude-3-7-sonnet",
        "system": "Follow the repo rules.",
        "max_tokens": 512,
        "messages": [
            {
                "role": "user",
                "content": [
                    { "type": "text", "text": "hello from claude" }
                ]
            }
        ]
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("anthropic-version", "2023-06-01")
                .header("anthropic-beta", "tools-2024-04-04")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["type"], "message");
    assert_eq!(json["role"], "assistant");
    assert_eq!(json["model"], "claude-3-7-sonnet");
    assert_eq!(json["content"][0]["text"], "Hello from anthropic upstream");
    assert_eq!(
        upstream_state.x_api_key.lock().unwrap().as_deref(),
        Some("sk-stateless-anthropic")
    );
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        None
    );
    assert_eq!(
        upstream_state.anthropic_version.lock().unwrap().as_deref(),
        Some("2023-06-01")
    );
    assert_eq!(
        upstream_state.anthropic_beta.lock().unwrap().as_deref(),
        Some("tools-2024-04-04")
    );
    assert_eq!(
        upstream_state.body.lock().unwrap().clone().unwrap(),
        payload
    );
}

#[serial(extension_env)]
#[tokio::test]
async fn stateless_anthropic_messages_route_prefers_native_dynamic_raw_plugin_for_explicit_custom_runtime(
) {
    let _fixture =
        support::prepare_native_dynamic_mock_package("anthropic-native-dynamic-stateless");

    let app = sdkwork_api_interface_http::gateway_router_with_stateless_config(
        sdkwork_api_interface_http::StatelessGatewayConfig::default().with_upstream(
            sdkwork_api_interface_http::StatelessGatewayUpstream::new_with_protocol_kind(
                FIXTURE_EXTENSION_ID,
                "custom",
                "https://native-dynamic.invalid/v1",
                "sk-native",
            ),
        ),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "model": "claude-3-7-sonnet",
                        "max_tokens": 256,
                        "messages": [
                            {
                                "role": "user",
                                "content": "route by anthropic plugin"
                            }
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["type"], "message");
    assert_eq!(json["role"], "assistant");
    assert_eq!(json["model"], "claude-3-7-sonnet");
    assert_eq!(
        json["content"][0]["text"],
        "Hello from native dynamic anthropic"
    );
    assert_eq!(json["stop_reason"], "end_turn");
    assert_eq!(json["usage"]["input_tokens"], 13);
    assert_eq!(json["usage"]["output_tokens"], 8);
}

#[serial]
#[tokio::test]
async fn stateful_anthropic_messages_route_passthroughs_native_anthropic_protocol_and_records_usage(
) {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/health", get(upstream_health_handler))
        .route("/v1/messages", post(upstream_anthropic_messages_handler))
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });
    support::wait_for_http_health(&format!("http://{address}")).await;

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, "tenant-1", "project-1").await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(&pool, admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    create_anthropic_provider(&admin_app, &admin_token, &address.to_string()).await;

    let payload = serde_json::json!({
        "model": "claude-3-7-sonnet",
        "max_tokens": 256,
        "messages": [
            {
                "role": "user",
                "content": "route by anthropic"
            }
        ]
    });

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("x-api-key", api_key.clone())
                .header("anthropic-version", "2023-06-01")
                .header("anthropic-beta", "tools-2024-04-04")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["content"][0]["text"], "Hello from anthropic upstream");
    assert_eq!(
        upstream_state.x_api_key.lock().unwrap().as_deref(),
        Some("sk-upstream-anthropic")
    );
    assert_eq!(
        upstream_state.body.lock().unwrap().clone().unwrap(),
        payload
    );

    support::assert_single_usage_record_and_decision_log(
        admin_app,
        &admin_token,
        "claude-3-7-sonnet",
        "provider-anthropic-official",
        "claude-3-7-sonnet",
    )
    .await;
}

#[serial]
#[tokio::test]
async fn stateful_anthropic_messages_route_derives_protocol_kind_for_legacy_blank_protocol_provider(
) {
    let tenant_id = "tenant-anthropic-legacy-blank-protocol";
    let project_id = "project-anthropic-legacy-blank-protocol";
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/health", get(upstream_health_handler))
        .route("/v1/messages", post(upstream_anthropic_messages_handler))
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });
    support::wait_for_http_health(&format!("http://{address}")).await;

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let store = SqliteAdminStore::new(pool.clone());
    let secret_manager = CredentialSecretManager::database_encrypted("local-dev-master-key");
    seed_legacy_blank_protocol_anthropic_provider(
        &store,
        &secret_manager,
        tenant_id,
        &address.to_string(),
    )
    .await;

    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(&pool, admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let payload = serde_json::json!({
        "model": "claude-3-7-sonnet",
        "max_tokens": 256,
        "messages": [
            {
                "role": "user",
                "content": "route by anthropic legacy blank protocol"
            }
        ]
    });

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
                .header("anthropic-beta", "tools-2024-04-04")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["content"][0]["text"], "Hello from anthropic upstream");
    assert_eq!(
        upstream_state.x_api_key.lock().unwrap().as_deref(),
        Some("sk-upstream-anthropic")
    );
    assert_eq!(
        upstream_state.body.lock().unwrap().clone().unwrap(),
        payload
    );

    support::assert_single_usage_record_and_decision_log(
        admin_app,
        &admin_token,
        "claude-3-7-sonnet",
        "provider-anthropic-legacy-blank-protocol",
        "claude-3-7-sonnet",
    )
    .await;
}

#[serial]
#[tokio::test]
async fn stateful_anthropic_messages_route_fails_over_before_execution_when_primary_lacks_tenant_credential(
) {
    let tenant_id = "tenant-anthropic-preflight-failover";
    let project_id = "project-anthropic-preflight-failover";
    let primary_state = UpstreamCaptureState::default();
    let backup_state = UpstreamCaptureState::default();

    let primary_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let primary_address = primary_listener.local_addr().unwrap();
    let primary_upstream = Router::new()
        .route("/health", get(upstream_health_handler))
        .route("/v1/messages", post(upstream_anthropic_messages_handler))
        .with_state(primary_state.clone());
    tokio::spawn(async move {
        axum::serve(primary_listener, primary_upstream)
            .await
            .unwrap();
    });

    let backup_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let backup_address = backup_listener.local_addr().unwrap();
    let backup_upstream = Router::new()
        .route("/health", get(upstream_health_handler))
        .route("/v1/messages", post(upstream_anthropic_messages_handler))
        .with_state(backup_state.clone());
    tokio::spawn(async move {
        axum::serve(backup_listener, backup_upstream).await.unwrap();
    });

    support::wait_for_http_health(&format!("http://{primary_address}")).await;
    support::wait_for_http_health(&format!("http://{backup_address}")).await;

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(&pool, admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let channel = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/channels")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"id\":\"anthropic\",\"name\":\"Anthropic\"}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(channel.status(), StatusCode::CREATED);

    create_stateful_anthropic_provider_for_failover(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-anthropic-primary-no-credential",
        &primary_address.to_string(),
        "Anthropic Primary Missing Credential",
        None,
        None,
    )
    .await;
    create_stateful_anthropic_provider_for_failover(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-anthropic-backup-with-credential",
        &backup_address.to_string(),
        "Anthropic Backup With Credential",
        Some("cred-anthropic-backup"),
        Some("sk-upstream-anthropic-backup"),
    )
    .await;

    let policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": "route-anthropic-preflight-failover",
                        "capability": "chat_completion",
                        "model_pattern": "claude-3-7-sonnet",
                        "enabled": true,
                        "priority": 300,
                        "execution_failover_enabled": true,
                        "ordered_provider_ids": [
                            "provider-anthropic-primary-no-credential",
                            "provider-anthropic-backup-with-credential"
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(policy.status(), StatusCode::CREATED);

    let payload = serde_json::json!({
        "model": "claude-3-7-sonnet",
        "max_tokens": 256,
        "messages": [
            {
                "role": "user",
                "content": "route by anthropic preflight failover"
            }
        ]
    });

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
                .header("anthropic-beta", "tools-2024-04-04")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["type"], "message");
    assert_eq!(json["content"][0]["text"], "Hello from anthropic upstream");
    assert!(primary_state.x_api_key.lock().unwrap().is_none());
    assert!(primary_state.body.lock().unwrap().is_none());
    assert_eq!(
        backup_state.x_api_key.lock().unwrap().as_deref(),
        Some("sk-upstream-anthropic-backup")
    );
    assert_eq!(backup_state.body.lock().unwrap().clone().unwrap(), payload);

    let usage = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/usage/records")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(usage.status(), StatusCode::OK);
    let usage_json = read_json(usage).await;
    assert_eq!(usage_json.as_array().unwrap().len(), 1);
    assert_eq!(usage_json[0]["model"], "claude-3-7-sonnet");
    assert_eq!(
        usage_json[0]["provider"],
        "provider-anthropic-backup-with-credential"
    );

    let logs = admin_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/routing/decision-logs")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(logs.status(), StatusCode::OK);
    let logs_json = read_json(logs).await;
    assert_eq!(logs_json.as_array().unwrap().len(), 1);
    assert_eq!(logs_json[0]["route_key"], "claude-3-7-sonnet");
    assert_eq!(
        logs_json[0]["selected_provider_id"],
        "provider-anthropic-backup-with-credential"
    );
    assert_eq!(
        logs_json[0]["fallback_reason"],
        "gateway_execution_failover"
    );
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_anthropic_messages_route_prefers_native_dynamic_raw_plugin_for_explicit_custom_runtime_and_records_usage(
) {
    let fixture = support::prepare_native_dynamic_mock_package("anthropic-native-dynamic-stateful");

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, "tenant-1", "project-1").await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(&pool, admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    create_native_dynamic_anthropic_provider(&admin_app, &admin_token, &fixture).await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("x-api-key", api_key.clone())
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "model": "claude-3-7-sonnet",
                        "max_tokens": 256,
                        "messages": [
                            {
                                "role": "user",
                                "content": "route by anthropic plugin"
                            }
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(
        json["content"][0]["text"],
        "Hello from native dynamic anthropic"
    );
    assert_eq!(json["usage"]["input_tokens"], 13);
    assert_eq!(json["usage"]["output_tokens"], 8);

    support::assert_single_usage_record_and_decision_log(
        admin_app,
        &admin_token,
        "claude-3-7-sonnet",
        "provider-native-anthropic-mock",
        "claude-3-7-sonnet",
    )
    .await;
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_anthropic_messages_route_uses_connector_runtime_translated_fallback_and_records_usage(
) {
    let fixture = support::prepare_connector_mock_package(
        "anthropic-connector-stateful",
        "sdkwork.provider.custom-openai.connector.anthropic",
    );

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
    support::wait_for_http_health(&format!("http://{address}")).await;

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, "tenant-1", "project-1").await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(&pool, admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    create_connector_anthropic_provider(&admin_app, &admin_token, &fixture, &address.to_string())
        .await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "model": "claude-3-7-sonnet",
                        "max_tokens": 256,
                        "messages": [
                            {
                                "role": "user",
                                "content": "route by anthropic connector"
                            }
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = read_text(response).await;
    assert_eq!(status, StatusCode::OK, "unexpected body: {body}");
    let json: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["content"][0]["text"], "Hello from upstream");
    assert_eq!(json["usage"]["input_tokens"], 11);
    assert_eq!(json["usage"]["output_tokens"], 7);
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );

    support::assert_single_usage_record_and_decision_log(
        admin_app,
        &admin_token,
        "claude-3-7-sonnet",
        "provider-connector-anthropic-mock",
        "claude-3-7-sonnet",
    )
    .await;
}

#[serial]
#[tokio::test]
async fn stateful_anthropic_messages_route_fails_closed_for_missing_explicit_native_dynamic_binding_and_persists_decision_log(
) {
    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, "tenant-1", "project-1").await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(&pool, admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    create_broken_native_dynamic_anthropic_provider(&admin_app, &admin_token).await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("x-api-key", api_key)
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "model": "claude-3-7-sonnet",
                        "max_tokens": 256,
                        "messages": [
                            {
                                "role": "user",
                                "content": "route by missing anthropic plugin"
                            }
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    let json = read_json(response).await;
    assert_eq!(json["type"], "error");
    assert_eq!(json["error"]["type"], "api_error");
    assert_eq!(
        json["error"]["message"],
        "failed to relay upstream anthropic message"
    );

    support::assert_no_usage_records(admin_app.clone(), &admin_token).await;
    support::assert_single_decision_log(
        admin_app,
        &admin_token,
        "claude-3-7-sonnet",
        "provider-broken-native-anthropic-mock",
    )
    .await;
}

#[serial]
#[tokio::test]
async fn stateful_anthropic_messages_route_returns_invalid_request_for_missing_model_without_usage()
{
    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(
        &pool,
        "tenant-anthropic-invalid",
        "project-anthropic-invalid",
    )
    .await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(&pool, admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("x-api-key", api_key)
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "model": "",
                        "messages": [
                            {
                                "role": "user",
                                "content": "route by anthropic"
                            }
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = read_json(response).await;
    assert_eq!(json["type"], "error");
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(
        json["error"]["message"],
        "Chat completion model is required."
    );

    support::assert_no_usage_records(admin_app, &admin_token).await;
}

#[serial]
#[tokio::test]
async fn stateless_anthropic_messages_stream_route_returns_anthropic_sse_events() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/chat/completions", post(upstream_chat_stream_handler))
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let app = sdkwork_api_interface_http::gateway_router_with_stateless_config(
        sdkwork_api_interface_http::StatelessGatewayConfig::default().with_upstream(
            sdkwork_api_interface_http::StatelessGatewayUpstream::from_adapter_kind(
                "openai",
                format!("http://{address}"),
                "sk-stateless-openai",
            ),
        ),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "model": "gpt-4.1",
                        "stream": true,
                        "max_tokens": 256,
                        "messages": [
                            {
                                "role": "user",
                                "content": "stream to claude"
                            }
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("text/event-stream")
    );

    let body = read_text(response).await;
    assert!(body.contains("event: message_start"));
    assert!(body.contains("event: content_block_delta"));
    assert!(body.contains("Hello"));
    assert!(body.contains("event: message_stop"));
}

#[serial(extension_env)]
#[tokio::test]
async fn stateless_anthropic_messages_stream_route_prefers_native_dynamic_raw_plugin_for_explicit_custom_runtime(
) {
    let _fixture =
        support::prepare_native_dynamic_mock_package("anthropic-native-dynamic-stream-stateless");

    let app = sdkwork_api_interface_http::gateway_router_with_stateless_config(
        sdkwork_api_interface_http::StatelessGatewayConfig::default().with_upstream(
            sdkwork_api_interface_http::StatelessGatewayUpstream::new_with_protocol_kind(
                FIXTURE_EXTENSION_ID,
                "custom",
                "https://native-dynamic.invalid/v1",
                "sk-native",
            ),
        ),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "model": "claude-3-7-sonnet",
                        "stream": true,
                        "max_tokens": 256,
                        "messages": [
                            {
                                "role": "user",
                                "content": "stream by anthropic plugin"
                            }
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("text/event-stream")
    );

    let body = read_text(response).await;
    assert!(body.contains("event: message_start"));
    assert!(body.contains("Hello from native dynamic anthropic"));
    assert!(body.contains("event: message_stop"));
}

#[serial]
#[tokio::test]
async fn stateless_anthropic_messages_stream_route_returns_invalid_request_for_missing_model() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "model": "",
                        "stream": true,
                        "messages": [
                            {
                                "role": "user",
                                "content": "stream to claude"
                            }
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = read_json(response).await;
    assert_eq!(json["type"], "error");
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(
        json["error"]["message"],
        "Chat completion model is required."
    );
}

#[serial]
#[tokio::test]
async fn stateful_anthropic_messages_stream_route_returns_invalid_request_for_missing_model_without_usage(
) {
    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(
        &pool,
        "tenant-anthropic-stream-invalid",
        "project-anthropic-stream-invalid",
    )
    .await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(&pool, admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("x-api-key", api_key)
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "model": "",
                        "stream": true,
                        "messages": [
                            {
                                "role": "user",
                                "content": "stream to claude"
                            }
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = read_json(response).await;
    assert_eq!(json["type"], "error");
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(
        json["error"]["message"],
        "Chat completion model is required."
    );

    support::assert_no_usage_records(admin_app, &admin_token).await;
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_anthropic_messages_stream_route_prefers_native_dynamic_raw_plugin_for_explicit_custom_runtime_and_records_usage(
) {
    let fixture =
        support::prepare_native_dynamic_mock_package("anthropic-native-dynamic-stream-stateful");

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, "tenant-1", "project-1").await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(&pool, admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    create_native_dynamic_anthropic_provider(&admin_app, &admin_token, &fixture).await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages")
                .header("x-api-key", api_key.clone())
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "model": "claude-3-7-sonnet",
                        "stream": true,
                        "max_tokens": 256,
                        "messages": [
                            {
                                "role": "user",
                                "content": "stream by anthropic plugin"
                            }
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("text/event-stream")
    );

    let body = read_text(response).await;
    assert!(body.contains("event: message_start"));
    assert!(body.contains("Hello from native dynamic anthropic"));
    assert!(body.contains("event: message_stop"));

    support::assert_single_usage_record_and_decision_log(
        admin_app,
        &admin_token,
        "claude-3-7-sonnet",
        "provider-native-anthropic-mock",
        "claude-3-7-sonnet",
    )
    .await;
}

#[serial]
#[tokio::test]
async fn anthropic_count_tokens_route_returns_invalid_request_without_upstream_provider() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages/count_tokens")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "model": "gpt-4.1",
                        "messages": [
                            {
                                "role": "user",
                                "content": "count my claude tokens"
                            }
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = read_json(response).await;
    assert_eq!(json["type"], "error");
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(
        json["error"]["message"],
        "Response input token counting is not supported in local fallback."
    );
}

#[serial(extension_env)]
#[tokio::test]
async fn stateless_anthropic_count_tokens_route_prefers_native_dynamic_raw_plugin_for_explicit_custom_runtime(
) {
    let _fixture =
        support::prepare_native_dynamic_mock_package("anthropic-native-dynamic-count-stateless");

    let app = sdkwork_api_interface_http::gateway_router_with_stateless_config(
        sdkwork_api_interface_http::StatelessGatewayConfig::default().with_upstream(
            sdkwork_api_interface_http::StatelessGatewayUpstream::new_with_protocol_kind(
                FIXTURE_EXTENSION_ID,
                "custom",
                "https://native-dynamic.invalid/v1",
                "sk-native",
            ),
        ),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages/count_tokens")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "model": "claude-3-7-sonnet",
                        "messages": [
                            {
                                "role": "user",
                                "content": "count by anthropic plugin"
                            }
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["input_tokens"], 42);
}

#[serial]
#[tokio::test]
async fn anthropic_count_tokens_route_returns_invalid_request_for_missing_model() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages/count_tokens")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "model": "",
                        "messages": [
                            {
                                "role": "user",
                                "content": "count my claude tokens"
                            }
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = read_json(response).await;
    assert_eq!(json["type"], "error");
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["message"], "Response model is required.");
}

#[serial]
#[tokio::test]
async fn stateful_anthropic_count_tokens_route_returns_invalid_request_for_missing_model_without_usage(
) {
    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(
        &pool,
        "tenant-anthropic-count-invalid",
        "project-anthropic-count-invalid",
    )
    .await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(&pool, admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/messages/count_tokens")
                .header("x-api-key", api_key)
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "model": "",
                        "messages": [
                            {
                                "role": "user",
                                "content": "count my claude tokens"
                            }
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = read_json(response).await;
    assert_eq!(json["type"], "error");
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["message"], "Response model is required.");

    support::assert_no_usage_records(admin_app, &admin_token).await;
}

async fn read_json(response: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn read_text(response: axum::response::Response) -> String {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}

async fn memory_pool() -> SqlitePool {
    sdkwork_api_storage_sqlite::run_migrations("sqlite::memory:")
        .await
        .unwrap()
}

async fn create_openai_provider(admin_app: &Router, admin_token: &str, address: &str) {
    let channel = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/channels")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"id\":\"openai\",\"name\":\"OpenAI\"}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(channel.status(), StatusCode::CREATED);

    let provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-openai-official\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{address}\",\"display_name\":\"OpenAI Official\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider.status(), StatusCode::CREATED);

    let credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"tenant_id\":\"tenant-1\",\"provider_id\":\"provider-openai-official\",\"key_reference\":\"cred-openai\",\"secret_value\":\"sk-upstream-openai\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(credential.status(), StatusCode::CREATED);

    let model = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-openai-official\",\"capabilities\":[\"chat_completions\"],\"streaming\":true}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(model.status(), StatusCode::CREATED);
}

async fn create_anthropic_provider(admin_app: &Router, admin_token: &str, address: &str) {
    let channel = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/channels")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"id\":\"anthropic\",\"name\":\"Anthropic\"}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(channel.status(), StatusCode::CREATED);

    let provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-anthropic-official\",\"channel_id\":\"anthropic\",\"adapter_kind\":\"anthropic\",\"base_url\":\"http://{address}\",\"display_name\":\"Anthropic Official\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider.status(), StatusCode::CREATED);

    let credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"tenant_id\":\"tenant-1\",\"provider_id\":\"provider-anthropic-official\",\"key_reference\":\"cred-anthropic\",\"secret_value\":\"sk-upstream-anthropic\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(credential.status(), StatusCode::CREATED);

    let model = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"claude-3-7-sonnet\",\"provider_id\":\"provider-anthropic-official\",\"capabilities\":[\"chat_completions\"],\"streaming\":true}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(model.status(), StatusCode::CREATED);
}

async fn seed_legacy_blank_protocol_anthropic_provider(
    store: &SqliteAdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    address: &str,
) {
    store
        .insert_channel(&Channel::new("anthropic", "Anthropic"))
        .await
        .unwrap();

    let mut provider = ProxyProvider::new(
        "provider-anthropic-legacy-blank-protocol",
        "anthropic",
        "anthropic",
        format!("http://{address}"),
        "Anthropic Legacy Blank Protocol",
    );
    provider.protocol_kind.clear();
    store.insert_provider(&provider).await.unwrap();

    persist_credential_with_secret_and_manager(
        store,
        secret_manager,
        tenant_id,
        &provider.id,
        "cred-anthropic-legacy-blank-protocol",
        "sk-upstream-anthropic",
    )
    .await
    .unwrap();

    store
        .insert_model(&ModelCatalogEntry::new("claude-3-7-sonnet", &provider.id))
        .await
        .unwrap();
}

async fn create_stateful_anthropic_provider_for_failover(
    admin_app: &Router,
    admin_token: &str,
    tenant_id: &str,
    provider_id: &str,
    address: &str,
    display_name: &str,
    credential_ref: Option<&str>,
    secret_value: Option<&str>,
) {
    let provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "id": provider_id,
                        "channel_id": "anthropic",
                        "adapter_kind": "openai",
                        "protocol_kind": "anthropic",
                        "base_url": format!("http://{address}"),
                        "display_name": display_name
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider.status(), StatusCode::CREATED);

    if let (Some(credential_ref), Some(secret_value)) = (credential_ref, secret_value) {
        let credential = admin_app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/admin/credentials")
                    .header("authorization", format!("Bearer {admin_token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({
                            "tenant_id": tenant_id,
                            "provider_id": provider_id,
                            "key_reference": credential_ref,
                            "secret_value": secret_value
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(credential.status(), StatusCode::CREATED);
    }

    let model = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "external_name": "claude-3-7-sonnet",
                        "provider_id": provider_id,
                        "capabilities": ["chat_completions"],
                        "streaming": true
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(model.status(), StatusCode::CREATED);
}

async fn create_native_dynamic_anthropic_provider(
    admin_app: &Router,
    admin_token: &str,
    fixture: &support::NativeDynamicMockPackage,
) {
    let channel = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/channels")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"id\":\"anthropic\",\"name\":\"Anthropic\"}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(channel.status(), StatusCode::CREATED);

    let provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-native-anthropic-mock\",\"channel_id\":\"anthropic\",\"adapter_kind\":\"native-dynamic\",\"protocol_kind\":\"custom\",\"base_url\":\"https://native-dynamic.invalid/v1\",\"display_name\":\"Native Anthropic Mock\",\"extension_id\":\"{FIXTURE_EXTENSION_ID}\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider.status(), StatusCode::CREATED);

    let credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"tenant_id\":\"tenant-1\",\"provider_id\":\"provider-native-anthropic-mock\",\"key_reference\":\"cred-native-anthropic\",\"secret_value\":\"sk-native\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(credential.status(), StatusCode::CREATED);

    let model = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"claude-3-7-sonnet\",\"provider_id\":\"provider-native-anthropic-mock\",\"capabilities\":[\"chat_completions\"],\"streaming\":true}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(model.status(), StatusCode::CREATED);

    let installation = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/installations")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "installation_id": "native-anthropic-installation",
                        "extension_id": FIXTURE_EXTENSION_ID,
                        "runtime": "native_dynamic",
                        "enabled": true,
                        "entrypoint": fixture.library_path.to_string_lossy(),
                        "config": {}
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(installation.status(), StatusCode::CREATED);

    let instance = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/instances")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "instance_id": "provider-native-anthropic-mock",
                        "installation_id": "native-anthropic-installation",
                        "extension_id": FIXTURE_EXTENSION_ID,
                        "enabled": true,
                        "base_url": "https://native-dynamic.invalid/v1",
                        "credential_ref": "cred-native-anthropic",
                        "config": {}
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(instance.status(), StatusCode::CREATED);
}

async fn create_connector_anthropic_provider(
    admin_app: &Router,
    admin_token: &str,
    fixture: &support::ConnectorMockPackage,
    address: &str,
) {
    let channel = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/channels")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"id\":\"anthropic\",\"name\":\"Anthropic\"}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(channel.status(), StatusCode::CREATED);

    let provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-connector-anthropic-mock\",\"channel_id\":\"anthropic\",\"adapter_kind\":\"custom-openai\",\"protocol_kind\":\"custom\",\"base_url\":\"http://{address}\",\"display_name\":\"Connector Anthropic Mock\",\"extension_id\":\"{}\"}}",
                    fixture.extension_id
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider.status(), StatusCode::CREATED);

    let credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"tenant_id\":\"tenant-1\",\"provider_id\":\"provider-connector-anthropic-mock\",\"key_reference\":\"cred-connector-anthropic\",\"secret_value\":\"sk-upstream-openai\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(credential.status(), StatusCode::CREATED);

    let model = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"claude-3-7-sonnet\",\"provider_id\":\"provider-connector-anthropic-mock\",\"capabilities\":[\"chat_completions\"],\"streaming\":true}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(model.status(), StatusCode::CREATED);

    let installation = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/installations")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "installation_id": "connector-anthropic-installation",
                        "extension_id": fixture.extension_id,
                        "runtime": "connector",
                        "enabled": true,
                        "entrypoint": "bin/sdkwork-provider-custom-openai",
                        "config": {
                            "health_path": "/health",
                            "startup_timeout_ms": 1000,
                            "startup_poll_interval_ms": 25
                        }
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(installation.status(), StatusCode::CREATED);

    let instance = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/instances")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "instance_id": "provider-connector-anthropic-mock",
                        "installation_id": "connector-anthropic-installation",
                        "extension_id": fixture.extension_id,
                        "enabled": true,
                        "base_url": format!("http://{address}"),
                        "credential_ref": "cred-connector-anthropic",
                        "config": {
                            "health_path": "/health"
                        }
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(instance.status(), StatusCode::CREATED);
}

async fn create_broken_native_dynamic_anthropic_provider(admin_app: &Router, admin_token: &str) {
    const BROKEN_EXTENSION_ID: &str = "sdkwork.provider.native.mock.broken.anthropic";

    let channel = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/channels")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"id\":\"anthropic\",\"name\":\"Anthropic\"}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(channel.status(), StatusCode::CREATED);

    let provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-broken-native-anthropic-mock\",\"channel_id\":\"anthropic\",\"adapter_kind\":\"native-dynamic\",\"protocol_kind\":\"custom\",\"base_url\":\"https://native-dynamic.invalid/v1\",\"display_name\":\"Broken Native Anthropic Mock\",\"extension_id\":\"{BROKEN_EXTENSION_ID}\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider.status(), StatusCode::CREATED);

    let credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"tenant_id\":\"tenant-1\",\"provider_id\":\"provider-broken-native-anthropic-mock\",\"key_reference\":\"cred-broken-native-anthropic\",\"secret_value\":\"sk-native\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(credential.status(), StatusCode::CREATED);

    let model = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"claude-3-7-sonnet\",\"provider_id\":\"provider-broken-native-anthropic-mock\",\"capabilities\":[\"chat_completions\"],\"streaming\":true}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(model.status(), StatusCode::CREATED);

    let installation = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/installations")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "installation_id": "broken-native-anthropic-installation",
                        "extension_id": BROKEN_EXTENSION_ID,
                        "runtime": "native_dynamic",
                        "enabled": true,
                        "entrypoint": "missing/native-dynamic-provider",
                        "config": {}
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(installation.status(), StatusCode::CREATED);

    let instance = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/instances")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "instance_id": "provider-broken-native-anthropic-mock",
                        "installation_id": "broken-native-anthropic-installation",
                        "extension_id": BROKEN_EXTENSION_ID,
                        "enabled": true,
                        "base_url": "https://native-dynamic.invalid/v1",
                        "credential_ref": "cred-broken-native-anthropic",
                        "config": {}
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(instance.status(), StatusCode::CREATED);
}

async fn upstream_chat_handler(
    State(state): State<UpstreamCaptureState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    capture_headers(&state, &headers);
    *state.body.lock().unwrap() = Some(body);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"chatcmpl_upstream",
            "object":"chat.completion",
            "model":"gpt-4.1",
            "choices":[{
                "index":0,
                "message":{
                    "role":"assistant",
                    "content":"Hello from upstream"
                },
                "finish_reason":"stop"
            }],
            "usage":{
                "prompt_tokens":11,
                "completion_tokens":7,
                "total_tokens":18
            }
        })),
    )
}

async fn upstream_health_handler() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({ "status": "ok" })))
}

async fn upstream_chat_stream_handler(
    State(state): State<UpstreamCaptureState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> axum::response::Response {
    capture_headers(&state, &headers);
    *state.body.lock().unwrap() = Some(body);

    (
        [(axum::http::header::CONTENT_TYPE, "text/event-stream")],
        concat!(
            "data: {\"id\":\"chatcmpl_stream\",\"object\":\"chat.completion.chunk\",\"model\":\"gpt-4.1\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Hello\"},\"finish_reason\":null}]}\n\n",
            "data: {\"id\":\"chatcmpl_stream\",\"object\":\"chat.completion.chunk\",\"model\":\"gpt-4.1\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\" world\"},\"finish_reason\":\"stop\"}]}\n\n",
            "data: [DONE]\n\n"
        ),
    )
        .into_response()
}

async fn upstream_anthropic_messages_handler(
    State(state): State<UpstreamCaptureState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    capture_headers(&state, &headers);
    *state.body.lock().unwrap() = Some(body);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"msg_upstream",
            "type":"message",
            "role":"assistant",
            "model":"claude-3-7-sonnet",
            "content":[
                {
                    "type":"text",
                    "text":"Hello from anthropic upstream"
                }
            ],
            "stop_reason":"end_turn",
            "stop_sequence":null,
            "usage":{
                "input_tokens":11,
                "output_tokens":7
            }
        })),
    )
}

fn capture_headers(state: &UpstreamCaptureState, headers: &HeaderMap) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    *state.x_api_key.lock().unwrap() = headers
        .get("x-api-key")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    *state.anthropic_version.lock().unwrap() = headers
        .get("anthropic-version")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    *state.anthropic_beta.lock().unwrap() = headers
        .get("anthropic-beta")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
}
