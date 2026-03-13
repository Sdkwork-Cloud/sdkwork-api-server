use axum::body::Body;
use axum::http::{Request, StatusCode};
use sdkwork_api_app_identity::persist_gateway_api_key;
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use serde_json::Value;
use sqlx::SqlitePool;
use tower::ServiceExt;

mod support;

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

#[tokio::test]
async fn stateful_gateway_requires_api_key_and_uses_request_context() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    let created = persist_gateway_api_key(&store, "tenant-live", "project-live", "live")
        .await
        .unwrap();

    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let admin_token = support::issue_admin_token(admin_app.clone()).await;

    let unauthorized = gateway_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"hi\"}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);

    let authorized = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {}", created.plaintext))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"hi\"}]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(authorized.status(), StatusCode::OK);

    let ledger = admin_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/billing/ledger")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(ledger.status(), StatusCode::OK);
    let ledger_json = read_json(ledger).await;
    assert_eq!(ledger_json[0]["project_id"], "project-live");
}

#[tokio::test]
async fn stateful_moderations_route_requires_gateway_api_key() {
    let pool = memory_pool().await;
    let _api_key = support::issue_gateway_api_key(&pool, "tenant-live", "project-live").await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let unauthorized = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/moderations")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"omni-moderation-latest\",\"input\":\"hi\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn stateful_images_generation_route_requires_gateway_api_key() {
    let pool = memory_pool().await;
    let _api_key = support::issue_gateway_api_key(&pool, "tenant-live", "project-live").await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let unauthorized = gateway_app
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

    assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);
}
