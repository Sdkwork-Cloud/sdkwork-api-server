use super::support::*;

#[tokio::test]
async fn create_and_list_extension_installations_and_instances() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let installation = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/installations")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"installation_id\":\"openrouter-builtin\",\"extension_id\":\"sdkwork.provider.openrouter\",\"runtime\":\"builtin\",\"enabled\":true,\"entrypoint\":null,\"config\":{\"trust_mode\":\"builtin\"}}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(installation.status(), StatusCode::CREATED);

    let instance = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/instances")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"instance_id\":\"provider-openrouter-main\",\"installation_id\":\"openrouter-builtin\",\"extension_id\":\"sdkwork.provider.openrouter\",\"enabled\":true,\"base_url\":\"https://openrouter.ai/api/v1\",\"credential_ref\":\"cred-openrouter\",\"config\":{\"region\":\"global\",\"weight\":100}}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(instance.status(), StatusCode::CREATED);

    let installations = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/extensions/installations")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(installations.status(), StatusCode::OK);
    let installations_json = read_json(installations).await;
    assert_eq!(
        installations_json[0]["extension_id"],
        "sdkwork.provider.openrouter"
    );
    assert_eq!(installations_json[0]["runtime"], "builtin");

    let instances = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/extensions/instances")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(instances.status(), StatusCode::OK);
    let instances_json = read_json(instances).await;
    assert_eq!(instances_json[0]["instance_id"], "provider-openrouter-main");
    assert_eq!(
        instances_json[0]["base_url"],
        "https://openrouter.ai/api/v1"
    );
    assert_eq!(instances_json[0]["config"]["region"], "global");
}

