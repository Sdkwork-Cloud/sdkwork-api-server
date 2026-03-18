use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use sdkwork_api_domain_catalog::{Channel, ModelCatalogEntry, ProxyProvider};
use sdkwork_api_domain_routing::{RoutingDecisionLog, RoutingDecisionSource};
use sdkwork_api_storage_sqlite::SqliteAdminStore;
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
async fn portal_routing_preferences_preview_and_logs_are_project_scoped() {
    let pool = memory_pool().await;
    let store = Arc::new(SqliteAdminStore::new(pool));
    store
        .insert_channel(&Channel::new("openai", "OpenAI"))
        .await
        .unwrap();
    store
        .insert_provider(&ProxyProvider::new(
            "provider-openrouter",
            "openai",
            "openai",
            "https://openrouter.example/v1",
            "OpenRouter",
        ))
        .await
        .unwrap();
    store
        .insert_provider(&ProxyProvider::new(
            "provider-openai-official",
            "openai",
            "openai",
            "https://openai.example/v1",
            "OpenAI Official",
        ))
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new("gpt-4.1", "provider-openrouter"))
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new(
            "gpt-4.1",
            "provider-openai-official",
        ))
        .await
        .unwrap();

    let app = sdkwork_api_interface_portal::portal_router_with_store(store.clone());
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
    let workspace_json = read_json(workspace_response).await;
    let project_id = workspace_json["project"]["id"].as_str().unwrap().to_owned();

    store
        .insert_routing_decision_log(
            &RoutingDecisionLog::new(
                "decision-other",
                RoutingDecisionSource::Gateway,
                "chat_completion",
                "gpt-4.1",
                "provider-openrouter",
                "deterministic_priority",
                100,
            )
            .with_project_id("project-other")
            .with_tenant_id("tenant-other"),
        )
        .await
        .unwrap();

    let preferences_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/routing/preferences")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"preset_id\":\"reliability\",\"strategy\":\"slo_aware\",\"ordered_provider_ids\":[\"provider-openrouter\",\"provider-openai-official\"],\"default_provider_id\":\"provider-openai-official\",\"max_cost\":0.3,\"max_latency_ms\":250,\"require_healthy\":true,\"preferred_region\":\"us-east\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(preferences_response.status(), StatusCode::OK);
    let preferences_json = read_json(preferences_response).await;
    assert_eq!(preferences_json["project_id"], project_id);
    assert_eq!(preferences_json["preset_id"], "reliability");
    assert_eq!(preferences_json["strategy"], "slo_aware");
    assert_eq!(preferences_json["preferred_region"], "us-east");

    let get_preferences_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/routing/preferences")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(get_preferences_response.status(), StatusCode::OK);
    let get_preferences_json = read_json(get_preferences_response).await;
    assert_eq!(get_preferences_json["project_id"], project_id);
    assert_eq!(
        get_preferences_json["ordered_provider_ids"][0],
        "provider-openrouter"
    );

    let preview_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/routing/preview")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"capability\":\"chat_completion\",\"model\":\"gpt-4.1\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(preview_response.status(), StatusCode::OK);
    let preview_json = read_json(preview_response).await;
    assert_eq!(preview_json["selected_provider_id"], "provider-openrouter");
    assert_eq!(preview_json["requested_region"], "us-east");
    assert_eq!(preview_json["strategy"], "slo_aware");

    let logs_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/routing/decision-logs")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(logs_response.status(), StatusCode::OK);
    let logs_json = read_json(logs_response).await;
    assert_eq!(logs_json.as_array().unwrap().len(), 1);
    assert_eq!(logs_json[0]["project_id"], project_id);
    assert_eq!(logs_json[0]["decision_source"], "portal_simulation");

    let summary_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/routing/summary")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(summary_response.status(), StatusCode::OK);
    let summary_json = read_json(summary_response).await;
    assert_eq!(summary_json["project_id"], project_id);
    assert_eq!(summary_json["preferences"]["preset_id"], "reliability");
    assert_eq!(summary_json["latest_model_hint"], "gpt-4.1");
    assert_eq!(
        summary_json["preview"]["selected_provider_id"],
        "provider-openrouter"
    );
    assert_eq!(
        summary_json["provider_options"].as_array().unwrap().len(),
        2
    );
}

#[tokio::test]
async fn portal_routing_routes_require_authentication() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);

    for route in [
        "/portal/routing/summary",
        "/portal/routing/preferences",
        "/portal/routing/decision-logs",
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(route)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "{route}");
    }
}
