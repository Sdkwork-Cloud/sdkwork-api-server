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
    assert_eq!(providers_json[0]["protocol_kind"], "openai");
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
async fn create_and_list_official_provider_configs() {
    let pool = memory_pool().await;
    let store = sdkwork_api_storage_sqlite::SqliteAdminStore::new(pool.clone());
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    create_provider_fixture(
        app.clone(),
        &token,
        r#"{"id":"provider-openai-official","channel_id":"openai","adapter_kind":"openai","base_url":"https://api.openai.com/v1","display_name":"OpenAI Official","channel_bindings":[{"channel_id":"openai","is_primary":true}]}"#,
    )
    .await;

    let created = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers/official-configs")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"provider_id":"provider-openai-official","base_url":"https://api.openai.com/v1","enabled":true,"api_key":"sk-platform-openai"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(created.status(), StatusCode::CREATED);
    let created_json = read_json(created).await;
    assert_eq!(created_json["provider_id"], "provider-openai-official");
    assert_eq!(created_json["base_url"], "https://api.openai.com/v1");
    assert_eq!(created_json["enabled"], true);
    assert_eq!(created_json["secret_configured"], true);

    let listed = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/providers/official-configs")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(listed.status(), StatusCode::OK);
    let listed_json = read_json(listed).await;
    assert_eq!(listed_json.as_array().unwrap().len(), 1);
    assert_eq!(listed_json[0]["provider_id"], "provider-openai-official");
    assert_eq!(listed_json[0]["enabled"], true);
    assert_eq!(listed_json[0]["secret_configured"], true);

    let secret = sdkwork_api_app_credential::resolve_official_provider_secret_with_manager(
        &store,
        &sdkwork_api_app_credential::CredentialSecretManager::database_encrypted(
            "local-dev-master-key",
        ),
        "provider-openai-official",
    )
    .await
    .unwrap();
    assert_eq!(secret.as_deref(), Some("sk-platform-openai"));
}

