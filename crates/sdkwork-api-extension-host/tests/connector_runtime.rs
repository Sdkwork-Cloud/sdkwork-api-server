#![cfg(windows)]

use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use sdkwork_api_extension_core::{
    ExtensionInstallation, ExtensionInstance, ExtensionKind, ExtensionManifest, ExtensionRuntime,
};
use sdkwork_api_extension_host::{
    discover_extension_packages, ensure_connector_runtime_started, shutdown_connector_runtime,
    ConnectorRuntimeStatus, ExtensionDiscoveryPolicy, ExtensionHost,
};

#[test]
fn host_starts_discovered_connector_process_and_reports_health() {
    let root = temp_extension_root("connector-runtime");
    let package_dir = root.join("sdkwork-provider-custom-openai");
    fs::create_dir_all(&package_dir).unwrap();

    let port = free_port();
    fs::write(
        package_dir.join("sdkwork-extension.toml"),
        connector_manifest("sdkwork.provider.custom-openai"),
    )
    .unwrap();
    fs::write(
        package_dir.join("connector.ps1"),
        connector_script_body(port, None),
    )
    .unwrap();

    let policy = ExtensionDiscoveryPolicy::new(vec![root.clone()])
        .with_connector_extensions(true)
        .with_native_dynamic_extensions(false);
    let packages = discover_extension_packages(&policy).unwrap();

    let mut host = ExtensionHost::new();
    host.register_builtin_manifest(
        ExtensionManifest::new(
            "sdkwork.provider.openai.official",
            ExtensionKind::Provider,
            "0.1.0",
            ExtensionRuntime::Builtin,
        )
        .with_entrypoint("powershell.exe"),
    );
    host.register_discovered_manifest(packages[0].clone());
    host.install(
        ExtensionInstallation::new(
            "custom-openai-installation",
            "sdkwork.provider.custom-openai",
            ExtensionRuntime::Connector,
        )
        .with_config(serde_json::json!({
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
        })),
    )
    .unwrap();
    host.mount_instance(
        ExtensionInstance::new(
            "provider-custom-openai",
            "custom-openai-installation",
            "sdkwork.provider.custom-openai",
        )
        .with_base_url(format!("http://127.0.0.1:{port}")),
    )
    .unwrap();

    let plan = host.load_plan("provider-custom-openai").unwrap();
    let status =
        ensure_connector_runtime_started(&plan, plan.base_url.as_deref().expect("base url"))
            .unwrap();

    assert_eq!(
        status,
        ConnectorRuntimeStatus {
            instance_id: "provider-custom-openai".to_owned(),
            extension_id: "sdkwork.provider.custom-openai".to_owned(),
            display_name: "Custom OpenAI".to_owned(),
            base_url: format!("http://127.0.0.1:{port}"),
            health_url: format!("http://127.0.0.1:{port}/health"),
            process_id: status.process_id,
            running: true,
            healthy: true,
        }
    );

    shutdown_connector_runtime("provider-custom-openai").unwrap();
    cleanup_dir(&root);
}

