use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::Value;
use serial_test::serial;
use sqlx::SqlitePool;
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

mod support;

#[serial(extension_env)]
#[tokio::test]
async fn local_music_routes_follow_truthful_fallback_contract() {
    let app = sdkwork_api_interface_http::gateway_router();

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/music")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"suno-v4\",\"prompt\":\"Write an uplifting electronic anthem\",\"duration_seconds\":123.0}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_response.status(), StatusCode::OK);
    let create_json = read_json(create_response).await;
    let track_id = create_json["data"][0]["id"].as_str().unwrap().to_owned();
    assert!(track_id.starts_with("music_local_"));
    assert_eq!(create_json["data"][0]["status"], "queued");

    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/music")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_json = read_json(list_response).await;
    assert_eq!(list_json["data"], Value::Array(Vec::new()));

    let retrieve_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/v1/music/{track_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_music_not_found(retrieve_response, "Requested music was not found.").await;

    let content_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/v1/music/{track_id}/content"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_music_not_found(content_response, "Requested music asset was not found.").await;

    let lyrics_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/music/lyrics")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"prompt\":\"Write uplifting synth-pop lyrics\",\"title\":\"Skyline Pulse\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_invalid_music_request(
        lyrics_response,
        "Local music lyrics fallback is not supported without a lyrics generation backend.",
    )
    .await;

    let delete_response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/v1/music/{track_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_response.status(), StatusCode::OK);
    let delete_json = read_json(delete_response).await;
    assert_eq!(delete_json["deleted"], true);
    assert_eq!(delete_json["id"], track_id);
}

#[serial(extension_env)]
#[tokio::test]
async fn music_content_route_returns_not_found_error_for_unknown_track() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/music/music_missing/content")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let json = read_json(response).await;
    assert_eq!(
        json["error"]["message"],
        "Requested music asset was not found."
    );
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "not_found");
}

#[serial(extension_env)]
#[tokio::test]
async fn music_retrieve_route_returns_not_found_error_for_unknown_track() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/music/music_missing")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_music_not_found(response, "Requested music was not found.").await;
}

#[serial(extension_env)]
#[tokio::test]
async fn music_delete_route_returns_not_found_error_for_unknown_track() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/music/music_missing")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_music_not_found(response, "Requested music was not found.").await;
}

#[serial(extension_env)]
#[tokio::test]
async fn stateless_music_routes_relay_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route(
            "/v1/music",
            get(upstream_music_list_handler).post(upstream_music_create_handler),
        )
        .route(
            "/v1/music/music_1",
            get(upstream_music_retrieve_handler).delete(upstream_music_delete_handler),
        )
        .route(
            "/v1/music/music_1/content",
            get(upstream_music_content_handler),
        )
        .route("/v1/music/lyrics", post(upstream_music_lyrics_handler))
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let app = sdkwork_api_interface_http::gateway_router_with_stateless_config(
        sdkwork_api_interface_http::StatelessGatewayConfig::default().with_upstream(
            sdkwork_api_interface_http::StatelessGatewayUpstream::new(
                "openai",
                format!("http://{address}"),
                "sk-stateless-music",
            ),
        ),
    );

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/music")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"suno-v4\",\"prompt\":\"Write an uplifting electronic anthem\",\"duration_seconds\":123.0}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_response.status(), StatusCode::OK);
    let create_json = read_json(create_response).await;
    assert_eq!(create_json["data"][0]["id"], "music_upstream");

    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/music")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_json = read_json(list_response).await;
    assert_eq!(list_json["data"][0]["id"], "music_1");

    let retrieve_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/music/music_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(retrieve_response.status(), StatusCode::OK);
    let retrieve_json = read_json(retrieve_response).await;
    assert_eq!(retrieve_json["id"], "music_1");

    let content_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/music/music_1/content")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(content_response.status(), StatusCode::OK);
    assert_eq!(
        read_bytes(content_response).await,
        b"UPSTREAM-MUSIC".to_vec()
    );

    let lyrics_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/music/lyrics")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"prompt\":\"Write uplifting synth-pop lyrics\",\"title\":\"Skyline Pulse\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(lyrics_response.status(), StatusCode::OK);
    let lyrics_json = read_json(lyrics_response).await;
    assert_eq!(lyrics_json["id"], "lyrics_1");
    assert_eq!(lyrics_json["object"], "music.lyrics");

    let delete_response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/music/music_1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_response.status(), StatusCode::OK);
    let delete_json = read_json(delete_response).await;
    assert_eq!(delete_json["deleted"], true);

    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-stateless-music")
    );
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_music_create_records_music_seconds_in_billing_events() {
    let tenant_id = "tenant-music-billing";
    let project_id = "project-music-billing";
    let request_model = "suno-v4";
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/music", post(upstream_music_create_handler))
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    create_channel(&admin_app, &admin_token).await;
    create_provider(
        &admin_app,
        &admin_token,
        "provider-music",
        &format!("http://{address}"),
        "Music Provider",
    )
    .await;
    create_credential(
        &admin_app,
        &admin_token,
        tenant_id,
        "provider-music",
        "cred-music",
        "sk-upstream-music",
    )
    .await;
    create_model(&admin_app, &admin_token, request_model, "provider-music").await;

    let create_response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/music")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"suno-v4\",\"prompt\":\"Write an uplifting electronic anthem\",\"duration_seconds\":123.0}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_response.status(), StatusCode::OK);
    let create_json = read_json(create_response).await;
    assert_eq!(create_json["data"][0]["id"], "music_upstream");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-music")
    );
    support::assert_single_usage_record_and_decision_log(
        admin_app.clone(),
        &admin_token,
        request_model,
        "provider-music",
        request_model,
    )
    .await;

    let billing_events = admin_app
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
    assert_eq!(billing_json.as_array().unwrap().len(), 1);
    assert_eq!(billing_json[0]["capability"], "music");
    assert_eq!(billing_json[0]["route_key"], request_model);
    assert_eq!(billing_json[0]["usage_model"], request_model);
    assert_eq!(billing_json[0]["provider_id"], "provider-music");
    assert_eq!(billing_json[0]["reference_id"], "music_upstream");
    assert_eq!(billing_json[0]["music_seconds"], 123.0);
}

