use axum::body::to_bytes;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use sdkwork_api_app_credential::CredentialSecretManager;
use sdkwork_api_secret_core::{master_key_id, SecretBackendKind};
use sdkwork_api_storage_core::{AdminStore, Reloadable};
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use serde_json::Value;
use serial_test::serial;
use sqlx::SqlitePool;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tower::ServiceExt;

async fn read_json(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn read_body(response: axum::response::Response) -> String {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}

fn metric_counter_value(body: &str, metric_key: &str) -> u64 {
    body.lines()
        .find_map(|line| {
            line.strip_prefix(metric_key)
                .map(str::trim)
                .and_then(|value| value.parse::<u64>().ok())
        })
        .unwrap_or(0)
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
#[serial]
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
                .body(Body::from(
                    "{\"email\":\"admin@sdkwork.local\",\"password\":\"ChangeMe123!\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(login.status(), StatusCode::OK);
    let login_request_id = login
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .unwrap()
        .to_owned();
    assert!(login_request_id.starts_with("sdkw-"));
    let login_json = read_json(login).await;
    let token = login_json["token"].as_str().unwrap().to_owned();

    let authorized = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/projects")
                .header("authorization", format!("Bearer {token}"))
                .header("x-request-id", "admin-caller-id")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(authorized.status(), StatusCode::OK);
    assert_eq!(
        authorized
            .headers()
            .get("x-request-id")
            .and_then(|value| value.to_str().ok())
            .unwrap(),
        "admin-caller-id"
    );
}

#[tokio::test]
#[serial]
async fn admin_openapi_inventory_routes_stay_public_without_bearer_token() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);

    let openapi = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(openapi.status(), StatusCode::OK);
    let openapi_json = read_json(openapi).await;
    assert_eq!(openapi_json["info"]["title"], "SDKWORK Admin API");

    let docs = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/docs")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(docs.status(), StatusCode::OK);
    let docs_body = read_body(docs).await;
    assert!(docs_body.contains("SDKWORK Admin API"));

    let docs_ui = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/docs/ui/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(docs_ui.status(), StatusCode::OK);
}

#[tokio::test]
#[serial]
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
                .body(Body::from(
                    "{\"email\":\"admin@sdkwork.local\",\"password\":\"ChangeMe123!\"}",
                ))
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
                .body(Body::from(
                    "{\"email\":\"admin@sdkwork.local\",\"password\":\"ChangeMe123!\"}",
                ))
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

#[tokio::test]
#[serial]
async fn admin_metrics_route_reports_login_and_authenticated_requests() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let login_metric_key =
        "sdkwork_http_requests_total{service=\"admin\",method=\"POST\",route=\"/admin/auth/login\",status=\"200\"}";
    let projects_metric_key =
        "sdkwork_http_requests_total{service=\"admin\",method=\"GET\",route=\"/admin/projects\",status=\"200\"}";

    let initial_metrics = app
        .clone()
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
    assert_eq!(initial_metrics.status(), StatusCode::OK);
    let initial_body = read_body(initial_metrics).await;
    let initial_login_total = metric_counter_value(&initial_body, login_metric_key);
    let initial_projects_total = metric_counter_value(&initial_body, projects_metric_key);

    let login = app
        .clone()
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
    assert_eq!(login.status(), StatusCode::OK);
    let token = read_json(login).await["token"].as_str().unwrap().to_owned();

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
    assert_eq!(projects.status(), StatusCode::OK);

    let unauthorized_metrics = app
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
    assert_eq!(unauthorized_metrics.status(), StatusCode::UNAUTHORIZED);

    let metrics = app
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
    assert_eq!(metrics.status(), StatusCode::OK);

    let body = read_body(metrics).await;
    assert!(body.contains("sdkwork_service_info{service=\"admin\"} 1"));
    assert_eq!(
        metric_counter_value(&body, login_metric_key),
        initial_login_total + 1
    );
    assert_eq!(
        metric_counter_value(&body, projects_metric_key),
        initial_projects_total + 1
    );
}

