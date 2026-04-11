use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use sdkwork_api_app_catalog::{
    create_provider_with_config, create_provider_with_default_plugin_family_and_bindings,
};
use sdkwork_api_domain_catalog::{Channel, ModelCatalogEntry, ProviderChannelBinding};
use sdkwork_api_domain_credential::UpstreamCredential;
use sdkwork_api_domain_routing::{
    CompiledRoutingSnapshotRecord, RoutingDecisionLog, RoutingDecisionSource,
};
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use serde_json::Value;
use sqlx::SqlitePool;
use tower::ServiceExt;

async fn read_json(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn find_provider_option<'a>(summary_json: &'a Value, provider_id: &str) -> &'a Value {
    summary_json["provider_options"]
        .as_array()
        .unwrap()
        .iter()
        .find(|option| option["provider_id"] == provider_id)
        .unwrap_or_else(|| panic!("missing provider option for {provider_id}"))
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
                    "{\"email\":\"portal@example.com\",\"password\":\"PortalPass123!\",\"display_name\":\"Portal User\"}",
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
    let openrouter_provider = create_provider_with_default_plugin_family_and_bindings(
        "provider-openrouter",
        "openrouter",
        "openrouter",
        "https://openrouter.example/v1",
        "OpenRouter",
        &[ProviderChannelBinding::new("provider-openrouter", "openai")],
    )
    .unwrap();
    assert_eq!(openrouter_provider.channel_id, "openrouter");
    assert_eq!(openrouter_provider.adapter_kind, "openrouter");
    assert_eq!(openrouter_provider.protocol_kind(), "openai");
    assert_eq!(
        openrouter_provider.extension_id,
        "sdkwork.provider.openrouter"
    );
    assert!(openrouter_provider
        .channel_bindings
        .contains(&ProviderChannelBinding::new(
            "provider-openrouter",
            "openai"
        )));
    store.insert_provider(&openrouter_provider).await.unwrap();
    store
        .insert_provider(
            &create_provider_with_config(
                "provider-openai-official",
                "openai",
                "openai",
                "https://openai.example/v1",
                "OpenAI Official",
            )
            .unwrap(),
        )
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
    let tenant_id = workspace_json["tenant"]["id"].as_str().unwrap().to_owned();
    let project_id = workspace_json["project"]["id"].as_str().unwrap().to_owned();

    store
        .insert_credential(&UpstreamCredential::new(
            &tenant_id,
            "provider-openrouter",
            "cred-openrouter",
        ))
        .await
        .unwrap();
    store
        .insert_credential(&UpstreamCredential::new(
            "tenant-other",
            "provider-openai-official",
            "cred-openai-other",
        ))
        .await
        .unwrap();

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
                    "{\"preset_id\":\"reliability\",\"strategy\":\"geo_affinity\",\"ordered_provider_ids\":[\"provider-openrouter\",\"provider-openai-official\"],\"default_provider_id\":\"provider-openai-official\",\"max_cost\":0.3,\"max_latency_ms\":250,\"require_healthy\":true,\"preferred_region\":\"us-east\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(preferences_response.status(), StatusCode::OK);
    let preferences_json = read_json(preferences_response).await;
    assert_eq!(preferences_json["project_id"], project_id);
    assert_eq!(preferences_json["preset_id"], "reliability");
    assert_eq!(preferences_json["strategy"], "geo_affinity");
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
    assert_eq!(preview_json["strategy"], "geo_affinity");
    let preview_snapshot_id = preview_json["compiled_routing_snapshot_id"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(preview_snapshot_id.starts_with("routing-snapshot-"));
    assert_eq!(
        preview_json["fallback_reason"],
        "no candidate matched requested region us-east"
    );

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
    assert_eq!(
        logs_json[0]["compiled_routing_snapshot_id"],
        preview_snapshot_id
    );
    assert_eq!(logs_json[0]["requested_region"], "us-east");
    assert_eq!(
        logs_json[0]["fallback_reason"],
        "no candidate matched requested region us-east"
    );

    let summary_response = app
        .clone()
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
    assert_eq!(summary_json["preview"]["strategy"], "geo_affinity");
    assert_eq!(
        summary_json["preview"]["compiled_routing_snapshot_id"],
        preview_snapshot_id
    );
    assert_eq!(summary_json["preview"]["requested_region"], "us-east");
    assert_eq!(
        summary_json["preview"]["fallback_reason"],
        "no candidate matched requested region us-east"
    );
    assert_eq!(
        summary_json["provider_options"].as_array().unwrap().len(),
        2
    );
    let openrouter_option = find_provider_option(&summary_json, "provider-openrouter");
    assert_eq!(openrouter_option["protocol_kind"], "openai");
    assert_eq!(openrouter_option["integration"]["mode"], "default_plugin");
    assert_eq!(
        openrouter_option["integration"]["default_plugin_family"],
        "openrouter"
    );
    assert_eq!(openrouter_option["credential_readiness"]["ready"], true);
    assert_eq!(
        openrouter_option["credential_readiness"]["state"],
        "configured"
    );
    let openai_option = find_provider_option(&summary_json, "provider-openai-official");
    assert_eq!(openai_option["protocol_kind"], "openai");
    assert_eq!(
        openai_option["integration"]["mode"],
        "standard_passthrough"
    );
    assert!(openai_option["integration"]["default_plugin_family"].is_null());
    assert_eq!(openai_option["credential_readiness"]["ready"], false);
    assert_eq!(openai_option["credential_readiness"]["state"], "missing");

    let snapshots_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/routing/snapshots")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(snapshots_response.status(), StatusCode::OK);
    let snapshots_json = read_json(snapshots_response).await;
    let snapshots = snapshots_json.as_array().unwrap();
    assert_eq!(snapshots.len(), 1);
    assert_eq!(snapshots[0]["snapshot_id"], preview_snapshot_id);
    assert_eq!(snapshots[0]["project_id"], project_id);
    assert_eq!(snapshots[0]["strategy"], "geo_affinity");
    assert_eq!(snapshots[0]["preferred_region"], "us-east");
}

