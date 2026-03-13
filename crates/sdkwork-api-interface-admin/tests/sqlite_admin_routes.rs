use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use axum::Router;
use serde_json::Value;
use serial_test::serial;
use sqlx::SqlitePool;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tower::ServiceExt;

#[cfg(windows)]
use sdkwork_api_extension_core::ExtensionRuntime;
#[cfg(windows)]
use sdkwork_api_extension_host::{
    ensure_connector_runtime_started, shutdown_all_connector_runtimes, ExtensionLoadPlan,
};
#[cfg(windows)]
use std::net::TcpListener;

async fn read_json(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn memory_pool() -> SqlitePool {
    sdkwork_api_storage_sqlite::run_migrations("sqlite::memory:")
        .await
        .unwrap()
}

async fn login_token(app: Router) -> String {
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/auth/login")
                .header("content-type", "application/json")
                .body(Body::from("{\"subject\":\"admin-user\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    read_json(response).await["token"]
        .as_str()
        .unwrap()
        .to_owned()
}

#[serial(extension_env)]
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
                .body(Body::from("{\"subject\":\"admin-user\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = read_json(response).await;
    assert_eq!(json["claims"]["sub"], "admin-user");
    assert_eq!(json["token"].as_str().unwrap().split('.').count(), 3);
}

#[serial(extension_env)]
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
    assert_eq!(json.as_array().unwrap().len(), 1);
    assert_eq!(json[0]["id"], "openai");
}

#[serial(extension_env)]
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

    assert_eq!(provider.status(), StatusCode::CREATED);

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

#[serial(extension_env)]
#[tokio::test]
async fn create_and_list_models() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

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
async fn routing_simulation_uses_catalog_models() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let create_openrouter = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-openrouter\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_openrouter.status(), StatusCode::CREATED);

    let create_openai = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-openai-official\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_openai.status(), StatusCode::CREATED);

    let simulate = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/simulations")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"capability\":\"chat_completion\",\"model\":\"gpt-4.1\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(simulate.status(), StatusCode::OK);
    let simulation_json = read_json(simulate).await;
    assert_eq!(
        simulation_json["selected_provider_id"],
        "provider-openai-official"
    );
    assert_eq!(
        simulation_json["candidate_ids"].as_array().unwrap().len(),
        2
    );
}

#[serial(extension_env)]
#[tokio::test]
async fn create_and_list_routing_policies() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"policy_id\":\"policy-gpt-4-1\",\"capability\":\"chat_completion\",\"model_pattern\":\"gpt-4.1\",\"enabled\":true,\"priority\":100,\"ordered_provider_ids\":[\"provider-openrouter\",\"provider-openai-official\"],\"default_provider_id\":\"provider-openai-official\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create.status(), StatusCode::CREATED);
    let created_json = read_json(create).await;
    assert_eq!(created_json["policy_id"], "policy-gpt-4-1");
    assert_eq!(created_json["priority"], 100);

    let list = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(list.status(), StatusCode::OK);
    let list_json = read_json(list).await;
    assert_eq!(list_json[0]["policy_id"], "policy-gpt-4-1");
    assert_eq!(
        list_json[0]["ordered_provider_ids"][0],
        "provider-openrouter"
    );
    assert_eq!(
        list_json[0]["default_provider_id"],
        "provider-openai-official"
    );
}

#[serial(extension_env)]
#[tokio::test]
async fn routing_simulation_reports_policy_selected_provider() {
    let pool = memory_pool().await;
    let app = sdkwork_api_interface_admin::admin_router_with_pool(pool);
    let token = login_token(app.clone()).await;

    let create_openrouter = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-openrouter\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_openrouter.status(), StatusCode::CREATED);

    let create_openai = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-openai-official\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_openai.status(), StatusCode::CREATED);

    let create_policy = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/policies")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"policy_id\":\"policy-gpt-4-1\",\"capability\":\"chat_completion\",\"model_pattern\":\"gpt-4.1\",\"enabled\":true,\"priority\":100,\"ordered_provider_ids\":[\"provider-openrouter\",\"provider-openai-official\"],\"default_provider_id\":\"provider-openai-official\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_policy.status(), StatusCode::CREATED);

    let simulate = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/routing/simulations")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"capability\":\"chat_completion\",\"model\":\"gpt-4.1\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(simulate.status(), StatusCode::OK);
    let simulation_json = read_json(simulate).await;
    assert_eq!(
        simulation_json["selected_provider_id"],
        "provider-openrouter"
    );
    assert_eq!(simulation_json["matched_policy_id"], "policy-gpt-4-1");
}

