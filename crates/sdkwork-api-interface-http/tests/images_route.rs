use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use serde_json::Value;
use sqlx::SqlitePool;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

mod support;

#[tokio::test]
async fn images_generation_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/images/generations")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-image-1\",\"prompt\":\"draw a lighthouse\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn images_generation_route_returns_invalid_request_for_missing_model() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/images/generations")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"\",\"prompt\":\"draw a lighthouse\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_invalid_image_generation_request(response).await;
}

#[tokio::test]
async fn images_edit_route_accepts_multipart() {
    let app = sdkwork_api_interface_http::gateway_router();
    let boundary = "sdkwork-boundary-edit";
    let response = app
        .oneshot(multipart_request(
            "/v1/images/edits",
            boundary,
            None,
            multipart_body(
                boundary,
                &[("model", "gpt-image-1"), ("prompt", "make it sunset")],
                &[
                    ("image", "source.png", "image/png", b"PNGDATA"),
                    ("mask", "mask.png", "image/png", b"MASKDATA"),
                ],
            ),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn images_variation_route_accepts_multipart() {
    let app = sdkwork_api_interface_http::gateway_router();
    let boundary = "sdkwork-boundary-variation";
    let response = app
        .oneshot(multipart_request(
            "/v1/images/variations",
            boundary,
            None,
            multipart_body(
                boundary,
                &[("model", "gpt-image-1")],
                &[("image", "source.png", "image/png", b"PNGDATA")],
            ),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn stateless_images_routes_relay_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/images/generations",
            post(upstream_images_generation_handler),
        )
        .route("/v1/images/edits", post(upstream_images_multipart_handler))
        .route(
            "/v1/images/variations",
            post(upstream_images_multipart_handler),
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

    let generation_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/images/generations")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-image-1\",\"prompt\":\"relay me\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(generation_response.status(), StatusCode::OK);
    let generation_json = read_json(generation_response).await;
    assert_eq!(generation_json["data"][0]["b64_json"], "upstream-image");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-stateless-openai")
    );

    let edit_boundary = "sdkwork-stateless-edit";
    let edit_response = app
        .clone()
        .oneshot(multipart_request(
            "/v1/images/edits",
            edit_boundary,
            None,
            multipart_body(
                edit_boundary,
                &[("model", "gpt-image-1"), ("prompt", "relay edit")],
                &[
                    ("image", "source.png", "image/png", b"PNGDATA"),
                    ("mask", "mask.png", "image/png", b"MASKDATA"),
                ],
            ),
        ))
        .await
        .unwrap();

    assert_eq!(edit_response.status(), StatusCode::OK);
    let edit_json = read_json(edit_response).await;
    assert_eq!(edit_json["data"][0]["b64_json"], "upstream-image");
    assert!(upstream_state
        .content_type
        .lock()
        .unwrap()
        .as_deref()
        .unwrap_or_default()
        .starts_with("multipart/form-data; boundary="));
    let edit_body = upstream_state.raw_body.lock().unwrap().clone().unwrap();
    let edit_body = String::from_utf8_lossy(&edit_body);
    assert!(edit_body.contains("relay edit"));
    assert!(edit_body.contains("source.png"));
    assert!(edit_body.contains("mask.png"));

    let variation_boundary = "sdkwork-stateless-variation";
    let variation_response = app
        .oneshot(multipart_request(
            "/v1/images/variations",
            variation_boundary,
            None,
            multipart_body(
                variation_boundary,
                &[("model", "gpt-image-1")],
                &[("image", "source.png", "image/png", b"PNGDATA")],
            ),
        ))
        .await
        .unwrap();

    assert_eq!(variation_response.status(), StatusCode::OK);
    let variation_json = read_json(variation_response).await;
    assert_eq!(variation_json["data"][0]["b64_json"], "upstream-image");
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
    raw_body: Arc<Mutex<Option<Vec<u8>>>>,
}

#[tokio::test]
async fn stateful_images_generation_route_relays_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/images/generations",
            post(upstream_images_generation_handler),
        )
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let (gateway_app, api_key) = configure_gateway_with_image_provider(address).await;
    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/images/generations")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-image-1\",\"prompt\":\"relay me\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["data"][0]["b64_json"], "upstream-image");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
}

#[tokio::test]
async fn stateful_images_generation_route_requires_gateway_api_key() {
    let pool = memory_pool().await;
    let _api_key = support::issue_gateway_api_key(&pool, "tenant-live", "project-live").await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/images/generations")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-image-1\",\"prompt\":\"draw a lighthouse\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn stateful_images_generation_route_returns_invalid_request_for_missing_model_without_usage()
{
    let pool = memory_pool().await;
    let api_key =
        support::issue_gateway_api_key(&pool, "tenant-image-invalid", "project-image-invalid")
            .await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let admin_token = support::issue_admin_token(admin_app.clone()).await;

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/images/generations")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"\",\"prompt\":\"draw a lighthouse\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_invalid_image_generation_request(response).await;
    support::assert_no_usage_records(admin_app, &admin_token).await;
}

#[tokio::test]
async fn stateful_images_edit_route_relays_multipart_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/images/edits", post(upstream_images_multipart_handler))
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let (gateway_app, api_key) = configure_gateway_with_image_provider(address).await;
    let boundary = "sdkwork-upstream-edit";
    let response = gateway_app
        .oneshot(multipart_request(
            "/v1/images/edits",
            boundary,
            Some(&api_key),
            multipart_body(
                boundary,
                &[("model", "gpt-image-1"), ("prompt", "relay edit")],
                &[
                    ("image", "source.png", "image/png", b"PNGDATA"),
                    ("mask", "mask.png", "image/png", b"MASKDATA"),
                ],
            ),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["data"][0]["b64_json"], "upstream-image");
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
        .starts_with("multipart/form-data; boundary="));
    let raw_body = upstream_state.raw_body.lock().unwrap().clone().unwrap();
    let body = String::from_utf8_lossy(&raw_body);
    assert!(body.contains("relay edit"));
    assert!(body.contains("source.png"));
    assert!(body.contains("mask.png"));
}

#[tokio::test]
async fn stateful_images_variation_route_relays_multipart_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/images/variations",
            post(upstream_images_multipart_handler),
        )
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let (gateway_app, api_key) = configure_gateway_with_image_provider(address).await;
    let boundary = "sdkwork-upstream-variation";
    let response = gateway_app
        .oneshot(multipart_request(
            "/v1/images/variations",
            boundary,
            Some(&api_key),
            multipart_body(
                boundary,
                &[("model", "gpt-image-1")],
                &[("image", "source.png", "image/png", b"PNGDATA")],
            ),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["data"][0]["b64_json"], "upstream-image");
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
        .starts_with("multipart/form-data; boundary="));
    let raw_body = upstream_state.raw_body.lock().unwrap().clone().unwrap();
    let body = String::from_utf8_lossy(&raw_body);
    assert!(body.contains("source.png"));
}