#[tokio::test]
async fn create_provider_accepts_explicit_protocol_kind() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let create_channel = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/channels")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"id\":\"claude\",\"name\":\"Claude\"}"))
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
                .body(Body::from(
                    "{\"id\":\"provider-claude-relay\",\"channel_id\":\"claude\",\"extension_id\":\"sdkwork.provider.claude.relay\",\"adapter_kind\":\"native-dynamic\",\"protocol_kind\":\"anthropic\",\"base_url\":\"https://relay.example.com\",\"display_name\":\"Claude Relay\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_provider.status(), StatusCode::CREATED);
    let created_json = read_json(create_provider).await;
    assert_eq!(created_json["adapter_kind"], "native-dynamic");
    assert_eq!(created_json["protocol_kind"], "anthropic");
    assert_eq!(
        created_json["extension_id"],
        "sdkwork.provider.claude.relay"
    );
    assert_eq!(created_json["integration"]["mode"], "custom_plugin");
    assert!(created_json["integration"]["default_plugin_family"].is_null());

    let list_providers = app
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

    assert_eq!(list_providers.status(), StatusCode::OK);
    let providers_json = read_json(list_providers).await;
    assert_eq!(providers_json[0]["id"], "provider-claude-relay");
    assert_eq!(providers_json[0]["adapter_kind"], "native-dynamic");
    assert_eq!(providers_json[0]["protocol_kind"], "anthropic");
}

#[tokio::test]
async fn create_provider_derives_protocol_kind_for_standard_passthrough_providers() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    for (channel_id, display_name) in [("anthropic", "Anthropic"), ("gemini", "Gemini")] {
        let create_channel = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/admin/channels")
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(format!(
                        "{{\"id\":\"{channel_id}\",\"name\":\"{display_name}\"}}"
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(create_channel.status(), StatusCode::CREATED);
    }

    let create_anthropic = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"id\":\"provider-anthropic-official\",\"channel_id\":\"anthropic\",\"adapter_kind\":\"anthropic\",\"base_url\":\"https://api.anthropic.com\",\"display_name\":\"Anthropic Official\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_anthropic.status(), StatusCode::CREATED);
    let anthropic_json = read_json(create_anthropic).await;
    assert_eq!(anthropic_json["protocol_kind"], "anthropic");
    assert_eq!(anthropic_json["extension_id"], "sdkwork.provider.anthropic");
    assert_eq!(
        anthropic_json["integration"]["mode"],
        "standard_passthrough"
    );
    assert!(anthropic_json["integration"]["default_plugin_family"].is_null());

    let create_gemini = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"id\":\"provider-gemini-official\",\"channel_id\":\"gemini\",\"adapter_kind\":\"gemini\",\"base_url\":\"https://generativelanguage.googleapis.com\",\"display_name\":\"Gemini Official\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_gemini.status(), StatusCode::CREATED);
    let gemini_json = read_json(create_gemini).await;
    assert_eq!(gemini_json["protocol_kind"], "gemini");
    assert_eq!(gemini_json["extension_id"], "sdkwork.provider.gemini");
    assert_eq!(gemini_json["integration"]["mode"], "standard_passthrough");
    assert!(gemini_json["integration"]["default_plugin_family"].is_null());
}

#[tokio::test]
async fn create_provider_accepts_default_plugin_family_for_openrouter() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let create_channel = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/channels")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"id\":\"openrouter\",\"name\":\"OpenRouter\"}"))
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
                .body(Body::from(
                    "{\"id\":\"provider-openrouter-main\",\"channel_id\":\"openrouter\",\"default_plugin_family\":\"openrouter\",\"base_url\":\"https://openrouter.ai/api/v1\",\"display_name\":\"OpenRouter Main\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_provider.status(), StatusCode::CREATED);
    let created_json = read_json(create_provider).await;
    assert_eq!(created_json["adapter_kind"], "openrouter");
    assert_eq!(created_json["protocol_kind"], "openai");
    assert_eq!(created_json["extension_id"], "sdkwork.provider.openrouter");
    assert_eq!(created_json["integration"]["mode"], "default_plugin");
    assert_eq!(
        created_json["integration"]["default_plugin_family"],
        "openrouter"
    );

    let list_providers = app
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

    assert_eq!(list_providers.status(), StatusCode::OK);
    let providers_json = read_json(list_providers).await;
    let provider = provider_json_by_id(&providers_json, "provider-openrouter-main");
    assert_eq!(provider["integration"]["mode"], "default_plugin");
    assert_eq!(
        provider["integration"]["default_plugin_family"],
        "openrouter"
    );
}

#[tokio::test]
async fn create_provider_accepts_default_plugin_family_for_ollama() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let create_channel = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/channels")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"id\":\"ollama\",\"name\":\"Ollama\"}"))
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
                .body(Body::from(
                    "{\"id\":\"provider-ollama-local\",\"channel_id\":\"ollama\",\"default_plugin_family\":\"ollama\",\"base_url\":\"http://localhost:11434/v1\",\"display_name\":\"Ollama Local\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_provider.status(), StatusCode::CREATED);
    let created_json = read_json(create_provider).await;
    assert_eq!(created_json["adapter_kind"], "ollama");
    assert_eq!(created_json["protocol_kind"], "custom");
    assert_eq!(created_json["extension_id"], "sdkwork.provider.ollama");
    assert_eq!(created_json["integration"]["mode"], "default_plugin");
    assert_eq!(created_json["integration"]["default_plugin_family"], "ollama");

    let list_providers = app
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

    assert_eq!(list_providers.status(), StatusCode::OK);
    let providers_json = read_json(list_providers).await;
    let provider = provider_json_by_id(&providers_json, "provider-ollama-local");
    assert_eq!(provider["integration"]["mode"], "default_plugin");
    assert_eq!(provider["integration"]["default_plugin_family"], "ollama");
}

#[tokio::test]
async fn create_provider_rejects_conflicting_default_plugin_family_and_adapter_kind() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let create_channel = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/channels")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"id\":\"openrouter\",\"name\":\"OpenRouter\"}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_channel.status(), StatusCode::CREATED);

    let create_provider = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"id\":\"provider-invalid-openrouter\",\"channel_id\":\"openrouter\",\"default_plugin_family\":\"openrouter\",\"adapter_kind\":\"ollama\",\"base_url\":\"https://openrouter.ai/api/v1\",\"display_name\":\"Invalid OpenRouter\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_provider.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn list_providers_exposes_implicit_standard_passthrough_execution_view() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    create_provider_fixture(
        app.clone(),
        &token,
        r#"{"id":"provider-openai-official","channel_id":"openai","adapter_kind":"openai","base_url":"https://api.openai.com","display_name":"OpenAI Official","channel_bindings":[{"channel_id":"openai","is_primary":true}]}"#,
    )
    .await;

    let list_providers = app
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

    assert_eq!(list_providers.status(), StatusCode::OK);
    let providers_json = read_json(list_providers).await;
    let provider = provider_json_by_id(&providers_json, "provider-openai-official");
    assert_eq!(provider["integration"]["mode"], "standard_passthrough");
    assert!(provider["integration"]["default_plugin_family"].is_null());
    assert_eq!(provider["execution"]["binding_kind"], "implicit_default");
    assert_eq!(provider["execution"]["runtime"], "builtin");
    assert_eq!(
        provider["execution"]["runtime_key"],
        "sdkwork.provider.openai.official"
    );
    assert_eq!(provider["execution"]["passthrough_protocol"], "openai");
    assert_eq!(provider["execution"]["supports_provider_adapter"], true);
    assert_eq!(provider["execution"]["supports_raw_plugin"], false);
    assert_eq!(provider["execution"]["fail_closed"], false);
    assert_eq!(provider["execution"]["route_readiness"]["openai"]["ready"], true);
    assert_eq!(
        provider["execution"]["route_readiness"]["openai"]["mode"],
        "provider_adapter"
    );
    assert_eq!(
        provider["execution"]["route_readiness"]["anthropic"]["ready"],
        true
    );
    assert_eq!(
        provider["execution"]["route_readiness"]["anthropic"]["mode"],
        "provider_adapter"
    );
    assert_eq!(provider["execution"]["route_readiness"]["gemini"]["ready"], true);
    assert_eq!(
        provider["execution"]["route_readiness"]["gemini"]["mode"],
        "provider_adapter"
    );
    assert!(provider["credential_readiness"].is_null());
}

