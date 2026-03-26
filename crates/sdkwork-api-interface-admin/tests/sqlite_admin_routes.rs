use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use axum::Router;
use sdkwork_api_storage_core::ServiceRuntimeNodeRecord;
use serde_json::{json, Value};
use serial_test::serial;
use sqlx::SqlitePool;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tower::ServiceExt;

const FIXTURE_EXTENSION_ID: &str = "sdkwork.provider.native.mock";

use sdkwork_api_extension_core::{ExtensionInstallation, ExtensionInstance, ExtensionRuntime};
#[cfg(windows)]
use sdkwork_api_extension_host::{
    ensure_connector_runtime_started, shutdown_all_connector_runtimes, ExtensionLoadPlan,
};
use sdkwork_api_extension_host::{
    load_native_dynamic_provider_adapter, shutdown_all_native_dynamic_runtimes,
};
#[cfg(windows)]
use std::net::TcpListener;

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
                .body(Body::from(
                    "{\"email\":\"admin@sdkwork.local\",\"password\":\"ChangeMe123!\"}",
                ))
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

async fn create_provider_fixture(app: Router, token: &str, body: &str) {
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(body.to_owned()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}

#[serial(extension_env)]
#[tokio::test]
async fn login_returns_a_gateway_jwt_like_token() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);

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
    let json = read_json(response).await;
    assert_eq!(json["user"]["email"], "admin@sdkwork.local");
    assert_eq!(json["claims"]["sub"], json["user"]["id"]);
    assert_eq!(json["token"].as_str().unwrap().split('.').count(), 3);
}

#[serial(extension_env)]
#[tokio::test]
async fn create_and_list_channels() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/channels")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"id\":\"openai\",\"name\":\"OpenAI\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create.status(), StatusCode::CREATED);

    let list = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/channels")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(list.status(), StatusCode::OK);
    let json = read_json(list).await;
    assert!(json.as_array().unwrap().len() >= 5);
    assert!(json
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["id"] == "openai" && item["name"] == "OpenAI"));
}

#[serial(extension_env)]
#[tokio::test]
async fn builtin_channels_channel_models_and_model_prices_are_exposed_through_admin_api() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let channels = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/channels")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(channels.status(), StatusCode::OK);
    let channels_json = read_json(channels).await;
    assert!(channels_json
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["id"] == "openai"));
    assert!(channels_json
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["id"] == "anthropic"));

    let provider = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"id":"provider-openai-official","channel_id":"openai","adapter_kind":"openai","base_url":"https://api.openai.com","display_name":"OpenAI Official","channel_bindings":[{"channel_id":"openai","is_primary":true}]}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(provider.status(), StatusCode::CREATED);

    let create_model = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/channel-models")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"channel_id":"openai","model_id":"gpt-4.1","model_display_name":"GPT-4.1","capabilities":["responses","chat_completions"],"streaming":true,"context_window":128000}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_model.status(), StatusCode::CREATED);

    let models = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/channel-models")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(models.status(), StatusCode::OK);
    let models_json = read_json(models).await;
    assert!(models_json
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["channel_id"] == "openai" && item["model_id"] == "gpt-4.1"));

    let create_price = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/model-prices")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"channel_id":"openai","model_id":"gpt-4.1","proxy_provider_id":"provider-openai-official","currency_code":"USD","price_unit":"per_1m_tokens","input_price":2.5,"output_price":10.0,"cache_read_price":0.3,"cache_write_price":1.0,"request_price":0.0,"is_active":true}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_price.status(), StatusCode::CREATED);

    let prices = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/model-prices")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(prices.status(), StatusCode::OK);
    let prices_json = read_json(prices).await;
    assert!(prices_json.as_array().unwrap().iter().any(|item| {
        item["channel_id"] == "openai"
            && item["model_id"] == "gpt-4.1"
            && item["proxy_provider_id"] == "provider-openai-official"
    }));
}

#[serial(extension_env)]
#[tokio::test]
async fn list_and_manage_operator_users_from_admin_api() {
    let pool = memory_pool().await;
    let store = sdkwork_api_storage_sqlite::SqliteAdminStore::new(pool.clone());
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let initial_list = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/users/operators")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(initial_list.status(), StatusCode::OK);
    let initial_json = read_json(initial_list).await;
    assert_eq!(initial_json.as_array().unwrap().len(), 1);
    assert_eq!(initial_json[0]["email"], "admin@sdkwork.local");

    let created = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/users/operators")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"email\":\"ops@example.com\",\"display_name\":\"Ops One\",\"password\":\"OperatorPass456!\",\"active\":true}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(created.status(), StatusCode::CREATED);
    let created_json = read_json(created).await;
    let user_id = created_json["id"].as_str().unwrap().to_owned();
    assert_eq!(created_json["email"], "ops@example.com");
    assert_eq!(created_json["active"], true);

    let duplicate = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/users/operators")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"email\":\"ops@example.com\",\"display_name\":\"Ops Duplicate\",\"password\":\"OperatorPass456!\",\"active\":true}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(duplicate.status(), StatusCode::CONFLICT);

    let deactivate = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/admin/users/operators/{user_id}/status"))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"active\":false}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(deactivate.status(), StatusCode::OK);
    assert_eq!(read_json(deactivate).await["active"], false);

    let reactivate = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/admin/users/operators/{user_id}/status"))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"active\":true}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(reactivate.status(), StatusCode::OK);
    assert_eq!(read_json(reactivate).await["active"], true);

    let reset_password = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/admin/users/operators/{user_id}/password"))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"new_password\":\"OperatorPass789!\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(reset_password.status(), StatusCode::OK);
    assert_eq!(read_json(reset_password).await["id"], user_id);

    let old_password_error = sdkwork_api_app_identity::login_admin_user(
        &store,
        "ops@example.com",
        "OperatorPass456!",
        "admin-test-secret",
    )
    .await
    .unwrap_err();
    assert_eq!(old_password_error.to_string(), "invalid email or password");

    let session = sdkwork_api_app_identity::login_admin_user(
        &store,
        "ops@example.com",
        "OperatorPass789!",
        "admin-test-secret",
    )
    .await
    .unwrap();
    assert_eq!(session.user.email, "ops@example.com");

    let delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/admin/users/operators/{user_id}"))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    let deleted_login = sdkwork_api_app_identity::login_admin_user(
        &store,
        "ops@example.com",
        "OperatorPass789!",
        "admin-test-secret",
    )
    .await
    .unwrap_err();
    assert_eq!(deleted_login.to_string(), "invalid email or password");

    let delete_default_admin = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/admin/users/operators/admin_local_default")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(delete_default_admin.status(), StatusCode::CONFLICT);

    let final_list = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/users/operators")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(final_list.status(), StatusCode::OK);
    let final_json = read_json(final_list).await;
    assert_eq!(final_json.as_array().unwrap().len(), 1);
    assert!(!final_json
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["email"] == "ops@example.com"));
}

