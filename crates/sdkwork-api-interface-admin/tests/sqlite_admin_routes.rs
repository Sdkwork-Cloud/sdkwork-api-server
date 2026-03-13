use axum::body::{to_bytes, Body};
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
                .body(Body::from("{\"subject\":\"admin-user\"}"))
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
async fn login_returns_a_gateway_jwt_like_token() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/auth/login")
                .header("content-type", "application/json")
                .body(Body::from("{\"subject\":\"admin-user\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["claims"]["sub"], "admin-user");
    assert_eq!(json["token"].as_str().unwrap().split('.').count(), 3);
}

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
    assert_eq!(json.as_array().unwrap().len(), 1);
    assert_eq!(json[0]["id"], "openai");
}

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
                    "{\"id\":\"provider-openai-official\",\"channel_id\":\"openai\",\"channel_bindings\":[{\"channel_id\":\"openai\",\"is_primary\":true},{\"channel_id\":\"responses-compatible\",\"is_primary\":false}],\"adapter_kind\":\"openai\",\"base_url\":\"https://api.openai.com\",\"display_name\":\"OpenAI Official\"}",
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

#[tokio::test]
async fn create_and_list_models() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

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

#[tokio::test]
async fn routing_simulation_uses_catalog_models() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

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
}

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