#[tokio::test]
async fn list_providers_exposes_tenant_scoped_credential_readiness_only_when_requested() {
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

    create_provider_fixture(
        app.clone(),
        &token,
        r#"{"id":"provider-openai-official","channel_id":"openai","adapter_kind":"openai","base_url":"https://api.openai.com","display_name":"OpenAI Official","channel_bindings":[{"channel_id":"openai","is_primary":true}]}"#,
    )
    .await;
    create_provider_fixture(
        app.clone(),
        &token,
        r#"{"id":"provider-openrouter-main","channel_id":"openrouter","default_plugin_family":"openrouter","base_url":"https://openrouter.ai/api/v1","display_name":"OpenRouter Main","channel_bindings":[{"channel_id":"openrouter","is_primary":true}]}"#,
    )
    .await;

    for credential in [
        r#"{"tenant_id":"tenant-1","provider_id":"provider-openrouter-main","key_reference":"cred-openrouter","secret_value":"sk-openrouter"}"#,
        r#"{"tenant_id":"tenant-other","provider_id":"provider-openai-official","key_reference":"cred-openai-other","secret_value":"sk-openai-other"}"#,
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/admin/credentials")
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(credential))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    let unscoped_list = app
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

    assert_eq!(unscoped_list.status(), StatusCode::OK);
    let unscoped_json = read_json(unscoped_list).await;
    assert!(provider_json_by_id(&unscoped_json, "provider-openai-official")["credential_readiness"].is_null());
    assert!(provider_json_by_id(&unscoped_json, "provider-openrouter-main")["credential_readiness"].is_null());

    let scoped_list = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/providers?tenant_id=tenant-1")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(scoped_list.status(), StatusCode::OK);
    let scoped_json = read_json(scoped_list).await;
    let openai_provider = provider_json_by_id(&scoped_json, "provider-openai-official");
    assert_eq!(openai_provider["credential_readiness"]["ready"], false);
    assert_eq!(openai_provider["credential_readiness"]["state"], "missing");
    let openrouter_provider = provider_json_by_id(&scoped_json, "provider-openrouter-main");
    assert_eq!(openrouter_provider["credential_readiness"]["ready"], true);
    assert_eq!(
        openrouter_provider["credential_readiness"]["state"],
        "configured"
    );
}

