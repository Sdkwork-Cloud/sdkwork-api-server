use sdkwork_api_app_extension::{
    list_connector_runtime_statuses, list_discovered_extension_packages,
};
use sdkwork_api_extension_host::{
    ensure_connector_runtime_started, shutdown_all_connector_runtimes, ExtensionDiscoveryPolicy,
    ExtensionLoadPlan,
};
use serde_json::json;
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

    let statuses = list_connector_runtime_statuses().unwrap();
    assert_eq!(statuses.len(), 1);
    assert_eq!(statuses[0].instance_id, "provider-custom-openai");
    assert!(statuses[0].running);
    assert!(statuses[0].healthy);

    shutdown_all_connector_runtimes().unwrap();
    cleanup_dir(&root);
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

#[cfg(windows)]
fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}
