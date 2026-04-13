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

async fn login_token(app: Router, email: &str, password: &str) -> Value {
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"email":"{email}","password":"{password}"}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    read_json(response).await
}

async fn create_operator(
    app: Router,
    token: &str,
    email: &str,
    display_name: &str,
    password: &str,
    role: &str,
) -> Value {
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/users/operators")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"email":"{email}","display_name":"{display_name}","password":"{password}","active":true,"role":"{role}"}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    read_json(response).await
}

async fn seed_workspace(app: Router, token: &str, tenant_id: &str, project_id: &str) {
    let create_tenant = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/tenants")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"id":"{tenant_id}","name":"{tenant_id}"}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_tenant.status(), StatusCode::CREATED);

    let create_project = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/projects")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"tenant_id":"{tenant_id}","id":"{project_id}","name":"{project_id}"}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_project.status(), StatusCode::CREATED);
}

async fn seed_provider_and_model(app: Router, token: &str, provider_id: &str, model_id: &str) {
    let create_channel = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/channels")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"id":"openai","name":"OpenAI"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_channel.status(), StatusCode::CREATED);

    let create_provider = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"id":"{provider_id}","channel_id":"openai","adapter_kind":"openai","base_url":"https://api.openai.com","display_name":"Seed Provider","channel_bindings":[{{"channel_id":"openai","is_primary":true}}]}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_provider.status(), StatusCode::CREATED);

    let create_model = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"external_name":"{model_id}","provider_id":"{provider_id}"}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_model.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn finance_operator_can_access_billing_but_cannot_manage_platform_or_identities() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let admin_login = login_token(app.clone(), "admin@sdkwork.local", "ChangeMe123!").await;
    let admin_token = admin_login["token"].as_str().unwrap();

    let finance_user = create_operator(
        app.clone(),
        admin_token,
        "finance@example.com",
        "Finance Operator",
        "FinancePass123!",
        "finance_operator",
    )
    .await;
    assert_eq!(finance_user["role"], "finance_operator");

    let finance_login = login_token(app.clone(), "finance@example.com", "FinancePass123!").await;
    let finance_token = finance_login["token"].as_str().unwrap();
    assert_eq!(finance_login["user"]["role"], "finance_operator");

    let billing_summary = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/billing/summary")
                .header("authorization", format!("Bearer {finance_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(billing_summary.status(), StatusCode::OK);

    let create_provider = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {finance_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"id":"provider-finance-test","channel_id":"openai","adapter_kind":"openai","base_url":"https://api.openai.com","display_name":"Finance Should Not Create","channel_bindings":[{"channel_id":"openai","is_primary":true}]}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_provider.status(), StatusCode::FORBIDDEN);

    let operator_list = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/users/operators")
                .header("authorization", format!("Bearer {finance_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(operator_list.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn read_only_operator_can_read_catalog_but_cannot_write_or_access_credentials() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let admin_login = login_token(app.clone(), "admin@sdkwork.local", "ChangeMe123!").await;
    let admin_token = admin_login["token"].as_str().unwrap();

    let read_only_user = create_operator(
        app.clone(),
        admin_token,
        "readonly@example.com",
        "Read Only Operator",
        "ReadOnlyPass123!",
        "read_only_operator",
    )
    .await;
    assert_eq!(read_only_user["role"], "read_only_operator");

    let read_only_login =
        login_token(app.clone(), "readonly@example.com", "ReadOnlyPass123!").await;
    let read_only_token = read_only_login["token"].as_str().unwrap();
    assert_eq!(read_only_login["user"]["role"], "read_only_operator");

    let list_providers = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {read_only_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_providers.status(), StatusCode::OK);

    let create_provider = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {read_only_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"id":"provider-read-only-test","channel_id":"openai","adapter_kind":"openai","base_url":"https://api.openai.com","display_name":"Read Only Should Not Create","channel_bindings":[{"channel_id":"openai","is_primary":true}]}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_provider.status(), StatusCode::FORBIDDEN);

    let credentials = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {read_only_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(credentials.status(), StatusCode::FORBIDDEN);

    let create_operator = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/users/operators")
                .header("authorization", format!("Bearer {read_only_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"email":"blocked@example.com","display_name":"Blocked","password":"BlockedPass123!","active":true,"role":"platform_operator"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_operator.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn platform_operator_cannot_mutate_model_prices() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let admin_login = login_token(app.clone(), "admin@sdkwork.local", "ChangeMe123!").await;
    let admin_token = admin_login["token"].as_str().unwrap();

    seed_provider_and_model(
        app.clone(),
        admin_token,
        "provider-platform-price",
        "gpt-4.1",
    )
    .await;

    let platform_user = create_operator(
        app.clone(),
        admin_token,
        "platform@example.com",
        "Platform Operator",
        "PlatformPass123!",
        "platform_operator",
    )
    .await;
    assert_eq!(platform_user["role"], "platform_operator");

    let platform_login = login_token(app.clone(), "platform@example.com", "PlatformPass123!").await;
    let platform_token = platform_login["token"].as_str().unwrap();

    let create_price = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/model-prices")
                .header("authorization", format!("Bearer {platform_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"channel_id":"openai","model_id":"gpt-4.1","proxy_provider_id":"provider-platform-price","currency_code":"USD","price_unit":"per_1m_tokens","input_price":2.5,"output_price":10.0,"cache_read_price":0.0,"cache_write_price":0.0,"request_price":0.01,"is_active":true}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_price.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn finance_operator_model_price_mutation_is_audited() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let admin_login = login_token(app.clone(), "admin@sdkwork.local", "ChangeMe123!").await;
    let admin_token = admin_login["token"].as_str().unwrap();

    seed_provider_and_model(
        app.clone(),
        admin_token,
        "provider-finance-price",
        "gpt-4.1",
    )
    .await;

    let finance_user = create_operator(
        app.clone(),
        admin_token,
        "pricing-finance@example.com",
        "Pricing Finance",
        "FinancePass123!",
        "finance_operator",
    )
    .await;

    let finance_login = login_token(
        app.clone(),
        "pricing-finance@example.com",
        "FinancePass123!",
    )
    .await;
    let finance_token = finance_login["token"].as_str().unwrap();

    let create_price = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/model-prices")
                .header("authorization", format!("Bearer {finance_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"channel_id":"openai","model_id":"gpt-4.1","proxy_provider_id":"provider-finance-price","currency_code":"USD","price_unit":"per_1m_tokens","input_price":2.5,"output_price":10.0,"cache_read_price":0.0,"cache_write_price":0.0,"request_price":0.01,"is_active":true}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_price.status(), StatusCode::CREATED);

    let audit_events = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/audit/events")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(audit_events.status(), StatusCode::OK);
    let audit_json = read_json(audit_events).await;
    let events = audit_json.as_array().unwrap();
    let event = events
        .iter()
        .find(|event| event["action"] == "model_price.create")
        .expect("expected model price audit event");
    assert_eq!(event["resource_type"], "model_price");
    assert_eq!(
        event["resource_id"],
        "openai:gpt-4.1:provider-finance-price"
    );
    assert_eq!(event["approval_scope"], "finance_control");
    assert_eq!(event["actor_user_id"], finance_user["id"]);
    assert_eq!(event["actor_email"], "pricing-finance@example.com");
    assert_eq!(event["actor_role"], "finance_operator");
}

#[tokio::test]
async fn super_admin_credential_mutation_is_audited() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let admin_login = login_token(app.clone(), "admin@sdkwork.local", "ChangeMe123!").await;
    let admin_token = admin_login["token"].as_str().unwrap();
    let admin_user = admin_login["user"].clone();

    seed_provider_and_model(app.clone(), admin_token, "provider-secret-audit", "gpt-4.1").await;

    let create_credential = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"tenant_id":"tenant-1","provider_id":"provider-secret-audit","key_reference":"cred-secret-audit","secret_value":"sk-secret-audit"}"#,
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
                .uri("/admin/credentials/tenant-1/providers/provider-secret-audit/keys/cred-secret-audit")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_credential.status(), StatusCode::NO_CONTENT);

    let audit_events = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/audit/events")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(audit_events.status(), StatusCode::OK);
    let audit_json = read_json(audit_events).await;
    let events = audit_json.as_array().unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0]["action"], "credential.delete");
    assert_eq!(events[0]["resource_type"], "credential");
    assert_eq!(
        events[0]["resource_id"],
        "tenant-1:provider-secret-audit:cred-secret-audit"
    );
    assert_eq!(events[0]["approval_scope"], "secret_control");
    assert_eq!(events[0]["actor_user_id"], admin_user["id"]);
    assert_eq!(events[0]["actor_role"], "super_admin");
    assert_eq!(events[1]["action"], "credential.create");
    assert_eq!(events[1]["resource_type"], "credential");
}

#[tokio::test]
async fn super_admin_operator_user_mutations_are_audited() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let admin_login = login_token(app.clone(), "admin@sdkwork.local", "ChangeMe123!").await;
    let admin_token = admin_login["token"].as_str().unwrap();
    let admin_user = admin_login["user"].clone();

    let created_user = create_operator(
        app.clone(),
        admin_token,
        "audited-operator@example.com",
        "Audited Operator",
        "AuditedPass123!",
        "platform_operator",
    )
    .await;
    let user_id = created_user["id"].as_str().unwrap().to_owned();

    let deactivate = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/admin/users/operators/{user_id}/status"))
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"active":false}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(deactivate.status(), StatusCode::OK);

    let reset_password = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/admin/users/operators/{user_id}/password"))
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"new_password":"AuditedPass456!"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(reset_password.status(), StatusCode::OK);

    let delete_user = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/admin/users/operators/{user_id}"))
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_user.status(), StatusCode::NO_CONTENT);

    let audit_events = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/audit/events")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(audit_events.status(), StatusCode::OK);
    let audit_json = read_json(audit_events).await;
    let events = audit_json.as_array().unwrap();
    assert_eq!(events.len(), 4);

    let actions = events
        .iter()
        .map(|event| event["action"].as_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    assert!(actions.contains(&"admin_user.create".to_owned()));
    assert!(actions.contains(&"admin_user.status.update".to_owned()));
    assert!(actions.contains(&"admin_user.password.reset".to_owned()));
    assert!(actions.contains(&"admin_user.delete".to_owned()));

    for event in events {
        assert_eq!(event["resource_type"], "admin_user");
        assert_eq!(event["resource_id"], user_id);
        assert_eq!(event["approval_scope"], "identity_control");
        assert_eq!(event["actor_user_id"], admin_user["id"]);
        assert_eq!(event["actor_role"], "super_admin");
    }
}

#[tokio::test]
async fn super_admin_api_key_group_and_key_mutations_are_audited() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let admin_login = login_token(app.clone(), "admin@sdkwork.local", "ChangeMe123!").await;
    let admin_token = admin_login["token"].as_str().unwrap();
    let admin_user = admin_login["user"].clone();

    seed_workspace(app.clone(), admin_token, "tenant-audit", "project-audit").await;

    let create_group = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/api-key-groups")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r##"{"tenant_id":"tenant-audit","project_id":"project-audit","environment":"production","name":"Audit Group","description":"Audit group","color":"#2563eb","default_capability_scope":"chat"}"##,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_group.status(), StatusCode::CREATED);
    let create_group_json = read_json(create_group).await;
    let group_id = create_group_json["group_id"].as_str().unwrap().to_owned();

    let update_group = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/admin/api-key-groups/{group_id}"))
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r##"{"tenant_id":"tenant-audit","project_id":"project-audit","environment":"production","name":"Audit Group Prime","slug":"audit-group-prime","description":"Updated audit group","color":"#0f766e","default_capability_scope":"responses"}"##,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(update_group.status(), StatusCode::OK);

    let disable_group = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/admin/api-key-groups/{group_id}/status"))
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"active":false}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(disable_group.status(), StatusCode::OK);

    let enable_group = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/admin/api-key-groups/{group_id}/status"))
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"active":true}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(enable_group.status(), StatusCode::OK);

    let create_key = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/api-keys")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"tenant_id":"tenant-audit","project_id":"project-audit","environment":"production","label":"Audit Key","notes":"Initial notes","api_key_group_id":"{group_id}"}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_key.status(), StatusCode::CREATED);
    let create_key_json = read_json(create_key).await;
    let hashed_key = create_key_json["hashed"].as_str().unwrap().to_owned();

    let update_key = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/admin/api-keys/{hashed_key}"))
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"tenant_id":"tenant-audit","project_id":"project-audit","environment":"production","label":"Audit Key Updated","notes":"Updated notes","api_key_group_id":"{group_id}"}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(update_key.status(), StatusCode::OK);

    let revoke_key = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/admin/api-keys/{hashed_key}/status"))
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"active":false}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(revoke_key.status(), StatusCode::OK);

    let delete_key = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/admin/api-keys/{hashed_key}"))
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_key.status(), StatusCode::NO_CONTENT);

    let delete_group = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/admin/api-key-groups/{group_id}"))
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_group.status(), StatusCode::NO_CONTENT);

    let audit_events = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/audit/events")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(audit_events.status(), StatusCode::OK);
    let audit_json = read_json(audit_events).await;
    let events = audit_json.as_array().unwrap();
    assert_eq!(events.len(), 9);

    let actions = events
        .iter()
        .map(|event| event["action"].as_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    assert!(actions.contains(&"api_key_group.create".to_owned()));
    assert!(actions.contains(&"api_key_group.update".to_owned()));
    assert!(actions.contains(&"api_key_group.status.update".to_owned()));
    assert!(actions.contains(&"api_key_group.delete".to_owned()));
    assert!(actions.contains(&"gateway_api_key.create".to_owned()));
    assert!(actions.contains(&"gateway_api_key.update".to_owned()));
    assert!(actions.contains(&"gateway_api_key.status.update".to_owned()));
    assert!(actions.contains(&"gateway_api_key.delete".to_owned()));

    let group_events = events
        .iter()
        .filter(|event| event["resource_type"] == "api_key_group")
        .collect::<Vec<_>>();
    assert_eq!(group_events.len(), 5);
    for event in &group_events {
        assert_eq!(event["resource_id"], group_id);
        assert_eq!(event["approval_scope"], "identity_control");
        assert_eq!(event["actor_user_id"], admin_user["id"]);
    }

    let key_events = events
        .iter()
        .filter(|event| event["resource_type"] == "gateway_api_key")
        .collect::<Vec<_>>();
    assert_eq!(key_events.len(), 4);
    for event in &key_events {
        assert_eq!(event["resource_id"], hashed_key);
        assert_eq!(event["approval_scope"], "secret_control");
        assert_eq!(event["actor_user_id"], admin_user["id"]);
    }
}