#[tokio::test]
async fn list_tenant_provider_readiness_exposes_focused_tenant_overlay_inventory() {
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
        r#"{"id":"provider-openai-official","channel_id":"openai","adapter_kind":"openai","base_url":"https://api.openai.com","display_name":"OpenAI Official","channel_bindings":[{"channel_id":"openai","is_primary":true}]}"#,
        r#"{"id":"provider-openrouter-main","channel_id":"openrouter","default_plugin_family":"openrouter","base_url":"https://openrouter.ai/api/v1","display_name":"OpenRouter Main","channel_bindings":[{"channel_id":"openrouter","is_primary":true}]}"#,
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

    let create_credential = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"tenant_id":"tenant-1","provider_id":"provider-openrouter-main","key_reference":"cred-openrouter","secret_value":"sk-openrouter"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_credential.status(), StatusCode::CREATED);

    let readiness_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/tenants/tenant-1/providers/readiness")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(readiness_response.status(), StatusCode::OK);
    let readiness_json = read_json(readiness_response).await;
    let openai_provider = provider_json_by_id(&readiness_json, "provider-openai-official");
    assert_eq!(openai_provider["display_name"], "OpenAI Official");
    assert_eq!(openai_provider["protocol_kind"], "openai");
    assert_eq!(openai_provider["integration"]["mode"], "standard_passthrough");
    assert_eq!(openai_provider["credential_readiness"]["ready"], false);
    assert_eq!(openai_provider["credential_readiness"]["state"], "missing");
    assert!(openai_provider["execution"].is_null());

    let openrouter_provider = provider_json_by_id(&readiness_json, "provider-openrouter-main");
    assert_eq!(openrouter_provider["display_name"], "OpenRouter Main");
    assert_eq!(openrouter_provider["protocol_kind"], "openai");
    assert_eq!(openrouter_provider["integration"]["mode"], "default_plugin");
    assert_eq!(
        openrouter_provider["integration"]["default_plugin_family"],
        "openrouter"
    );
    assert_eq!(openrouter_provider["credential_readiness"]["ready"], true);
    assert_eq!(
        openrouter_provider["credential_readiness"]["state"],
        "configured"
    );
    assert!(openrouter_provider["execution"].is_null());
}