#[serial(extension_env)]
#[tokio::test]
async fn list_and_manage_portal_users_from_admin_api() {
    let pool = memory_pool().await;
    let store = sdkwork_api_storage_sqlite::SqliteAdminStore::new(pool.clone());
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let initial_list = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/users/portal")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(initial_list.status(), StatusCode::OK);
    let initial_json = read_json(initial_list).await;
    assert_eq!(initial_json.as_array().unwrap().len(), 1);
    assert_eq!(initial_json[0]["email"], "portal@sdkwork.local");
    assert_eq!(initial_json[0]["workspace_tenant_id"], "tenant_local_demo");
    assert_eq!(
        initial_json[0]["workspace_project_id"],
        "project_local_demo"
    );

    let created = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/users/portal")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"email\":\"alice@example.com\",\"display_name\":\"Alice Portal\",\"password\":\"PortalPass456!\",\"workspace_tenant_id\":\"tenant_local_demo\",\"workspace_project_id\":\"project_local_demo\",\"active\":true}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(created.status(), StatusCode::CREATED);
    let created_json = read_json(created).await;
    let user_id = created_json["id"].as_str().unwrap().to_owned();
    assert_eq!(created_json["email"], "alice@example.com");
    assert_eq!(created_json["workspace_project_id"], "project_local_demo");

    let duplicate = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/users/portal")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"email\":\"alice@example.com\",\"display_name\":\"Alice Duplicate\",\"password\":\"PortalPass456!\",\"workspace_tenant_id\":\"tenant_local_demo\",\"workspace_project_id\":\"project_local_demo\",\"active\":true}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(duplicate.status(), StatusCode::CONFLICT);

    let deactivate = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/admin/users/portal/{user_id}/status"))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"active\":false}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(deactivate.status(), StatusCode::OK);
    assert_eq!(read_json(deactivate).await["active"], false);

    let reactivate = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/admin/users/portal/{user_id}/status"))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"active\":true}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(reactivate.status(), StatusCode::OK);
    assert_eq!(read_json(reactivate).await["active"], true);

    let reset_password = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/admin/users/portal/{user_id}/password"))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"new_password\":\"PortalPass789!\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(reset_password.status(), StatusCode::OK);
    assert_eq!(read_json(reset_password).await["id"], user_id);

    let old_password_error = sdkwork_api_app_identity::login_portal_user(
        &store,
        "alice@example.com",
        "PortalPass456!",
        "portal-test-secret",
    )
    .await
    .unwrap_err();
    assert_eq!(old_password_error.to_string(), "invalid email or password");

    let session = sdkwork_api_app_identity::login_portal_user(
        &store,
        "alice@example.com",
        "PortalPass789!",
        "portal-test-secret",
    )
    .await
    .unwrap();
    assert_eq!(session.user.email, "alice@example.com");

    let delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/admin/users/portal/{user_id}"))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    let deleted_login = sdkwork_api_app_identity::login_portal_user(
        &store,
        "alice@example.com",
        "PortalPass789!",
        "portal-test-secret",
    )
    .await
    .unwrap_err();
    assert_eq!(deleted_login.to_string(), "invalid email or password");

    let delete_default_portal = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/admin/users/portal/user_local_demo")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(delete_default_portal.status(), StatusCode::CONFLICT);

    let final_list = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/users/portal")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(final_list.status(), StatusCode::OK);
    let final_json = read_json(final_list).await;
    assert_eq!(final_json.as_array().unwrap().len(), 1);
    assert!(!final_json
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["email"] == "alice@example.com"));
}

#[serial(extension_env)]
#[tokio::test]
async fn create_and_list_providers_and_credentials() {
    let pool = memory_pool().await;
    let store = sdkwork_api_storage_sqlite::SqliteAdminStore::new(pool.clone());
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/channels")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"id\":\"openai\",\"name\":\"OpenAI\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    let provider = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"id\":\"provider-openai-official\",\"channel_id\":\"openai\",\"extension_id\":\"sdkwork.provider.openai.official\",\"channel_bindings\":[{\"channel_id\":\"openai\",\"is_primary\":true},{\"channel_id\":\"responses-compatible\",\"is_primary\":false}],\"adapter_kind\":\"openai\",\"base_url\":\"https://api.openai.com\",\"display_name\":\"OpenAI Official\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(provider.status(), StatusCode::CREATED);

    let credential = app
        .clone()
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

    assert_eq!(credential.status(), StatusCode::CREATED);

    let providers = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let providers_json = read_json(providers).await;
    assert_eq!(providers_json[0]["channel_id"], "openai");
    assert_eq!(
        providers_json[0]["extension_id"],
        "sdkwork.provider.openai.official"
    );
    assert_eq!(providers_json[0]["adapter_kind"], "openai");
    assert_eq!(providers_json[0]["base_url"], "https://api.openai.com");
    assert_eq!(
        providers_json[0]["channel_bindings"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        providers_json[0]["channel_bindings"][1]["channel_id"],
        "responses-compatible"
    );
    assert_eq!(providers_json[0]["channel_bindings"][0]["is_primary"], true);

    let credentials = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let credentials_json = read_json(credentials).await;
    assert_eq!(
        credentials_json[0]["provider_id"],
        "provider-openai-official"
    );
    assert!(credentials_json[0]["secret_value"].is_null());

    let secret = sdkwork_api_app_credential::resolve_credential_secret(
        &store,
        "local-dev-master-key",
        "tenant-1",
        "provider-openai-official",
        "cred-openai",
    )
    .await
    .unwrap();
    assert_eq!(secret, "sk-upstream-openai");
}

#[serial(extension_env)]
#[tokio::test]
async fn delete_credentials_through_admin_api() {
    let pool = memory_pool().await;
    let store = sdkwork_api_storage_sqlite::SqliteAdminStore::new(pool.clone());
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/channels")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"id\":\"openai\",\"name\":\"OpenAI\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"id\":\"provider-openai-official\",\"channel_id\":\"openai\",\"extension_id\":\"sdkwork.provider.openai.official\",\"channel_bindings\":[{\"channel_id\":\"openai\",\"is_primary\":true}],\"adapter_kind\":\"openai\",\"base_url\":\"https://api.openai.com\",\"display_name\":\"OpenAI Official\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let create_credential = app
        .clone()
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

    assert_eq!(create_credential.status(), StatusCode::CREATED);

    let delete_credential = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/admin/credentials/tenant-1/providers/provider-openai-official/keys/cred-openai")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(delete_credential.status(), StatusCode::NO_CONTENT);

    let credentials = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(credentials.status(), StatusCode::OK);
    assert!(read_json(credentials).await.as_array().unwrap().is_empty());

    let secret = sdkwork_api_app_credential::resolve_credential_secret(
        &store,
        "local-dev-master-key",
        "tenant-1",
        "provider-openai-official",
        "cred-openai",
    )
    .await;
    assert!(secret.is_err());
    assert_eq!(
        secret.unwrap_err().to_string(),
        "credential secret not found"
    );
}

#[serial(extension_env)]
#[tokio::test]
async fn create_and_list_models() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    create_provider_fixture(
        app.clone(),
        &token,
        r#"{"id":"provider-openai-official","channel_id":"openai","adapter_kind":"openai","base_url":"https://api.openai.com","display_name":"OpenAI Official","channel_bindings":[{"channel_id":"openai","is_primary":true}]}"#,
    )
    .await;

    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-openai-official\",\"capabilities\":[\"responses\",\"chat_completions\"],\"streaming\":true,\"context_window\":128000}",
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
                .uri("/admin/models")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let models_json = read_json(list).await;
    assert_eq!(models_json[0]["external_name"], "gpt-4.1");
    assert_eq!(models_json[0]["capabilities"].as_array().unwrap().len(), 2);
    assert_eq!(models_json[0]["streaming"], true);
    assert_eq!(models_json[0]["context_window"], 128000);
}

#[serial(extension_env)]
#[tokio::test]
async fn create_update_list_and_delete_coupons() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let initial = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/coupons")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(initial.status(), StatusCode::OK);
    assert_eq!(read_json(initial).await.as_array().unwrap().len(), 0);

    let created = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/coupons")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"id\":\"coupon_spring_launch\",\"code\":\"SPRING20\",\"discount_label\":\"20% launch discount\",\"audience\":\"new_signup\",\"remaining\":120,\"active\":true,\"note\":\"Spring launch campaign\",\"expires_on\":\"2026-05-31\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(created.status(), StatusCode::CREATED);
    let created_json = read_json(created).await;
    assert_eq!(created_json["id"], "coupon_spring_launch");
    assert_eq!(created_json["code"], "SPRING20");
    assert_eq!(created_json["remaining"], 120);

    let updated = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/coupons")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"id\":\"coupon_spring_launch\",\"code\":\"SPRING20\",\"discount_label\":\"20% launch discount\",\"audience\":\"enterprise_trial\",\"remaining\":64,\"active\":false,\"note\":\"Reserved for enterprise conversions\",\"expires_on\":\"2026-06-30\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(updated.status(), StatusCode::CREATED);
    let updated_json = read_json(updated).await;
    assert_eq!(updated_json["audience"], "enterprise_trial");
    assert_eq!(updated_json["remaining"], 64);
    assert_eq!(updated_json["active"], false);

    let listed = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/coupons")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(listed.status(), StatusCode::OK);
    let listed_json = read_json(listed).await;
    assert_eq!(listed_json.as_array().unwrap().len(), 1);
    assert_eq!(listed_json[0]["expires_on"], "2026-06-30");

    let deleted = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/admin/coupons/coupon_spring_launch")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(deleted.status(), StatusCode::NO_CONTENT);

    let after_delete = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/coupons")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(after_delete.status(), StatusCode::OK);
    assert_eq!(read_json(after_delete).await.as_array().unwrap().len(), 0);
}

