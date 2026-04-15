use super::*;

#[derive(Clone, Default)]
pub(super) struct UpstreamCaptureState {
    pub(super) authorization: Arc<Mutex<Option<String>>>,
}

pub(super) fn chat_request(model: &str) -> CreateChatCompletionRequest {
    CreateChatCompletionRequest {
        model: model.to_owned(),
        messages: vec![ChatMessageInput {
            role: "user".to_owned(),
            content: Value::String("hello".to_owned()),
            extra: serde_json::Map::new(),
        }],
        stream: None,
        extra: serde_json::Map::new(),
    }
}

pub(super) async fn upstream_chat_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> Json<Value> {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    Json(json!({
        "id": "chatcmpl_upstream",
        "object": "chat.completion",
        "model": "gpt-4.1",
        "choices": []
    }))
}

pub(super) async fn upstream_health_handler() -> Json<Value> {
    Json(json!({
        "status": "ok"
    }))
}

pub(super) fn serve_connector_compatible_upstream(
    listener: std::net::TcpListener,
    state: UpstreamCaptureState,
    expected_requests: usize,
) {
    listener.set_nonblocking(true).unwrap();
    let mut handled_requests = 0_usize;
    let mut idle_since = None;
    loop {
        let (mut stream, _) = match listener.accept() {
            Ok(connection) => {
                idle_since = None;
                connection
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                if handled_requests >= expected_requests {
                    let idle_for = idle_since.get_or_insert_with(std::time::Instant::now);
                    if idle_for.elapsed() >= Duration::from_millis(200) {
                        break;
                    }
                }
                thread::sleep(Duration::from_millis(10));
                continue;
            }
            Err(error) => panic!("connector-compatible upstream accept failed: {error}"),
        };
        stream.set_nonblocking(false).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(1)))
            .unwrap();
        handled_requests += 1;
        let mut buffer = [0_u8; 4096];
        let bytes_read = stream.read(&mut buffer).unwrap();
        let request = String::from_utf8_lossy(&buffer[..bytes_read]);
        let request_line = request.lines().next().unwrap_or_default();
        let authorization = request
            .lines()
            .find_map(|line| {
                line.strip_prefix("authorization: ")
                    .or_else(|| line.strip_prefix("Authorization: "))
            })
            .map(str::trim)
            .map(ToOwned::to_owned);

        let (status, body) = if request_line.starts_with("GET /health") {
            ("HTTP/1.1 200 OK", r#"{"status":"ok"}"#.to_owned())
        } else if request_line.starts_with("POST /v1/chat/completions") {
            *state.authorization.lock().unwrap() = authorization;
            (
                "HTTP/1.1 200 OK",
                r#"{"id":"chatcmpl_upstream","object":"chat.completion","model":"gpt-4.1","choices":[]}"#
                    .to_owned(),
            )
        } else {
            (
                "HTTP/1.1 404 Not Found",
                r#"{"error":"not_found"}"#.to_owned(),
            )
        };

        let response = format!(
            "{status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }
}

pub(super) fn discovered_connector_manifest() -> &'static str {
    r#"
api_version = "sdkwork.extension/v1"
id = "sdkwork.provider.custom-openai"
kind = "provider"
version = "0.1.0"
display_name = "Custom OpenAI"
runtime = "connector"
protocol = "openai"
entrypoint = "bin/sdkwork-provider-custom-openai"
channel_bindings = ["sdkwork.channel.openai"]
permissions = ["network_outbound", "spawn_process"]

[health]
path = "/health"
interval_secs = 30

[[capabilities]]
operation = "chat.completions.create"
compatibility = "relay"
"#
}

pub(super) fn temp_extension_root(suffix: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    path.push(format!("sdkwork-app-gateway-{suffix}-{millis}"));
    path
}

pub(super) fn cleanup_dir(path: &Path) {
    let _ = fs::remove_dir_all(path);
}

pub(super) async fn wait_for_health(base_url: &str) {
    let health_url = format!("{}/health", base_url.trim_end_matches('/'));
    for _ in 0..20 {
        if let Ok(response) = reqwest::get(&health_url).await {
            if response.status().is_success() {
                return;
            }
        }
        sleep(Duration::from_millis(25)).await;
    }

    panic!("health endpoint did not become ready: {health_url}");
}

pub(super) async fn wait_for_lifecycle_log(path: &Path, expected: &[&str]) {
    for _ in 0..120 {
        if read_log_lines(path)
            == expected
                .iter()
                .map(|line| (*line).to_owned())
                .collect::<Vec<_>>()
        {
            return;
        }
        sleep(Duration::from_millis(25)).await;
    }

    panic!(
        "lifecycle log did not reach expected state: {}",
        path.display()
    );
}

