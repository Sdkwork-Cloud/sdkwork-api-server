use super::support::*;

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

    let create_provider_model = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/provider-models")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"proxy_provider_id":"provider-openai-official","channel_id":"openai","model_id":"gpt-4.1","provider_model_id":"gpt-4.1","capabilities":["responses","chat_completions"],"streaming":true,"context_window":128000,"max_output_tokens":32768,"supports_prompt_caching":true,"supports_reasoning_usage":false,"supports_tool_usage_metrics":true,"is_default_route":true,"is_active":true}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_provider_model.status(), StatusCode::CREATED);

    let create_price = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/model-prices")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"channel_id":"openai","model_id":"gpt-4.1","proxy_provider_id":"provider-openai-official","currency_code":"USD","price_unit":"per_1m_tokens","input_price":2.5,"output_price":10.0,"cache_read_price":0.3,"cache_write_price":1.0,"request_price":0.0,"price_source_kind":"official","billing_notes":"Published from official OpenAI pricing.","pricing_tiers":[{"tier_id":"default","display_name":"Default","condition_kind":"default","currency_code":"USD","price_unit":"per_1m_tokens","input_price":2.5,"output_price":10.0,"cache_read_price":0.3,"cache_write_price":1.0,"request_price":0.0}],"is_active":true}"#,
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
            && item["price_source_kind"] == "official"
            && item["billing_notes"] == "Published from official OpenAI pricing."
            && item["pricing_tiers"].as_array().is_some_and(|tiers| tiers.len() == 1)
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