#[serial(extension_env)]
#[tokio::test]
async fn delete_model_keeps_same_external_name_on_other_provider() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    for channel in [
        r#"{"id":"openai","name":"OpenAI"}"#,
        r#"{"id":"openrouter","name":"OpenRouter"}"#,
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/admin/channels")
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(channel))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    for provider in [
        r#"{"id":"provider-openai-official","channel_id":"openai","display_name":"OpenAI Official","adapter_kind":"openai","base_url":"https://api.openai.com","channel_bindings":[{"channel_id":"openai","is_primary":true}]}"#,
        r#"{"id":"provider-openrouter","channel_id":"openrouter","display_name":"OpenRouter","adapter_kind":"openai","base_url":"https://openrouter.ai/api/v1","channel_bindings":[{"channel_id":"openrouter","is_primary":true}]}"#,
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/admin/providers")
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(provider))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    for model in [
        r#"{"external_name":"gpt-4.1","provider_id":"provider-openai-official","capabilities":["responses"],"streaming":true,"context_window":128000}"#,
        r#"{"external_name":"gpt-4.1","provider_id":"provider-openrouter","capabilities":["responses"],"streaming":true,"context_window":128000}"#,
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/admin/models")
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(model))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    let deleted = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/admin/models/gpt-4.1/providers/provider-openai-official")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(deleted.status(), StatusCode::NO_CONTENT);

    let listed = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(listed.status(), StatusCode::OK);
    let models_json = read_json(listed).await;
    assert_eq!(models_json.as_array().unwrap().len(), 1);
    assert_eq!(models_json[0]["external_name"], "gpt-4.1");
    assert_eq!(models_json[0]["provider_id"], "provider-openrouter");
}

#[serial(extension_env)]
#[tokio::test]
async fn delete_catalog_and_workspace_entities_from_admin_api() {
    let pool = memory_pool().await;
    let store = sdkwork_api_storage_sqlite::SqliteAdminStore::new(pool.clone());
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    for request in [
        ("/admin/tenants", r#"{"id":"tenant-acme","name":"Acme"}"#),
        (
            "/admin/projects",
            r#"{"tenant_id":"tenant-acme","id":"project-acme","name":"Acme Production"}"#,
        ),
        ("/admin/channels", r#"{"id":"openai","name":"OpenAI"}"#),
        (
            "/admin/providers",
            r#"{"id":"provider-openai-official","channel_id":"openai","display_name":"OpenAI Official","adapter_kind":"openai","base_url":"https://api.openai.com","channel_bindings":[{"channel_id":"openai","is_primary":true}]}"#,
        ),
        (
            "/admin/credentials",
            r#"{"tenant_id":"tenant-acme","provider_id":"provider-openai-official","key_reference":"cred-openai","secret_value":"sk-upstream-openai"}"#,
        ),
        (
            "/admin/api-keys",
            r#"{"tenant_id":"tenant-acme","project_id":"project-acme","environment":"production"}"#,
        ),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(request.0)
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(request.1))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    let created_api_key = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/api-keys")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"tenant_id":"tenant-acme","project_id":"project-acme","environment":"staging"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(created_api_key.status(), StatusCode::CREATED);
    let created_api_key_json = read_json(created_api_key).await;
    let plaintext_key = created_api_key_json["plaintext"]
        .as_str()
        .unwrap()
        .to_owned();
    let hashed_key = created_api_key_json["hashed"].as_str().unwrap().to_owned();

    let request_context =
        sdkwork_api_app_identity::resolve_gateway_request_context(&store, &plaintext_key)
            .await
            .unwrap();
    assert!(request_context.is_some());

    let revoked = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/admin/api-keys/{hashed_key}/status"))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"active":false}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(revoked.status(), StatusCode::OK);
    let revoked_json = read_json(revoked).await;
    assert_eq!(revoked_json["active"], false);

    let revoked_request_context =
        sdkwork_api_app_identity::resolve_gateway_request_context(&store, &plaintext_key)
            .await
            .unwrap();
    assert!(revoked_request_context.is_none());

    let restored = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/admin/api-keys/{hashed_key}/status"))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"active":true}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(restored.status(), StatusCode::OK);
    let restored_json = read_json(restored).await;
    assert_eq!(restored_json["active"], true);

    let restored_request_context =
        sdkwork_api_app_identity::resolve_gateway_request_context(&store, &plaintext_key)
            .await
            .unwrap();
    assert!(restored_request_context.is_some());

    for request in [
        (
            "/admin/providers/provider-openai-official",
            "/admin/providers",
            "provider-openai-official",
        ),
        ("/admin/channels/openai", "/admin/channels", "openai"),
        (
            "/admin/projects/project-acme",
            "/admin/projects",
            "project-acme",
        ),
        (
            "/admin/tenants/tenant-acme",
            "/admin/tenants",
            "tenant-acme",
        ),
    ] {
        let deleted = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(request.0)
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(deleted.status(), StatusCode::NO_CONTENT);

        let listed = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(request.1)
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(listed.status(), StatusCode::OK);
        let listed_json = read_json(listed).await;
        assert!(!listed_json
            .as_array()
            .unwrap()
            .iter()
            .any(|item| { item["id"] == request.2 }));
    }

    let credentials = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(credentials.status(), StatusCode::OK);
    assert_eq!(read_json(credentials).await.as_array().unwrap().len(), 0);

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
    assert_eq!(api_keys.status(), StatusCode::OK);
    assert_eq!(read_json(api_keys).await.as_array().unwrap().len(), 0);
}

#[serial(extension_env)]
#[tokio::test]
async fn delete_gateway_api_key_from_admin_api() {
    let pool = memory_pool().await;
    let store = sdkwork_api_storage_sqlite::SqliteAdminStore::new(pool.clone());
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    for request in [
        ("/admin/tenants", r#"{"id":"tenant-acme","name":"Acme"}"#),
        (
            "/admin/projects",
            r#"{"tenant_id":"tenant-acme","id":"project-acme","name":"Acme Production"}"#,
        ),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(request.0)
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(request.1))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    let created = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/api-keys")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"tenant_id":"tenant-acme","project_id":"project-acme","environment":"production"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(created.status(), StatusCode::CREATED);
    let created_json = read_json(created).await;
    let plaintext_key = created_json["plaintext"].as_str().unwrap().to_owned();
    let hashed_key = created_json["hashed"].as_str().unwrap().to_owned();

    let request_context =
        sdkwork_api_app_identity::resolve_gateway_request_context(&store, &plaintext_key)
            .await
            .unwrap();
    assert!(request_context.is_some());

    let deleted = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/admin/api-keys/{hashed_key}"))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(deleted.status(), StatusCode::NO_CONTENT);

    let request_context_after_delete =
        sdkwork_api_app_identity::resolve_gateway_request_context(&store, &plaintext_key)
            .await
            .unwrap();
    assert!(request_context_after_delete.is_none());

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

    assert_eq!(api_keys.status(), StatusCode::OK);
    assert!(read_json(api_keys).await.as_array().unwrap().is_empty());
}

#[serial(extension_env)]
#[tokio::test]
async fn gateway_api_keys_persist_raw_key_in_canonical_ai_app_api_keys_table() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let token = login_token(app.clone()).await;

    for request in [
        ("/admin/tenants", r#"{"id":"tenant-acme","name":"Acme"}"#),
        (
            "/admin/projects",
            r#"{"tenant_id":"tenant-acme","id":"project-acme","name":"Acme Production"}"#,
        ),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(request.0)
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(request.1))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    let created = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/api-keys")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"tenant_id":"tenant-acme","project_id":"project-acme","environment":"production","label":"Production App Key","notes":"retained for admin inventory","expires_at_ms":4102444800000}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(created.status(), StatusCode::CREATED);
    let created_json = read_json(created).await;
    let plaintext_key = created_json["plaintext"].as_str().unwrap().to_owned();
    let hashed_key = created_json["hashed"].as_str().unwrap().to_owned();

    let listed = app
        .clone()
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

    assert_eq!(listed.status(), StatusCode::OK);
    let listed_json = read_json(listed).await;
    assert_eq!(listed_json[0]["raw_key"], plaintext_key);
    assert_eq!(listed_json[0]["label"], "Production App Key");
    assert_eq!(listed_json[0]["notes"], "retained for admin inventory");
    assert_eq!(listed_json[0]["expires_at_ms"], 4102444800000_u64);

    let stored_row: (String, Option<String>, Option<i64>) = sqlx::query_as(
        "SELECT raw_key, notes, expires_at_ms FROM ai_app_api_keys WHERE hashed_key = ?",
    )
    .bind(&hashed_key)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(stored_row.0, plaintext_key);
    assert_eq!(
        stored_row.1.as_deref(),
        Some("retained for admin inventory")
    );
    assert_eq!(stored_row.2, Some(4_102_444_800_000));
}

#[serial(extension_env)]
#[tokio::test]
async fn update_gateway_api_key_metadata_from_admin_api() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let token = login_token(app.clone()).await;

    for request in [
        ("/admin/tenants", r#"{"id":"tenant-acme","name":"Acme"}"#),
        (
            "/admin/projects",
            r#"{"tenant_id":"tenant-acme","id":"project-acme","name":"Acme Production"}"#,
        ),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(request.0)
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(request.1))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    let created = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/api-keys")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"tenant_id":"tenant-acme","project_id":"project-acme","environment":"production","label":"Production App Key","notes":"initial notes","expires_at_ms":4102444800000}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(created.status(), StatusCode::CREATED);
    let created_json = read_json(created).await;
    let hashed_key = created_json["hashed"].as_str().unwrap().to_owned();
    let plaintext_key = created_json["plaintext"].as_str().unwrap().to_owned();

    let updated = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/admin/api-keys/{hashed_key}"))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"tenant_id":"tenant-acme","project_id":"project-acme","environment":"production","label":"Production Key Updated","notes":"rotated by operator","expires_at_ms":4105123200000}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(updated.status(), StatusCode::OK);
    let updated_json = read_json(updated).await;
    assert_eq!(updated_json["hashed_key"], hashed_key);
    assert_eq!(updated_json["raw_key"], plaintext_key);
    assert_eq!(updated_json["label"], "Production Key Updated");
    assert_eq!(updated_json["notes"], "rotated by operator");
    assert_eq!(updated_json["expires_at_ms"], 4105123200000_u64);

    let listed = app
        .clone()
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

    assert_eq!(listed.status(), StatusCode::OK);
    let listed_json = read_json(listed).await;
    assert_eq!(listed_json[0]["label"], "Production Key Updated");
    assert_eq!(listed_json[0]["notes"], "rotated by operator");
    assert_eq!(listed_json[0]["expires_at_ms"], 4105123200000_u64);

    let stored_row: (String, Option<String>, Option<i64>) = sqlx::query_as(
        "SELECT label, notes, expires_at_ms FROM ai_app_api_keys WHERE hashed_key = ?",
    )
    .bind(&hashed_key)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(stored_row.0, "Production Key Updated");
    assert_eq!(stored_row.1.as_deref(), Some("rotated by operator"));
    assert_eq!(stored_row.2, Some(4_105_123_200_000));
}

#[serial(extension_env)]
#[tokio::test]
async fn routing_simulation_uses_catalog_models() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    create_provider_fixture(
        app.clone(),
        &token,
        r#"{"id":"provider-openrouter","channel_id":"openrouter","adapter_kind":"openai","base_url":"https://openrouter.ai/api/v1","display_name":"OpenRouter","channel_bindings":[{"channel_id":"openrouter","is_primary":true}]}"#,
    )
    .await;
    create_provider_fixture(
        app.clone(),
        &token,
        r#"{"id":"provider-openai-official","channel_id":"openai","adapter_kind":"openai","base_url":"https://api.openai.com","display_name":"OpenAI Official","channel_bindings":[{"channel_id":"openai","is_primary":true}]}"#,
    )
    .await;

    let create_openrouter = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-openrouter\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_openrouter.status(), StatusCode::CREATED);

    let create_openai = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-openai-official\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_openai.status(), StatusCode::CREATED);

    let simulate = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/simulations")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"capability\":\"chat_completion\",\"model\":\"gpt-4.1\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(simulate.status(), StatusCode::OK);
    let simulation_json = read_json(simulate).await;
    assert_eq!(
        simulation_json["selected_provider_id"],
        "provider-openai-official"
    );
    assert_eq!(
        simulation_json["candidate_ids"].as_array().unwrap().len(),
        2
    );
    assert_eq!(simulation_json["strategy"], "deterministic_priority");
    assert!(simulation_json["selection_reason"].as_str().is_some());
    assert_eq!(simulation_json["assessments"].as_array().unwrap().len(), 2);
}

