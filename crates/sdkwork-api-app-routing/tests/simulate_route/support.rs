use super::*;
use sdkwork_api_app_catalog::{
    create_provider_with_config, create_provider_with_default_plugin_family_and_bindings,
};

pub(super) const PROVIDER_HEALTH_TTL_ENV: &str = "SDKWORK_ROUTING_PROVIDER_HEALTH_FRESHNESS_TTL_MS";
pub(super) const PROVIDER_HEALTH_RECOVERY_PROBE_PERCENT_ENV: &str =
    "SDKWORK_ROUTING_PROVIDER_HEALTH_RECOVERY_PROBE_PERCENT";
pub(super) const PROVIDER_HEALTH_RECOVERY_PROBE_LOCK_TTL_ENV: &str =
    "SDKWORK_ROUTING_PROVIDER_HEALTH_RECOVERY_PROBE_LOCK_TTL_MS";

pub(super) async fn create_store_with_openai_channel() -> SqliteAdminStore {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    let store = SqliteAdminStore::new(pool);
    store
        .insert_channel(&Channel::new("openai", "OpenAI"))
        .await
        .unwrap();
    store
        .insert_channel(&Channel::new("openrouter", "OpenRouter"))
        .await
        .unwrap();
    store
}

pub(super) async fn insert_openai_provider(
    store: &SqliteAdminStore,
    provider_id: &str,
    base_url: &str,
    display_name: &str,
) {
    store
        .insert_provider(
            &create_provider_with_config(
                provider_id,
                "openai",
                "openai",
                base_url,
                display_name,
            )
            .unwrap(),
        )
        .await
        .unwrap();
}

pub(super) async fn insert_openrouter_provider(
    store: &SqliteAdminStore,
    provider_id: &str,
    base_url: &str,
    display_name: &str,
) {
    store
        .insert_provider(
            &create_provider_with_default_plugin_family_and_bindings(
                provider_id,
                "openrouter",
                "openrouter",
                base_url,
                display_name,
                &[ProviderChannelBinding::new(provider_id, "openai")],
            )
            .unwrap(),
        )
        .await
        .unwrap();
}

pub(super) fn observed_at_now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

pub(super) struct ScopedEnvVar {
    key: &'static str,
    previous: Option<String>,
}

impl ScopedEnvVar {
    pub(super) fn set(key: &'static str, value: &str) -> Self {
        let previous = std::env::var(key).ok();
        std::env::set_var(key, value);
        Self { key, previous }
    }
}

impl Drop for ScopedEnvVar {
    fn drop(&mut self) {
        if let Some(previous) = self.previous.as_deref() {
            std::env::set_var(self.key, previous);
        } else {
            std::env::remove_var(self.key);
        }
    }
}

pub(super) fn temp_extension_root(suffix: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    path.push(format!("sdkwork-app-routing-{suffix}-{millis}"));
    path
}

#[cfg(windows)]
pub(super) fn cleanup_dir(path: &Path) {
    let _ = fs::remove_dir_all(path);
}

#[cfg(windows)]
pub(super) fn unstable_connector_script_body(port: u16, degrade_file: &Path) -> String {
    format!(
        r#"
$degradeFile = '{}'
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
        if (Test-Path $degradeFile) {{
            $status = 'HTTP/1.1 503 Service Unavailable'
            $body = '{{"status":"degraded"}}'
        }} else {{
            $status = 'HTTP/1.1 200 OK'
            $body = '{{"status":"ok"}}'
        }}
    }} else {{
        $status = 'HTTP/1.1 404 Not Found'
        $body = '{{"error":"not_found"}}'
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
"#,
        degrade_file.display().to_string().replace('\\', "\\\\")
    )
}

#[cfg(windows)]
pub(super) fn native_dynamic_fixture_library_path() -> PathBuf {
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
pub(super) fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}
