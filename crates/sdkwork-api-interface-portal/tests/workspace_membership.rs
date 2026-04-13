use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use sdkwork_api_app_identity::verify_portal_jwt;
use sdkwork_api_domain_identity::PortalWorkspaceMembershipRecord;
use sdkwork_api_domain_tenant::{Project, Tenant};
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use serde_json::Value;
use sqlx::SqlitePool;
use std::sync::Arc;
use tower::ServiceExt;

const TEST_PORTAL_JWT_SECRET: &str = "portal-workspace-membership-secret";

async fn read_json(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn memory_pool() -> SqlitePool {
    sdkwork_api_storage_sqlite::run_migrations("sqlite::memory:")
        .await
        .unwrap()
}

async fn register_portal_user(app: axum::Router) -> Value {
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"email":"membership@example.com","password":"PortalPass123!","display_name":"Membership User"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    read_json(response).await
}

#[tokio::test]
async fn portal_lists_workspace_memberships_and_switches_active_workspace() {
    let pool = memory_pool().await;
    let store = Arc::new(SqliteAdminStore::new(pool));
    let app = sdkwork_api_interface_portal::portal_router_with_store_and_jwt_secret(
        store.clone(),
        TEST_PORTAL_JWT_SECRET,
    );

    let registration = register_portal_user(app.clone()).await;
    let user_id = registration["user"]["id"].as_str().unwrap().to_owned();
    let original_token = registration["token"].as_str().unwrap().to_owned();
    let original_tenant_id = registration["workspace"]["tenant_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let original_project_id = registration["workspace"]["project_id"]
        .as_str()
        .unwrap()
        .to_owned();

    let tenant = Tenant::new("tenant_enterprise_ops", "Enterprise Ops");
    let project = Project::new("tenant_enterprise_ops", "project_enterprise_ops", "ops");
    store.insert_tenant(&tenant).await.unwrap();
    store.insert_project(&project).await.unwrap();
    store
        .insert_portal_workspace_membership(&PortalWorkspaceMembershipRecord::new(
            "membership_enterprise_ops",
            &user_id,
            &tenant.id,
            &project.id,
            1_710_000_000_000,
        ))
        .await
        .unwrap();

    let workspace_list = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/workspaces")
                .header("authorization", format!("Bearer {original_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(workspace_list.status(), StatusCode::OK);
    let workspaces_json = read_json(workspace_list).await;
    assert_eq!(workspaces_json.as_array().unwrap().len(), 2);
    assert!(workspaces_json
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["tenant"]["id"] == original_tenant_id
            && item["project"]["id"] == original_project_id
            && item["current"] == true
            && item["role"] == "owner"));
    assert!(workspaces_json
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["tenant"]["id"] == "tenant_enterprise_ops"
            && item["project"]["id"] == "project_enterprise_ops"
            && item["current"] == false
            && item["role"] == "member"));

    let switch_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/workspaces/select")
                .header("authorization", format!("Bearer {original_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"tenant_id":"tenant_enterprise_ops","project_id":"project_enterprise_ops"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(switch_response.status(), StatusCode::OK);
    let switch_json = read_json(switch_response).await;
    let switched_token = switch_json["token"].as_str().unwrap().to_owned();
    assert_eq!(
        switch_json["workspace"]["tenant_id"],
        "tenant_enterprise_ops"
    );
    assert_eq!(
        switch_json["workspace"]["project_id"],
        "project_enterprise_ops"
    );
    assert_eq!(
        switch_json["user"]["workspace_tenant_id"],
        "tenant_enterprise_ops"
    );
    assert_eq!(
        switch_json["user"]["workspace_project_id"],
        "project_enterprise_ops"
    );

    let switched_claims = verify_portal_jwt(&switched_token, TEST_PORTAL_JWT_SECRET).unwrap();
    assert_eq!(switched_claims.workspace_tenant_id, "tenant_enterprise_ops");
    assert_eq!(
        switched_claims.workspace_project_id,
        "project_enterprise_ops"
    );

    let stale_workspace = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/workspace")
                .header("authorization", format!("Bearer {original_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(stale_workspace.status(), StatusCode::UNAUTHORIZED);

    let selected_workspace = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/workspace")
                .header("authorization", format!("Bearer {switched_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(selected_workspace.status(), StatusCode::OK);
    let selected_workspace_json = read_json(selected_workspace).await;
    assert_eq!(
        selected_workspace_json["tenant"]["id"],
        "tenant_enterprise_ops"
    );
    assert_eq!(
        selected_workspace_json["project"]["id"],
        "project_enterprise_ops"
    );

    let create_key = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/api-keys")
                .header("authorization", format!("Bearer {switched_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"environment":"live","label":"Enterprise Ops Key"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_key.status(), StatusCode::CREATED);

    let list_keys = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/portal/api-keys")
                .header("authorization", format!("Bearer {switched_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_keys.status(), StatusCode::OK);
    let keys_json = read_json(list_keys).await;
    assert_eq!(keys_json.as_array().unwrap().len(), 1);
    assert_eq!(keys_json[0]["tenant_id"], "tenant_enterprise_ops");
    assert_eq!(keys_json[0]["project_id"], "project_enterprise_ops");
}

#[tokio::test]
async fn portal_workspace_switch_rejects_non_member_targets() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_portal::portal_router_with_pool_and_jwt_secret(
        pool,
        TEST_PORTAL_JWT_SECRET,
    );

    let registration = register_portal_user(app.clone()).await;
    let token = registration["token"].as_str().unwrap();

    let switch_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/portal/workspaces/select")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"tenant_id":"tenant_unknown","project_id":"project_unknown"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(switch_response.status(), StatusCode::NOT_FOUND);
}
