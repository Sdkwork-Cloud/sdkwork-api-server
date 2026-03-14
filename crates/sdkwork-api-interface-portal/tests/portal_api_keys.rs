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
    sdkwork_api_storage_sqlite::run_migrations("sqlite::memory:")
        .await
        .unwrap()
}

async fn portal_token(app: axum::Router) -> String {
    let register_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"email\":\"portal@example.com\",\"password\":\"hunter2!\",\"display_name\":\"Portal User\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(register_response.status(), StatusCode::CREATED);
    read_json(register_response).await["token"]
        .as_str()
        .unwrap()
        .to_owned()
}

#[tokio::test]
async fn portal_workspace_and_api_key_self_service_are_scoped() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);
    let token = portal_token(app.clone()).await;

    let workspace_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/workspace")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(workspace_response.status(), StatusCode::OK);
    let workspace_json = read_json(workspace_response).await;
    assert_eq!(workspace_json["user"]["email"], "portal@example.com");
    assert!(workspace_json["tenant"]["id"]
        .as_str()
        .unwrap()
        .starts_with("tenant_"));
    assert!(workspace_json["project"]["id"]
        .as_str()
        .unwrap()
        .starts_with("project_"));

    let create_key_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/api-keys")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"environment\":\"live\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_key_response.status(), StatusCode::CREATED);
    let create_key_json = read_json(create_key_response).await;
    assert!(create_key_json["plaintext"]
        .as_str()
        .unwrap()
        .starts_with("skw_live_"));

    let list_keys_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/api-keys")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(list_keys_response.status(), StatusCode::OK);
    let list_keys_json = read_json(list_keys_response).await;
    assert_eq!(list_keys_json.as_array().unwrap().len(), 1);
    assert_eq!(list_keys_json[0]["environment"], "live");
    assert!(list_keys_json[0].get("plaintext").is_none());
}

#[tokio::test]
async fn portal_routes_require_authentication() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);

    let workspace_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/workspace")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(workspace_response.status(), StatusCode::UNAUTHORIZED);

    let api_keys_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/api-keys")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(api_keys_response.status(), StatusCode::UNAUTHORIZED);
}