#[serial(extension_env)]
#[tokio::test]
async fn list_providers_exposes_native_dynamic_raw_plugin_execution_view() {
    shutdown_all_native_dynamic_runtimes().unwrap();

    let extension_root = temp_extension_root("admin-provider-native-dynamic-view");
    let package_dir = extension_root.join("sdkwork-provider-native-mock");
    fs::create_dir_all(&package_dir).unwrap();
    let library_path = native_dynamic_fixture_library_path();
    fs::write(
        package_dir.join("sdkwork-extension.toml"),
        native_dynamic_manifest(&library_path),
    )
    .unwrap();
    let _guard = native_dynamic_extension_env_guard(&extension_root);

    let pool = memory_pool().await;
    let store = sdkwork_api_storage_sqlite::SqliteAdminStore::new(pool.clone());
    store
        .insert_extension_installation(
            &ExtensionInstallation::new(
                "native-mock-installation",
                FIXTURE_EXTENSION_ID,
                ExtensionRuntime::NativeDynamic,
            )
            .with_enabled(true),
        )
        .await
        .unwrap();
    store
        .insert_extension_instance(
            &ExtensionInstance::new(
                "provider-claude-plugin",
                "native-mock-installation",
                FIXTURE_EXTENSION_ID,
            )
            .with_enabled(true)
            .with_base_url("https://relay.example.com"),
        )
        .await
        .unwrap();

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    create_provider_fixture(
        app.clone(),
        &token,
        &format!(
            r#"{{"id":"provider-claude-plugin","channel_id":"anthropic","extension_id":"{extension_id}","adapter_kind":"native-dynamic","protocol_kind":"anthropic","base_url":"https://relay.example.com","display_name":"Claude Native Plugin","channel_bindings":[{{"channel_id":"anthropic","is_primary":true}}]}}"#,
            extension_id = FIXTURE_EXTENSION_ID,
        ),
    )
    .await;

    let list_providers = app
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

    assert_eq!(list_providers.status(), StatusCode::OK);
    let providers_json = read_json(list_providers).await;
    let provider = provider_json_by_id(&providers_json, "provider-claude-plugin");
    assert_eq!(provider["integration"]["mode"], "custom_plugin");
    assert!(provider["integration"]["default_plugin_family"].is_null());
    assert_eq!(provider["execution"]["binding_kind"], "explicit_instance");
    assert_eq!(provider["execution"]["runtime"], "native_dynamic");
    assert_eq!(provider["execution"]["runtime_key"], FIXTURE_EXTENSION_ID);
    assert_eq!(provider["execution"]["passthrough_protocol"], "anthropic");
    assert_eq!(provider["execution"]["supports_provider_adapter"], true);
    assert_eq!(provider["execution"]["supports_raw_plugin"], true);
    assert_eq!(provider["execution"]["fail_closed"], false);
    assert_eq!(provider["execution"]["route_readiness"]["openai"]["ready"], true);
    assert_eq!(
        provider["execution"]["route_readiness"]["openai"]["mode"],
        "provider_adapter"
    );
    assert_eq!(
        provider["execution"]["route_readiness"]["anthropic"]["ready"],
        true
    );
    assert_eq!(
        provider["execution"]["route_readiness"]["anthropic"]["mode"],
        "standard_passthrough"
    );
    assert_eq!(provider["execution"]["route_readiness"]["gemini"]["ready"], true);
    assert_eq!(
        provider["execution"]["route_readiness"]["gemini"]["mode"],
        "raw_plugin"
    );

    shutdown_all_native_dynamic_runtimes().unwrap();
    cleanup_dir(&extension_root);
}