#[tokio::test]
async fn portal_routing_snapshots_are_workspace_scoped() {
    let pool = memory_pool().await;
    let store = Arc::new(SqliteAdminStore::new(pool));

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
    let tenant_id = workspace_json["tenant"]["id"].as_str().unwrap().to_owned();
    let project_id = workspace_json["project"]["id"].as_str().unwrap().to_owned();

    store
        .insert_compiled_routing_snapshot(
            &CompiledRoutingSnapshotRecord::new("snapshot-workspace", "chat_completion", "gpt-4.1")
                .with_tenant_id(&tenant_id)
                .with_project_id(&project_id)
                .with_api_key_group_id("group-live")
                .with_matched_policy_id("policy-live")
                .with_applied_routing_profile_id("profile-live")
                .with_strategy("geo_affinity")
                .with_ordered_provider_ids(vec![
                    "provider-openai".to_owned(),
                    "provider-anthropic".to_owned(),
                ])
                .with_default_provider_id("provider-openai")
                .with_max_latency_ms(800)
                .with_require_healthy(true)
                .with_preferred_region("us-east")
                .with_created_at_ms(1_700_000_000_000)
                .with_updated_at_ms(1_700_000_000_100),
        )
        .await
        .unwrap();
    store
        .insert_compiled_routing_snapshot(
            &CompiledRoutingSnapshotRecord::new("snapshot-foreign", "chat_completion", "gpt-4.1")
                .with_tenant_id("tenant-other")
                .with_project_id("project-other")
                .with_strategy("weighted_random")
                .with_ordered_provider_ids(vec!["provider-other".to_owned()])
                .with_created_at_ms(1_700_000_000_000)
                .with_updated_at_ms(1_700_000_000_100),
        )
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/routing/snapshots")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let snapshots_json = read_json(response).await;
    let snapshots = snapshots_json.as_array().unwrap();

    assert_eq!(snapshots.len(), 1);
    assert_eq!(snapshots[0]["snapshot_id"], "snapshot-workspace");
    assert_eq!(snapshots[0]["tenant_id"], tenant_id);
    assert_eq!(snapshots[0]["project_id"], project_id);
    assert_eq!(snapshots[0]["api_key_group_id"], "group-live");
    assert_eq!(snapshots[0]["matched_policy_id"], "policy-live");
    assert_eq!(snapshots[0]["applied_routing_profile_id"], "profile-live");
    assert_eq!(snapshots[0]["strategy"], "geo_affinity");
    assert_eq!(snapshots[0]["default_provider_id"], "provider-openai");
    assert_eq!(snapshots[0]["preferred_region"], "us-east");
    assert_eq!(snapshots[0]["require_healthy"], true);
}

#[tokio::test]
async fn portal_routing_routes_require_authentication() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool(pool);

    for route in [
        "/portal/routing/summary",
        "/portal/routing/preferences",
        "/portal/routing/decision-logs",
        "/portal/routing/snapshots",
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