async fn create_channel(admin_app: &Router, admin_token: &str) {
    let response = admin_app
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
    assert_eq!(response.status(), StatusCode::CREATED);
}

async fn create_provider(
    admin_app: &Router,
    admin_token: &str,
    provider_id: &str,
    base_url: &str,
    display_name: &str,
) {
    let response = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"{provider_id}\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"{base_url}\",\"display_name\":\"{display_name}\"}}"
                )))
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

async fn create_model(
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

#[serial(extension_env)]
#[tokio::test]
async fn stateful_music_retrieve_route_returns_not_found_without_usage() {
    let ctx = local_music_test_context(
        "tenant-music-retrieve-missing",
        "project-music-retrieve-missing",
    )
    .await;

    let response = ctx
        .gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/music/music_missing")
                .header("authorization", format!("Bearer {}", ctx.api_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_music_not_found(response, "Requested music was not found.").await;
    support::assert_no_usage_records(ctx.admin_app, &ctx.admin_token).await;
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_music_delete_route_returns_not_found_without_usage() {
    let ctx = local_music_test_context(
        "tenant-music-delete-missing",
        "project-music-delete-missing",
    )
    .await;

    let response = ctx
        .gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/music/music_missing")
                .header("authorization", format!("Bearer {}", ctx.api_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_music_not_found(response, "Requested music was not found.").await;
    support::assert_no_usage_records(ctx.admin_app, &ctx.admin_token).await;
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_music_content_route_returns_not_found_without_usage() {
    let ctx = local_music_test_context(
        "tenant-music-content-missing",
        "project-music-content-missing",
    )
    .await;

    let response = ctx
        .gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/music/music_missing/content")
                .header("authorization", format!("Bearer {}", ctx.api_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_music_not_found(response, "Requested music asset was not found.").await;
    support::assert_no_usage_records(ctx.admin_app, &ctx.admin_token).await;
}

#[serial(extension_env)]
#[tokio::test]
async fn stateful_music_lyrics_route_returns_invalid_request_without_usage() {
    let ctx = local_music_test_context(
        "tenant-music-lyrics-unsupported",
        "project-music-lyrics-unsupported",
    )
    .await;

    let response = ctx
        .gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/music/lyrics")
                .header("authorization", format!("Bearer {}", ctx.api_key))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"prompt\":\"Write uplifting synth-pop lyrics\",\"title\":\"Skyline Pulse\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_invalid_music_request(
        response,
        "Local music lyrics fallback is not supported without a lyrics generation backend.",
    )
    .await;
    support::assert_no_usage_records(ctx.admin_app, &ctx.admin_token).await;
}

async fn read_json(response: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn assert_music_not_found(response: axum::response::Response, message: &str) {
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let json = read_json(response).await;
    assert_eq!(json["error"]["message"], message);
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "not_found");
}

async fn assert_invalid_music_request(response: axum::response::Response, message: &str) {
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = read_json(response).await;
    assert_eq!(json["error"]["message"], message);
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "invalid_music_request");
}

async fn read_bytes(response: axum::response::Response) -> Vec<u8> {
    axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap()
        .to_vec()
}

async fn memory_pool() -> SqlitePool {
    sdkwork_api_storage_sqlite::run_migrations("sqlite::memory:")
        .await
        .unwrap()
}

struct LocalMusicTestContext {
    admin_app: Router,
    admin_token: String,
    api_key: String,
    gateway_app: Router,
}

async fn local_music_test_context(tenant_id: &str, project_id: &str) -> LocalMusicTestContext {
    let pool = memory_pool().await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let api_key = support::issue_gateway_api_key(&pool, tenant_id, project_id).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    LocalMusicTestContext {
        admin_app,
        admin_token,
        api_key,
        gateway_app,
    }
}

#[derive(Clone, Default)]
struct UpstreamCaptureState {
    authorization: Arc<Mutex<Option<String>>>,
}

async fn upstream_music_create_handler(
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
                "id":"music_upstream",
                "object":"music",
                "status":"completed",
                "model":"suno-v4",
                "audio_url":"https://example.com/music.mp3",
                "duration_seconds":123.0
            }]
        })),
    )
}

async fn upstream_music_list_handler(
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
                "id":"music_1",
                "object":"music",
                "status":"completed",
                "audio_url":"https://example.com/music.mp3"
            }]
        })),
    )
}

async fn upstream_music_retrieve_handler(
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
            "id":"music_1",
            "object":"music",
            "status":"completed",
            "audio_url":"https://example.com/music.mp3"
        })),
    )
}

async fn upstream_music_delete_handler(
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
            "id":"music_1",
            "object":"music.deleted",
            "deleted":true
        })),
    )
}

async fn upstream_music_content_handler(
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
        [(axum::http::header::CONTENT_TYPE, "audio/mpeg")],
        b"UPSTREAM-MUSIC",
    )
}

async fn upstream_music_lyrics_handler(
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
            "id":"lyrics_1",
            "object":"music.lyrics",
            "status":"completed",
            "title":"Skyline Pulse",
            "text":"We rise with the skyline"
        })),
    )
}
