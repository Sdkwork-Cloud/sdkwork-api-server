use axum::body::to_bytes;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use serde_json::Value;
use sqlx::SqlitePool;
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

async fn login_token(app: Router) -> String {
    let response = app
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

    assert_eq!(response.status(), StatusCode::OK);
    read_json(response).await["token"]
        .as_str()
        .unwrap()
        .to_owned()
}

#[tokio::test]
async fn login_route_exists() {
    let app = sdkwork_api_interface_admin::admin_router();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/admin/auth/login")
                .method("POST")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_ne!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn create_and_list_tenants_projects_and_api_keys() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let tenant = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/tenants")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"id\":\"tenant-1\",\"name\":\"Tenant One\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(tenant.status(), StatusCode::CREATED);

    let project = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/projects")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"tenant_id\":\"tenant-1\",\"id\":\"project-1\",\"name\":\"Project One\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(project.status(), StatusCode::CREATED);

    let create_key = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/api-keys")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"tenant_id\":\"tenant-1\",\"project_id\":\"project-1\",\"environment\":\"live\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_key.status(), StatusCode::CREATED);
    let created_key_json = read_json(create_key).await;
    assert!(created_key_json["plaintext"]
        .as_str()
        .unwrap()
        .starts_with("skw_live_"));

    let tenants = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/tenants")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let tenants_json = read_json(tenants).await;
    assert_eq!(tenants_json[0]["id"], "tenant-1");

    let projects = app
        .clone()
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
    let projects_json = read_json(projects).await;
    assert_eq!(projects_json[0]["tenant_id"], "tenant-1");

    let api_keys = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/api-keys")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let api_keys_json = read_json(api_keys).await;
    assert_eq!(api_keys_json[0]["project_id"], "project-1");
    assert_eq!(api_keys_json[0]["environment"], "live");
}
