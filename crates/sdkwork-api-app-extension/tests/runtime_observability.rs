use sdkwork_api_app_extension::{
    capture_provider_health_snapshots, list_discovered_extension_packages,
    list_extension_runtime_statuses,
};
use sdkwork_api_domain_catalog::{Channel, ProxyProvider};
use sdkwork_api_extension_host::{
    ensure_connector_runtime_started, load_native_dynamic_provider_adapter,
    shutdown_all_connector_runtimes, shutdown_all_native_dynamic_runtimes,
    ExtensionDiscoveryPolicy, ExtensionLoadPlan,
};
use sdkwork_api_storage_sqlite::{run_migrations, SqliteAdminStore};
use serde_json::json;
use serial_test::serial;
use std::fs;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn lists_discovered_extension_packages_from_policy() {
    let root = temp_extension_root("discovery-observability");
    let package_dir = root.join("sdkwork-provider-custom-openai");
    fs::create_dir_all(&package_dir).unwrap();
    fs::write(
        package_dir.join("sdkwork-extension.toml"),
        connector_manifest("sdkwork.provider.custom-openai"),
    )
    .unwrap();

    let packages =
        list_discovered_extension_packages(&ExtensionDiscoveryPolicy::new(vec![root.clone()]))
            .unwrap();

    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].manifest.id, "sdkwork.provider.custom-openai");
    assert_eq!(packages[0].root_dir, package_dir);
    assert_eq!(packages[0].trust.state.as_str(), "unsigned");
    assert!(!packages[0].trust.signature_present);
    assert!(packages[0].trust.load_allowed);
    cleanup_dir(&root);
}

#[cfg(windows)]
#[test]
fn lists_active_connector_runtime_statuses_from_host_registry() {
    let root = temp_extension_root("runtime-status-observability");
    fs::create_dir_all(&root).unwrap();

    let port = free_port();
    fs::write(root.join("connector.ps1"), connector_script_body(port)).unwrap();

    let load_plan = ExtensionLoadPlan {
        instance_id: "provider-custom-openai".to_owned(),
        installation_id: "custom-openai-installation".to_owned(),
        extension_id: "sdkwork.provider.custom-openai".to_owned(),
        enabled: true,
        runtime: sdkwork_api_extension_core::ExtensionRuntime::Connector,
        display_name: "Custom OpenAI".to_owned(),
        entrypoint: Some("powershell.exe".to_owned()),
        base_url: Some(format!("http://127.0.0.1:{port}")),
        credential_ref: None,
        config_schema: None,
        credential_schema: None,
        package_root: Some(root.clone()),
        config: json!({
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

    let statuses = list_extension_runtime_statuses().unwrap();
    assert_eq!(statuses.len(), 1);
    assert_eq!(statuses[0].runtime, "connector");
    assert_eq!(statuses[0].extension_id, "sdkwork.provider.custom-openai");
    assert_eq!(statuses[0].instance_id, "provider-custom-openai");
    assert!(statuses[0].running);
    assert!(statuses[0].healthy);

    shutdown_all_connector_runtimes().unwrap();
    cleanup_dir(&root);
}

#[serial(extension_runtime)]
#[test]
fn lists_active_native_dynamic_runtime_statuses_from_host_registry() {
    shutdown_all_native_dynamic_runtimes().unwrap();

    let library_path = native_dynamic_fixture_library_path();
    let _adapter =
        load_native_dynamic_provider_adapter(&library_path, "https://example.com/v1").unwrap();

    let statuses = list_extension_runtime_statuses().unwrap();
    assert_eq!(statuses.len(), 1);
    assert_eq!(statuses[0].runtime, "native_dynamic");
    assert_eq!(statuses[0].extension_id, "sdkwork.provider.native.mock");
    assert!(statuses[0].instance_id.is_empty());
    assert!(statuses[0].running);
    assert!(statuses[0].healthy);
    assert!(statuses[0].supports_health_check);
    assert!(statuses[0].supports_shutdown);
    assert_eq!(statuses[0].message.as_deref(), Some("native mock healthy"));

    shutdown_all_native_dynamic_runtimes().unwrap();
}

#[serial(extension_runtime)]
#[tokio::test]
async fn captures_provider_health_snapshots_from_runtime_statuses() {
    shutdown_all_native_dynamic_runtimes().unwrap();

    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    store
        .insert_channel(&Channel::new("openai", "OpenAI"))
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                "provider-native-mock",
                "openai",
                "openai",
                "https://example.com/v1",
                "Native Mock",
            )
            .with_extension_id("sdkwork.provider.native.mock"),
        )
        .await
        .unwrap();

    let library_path = native_dynamic_fixture_library_path();
    let _adapter =
        load_native_dynamic_provider_adapter(&library_path, "https://example.com/v1").unwrap();

    let captured = capture_provider_health_snapshots(&store).await.unwrap();
    assert_eq!(captured.len(), 1);
    assert_eq!(captured[0].provider_id, "provider-native-mock");
    assert_eq!(captured[0].runtime, "native_dynamic");
    assert!(captured[0].healthy);

    let stored = store.list_provider_health_snapshots().await.unwrap();
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].provider_id, "provider-native-mock");

    shutdown_all_native_dynamic_runtimes().unwrap();
}

fn connector_manifest(extension_id: &str) -> String {
    format!(
        r#"
api_version = "sdkwork.extension/v1"
id = "{extension_id}"
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
"#
    )
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

fn temp_extension_root(suffix: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    path.push(format!("sdkwork-app-extension-{suffix}-{millis}"));
    path
}

fn cleanup_dir(path: &Path) {
    let _ = fs::remove_dir_all(path);
}

fn native_dynamic_fixture_library_path() -> PathBuf {
    let current_exe = std::env::current_exe().expect("current exe");
    let directory = current_exe.parent().expect("exe dir");
    let prefix = if cfg!(windows) {
        "sdkwork_api_ext_provider_native_mock"
    } else {
        "libsdkwork_api_ext_provider_native_mock"
    };
    let extension = if cfg!(windows) {
        "dll"
    } else if cfg!(target_os = "macos") {
        "dylib"
    } else {
        "so"
    };

    std::fs::read_dir(directory)
        .expect("deps dir")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| {
            path.extension().and_then(|value| value.to_str()) == Some(extension)
                && path
                    .file_stem()
                    .and_then(|value| value.to_str())
                    .is_some_and(|stem| stem.starts_with(prefix))
        })
        .expect("native dynamic fixture library")
}

#[cfg(windows)]
fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}