#[tokio::test]
#[serial(extension_env)]
async fn list_discovered_extension_packages_from_admin_api() {
    let root = temp_extension_root("admin-extension-packages");
    let package_dir = root.join("sdkwork-provider-custom-openai");
    fs::create_dir_all(&package_dir).unwrap();
    fs::write(
        package_dir.join("sdkwork-extension.toml"),
        r#"
api_version = "sdkwork.extension/v1"
id = "sdkwork.provider.custom-openai"
kind = "provider"
version = "0.1.0"
display_name = "Custom OpenAI"
runtime = "connector"
runtime_compat_version = "sdkwork.runtime/v1"
protocol = "openai"
entrypoint = "powershell.exe"
config_schema_version = "1.0"
channel_bindings = ["sdkwork.channel.openai"]
supported_modalities = ["text", "image", "audio", "video", "file"]
permissions = ["network_outbound", "spawn_process"]

[health]
path = "/health"
interval_secs = 30

[[capabilities]]
operation = "chat.completions.create"
compatibility = "relay"
"#,
    )
    .unwrap();
    let _guard = extension_env_guard(&root);

    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/extensions/packages")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json.as_array().unwrap().len(), 1);
    assert_eq!(json[0]["manifest"]["id"], "sdkwork.provider.custom-openai");
    assert_eq!(
        json[0]["root_dir"],
        package_dir.to_string_lossy().to_string()
    );
    assert_eq!(
        json[0]["distribution_name"],
        "sdkwork-provider-custom-openai"
    );
    assert_eq!(
        json[0]["crate_name"],
        "sdkwork-api-ext-provider-custom-openai"
    );
    assert_eq!(json[0]["validation"]["valid"], true);
    assert_eq!(json[0]["validation"]["issues"].as_array().unwrap().len(), 0);
    assert_eq!(json[0]["trust"]["state"], "unsigned");
    assert_eq!(json[0]["trust"]["signature_present"], false);
    assert_eq!(json[0]["trust"]["load_allowed"], true);

    cleanup_dir(&root);
}
#[tokio::test]
#[serial(extension_env)]
async fn list_discovered_extension_packages_reloads_current_extension_policy_from_environment() {
    let root_one = temp_extension_root("admin-extension-packages-dynamic-one");
    let root_two = temp_extension_root("admin-extension-packages-dynamic-two");
    let package_dir = root_two.join("sdkwork-provider-custom-openai");
    fs::create_dir_all(&package_dir).unwrap();
    fs::write(
        package_dir.join("sdkwork-extension.toml"),
        r#"
api_version = "sdkwork.extension/v1"
id = "sdkwork.provider.custom-openai"
kind = "provider"
version = "0.1.0"
display_name = "Custom OpenAI"
runtime = "connector"
runtime_compat_version = "sdkwork.runtime/v1"
protocol = "openai"
entrypoint = "powershell.exe"
config_schema_version = "1.0"
channel_bindings = ["sdkwork.channel.openai"]
supported_modalities = ["text", "image", "audio", "video", "file"]
permissions = ["network_outbound", "spawn_process"]

[health]
path = "/health"
interval_secs = 30

[[capabilities]]
operation = "chat.completions.create"
compatibility = "relay"
"#,
    )
    .unwrap();

    let _guard = extension_env_guard(&root_one);

    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    std::env::set_var(
        "SDKWORK_EXTENSION_PATHS",
        std::env::join_paths([&root_two]).unwrap(),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/extensions/packages")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json.as_array().unwrap().len(), 1);
    assert_eq!(json[0]["manifest"]["id"], "sdkwork.provider.custom-openai");
    assert_eq!(
        json[0]["root_dir"],
        package_dir.to_string_lossy().to_string()
    );

    cleanup_dir(&root_one);
    cleanup_dir(&root_two);
}

#[cfg(windows)]
#[serial(extension_env)]
#[tokio::test]
async fn list_active_connector_runtime_statuses_from_admin_api() {
    let root = temp_extension_root("admin-runtime-statuses");
    fs::create_dir_all(&root).unwrap();
    let port = free_port();
    fs::write(root.join("connector.ps1"), connector_script_body(port)).unwrap();

    let load_plan = ExtensionLoadPlan {
        instance_id: "provider-custom-openai".to_owned(),
        installation_id: "custom-openai-installation".to_owned(),
        extension_id: "sdkwork.provider.custom-openai".to_owned(),
        enabled: true,
        runtime: ExtensionRuntime::Connector,
        display_name: "Custom OpenAI".to_owned(),
        entrypoint: Some("powershell.exe".to_owned()),
        base_url: Some(format!("http://127.0.0.1:{port}")),
        credential_ref: None,
        config_schema: None,
        credential_schema: None,
        package_root: Some(root.clone()),
        config: serde_json::json!({
            "command_args": [
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
                "connector.ps1"
            ],
            "health_path": "/health",
            "startup_timeout_ms": 4000,
            "startup_poll_interval_ms": 50
        }),
    };

    ensure_connector_runtime_started(&load_plan, load_plan.base_url.as_deref().expect("base url"))
        .unwrap();

    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/extensions/runtime-statuses")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json.as_array().unwrap().len(), 1);
    assert_eq!(json[0]["runtime"], "connector");
    assert_eq!(json[0]["extension_id"], "sdkwork.provider.custom-openai");
    assert_eq!(json[0]["instance_id"], "provider-custom-openai");
    assert_eq!(json[0]["running"], true);
    assert_eq!(json[0]["healthy"], true);

    shutdown_all_connector_runtimes().unwrap();
    cleanup_dir(&root);
}

#[serial(extension_env)]
#[tokio::test]
async fn list_active_native_dynamic_runtime_statuses_from_admin_api() {
    shutdown_all_native_dynamic_runtimes().unwrap();

    let library_path = native_dynamic_fixture_library_path();
    let _adapter =
        load_native_dynamic_provider_adapter(&library_path, "https://example.com/v1").unwrap();

    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/extensions/runtime-statuses")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json.as_array().unwrap().len(), 1);
    assert_eq!(json[0]["runtime"], "native_dynamic");
    assert_eq!(json[0]["extension_id"], "sdkwork.provider.native.mock");
    assert_eq!(json[0]["running"], true);
    assert_eq!(json[0]["healthy"], true);
    assert_eq!(json[0]["supports_health_check"], true);
    assert_eq!(json[0]["supports_shutdown"], true);
    assert_eq!(json[0]["message"], "native mock healthy");

    shutdown_all_native_dynamic_runtimes().unwrap();
}

#[serial(extension_env)]
#[tokio::test]
async fn extension_runtime_reload_endpoint_rebuilds_runtime_state() {
    shutdown_all_native_dynamic_runtimes().unwrap();

    let log_guard = NativeDynamicLifecycleLogGuard::new();
    let extension_root = temp_extension_root("admin-runtime-reload");
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
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let first = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/runtime-reloads")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::OK);

    let second = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/runtime-reloads")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(second.status(), StatusCode::OK);
    let json = read_json(second).await;
    assert_eq!(json["discovered_package_count"], 1);
    assert_eq!(json["loadable_package_count"], 1);
    assert_eq!(json["active_runtime_count"], 1);
    assert_eq!(json["runtime_statuses"][0]["runtime"], "native_dynamic");
    assert_eq!(
        json["runtime_statuses"][0]["extension_id"],
        FIXTURE_EXTENSION_ID
    );
    assert_eq!(json["runtime_statuses"][0]["running"], true);
    assert_eq!(json["runtime_statuses"][0]["healthy"], true);
    assert_eq!(
        std::fs::read_to_string(log_guard.path())
            .unwrap()
            .lines()
            .collect::<Vec<_>>(),
        vec!["init", "shutdown", "init"]
    );

    shutdown_all_native_dynamic_runtimes().unwrap();
    cleanup_dir(&extension_root);
}

#[serial(extension_env)]
#[tokio::test]
async fn extension_runtime_reload_endpoint_supports_targeted_scope() {
    shutdown_all_native_dynamic_runtimes().unwrap();

    let log_guard = NativeDynamicLifecycleLogGuard::new();
    let extension_root = temp_extension_root("admin-runtime-reload-targeted");
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
                "connector-mock-installation",
                "sdkwork.provider.connector.mock",
                ExtensionRuntime::Connector,
            )
            .with_enabled(true)
            .with_entrypoint("connector-mock"),
        )
        .await
        .unwrap();
    store
        .insert_extension_instance(
            &ExtensionInstance::new(
                "provider-connector-mock",
                "connector-mock-installation",
                "sdkwork.provider.connector.mock",
            )
            .with_enabled(true)
            .with_base_url("http://127.0.0.1:9"),
        )
        .await
        .unwrap();

    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let initial = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/runtime-reloads")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(initial.status(), StatusCode::OK);
    assert_eq!(
        std::fs::read_to_string(log_guard.path())
            .unwrap()
            .lines()
            .collect::<Vec<_>>(),
        vec!["init"]
    );

    let by_extension = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/runtime-reloads")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "extension_id": FIXTURE_EXTENSION_ID,
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(by_extension.status(), StatusCode::OK);
    let extension_json = read_json(by_extension).await;
    assert_eq!(extension_json["scope"], "extension");
    assert_eq!(
        extension_json["requested_extension_id"],
        FIXTURE_EXTENSION_ID
    );
    assert_eq!(extension_json["requested_instance_id"], Value::Null);
    assert_eq!(
        extension_json["resolved_extension_id"],
        FIXTURE_EXTENSION_ID
    );
    assert_eq!(extension_json["discovered_package_count"], 1);
    assert_eq!(extension_json["loadable_package_count"], 1);
    assert_eq!(extension_json["active_runtime_count"], 1);
    assert_eq!(
        std::fs::read_to_string(log_guard.path())
            .unwrap()
            .lines()
            .collect::<Vec<_>>(),
        vec!["init", "shutdown", "init"]
    );

    let by_instance = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/runtime-reloads")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "instance_id": "provider-connector-mock",
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(by_instance.status(), StatusCode::OK);
    let instance_json = read_json(by_instance).await;
    assert_eq!(instance_json["scope"], "instance");
    assert_eq!(instance_json["requested_extension_id"], Value::Null);
    assert_eq!(
        instance_json["requested_instance_id"],
        "provider-connector-mock"
    );
    assert_eq!(
        instance_json["resolved_extension_id"],
        "sdkwork.provider.connector.mock"
    );
    assert_eq!(instance_json["active_runtime_count"], 1);
    assert_eq!(
        std::fs::read_to_string(log_guard.path())
            .unwrap()
            .lines()
            .collect::<Vec<_>>(),
        vec!["init", "shutdown", "init"]
    );

    let invalid = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/runtime-reloads")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "extension_id": FIXTURE_EXTENSION_ID,
                        "instance_id": "provider-connector-mock",
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(invalid.status(), StatusCode::BAD_REQUEST);

    shutdown_all_native_dynamic_runtimes().unwrap();
    cleanup_dir(&extension_root);
}
