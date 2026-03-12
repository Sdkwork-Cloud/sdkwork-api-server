use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::Value;
use sqlx::SqlitePool;
use tower::ServiceExt;

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
async fn models_route_reads_persisted_catalog_models() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_http::gateway_router_with_pool(pool.clone());

    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let create = admin_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
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
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["data"][0]["id"], "gpt-4.1");
}