#[serial(extension_env)]
#[tokio::test]
async fn create_and_list_routing_policies() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"policy_id\":\"policy-gpt-4-1\",\"capability\":\"chat_completion\",\"model_pattern\":\"gpt-4.1\",\"enabled\":true,\"priority\":100,\"strategy\":\"slo_aware\",\"ordered_provider_ids\":[\"provider-openrouter\",\"provider-openai-official\"],\"default_provider_id\":\"provider-openai-official\",\"max_cost\":0.25,\"max_latency_ms\":200,\"require_healthy\":true}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create.status(), StatusCode::CREATED);
    let created_json = read_json(create).await;
    assert_eq!(created_json["policy_id"], "policy-gpt-4-1");
    assert_eq!(created_json["priority"], 100);
    assert_eq!(created_json["strategy"], "slo_aware");
    assert_eq!(created_json["max_cost"], 0.25);
    assert_eq!(created_json["max_latency_ms"], 200);
    assert_eq!(created_json["require_healthy"], true);

    let list = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(list.status(), StatusCode::OK);
    let list_json = read_json(list).await;
    assert_eq!(list_json[0]["policy_id"], "policy-gpt-4-1");
    assert_eq!(
        list_json[0]["ordered_provider_ids"][0],
        "provider-openrouter"
    );
    assert_eq!(
        list_json[0]["default_provider_id"],
        "provider-openai-official"
    );
    assert_eq!(list_json[0]["strategy"], "slo_aware");
    assert_eq!(list_json[0]["max_cost"], 0.25);
    assert_eq!(list_json[0]["max_latency_ms"], 200);
    assert_eq!(list_json[0]["require_healthy"], true);
}

#[serial(extension_env)]
#[tokio::test]
async fn routing_simulation_persists_decision_log_and_lists_it() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-openrouter\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let simulation = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/simulations")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"capability\":\"chat_completion\",\"model\":\"gpt-4.1\",\"selection_seed\":7}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(simulation.status(), StatusCode::OK);

    let logs = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/routing/decision-logs")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(logs.status(), StatusCode::OK);
    let logs_json = read_json(logs).await;
    assert_eq!(logs_json.as_array().unwrap().len(), 1);
    assert_eq!(logs_json[0]["decision_source"], "admin_simulation");
    assert_eq!(logs_json[0]["capability"], "chat_completion");
    assert_eq!(logs_json[0]["route_key"], "gpt-4.1");
    assert_eq!(logs_json[0]["selection_seed"], 7);
}

#[serial(extension_env)]
#[tokio::test]
async fn routing_simulation_reports_policy_selected_provider() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    create_provider_fixture(
        app.clone(),
        &token,
        r#"{"id":"provider-openrouter","channel_id":"openrouter","adapter_kind":"openai","base_url":"https://openrouter.ai/api/v1","display_name":"OpenRouter","channel_bindings":[{"channel_id":"openrouter","is_primary":true}]}"#,
    )
    .await;
    create_provider_fixture(
        app.clone(),
        &token,
        r#"{"id":"provider-openai-official","channel_id":"openai","adapter_kind":"openai","base_url":"https://api.openai.com","display_name":"OpenAI Official","channel_bindings":[{"channel_id":"openai","is_primary":true}]}"#,
    )
    .await;

    let create_openrouter = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-openrouter\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_openrouter.status(), StatusCode::CREATED);

    let create_openai = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-openai-official\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_openai.status(), StatusCode::CREATED);

    let create_policy = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"policy_id\":\"policy-gpt-4-1\",\"capability\":\"chat_completion\",\"model_pattern\":\"gpt-4.1\",\"enabled\":true,\"priority\":100,\"strategy\":\"weighted_random\",\"ordered_provider_ids\":[\"provider-openrouter\",\"provider-openai-official\"],\"default_provider_id\":\"provider-openai-official\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_policy.status(), StatusCode::CREATED);

    let simulate = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/simulations")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"capability\":\"chat_completion\",\"model\":\"gpt-4.1\",\"selection_seed\":11}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(simulate.status(), StatusCode::OK);
    let simulation_json = read_json(simulate).await;
    assert_eq!(
        simulation_json["selected_provider_id"],
        "provider-openrouter"
    );
    assert_eq!(simulation_json["matched_policy_id"], "policy-gpt-4-1");
    assert_eq!(simulation_json["strategy"], "weighted_random");
    assert_eq!(simulation_json["selection_seed"], 11);
    assert!(simulation_json["selection_reason"].as_str().is_some());
    assert_eq!(
        simulation_json["assessments"][0]["provider_id"],
        "provider-openrouter"
    );
    assert!(simulation_json["assessments"][0]["reasons"]
        .as_array()
        .is_some());
}

