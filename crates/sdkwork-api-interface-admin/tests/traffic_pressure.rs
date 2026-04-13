use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use sdkwork_api_app_credential::CredentialSecretManager;
use sdkwork_api_app_identity::hash_gateway_api_key;
use sdkwork_api_app_rate_limit::{
    CommercialAdmissionPolicy, GatewayTrafficController, GatewayTrafficRequestContext,
    InMemoryGatewayTrafficController,
};
use sdkwork_api_storage_core::AdminStore;
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
    read_json(response).await["token"]
        .as_str()
        .unwrap()
        .to_owned()
}

#[tokio::test]
async fn admin_gateway_traffic_pressure_reports_live_project_key_and_provider_saturation() {
    let pool = memory_pool().await;
    let store = Arc::new(SqliteAdminStore::new(pool)) as Arc<dyn AdminStore>;
    let controller = Arc::new(InMemoryGatewayTrafficController::new());
    let hashed_key = hash_gateway_api_key("sk-live-pressure-admin");

    controller.replace_policies(vec![
        CommercialAdmissionPolicy::new("project-live-cap", "project-live")
            .with_project_concurrency_limit(1),
        CommercialAdmissionPolicy::new("key-live-cap", "project-live")
            .with_api_key_hash(&hashed_key)
            .with_api_key_concurrency_limit(1),
        CommercialAdmissionPolicy::new("provider-live-cap", "project-live")
            .with_provider_id("provider-live")
            .with_provider_concurrency_limit(1),
    ]);

    let request_context = GatewayTrafficRequestContext::new(
        "tenant-live",
        "project-live",
        &hashed_key,
        "chat.completions",
    )
    .with_api_key_group_id_option(Some("group-live".to_owned()))
    .with_model_name_option(Some("gpt-4.1".to_owned()));

    let _request_permit = controller
        .acquire_request_admission(&request_context)
        .await
        .unwrap();
    let _provider_permit = controller
        .acquire_provider_admission(&request_context, "provider-live")
        .await
        .unwrap();

    let app = sdkwork_api_interface_admin::admin_router_with_state(
        sdkwork_api_interface_admin::AdminApiState::with_store_and_secret_manager_and_jwt_secret_and_traffic_controller(
            store,
            CredentialSecretManager::database_encrypted("local-dev-master-key"),
            "admin-live-pressure-secret",
            controller,
        ),
    );
    let token = issue_admin_token(app.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/gateway/traffic-pressure")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["snapshot_count"], 3);
    assert_eq!(json["saturated_snapshot_count"], 3);
    assert_eq!(json["pressured_projects"].as_array().unwrap().len(), 1);
    assert_eq!(json["throttled_api_keys"].as_array().unwrap().len(), 1);
    assert_eq!(json["saturated_providers"].as_array().unwrap().len(), 1);
    assert_eq!(json["pressured_projects"][0]["project_id"], "project-live");
    assert_eq!(json["pressured_projects"][0]["scope_kind"], "project");
    assert_eq!(json["pressured_projects"][0]["current_in_flight"], 1);
    assert_eq!(json["pressured_projects"][0]["remaining"], 0);
    assert_eq!(json["throttled_api_keys"][0]["scope_kind"], "api_key");
    assert_eq!(json["throttled_api_keys"][0]["api_key_hash"], hashed_key);
    assert_eq!(json["saturated_providers"][0]["scope_kind"], "provider");
    assert_eq!(
        json["saturated_providers"][0]["provider_id"],
        "provider-live"
    );
}
