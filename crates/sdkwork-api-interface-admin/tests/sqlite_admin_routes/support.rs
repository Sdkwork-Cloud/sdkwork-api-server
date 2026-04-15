pub(crate) use axum::body::{to_bytes, Body};
pub(crate) use axum::http::{Request, StatusCode};
pub(crate) use axum::Router;
pub(crate) use sdkwork_api_ext_provider_native_mock::FIXTURE_EXTENSION_ID;
pub(crate) use sdkwork_api_storage_core::ServiceRuntimeNodeRecord;
pub(crate) use serde_json::{json, Value};
pub(crate) use serial_test::serial;
pub(crate) use sqlx::SqlitePool;
pub(crate) use std::fs;
pub(crate) use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
pub(crate) use std::time::{SystemTime, UNIX_EPOCH};
pub(crate) use tower::ServiceExt;

pub(crate) use sdkwork_api_extension_core::{
    ExtensionInstallation, ExtensionInstance, ExtensionRuntime,
};
#[cfg(windows)]
pub(crate) use sdkwork_api_extension_host::{
    ensure_connector_runtime_started, shutdown_all_connector_runtimes, ExtensionLoadPlan,
};
pub(crate) use sdkwork_api_extension_host::{
    load_native_dynamic_provider_adapter, shutdown_all_native_dynamic_runtimes,
};
#[cfg(windows)]
pub(crate) use std::net::TcpListener;

static TEST_RESOURCE_SEQUENCE: AtomicU64 = AtomicU64::new(1);

pub(super) async fn read_json(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

pub(super) async fn memory_pool() -> SqlitePool {
    let pool = sdkwork_api_storage_sqlite::run_migrations("sqlite::memory:")
        .await
        .unwrap();
    let store = sdkwork_api_storage_sqlite::SqliteAdminStore::new(pool.clone());
    sdkwork_api_app_identity::upsert_admin_user(
        &store,
        Some("admin_local_default"),
        "admin@sdkwork.local",
        "Admin Operator",
        Some("ChangeMe123!"),
        Some(sdkwork_api_domain_identity::AdminUserRole::SuperAdmin),
        true,
    )
    .await
    .unwrap();
    pool
}

pub(super) async fn login_token(app: Router) -> String {
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"email\":\"admin@sdkwork.local\",\"password\":\"ChangeMe123!\"}",
                ))
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

pub(super) async fn create_provider_fixture(app: Router, token: &str, body: &str) {
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(body.to_owned()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}

pub(super) fn temp_extension_root(suffix: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let sequence = TEST_RESOURCE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    path.push(format!(
        "sdkwork-admin-routes-{suffix}-{millis}-{}-{sequence}",
        std::process::id()
    ));
    path
}

pub(super) fn cleanup_dir(path: &Path) {
    let _ = fs::remove_dir_all(path);
}

pub(super) fn unix_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("unix time")
        .as_millis() as u64
}