#[serial(extension_env)]
#[tokio::test]
async fn routing_simulation_accepts_requested_region_and_persists_logs() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let create_channel = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/channels")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"id\":\"geo-openai\",\"name\":\"Geo OpenAI\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_channel.status(), StatusCode::CREATED);

    let create_eu_provider = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"id\":\"provider-eu-west\",\"channel_id\":\"geo-openai\",\"extension_id\":\"sdkwork.provider.openai.official\",\"adapter_kind\":\"openai\",\"base_url\":\"https://eu-west.example/v1\",\"display_name\":\"EU West Provider\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_eu_provider.status(), StatusCode::CREATED);

    let create_us_provider = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"id\":\"provider-us-east\",\"channel_id\":\"geo-openai\",\"extension_id\":\"sdkwork.provider.openai.official\",\"adapter_kind\":\"openai\",\"base_url\":\"https://us-east.example/v1\",\"display_name\":\"US East Provider\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_us_provider.status(), StatusCode::CREATED);

    let create_eu_model = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-eu-west\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_eu_model.status(), StatusCode::CREATED);

    let create_us_model = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-us-east\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_us_model.status(), StatusCode::CREATED);

    let openrouter_installation = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/installations")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"installation_id\":\"geo-eu-installation\",\"extension_id\":\"sdkwork.provider.openai.official\",\"runtime\":\"builtin\",\"enabled\":true,\"entrypoint\":null,\"config\":{}}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(openrouter_installation.status(), StatusCode::CREATED);

    let openai_installation = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/installations")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"installation_id\":\"geo-us-installation\",\"extension_id\":\"sdkwork.provider.openai.official\",\"runtime\":\"builtin\",\"enabled\":true,\"entrypoint\":null,\"config\":{}}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(openai_installation.status(), StatusCode::CREATED);

    let openrouter_instance = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/instances")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"instance_id\":\"provider-eu-west\",\"installation_id\":\"geo-eu-installation\",\"extension_id\":\"sdkwork.provider.openai.official\",\"enabled\":true,\"base_url\":\"https://eu-west.example/v1\",\"credential_ref\":null,\"config\":{\"region\":\"eu-west\"}}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(openrouter_instance.status(), StatusCode::CREATED);

    let openai_instance = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/instances")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"instance_id\":\"provider-us-east\",\"installation_id\":\"geo-us-installation\",\"extension_id\":\"sdkwork.provider.openai.official\",\"enabled\":true,\"base_url\":\"https://us-east.example/v1\",\"credential_ref\":null,\"config\":{\"routing\":{\"region\":\"us-east\"}}}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(openai_instance.status(), StatusCode::CREATED);

    let create_policy = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"policy_id\":\"policy-geo-affinity\",\"capability\":\"chat_completion\",\"model_pattern\":\"gpt-4.1\",\"enabled\":true,\"priority\":100,\"strategy\":\"geo_affinity\",\"ordered_provider_ids\":[\"provider-eu-west\",\"provider-us-east\"]}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_policy.status(), StatusCode::CREATED);

    let simulation = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/simulations")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"capability\":\"chat_completion\",\"model\":\"gpt-4.1\",\"requested_region\":\"us-east\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(simulation.status(), StatusCode::OK);
    let simulation_json = read_json(simulation).await;
    assert_eq!(simulation_json["selected_provider_id"], "provider-us-east");
    assert_eq!(simulation_json["strategy"], "geo_affinity");
    assert_eq!(simulation_json["requested_region"], "us-east");
    assert_eq!(simulation_json["assessments"][0]["region"], "eu-west");
    assert_eq!(simulation_json["assessments"][0]["region_match"], false);
    assert_eq!(simulation_json["assessments"][1]["region"], "us-east");
    assert_eq!(simulation_json["assessments"][1]["region_match"], true);

    let logs = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/routing/decision-logs")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(logs.status(), StatusCode::OK);
    let logs_json = read_json(logs).await;
    assert_eq!(logs_json[0]["requested_region"], "us-east");
}

#[serial(extension_env)]
#[tokio::test]
async fn create_and_list_extension_installations_and_instances() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let installation = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/installations")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"installation_id\":\"openrouter-builtin\",\"extension_id\":\"sdkwork.provider.openrouter\",\"runtime\":\"builtin\",\"enabled\":true,\"entrypoint\":null,\"config\":{\"trust_mode\":\"builtin\"}}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(installation.status(), StatusCode::CREATED);

    let instance = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/instances")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"instance_id\":\"provider-openrouter-main\",\"installation_id\":\"openrouter-builtin\",\"extension_id\":\"sdkwork.provider.openrouter\",\"enabled\":true,\"base_url\":\"https://openrouter.ai/api/v1\",\"credential_ref\":\"cred-openrouter\",\"config\":{\"region\":\"global\",\"weight\":100}}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(instance.status(), StatusCode::CREATED);

    let installations = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/extensions/installations")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(installations.status(), StatusCode::OK);
    let installations_json = read_json(installations).await;
    assert_eq!(
        installations_json[0]["extension_id"],
        "sdkwork.provider.openrouter"
    );
    assert_eq!(installations_json[0]["runtime"], "builtin");

    let instances = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/extensions/instances")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(instances.status(), StatusCode::OK);
    let instances_json = read_json(instances).await;
    assert_eq!(instances_json[0]["instance_id"], "provider-openrouter-main");
    assert_eq!(
        instances_json[0]["base_url"],
        "https://openrouter.ai/api/v1"
    );
    assert_eq!(instances_json[0]["config"]["region"], "global");
}

#[serial(extension_env)]
#[tokio::test]
async fn list_discovered_extension_packages_from_admin_api() {
    let root = temp_extension_root("admin-extension-packages");
    let package_dir = root.join("sdkwork-provider-custom-openai");
    fs::create_dir_all(&package_dir).unwrap();
    fs::write(
        package_dir.join("sdkwork-extension.toml"),
        r#"
api_version = "sdkwork.extension/v1"
id = "sdkwork.provider.custom-openai"
kind = "provider"
version = "0.1.0"
display_name = "Custom OpenAI"
runtime = "connector"
protocol = "openai"
entrypoint = "powershell.exe"
channel_bindings = ["sdkwork.channel.openai"]
permissions = ["network_outbound", "spawn_process"]

[health]
path = "/health"
interval_secs = 30

[[capabilities]]
operation = "chat.completions.create"
compatibility = "relay"
"#,
    )
    .unwrap();
    let _guard = extension_env_guard(&root);

    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/extensions/packages")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json.as_array().unwrap().len(), 1);
    assert_eq!(json[0]["manifest"]["id"], "sdkwork.provider.custom-openai");
    assert_eq!(
        json[0]["root_dir"],
        package_dir.to_string_lossy().to_string()
    );
    assert_eq!(
        json[0]["distribution_name"],
        "sdkwork-provider-custom-openai"
    );
    assert_eq!(
        json[0]["crate_name"],
        "sdkwork-api-ext-provider-custom-openai"
    );
    assert_eq!(json[0]["validation"]["valid"], true);
    assert_eq!(json[0]["validation"]["issues"].as_array().unwrap().len(), 0);
    assert_eq!(json[0]["trust"]["state"], "unsigned");
    assert_eq!(json[0]["trust"]["signature_present"], false);
    assert_eq!(json[0]["trust"]["load_allowed"], true);

    cleanup_dir(&root);
}

