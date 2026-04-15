use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use sdkwork_api_app_identity::{hash_gateway_api_key, resolve_gateway_request_context};
use sdkwork_api_domain_identity::{ApiKeyGroupRecord, GatewayApiKeyRecord};
use sdkwork_api_domain_routing::{RoutingProfileRecord, RoutingStrategy};
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

#[tokio::test]
async fn portal_api_key_groups_are_workspace_scoped_and_can_bind_api_keys() {
    let pool = memory_pool().await;
    let store = std::sync::Arc::new(sdkwork_api_storage_sqlite::SqliteAdminStore::new(pool));
    let app = sdkwork_api_interface_portal::portal_router_with_store(store.clone());
    let token = portal_token(app.clone()).await;

    store
        .insert_api_key_group(
            &ApiKeyGroupRecord::new(
                "foreign-group",
                "tenant-other",
                "project-other",
                "live",
                "Foreign Keys",
                "foreign-keys",
            )
            .with_created_at_ms(1_700_000_000_000)
            .with_updated_at_ms(1_700_000_000_000),
        )
        .await
        .unwrap();

    let created_group = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/api-key-groups")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"environment":"live","name":"Production Keys","description":"Primary production pool","default_accounting_mode":"byok"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(created_group.status(), StatusCode::CREATED);
    let created_group_json = read_json(created_group).await;
    let group_id = created_group_json["group_id"].as_str().unwrap().to_owned();
    assert_eq!(created_group_json["slug"], "production-keys");
    assert_eq!(created_group_json["default_accounting_mode"], "byok");

    let listed_groups = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/api-key-groups")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(listed_groups.status(), StatusCode::OK);
    let listed_groups_json = read_json(listed_groups).await;
    assert_eq!(listed_groups_json.as_array().unwrap().len(), 1);
    assert_eq!(listed_groups_json[0]["group_id"], group_id);
    assert_eq!(listed_groups_json[0]["default_accounting_mode"], "byok");

    let create_key_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/api-keys")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"environment":"live","label":"Production rollout","api_key_group_id":"{group_id}"}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_key_response.status(), StatusCode::CREATED);
    let created_key_json = read_json(create_key_response).await;
    let hashed_key = created_key_json["hashed"].as_str().unwrap().to_owned();
    assert_eq!(created_key_json["api_key_group_id"], group_id);

    let persisted = store
        .find_gateway_api_key(&hashed_key)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        persisted.api_key_group_id.as_deref(),
        Some(group_id.as_str())
    );
}

