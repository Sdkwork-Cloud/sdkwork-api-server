use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use sdkwork_api_config::HttpExposureConfig;
use sdkwork_api_storage_core::{AdminStore, Reloadable};
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
async fn portal_register_login_and_me_flow_works() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);

    let register_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"email\":\"portal@example.com\",\"password\":\"PortalPass123!\",\"display_name\":\"Portal User\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(register_response.status(), StatusCode::CREATED);
    let register_json = read_json(register_response).await;
    assert_eq!(register_json["user"]["email"], "portal@example.com");
    assert_eq!(register_json["user"]["display_name"], "Portal User");
    assert!(register_json["token"].as_str().unwrap().len() > 10);
    assert!(register_json["workspace"]["tenant_id"]
        .as_str()
        .unwrap()
        .starts_with("tenant_"));
    assert!(register_json["workspace"]["project_id"]
        .as_str()
        .unwrap()
        .starts_with("project_"));

    let duplicate_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"email\":\"portal@example.com\",\"password\":\"PortalPass123!\",\"display_name\":\"Portal User\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(duplicate_response.status(), StatusCode::CONFLICT);

    let login_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"email\":\"portal@example.com\",\"password\":\"PortalPass123!\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(login_response.status(), StatusCode::OK);
    let login_json = read_json(login_response).await;
    let token = login_json["token"].as_str().unwrap().to_owned();

    let me_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/auth/me")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(me_response.status(), StatusCode::OK);
    let me_json = read_json(me_response).await;
    assert_eq!(me_json["email"], "portal@example.com");
    assert_eq!(me_json["display_name"], "Portal User");
}

#[tokio::test]
async fn portal_login_preflight_includes_cors_headers() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/portal/auth/login")
                .header("origin", "http://localhost:5174")
                .header("access-control-request-method", "POST")
                .header("access-control-request-headers", "content-type")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success());
    assert_eq!(
        response
            .headers()
            .get("access-control-allow-origin")
            .and_then(|value| value.to_str().ok()),
        Some("http://localhost:5174")
    );
    assert!(response
        .headers()
        .get("access-control-allow-methods")
        .is_some());
}

#[tokio::test]
async fn portal_login_preflight_ignores_invalid_configured_cors_origins() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_state_and_http_exposure(
        sdkwork_api_interface_portal::PortalApiState::with_store_and_jwt_secret(
            Arc::new(SqliteAdminStore::new(pool)),
            "portal-test-secret",
        ),
        HttpExposureConfig {
            metrics_bearer_token: "portal-metrics-token".to_owned(),
            browser_allowed_origins: vec![
                "https://console.example.com".to_owned(),
                "https://bad\norigin.example.com".to_owned(),
            ],
        },
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/portal/auth/login")
                .header("origin", "https://console.example.com")
                .header("access-control-request-method", "POST")
                .header("access-control-request-headers", "content-type")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_success());
    assert_eq!(
        response
            .headers()
            .get("access-control-allow-origin")
            .and_then(|value| value.to_str().ok()),
        Some("https://console.example.com")
    );
}

#[tokio::test]
async fn portal_metrics_route_requires_bearer_token() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);

    let unauthorized = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);

    let authorized = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/metrics")
                .header("authorization", "Bearer local-dev-metrics-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(authorized.status(), StatusCode::OK);
}

#[tokio::test]
async fn portal_login_rejects_invalid_password() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);

    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"email\":\"portal@example.com\",\"password\":\"PortalPass123!\",\"display_name\":\"Portal User\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"email\":\"portal@example.com\",\"password\":\"wrong-password\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn portal_default_login_bootstraps_the_demo_account() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"email\":\"portal@sdkwork.local\",\"password\":\"ChangeMe123!\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["user"]["email"], "portal@sdkwork.local");
    assert_eq!(json["user"]["display_name"], "Portal Demo");
}

#[tokio::test]
async fn portal_password_change_rotates_the_login_secret() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);

    let login = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"email\":\"portal@sdkwork.local\",\"password\":\"ChangeMe123!\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(login.status(), StatusCode::OK);
    let token = read_json(login).await["token"].as_str().unwrap().to_owned();

    let changed = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/auth/change-password")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"current_password\":\"ChangeMe123!\",\"new_password\":\"PortalPassword456!\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(changed.status(), StatusCode::OK);

    let old_login = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"email\":\"portal@sdkwork.local\",\"password\":\"ChangeMe123!\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(old_login.status(), StatusCode::UNAUTHORIZED);

    let new_login = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"email\":\"portal@sdkwork.local\",\"password\":\"PortalPassword456!\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(new_login.status(), StatusCode::OK);
}

#[tokio::test]
async fn portal_routes_apply_rotated_live_jwt_secret_to_new_requests() {
    let pool = memory_pool().await;
    let live_store = Reloadable::new(Arc::new(SqliteAdminStore::new(pool)) as Arc<dyn AdminStore>);
    let live_jwt = Reloadable::new("initial-portal-jwt-secret".to_owned());
    let app = sdkwork_api_interface_portal::portal_router_with_state(
        sdkwork_api_interface_portal::PortalApiState::with_live_store_and_jwt_secret_handle(
            live_store,
            live_jwt.clone(),
        ),
    );

    let register_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"email\":\"rotate@example.com\",\"password\":\"PortalPass123!\",\"display_name\":\"Rotate User\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(register_response.status(), StatusCode::CREATED);
    let initial_token = read_json(register_response).await["token"]
        .as_str()
        .unwrap()
        .to_owned();

    let initially_authorized = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/auth/me")
                .header("authorization", format!("Bearer {initial_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(initially_authorized.status(), StatusCode::OK);

    live_jwt.replace("rotated-portal-jwt-secret".to_owned());

    let rejected = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/auth/me")
                .header("authorization", format!("Bearer {initial_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rejected.status(), StatusCode::UNAUTHORIZED);

    let rotated_login = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"email\":\"rotate@example.com\",\"password\":\"PortalPass123!\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rotated_login.status(), StatusCode::OK);
    let rotated_token = read_json(rotated_login).await["token"]
        .as_str()
        .unwrap()
        .to_owned();

    let rotated_authorized = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/auth/me")
                .header("authorization", format!("Bearer {rotated_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rotated_authorized.status(), StatusCode::OK);
}
