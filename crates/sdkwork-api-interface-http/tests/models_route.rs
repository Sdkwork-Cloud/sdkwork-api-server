use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::routing::get;
use axum::{Json, Router};
use sdkwork_api_app_credential::CredentialSecretManager;
use sdkwork_api_app_gateway::{
    clear_capability_catalog_cache_store, configure_capability_catalog_cache_store,
};
use sdkwork_api_app_identity::hash_gateway_api_key;
use sdkwork_api_cache_core::CacheStore;
use sdkwork_api_cache_memory::MemoryCacheStore;
use sdkwork_api_cache_redis::RedisCacheStore;
use sdkwork_api_domain_catalog::{Channel, ModelCatalogEntry, ProxyProvider};
use sdkwork_api_domain_identity::GatewayApiKeyRecord;
use sdkwork_api_storage_core::{AdminStore, Reloadable};
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use serde_json::Value;
use serial_test::serial;
use sqlx::SqlitePool;
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

mod support;

struct CapabilityCatalogCacheResetGuard;

impl Drop for CapabilityCatalogCacheResetGuard {
    fn drop(&mut self) {
        clear_capability_catalog_cache_store();
    }
}

fn capability_catalog_cache_reset_guard() -> CapabilityCatalogCacheResetGuard {
    clear_capability_catalog_cache_store();
    CapabilityCatalogCacheResetGuard
}

#[tokio::test]
async fn models_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/models")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn model_retrieve_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/models/gpt-4.1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn model_retrieve_route_returns_not_found_for_unknown_model() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/models/gpt-missing")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let json = read_json(response).await;
    assert_eq!(json["error"]["message"], "Requested model was not found.");
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "not_found");
}

#[tokio::test]
async fn model_delete_route_returns_ok() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/models/ft:gpt-4.1:sdkwork")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn model_delete_route_returns_not_found_for_unknown_model() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/models/ft:gpt-missing:sdkwork")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let json = read_json(response).await;
    assert_eq!(json["error"]["message"], "Requested model was not found.");
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "not_found");
}

#[derive(Clone, Default)]
struct UpstreamCaptureState {
    authorization: Arc<Mutex<Option<String>>>,
}

#[tokio::test]
async fn stateless_models_route_relays_to_openai_compatible_provider_when_configured() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/models", get(upstream_models_list_handler))
        .route("/v1/models/gpt-4.1", get(upstream_model_retrieve_handler))
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

    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/models")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(list_response.status(), StatusCode::OK);
    let list_json = read_json(list_response).await;
    assert_eq!(list_json["object"], "list");
    assert_eq!(list_json["data"][0]["id"], "gpt-4.1");
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-stateless-openai")
    );

    let retrieve_response = app
        .oneshot(
            Request::builder()
                .uri("/v1/models/gpt-4.1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(retrieve_response.status(), StatusCode::OK);
    let retrieve_json = read_json(retrieve_response).await;
    assert_eq!(retrieve_json["id"], "gpt-4.1");
}

#[tokio::test]
async fn stateless_models_route_falls_back_when_runtime_key_is_unknown() {
    let app = sdkwork_api_interface_http::gateway_router_with_stateless_config(
        sdkwork_api_interface_http::StatelessGatewayConfig::default().with_upstream(
            sdkwork_api_interface_http::StatelessGatewayUpstream::new(
                "missing-runtime",
                "http://127.0.0.1:1",
                "sk-unused",
            ),
        ),
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/models")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["object"], "list");
    assert_eq!(json["data"][0]["object"], "model");
}

#[tokio::test]
async fn models_route_returns_openai_error_envelope_on_upstream_failure() {
    let app = sdkwork_api_interface_http::gateway_router_with_stateless_config(
        sdkwork_api_interface_http::StatelessGatewayConfig::default().with_upstream(
            sdkwork_api_interface_http::StatelessGatewayUpstream::from_adapter_kind(
                "openai",
                "http://127.0.0.1:1",
                "sk-stateless-openai",
            ),
        ),
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/models")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    let json = read_json(response).await;
    assert_eq!(
        json["error"]["message"],
        "failed to relay upstream model list"
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

async fn seed_openai_provider(store: &SqliteAdminStore) {
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
}

async fn seeded_gateway_store(model_id: &str, api_key: &str) -> Arc<dyn AdminStore> {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool);
    store
        .insert_gateway_api_key(&GatewayApiKeyRecord::new(
            "tenant-1",
            "project-1",
            "live",
            hash_gateway_api_key(api_key),
        ))
        .await
        .unwrap();
    seed_openai_provider(&store).await;
    store
        .insert_model(&ModelCatalogEntry::new(
            model_id,
            "provider-openai-official",
        ))
        .await
        .unwrap();
    Arc::new(store)
}

#[tokio::test]
async fn models_route_reads_persisted_catalog_models() {
    let pool = memory_pool().await;
    seed_openai_provider(&SqliteAdminStore::new(pool.clone())).await;
    let api_key = support::issue_gateway_api_key(&pool, "tenant-1", "project-1").await;
    let app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());

    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let create = admin_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-openai-official\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create.status(), StatusCode::CREATED);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/models")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["data"][0]["id"], "gpt-4.1");
}

#[tokio::test]
#[serial]
async fn models_route_refreshes_after_admin_catalog_mutation_invalidates_capability_cache() {
    let _cache_guard = capability_catalog_cache_reset_guard();
    let cache_store: Arc<dyn CacheStore> = Arc::new(MemoryCacheStore::default());
    configure_capability_catalog_cache_store(cache_store);

    let pool = memory_pool().await;
    seed_openai_provider(&SqliteAdminStore::new(pool.clone())).await;
    let api_key =
        support::issue_gateway_api_key(&pool, "tenant-cache-memory", "project-cache-memory").await;
    let app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());

    let initial_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/models")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(initial_response.status(), StatusCode::OK);
    let initial_json = read_json(initial_response).await;
    assert_eq!(initial_json["data"], serde_json::json!([]));

    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let create = admin_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-openai-official\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create.status(), StatusCode::CREATED);

    let refreshed_response = app
        .oneshot(
            Request::builder()
                .uri("/v1/models")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(refreshed_response.status(), StatusCode::OK);
    let refreshed_json = read_json(refreshed_response).await;
    assert_eq!(refreshed_json["data"][0]["id"], "gpt-4.1");
}

#[tokio::test]
#[serial]
async fn models_route_refreshes_after_admin_catalog_mutation_invalidates_shared_redis_capability_cache(
) {
    let _cache_guard = capability_catalog_cache_reset_guard();
    let redis_server = support::FakeRedisServer::start();
    let redis_url = redis_server.url_with_db(7);
    let gateway_cache_store: Arc<dyn CacheStore> =
        Arc::new(RedisCacheStore::connect(&redis_url).await.unwrap());
    configure_capability_catalog_cache_store(gateway_cache_store.clone());

    let pool = memory_pool().await;
    seed_openai_provider(&SqliteAdminStore::new(pool.clone())).await;
    let api_key =
        support::issue_gateway_api_key(&pool, "tenant-cache-redis", "project-cache-redis").await;
    let app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());

    let initial_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/models")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(initial_response.status(), StatusCode::OK);
    let initial_json = read_json(initial_response).await;
    assert_eq!(initial_json["data"], serde_json::json!([]));

    let admin_cache_store: Arc<dyn CacheStore> =
        Arc::new(RedisCacheStore::connect(&redis_url).await.unwrap());
    configure_capability_catalog_cache_store(admin_cache_store);

    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let create = admin_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-openai-official\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create.status(), StatusCode::CREATED);

    configure_capability_catalog_cache_store(gateway_cache_store);

    let refreshed_response = app
        .oneshot(
            Request::builder()
                .uri("/v1/models")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(refreshed_response.status(), StatusCode::OK);
    let refreshed_json = read_json(refreshed_response).await;
    assert_eq!(refreshed_json["data"][0]["id"], "gpt-4.1");
}

