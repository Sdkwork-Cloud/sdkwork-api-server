use axum::body::to_bytes;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use sdkwork_api_app_credential::CredentialSecretManager;
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use serde_json::Value;
use sqlx::SqlitePool;
use std::sync::Arc;
use tower::ServiceExt;

async fn read_json(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn memory_pool() -> SqlitePool {
    sdkwork_api_storage_sqlite::run_migrations("sqlite::memory:")
        .await
        .unwrap()
}

#[tokio::test]
async fn admin_routes_require_valid_bearer_token() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);

    let unauthorized = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/projects")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);

    let login = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/auth/login")
                .header("content-type", "application/json")
                .body(Body::from("{\"subject\":\"admin-1\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(login.status(), StatusCode::OK);
    let login_json = read_json(login).await;
    let token = login_json["token"].as_str().unwrap().to_owned();

    let authorized = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/projects")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(authorized.status(), StatusCode::OK);
}

#[tokio::test]
async fn admin_routes_use_the_configured_jwt_signing_secret() {
    let pool = memory_pool().await;
    let store = Arc::new(SqliteAdminStore::new(pool));
    let default_app = sdkwork_api_interface_admin::admin_router_with_store_and_secret_manager(
        store.clone(),
        CredentialSecretManager::database_encrypted("local-dev-master-key"),
    );
    let custom_app =
        sdkwork_api_interface_admin::admin_router_with_store_and_secret_manager_and_jwt_secret(
            store,
            CredentialSecretManager::database_encrypted("local-dev-master-key"),
            "custom-admin-jwt-secret",
        );

    let default_login = default_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/auth/login")
                .header("content-type", "application/json")
                .body(Body::from("{\"subject\":\"admin-default\"}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(default_login.status(), StatusCode::OK);
    let default_token = read_json(default_login).await["token"]
        .as_str()
        .unwrap()
        .to_owned();

    let rejected = custom_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/projects")
                .header("authorization", format!("Bearer {default_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rejected.status(), StatusCode::UNAUTHORIZED);

    let custom_login = custom_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/auth/login")
                .header("content-type", "application/json")
                .body(Body::from("{\"subject\":\"admin-custom\"}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(custom_login.status(), StatusCode::OK);
    let custom_token = read_json(custom_login).await["token"]
        .as_str()
        .unwrap()
        .to_owned();

    let authorized = custom_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/projects")
                .header("authorization", format!("Bearer {custom_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(authorized.status(), StatusCode::OK);
}
