use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::Value;
use serial_test::serial;
use sqlx::SqlitePool;
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

mod support;

#[serial(extension_env)]
#[tokio::test]
async fn files_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/files")
                .header(
                    "content-type",
                    "multipart/form-data; boundary=----sdkwork-boundary",
                )
                .body(Body::from(build_file_multipart_body(
                    "----sdkwork-boundary",
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[serial(extension_env)]
#[tokio::test]
async fn files_list_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/files")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[serial(extension_env)]
#[tokio::test]
async fn file_retrieve_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/files/file_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[serial(extension_env)]
#[tokio::test]
async fn file_delete_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/files/file_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[serial(extension_env)]
#[tokio::test]
async fn file_content_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/files/file_1/content")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[serial(extension_env)]
#[tokio::test]
async fn stateless_files_routes_relay_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/files",
            get(upstream_files_list_handler).post(upstream_files_handler),
        )
        .route(
            "/v1/files/file_1",
            get(upstream_file_retrieve_handler).delete(upstream_file_delete_handler),
        )
        .route(
            "/v1/files/file_1/content",
            get(upstream_file_content_handler),
        )
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let app = sdkwork_api_interface_http::gateway_router_with_stateless_config(
        sdkwork_api_interface_http::StatelessGatewayConfig::default().with_upstream(
            sdkwork_api_interface_http::StatelessGatewayUpstream::new(
                "openai",
                format!("http://{address}"),
                "sk-stateless-openai",
            ),
        ),
    );

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/files")
                .header(
                    "content-type",
                    "multipart/form-data; boundary=----sdkwork-boundary",
                )
                .body(Body::from(build_file_multipart_body(
                    "----sdkwork-boundary",
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "file_upstream");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-stateless-openai")
    );
    assert!(upstream_state
        .content_type
        .lock()
        .unwrap()
        .as_deref()
        .unwrap_or_default()
        .starts_with("multipart/form-data"));

    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/files")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_json = read_json(list_response).await;
    assert_eq!(list_json["object"], "list");

    let retrieve_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/files/file_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(retrieve_response.status(), StatusCode::OK);
    let retrieve_json = read_json(retrieve_response).await;
    assert_eq!(retrieve_json["id"], "file_1");

    let delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/files/file_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_response.status(), StatusCode::OK);
    let delete_json = read_json(delete_response).await;
    assert_eq!(delete_json["deleted"], true);

    let content_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/files/file_1/content")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(content_response.status(), StatusCode::OK);
    assert_eq!(
        content_response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("application/jsonl")
    );
    let content_bytes = axum::body::to_bytes(content_response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&content_bytes[..], b"{}");
}

#[serial(extension_env)]
#[tokio::test]
async fn stateless_file_content_route_returns_openai_error_envelope_on_upstream_failure() {
    let app = sdkwork_api_interface_http::gateway_router_with_stateless_config(
        sdkwork_api_interface_http::StatelessGatewayConfig::default().with_upstream(
            sdkwork_api_interface_http::StatelessGatewayUpstream::new(
                "openai",
                "http://127.0.0.1:1",
                "sk-stateless-openai",
            ),
        ),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/files/file_1/content")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    let json = read_json(response).await;
    assert_eq!(
        json["error"]["message"],
        "failed to relay upstream file content"
    );
    assert_eq!(json["error"]["type"], "server_error");
    assert_eq!(json["error"]["code"], "bad_gateway");
}

async fn read_json(response: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn memory_pool() -> SqlitePool {
    sdkwork_api_storage_sqlite::run_migrations("sqlite::memory:")
        .await
        .unwrap()
}

#[derive(Clone, Default)]
struct UpstreamCaptureState {
    authorization: Arc<Mutex<Option<String>>>,
    content_type: Arc<Mutex<Option<String>>>,
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_files_route_relays_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/files",
            get(upstream_files_list_handler).post(upstream_files_handler),
        )
        .route(
            "/v1/files/file_1",
            get(upstream_file_retrieve_handler).delete(upstream_file_delete_handler),
        )
        .route(
            "/v1/files/file_1/content",
            get(upstream_file_content_handler),
        )
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let api_key = support::issue_gateway_api_key(&pool, "tenant-1", "project-1").await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let _ = admin_app
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

    let response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/files")
                .header("authorization", format!("Bearer {api_key}"))
                .header(
                    "content-type",
                    "multipart/form-data; boundary=----sdkwork-boundary",
                )
                .body(Body::from(build_file_multipart_body(
                    "----sdkwork-boundary",
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "file_upstream");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
    assert!(upstream_state
        .content_type
        .lock()
        .unwrap()
        .as_deref()
        .unwrap_or_default()
        .starts_with("multipart/form-data"));

    let list_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/files")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_json = read_json(list_response).await;
    assert_eq!(list_json["object"], "list");

    let retrieve_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/files/file_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(retrieve_response.status(), StatusCode::OK);
    let retrieve_json = read_json(retrieve_response).await;
    assert_eq!(retrieve_json["id"], "file_1");

    let delete_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/files/file_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_response.status(), StatusCode::OK);
    let delete_json = read_json(delete_response).await;
    assert_eq!(delete_json["deleted"], true);

    let content_response = gateway_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/files/file_1/content")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(content_response.status(), StatusCode::OK);
    let content_type = content_response
        .headers()
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned();
    assert_eq!(content_type, "application/jsonl");
    let content_bytes = axum::body::to_bytes(content_response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&content_bytes[..], b"{}");
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_file_content_route_relays_to_native_dynamic_provider() {
    let fixture = support::prepare_native_dynamic_mock_package("native-dynamic-file-content");

    let pool = memory_pool().await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let api_key = support::issue_gateway_api_key(&pool, "tenant-1", "project-1").await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let _ = admin_app
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

    let provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"id\":\"provider-native-mock\",\"channel_id\":\"openai\",\"adapter_kind\":\"native-dynamic\",\"base_url\":\"https://native-dynamic.invalid/v1\",\"display_name\":\"Native Mock\",\"extension_id\":\"sdkwork.provider.native.mock\"}",
                ))
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
                    "{\"tenant_id\":\"tenant-1\",\"provider_id\":\"provider-native-mock\",\"key_reference\":\"cred-native-mock\",\"secret_value\":\"sk-native\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(credential.status(), StatusCode::CREATED);

    let routing_policy = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "policy_id": "route-files-to-native-mock",
                        "capability": "files",
                        "model_pattern": "*",
                        "enabled": true,
                        "priority": 100,
                        "ordered_provider_ids": ["provider-native-mock"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(routing_policy.status(), StatusCode::CREATED);

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
                        "installation_id": "native-mock-installation",
                        "extension_id": "sdkwork.provider.native.mock",
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
                        "instance_id": "provider-native-mock",
                        "installation_id": "native-mock-installation",
                        "extension_id": "sdkwork.provider.native.mock",
                        "enabled": true,
                        "base_url": "https://native-dynamic.invalid/v1",
                        "credential_ref": "cred-native-mock",
                        "config": {}
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(instance.status(), StatusCode::CREATED);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/files/file_1/content")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
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
        Some("application/jsonl")
    );
    let content_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&content_bytes[..], b"{\"source\":\"native_dynamic\"}\n");
}

fn build_file_multipart_body(boundary: &str) -> Vec<u8> {
    format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"purpose\"\r\n\r\nfine-tune\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"train.jsonl\"\r\nContent-Type: application/jsonl\r\n\r\n{{}}\r\n--{boundary}--\r\n"
    )
    .into_bytes()
}

async fn upstream_files_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    *state.content_type.lock().unwrap() = headers
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"file_upstream",
            "object":"file",
            "purpose":"fine-tune",
            "filename":"train.jsonl",
            "bytes":2,
            "status":"processed"
        })),
    )
}

async fn upstream_files_list_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "object":"list",
            "data":[{
                "id":"file_1",
                "object":"file",
                "purpose":"fine-tune",
                "filename":"train.jsonl",
                "bytes":2,
                "status":"processed"
            }]
        })),
    )
}

async fn upstream_file_retrieve_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"file_1",
            "object":"file",
            "purpose":"fine-tune",
            "filename":"train.jsonl",
            "bytes":2,
            "status":"processed"
        })),
    )
}

async fn upstream_file_delete_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id":"file_1",
            "object":"file",
            "deleted":true
        })),
    )
}

async fn upstream_file_content_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> (
    [(axum::http::header::HeaderName, &'static str); 1],
    &'static [u8],
) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    (
        [(axum::http::header::CONTENT_TYPE, "application/jsonl")],
        b"{}",
    )
}