#[serial(extension_env)]
#[tokio::test]
async fn list_discovered_extension_packages_reloads_current_extension_policy_from_environment() {
    let root_one = temp_extension_root("admin-extension-packages-dynamic-one");
    let root_two = temp_extension_root("admin-extension-packages-dynamic-two");
    let package_dir = root_two.join("sdkwork-provider-custom-openai");
    fs::create_dir_all(&package_dir).unwrap();
    fs::write(
        package_dir.join("sdkwork-extension.toml"),
        r#"
api_version = "sdkwork.extension/v1"
id = "sdkwork.provider.custom-openai"
kind = "provider"
version = "0.1.0"
display_name = "Custom OpenAI"
runtime = "connector"
protocol = "openai"
entrypoint = "powershell.exe"
channel_bindings = ["sdkwork.channel.openai"]
permissions = ["network_outbound", "spawn_process"]

[health]
path = "/health"
interval_secs = 30

[[capabilities]]
operation = "chat.completions.create"
compatibility = "relay"
"#,
    )
    .unwrap();

    let _guard = extension_env_guard(&root_one);

    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    std::env::set_var(
        "SDKWORK_EXTENSION_PATHS",
        std::env::join_paths([&root_two]).unwrap(),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/extensions/packages")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json.as_array().unwrap().len(), 1);
    assert_eq!(json[0]["manifest"]["id"], "sdkwork.provider.custom-openai");
    assert_eq!(
        json[0]["root_dir"],
        package_dir.to_string_lossy().to_string()
    );

    cleanup_dir(&root_one);
    cleanup_dir(&root_two);
}

#[cfg(windows)]
#[serial(extension_env)]
#[tokio::test]
async fn list_active_connector_runtime_statuses_from_admin_api() {
    let root = temp_extension_root("admin-runtime-statuses");
    fs::create_dir_all(&root).unwrap();
    let port = free_port();
    fs::write(root.join("connector.ps1"), connector_script_body(port)).unwrap();

    let load_plan = ExtensionLoadPlan {
        instance_id: "provider-custom-openai".to_owned(),
        installation_id: "custom-openai-installation".to_owned(),
        extension_id: "sdkwork.provider.custom-openai".to_owned(),
        enabled: true,
        runtime: ExtensionRuntime::Connector,
        display_name: "Custom OpenAI".to_owned(),
        entrypoint: Some("powershell.exe".to_owned()),
        base_url: Some(format!("http://127.0.0.1:{port}")),
        credential_ref: None,
        config_schema: None,
        credential_schema: None,
        package_root: Some(root.clone()),
        config: serde_json::json!({
            "command_args": [
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
                "connector.ps1"
            ],
            "health_path": "/health",
            "startup_timeout_ms": 4000,
            "startup_poll_interval_ms": 50
        }),
    };

    ensure_connector_runtime_started(&load_plan, load_plan.base_url.as_deref().expect("base url"))
        .unwrap();

    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/extensions/runtime-statuses")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json.as_array().unwrap().len(), 1);
    assert_eq!(json[0]["runtime"], "connector");
    assert_eq!(json[0]["extension_id"], "sdkwork.provider.custom-openai");
    assert_eq!(json[0]["instance_id"], "provider-custom-openai");
    assert_eq!(json[0]["running"], true);
    assert_eq!(json[0]["healthy"], true);

    shutdown_all_connector_runtimes().unwrap();
    cleanup_dir(&root);
}

#[serial(extension_env)]
#[tokio::test]
async fn list_active_native_dynamic_runtime_statuses_from_admin_api() {
    shutdown_all_native_dynamic_runtimes().unwrap();

    let library_path = native_dynamic_fixture_library_path();
    let _adapter =
        load_native_dynamic_provider_adapter(&library_path, "https://example.com/v1").unwrap();

    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/extensions/runtime-statuses")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json.as_array().unwrap().len(), 1);
    assert_eq!(json[0]["runtime"], "native_dynamic");
    assert_eq!(json[0]["extension_id"], "sdkwork.provider.native.mock");
    assert_eq!(json[0]["running"], true);
    assert_eq!(json[0]["healthy"], true);
    assert_eq!(json[0]["supports_health_check"], true);
    assert_eq!(json[0]["supports_shutdown"], true);
    assert_eq!(json[0]["message"], "native mock healthy");

    shutdown_all_native_dynamic_runtimes().unwrap();
}

#[serial(extension_env)]
#[tokio::test]
async fn extension_runtime_reload_endpoint_rebuilds_runtime_state() {
    shutdown_all_native_dynamic_runtimes().unwrap();

    let log_guard = NativeDynamicLifecycleLogGuard::new();
    let extension_root = temp_extension_root("admin-runtime-reload");
    let package_dir = extension_root.join("sdkwork-provider-native-mock");
    fs::create_dir_all(&package_dir).unwrap();
    let library_path = native_dynamic_fixture_library_path();
    fs::write(
        package_dir.join("sdkwork-extension.toml"),
        native_dynamic_manifest(&library_path),
    )
    .unwrap();

    let _guard = native_dynamic_extension_env_guard(&extension_root);

    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let first = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/runtime-reloads")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::OK);

    let second = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/runtime-reloads")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(second.status(), StatusCode::OK);
    let json = read_json(second).await;
    assert_eq!(json["discovered_package_count"], 1);
    assert_eq!(json["loadable_package_count"], 1);
    assert_eq!(json["active_runtime_count"], 1);
    assert_eq!(json["runtime_statuses"][0]["runtime"], "native_dynamic");
    assert_eq!(
        json["runtime_statuses"][0]["extension_id"],
        FIXTURE_EXTENSION_ID
    );
    assert_eq!(json["runtime_statuses"][0]["running"], true);
    assert_eq!(json["runtime_statuses"][0]["healthy"], true);
    assert_eq!(
        std::fs::read_to_string(log_guard.path())
            .unwrap()
            .lines()
            .collect::<Vec<_>>(),
        vec!["init", "shutdown", "init"]
    );

    shutdown_all_native_dynamic_runtimes().unwrap();
    cleanup_dir(&extension_root);
}

#[serial(extension_env)]
#[tokio::test]
async fn extension_runtime_reload_endpoint_supports_targeted_scope() {
    shutdown_all_native_dynamic_runtimes().unwrap();

    let log_guard = NativeDynamicLifecycleLogGuard::new();
    let extension_root = temp_extension_root("admin-runtime-reload-targeted");
    let package_dir = extension_root.join("sdkwork-provider-native-mock");
    fs::create_dir_all(&package_dir).unwrap();
    let library_path = native_dynamic_fixture_library_path();
    fs::write(
        package_dir.join("sdkwork-extension.toml"),
        native_dynamic_manifest(&library_path),
    )
    .unwrap();

    let _guard = native_dynamic_extension_env_guard(&extension_root);

    let pool = memory_pool().await;
    let store = sdkwork_api_storage_sqlite::SqliteAdminStore::new(pool.clone());
    store
        .insert_extension_installation(
            &ExtensionInstallation::new(
                "connector-mock-installation",
                "sdkwork.provider.connector.mock",
                ExtensionRuntime::Connector,
            )
            .with_enabled(true)
            .with_entrypoint("connector-mock"),
        )
        .await
        .unwrap();
    store
        .insert_extension_instance(
            &ExtensionInstance::new(
                "provider-connector-mock",
                "connector-mock-installation",
                "sdkwork.provider.connector.mock",
            )
            .with_enabled(true)
            .with_base_url("http://127.0.0.1:9"),
        )
        .await
        .unwrap();

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let initial = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/runtime-reloads")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(initial.status(), StatusCode::OK);
    assert_eq!(
        std::fs::read_to_string(log_guard.path())
            .unwrap()
            .lines()
            .collect::<Vec<_>>(),
        vec!["init"]
    );

    let by_extension = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/runtime-reloads")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "extension_id": FIXTURE_EXTENSION_ID,
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(by_extension.status(), StatusCode::OK);
    let extension_json = read_json(by_extension).await;
    assert_eq!(extension_json["scope"], "extension");
    assert_eq!(
        extension_json["requested_extension_id"],
        FIXTURE_EXTENSION_ID
    );
    assert_eq!(extension_json["requested_instance_id"], Value::Null);
    assert_eq!(
        extension_json["resolved_extension_id"],
        FIXTURE_EXTENSION_ID
    );
    assert_eq!(extension_json["discovered_package_count"], 1);
    assert_eq!(extension_json["loadable_package_count"], 1);
    assert_eq!(extension_json["active_runtime_count"], 1);
    assert_eq!(
        std::fs::read_to_string(log_guard.path())
            .unwrap()
            .lines()
            .collect::<Vec<_>>(),
        vec!["init", "shutdown", "init"]
    );

    let by_instance = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/runtime-reloads")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "instance_id": "provider-connector-mock",
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(by_instance.status(), StatusCode::OK);
    let instance_json = read_json(by_instance).await;
    assert_eq!(instance_json["scope"], "instance");
    assert_eq!(instance_json["requested_extension_id"], Value::Null);
    assert_eq!(
        instance_json["requested_instance_id"],
        "provider-connector-mock"
    );
    assert_eq!(
        instance_json["resolved_extension_id"],
        "sdkwork.provider.connector.mock"
    );
    assert_eq!(instance_json["active_runtime_count"], 1);
    assert_eq!(
        std::fs::read_to_string(log_guard.path())
            .unwrap()
            .lines()
            .collect::<Vec<_>>(),
        vec!["init", "shutdown", "init"]
    );

    let invalid = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/runtime-reloads")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "extension_id": FIXTURE_EXTENSION_ID,
                        "instance_id": "provider-connector-mock",
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(invalid.status(), StatusCode::BAD_REQUEST);

    shutdown_all_native_dynamic_runtimes().unwrap();
    cleanup_dir(&extension_root);
}

#[serial(extension_env)]
#[tokio::test]
async fn cluster_runtime_rollout_creation_snapshots_active_nodes() {
    let pool = memory_pool().await;
    let store = sdkwork_api_storage_sqlite::SqliteAdminStore::new(pool.clone());
    let now_ms = unix_timestamp_ms();
    store
        .upsert_service_runtime_node(&ServiceRuntimeNodeRecord::new(
            "gateway-node-a",
            "gateway",
            now_ms - 1_000,
        ))
        .await
        .unwrap();
    store
        .upsert_service_runtime_node(&ServiceRuntimeNodeRecord::new(
            "admin-node-a",
            "admin",
            now_ms - 1_000,
        ))
        .await
        .unwrap();
    store
        .upsert_service_runtime_node(
            &ServiceRuntimeNodeRecord::new("stale-gateway-node", "gateway", 0)
                .with_last_seen_at_ms(0),
        )
        .await
        .unwrap();

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/runtime-rollouts")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "extension_id": FIXTURE_EXTENSION_ID,
                        "timeout_secs": 30,
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create.status(), StatusCode::CREATED);
    let create_json = read_json(create).await;
    assert_eq!(create_json["status"], "pending");
    assert_eq!(create_json["scope"], "extension");
    assert_eq!(create_json["requested_extension_id"], FIXTURE_EXTENSION_ID);
    assert_eq!(create_json["participant_count"], 2);
    assert_eq!(create_json["participants"].as_array().unwrap().len(), 2);
    assert_eq!(create_json["participants"][0]["node_id"], "admin-node-a");
    assert_eq!(create_json["participants"][1]["node_id"], "gateway-node-a");

    let rollout_id = create_json["rollout_id"].as_str().unwrap();
    let list = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/extensions/runtime-rollouts")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(list.status(), StatusCode::OK);
    let list_json = read_json(list).await;
    assert_eq!(list_json[0]["rollout_id"], rollout_id);
    assert_eq!(list_json[0]["participant_count"], 2);

    let get = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/admin/extensions/runtime-rollouts/{rollout_id}"))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(get.status(), StatusCode::OK);
    let get_json = read_json(get).await;
    assert_eq!(get_json["rollout_id"], rollout_id);
    assert_eq!(get_json["participant_count"], 2);
}

