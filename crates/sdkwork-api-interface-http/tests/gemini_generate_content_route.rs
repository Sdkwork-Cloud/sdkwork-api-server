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
    x_goog_api_key: Arc<Mutex<Option<String>>>,
    body: Arc<Mutex<Option<Value>>>,
}

#[serial]
#[tokio::test]
async fn stateless_gemini_generate_content_route_translates_to_chat_completions() {
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
                .uri("/v1beta/models/gemini-2.5-pro:generateContent")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "contents": [
                            {
                                "role": "user",
                                "parts": [
                                    { "text": "hello from gemini" }
                                ]
                            }
                        ],
                        "generationConfig": {
                            "temperature": 0.2,
                            "maxOutputTokens": 256
                        }
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["candidates"][0]["content"]["role"], "model");
    assert_eq!(
        json["candidates"][0]["content"]["parts"][0]["text"],
        "Hello from upstream"
    );
    assert_eq!(json["candidates"][0]["finishReason"], "STOP");
    assert_eq!(json["usageMetadata"]["promptTokenCount"], 9);
    assert_eq!(json["usageMetadata"]["candidatesTokenCount"], 5);
    assert_eq!(json["usageMetadata"]["totalTokenCount"], 14);

    let upstream_body = upstream_state.body.lock().unwrap().clone().unwrap();
    assert_eq!(upstream_body["model"], "gemini-2.5-pro");
    assert_eq!(upstream_body["messages"][0]["role"], "user");
    assert_eq!(upstream_body["messages"][0]["content"], "hello from gemini");
    assert_eq!(upstream_body["temperature"], 0.2);
    assert_eq!(upstream_body["max_tokens"], 256);
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-stateless-openai")
    );
}

#[serial]
#[tokio::test]
async fn stateless_gemini_generate_content_route_returns_invalid_request_for_missing_model() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1beta/models/:generateContent")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "contents": [
                            {
                                "role": "user",
                                "parts": [
                                    { "text": "hello from gemini" }
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

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = read_json(response).await;
    assert_eq!(json["error"]["code"], 400);
    assert_eq!(json["error"]["status"], "INVALID_ARGUMENT");
    assert_eq!(json["error"]["message"], "Chat completion model is required.");
}

#[serial]
#[tokio::test]
async fn stateful_gemini_generate_content_route_accepts_query_key_and_records_usage() {
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
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    create_openai_provider(&admin_app, &admin_token, &address.to_string()).await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/v1beta/models/gemini-2.5-pro:generateContent?key={api_key}"
                ))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "contents": [
                            {
                                "role": "user",
                                "parts": [
                                    { "text": "route by gemini" }
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
    assert_eq!(
        json["candidates"][0]["content"]["parts"][0]["text"],
        "Hello from upstream"
    );

    support::assert_single_usage_record_and_decision_log(
        admin_app,
        &admin_token,
        "gemini-2.5-pro",
        "provider-openai-official",
        "gemini-2.5-pro",
    )
    .await;
}

#[serial]
#[tokio::test]
async fn stateless_gemini_generate_content_route_passthroughs_native_gemini_protocol() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/health", get(upstream_health_handler))
        .route(
            "/v1beta/models/gemini-2.5-pro:generateContent",
            post(upstream_gemini_generate_content_handler),
        )
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let app = sdkwork_api_interface_http::gateway_router_with_stateless_config(
        sdkwork_api_interface_http::StatelessGatewayConfig::default().with_upstream(
            sdkwork_api_interface_http::StatelessGatewayUpstream::from_adapter_kind(
                "gemini",
                format!("http://{address}"),
                "sk-stateless-gemini",
            ),
        ),
    );

    let payload = serde_json::json!({
        "contents": [
            {
                "role": "user",
                "parts": [
                    { "text": "hello from gemini" }
                ]
            }
        ],
        "generationConfig": {
            "temperature": 0.2,
            "maxOutputTokens": 256
        }
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1beta/models/gemini-2.5-pro:generateContent")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["candidates"][0]["content"]["role"], "model");
    assert_eq!(
        json["candidates"][0]["content"]["parts"][0]["text"],
        "Hello from gemini upstream"
    );
    assert_eq!(
        upstream_state.x_goog_api_key.lock().unwrap().as_deref(),
        Some("sk-stateless-gemini")
    );
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        None
    );
    assert_eq!(upstream_state.body.lock().unwrap().clone().unwrap(), payload);
}

#[serial(extension_env)]
#[tokio::test]
async fn stateless_gemini_generate_content_route_prefers_native_dynamic_raw_plugin_for_explicit_custom_runtime(
) {
    let _fixture = support::prepare_native_dynamic_mock_package("gemini-native-dynamic-stateless");

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
                .uri("/v1beta/models/gemini-2.5-pro:generateContent")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "contents": [
                            {
                                "role": "user",
                                "parts": [
                                    { "text": "route by gemini plugin" }
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
    assert_eq!(json["candidates"][0]["content"]["role"], "model");
    assert_eq!(
        json["candidates"][0]["content"]["parts"][0]["text"],
        "Hello from native dynamic gemini"
    );
    assert_eq!(json["candidates"][0]["finishReason"], "STOP");
    assert_eq!(json["usageMetadata"]["promptTokenCount"], 12);
    assert_eq!(json["usageMetadata"]["candidatesTokenCount"], 6);
    assert_eq!(json["usageMetadata"]["totalTokenCount"], 18);
}

#[serial]
#[tokio::test]
async fn stateful_gemini_generate_content_route_passthroughs_native_gemini_protocol_and_records_usage(
) {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/health", get(upstream_health_handler))
        .route(
            "/v1beta/models/gemini-2.5-pro:generateContent",
            post(upstream_gemini_generate_content_handler),
        )
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });
    support::wait_for_http_health(&format!("http://{address}")).await;

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, "tenant-1", "project-1").await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    create_gemini_provider(&admin_app, &admin_token, &address.to_string()).await;

    let payload = serde_json::json!({
        "contents": [
            {
                "role": "user",
                "parts": [
                    { "text": "route by gemini" }
                ]
            }
        ]
    });

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/v1beta/models/gemini-2.5-pro:generateContent?key={api_key}"
                ))
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(
        json["candidates"][0]["content"]["parts"][0]["text"],
        "Hello from gemini upstream"
    );
    assert_eq!(
        upstream_state.x_goog_api_key.lock().unwrap().as_deref(),
        Some("sk-upstream-gemini")
    );
    assert_eq!(upstream_state.body.lock().unwrap().clone().unwrap(), payload);

    support::assert_single_usage_record_and_decision_log(
        admin_app,
        &admin_token,
        "gemini-2.5-pro",
        "provider-gemini-official",
        "gemini-2.5-pro",
    )
    .await;
}