#[test]
fn host_reuses_healthy_external_connector_endpoint_without_spawning() {
    let root = temp_extension_root("connector-external-runtime");
    let package_dir = root.join("sdkwork-provider-custom-openai");
    fs::create_dir_all(&package_dir).unwrap();

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buffer = [0_u8; 1024];
            let _ = stream.read(&mut buffer);
            let response =
                b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 15\r\nConnection: close\r\n\r\n{\"status\":\"ok\"}";
            let _ = stream.write_all(response);
            let _ = stream.flush();
        }
    });

    fs::write(
        package_dir.join("sdkwork-extension.toml"),
        connector_manifest("sdkwork.provider.custom-openai"),
    )
    .unwrap();

    let policy = ExtensionDiscoveryPolicy::new(vec![root.clone()])
        .with_connector_extensions(true)
        .with_native_dynamic_extensions(false);
    let packages = discover_extension_packages(&policy).unwrap();

    let mut host = ExtensionHost::new();
    host.register_discovered_manifest(packages[0].clone());
    host.install(
        ExtensionInstallation::new(
            "custom-openai-installation",
            "sdkwork.provider.custom-openai",
            ExtensionRuntime::Connector,
        )
        .with_entrypoint("bin/sdkwork-provider-custom-openai"),
    )
    .unwrap();
    host.mount_instance(
        ExtensionInstance::new(
            "provider-custom-openai-external",
            "custom-openai-installation",
            "sdkwork.provider.custom-openai",
        )
        .with_base_url(format!("http://127.0.0.1:{port}"))
        .with_config(serde_json::json!({
            "health_path": "/health"
        })),
    )
    .unwrap();

    let plan = host.load_plan("provider-custom-openai-external").unwrap();
    let status =
        ensure_connector_runtime_started(&plan, plan.base_url.as_deref().expect("base url"))
            .unwrap();

    assert_eq!(status.process_id, None);
    assert!(status.running);
    assert!(status.healthy);
    assert_eq!(status.health_url, format!("http://127.0.0.1:{port}/health"));

    server.join().unwrap();
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

[[capabilities]]
operation = "chat.completions.create"
compatibility = "relay"
"#
    )
}

fn connector_script_body(port: u16, auth_file: Option<&Path>) -> String {
    let auth_line = match auth_file {
        Some(path) => format!(
            "$authHeader = $headers['authorization']; [System.IO.File]::WriteAllText('{}', $authHeader)",
            path.display().to_string().replace('\\', "\\\\")
        ),
        None => "$null = $headers".to_owned(),
    };

    format!(
        r#"
$listener = [System.Net.Sockets.TcpListener]::new([System.Net.IPAddress]::Parse("127.0.0.1"), {port})
$listener.Start()
while ($true) {{
    $client = $listener.AcceptTcpClient()
    $stream = $client.GetStream()
    $reader = New-Object System.IO.StreamReader($stream, [System.Text.Encoding]::ASCII, $false, 1024, $true)
    $requestLine = $reader.ReadLine()
    $headers = @{{}}
    $contentLength = 0
    while ($true) {{
        $line = $reader.ReadLine()
        if ([string]::IsNullOrEmpty($line)) {{
            break
        }}
        $parts = $line.Split(':', 2)
        if ($parts.Length -eq 2) {{
            $name = $parts[0].Trim().ToLowerInvariant()
            $value = $parts[1].Trim()
            $headers[$name] = $value
            if ($name -eq 'content-length') {{
                $contentLength = [int]$value
            }}
        }}
    }}
    if ($contentLength -gt 0) {{
        $buffer = New-Object char[] $contentLength
        [void]$reader.ReadBlock($buffer, 0, $contentLength)
    }}

    if ($requestLine.StartsWith('GET /health')) {{
        $body = '{{"status":"ok"}}'
    }} elseif ($requestLine.StartsWith('POST /v1/chat/completions')) {{
        {auth_line}
        $body = '{{"id":"chatcmpl_connector","object":"chat.completion","model":"gpt-4.1","choices":[]}}'
    }} else {{
        $body = '{{"error":"not_found"}}'
    }}

    $bodyBytes = [System.Text.Encoding]::UTF8.GetBytes($body)
    $writer = New-Object System.IO.StreamWriter($stream, [System.Text.Encoding]::ASCII, 1024, $true)
    $writer.NewLine = "`r`n"
    if ($requestLine.StartsWith('GET /health') -or $requestLine.StartsWith('POST /v1/chat/completions')) {{
        $writer.WriteLine('HTTP/1.1 200 OK')
        $writer.WriteLine('Content-Type: application/json')
    }} else {{
        $writer.WriteLine('HTTP/1.1 404 Not Found')
        $writer.WriteLine('Content-Type: application/json')
    }}
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
    path.push(format!("sdkwork-extension-host-{suffix}-{millis}"));
    path
}

fn cleanup_dir(path: &Path) {
    let _ = fs::remove_dir_all(path);
}

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}