#[serial(extension_env)]
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

#[serial(extension_env)]
#[tokio::test]
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
protocol = "openai"
entrypoint = "powershell.exe"
channel_bindings = ["sdkwork.channel.openai"]
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
    assert_eq!(json[0]["instance_id"], "provider-custom-openai");
    assert_eq!(json[0]["running"], true);
    assert_eq!(json[0]["healthy"], true);

    shutdown_all_connector_runtimes().unwrap();
    cleanup_dir(&root);
}

fn temp_extension_root(suffix: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    path.push(format!("sdkwork-admin-routes-{suffix}-{millis}"));
    path
}

fn cleanup_dir(path: &Path) {
    let _ = fs::remove_dir_all(path);
}

fn extension_env_guard(path: &Path) -> ExtensionEnvGuard {
    let previous_paths = std::env::var("SDKWORK_EXTENSION_PATHS").ok();
    let previous_connector = std::env::var("SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS").ok();
    let previous_native = std::env::var("SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS").ok();

    let joined_paths = std::env::join_paths([path]).unwrap();
    std::env::set_var("SDKWORK_EXTENSION_PATHS", joined_paths);
    std::env::set_var("SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS", "true");
    std::env::set_var(
        "SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS",
        "false",
    );

    ExtensionEnvGuard {
        previous_paths,
        previous_connector,
        previous_native,
    }
}

struct ExtensionEnvGuard {
    previous_paths: Option<String>,
    previous_connector: Option<String>,
    previous_native: Option<String>,
}

impl Drop for ExtensionEnvGuard {
    fn drop(&mut self) {
        restore_env_var("SDKWORK_EXTENSION_PATHS", self.previous_paths.as_deref());
        restore_env_var(
            "SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS",
            self.previous_connector.as_deref(),
        );
        restore_env_var(
            "SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS",
            self.previous_native.as_deref(),
        );
    }
}

fn restore_env_var(key: &str, value: Option<&str>) {
    match value {
        Some(value) => std::env::set_var(key, value),
        None => std::env::remove_var(key),
    }
}

#[cfg(windows)]
fn connector_script_body(port: u16) -> String {
    format!(
        r#"
$listener = [System.Net.Sockets.TcpListener]::new([System.Net.IPAddress]::Parse("127.0.0.1"), {port})
$listener.Start()
while ($true) {{
    $client = $listener.AcceptTcpClient()
    $stream = $client.GetStream()
    $reader = New-Object System.IO.StreamReader($stream, [System.Text.Encoding]::ASCII, $false, 1024, $true)
    $requestLine = $reader.ReadLine()
    while ($true) {{
        $line = $reader.ReadLine()
        if ([string]::IsNullOrEmpty($line)) {{
            break
        }}
    }}

    if ($requestLine.StartsWith('GET /health')) {{
        $body = '{{"status":"ok"}}'
        $status = 'HTTP/1.1 200 OK'
    }} else {{
        $body = '{{"error":"not_found"}}'
        $status = 'HTTP/1.1 404 Not Found'
    }}

    $bodyBytes = [System.Text.Encoding]::UTF8.GetBytes($body)
    $writer = New-Object System.IO.StreamWriter($stream, [System.Text.Encoding]::ASCII, 1024, $true)
    $writer.NewLine = "`r`n"
    $writer.WriteLine($status)
    $writer.WriteLine('Content-Type: application/json')
    $writer.WriteLine(('Content-Length: ' + $bodyBytes.Length))
    $writer.WriteLine('Connection: close')
    $writer.WriteLine()
    $writer.Flush()
    $stream.Write($bodyBytes, 0, $bodyBytes.Length)
    $stream.Flush()
    $client.Close()
}}
"#
    )
}

#[cfg(windows)]
fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}
