use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::Value;
use sqlx::SqlitePool;
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

mod support;

#[tokio::test]
async fn vector_store_file_batches_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/vector_stores/vs_1/file_batches")
                .header("content-type", "application/json")
                .body(Body::from("{\"file_ids\":[\"file_1\"]}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn vector_store_file_batch_retrieve_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/vector_stores/vs_1/file_batches/vsfb_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn vector_store_file_batch_cancel_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/vector_stores/vs_1/file_batches/vsfb_1/cancel")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn vector_store_file_batch_files_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/vector_stores/vs_1/file_batches/vsfb_1/files")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn stateless_vector_store_file_batches_route_relays_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/vector_stores/vs_1/file_batches",
            axum::routing::post(upstream_vector_store_file_batches_handler),
        )
        .route(
            "/v1/vector_stores/vs_1/file_batches/vsfb_1",
            get(upstream_vector_store_file_batch_retrieve_handler),
        )
        .route(
            "/v1/vector_stores/vs_1/file_batches/vsfb_1/cancel",
            axum::routing::post(upstream_vector_store_file_batch_cancel_handler),
        )
        .route(
            "/v1/vector_stores/vs_1/file_batches/vsfb_1/files",
            get(upstream_vector_store_file_batch_files_handler),
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
                .uri("/v1/vector_stores/vs_1/file_batches")
                .header("content-type", "application/json")
                .body(Body::from("{\"file_ids\":[\"file_1\"]}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "vsfb_1");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-stateless-openai")
    );

    let retrieve_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/vector_stores/vs_1/file_batches/vsfb_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(retrieve_response.status(), StatusCode::OK);
    let retrieve_json = read_json(retrieve_response).await;
    assert_eq!(retrieve_json["id"], "vsfb_1");

    let cancel_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/vector_stores/vs_1/file_batches/vsfb_1/cancel")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cancel_response.status(), StatusCode::OK);
    let cancel_json = read_json(cancel_response).await;
    assert_eq!(cancel_json["status"], "cancelled");

    let list_files_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/vector_stores/vs_1/file_batches/vsfb_1/files")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_files_response.status(), StatusCode::OK);
    let list_files_json = read_json(list_files_response).await;
    assert_eq!(list_files_json["data"][0]["id"], "file_1");
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
}

#[tokio::test]
async fn stateful_vector_store_file_batches_route_relays_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/vector_stores/vs_1/file_batches",
            axum::routing::post(upstream_vector_store_file_batches_handler),
        )
        .route(
            "/v1/vector_stores/vs_1/file_batches/vsfb_1",
            get(upstream_vector_store_file_batch_retrieve_handler),
        )
        .route(
            "/v1/vector_stores/vs_1/file_batches/vsfb_1/cancel",
            axum::routing::post(upstream_vector_store_file_batch_cancel_handler),
        )
        .route(
            "/v1/vector_stores/vs_1/file_batches/vsfb_1/files",
            get(upstream_vector_store_file_batch_files_handler),
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
                .uri("/v1/vector_stores/vs_1/file_batches")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"file_ids\":[\"file_1\"]}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "vsfb_1");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );

    let retrieve_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/vector_stores/vs_1/file_batches/vsfb_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(retrieve_response.status(), StatusCode::OK);
    let retrieve_json = read_json(retrieve_response).await;
    assert_eq!(retrieve_json["id"], "vsfb_1");

    let cancel_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/vector_stores/vs_1/file_batches/vsfb_1/cancel")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cancel_response.status(), StatusCode::OK);
    let cancel_json = read_json(cancel_response).await;
    assert_eq!(cancel_json["status"], "cancelled");

    let list_files_response = gateway_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/vector_stores/vs_1/file_batches/vsfb_1/files")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_files_response.status(), StatusCode::OK);
    let list_files_json = read_json(list_files_response).await;
    assert_eq!(list_files_json["object"], "list");
}

async fn upstream_vector_store_file_batches_handler(
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
            "id":"vsfb_1",
            "object":"vector_store.file_batch",
            "status":"in_progress"
        })),
    )
}

async fn upstream_vector_store_file_batch_retrieve_handler(
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
            "id":"vsfb_1",
            "object":"vector_store.file_batch",
            "status":"completed"
        })),
    )
}

async fn upstream_vector_store_file_batch_cancel_handler(
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
            "id":"vsfb_1",
            "object":"vector_store.file_batch",
            "status":"cancelled"
        })),
    )
}

async fn upstream_vector_store_file_batch_files_handler(
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
                "object":"vector_store.file",
                "status":"completed"
            }]
        })),
    )
}