#[tokio::test]
async fn list_providers_exposes_fail_closed_execution_view_for_broken_explicit_binding() {
    let pool = memory_pool().await;
    let store = sdkwork_api_storage_sqlite::SqliteAdminStore::new(pool.clone());
    store
        .insert_extension_installation(
            &ExtensionInstallation::new(
                "broken-native-installation",
                "sdkwork.provider.claude.broken",
                ExtensionRuntime::NativeDynamic,
            )
            .with_enabled(true),
        )
        .await
        .unwrap();
    store
        .insert_extension_instance(
            &ExtensionInstance::new(
                "provider-claude-broken",
                "broken-native-installation",
                "sdkwork.provider.claude.broken",
            )
            .with_enabled(true)
            .with_base_url("https://broken.example.com"),
        )
        .await
        .unwrap();

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    create_provider_fixture(
        app.clone(),
        &token,
        r#"{"id":"provider-claude-broken","channel_id":"anthropic","extension_id":"sdkwork.provider.claude.broken","adapter_kind":"native-dynamic","protocol_kind":"anthropic","base_url":"https://broken.example.com","display_name":"Claude Broken Plugin","channel_bindings":[{"channel_id":"anthropic","is_primary":true}]}"#,
    )
    .await;

    let list_providers = app
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

    assert_eq!(list_providers.status(), StatusCode::OK);
    let providers_json = read_json(list_providers).await;
    let provider = provider_json_by_id(&providers_json, "provider-claude-broken");
    assert_eq!(provider["execution"]["binding_kind"], "explicit_instance");
    assert_eq!(provider["execution"]["runtime"], "native_dynamic");
    assert_eq!(provider["execution"]["passthrough_protocol"], "anthropic");
    assert_eq!(provider["execution"]["supports_provider_adapter"], false);
    assert_eq!(provider["execution"]["supports_raw_plugin"], false);
    assert_eq!(provider["execution"]["fail_closed"], true);
    assert_eq!(
        provider["execution"]["reason"],
        "explicit runtime binding is not currently loadable, so gateway execution would fail closed instead of silently downgrading"
    );
    assert_eq!(provider["execution"]["route_readiness"]["openai"]["ready"], false);
    assert_eq!(
        provider["execution"]["route_readiness"]["openai"]["mode"],
        "fail_closed"
    );
    assert_eq!(
        provider["execution"]["route_readiness"]["anthropic"]["ready"],
        true
    );
    assert_eq!(
        provider["execution"]["route_readiness"]["anthropic"]["mode"],
        "standard_passthrough"
    );
    assert_eq!(provider["execution"]["route_readiness"]["gemini"]["ready"], false);
    assert_eq!(
        provider["execution"]["route_readiness"]["gemini"]["mode"],
        "fail_closed"
    );
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

#[tokio::test]
async fn create_model_price_requires_provider_model_support() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

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

    create_provider_fixture(
        app.clone(),
        &token,
        r#"{"id":"provider-openai-official","channel_id":"openai","adapter_kind":"openai","base_url":"https://api.openai.com/v1","display_name":"OpenAI Official","channel_bindings":[{"channel_id":"openai","is_primary":true}]}"#,
    )
    .await;

    let create_channel_model = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/channel-models")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"channel_id":"openai","model_id":"gpt-4.1","model_display_name":"GPT-4.1","capabilities":["responses"],"streaming":true,"context_window":128000}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_channel_model.status(), StatusCode::CREATED);

    let create_price_without_provider_model = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/model-prices")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"channel_id":"openai","model_id":"gpt-4.1","proxy_provider_id":"provider-openai-official","currency_code":"USD","price_unit":"per_1m_tokens","input_price":2,"output_price":8,"cache_read_price":0.5,"cache_write_price":0,"request_price":0,"price_source_kind":"official","billing_notes":"Official pricing snapshot.","pricing_tiers":[],"is_active":true}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_price_without_provider_model.status(), StatusCode::BAD_REQUEST);

    let create_provider_model = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/provider-models")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"proxy_provider_id":"provider-openai-official","channel_id":"openai","model_id":"gpt-4.1","provider_model_id":"gpt-4.1","capabilities":["responses"],"streaming":true,"context_window":128000,"max_output_tokens":32768,"supports_prompt_caching":true,"supports_reasoning_usage":false,"supports_tool_usage_metrics":true,"is_default_route":true,"is_active":true}"#,
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
                    r#"{"channel_id":"openai","model_id":"gpt-4.1","proxy_provider_id":"provider-openai-official","currency_code":"USD","price_unit":"per_1m_tokens","input_price":2,"output_price":8,"cache_read_price":0.5,"cache_write_price":0,"request_price":0,"price_source_kind":"official","billing_notes":"Official pricing snapshot.","pricing_tiers":[],"is_active":true}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_price.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn create_provider_syncs_supported_models_and_exposes_provider_model_registry() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    for channel in [
        r#"{"id":"openai","name":"OpenAI"}"#,
        r#"{"id":"anthropic","name":"Anthropic"}"#,
        r#"{"id":"gemini","name":"Gemini"}"#,
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

    for model in [
        r#"{"channel_id":"openai","model_id":"gpt-4.1","model_display_name":"GPT-4.1","capabilities":["responses"],"streaming":true,"context_window":128000}"#,
        r#"{"channel_id":"anthropic","model_id":"claude-3-7-sonnet","model_display_name":"Claude 3.7 Sonnet","capabilities":["responses"],"streaming":true,"context_window":200000}"#,
        r#"{"channel_id":"gemini","model_id":"gemini-2.5-pro","model_display_name":"Gemini 2.5 Pro","capabilities":["responses"],"streaming":true,"context_window":1048576}"#,
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/admin/channel-models")
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(model))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    let create_provider = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "id":"provider-openrouter-main",
                        "channel_id":"openai",
                        "default_plugin_family":"openrouter",
                        "base_url":"https://openrouter.ai/api/v1",
                        "display_name":"OpenRouter Main",
                        "channel_bindings":[
                            {"channel_id":"openai","is_primary":true},
                            {"channel_id":"anthropic","is_primary":false},
                            {"channel_id":"gemini","is_primary":false}
                        ],
                        "supported_models":[
                            {
                                "channel_id":"openai",
                                "model_id":"gpt-4.1",
                                "provider_model_id":"openai/gpt-4.1",
                                "provider_model_family":"openai",
                                "capabilities":["responses"],
                                "streaming":true,
                                "context_window":128000,
                                "max_output_tokens":32768,
                                "supports_reasoning_usage":true,
                                "is_default_route":true,
                                "is_active":true
                            },
                            {
                                "channel_id":"anthropic",
                                "model_id":"claude-3-7-sonnet",
                                "provider_model_id":"anthropic/claude-3.7-sonnet",
                                "provider_model_family":"anthropic",
                                "capabilities":["responses"],
                                "streaming":true,
                                "context_window":200000,
                                "supports_prompt_caching":true,
                                "is_active":true
                            }
                        ]
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_provider.status(), StatusCode::CREATED);

    let listed = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/provider-models")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(listed.status(), StatusCode::OK);
    let listed_json = read_json(listed).await;
    let provider_models = listed_json.as_array().unwrap();
    assert_eq!(provider_models.len(), 2);
    assert!(provider_models.iter().any(|record| {
        record["proxy_provider_id"] == "provider-openrouter-main"
            && record["channel_id"] == "openai"
            && record["model_id"] == "gpt-4.1"
            && record["provider_model_id"] == "openai/gpt-4.1"
            && record["is_default_route"] == true
    }));
    assert!(provider_models.iter().any(|record| {
        record["proxy_provider_id"] == "provider-openrouter-main"
            && record["channel_id"] == "anthropic"
            && record["model_id"] == "claude-3-7-sonnet"
            && record["supports_prompt_caching"] == true
    }));

    let deleted = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(
                    "/admin/provider-models/provider-openrouter-main/channels/anthropic/models/claude-3-7-sonnet",
                )
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(deleted.status(), StatusCode::NO_CONTENT);
}

#[serial(extension_env)]
#[tokio::test]
async fn legacy_coupon_route_is_removed() {
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

    assert_eq!(initial.status(), StatusCode::NOT_FOUND);

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

    assert_eq!(created.status(), StatusCode::NOT_FOUND);

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

    assert_eq!(deleted.status(), StatusCode::NOT_FOUND);

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

    assert_eq!(after_delete.status(), StatusCode::NOT_FOUND);
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
        r#"{"id":"provider-openrouter","channel_id":"openrouter","display_name":"OpenRouter","default_plugin_family":"openrouter","base_url":"https://openrouter.ai/api/v1","channel_bindings":[{"channel_id":"openrouter","is_primary":true}]}"#,
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

fn provider_json_by_id<'a>(providers_json: &'a Value, provider_id: &str) -> &'a Value {
    providers_json
        .as_array()
        .and_then(|providers| {
            providers
                .iter()
                .find(|provider| provider["id"].as_str() == Some(provider_id))
        })
        .unwrap_or_else(|| panic!("provider {provider_id} not found in {providers_json}"))
}
