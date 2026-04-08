use super::support::*;

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
                    "{\"id\":\"provider-openai-official\",\"channel_id\":\"openai\",\"extension_id\":\"sdkwork.provider.openai.official\",\"channel_bindings\":[{\"channel_id\":\"openai\",\"is_primary\":true},{\"channel_id\":\"responses-compatible\",\"is_primary\":false}],\"adapter_kind\":\"openai\",\"base_url\":\"https://api.openai.com\",\"display_name\":\"OpenAI Official\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let provider_status = provider.status();
    let provider_body = String::from_utf8(
        to_bytes(provider.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert_eq!(
        provider_status,
        StatusCode::CREATED,
        "body={provider_body:?}"
    );

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
    assert_eq!(
        providers_json[0]["extension_id"],
        "sdkwork.provider.openai.official"
    );
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
async fn delete_credentials_through_admin_api() {
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

    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"id\":\"provider-openai-official\",\"channel_id\":\"openai\",\"extension_id\":\"sdkwork.provider.openai.official\",\"channel_bindings\":[{\"channel_id\":\"openai\",\"is_primary\":true}],\"adapter_kind\":\"openai\",\"base_url\":\"https://api.openai.com\",\"display_name\":\"OpenAI Official\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let create_credential = app
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

    assert_eq!(create_credential.status(), StatusCode::CREATED);

    let delete_credential = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/admin/credentials/tenant-1/providers/provider-openai-official/keys/cred-openai")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(delete_credential.status(), StatusCode::NO_CONTENT);

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
    assert!(read_json(credentials).await.as_array().unwrap().is_empty());

    let secret = sdkwork_api_app_credential::resolve_credential_secret(
        &store,
        "local-dev-master-key",
        "tenant-1",
        "provider-openai-official",
        "cred-openai",
    )
    .await;
    assert!(secret.is_err());
    assert_eq!(
        secret.unwrap_err().to_string(),
        "credential secret not found"
    );
}
#[tokio::test]
async fn create_and_list_models() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    create_provider_fixture(
        app.clone(),
        &token,
        r#"{"id":"provider-openai-official","channel_id":"openai","adapter_kind":"openai","base_url":"https://api.openai.com","display_name":"OpenAI Official","channel_bindings":[{"channel_id":"openai","is_primary":true}]}"#,
    )
    .await;

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

#[serial(extension_env)]
#[tokio::test]
async fn create_update_list_and_delete_coupons() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let initial = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/coupons")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(initial.status(), StatusCode::OK);
    assert_eq!(read_json(initial).await.as_array().unwrap().len(), 0);

    let created = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/coupons")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"id\":\"coupon_spring_launch\",\"code\":\"SPRING20\",\"discount_label\":\"20% launch discount\",\"audience\":\"new_signup\",\"remaining\":120,\"active\":true,\"note\":\"Spring launch campaign\",\"expires_on\":\"2026-05-31\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(created.status(), StatusCode::CREATED);
    let created_json = read_json(created).await;
    assert_eq!(created_json["id"], "coupon_spring_launch");
    assert_eq!(created_json["code"], "SPRING20");
    assert_eq!(created_json["remaining"], 120);

    let updated = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/coupons")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"id\":\"coupon_spring_launch\",\"code\":\"SPRING20\",\"discount_label\":\"20% launch discount\",\"audience\":\"enterprise_trial\",\"remaining\":64,\"active\":false,\"note\":\"Reserved for enterprise conversions\",\"expires_on\":\"2026-06-30\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(updated.status(), StatusCode::CREATED);
    let updated_json = read_json(updated).await;
    assert_eq!(updated_json["audience"], "enterprise_trial");
    assert_eq!(updated_json["remaining"], 64);
    assert_eq!(updated_json["active"], false);

    let listed = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/coupons")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(listed.status(), StatusCode::OK);
    let listed_json = read_json(listed).await;
    assert_eq!(listed_json.as_array().unwrap().len(), 1);
    assert_eq!(listed_json[0]["expires_on"], "2026-06-30");

    let deleted = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/admin/coupons/coupon_spring_launch")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(deleted.status(), StatusCode::NO_CONTENT);

    let after_delete = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/coupons")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(after_delete.status(), StatusCode::OK);
    assert_eq!(read_json(after_delete).await.as_array().unwrap().len(), 0);
}

#[serial(extension_env)]
#[tokio::test]
async fn delete_model_keeps_same_external_name_on_other_provider() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    for channel in [
        r#"{"id":"openai","name":"OpenAI"}"#,
        r#"{"id":"openrouter","name":"OpenRouter"}"#,
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/admin/channels")
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(channel))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    for provider in [
        r#"{"id":"provider-openai-official","channel_id":"openai","display_name":"OpenAI Official","adapter_kind":"openai","base_url":"https://api.openai.com","channel_bindings":[{"channel_id":"openai","is_primary":true}]}"#,
        r#"{"id":"provider-openrouter","channel_id":"openrouter","display_name":"OpenRouter","adapter_kind":"openai","base_url":"https://openrouter.ai/api/v1","channel_bindings":[{"channel_id":"openrouter","is_primary":true}]}"#,
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/admin/providers")
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(provider))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    for model in [
        r#"{"external_name":"gpt-4.1","provider_id":"provider-openai-official","capabilities":["responses"],"streaming":true,"context_window":128000}"#,
        r#"{"external_name":"gpt-4.1","provider_id":"provider-openrouter","capabilities":["responses"],"streaming":true,"context_window":128000}"#,
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/admin/models")
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(model))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    let deleted = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/admin/models/gpt-4.1/providers/provider-openai-official")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(deleted.status(), StatusCode::NO_CONTENT);

    let listed = app
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

    assert_eq!(listed.status(), StatusCode::OK);
    let models_json = read_json(listed).await;
    assert_eq!(models_json.as_array().unwrap().len(), 1);
    assert_eq!(models_json[0]["external_name"], "gpt-4.1");
    assert_eq!(models_json[0]["provider_id"], "provider-openrouter");
}