#[serial]
#[tokio::test]
async fn stateful_gemini_generate_content_route_derives_protocol_kind_for_legacy_blank_protocol_provider(
) {
    let tenant_id = "tenant-gemini-legacy-blank-protocol";
    let project_id = "project-gemini-legacy-blank-protocol";
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/health", get(upstream_health_handler))
        .route(
            "/v1beta/models/gemini-2.5-pro:generateContent",
            post(upstream_gemini_generate_content_handler),
        )
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });
    support::wait_for_http_health(&format!("http://{address}")).await;

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let store = SqliteAdminStore::new(pool.clone());
    let secret_manager = CredentialSecretManager::database_encrypted("local-dev-master-key");
    seed_legacy_blank_protocol_gemini_provider(
        &store,
        &secret_manager,
        tenant_id,
        &address.to_string(),
    )
    .await;

    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let payload = serde_json::json!({
        "contents": [
            {
                "role": "user",
                "parts": [
                    { "text": "route by gemini legacy blank protocol" }
                ]
            }
        ]
    });

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/v1beta/models/gemini-2.5-pro:generateContent?key={api_key}"
                ))
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(
        json["candidates"][0]["content"]["parts"][0]["text"],
        "Hello from gemini upstream"
    );
    assert_eq!(
        upstream_state.x_goog_api_key.lock().unwrap().as_deref(),
        Some("sk-upstream-gemini")
    );
    assert_eq!(upstream_state.body.lock().unwrap().clone().unwrap(), payload);

    support::assert_single_usage_record_and_decision_log(
        admin_app,
        &admin_token,
        "gemini-2.5-pro",
        "provider-gemini-legacy-blank-protocol",
        "gemini-2.5-pro",
    )
    .await;
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_gemini_generate_content_route_prefers_native_dynamic_raw_plugin_for_explicit_custom_runtime_and_records_usage(
) {
    let fixture = support::prepare_native_dynamic_mock_package("gemini-native-dynamic-stateful");

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, "tenant-1", "project-1").await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    create_native_dynamic_gemini_provider(&admin_app, &admin_token, &fixture).await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/v1beta/models/gemini-2.5-pro:generateContent?key={api_key}"
                ))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "contents": [
                            {
                                "role": "user",
                                "parts": [
                                    { "text": "route by gemini plugin" }
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
    assert_eq!(
        json["candidates"][0]["content"]["parts"][0]["text"],
        "Hello from native dynamic gemini"
    );
    assert_eq!(json["usageMetadata"]["promptTokenCount"], 12);
    assert_eq!(json["usageMetadata"]["candidatesTokenCount"], 6);
    assert_eq!(json["usageMetadata"]["totalTokenCount"], 18);

    support::assert_single_usage_record_and_decision_log(
        admin_app,
        &admin_token,
        "gemini-2.5-pro",
        "provider-native-gemini-mock",
        "gemini-2.5-pro",
    )
    .await;
}

#[serial]
#[tokio::test]
async fn stateful_gemini_stream_generate_content_route_fails_closed_for_missing_explicit_native_dynamic_binding_and_persists_decision_log(
) {
    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, "tenant-1", "project-1").await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    create_broken_native_dynamic_gemini_provider(&admin_app, &admin_token).await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/v1beta/models/gemini-2.5-pro:streamGenerateContent?alt=sse&key={api_key}"
                ))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "contents": [
                            {
                                "role": "user",
                                "parts": [
                                    { "text": "stream by missing gemini plugin" }
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

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    let json = read_json(response).await;
    assert_eq!(json["error"]["code"], 502);
    assert_eq!(json["error"]["status"], "BAD_GATEWAY");
    assert_eq!(
        json["error"]["message"],
        "failed to relay upstream gemini streamGenerateContent request"
    );

    support::assert_no_usage_records(admin_app.clone(), &admin_token).await;
    support::assert_single_decision_log(
        admin_app,
        &admin_token,
        "gemini-2.5-pro",
        "provider-broken-native-gemini-mock",
    )
    .await;
}

#[serial]
#[tokio::test]
async fn stateful_gemini_generate_content_route_returns_invalid_request_for_missing_model_without_usage(
) {
    let pool = memory_pool().await;
    let api_key =
        support::issue_gateway_api_key(&pool, "tenant-gemini-invalid", "project-gemini-invalid")
            .await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/v1beta/models/:generateContent?key={api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "contents": [
                            {
                                "role": "user",
                                "parts": [
                                    { "text": "route by gemini" }
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

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = read_json(response).await;
    assert_eq!(json["error"]["code"], 400);
    assert_eq!(json["error"]["status"], "INVALID_ARGUMENT");
    assert_eq!(json["error"]["message"], "Chat completion model is required.");

    support::assert_no_usage_records(admin_app, &admin_token).await;
}

#[serial]
#[tokio::test]
async fn stateless_gemini_stream_generate_content_route_returns_gemini_sse_events() {
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
                .uri("/v1beta/models/gemini-2.5-pro:streamGenerateContent?alt=sse")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "contents": [
                            {
                                "role": "user",
                                "parts": [
                                    { "text": "stream to gemini" }
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
    assert_eq!(
        response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("text/event-stream")
    );

    let body = read_text(response).await;
    assert!(body.contains("data: "));
    assert!(body.contains("\"candidates\""));
    assert!(body.contains("Hello"));
    assert!(body.contains("\"finishReason\":\"STOP\""));
}

#[serial(extension_env)]
#[tokio::test]
async fn stateless_gemini_stream_generate_content_route_prefers_native_dynamic_raw_plugin_for_explicit_custom_runtime(
) {
    let _fixture = support::prepare_native_dynamic_mock_package("gemini-native-dynamic-stream-stateless");

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
                .uri("/v1beta/models/gemini-2.5-pro:streamGenerateContent?alt=sse")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "contents": [
                            {
                                "role": "user",
                                "parts": [
                                    { "text": "stream by gemini plugin" }
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
    assert_eq!(
        response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("text/event-stream")
    );

    let body = read_text(response).await;
    assert!(body.contains("data: "));
    assert!(body.contains("Hello from native dynamic gemini"));
    assert!(body.contains("\"finishReason\":\"STOP\""));
}

#[serial]
#[tokio::test]
async fn stateless_gemini_stream_generate_content_route_returns_invalid_request_for_missing_model(
) {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1beta/models/:streamGenerateContent?alt=sse")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "contents": [
                            {
                                "role": "user",
                                "parts": [
                                    { "text": "stream to gemini" }
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

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = read_json(response).await;
    assert_eq!(json["error"]["code"], 400);
    assert_eq!(json["error"]["status"], "INVALID_ARGUMENT");
    assert_eq!(json["error"]["message"], "Chat completion model is required.");
}

#[serial]
#[tokio::test]
async fn stateful_gemini_stream_generate_content_route_returns_invalid_request_for_missing_model_without_usage(
) {
    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(
        &pool,
        "tenant-gemini-stream-invalid",
        "project-gemini-stream-invalid",
    )
    .await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/v1beta/models/:streamGenerateContent?alt=sse&key={api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "contents": [
                            {
                                "role": "user",
                                "parts": [
                                    { "text": "stream to gemini" }
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

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = read_json(response).await;
    assert_eq!(json["error"]["code"], 400);
    assert_eq!(json["error"]["status"], "INVALID_ARGUMENT");
    assert_eq!(json["error"]["message"], "Chat completion model is required.");

    support::assert_no_usage_records(admin_app, &admin_token).await;
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_gemini_stream_generate_content_route_prefers_native_dynamic_raw_plugin_for_explicit_custom_runtime_and_records_usage(
) {
    let fixture = support::prepare_native_dynamic_mock_package("gemini-native-dynamic-stream-stateful");

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, "tenant-1", "project-1").await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    create_native_dynamic_gemini_provider(&admin_app, &admin_token, &fixture).await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/v1beta/models/gemini-2.5-pro:streamGenerateContent?alt=sse&key={api_key}"
                ))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "contents": [
                            {
                                "role": "user",
                                "parts": [
                                    { "text": "stream by gemini plugin" }
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
    assert_eq!(
        response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("text/event-stream")
    );

    let body = read_text(response).await;
    assert!(body.contains("data: "));
    assert!(body.contains("Hello from native dynamic gemini"));
    assert!(body.contains("\"finishReason\":\"STOP\""));

    support::assert_single_usage_record_and_decision_log(
        admin_app,
        &admin_token,
        "gemini-2.5-pro",
        "provider-native-gemini-mock",
        "gemini-2.5-pro",
    )
    .await;
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_gemini_stream_generate_content_route_uses_connector_runtime_translated_fallback_and_records_usage(
) {
    let fixture = support::prepare_connector_mock_package(
        "gemini-connector-stream-stateful",
        "sdkwork.provider.custom-openai.connector.gemini",
    );

    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/health", get(upstream_health_handler))
        .route("/v1/chat/completions", post(upstream_chat_stream_handler))
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });
    support::wait_for_http_health(&format!("http://{address}")).await;

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, "tenant-1", "project-1").await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    create_connector_gemini_provider(&admin_app, &admin_token, &fixture, &address.to_string())
        .await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/v1beta/models/gemini-2.5-pro:streamGenerateContent?alt=sse&key={api_key}"
                ))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "contents": [
                            {
                                "role": "user",
                                "parts": [
                                    { "text": "stream by gemini connector" }
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

    let status = response.status();
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    let body = read_text(response).await;
    assert_eq!(status, StatusCode::OK, "unexpected body: {body}");
    assert_eq!(
        content_type.as_deref(),
        Some("text/event-stream")
    );
    assert!(body.contains("Hello"));
    assert!(body.contains("\"finishReason\":\"STOP\""));
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );

    support::assert_single_usage_record_and_decision_log(
        admin_app,
        &admin_token,
        "gemini-2.5-pro",
        "provider-connector-gemini-mock",
        "gemini-2.5-pro",
    )
    .await;
}

#[serial]
#[tokio::test]
async fn gemini_count_tokens_route_returns_total_tokens() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1beta/models/gemini-2.5-pro:countTokens")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "contents": [
                            {
                                "role": "user",
                                "parts": [
                                    { "text": "count my gemini tokens" }
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
    assert_eq!(json["totalTokens"], 42);
}

#[serial(extension_env)]
#[tokio::test]
async fn stateless_gemini_count_tokens_route_prefers_native_dynamic_raw_plugin_for_explicit_custom_runtime(
) {
    let _fixture = support::prepare_native_dynamic_mock_package("gemini-native-dynamic-count-stateless");

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
                .uri("/v1beta/models/gemini-2.5-pro:countTokens")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "contents": [
                            {
                                "role": "user",
                                "parts": [
                                    { "text": "count by gemini plugin" }
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
    assert_eq!(json["totalTokens"], 42);
}

#[serial]
#[tokio::test]
async fn gemini_count_tokens_route_returns_invalid_request_for_missing_model() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1beta/models/:countTokens")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "contents": [
                            {
                                "role": "user",
                                "parts": [
                                    { "text": "count my gemini tokens" }
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

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = read_json(response).await;
    assert_eq!(json["error"]["code"], 400);
    assert_eq!(json["error"]["status"], "INVALID_ARGUMENT");
    assert_eq!(json["error"]["message"], "Response model is required.");
}

#[serial]
#[tokio::test]
async fn stateful_gemini_count_tokens_route_returns_invalid_request_for_missing_model_without_usage(
) {
    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(
        &pool,
        "tenant-gemini-count-invalid",
        "project-gemini-count-invalid",
    )
    .await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/v1beta/models/:countTokens?key={api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "contents": [
                            {
                                "role": "user",
                                "parts": [
                                    { "text": "count my gemini tokens" }
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

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = read_json(response).await;
    assert_eq!(json["error"]["code"], 400);
    assert_eq!(json["error"]["status"], "INVALID_ARGUMENT");
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
                    "{\"external_name\":\"gemini-2.5-pro\",\"provider_id\":\"provider-openai-official\",\"capabilities\":[\"chat_completions\"],\"streaming\":true}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(model.status(), StatusCode::CREATED);
}

async fn create_gemini_provider(admin_app: &Router, admin_token: &str, address: &str) {
    let channel = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/channels")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"id\":\"gemini\",\"name\":\"Gemini\"}"))
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
                    "{{\"id\":\"provider-gemini-official\",\"channel_id\":\"gemini\",\"adapter_kind\":\"gemini\",\"base_url\":\"http://{address}\",\"display_name\":\"Gemini Official\"}}"
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
                    "{\"tenant_id\":\"tenant-1\",\"provider_id\":\"provider-gemini-official\",\"key_reference\":\"cred-gemini\",\"secret_value\":\"sk-upstream-gemini\"}",
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
                    "{\"external_name\":\"gemini-2.5-pro\",\"provider_id\":\"provider-gemini-official\",\"capabilities\":[\"chat_completions\"],\"streaming\":true}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(model.status(), StatusCode::CREATED);
}

async fn seed_legacy_blank_protocol_gemini_provider(
    store: &SqliteAdminStore,
    secret_manager: &CredentialSecretManager,
    tenant_id: &str,
    address: &str,
) {
    store
        .insert_channel(&Channel::new("gemini", "Gemini"))
        .await
        .unwrap();

    let mut provider = ProxyProvider::new(
        "provider-gemini-legacy-blank-protocol",
        "gemini",
        "gemini",
        format!("http://{address}"),
        "Gemini Legacy Blank Protocol",
    );
    provider.protocol_kind.clear();
    store.insert_provider(&provider).await.unwrap();

    persist_credential_with_secret_and_manager(
        store,
        secret_manager,
        tenant_id,
        &provider.id,
        "cred-gemini-legacy-blank-protocol",
        "sk-upstream-gemini",
    )
    .await
    .unwrap();

    store
        .insert_model(&ModelCatalogEntry::new("gemini-2.5-pro", &provider.id))
        .await
        .unwrap();
}

async fn create_native_dynamic_gemini_provider(
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
                .body(Body::from("{\"id\":\"gemini\",\"name\":\"Gemini\"}"))
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
                    "{{\"id\":\"provider-native-gemini-mock\",\"channel_id\":\"gemini\",\"adapter_kind\":\"native-dynamic\",\"protocol_kind\":\"custom\",\"base_url\":\"https://native-dynamic.invalid/v1\",\"display_name\":\"Native Gemini Mock\",\"extension_id\":\"{FIXTURE_EXTENSION_ID}\"}}"
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
                    "{\"tenant_id\":\"tenant-1\",\"provider_id\":\"provider-native-gemini-mock\",\"key_reference\":\"cred-native-gemini\",\"secret_value\":\"sk-native\"}",
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
                    "{\"external_name\":\"gemini-2.5-pro\",\"provider_id\":\"provider-native-gemini-mock\",\"capabilities\":[\"chat_completions\"],\"streaming\":true}",
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
                        "installation_id": "native-gemini-installation",
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
                        "instance_id": "provider-native-gemini-mock",
                        "installation_id": "native-gemini-installation",
                        "extension_id": FIXTURE_EXTENSION_ID,
                        "enabled": true,
                        "base_url": "https://native-dynamic.invalid/v1",
                        "credential_ref": "cred-native-gemini",
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

async fn create_connector_gemini_provider(
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
                .body(Body::from("{\"id\":\"gemini\",\"name\":\"Gemini\"}"))
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
                    "{{\"id\":\"provider-connector-gemini-mock\",\"channel_id\":\"gemini\",\"adapter_kind\":\"custom-openai\",\"protocol_kind\":\"custom\",\"base_url\":\"http://{address}\",\"display_name\":\"Connector Gemini Mock\",\"extension_id\":\"{}\"}}",
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
                    "{\"tenant_id\":\"tenant-1\",\"provider_id\":\"provider-connector-gemini-mock\",\"key_reference\":\"cred-connector-gemini\",\"secret_value\":\"sk-upstream-openai\"}",
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
                    "{\"external_name\":\"gemini-2.5-pro\",\"provider_id\":\"provider-connector-gemini-mock\",\"capabilities\":[\"chat_completions\"],\"streaming\":true}",
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
                        "installation_id": "connector-gemini-installation",
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
                        "instance_id": "provider-connector-gemini-mock",
                        "installation_id": "connector-gemini-installation",
                        "extension_id": fixture.extension_id,
                        "enabled": true,
                        "base_url": format!("http://{address}"),
                        "credential_ref": "cred-connector-gemini",
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

async fn create_broken_native_dynamic_gemini_provider(admin_app: &Router, admin_token: &str) {
    const BROKEN_EXTENSION_ID: &str = "sdkwork.provider.native.mock.broken.gemini";

    let channel = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/channels")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"id\":\"gemini\",\"name\":\"Gemini\"}"))
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
                    "{{\"id\":\"provider-broken-native-gemini-mock\",\"channel_id\":\"gemini\",\"adapter_kind\":\"native-dynamic\",\"protocol_kind\":\"custom\",\"base_url\":\"https://native-dynamic.invalid/v1\",\"display_name\":\"Broken Native Gemini Mock\",\"extension_id\":\"{BROKEN_EXTENSION_ID}\"}}"
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
                    "{\"tenant_id\":\"tenant-1\",\"provider_id\":\"provider-broken-native-gemini-mock\",\"key_reference\":\"cred-broken-native-gemini\",\"secret_value\":\"sk-native\"}",
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
                    "{\"external_name\":\"gemini-2.5-pro\",\"provider_id\":\"provider-broken-native-gemini-mock\",\"capabilities\":[\"chat_completions\"],\"streaming\":true}",
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
                        "installation_id": "broken-native-gemini-installation",
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
                        "instance_id": "provider-broken-native-gemini-mock",
                        "installation_id": "broken-native-gemini-installation",
                        "extension_id": BROKEN_EXTENSION_ID,
                        "enabled": true,
                        "base_url": "https://native-dynamic.invalid/v1",
                        "credential_ref": "cred-broken-native-gemini",
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
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    *state.body.lock().unwrap() = Some(body);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"chatcmpl_upstream",
            "object":"chat.completion",
            "model":"gemini-2.5-pro",
            "choices":[{
                "index":0,
                "message":{
                    "role":"assistant",
                    "content":"Hello from upstream"
                },
                "finish_reason":"stop"
            }],
            "usage":{
                "prompt_tokens":9,
                "completion_tokens":5,
                "total_tokens":14
            }
        })),
    )
}

async fn upstream_gemini_generate_content_handler(
    State(state): State<UpstreamCaptureState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    capture_headers(&state, &headers);
    *state.body.lock().unwrap() = Some(body);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "candidates":[
                {
                    "content":{
                        "role":"model",
                        "parts":[
                            { "text":"Hello from gemini upstream" }
                        ]
                    },
                    "finishReason":"STOP"
                }
            ],
            "usageMetadata":{
                "promptTokenCount":9,
                "candidatesTokenCount":5,
                "totalTokenCount":14
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
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    *state.body.lock().unwrap() = Some(body);

    (
        [(axum::http::header::CONTENT_TYPE, "text/event-stream")],
        concat!(
            "data: {\"id\":\"chatcmpl_stream\",\"object\":\"chat.completion.chunk\",\"model\":\"gemini-2.5-pro\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Hello\"},\"finish_reason\":null}]}\n\n",
            "data: {\"id\":\"chatcmpl_stream\",\"object\":\"chat.completion.chunk\",\"model\":\"gemini-2.5-pro\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\" from gemini\"},\"finish_reason\":\"stop\"}]}\n\n",
            "data: [DONE]\n\n"
        ),
    )
        .into_response()
}

fn capture_headers(state: &UpstreamCaptureState, headers: &HeaderMap) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    *state.x_goog_api_key.lock().unwrap() = headers
        .get("x-goog-api-key")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
}
