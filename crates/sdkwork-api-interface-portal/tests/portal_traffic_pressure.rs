use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
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

async fn portal_token(app: axum::Router) -> String {
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"email\":\"traffic@example.com\",\"password\":\"hunter2!\",\"display_name\":\"Traffic User\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    read_json(response).await["token"]
        .as_str()
        .unwrap()
        .to_owned()
}

async fn workspace_json(app: axum::Router, token: &str) -> Value {
    let response = app
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

    assert_eq!(response.status(), StatusCode::OK);
    read_json(response).await
}

#[tokio::test]
async fn portal_gateway_traffic_pressure_is_filtered_to_the_current_workspace_project() {
    let pool = memory_pool().await;
    let store = Arc::new(SqliteAdminStore::new(pool)) as Arc<dyn AdminStore>;
    let controller = Arc::new(InMemoryGatewayTrafficController::new());
    let app = sdkwork_api_interface_portal::portal_router_with_state(
        sdkwork_api_interface_portal::PortalApiState::with_store_and_jwt_secret_and_traffic_controller(
            store,
            "portal-live-pressure-secret",
            controller.clone(),
        ),
    );

    let token = portal_token(app.clone()).await;
    let workspace = workspace_json(app.clone(), &token).await;
    let project_id = workspace["project"]["id"].as_str().unwrap().to_owned();
    let tenant_id = workspace["tenant"]["id"].as_str().unwrap().to_owned();
    let hashed_key = hash_gateway_api_key("sk-live-pressure-portal");
    let foreign_hashed_key = hash_gateway_api_key("sk-live-pressure-foreign");

    controller.replace_policies(vec![
        CommercialAdmissionPolicy::new("project-current-cap", &project_id)
            .with_project_concurrency_limit(1),
        CommercialAdmissionPolicy::new("key-current-cap", &project_id)
            .with_api_key_hash(&hashed_key)
            .with_api_key_concurrency_limit(1),
        CommercialAdmissionPolicy::new("provider-current-cap", &project_id)
            .with_provider_id("provider-current")
            .with_provider_concurrency_limit(1),
        CommercialAdmissionPolicy::new("provider-foreign-cap", "project-foreign")
            .with_api_key_hash(&foreign_hashed_key)
            .with_provider_id("provider-foreign")
            .with_provider_concurrency_limit(1),
    ]);

    let current_context =
        GatewayTrafficRequestContext::new(&tenant_id, &project_id, &hashed_key, "responses")
            .with_api_key_group_id_option(Some("group-current".to_owned()))
            .with_model_name_option(Some("gpt-4.1".to_owned()));
    let foreign_context = GatewayTrafficRequestContext::new(
        "tenant-foreign",
        "project-foreign",
        &foreign_hashed_key,
        "responses",
    )
    .with_api_key_group_id_option(Some("group-foreign".to_owned()))
    .with_model_name_option(Some("gpt-4.1-mini".to_owned()));

    let _current_request_permit = controller
        .acquire_request_admission(&current_context)
        .await
        .unwrap();
    let _current_provider_permit = controller
        .acquire_provider_admission(&current_context, "provider-current")
        .await
        .unwrap();
    let _foreign_provider_permit = controller
        .acquire_provider_admission(&foreign_context, "provider-foreign")
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/gateway/traffic-pressure")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["project_id"], project_id);
    assert_eq!(json["snapshot_count"], 3);
    assert_eq!(json["saturated_snapshot_count"], 3);
    assert_eq!(json["throttled_api_key_count"], 1);
    assert_eq!(json["saturated_provider_count"], 1);
    assert_eq!(json["snapshots"].as_array().unwrap().len(), 3);
    assert_eq!(json["throttled_api_keys"].as_array().unwrap().len(), 1);
    assert_eq!(json["saturated_providers"].as_array().unwrap().len(), 1);
    assert_eq!(
        json["saturated_providers"][0]["provider_id"],
        "provider-current"
    );
    assert!(json["detail"].as_str().unwrap().contains("project"));
    assert!(json["snapshots"]
        .as_array()
        .unwrap()
        .iter()
        .all(|snapshot| snapshot["project_id"] == project_id));
}