#[serial(extension_env)]
#[tokio::test]
async fn cluster_runtime_config_rollout_creation_snapshots_active_nodes() {
    let pool = memory_pool().await;
    let store = sdkwork_api_storage_sqlite::SqliteAdminStore::new(pool.clone());
    let now_ms = unix_timestamp_ms();
    store
        .upsert_service_runtime_node(&ServiceRuntimeNodeRecord::new(
            "gateway-node-a",
            "gateway",
            now_ms - 1_000,
        ))
        .await
        .unwrap();
    store
        .upsert_service_runtime_node(&ServiceRuntimeNodeRecord::new(
            "admin-node-a",
            "admin",
            now_ms - 1_000,
        ))
        .await
        .unwrap();
    store
        .upsert_service_runtime_node(&ServiceRuntimeNodeRecord::new(
            "portal-node-a",
            "portal",
            now_ms - 1_000,
        ))
        .await
        .unwrap();
    store
        .upsert_service_runtime_node(
            &ServiceRuntimeNodeRecord::new("stale-portal-node", "portal", 0)
                .with_last_seen_at_ms(0),
        )
        .await
        .unwrap();

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/runtime-config/rollouts")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "service_kind": "portal",
                        "timeout_secs": 30,
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create.status(), StatusCode::CREATED);
    let create_json = read_json(create).await;
    assert_eq!(create_json["status"], "pending");
    assert_eq!(create_json["requested_service_kind"], "portal");
    assert_eq!(create_json["participant_count"], 1);
    assert_eq!(create_json["participants"].as_array().unwrap().len(), 1);
    assert_eq!(create_json["participants"][0]["node_id"], "portal-node-a");
    assert_eq!(create_json["participants"][0]["service_kind"], "portal");

    let rollout_id = create_json["rollout_id"].as_str().unwrap();
    let list = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/runtime-config/rollouts")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(list.status(), StatusCode::OK);
    let list_json = read_json(list).await;
    assert_eq!(list_json[0]["rollout_id"], rollout_id);
    assert_eq!(list_json[0]["participant_count"], 1);

    let get = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/admin/runtime-config/rollouts/{rollout_id}"))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(get.status(), StatusCode::OK);
    let get_json = read_json(get).await;
    assert_eq!(get_json["rollout_id"], rollout_id);
    assert_eq!(get_json["participant_count"], 1);
}

#[serial(extension_env)]
#[tokio::test]
async fn list_provider_health_snapshots_from_admin_api() {
    let pool = memory_pool().await;
    let store = sdkwork_api_storage_sqlite::SqliteAdminStore::new(pool.clone());
    store
        .insert_provider_health_snapshot(
            &sdkwork_api_domain_routing::ProviderHealthSnapshot::new(
                "provider-openai-official",
                "sdkwork.provider.openai.official",
                "builtin",
                1234,
            )
            .with_running(true)
            .with_healthy(true)
            .with_message("healthy"),
        )
        .await
        .unwrap();

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/routing/health-snapshots")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json.as_array().unwrap().len(), 1);
    assert_eq!(json[0]["provider_id"], "provider-openai-official");
    assert_eq!(json[0]["healthy"], true);
    assert_eq!(json[0]["message"], "healthy");
}

#[serial(extension_env)]
#[tokio::test]
async fn usage_summary_from_admin_api_reports_grouped_counts() {
    let pool = memory_pool().await;
    let store = sdkwork_api_storage_sqlite::SqliteAdminStore::new(pool.clone());
    store
        .insert_usage_record(&sdkwork_api_domain_usage::UsageRecord::new(
            "project-1",
            "gpt-4.1",
            "provider-openai",
        ))
        .await
        .unwrap();
    store
        .insert_usage_record(&sdkwork_api_domain_usage::UsageRecord::new(
            "project-1",
            "gpt-4.1",
            "provider-openai",
        ))
        .await
        .unwrap();
    store
        .insert_usage_record(&sdkwork_api_domain_usage::UsageRecord::new(
            "project-2",
            "text-embedding-3-large",
            "provider-openrouter",
        ))
        .await
        .unwrap();

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/usage/summary")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["total_requests"], 3);
    assert_eq!(json["project_count"], 2);
    assert_eq!(json["provider_count"], 2);
    assert_eq!(json["projects"][0]["project_id"], "project-1");
    assert_eq!(json["projects"][0]["request_count"], 2);
    assert_eq!(json["providers"][0]["provider"], "provider-openai");
    assert_eq!(json["providers"][0]["request_count"], 2);
    assert_eq!(json["models"][0]["model"], "gpt-4.1");
    assert_eq!(json["models"][0]["request_count"], 2);
}

#[serial(extension_env)]
#[tokio::test]
async fn billing_summary_from_admin_api_reports_quota_posture() {
    let pool = memory_pool().await;
    let store = sdkwork_api_storage_sqlite::SqliteAdminStore::new(pool.clone());
    store
        .insert_ledger_entry(&sdkwork_api_domain_billing::LedgerEntry::new(
            "project-1",
            70,
            0.70,
        ))
        .await
        .unwrap();
    store
        .insert_ledger_entry(&sdkwork_api_domain_billing::LedgerEntry::new(
            "project-1",
            40,
            0.40,
        ))
        .await
        .unwrap();
    store
        .insert_quota_policy(&sdkwork_api_domain_billing::QuotaPolicy::new(
            "quota-project-1",
            "project-1",
            100,
        ))
        .await
        .unwrap();

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/billing/summary")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["total_entries"], 2);
    assert_eq!(json["total_units"], 110);
    assert_eq!(json["active_quota_policy_count"], 1);
    assert_eq!(json["exhausted_project_count"], 1);
    assert_eq!(json["projects"][0]["project_id"], "project-1");
    assert_eq!(json["projects"][0]["entry_count"], 2);
    assert_eq!(json["projects"][0]["used_units"], 110);
    assert_eq!(json["projects"][0]["quota_policy_id"], "quota-project-1");
    assert_eq!(json["projects"][0]["remaining_units"], 0);
    assert_eq!(json["projects"][0]["exhausted"], true);
}

