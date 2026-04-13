use super::support::*;

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
#[tokio::test]
async fn gateway_api_keys_do_not_persist_raw_key_in_canonical_ai_app_api_keys_table() {
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
    assert!(listed_json[0].get("raw_key").is_none());
    assert_eq!(listed_json[0]["label"], "Production App Key");
    assert_eq!(listed_json[0]["notes"], "retained for admin inventory");
    assert_eq!(listed_json[0]["expires_at_ms"], 4102444800000_u64);

    let stored_row: (Option<String>, Option<String>, Option<i64>) = sqlx::query_as(
        "SELECT raw_key, notes, expires_at_ms FROM ai_app_api_keys WHERE hashed_key = ?",
    )
    .bind(&hashed_key)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(stored_row.0, None);
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
    assert!(updated_json.get("raw_key").is_none());
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
async fn manage_api_key_groups_through_admin_api() {
    let pool = memory_pool().await;
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
                .uri("/admin/api-key-groups")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r##"{"tenant_id":"tenant-acme","project_id":"project-acme","environment":"production","name":"Production Keys","description":"Primary production pool","color":"#2563eb","default_capability_scope":"chat,responses"}"##,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(created.status(), StatusCode::CREATED);
    let created_json = read_json(created).await;
    let group_id = created_json["group_id"].as_str().unwrap().to_owned();
    assert_eq!(created_json["slug"], "production-keys");
    assert_eq!(created_json["active"], true);

    let listed = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/api-key-groups")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(listed.status(), StatusCode::OK);
    let listed_json = read_json(listed).await;
    assert_eq!(listed_json[0]["group_id"], group_id);
    assert_eq!(listed_json[0]["description"], "Primary production pool");

    let updated = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/admin/api-key-groups/{group_id}"))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r##"{"tenant_id":"tenant-acme","project_id":"project-acme","environment":"production","name":"Enterprise Keys","slug":"enterprise-keys","description":"Premium production pool","color":"#0f766e","default_capability_scope":"responses"}"##,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(updated.status(), StatusCode::OK);
    let updated_json = read_json(updated).await;
    assert_eq!(updated_json["name"], "Enterprise Keys");
    assert_eq!(updated_json["slug"], "enterprise-keys");
    assert_eq!(updated_json["color"], "#0f766e");

    let disabled = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/admin/api-key-groups/{group_id}/status"))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"active":false}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(disabled.status(), StatusCode::OK);
    assert_eq!(read_json(disabled).await["active"], false);
}

#[serial(extension_env)]
#[tokio::test]
async fn create_gateway_api_keys_through_admin_api_validates_group_assignment() {
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

    let live_group = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/api-key-groups")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"tenant_id":"tenant-acme","project_id":"project-acme","environment":"production","name":"Production Keys"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(live_group.status(), StatusCode::CREATED);
    let live_group_id = read_json(live_group).await["group_id"]
        .as_str()
        .unwrap()
        .to_owned();

    let staging_group = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/api-key-groups")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"tenant_id":"tenant-acme","project_id":"project-acme","environment":"staging","name":"Staging Keys"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(staging_group.status(), StatusCode::CREATED);
    let staging_group_id = read_json(staging_group).await["group_id"]
        .as_str()
        .unwrap()
        .to_owned();

    let mismatched = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/api-keys")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"tenant_id":"tenant-acme","project_id":"project-acme","environment":"production","label":"Invalid key","api_key_group_id":"{staging_group_id}"}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(mismatched.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        read_json(mismatched).await["error"]["message"],
        "api key group environment does not match"
    );

    let created = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/api-keys")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"tenant_id":"tenant-acme","project_id":"project-acme","environment":"production","label":"Production App Key","api_key_group_id":"{live_group_id}"}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(created.status(), StatusCode::CREATED);
    let created_json = read_json(created).await;
    let hashed_key = created_json["hashed"].as_str().unwrap().to_owned();
    assert_eq!(created_json["api_key_group_id"], live_group_id);

    let stored_row: Option<String> =
        sqlx::query_scalar("SELECT api_key_group_id FROM ai_app_api_keys WHERE hashed_key = ?")
            .bind(&hashed_key)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(stored_row.as_deref(), Some(live_group_id.as_str()));
}