async fn configure_gateway_with_image_provider(address: SocketAddr) -> (Router, String) {
    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, "tenant-1", "project-1").await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
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

    let model = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
.header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-image-1\",\"provider_id\":\"provider-openai-official\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(model.status(), StatusCode::CREATED);

    (gateway_app, api_key)
}

fn multipart_request(
    uri: &str,
    boundary: &str,
    api_key: Option<&str>,
    body: Vec<u8>,
) -> Request<Body> {
    let mut builder = Request::builder().method("POST").uri(uri).header(
        "content-type",
        format!("multipart/form-data; boundary={boundary}"),
    );

    if let Some(api_key) = api_key {
        builder = builder.header("authorization", format!("Bearer {api_key}"));
    }

    builder.body(Body::from(body)).unwrap()
}

fn multipart_body(
    boundary: &str,
    text_fields: &[(&str, &str)],
    file_fields: &[(&str, &str, &str, &[u8])],
) -> Vec<u8> {
    let mut body = Vec::new();

    for (name, value) in text_fields {
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(
            format!("Content-Disposition: form-data; name=\"{name}\"\r\n\r\n").as_bytes(),
        );
        body.extend_from_slice(value.as_bytes());
        body.extend_from_slice(b"\r\n");
    }

    for (name, filename, content_type, bytes) in file_fields {
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(
            format!("Content-Disposition: form-data; name=\"{name}\"; filename=\"{filename}\"\r\n")
                .as_bytes(),
        );
        body.extend_from_slice(format!("Content-Type: {content_type}\r\n\r\n").as_bytes());
        body.extend_from_slice(bytes);
        body.extend_from_slice(b"\r\n");
    }

    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
    body
}

async fn upstream_images_generation_handler(
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
            "data":[{"b64_json":"upstream-image"}]
        })),
    )
}

async fn upstream_images_multipart_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
    body: axum::body::Bytes,
) -> (StatusCode, Json<Value>) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    *state.content_type.lock().unwrap() = headers
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    *state.raw_body.lock().unwrap() = Some(body.to_vec());

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "data":[{"b64_json":"upstream-image"}]
        })),
    )
}

async fn assert_invalid_image_generation_request(response: axum::response::Response) {
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = read_json(response).await;
    assert_eq!(
        json["error"]["message"],
        "Image generation model is required."
    );
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "invalid_model");
}
