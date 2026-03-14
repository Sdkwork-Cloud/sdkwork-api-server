use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use serde_json::Value;
use sqlx::SqlitePool;
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

mod support;

#[tokio::test]
async fn uploads_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/uploads")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"purpose\":\"batch\",\"filename\":\"input.jsonl\",\"mime_type\":\"application/jsonl\",\"bytes\":1024}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn upload_parts_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/uploads/upload_1/parts")
                .header(
                    "content-type",
                    "multipart/form-data; boundary=----sdkwork-upload-part",
                )
                .body(Body::from(build_upload_part_multipart_body(
                    "----sdkwork-upload-part",
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn upload_complete_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/uploads/upload_1/complete")
                .header("content-type", "application/json")
                .body(Body::from("{\"part_ids\":[\"part_1\",\"part_2\"]}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn upload_cancel_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/uploads/upload_1/cancel")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn stateless_upload_routes_relay_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/uploads", post(upstream_uploads_handler))
        .route(
            "/v1/uploads/upload_1/parts",
            post(upstream_upload_parts_handler),
        )
        .route(
            "/v1/uploads/upload_1/complete",
            post(upstream_upload_complete_handler),
        )
        .route(
            "/v1/uploads/upload_1/cancel",
            post(upstream_upload_cancel_handler),
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

    let upload_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/uploads")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"purpose\":\"batch\",\"filename\":\"input.jsonl\",\"mime_type\":\"application/jsonl\",\"bytes\":1024}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(upload_response.status(), StatusCode::OK);
    let upload_json = read_json(upload_response).await;
    assert_eq!(upload_json["id"], "upload_upstream");

    let part_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/uploads/upload_1/parts")
                .header(
                    "content-type",
                    "multipart/form-data; boundary=----sdkwork-upload-part",
                )
                .body(Body::from(build_upload_part_multipart_body(
                    "----sdkwork-upload-part",
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(part_response.status(), StatusCode::OK);
    let part_json = read_json(part_response).await;
    assert_eq!(part_json["id"], "part_upstream");

    let complete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/uploads/upload_1/complete")
                .header("content-type", "application/json")
                .body(Body::from("{\"part_ids\":[\"part_1\",\"part_2\"]}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(complete_response.status(), StatusCode::OK);
    let complete_json = read_json(complete_response).await;
    assert_eq!(complete_json["part_ids"][1], "part_2");
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

    let cancel_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/uploads/upload_1/cancel")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cancel_response.status(), StatusCode::OK);
    let cancel_json = read_json(cancel_response).await;
    assert_eq!(cancel_json["status"], "cancelled");
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

#[tokio::test]
async fn stateful_upload_routes_relay_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/uploads", post(upstream_uploads_handler))
        .route(
            "/v1/uploads/upload_1/parts",
            post(upstream_upload_parts_handler),
        )
        .route(
            "/v1/uploads/upload_1/complete",
            post(upstream_upload_complete_handler),
        )
        .route(
            "/v1/uploads/upload_1/cancel",
            post(upstream_upload_cancel_handler),
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

    let upload_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/uploads")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"purpose\":\"batch\",\"filename\":\"input.jsonl\",\"mime_type\":\"application/jsonl\",\"bytes\":1024}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(upload_response.status(), StatusCode::OK);
    let upload_json = read_json(upload_response).await;
    assert_eq!(upload_json["id"], "upload_upstream");

    let part_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/uploads/upload_1/parts")
                .header("authorization", format!("Bearer {api_key}"))
                .header(
                    "content-type",
                    "multipart/form-data; boundary=----sdkwork-upload-part",
                )
                .body(Body::from(build_upload_part_multipart_body(
                    "----sdkwork-upload-part",
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(part_response.status(), StatusCode::OK);
    let part_json = read_json(part_response).await;
    assert_eq!(part_json["id"], "part_upstream");

    let complete_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/uploads/upload_1/complete")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"part_ids\":[\"part_1\",\"part_2\"]}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(complete_response.status(), StatusCode::OK);
    let complete_json = read_json(complete_response).await;
    assert_eq!(complete_json["part_ids"][1], "part_2");
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

    let cancel_response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/uploads/upload_1/cancel")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cancel_response.status(), StatusCode::OK);
    let cancel_json = read_json(cancel_response).await;
    assert_eq!(cancel_json["status"], "cancelled");
}

fn build_upload_part_multipart_body(boundary: &str) -> Vec<u8> {
    format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"data\"; filename=\"part-1.bin\"\r\nContent-Type: application/octet-stream\r\n\r\npart-data\r\n--{boundary}--\r\n"
    )
    .into_bytes()
}

async fn upstream_uploads_handler(
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
            "id":"upload_upstream",
            "object":"upload",
            "purpose":"batch",
            "filename":"input.jsonl",
            "mime_type":"application/jsonl",
            "bytes":1024,
            "part_ids":[],
            "status":"pending"
        })),
    )
}

async fn upstream_upload_parts_handler(
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
            "id":"part_upstream",
            "object":"upload.part",
            "upload_id":"upload_1",
            "status":"completed"
        })),
    )
}

async fn upstream_upload_complete_handler(
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
            "id":"upload_upstream",
            "object":"upload",
            "purpose":"batch",
            "filename":"input.jsonl",
            "mime_type":"application/jsonl",
            "bytes":1024,
            "part_ids":["part_1","part_2"],
            "status":"completed"
        })),
    )
}

async fn upstream_upload_cancel_handler(
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
            "id":"upload_upstream",
            "object":"upload",
            "purpose":"batch",
            "filename":"input.jsonl",
            "mime_type":"application/jsonl",
            "bytes":1024,
            "part_ids":["part_1","part_2"],
            "status":"cancelled"
        })),
    )
}
