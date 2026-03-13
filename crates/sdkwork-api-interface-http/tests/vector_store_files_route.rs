use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::Value;
use sqlx::SqlitePool;
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

#[tokio::test]
async fn vector_store_files_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/vector_stores/vs_1/files")
                .header("content-type", "application/json")
                .body(Body::from("{\"file_id\":\"file_1\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn vector_store_files_list_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/vector_stores/vs_1/files")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn vector_store_file_retrieve_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/vector_stores/vs_1/files/file_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn vector_store_file_delete_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/vector_stores/vs_1/files/file_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
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
async fn stateful_vector_store_files_route_relays_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/vector_stores/vs_1/files",
            get(upstream_vector_store_files_list_handler).post(upstream_vector_store_files_handler),
        )
        .route(
            "/v1/vector_stores/vs_1/files/file_1",
            get(upstream_vector_store_file_retrieve_handler)
                .delete(upstream_vector_store_file_delete_handler),
        )
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let _ = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/channels")
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
                .uri("/v1/vector_stores/vs_1/files")
                .header("content-type", "application/json")
                .body(Body::from("{\"file_id\":\"file_1\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "file_1");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );

    let list_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/vector_stores/vs_1/files")
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
                .uri("/v1/vector_stores/vs_1/files/file_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(retrieve_response.status(), StatusCode::OK);
    let retrieve_json = read_json(retrieve_response).await;
    assert_eq!(retrieve_json["id"], "file_1");

    let delete_response = gateway_app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/vector_stores/vs_1/files/file_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_response.status(), StatusCode::OK);
    let delete_json = read_json(delete_response).await;
    assert_eq!(delete_json["deleted"], true);
}

async fn upstream_vector_store_files_handler(
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
            "object":"vector_store.file",
            "status":"completed"
        })),
    )
}

async fn upstream_vector_store_files_list_handler(
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

async fn upstream_vector_store_file_retrieve_handler(
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
            "object":"vector_store.file",
            "status":"completed"
        })),
    )
}

async fn upstream_vector_store_file_delete_handler(
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
            "object":"vector_store.file.deleted",
            "deleted":true
        })),
    )
}
