use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use sdkwork_api_app_identity::{hash_gateway_api_key, resolve_gateway_request_context};
use sdkwork_api_domain_identity::GatewayApiKeyRecord;
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
                .body(Body::from(
                    "{\"environment\":\"live\",\"label\":\"Production rollout\",\"expires_at_ms\":1900000000000}",
                ))
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
    assert_eq!(create_key_json["label"], "Production rollout");
    assert_eq!(create_key_json["expires_at_ms"], 1900000000000_u64);
    assert!(create_key_json["created_at_ms"].as_u64().unwrap() > 0);

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
    assert_eq!(list_keys_json[0]["label"], "Production rollout");
    assert_eq!(list_keys_json[0]["expires_at_ms"], 1900000000000_u64);
    assert!(list_keys_json[0]["created_at_ms"].as_u64().unwrap() > 0);
    assert!(list_keys_json[0]["last_used_at_ms"].is_null());
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

#[tokio::test]
async fn portal_api_key_status_and_delete_are_project_scoped() {
    let pool = memory_pool().await;
    let store = std::sync::Arc::new(sdkwork_api_storage_sqlite::SqliteAdminStore::new(pool));
    let app = sdkwork_api_interface_portal::portal_router_with_store(store.clone());
    let token = portal_token(app.clone()).await;

    let create_key_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/api-keys")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"environment\":\"live\",\"label\":\"Production rollout\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_key_response.status(), StatusCode::CREATED);
    let created_json = read_json(create_key_response).await;
    let plaintext = created_json["plaintext"].as_str().unwrap().to_owned();
    let hashed_key = created_json["hashed"].as_str().unwrap().to_owned();

    let foreign_plaintext = "foreign-plaintext";
    store
        .insert_gateway_api_key(
            &GatewayApiKeyRecord::new(
                "tenant-other",
                "project-other",
                "live",
                hash_gateway_api_key(foreign_plaintext),
            )
            .with_label("Other project key")
            .with_created_at_ms(1_700_000_000_000),
        )
        .await
        .unwrap();

    let revoke_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/portal/api-keys/{hashed_key}/status"))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"active\":false}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(revoke_response.status(), StatusCode::OK);
    let revoke_json = read_json(revoke_response).await;
    assert_eq!(revoke_json["active"], false);

    let request_context = resolve_gateway_request_context(store.as_ref(), &plaintext)
        .await
        .unwrap();
    assert!(request_context.is_none());

    let restore_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/portal/api-keys/{hashed_key}/status"))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"active\":true}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(restore_response.status(), StatusCode::OK);
    assert_eq!(read_json(restore_response).await["active"], true);

    let restored_context = resolve_gateway_request_context(store.as_ref(), &plaintext)
        .await
        .unwrap();
    assert!(restored_context.is_some());

    let foreign_revoke_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/portal/api-keys/{}/status",
                    hash_gateway_api_key(foreign_plaintext)
                ))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"active\":false}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(foreign_revoke_response.status(), StatusCode::NOT_FOUND);

    let delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/portal/api-keys/{hashed_key}"))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    let deleted_context = resolve_gateway_request_context(store.as_ref(), &plaintext)
        .await
        .unwrap();
    assert!(deleted_context.is_none());
}

#[tokio::test]
async fn portal_api_keys_support_custom_plaintext_creation_without_exposing_it_in_lists() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);
    let token = portal_token(app.clone()).await;
    let custom_plaintext = "skw_live_custom_portal_secret";

    let create_key_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/api-keys")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"environment\":\"live\",\"label\":\"Custom live key\",\"notes\":\"Operator-managed migration key\",\"api_key\":\"{custom_plaintext}\",\"expires_at_ms\":1900000000000}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_key_response.status(), StatusCode::CREATED);
    let created_json = read_json(create_key_response).await;
    assert_eq!(created_json["plaintext"], custom_plaintext);
    assert_eq!(created_json["label"], "Custom live key");
    assert_eq!(created_json["environment"], "live");
    assert_eq!(created_json["notes"], "Operator-managed migration key");

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
    assert_eq!(list_keys_json[0]["label"], "Custom live key");
    assert_eq!(list_keys_json[0]["environment"], "live");
    assert_eq!(list_keys_json[0]["notes"], "Operator-managed migration key");
    assert_eq!(list_keys_json[0]["expires_at_ms"], 1900000000000_u64);
    assert!(list_keys_json[0]["hashed_key"].as_str().unwrap().len() > 10);
    assert!(list_keys_json[0].get("plaintext").is_none());
}
