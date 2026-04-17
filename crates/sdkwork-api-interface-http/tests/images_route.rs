use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{Request, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::Value;
use sqlx::SqlitePool;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

mod support;

#[tokio::test]
async fn images_generation_route_returns_invalid_request_without_image_backend() {
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

    assert_invalid_image_backend_request(
        response,
        "Local image generation fallback is not supported without an image backend.",
    )
    .await;
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
async fn images_edit_route_returns_invalid_request_without_image_backend() {
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

    assert_invalid_image_backend_request(
        response,
        "Local image edit fallback is not supported without an image backend.",
    )
    .await;
}

#[tokio::test]
async fn images_variation_route_returns_invalid_request_without_image_backend() {
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

    assert_invalid_image_backend_request(
        response,
        "Local image variation fallback is not supported without an image backend.",
    )
    .await;
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

#[tokio::test]
async fn stateless_images_kling_routes_relay_to_official_paths() {
    let upstream_state = KlingImageUpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/api/v1/services/aigc/image-generation/generation",
            post(upstream_kling_image_generation_handler),
        )
        .route(
            "/api/v1/tasks/{task_id}",
            get(upstream_kling_image_task_handler),
        )
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let app = sdkwork_api_interface_http::gateway_router_with_stateless_config(
        sdkwork_api_interface_http::StatelessGatewayConfig::default().with_upstream(
            sdkwork_api_interface_http::StatelessGatewayUpstream::new_with_protocol_kind_and_identity(
                "sdkwork.provider.kling",
                "custom",
                "kling",
                format!("http://{address}"),
                "sk-stateless-kling",
            ),
        ),
    );

    let generation_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/services/aigc/image-generation/generation")
                .header("content-type", "application/json")
                .header("X-DashScope-Async", "enable")
                .body(Body::from(
                    "{\"model\":\"kling-v1\",\"input\":{\"prompt\":\"draw a lighthouse\"}}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(generation_response.status(), StatusCode::OK);
    let generation_json = read_json(generation_response).await;
    assert_eq!(generation_json["output"]["task_id"], "task_kling_image_1");

    let task_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/tasks/task_kling_image_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(task_response.status(), StatusCode::OK);
    let task_json = read_json(task_response).await;
    assert_eq!(task_json["output"]["task_id"], "task_kling_image_1");
    assert_eq!(
        task_json["output"]["choices"][0]["message"]["content"][0]["image"],
        "https://cdn.example.com/task_kling_image_1.png"
    );

    assert_eq!(
        upstream_state.authorizations.lock().unwrap().clone(),
        vec![
            "Bearer sk-stateless-kling".to_owned(),
            "Bearer sk-stateless-kling".to_owned()
        ]
    );
    assert_eq!(
        upstream_state
            .create_async_header
            .lock()
            .unwrap()
            .as_deref(),
        Some("enable")
    );
    assert_eq!(
        upstream_state.generation_body.lock().unwrap().clone(),
        Some(serde_json::json!({
            "model":"kling-v1",
            "input":{"prompt":"draw a lighthouse"}
        }))
    );
    assert_eq!(
        upstream_state.task_ids.lock().unwrap().clone(),
        vec!["task_kling_image_1".to_owned()]
    );
}

#[tokio::test]
async fn stateless_images_aliyun_routes_relay_to_official_paths() {
    let upstream_state = AliyunImageUpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/api/v1/services/aigc/image-generation/generation",
            post(upstream_aliyun_image_generation_handler),
        )
        .route(
            "/api/v1/tasks/{task_id}",
            get(upstream_aliyun_image_task_handler),
        )
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let app = sdkwork_api_interface_http::gateway_router_with_stateless_config(
        sdkwork_api_interface_http::StatelessGatewayConfig::default().with_upstream(
            sdkwork_api_interface_http::StatelessGatewayUpstream::new_with_protocol_kind_and_identity(
                "sdkwork.provider.aliyun",
                "custom",
                "aliyun",
                format!("http://{address}"),
                "sk-stateless-aliyun",
            ),
        ),
    );

    let generation_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/services/aigc/image-generation/generation")
                .header("content-type", "application/json")
                .header("X-DashScope-Async", "enable")
                .body(Body::from(
                    "{\"model\":\"wanx2.1-t2i-turbo\",\"input\":{\"prompt\":\"draw a lighthouse\"}}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(generation_response.status(), StatusCode::OK);
    let generation_json = read_json(generation_response).await;
    assert_eq!(generation_json["output"]["task_id"], "task_aliyun_image_1");

    let task_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/tasks/task_aliyun_image_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(task_response.status(), StatusCode::OK);
    let task_json = read_json(task_response).await;
    assert_eq!(task_json["output"]["task_id"], "task_aliyun_image_1");
    assert_eq!(
        task_json["output"]["results"][0]["url"],
        "https://cdn.example.com/task_aliyun_image_1.png"
    );

    assert_eq!(
        upstream_state.authorizations.lock().unwrap().clone(),
        vec![
            "Bearer sk-stateless-aliyun".to_owned(),
            "Bearer sk-stateless-aliyun".to_owned()
        ]
    );
    assert_eq!(
        upstream_state
            .create_async_header
            .lock()
            .unwrap()
            .as_deref(),
        Some("enable")
    );
    assert_eq!(
        upstream_state.generation_body.lock().unwrap().clone(),
        Some(serde_json::json!({
            "model":"wanx2.1-t2i-turbo",
            "input":{"prompt":"draw a lighthouse"}
        }))
    );
    assert_eq!(
        upstream_state.task_ids.lock().unwrap().clone(),
        vec!["task_aliyun_image_1".to_owned()]
    );
}

#[tokio::test]
async fn stateless_images_volcengine_routes_relay_to_official_paths() {
    let upstream_state = VolcengineImageUpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/api/v3/images/generations",
            post(upstream_volcengine_image_generation_handler),
        )
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let app = sdkwork_api_interface_http::gateway_router_with_stateless_config(
        sdkwork_api_interface_http::StatelessGatewayConfig::default().with_upstream(
            sdkwork_api_interface_http::StatelessGatewayUpstream::new_with_protocol_kind_and_identity(
                "sdkwork.provider.volcengine",
                "custom",
                "volcengine",
                format!("http://{address}"),
                "sk-stateless-volcengine-image",
            ),
        ),
    );

    let generation_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v3/images/generations")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"seedream-3.0\",\"prompt\":\"a glass lighthouse on the moon\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(generation_response.status(), StatusCode::OK);
    let generation_json = read_json(generation_response).await;
    assert_eq!(
        generation_json["data"][0]["url"],
        "https://cdn.example.com/volcengine-image-1.png"
    );

    assert_eq!(
        upstream_state.authorizations.lock().unwrap().clone(),
        vec!["Bearer sk-stateless-volcengine-image".to_owned()]
    );
    assert_eq!(
        upstream_state.generation_body.lock().unwrap().clone(),
        Some(serde_json::json!({
            "model":"seedream-3.0",
            "prompt":"a glass lighthouse on the moon"
        }))
    );
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

async fn issue_funded_gateway_api_key(
    pool: &SqlitePool,
    tenant_id: &str,
    project_id: &str,
) -> String {
    let api_key = support::issue_gateway_api_key(pool, tenant_id, project_id).await;
    support::seed_primary_commercial_credit_account(pool, tenant_id, project_id, &api_key).await;
    api_key
}

#[derive(Clone, Default)]
struct UpstreamCaptureState {
    authorization: Arc<Mutex<Option<String>>>,
    content_type: Arc<Mutex<Option<String>>>,
    raw_body: Arc<Mutex<Option<Vec<u8>>>>,
}

#[derive(Clone, Default)]
struct KlingImageUpstreamCaptureState {
    authorizations: Arc<Mutex<Vec<String>>>,
    create_async_header: Arc<Mutex<Option<String>>>,
    generation_body: Arc<Mutex<Option<Value>>>,
    task_ids: Arc<Mutex<Vec<String>>>,
}

#[derive(Clone, Default)]
struct AliyunImageUpstreamCaptureState {
    authorizations: Arc<Mutex<Vec<String>>>,
    create_async_header: Arc<Mutex<Option<String>>>,
    generation_body: Arc<Mutex<Option<Value>>>,
    task_ids: Arc<Mutex<Vec<String>>>,
}

#[derive(Clone, Default)]
struct VolcengineImageUpstreamCaptureState {
    authorizations: Arc<Mutex<Vec<String>>>,
    generation_body: Arc<Mutex<Option<Value>>>,
}

#[derive(Clone, Default)]
struct GenericImageUpstreamCaptureState {
    hits: Arc<Mutex<u64>>,
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
        issue_funded_gateway_api_key(&pool, "tenant-image-invalid", "project-image-invalid").await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(&pool, admin_app.clone()).await;

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

#[tokio::test]
async fn stateful_images_kling_routes_use_kling_provider_identity() {
    let tenant_id = "tenant-images-kling-stateful";
    let project_id = "project-images-kling-stateful";

    let generic_state = GenericImageUpstreamCaptureState::default();
    let generic_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let generic_address = generic_listener.local_addr().unwrap();
    let generic_upstream = Router::new()
        .route(
            "/api/v1/services/aigc/image-generation/generation",
            post(upstream_generic_kling_generation_handler),
        )
        .route(
            "/api/v1/tasks/{task_id}",
            get(upstream_generic_kling_task_handler),
        )
        .with_state(generic_state.clone());
    tokio::spawn(async move {
        axum::serve(generic_listener, generic_upstream)
            .await
            .unwrap();
    });

    let kling_state = KlingImageUpstreamCaptureState::default();
    let kling_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let kling_address = kling_listener.local_addr().unwrap();
    let kling_upstream = Router::new()
        .route(
            "/api/v1/services/aigc/image-generation/generation",
            post(upstream_kling_image_generation_handler),
        )
        .route(
            "/api/v1/tasks/{task_id}",
            get(upstream_kling_image_task_handler),
        )
        .with_state(kling_state.clone());
    tokio::spawn(async move {
        axum::serve(kling_listener, kling_upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let api_key = issue_funded_gateway_api_key(&pool, tenant_id, project_id).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(&pool, admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    create_channel_with_name(&admin_app, &admin_token, "openai", "OpenAI").await;
    create_channel_with_name(&admin_app, &admin_token, "kling", "Kling").await;
    create_provider_with_payload(
        &admin_app,
        &admin_token,
        &format!(
            "{{\"id\":\"provider-a-generic\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{generic_address}\",\"display_name\":\"Generic Provider\"}}"
        ),
    )
    .await;
    create_provider_with_payload(
        &admin_app,
        &admin_token,
        &format!(
            "{{\"id\":\"provider-z-kling\",\"channel_id\":\"kling\",\"extension_id\":\"sdkwork.provider.kling\",\"adapter_kind\":\"kling\",\"protocol_kind\":\"custom\",\"base_url\":\"http://{kling_address}\",\"display_name\":\"Kling Provider\"}}"
        ),
    )
    .await;
    create_credential(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-a-generic",
        "cred-generic",
        "sk-generic-upstream",
    )
    .await;
    create_credential(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-z-kling",
        "cred-kling",
        "sk-kling-upstream",
    )
    .await;
    create_model_binding(&admin_app, &admin_token, "kling-v1", "provider-z-kling").await;

    let generation_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/services/aigc/image-generation/generation")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .header("X-DashScope-Async", "enable")
                .body(Body::from(
                    "{\"model\":\"kling-v1\",\"input\":{\"prompt\":\"draw a lighthouse\"}}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(generation_response.status(), StatusCode::OK);
    let generation_json = read_json(generation_response).await;
    assert_eq!(generation_json["output"]["task_id"], "task_kling_image_1");

    let task_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/tasks/task_kling_image_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(task_response.status(), StatusCode::OK);
    let task_json = read_json(task_response).await;
    assert_eq!(task_json["output"]["task_id"], "task_kling_image_1");

    assert_eq!(*generic_state.hits.lock().unwrap(), 0);
    assert_eq!(
        kling_state.authorizations.lock().unwrap().clone(),
        vec![
            "Bearer sk-kling-upstream".to_owned(),
            "Bearer sk-kling-upstream".to_owned()
        ]
    );
    assert_eq!(
        kling_state.create_async_header.lock().unwrap().as_deref(),
        Some("enable")
    );
    assert_eq!(
        kling_state.generation_body.lock().unwrap().clone(),
        Some(serde_json::json!({
            "model":"kling-v1",
            "input":{"prompt":"draw a lighthouse"}
        }))
    );
    assert_eq!(
        kling_state.task_ids.lock().unwrap().clone(),
        vec!["task_kling_image_1".to_owned()]
    );

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
    let usage_records = usage_json.as_array().unwrap();
    assert_eq!(usage_records.len(), 2);
    assert!(usage_records.iter().any(|record| {
        record["model"] == "images.kling.generation" && record["provider"] == "provider-z-kling"
    }));
    assert!(usage_records.iter().any(|record| {
        record["model"] == "provider.kling.tasks.get" && record["provider"] == "provider-z-kling"
    }));

    let billing_events = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/billing/events")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(billing_events.status(), StatusCode::OK);
    let billing_json = read_json(billing_events).await;
    let billing_records = billing_json.as_array().unwrap();
    assert_eq!(billing_records.len(), 2);
    assert!(billing_records.iter().any(|record| {
        record["route_key"] == "images.kling.generation"
            && record["provider_id"] == "provider-z-kling"
            && record["reference_id"] == "task_kling_image_1"
    }));
    assert!(billing_records.iter().any(|record| {
        record["route_key"] == "provider.kling.tasks.get"
            && record["provider_id"] == "provider-z-kling"
            && record["reference_id"] == "task_kling_image_1"
    }));

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
    assert_eq!(logs_json.as_array().unwrap().len(), 2);
    assert_eq!(logs_json[0]["route_key"], "provider.kling.tasks.get");
    assert_eq!(logs_json[0]["selected_provider_id"], "provider-z-kling");
    assert_eq!(logs_json[1]["route_key"], "images.kling.generation");
    assert_eq!(logs_json[1]["selected_provider_id"], "provider-z-kling");
}

#[tokio::test]
async fn stateful_images_aliyun_routes_use_task_ownership_resolution() {
    let tenant_id = "tenant-images-aliyun-stateful";
    let project_id = "project-images-aliyun-stateful";

    let generic_state = GenericImageUpstreamCaptureState::default();
    let generic_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let generic_address = generic_listener.local_addr().unwrap();
    let generic_upstream = Router::new()
        .route(
            "/api/v1/services/aigc/image-generation/generation",
            post(upstream_generic_kling_generation_handler),
        )
        .route(
            "/api/v1/tasks/{task_id}",
            get(upstream_generic_kling_task_handler),
        )
        .with_state(generic_state.clone());
    tokio::spawn(async move {
        axum::serve(generic_listener, generic_upstream)
            .await
            .unwrap();
    });

    let kling_state = KlingImageUpstreamCaptureState::default();
    let kling_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let kling_address = kling_listener.local_addr().unwrap();
    let kling_upstream = Router::new()
        .route(
            "/api/v1/services/aigc/image-generation/generation",
            post(upstream_kling_image_generation_handler),
        )
        .route(
            "/api/v1/tasks/{task_id}",
            get(upstream_kling_image_task_handler),
        )
        .with_state(kling_state.clone());
    tokio::spawn(async move {
        axum::serve(kling_listener, kling_upstream).await.unwrap();
    });

    let aliyun_state = AliyunImageUpstreamCaptureState::default();
    let aliyun_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let aliyun_address = aliyun_listener.local_addr().unwrap();
    let aliyun_upstream = Router::new()
        .route(
            "/api/v1/services/aigc/image-generation/generation",
            post(upstream_aliyun_image_generation_handler),
        )
        .route(
            "/api/v1/tasks/{task_id}",
            get(upstream_aliyun_image_task_handler),
        )
        .with_state(aliyun_state.clone());
    tokio::spawn(async move {
        axum::serve(aliyun_listener, aliyun_upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let api_key = issue_funded_gateway_api_key(&pool, tenant_id, project_id).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(&pool, admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    create_channel_with_name(&admin_app, &admin_token, "openai", "OpenAI").await;
    create_channel_with_name(&admin_app, &admin_token, "kling", "Kling").await;
    create_channel_with_name(&admin_app, &admin_token, "aliyun", "Aliyun").await;
    create_provider_with_payload(
        &admin_app,
        &admin_token,
        &format!(
            "{{\"id\":\"provider-a-generic\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{generic_address}\",\"display_name\":\"Generic Provider\"}}"
        ),
    )
    .await;
    create_provider_with_payload(
        &admin_app,
        &admin_token,
        &format!(
            "{{\"id\":\"provider-z-kling\",\"channel_id\":\"kling\",\"extension_id\":\"sdkwork.provider.kling\",\"adapter_kind\":\"kling\",\"protocol_kind\":\"custom\",\"base_url\":\"http://{kling_address}\",\"display_name\":\"Kling Provider\"}}"
        ),
    )
    .await;
    create_provider_with_payload(
        &admin_app,
        &admin_token,
        &format!(
            "{{\"id\":\"provider-z-aliyun\",\"channel_id\":\"aliyun\",\"extension_id\":\"sdkwork.provider.aliyun\",\"adapter_kind\":\"native-dynamic\",\"protocol_kind\":\"custom\",\"base_url\":\"http://{aliyun_address}\",\"display_name\":\"Aliyun Provider\"}}"
        ),
    )
    .await;
    create_credential(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-a-generic",
        "cred-generic",
        "sk-generic-upstream",
    )
    .await;
    create_credential(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-z-kling",
        "cred-kling",
        "sk-kling-upstream",
    )
    .await;
    create_credential(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-z-aliyun",
        "cred-aliyun",
        "sk-aliyun-upstream",
    )
    .await;
    create_model_binding(
        &admin_app,
        &admin_token,
        "wanx2.1-t2i-turbo",
        "provider-z-aliyun",
    )
    .await;
    create_model_binding(&admin_app, &admin_token, "kling-v1", "provider-z-kling").await;

    let generation_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/services/aigc/image-generation/generation")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .header("X-DashScope-Async", "enable")
                .body(Body::from(
                    "{\"model\":\"wanx2.1-t2i-turbo\",\"input\":{\"prompt\":\"draw a lighthouse\"}}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(generation_response.status(), StatusCode::OK);
    let generation_json = read_json(generation_response).await;
    assert_eq!(generation_json["output"]["task_id"], "task_aliyun_image_1");

    let task_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/tasks/task_aliyun_image_1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(task_response.status(), StatusCode::OK);
    let task_json = read_json(task_response).await;
    assert_eq!(task_json["output"]["task_id"], "task_aliyun_image_1");
    assert_eq!(
        task_json["output"]["results"][0]["url"],
        "https://cdn.example.com/task_aliyun_image_1.png"
    );

    assert_eq!(*generic_state.hits.lock().unwrap(), 0);
    assert!(kling_state.authorizations.lock().unwrap().is_empty());
    assert_eq!(
        aliyun_state.authorizations.lock().unwrap().clone(),
        vec![
            "Bearer sk-aliyun-upstream".to_owned(),
            "Bearer sk-aliyun-upstream".to_owned()
        ]
    );
    assert_eq!(
        aliyun_state.create_async_header.lock().unwrap().as_deref(),
        Some("enable")
    );
    assert_eq!(
        aliyun_state.generation_body.lock().unwrap().clone(),
        Some(serde_json::json!({
            "model":"wanx2.1-t2i-turbo",
            "input":{"prompt":"draw a lighthouse"}
        }))
    );
    assert_eq!(
        aliyun_state.task_ids.lock().unwrap().clone(),
        vec!["task_aliyun_image_1".to_owned()]
    );

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
    let usage_records = usage_json.as_array().unwrap();
    assert_eq!(usage_records.len(), 2);
    assert!(usage_records.iter().any(|record| {
        record["model"] == "images.aliyun.generation" && record["provider"] == "provider-z-aliyun"
    }));
    assert!(usage_records.iter().any(|record| {
        record["model"] == "provider.aliyun.tasks.get" && record["provider"] == "provider-z-aliyun"
    }));

    let billing_events = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/billing/events")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(billing_events.status(), StatusCode::OK);
    let billing_json = read_json(billing_events).await;
    let billing_records = billing_json.as_array().unwrap();
    assert_eq!(billing_records.len(), 2);
    assert!(billing_records.iter().any(|record| {
        record["route_key"] == "images.aliyun.generation"
            && record["provider_id"] == "provider-z-aliyun"
            && record["reference_id"] == "task_aliyun_image_1"
    }));
    assert!(billing_records.iter().any(|record| {
        record["route_key"] == "provider.aliyun.tasks.get"
            && record["provider_id"] == "provider-z-aliyun"
            && record["reference_id"] == "task_aliyun_image_1"
    }));

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
    let logs = logs_json.as_array().unwrap();
    assert_eq!(logs.len(), 2);
    assert!(logs.iter().any(|record| {
        record["route_key"] == "images.aliyun.generation"
            && record["selected_provider_id"] == "provider-z-aliyun"
    }));
    assert!(logs.iter().any(|record| {
        record["route_key"] == "provider.aliyun.tasks.get"
            && record["selected_provider_id"] == "provider-z-aliyun"
    }));
}

#[tokio::test]
async fn stateful_images_volcengine_routes_use_volcengine_provider_identity() {
    let tenant_id = "tenant-images-volcengine-stateful";
    let project_id = "project-images-volcengine-stateful";

    let generic_state = GenericImageUpstreamCaptureState::default();
    let generic_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let generic_address = generic_listener.local_addr().unwrap();
    let generic_upstream = Router::new()
        .route(
            "/api/v3/images/generations",
            post(upstream_generic_volcengine_image_generation_handler),
        )
        .with_state(generic_state.clone());
    tokio::spawn(async move {
        axum::serve(generic_listener, generic_upstream)
            .await
            .unwrap();
    });

    let volcengine_state = VolcengineImageUpstreamCaptureState::default();
    let volcengine_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let volcengine_address = volcengine_listener.local_addr().unwrap();
    let volcengine_upstream = Router::new()
        .route(
            "/api/v3/images/generations",
            post(upstream_volcengine_image_generation_handler),
        )
        .with_state(volcengine_state.clone());
    tokio::spawn(async move {
        axum::serve(volcengine_listener, volcengine_upstream)
            .await
            .unwrap();
    });

    let pool = memory_pool().await;
    let api_key = issue_funded_gateway_api_key(&pool, tenant_id, project_id).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(&pool, admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    create_channel_with_name(&admin_app, &admin_token, "openai", "OpenAI").await;
    create_channel_with_name(&admin_app, &admin_token, "volcengine", "Volcengine").await;
    create_provider_with_payload(
        &admin_app,
        &admin_token,
        &format!(
            "{{\"id\":\"provider-a-generic\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{generic_address}\",\"display_name\":\"Generic Provider\"}}"
        ),
    )
    .await;
    create_provider_with_payload(
        &admin_app,
        &admin_token,
        &format!(
            "{{\"id\":\"provider-z-volcengine\",\"channel_id\":\"volcengine\",\"extension_id\":\"sdkwork.provider.volcengine\",\"adapter_kind\":\"native-dynamic\",\"protocol_kind\":\"custom\",\"base_url\":\"http://{volcengine_address}\",\"display_name\":\"Volcengine Provider\"}}"
        ),
    )
    .await;
    create_credential(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-a-generic",
        "cred-generic",
        "sk-generic-upstream",
    )
    .await;
    create_credential(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-z-volcengine",
        "cred-volcengine",
        "sk-volcengine-upstream",
    )
    .await;
    create_model_binding(
        &admin_app,
        &admin_token,
        "seedream-3.0",
        "provider-z-volcengine",
    )
    .await;

    let generation_response = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v3/images/generations")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"seedream-3.0\",\"prompt\":\"a glass lighthouse on the moon\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(generation_response.status(), StatusCode::OK);
    let generation_json = read_json(generation_response).await;
    assert_eq!(
        generation_json["data"][0]["url"],
        "https://cdn.example.com/volcengine-image-1.png"
    );

    assert_eq!(*generic_state.hits.lock().unwrap(), 0);
    assert_eq!(
        volcengine_state.authorizations.lock().unwrap().clone(),
        vec!["Bearer sk-volcengine-upstream".to_owned()]
    );
    assert_eq!(
        volcengine_state.generation_body.lock().unwrap().clone(),
        Some(serde_json::json!({
            "model":"seedream-3.0",
            "prompt":"a glass lighthouse on the moon"
        }))
    );

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
    let usage_records = usage_json.as_array().unwrap();
    assert_eq!(usage_records.len(), 1);
    assert!(usage_records.iter().any(|record| {
        record["model"] == "images.volcengine.generate"
            && record["provider"] == "provider-z-volcengine"
    }));

    let billing_events = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/billing/events")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(billing_events.status(), StatusCode::OK);
    let billing_json = read_json(billing_events).await;
    let billing_records = billing_json.as_array().unwrap();
    assert_eq!(billing_records.len(), 1);
    assert!(billing_records.iter().any(|record| {
        record["route_key"] == "images.volcengine.generate"
            && record["provider_id"] == "provider-z-volcengine"
            && record["reference_id"].is_null()
    }));

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
    let logs = logs_json.as_array().unwrap();
    assert_eq!(logs.len(), 1);
    assert!(logs.iter().any(|record| {
        record["route_key"] == "images.volcengine.generate"
            && record["selected_provider_id"] == "provider-z-volcengine"
    }));
}

async fn configure_gateway_with_image_provider(address: SocketAddr) -> (Router, String) {
    let pool = memory_pool().await;
    let api_key = issue_funded_gateway_api_key(&pool, "tenant-1", "project-1").await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(&pool, admin_app.clone()).await;
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

async fn create_channel_with_name(
    admin_app: &Router,
    admin_token: &str,
    channel_id: &str,
    name: &str,
) {
    let response = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/channels")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"{channel_id}\",\"name\":\"{name}\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}

async fn create_provider_with_payload(admin_app: &Router, admin_token: &str, payload: &str) {
    let response = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(payload.to_owned()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}

async fn create_credential(
    admin_app: &Router,
    admin_token: &str,
    tenant_id: &str,
    provider_id: &str,
    key_reference: &str,
    secret_value: &str,
) {
    let response = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"tenant_id\":\"{tenant_id}\",\"provider_id\":\"{provider_id}\",\"key_reference\":\"{key_reference}\",\"secret_value\":\"{secret_value}\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}

async fn create_model_binding(
    admin_app: &Router,
    admin_token: &str,
    external_name: &str,
    provider_id: &str,
) {
    let response = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"external_name\":\"{external_name}\",\"provider_id\":\"{provider_id}\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
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

async fn upstream_kling_image_generation_handler(
    State(state): State<KlingImageUpstreamCaptureState>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<Value>,
) -> (StatusCode, Json<Value>) {
    if let Some(authorization) = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
    {
        state
            .authorizations
            .lock()
            .unwrap()
            .push(authorization.to_owned());
    }
    *state.create_async_header.lock().unwrap() = headers
        .get("X-DashScope-Async")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    *state.generation_body.lock().unwrap() = Some(payload);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "request_id":"req-kling-image-1",
            "output":{
                "task_id":"task_kling_image_1",
                "task_status":"PENDING"
            }
        })),
    )
}

async fn upstream_kling_image_task_handler(
    State(state): State<KlingImageUpstreamCaptureState>,
    Path(task_id): Path<String>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    if let Some(authorization) = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
    {
        state
            .authorizations
            .lock()
            .unwrap()
            .push(authorization.to_owned());
    }
    state.task_ids.lock().unwrap().push(task_id.clone());

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "request_id":"req-kling-task-1",
            "output":{
                "task_id":task_id,
                "task_status":"SUCCEEDED",
                "choices":[{
                    "finish_reason":"stop",
                    "message":{
                        "role":"assistant",
                        "content":[{
                            "type":"image",
                            "image":"https://cdn.example.com/task_kling_image_1.png"
                        }]
                    }
                }]
            }
        })),
    )
}

async fn upstream_aliyun_image_generation_handler(
    State(state): State<AliyunImageUpstreamCaptureState>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<Value>,
) -> (StatusCode, Json<Value>) {
    if let Some(authorization) = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
    {
        state
            .authorizations
            .lock()
            .unwrap()
            .push(authorization.to_owned());
    }
    *state.create_async_header.lock().unwrap() = headers
        .get("X-DashScope-Async")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    *state.generation_body.lock().unwrap() = Some(payload);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "request_id":"req-aliyun-image-1",
            "output":{
                "task_id":"task_aliyun_image_1",
                "task_status":"PENDING"
            }
        })),
    )
}

async fn upstream_aliyun_image_task_handler(
    State(state): State<AliyunImageUpstreamCaptureState>,
    Path(task_id): Path<String>,
    headers: axum::http::HeaderMap,
) -> (StatusCode, Json<Value>) {
    if let Some(authorization) = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
    {
        state
            .authorizations
            .lock()
            .unwrap()
            .push(authorization.to_owned());
    }
    state.task_ids.lock().unwrap().push(task_id.clone());

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "request_id":"req-aliyun-task-1",
            "output":{
                "task_id":task_id,
                "task_status":"SUCCEEDED",
                "results":[
                    {"url":"https://cdn.example.com/task_aliyun_image_1.png"}
                ]
            }
        })),
    )
}

async fn upstream_volcengine_image_generation_handler(
    State(state): State<VolcengineImageUpstreamCaptureState>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<Value>,
) -> (StatusCode, Json<Value>) {
    if let Some(authorization) = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
    {
        state
            .authorizations
            .lock()
            .unwrap()
            .push(authorization.to_owned());
    }
    *state.generation_body.lock().unwrap() = Some(payload);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "created":1744891200,
            "data":[
                {"url":"https://cdn.example.com/volcengine-image-1.png"}
            ]
        })),
    )
}

async fn upstream_generic_kling_generation_handler(
    State(state): State<GenericImageUpstreamCaptureState>,
) -> (StatusCode, Json<Value>) {
    *state.hits.lock().unwrap() += 1;
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "request_id":"req-generic-image-1",
            "output":{
                "task_id":"task_generic_image_1",
                "task_status":"PENDING"
            }
        })),
    )
}

async fn upstream_generic_volcengine_image_generation_handler(
    State(state): State<GenericImageUpstreamCaptureState>,
) -> (StatusCode, Json<Value>) {
    *state.hits.lock().unwrap() += 1;
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "data":[{"url":"https://cdn.example.com/generic-should-not-be-used.png"}]
        })),
    )
}

async fn upstream_generic_kling_task_handler(
    State(state): State<GenericImageUpstreamCaptureState>,
    Path(task_id): Path<String>,
) -> (StatusCode, Json<Value>) {
    *state.hits.lock().unwrap() += 1;
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "request_id":"req-generic-task-1",
            "output":{
                "task_id":task_id,
                "task_status":"SUCCEEDED"
            }
        })),
    )
}

async fn assert_invalid_image_generation_request(response: axum::response::Response) {
    assert_invalid_image_backend_request(response, "Image generation model is required.").await;
}

async fn assert_invalid_image_backend_request(response: axum::response::Response, message: &str) {
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = read_json(response).await;
    assert_eq!(json["error"]["message"], message);
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "invalid_image_request");
}
