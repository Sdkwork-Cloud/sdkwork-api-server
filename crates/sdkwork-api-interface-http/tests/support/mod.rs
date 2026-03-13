use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use sdkwork_api_app_identity::persist_gateway_api_key;
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use sqlx::SqlitePool;
use tower::ServiceExt;

#[allow(dead_code)]
pub async fn issue_gateway_api_key(pool: &SqlitePool, tenant_id: &str, project_id: &str) -> String {
    let store = SqliteAdminStore::new(pool.clone());
    persist_gateway_api_key(&store, tenant_id, project_id, "live")
        .await
        .unwrap()
        .plaintext
}

#[allow(dead_code)]
pub async fn issue_admin_token(app: Router) -> String {
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/auth/login")
                .header("content-type", "application/json")
                .body(Body::from("{\"subject\":\"http-test-admin\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    json["token"].as_str().unwrap().to_owned()
}