pub(super) fn extension_env_guard(path: &Path) -> ExtensionEnvGuard {
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

pub(super) struct ExtensionEnvGuard {
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

pub(super) fn restore_env_var(key: &str, value: Option<&str>) {
    match value {
        Some(value) => std::env::set_var(key, value),
        None => std::env::remove_var(key),
    }
}

pub(super) fn native_dynamic_extension_env_guard(path: &Path) -> NativeDynamicEnvGuard {
    let previous_paths = std::env::var("SDKWORK_EXTENSION_PATHS").ok();
    let previous_connector = std::env::var("SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS").ok();
    let previous_native = std::env::var("SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS").ok();
    let previous_native_signature =
        std::env::var("SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS").ok();
    let previous_trusted_signers = std::env::var("SDKWORK_EXTENSION_TRUSTED_SIGNERS").ok();

    let joined_paths = std::env::join_paths([path]).unwrap();
    std::env::set_var("SDKWORK_EXTENSION_PATHS", joined_paths);
    std::env::set_var("SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS", "false");
    std::env::set_var("SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS", "true");
    std::env::set_var(
        "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS",
        "false",
    );
    std::env::remove_var("SDKWORK_EXTENSION_TRUSTED_SIGNERS");

    NativeDynamicEnvGuard {
        previous_paths,
        previous_connector,
        previous_native,
        previous_native_signature,
        previous_trusted_signers,
    }
}

pub(super) struct NativeDynamicEnvGuard {
    previous_paths: Option<String>,
    previous_connector: Option<String>,
    previous_native: Option<String>,
    previous_native_signature: Option<String>,
    previous_trusted_signers: Option<String>,
}

impl Drop for NativeDynamicEnvGuard {
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
        restore_env_var(
            "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS",
            self.previous_native_signature.as_deref(),
        );
        restore_env_var(
            "SDKWORK_EXTENSION_TRUSTED_SIGNERS",
            self.previous_trusted_signers.as_deref(),
        );
    }
}

pub(super) fn native_dynamic_manifest(entrypoint: &Path) -> String {
    toml::to_string(
        &sdkwork_api_extension_core::ExtensionManifest::new(
            FIXTURE_EXTENSION_ID,
            sdkwork_api_extension_core::ExtensionKind::Provider,
            "0.1.0",
            sdkwork_api_extension_core::ExtensionRuntime::NativeDynamic,
        )
        .with_display_name("Native Mock")
        .with_protocol(sdkwork_api_extension_core::ExtensionProtocol::OpenAi)
        .with_entrypoint(entrypoint.to_string_lossy())
        .with_supported_modality(sdkwork_api_extension_core::ExtensionModality::Audio)
        .with_supported_modality(sdkwork_api_extension_core::ExtensionModality::Video)
        .with_supported_modality(sdkwork_api_extension_core::ExtensionModality::File)
        .with_channel_binding("sdkwork.channel.openai")
        .with_permission(sdkwork_api_extension_core::ExtensionPermission::NetworkOutbound)
        .with_capability(sdkwork_api_extension_core::CapabilityDescriptor::new(
            "chat.completions.create",
            sdkwork_api_extension_core::CompatibilityLevel::Native,
        ))
        .with_capability(sdkwork_api_extension_core::CapabilityDescriptor::new(
            "chat.completions.stream",
            sdkwork_api_extension_core::CompatibilityLevel::Native,
        ))
        .with_capability(sdkwork_api_extension_core::CapabilityDescriptor::new(
            "responses.create",
            sdkwork_api_extension_core::CompatibilityLevel::Native,
        ))
        .with_capability(sdkwork_api_extension_core::CapabilityDescriptor::new(
            "responses.stream",
            sdkwork_api_extension_core::CompatibilityLevel::Native,
        ))
        .with_capability(sdkwork_api_extension_core::CapabilityDescriptor::new(
            "anthropic.messages.create",
            sdkwork_api_extension_core::CompatibilityLevel::Native,
        ))
        .with_capability(sdkwork_api_extension_core::CapabilityDescriptor::new(
            "anthropic.messages.count_tokens",
            sdkwork_api_extension_core::CompatibilityLevel::Native,
        ))
        .with_capability(sdkwork_api_extension_core::CapabilityDescriptor::new(
            "gemini.generate_content",
            sdkwork_api_extension_core::CompatibilityLevel::Native,
        ))
        .with_capability(sdkwork_api_extension_core::CapabilityDescriptor::new(
            "gemini.stream_generate_content",
            sdkwork_api_extension_core::CompatibilityLevel::Native,
        ))
        .with_capability(sdkwork_api_extension_core::CapabilityDescriptor::new(
            "gemini.count_tokens",
            sdkwork_api_extension_core::CompatibilityLevel::Native,
        ))
        .with_capability(sdkwork_api_extension_core::CapabilityDescriptor::new(
            "audio.speech.create",
            sdkwork_api_extension_core::CompatibilityLevel::Native,
        ))
        .with_capability(sdkwork_api_extension_core::CapabilityDescriptor::new(
            "files.content",
            sdkwork_api_extension_core::CompatibilityLevel::Native,
        ))
        .with_capability(sdkwork_api_extension_core::CapabilityDescriptor::new(
            "videos.content",
            sdkwork_api_extension_core::CompatibilityLevel::Native,
        )),
    )
    .expect("native dynamic manifest toml")
}

pub(super) struct NativeDynamicLifecycleLogGuard {
    path: PathBuf,
    previous: Option<String>,
}

impl NativeDynamicLifecycleLogGuard {
    pub(super) fn new() -> Self {
        let mut path = std::env::temp_dir();
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("unix time")
            .as_millis();
        let sequence = TEST_RESOURCE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        path.push(format!(
            "sdkwork-admin-native-dynamic-lifecycle-{millis}-{}-{sequence}.log",
            std::process::id()
        ));

        let previous = std::env::var("SDKWORK_NATIVE_MOCK_LIFECYCLE_LOG").ok();
        std::env::set_var("SDKWORK_NATIVE_MOCK_LIFECYCLE_LOG", &path);

        Self { path, previous }
    }

    pub(super) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for NativeDynamicLifecycleLogGuard {
    fn drop(&mut self) {
        restore_env_var(
            "SDKWORK_NATIVE_MOCK_LIFECYCLE_LOG",
            self.previous.as_deref(),
        );
        let _ = std::fs::remove_file(&self.path);
    }
}

#[cfg(windows)]
pub(super) fn connector_script_body(port: u16) -> String {
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