#[tokio::test]
async fn portal_api_key_group_updates_and_status_changes_are_workspace_scoped() {
    let pool = memory_pool().await;
    let store = std::sync::Arc::new(sdkwork_api_storage_sqlite::SqliteAdminStore::new(pool));
    let app = sdkwork_api_interface_portal::portal_router_with_store(store.clone());
    let token = portal_token(app.clone()).await;

    let created_group = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/api-key-groups")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"environment":"live","name":"Production Keys"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(created_group.status(), StatusCode::CREATED);
    let group_id = read_json(created_group).await["group_id"]
        .as_str()
        .unwrap()
        .to_owned();

    store
        .insert_api_key_group(
            &ApiKeyGroupRecord::new(
                "foreign-group",
                "tenant-other",
                "project-other",
                "live",
                "Foreign Keys",
                "foreign-keys",
            )
            .with_created_at_ms(1_700_000_000_000)
            .with_updated_at_ms(1_700_000_000_000),
        )
        .await
        .unwrap();

    let updated = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/portal/api-key-groups/{group_id}"))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"environment":"live","name":"VIP Keys","slug":"vip-keys","description":"Premium pool"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(updated.status(), StatusCode::OK);
    let updated_json = read_json(updated).await;
    assert_eq!(updated_json["slug"], "vip-keys");
    assert_eq!(updated_json["description"], "Premium pool");

    let deactivated = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/portal/api-key-groups/{group_id}/status"))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"active":false}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(deactivated.status(), StatusCode::OK);
    assert_eq!(read_json(deactivated).await["active"], false);

    let foreign_status = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/api-key-groups/foreign-group/status")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"active":false}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(foreign_status.status(), StatusCode::NOT_FOUND);

    let foreign_delete = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/portal/api-key-groups/foreign-group")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(foreign_delete.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn portal_routing_profiles_are_workspace_scoped_and_creatable_for_group_binding() {
    let pool = memory_pool().await;
    let store = std::sync::Arc::new(sdkwork_api_storage_sqlite::SqliteAdminStore::new(pool));
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
    assert_eq!(workspace_response.status(), StatusCode::OK);
    let workspace_json = read_json(workspace_response).await;
    let tenant_id = workspace_json["tenant"]["id"].as_str().unwrap().to_owned();
    let project_id = workspace_json["project"]["id"].as_str().unwrap().to_owned();

    store
        .insert_routing_profile(
            &RoutingProfileRecord::new(
                "profile-live",
                &tenant_id,
                &project_id,
                "Live profile",
                "live-profile",
            )
            .with_strategy(RoutingStrategy::GeoAffinity)
            .with_preferred_region("us-east")
            .with_updated_at_ms(1_700_000_000_200),
        )
        .await
        .unwrap();
    store
        .insert_routing_profile(
            &RoutingProfileRecord::new(
                "profile-inactive",
                &tenant_id,
                &project_id,
                "Inactive profile",
                "inactive-profile",
            )
            .with_active(false)
            .with_strategy(RoutingStrategy::DeterministicPriority)
            .with_updated_at_ms(1_700_000_000_100),
        )
        .await
        .unwrap();
    store
        .insert_routing_profile(
            &RoutingProfileRecord::new(
                "foreign-profile",
                "tenant-other",
                "project-other",
                "Foreign profile",
                "foreign-profile",
            )
            .with_updated_at_ms(1_700_000_000_300),
        )
        .await
        .unwrap();

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/routing/profiles")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name":"Balanced live posture","description":"Created from portal routing posture","strategy":"geo_affinity","ordered_provider_ids":["provider-openai","provider-anthropic"],"default_provider_id":"provider-openai","max_cost":0.4,"max_latency_ms":700,"require_healthy":true,"preferred_region":"us-east"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_response.status(), StatusCode::CREATED);
    let created_profile_json = read_json(create_response).await;
    assert!(created_profile_json["profile_id"]
        .as_str()
        .unwrap()
        .starts_with("routing-profile-"));
    assert_eq!(created_profile_json["tenant_id"], tenant_id);
    assert_eq!(created_profile_json["project_id"], project_id);
    assert_eq!(created_profile_json["name"], "Balanced live posture");
    assert_eq!(created_profile_json["slug"], "balanced-live-posture");
    assert_eq!(
        created_profile_json["description"],
        "Created from portal routing posture"
    );
    assert_eq!(created_profile_json["strategy"], "geo_affinity");
    assert_eq!(
        created_profile_json["default_provider_id"],
        "provider-openai"
    );
    assert_eq!(created_profile_json["max_cost"], 0.4);
    assert_eq!(created_profile_json["max_latency_ms"], 700);
    assert_eq!(created_profile_json["require_healthy"], true);
    assert_eq!(created_profile_json["preferred_region"], "us-east");

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/routing/profiles")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let profiles_json = read_json(response).await;
    let profiles = profiles_json.as_array().unwrap();

    assert_eq!(profiles.len(), 3);
    assert!(profiles.iter().any(|profile| {
        profile["profile_id"].as_str() == Some("profile-live")
            && profile["tenant_id"].as_str() == Some(tenant_id.as_str())
            && profile["project_id"].as_str() == Some(project_id.as_str())
    }));
    assert!(profiles.iter().any(|profile| {
        profile["name"].as_str() == Some("Balanced live posture")
            && profile["slug"].as_str() == Some("balanced-live-posture")
            && profile["tenant_id"].as_str() == Some(tenant_id.as_str())
            && profile["project_id"].as_str() == Some(project_id.as_str())
    }));
    assert!(profiles.iter().any(|profile| {
        profile["profile_id"].as_str() == Some("profile-inactive")
            && !profile["active"].as_bool().unwrap_or(false)
    }));
    assert!(profiles.iter().all(|profile| {
        profile["tenant_id"].as_str() == Some(tenant_id.as_str())
            && profile["project_id"].as_str() == Some(project_id.as_str())
    }));
}