pub(super) async fn wait_for_log_line(path: &Path, expected: &str) {
    for _ in 0..120 {
        if read_log_lines(path).iter().any(|line| line == expected) {
            return;
        }
        sleep(Duration::from_millis(25)).await;
    }

    panic!("log did not contain {expected}: {}", path.display());
}

pub(super) fn read_log_lines(path: &Path) -> Vec<String> {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .map(ToOwned::to_owned)
        .collect()
}

pub(super) fn extension_env_guard(path: &Path) -> ExtensionEnvGuard {
    extension_env_guard_with_signature_requirement(path, false)
}

pub(super) fn extension_env_guard_with_signature_requirement(
    path: &Path,
    require_connector_signature: bool,
) -> ExtensionEnvGuard {
    let previous_paths = std::env::var("SDKWORK_EXTENSION_PATHS").ok();
    let previous_connector = std::env::var("SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS").ok();
    let previous_native = std::env::var("SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS").ok();
    let previous_connector_signature =
        std::env::var("SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS").ok();

    let joined_paths = std::env::join_paths([path]).unwrap();
    std::env::set_var("SDKWORK_EXTENSION_PATHS", joined_paths);
    std::env::set_var("SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS", "true");
    std::env::set_var(
        "SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS",
        "false",
    );
    std::env::set_var(
        "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS",
        if require_connector_signature {
            "true"
        } else {
            "false"
        },
    );

    ExtensionEnvGuard {
        previous_paths,
        previous_connector,
        previous_native,
        previous_connector_signature,
        previous_native_signature: None,
        previous_trusted_signers: None,
    }
}

pub(super) struct ExtensionEnvGuard {
    previous_paths: Option<String>,
    previous_connector: Option<String>,
    previous_native: Option<String>,
    previous_connector_signature: Option<String>,
    previous_native_signature: Option<String>,
    previous_trusted_signers: Option<String>,
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
        restore_env_var(
            "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS",
            self.previous_connector_signature.as_deref(),
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

pub(super) fn restore_env_var(key: &str, value: Option<&str>) {
    match value {
        Some(value) => std::env::set_var(key, value),
        None => std::env::remove_var(key),
    }
}

pub(super) fn native_dynamic_env_guard(path: &Path, public_key: &str) -> ExtensionEnvGuard {
    let previous_paths = std::env::var("SDKWORK_EXTENSION_PATHS").ok();
    let previous_connector = std::env::var("SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS").ok();
    let previous_native = std::env::var("SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS").ok();
    let previous_connector_signature =
        std::env::var("SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS").ok();
    let previous_native_signature =
        std::env::var("SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS").ok();
    let previous_trusted_signers = std::env::var("SDKWORK_EXTENSION_TRUSTED_SIGNERS").ok();

    let joined_paths = std::env::join_paths([path]).unwrap();
    std::env::set_var("SDKWORK_EXTENSION_PATHS", joined_paths);
    std::env::set_var("SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS", "false");
    std::env::set_var("SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS", "true");
    std::env::set_var(
        "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS",
        "false",
    );
    std::env::set_var(
        "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS",
        "true",
    );
    std::env::set_var(
        "SDKWORK_EXTENSION_TRUSTED_SIGNERS",
        format!("sdkwork={public_key}"),
    );

    ExtensionEnvGuard {
        previous_paths,
        previous_connector,
        previous_native,
        previous_connector_signature,
        previous_native_signature,
        previous_trusted_signers,
    }
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

pub(super) fn native_dynamic_manifest(
    library_path: &Path,
) -> sdkwork_api_extension_core::ExtensionManifest {
    sdkwork_api_extension_core::ExtensionManifest::new(
        FIXTURE_EXTENSION_ID,
        sdkwork_api_extension_core::ExtensionKind::Provider,
        "0.1.0",
        ExtensionRuntime::NativeDynamic,
    )
    .with_display_name("Native Mock")
    .with_protocol(sdkwork_api_extension_core::ExtensionProtocol::OpenAi)
    .with_entrypoint(library_path.to_string_lossy())
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
    ))
}

pub(super) fn sign_native_dynamic_package(
    package_dir: &Path,
    manifest: &sdkwork_api_extension_core::ExtensionManifest,
    signing_key: &SigningKey,
) -> String {
    use ed25519_dalek::Signer;

    #[derive(serde::Serialize)]
    struct PackageSignaturePayload<'a> {
        manifest: &'a sdkwork_api_extension_core::ExtensionManifest,
        files: Vec<PackageFileDigest>,
    }

