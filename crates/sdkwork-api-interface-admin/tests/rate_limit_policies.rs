use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use serde_json::Value;
use sqlx::SqlitePool;
use tower::ServiceExt;

async fn read_json(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn memory_pool() -> SqlitePool {
    let pool = sdkwork_api_storage_sqlite::run_migrations("sqlite::memory:")
        .await
        .unwrap();
    let store = sdkwork_api_storage_sqlite::SqliteAdminStore::new(pool.clone());
    sdkwork_api_app_identity::upsert_admin_user(
        &store,
        Some("admin_local_default"),
        "admin@sdkwork.local",
        "Admin Operator",
        Some("ChangeMe123!"),
        Some(sdkwork_api_domain_identity::AdminUserRole::SuperAdmin),
        true,
    )
    .await
    .unwrap();
    pool
}

#[tokio::test]
async fn create_and_list_rate_limit_policies_from_admin_api() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = issue_admin_token(app.clone()).await;

    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/gateway/rate-limit-policies")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"policy_id\":\"rate-project-1\",\"project_id\":\"project-1\",\"requests_per_window\":60,\"window_seconds\":60,\"burst_requests\":120,\"enabled\":true}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create.status(), StatusCode::CREATED);

    let list = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/gateway/rate-limit-policies")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(list.status(), StatusCode::OK);
    let json = read_json(list).await;
    assert_eq!(json.as_array().unwrap().len(), 1);
    assert_eq!(json[0]["policy_id"], "rate-project-1");
    assert_eq!(json[0]["project_id"], "project-1");
    assert_eq!(json[0]["requests_per_window"], 60);
    assert_eq!(json[0]["burst_requests"], 120);
}

async fn issue_admin_token(app: axum::Router) -> String {
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"email\":\"admin@sdkwork.local\",\"password\":\"ChangeMe123!\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    json["token"].as_str().unwrap().to_owned()
}