#[serial(extension_env)]
#[tokio::test]
async fn create_and_list_quota_policies_from_admin_api() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/billing/quota-policies")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"policy_id\":\"quota-project-1\",\"project_id\":\"project-1\",\"max_units\":1000,\"enabled\":true}",
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
                .uri("/admin/billing/quota-policies")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(list.status(), StatusCode::OK);
    let json = read_json(list).await;
    assert_eq!(json.as_array().unwrap().len(), 1);
    assert_eq!(json[0]["policy_id"], "quota-project-1");
    assert_eq!(json[0]["project_id"], "project-1");
    assert_eq!(json[0]["max_units"], 1000);
}

fn temp_extension_root(suffix: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    path.push(format!("sdkwork-admin-routes-{suffix}-{millis}"));
    path
}

fn cleanup_dir(path: &Path) {
    let _ = fs::remove_dir_all(path);
}

fn unix_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("unix time")
        .as_millis() as u64
}

fn extension_env_guard(path: &Path) -> ExtensionEnvGuard {
    let previous_paths = std::env::var("SDKWORK_EXTENSION_PATHS").ok();
    let previous_connector = std::env::var("SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS").ok();
    let previous_native = std::env::var("SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS").ok();

    let joined_paths = std::env::join_paths([path]).unwrap();
    std::env::set_var("SDKWORK_EXTENSION_PATHS", joined_paths);
    std::env::set_var("SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS", "true");
    std::env::set_var(
        "SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS",
        "false",
    );

    ExtensionEnvGuard {
        previous_paths,
        previous_connector,
        previous_native,
    }
}

struct ExtensionEnvGuard {
    previous_paths: Option<String>,
    previous_connector: Option<String>,
    previous_native: Option<String>,
}

impl Drop for ExtensionEnvGuard {
    fn drop(&mut self) {
        restore_env_var("SDKWORK_EXTENSION_PATHS", self.previous_paths.as_deref());
        restore_env_var(
            "SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS",
            self.previous_connector.as_deref(),
        );
        restore_env_var(
            "SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS",
            self.previous_native.as_deref(),
        );
    }
}

fn restore_env_var(key: &str, value: Option<&str>) {
    match value {
        Some(value) => std::env::set_var(key, value),
        None => std::env::remove_var(key),
    }
}

fn native_dynamic_extension_env_guard(path: &Path) -> NativeDynamicEnvGuard {
    let previous_paths = std::env::var("SDKWORK_EXTENSION_PATHS").ok();
    let previous_connector = std::env::var("SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS").ok();
    let previous_native = std::env::var("SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS").ok();
    let previous_native_signature =
        std::env::var("SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS").ok();
    let previous_trusted_signers = std::env::var("SDKWORK_EXTENSION_TRUSTED_SIGNERS").ok();

    let joined_paths = std::env::join_paths([path]).unwrap();
    std::env::set_var("SDKWORK_EXTENSION_PATHS", joined_paths);
    std::env::set_var("SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS", "false");
    std::env::set_var("SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS", "true");
    std::env::set_var(
        "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS",
        "false",
    );
    std::env::remove_var("SDKWORK_EXTENSION_TRUSTED_SIGNERS");

    NativeDynamicEnvGuard {
        previous_paths,
        previous_connector,
        previous_native,
        previous_native_signature,
        previous_trusted_signers,
    }
}

struct NativeDynamicEnvGuard {
    previous_paths: Option<String>,
    previous_connector: Option<String>,
    previous_native: Option<String>,
    previous_native_signature: Option<String>,
    previous_trusted_signers: Option<String>,
}

impl Drop for NativeDynamicEnvGuard {
    fn drop(&mut self) {
        restore_env_var("SDKWORK_EXTENSION_PATHS", self.previous_paths.as_deref());
        restore_env_var(
            "SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS",
            self.previous_connector.as_deref(),
        );
        restore_env_var(
            "SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS",
            self.previous_native.as_deref(),
        );
        restore_env_var(
            "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS",
            self.previous_native_signature.as_deref(),
        );
        restore_env_var(
            "SDKWORK_EXTENSION_TRUSTED_SIGNERS",
            self.previous_trusted_signers.as_deref(),
        );
    }
}

fn native_dynamic_manifest(entrypoint: &Path) -> String {
    format!(
        r#"
api_version = "sdkwork.extension/v1"
id = "{FIXTURE_EXTENSION_ID}"
kind = "provider"
version = "0.1.0"
display_name = "Native Mock"
runtime = "native_dynamic"
protocol = "openai"
entrypoint = "{}"
channel_bindings = ["sdkwork.channel.openai"]
permissions = ["network_outbound"]

[[capabilities]]
operation = "chat.completions.create"
compatibility = "native"

[[capabilities]]
operation = "chat.completions.stream"
compatibility = "native"

[[capabilities]]
operation = "responses.create"
compatibility = "native"

[[capabilities]]
operation = "responses.stream"
compatibility = "native"

[[capabilities]]
operation = "audio.speech.create"
compatibility = "native"

[[capabilities]]
operation = "files.content"
compatibility = "native"

[[capabilities]]
operation = "videos.content"
compatibility = "native"
"#,
        config_path_value(entrypoint)
    )
}

fn config_path_value(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

struct NativeDynamicLifecycleLogGuard {
    path: PathBuf,
    previous: Option<String>,
}

impl NativeDynamicLifecycleLogGuard {
    fn new() -> Self {
        let mut path = std::env::temp_dir();
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("unix time")
            .as_millis();
        path.push(format!(
            "sdkwork-admin-native-dynamic-lifecycle-{millis}.log"
        ));

        let previous = std::env::var("SDKWORK_NATIVE_MOCK_LIFECYCLE_LOG").ok();
        std::env::set_var("SDKWORK_NATIVE_MOCK_LIFECYCLE_LOG", &path);

        Self { path, previous }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for NativeDynamicLifecycleLogGuard {
    fn drop(&mut self) {
        restore_env_var(
            "SDKWORK_NATIVE_MOCK_LIFECYCLE_LOG",
            self.previous.as_deref(),
        );
        let _ = std::fs::remove_file(&self.path);
    }
}

#[cfg(windows)]
fn connector_script_body(port: u16) -> String {
    format!(
        r#"
$listener = [System.Net.Sockets.TcpListener]::new([System.Net.IPAddress]::Parse("127.0.0.1"), {port})
$listener.Start()
while ($true) {{
    $client = $listener.AcceptTcpClient()
    $stream = $client.GetStream()
    $reader = New-Object System.IO.StreamReader($stream, [System.Text.Encoding]::ASCII, $false, 1024, $true)
    $requestLine = $reader.ReadLine()
    while ($true) {{
        $line = $reader.ReadLine()
        if ([string]::IsNullOrEmpty($line)) {{
            break
        }}
    }}

    if ($requestLine.StartsWith('GET /health')) {{
        $body = '{{"status":"ok"}}'
        $status = 'HTTP/1.1 200 OK'
    }} else {{
        $body = '{{"error":"not_found"}}'
        $status = 'HTTP/1.1 404 Not Found'
    }}

    $bodyBytes = [System.Text.Encoding]::UTF8.GetBytes($body)
    $writer = New-Object System.IO.StreamWriter($stream, [System.Text.Encoding]::ASCII, 1024, $true)
    $writer.NewLine = "`r`n"
    $writer.WriteLine($status)
    $writer.WriteLine('Content-Type: application/json')
    $writer.WriteLine(('Content-Length: ' + $bodyBytes.Length))
    $writer.WriteLine('Connection: close')
    $writer.WriteLine()
    $writer.Flush()
    $stream.Write($bodyBytes, 0, $bodyBytes.Length)
    $stream.Flush()
    $client.Close()
}}
"#
    )
}

fn native_dynamic_fixture_library_path() -> PathBuf {
    let current_exe = std::env::current_exe().expect("current exe");
    let directory = current_exe.parent().expect("exe dir");
    let prefix = if cfg!(windows) {
        "sdkwork_api_ext_provider_native_mock"
    } else {
        "libsdkwork_api_ext_provider_native_mock"
    };
    let extension = if cfg!(windows) {
        "dll"
    } else if cfg!(target_os = "macos") {
        "dylib"
    } else {
        "so"
    };

    std::fs::read_dir(directory)
        .expect("deps dir")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| {
            path.extension().and_then(|value| value.to_str()) == Some(extension)
                && path
                    .file_stem()
                    .and_then(|value| value.to_str())
                    .is_some_and(|stem| stem.starts_with(prefix))
        })
        .expect("native dynamic fixture library")
}

#[cfg(windows)]
fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}