    #[derive(serde::Serialize)]
    struct PackageFileDigest {
        path: String,
        sha256: String,
    }

    let files = std::fs::read_dir(package_dir)
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name().and_then(|value| value.to_str()) != Some("sdkwork-extension.toml")
        })
        .map(|path| PackageFileDigest {
            path: path
                .strip_prefix(package_dir)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/"),
            sha256: sha256_hex_path(&path),
        })
        .collect::<Vec<_>>();

    let payload = serde_json::to_vec(&PackageSignaturePayload { manifest, files }).unwrap();
    let signature = signing_key.sign(&payload);
    STANDARD.encode(signature.to_bytes())
}

pub(super) fn sha256_hex_path(path: &Path) -> String {
    let digest = Sha256::digest(std::fs::read(path).unwrap());
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        encoded.push_str(&format!("{byte:02x}"));
    }
    encoded
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
        path.push(format!(
            "sdkwork-app-gateway-native-dynamic-lifecycle-{millis}.log"
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

pub(super) struct NativeDynamicInvocationLogGuard {
    path: PathBuf,
    previous: Option<String>,
}

impl NativeDynamicInvocationLogGuard {
    pub(super) fn new() -> Self {
        let mut path = std::env::temp_dir();
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("unix time")
            .as_millis();
        path.push(format!(
            "sdkwork-app-gateway-native-dynamic-invocation-{millis}.log"
        ));

        let previous = std::env::var("SDKWORK_NATIVE_MOCK_INVOCATION_LOG").ok();
        std::env::set_var("SDKWORK_NATIVE_MOCK_INVOCATION_LOG", &path);

        Self { path, previous }
    }

    pub(super) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for NativeDynamicInvocationLogGuard {
    fn drop(&mut self) {
        restore_env_var(
            "SDKWORK_NATIVE_MOCK_INVOCATION_LOG",
            self.previous.as_deref(),
        );
        let _ = std::fs::remove_file(&self.path);
    }
}

pub(super) struct NativeDynamicMockDelayGuard {
    previous_json_delay_ms: Option<String>,
    previous_stream_delay_ms: Option<String>,
}

impl NativeDynamicMockDelayGuard {
    pub(super) fn json(delay_ms: u64) -> Self {
        let previous_json_delay_ms = std::env::var("SDKWORK_NATIVE_MOCK_JSON_DELAY_MS").ok();
        let previous_stream_delay_ms = std::env::var("SDKWORK_NATIVE_MOCK_STREAM_DELAY_MS").ok();
        std::env::set_var("SDKWORK_NATIVE_MOCK_JSON_DELAY_MS", delay_ms.to_string());
        std::env::remove_var("SDKWORK_NATIVE_MOCK_STREAM_DELAY_MS");
        Self {
            previous_json_delay_ms,
            previous_stream_delay_ms,
        }
    }
}

impl Drop for NativeDynamicMockDelayGuard {
    fn drop(&mut self) {
        restore_env_var(
            "SDKWORK_NATIVE_MOCK_JSON_DELAY_MS",
            self.previous_json_delay_ms.as_deref(),
        );
        restore_env_var(
            "SDKWORK_NATIVE_MOCK_STREAM_DELAY_MS",
            self.previous_stream_delay_ms.as_deref(),
        );
    }
}

pub(super) struct NativeDynamicDrainTimeoutGuard {
    previous_timeout_ms: Option<String>,
}

impl NativeDynamicDrainTimeoutGuard {
    pub(super) fn new(timeout_ms: u64) -> Self {
        let previous_timeout_ms =
            std::env::var("SDKWORK_NATIVE_DYNAMIC_SHUTDOWN_DRAIN_TIMEOUT_MS").ok();
        std::env::set_var(
            "SDKWORK_NATIVE_DYNAMIC_SHUTDOWN_DRAIN_TIMEOUT_MS",
            timeout_ms.to_string(),
        );
        Self {
            previous_timeout_ms,
        }
    }
}

impl Drop for NativeDynamicDrainTimeoutGuard {
    fn drop(&mut self) {
        restore_env_var(
            "SDKWORK_NATIVE_DYNAMIC_SHUTDOWN_DRAIN_TIMEOUT_MS",
            self.previous_timeout_ms.as_deref(),
        );
    }
}
