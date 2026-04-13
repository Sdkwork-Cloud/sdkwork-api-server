use super::support::*;
use sdkwork_api_storage_sqlite::SqliteAdminStore;

#[tokio::test]
async fn create_list_and_delete_provider_accounts() {
    let pool = memory_pool().await;
    let store = SqliteAdminStore::new(pool.clone());
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    create_provider_fixture(
        app.clone(),
        &token,
        r#"{"id":"provider-openai-official","channel_id":"openai","adapter_kind":"openai","base_url":"https://api.openai.com/v1","display_name":"OpenAI Official","channel_bindings":[{"channel_id":"openai","is_primary":true}]}"#,
    )
    .await;

    store
        .insert_extension_installation(
            &ExtensionInstallation::new(
                "install-openai-builtin",
                "sdkwork.provider.openai.official",
                ExtensionRuntime::Builtin,
            )
            .with_enabled(true)
            .with_config(json!({})),
        )
        .await
        .unwrap();
    store
        .insert_extension_instance(
            &ExtensionInstance::new(
                "instance-openai-primary",
                "install-openai-builtin",
                "sdkwork.provider.openai.official",
            )
            .with_enabled(true)
            .with_base_url("https://api.openai.com/v1")
            .with_credential_ref("cred-openai-primary")
            .with_config(json!({})),
        )
        .await
        .unwrap();

    let created = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/provider-accounts")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"provider_account_id":"acct-openai-primary","provider_id":"provider-openai-official","display_name":"OpenAI Primary","account_kind":"api_key","execution_instance_id":"instance-openai-primary","region":"us-east","priority":100,"weight":10,"enabled":true}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(created.status(), StatusCode::CREATED);
    let created_json = read_json(created).await;
    assert_eq!(created_json["provider_account_id"], "acct-openai-primary");
    assert_eq!(created_json["provider_id"], "provider-openai-official");
    assert_eq!(
        created_json["execution_instance_id"],
        "instance-openai-primary"
    );
    assert_eq!(created_json["region"], "us-east");

    let listed = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/provider-accounts")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(listed.status(), StatusCode::OK);
    let listed_json = read_json(listed).await;
    assert_eq!(listed_json.as_array().unwrap().len(), 1);
    assert_eq!(listed_json[0]["provider_account_id"], "acct-openai-primary");

    let deleted = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/admin/provider-accounts/acct-openai-primary")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(deleted.status(), StatusCode::NO_CONTENT);
}