#[tokio::test]
#[serial]
async fn gateway_router_uses_replaced_live_store_for_new_requests() {
    let _cache_guard = capability_catalog_cache_reset_guard();
    let api_key = "skw_live_reloadable_models";
    let live_store = Reloadable::new(seeded_gateway_store("gpt-4.1-old", api_key).await);
    let app = sdkwork_api_interface_http::gateway_router_with_state(
        sdkwork_api_interface_http::GatewayApiState::with_live_store_and_secret_manager(
            live_store.clone(),
            CredentialSecretManager::database_encrypted("local-dev-master-key"),
        ),
    );

    let first_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/models")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(first_response.status(), StatusCode::OK);
    let first_json = read_json(first_response).await;
    assert_eq!(first_json["data"][0]["id"], "gpt-4.1-old");

    live_store.replace(seeded_gateway_store("gpt-4.1-new", api_key).await);

    let second_response = app
        .oneshot(
            Request::builder()
                .uri("/v1/models")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(second_response.status(), StatusCode::OK);
    let second_json = read_json(second_response).await;
    assert_eq!(second_json["data"][0]["id"], "gpt-4.1-new");
}

#[tokio::test]
async fn model_retrieve_route_reads_persisted_catalog_model() {
    let pool = memory_pool().await;
    seed_openai_provider(&SqliteAdminStore::new(pool.clone())).await;
    let api_key = support::issue_gateway_api_key(&pool, "tenant-1", "project-1").await;
    let app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());

    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let create = admin_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-openai-official\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create.status(), StatusCode::CREATED);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/models/gpt-4.1")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["id"], "gpt-4.1");
}

#[tokio::test]
async fn model_delete_route_removes_persisted_catalog_model() {
    let pool = memory_pool().await;
    seed_openai_provider(&SqliteAdminStore::new(pool.clone())).await;
    let api_key = support::issue_gateway_api_key(&pool, "tenant-1", "project-1").await;
    let app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());

    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let create = admin_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"ft:gpt-4.1:sdkwork\",\"provider_id\":\"provider-openai-official\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create.status(), StatusCode::CREATED);

    let delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/models/ft:gpt-4.1:sdkwork")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(delete_response.status(), StatusCode::OK);
    let json = read_json(delete_response).await;
    assert_eq!(json["id"], "ft:gpt-4.1:sdkwork");
    assert_eq!(json["deleted"], true);

    let retrieve_response = app
        .oneshot(
            Request::builder()
                .uri("/v1/models/ft:gpt-4.1:sdkwork")
                .header("authorization", format!("Bearer {api_key}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(retrieve_response.status(), StatusCode::NOT_FOUND);
}

async fn upstream_models_list_handler(
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
                "id":"gpt-4.1",
                "object":"model",
                "created":1710000000,
                "owned_by":"openai"
            }]
        })),
    )
}

async fn upstream_model_retrieve_handler(
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
            "id":"gpt-4.1",
            "object":"model",
            "created":1710000000,
            "owned_by":"openai"
        })),
    )
}