#[tokio::test]
#[serial]
async fn admin_routes_apply_rotated_live_jwt_secret_to_new_requests() {
    let pool = memory_pool().await;
    let live_store = Reloadable::new(Arc::new(SqliteAdminStore::new(pool)) as Arc<dyn AdminStore>);
    let live_jwt = Reloadable::new("initial-admin-jwt-secret".to_owned());
    let app = sdkwork_api_interface_admin::admin_router_with_state(
        sdkwork_api_interface_admin::AdminApiState::with_live_store_and_secret_manager_and_jwt_secret_handle(
            live_store,
            CredentialSecretManager::database_encrypted("local-dev-master-key"),
            live_jwt.clone(),
        ),
    );

    let initial_login = app
        .clone()
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
    assert_eq!(initial_login.status(), StatusCode::OK);
    let initial_token = read_json(initial_login).await["token"]
        .as_str()
        .unwrap()
        .to_owned();

    let initially_authorized = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/projects")
                .header("authorization", format!("Bearer {initial_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(initially_authorized.status(), StatusCode::OK);

    live_jwt.replace("rotated-admin-jwt-secret".to_owned());

    let rejected = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/projects")
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
                .uri("/admin/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"email\":\"admin@sdkwork.local\",\"password\":\"ChangeMe123!\"}",
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
                .uri("/admin/projects")
                .header("authorization", format!("Bearer {rotated_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rotated_authorized.status(), StatusCode::OK);
}

#[tokio::test]
#[serial]
async fn admin_routes_apply_replaced_live_secret_manager_to_new_requests() {
    let pool = memory_pool().await;
    let store = Arc::new(SqliteAdminStore::new(pool));
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let initial_path = std::env::temp_dir().join(format!(
        "sdkwork-admin-secret-manager-initial-{}-{unique}.json",
        std::process::id()
    ));
    let rotated_path = std::env::temp_dir().join(format!(
        "sdkwork-admin-secret-manager-rotated-{}-{unique}.json",
        std::process::id()
    ));
    let live_store = Reloadable::new(store.clone() as Arc<dyn AdminStore>);
    let live_secret_manager =
        Reloadable::new(CredentialSecretManager::new_with_legacy_master_keys(
            SecretBackendKind::LocalEncryptedFile,
            "initial-master-key",
            Vec::new(),
            &initial_path,
            "sdkwork-api-server",
        ));
    let live_jwt = Reloadable::new("initial-admin-jwt-secret".to_owned());
    let app = sdkwork_api_interface_admin::admin_router_with_state(
        sdkwork_api_interface_admin::AdminApiState::with_live_store_and_secret_manager_handle_and_jwt_secret_handle(
            live_store,
            live_secret_manager.clone(),
            live_jwt,
        ),
    );

    let login = app
        .clone()
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
    assert_eq!(login.status(), StatusCode::OK);
    let token = read_json(login).await["token"].as_str().unwrap().to_owned();

    live_secret_manager.replace(CredentialSecretManager::new_with_legacy_master_keys(
        SecretBackendKind::LocalEncryptedFile,
        "rotated-master-key",
        vec!["initial-master-key".to_owned()],
        &rotated_path,
        "sdkwork-api-server",
    ));

    let create = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"tenant_id\":\"tenant-1\",\"provider_id\":\"provider-openai-official\",\"key_reference\":\"cred-openai\",\"secret_value\":\"sk-upstream-openai\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::CREATED);

    let credential = store
        .find_credential("tenant-1", "provider-openai-official", "cred-openai")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(credential.secret_backend, "local_encrypted_file");
    assert_eq!(
        credential.secret_local_file.as_deref(),
        Some(rotated_path.to_string_lossy().as_ref())
    );
    assert_eq!(
        credential.secret_master_key_id.as_deref(),
        Some(master_key_id("rotated-master-key").as_str())
    );

    let _ = std::fs::remove_file(initial_path);
    let _ = std::fs::remove_file(rotated_path);
}

#[tokio::test]
#[serial]
async fn admin_password_change_requires_current_password_and_rotates_login_secret() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);

    let login = app
        .clone()
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
    assert_eq!(login.status(), StatusCode::OK);
    let token = read_json(login).await["token"].as_str().unwrap().to_owned();

    let changed = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/auth/change-password")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"current_password\":\"ChangeMe123!\",\"new_password\":\"AdminPassword456!\"}",
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
                .uri("/admin/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"email\":\"admin@sdkwork.local\",\"password\":\"ChangeMe123!\"}",
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
                .uri("/admin/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"email\":\"admin@sdkwork.local\",\"password\":\"AdminPassword456!\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(new_login.status(), StatusCode::OK);
}
