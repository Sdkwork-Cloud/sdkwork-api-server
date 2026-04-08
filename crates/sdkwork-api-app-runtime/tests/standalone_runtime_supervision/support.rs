use super::*;

fn write_runtime_config(root: &Path, enable_native_dynamic: bool, extension_root: &Path) {
    fs::write(
        root.join("config.yaml"),
        format!(
            r#"
extension_paths:
  - "{}"
enable_connector_extensions: false
enable_native_dynamic_extensions: {}
require_signed_native_dynamic_extensions: false
runtime_snapshot_interval_secs: 0
extension_hot_reload_interval_secs: 0
"#,
            config_path_value(extension_root),
            enable_native_dynamic,
        ),
    )
    .unwrap();
}

fn write_gateway_runtime_config(root: &Path, gateway_bind: &str) {
    write_gateway_runtime_config_with_cache(root, gateway_bind, CacheBackendKind::Memory, None);
}

fn write_gateway_store_runtime_config_with_cache(
    root: &Path,
    gateway_bind: &str,
    database_url: &str,
    cache_backend: CacheBackendKind,
    cache_url: Option<&str>,
) {
    let cache_url = cache_url
        .map(|value| format!("cache_url: \"{value}\"\n"))
        .unwrap_or_default();
    fs::write(
        root.join("config.yaml"),
        format!(
            r#"
gateway_bind: "{gateway_bind}"
database_url: "{database_url}"
cache_backend: "{}"
{cache_url}enable_connector_extensions: false
enable_native_dynamic_extensions: false
runtime_snapshot_interval_secs: 0
extension_hot_reload_interval_secs: 0
"#,
            cache_backend.as_str(),
        ),
    )
    .unwrap();
}

fn write_gateway_runtime_config_with_cache(
    root: &Path,
    gateway_bind: &str,
    cache_backend: CacheBackendKind,
    cache_url: Option<&str>,
) {
    write_gateway_store_runtime_config_with_cache(
        root,
        gateway_bind,
        "sqlite://sdkwork-api-server.db",
        cache_backend,
        cache_url,
    );
}

fn write_gateway_secret_manager_runtime_config(
    root: &Path,
    secret_local_file: &Path,
    credential_master_key: &str,
    credential_legacy_master_keys: &[&str],
) {
    let legacy_keys = if credential_legacy_master_keys.is_empty() {
        "credential_legacy_master_keys: []".to_owned()
    } else {
        format!(
            "credential_legacy_master_keys:\n{}",
            credential_legacy_master_keys
                .iter()
                .map(|value| format!("  - \"{value}\""))
                .collect::<Vec<_>>()
                .join("\n")
        )
    };
    fs::write(
        root.join("config.yaml"),
        format!(
            r#"
secret_backend: "local_encrypted_file"
secret_local_file: "{}"
credential_master_key: "{credential_master_key}"
{legacy_keys}
enable_connector_extensions: false
enable_native_dynamic_extensions: false
runtime_snapshot_interval_secs: 0
extension_hot_reload_interval_secs: 0
"#,
            config_path_value(secret_local_file),
        ),
    )
    .unwrap();
}

fn write_gateway_security_posture_runtime_config(
    root: &Path,
    gateway_bind: &str,
    admin_jwt_signing_secret: &str,
    portal_jwt_signing_secret: &str,
    credential_master_key: &str,
    allow_insecure_dev_defaults: bool,
) {
    fs::write(
        root.join("config.yaml"),
        format!(
            r#"
gateway_bind: "{gateway_bind}"
database_url: "sqlite://sdkwork-api-server.db"
admin_jwt_signing_secret: "{admin_jwt_signing_secret}"
portal_jwt_signing_secret: "{portal_jwt_signing_secret}"
credential_master_key: "{credential_master_key}"
allow_insecure_dev_defaults: {allow_insecure_dev_defaults}
enable_connector_extensions: false
enable_native_dynamic_extensions: false
runtime_snapshot_interval_secs: 0
extension_hot_reload_interval_secs: 0
"#,
        ),
    )
    .unwrap();
}

fn temp_root(suffix: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    path.push(format!("sdkwork-app-runtime-{suffix}-{millis}"));
    fs::create_dir_all(&path).unwrap();
    path
}

fn cleanup_dir(path: &Path) {
    let _ = fs::remove_dir_all(path);
}

fn unix_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("unix time")
        .as_millis() as u64
}

async fn seed_model_store(database_url: &str, model_id: &str) -> Arc<dyn AdminStore> {
    if let Some(path) = sqlite_path_from_url(database_url) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let _ = fs::File::create(path).unwrap();
    }
    let pool = run_migrations(database_url).await.unwrap();
    let store = SqliteAdminStore::new(pool);
    store
        .insert_channel(&Channel::new("openai", "OpenAI"))
        .await
        .unwrap();
    store
        .insert_provider(
            &ProxyProvider::new(
                "provider-openai-official",
                "openai",
                "openai",
                "https://api.openai.com/v1",
                "OpenAI Official",
            )
            .with_extension_id("sdkwork.provider.openai.official"),
        )
        .await
        .unwrap();
    store
        .insert_model(&ModelCatalogEntry::new(
            model_id,
            "provider-openai-official",
        ))
        .await
        .unwrap();
    Arc::new(store)
}

async fn empty_store() -> Arc<dyn AdminStore> {
    let pool = run_migrations("sqlite::memory:").await.unwrap();
    Arc::new(SqliteAdminStore::new(pool))
}

fn health_router(label: &'static str) -> Router {
    Router::new().route("/health", get(move || async move { label }))
}

fn http_client() -> Client {
    Client::builder()
        .timeout(StdDuration::from_millis(200))
        .build()
        .unwrap()
}

fn available_bind() -> String {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let bind = listener.local_addr().unwrap().to_string();
    drop(listener);
    bind
}

async fn wait_for_health_response(bind: &str, expected: &str) {
    let client = http_client();
    let url = format!("http://{bind}/health");
    for _ in 0..240 {
        if let Ok(response) = client.get(&url).send().await {
            if response.status().is_success()
                && response.text().await.unwrap_or_default() == expected
            {
                return;
            }
        }
        sleep(Duration::from_millis(25)).await;
    }

    panic!("listener did not respond with expected health payload: {url}");
}

async fn wait_for_health_unreachable(bind: &str) {
    let client = http_client();
    let url = format!("http://{bind}/health");
    for _ in 0..240 {
        if client.get(&url).send().await.is_err() {
            return;
        }
        sleep(Duration::from_millis(25)).await;
    }

    panic!("listener remained reachable unexpectedly: {url}");
}

async fn wait_for_lifecycle_log(path: &Path, expected: &[&str]) {
    for _ in 0..160 {
        if fs::read_to_string(path)
            .ok()
            .map(|contents| contents.lines().map(str::to_owned).collect::<Vec<_>>())
            .is_some_and(|lines| {
                lines
                    == expected
                        .iter()
                        .map(|line| (*line).to_owned())
                        .collect::<Vec<_>>()
            })
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

async fn wait_for_models(live_store: &Reloadable<Arc<dyn AdminStore>>, expected: &[&str]) {
    for _ in 0..200 {
        let current = live_store
            .snapshot()
            .list_models()
            .await
            .unwrap()
            .into_iter()
            .map(|entry| entry.external_name)
            .collect::<Vec<_>>();
        if current
            == expected
                .iter()
                .map(|value| (*value).to_owned())
                .collect::<Vec<_>>()
        {
            return;
        }
        sleep(Duration::from_millis(25)).await;
    }

    panic!("live store did not reach expected models");
}

async fn wait_for_reloadable_string(live_value: &Reloadable<String>, expected: &str) {
    for _ in 0..200 {
        if live_value.snapshot() == expected {
            return;
        }
        sleep(Duration::from_millis(25)).await;
    }

    panic!("reloadable string did not reach expected value");
}

async fn wait_for_pricing_plan_status(
    store: &SqliteAdminStore,
    pricing_plan_id: u64,
    expected_status: &str,
) {
    for _ in 0..200 {
        if store
            .list_pricing_plan_records()
            .await
            .unwrap()
            .into_iter()
            .find(|plan| plan.pricing_plan_id == pricing_plan_id)
            .is_some_and(|plan| plan.status == expected_status)
        {
            return;
        }
        sleep(Duration::from_millis(25)).await;
    }

    panic!("pricing plan did not reach expected status");
}

async fn wait_for_pricing_rate_status(
    store: &SqliteAdminStore,
    pricing_rate_id: u64,
    expected_status: &str,
) {
    for _ in 0..200 {
        if store
            .list_pricing_rate_records()
            .await
            .unwrap()
            .into_iter()
            .find(|rate| rate.pricing_rate_id == pricing_rate_id)
            .is_some_and(|rate| rate.status == expected_status)
        {
            return;
        }
        sleep(Duration::from_millis(25)).await;
    }

    panic!("pricing rate did not reach expected status");
}

async fn wait_for_secret_manager_master_key(
    live_secret_manager: &Reloadable<CredentialSecretManager>,
    expected_master_key: &str,
) {
    for _ in 0..200 {
        if live_secret_manager.snapshot().current_master_key_id()
            == sdkwork_api_secret_core::master_key_id(expected_master_key)
        {
            return;
        }
        sleep(Duration::from_millis(25)).await;
    }

    panic!("live secret manager did not reach expected master key");
}

async fn wait_for_extension_runtime_rollout_status(
    store: &dyn AdminStore,
    rollout_id: &str,
    expected_status: &str,
) {
    for _ in 0..200 {
        let Some(rollout) = find_extension_runtime_rollout(store, rollout_id)
            .await
            .unwrap()
        else {
            sleep(Duration::from_millis(25)).await;
            continue;
        };

        if rollout.status == expected_status {
            return;
        }
        sleep(Duration::from_millis(25)).await;
    }

    panic!("extension runtime rollout did not reach expected status");
}

async fn wait_for_standalone_config_rollout_status(
    store: &dyn AdminStore,
    rollout_id: &str,
    expected_status: &str,
) {
    for _ in 0..200 {
        let Some(rollout) = find_standalone_config_rollout(store, rollout_id)
            .await
            .unwrap()
        else {
            sleep(Duration::from_millis(25)).await;
            continue;
        };

        if rollout.status == expected_status {
            return;
        }
        sleep(Duration::from_millis(25)).await;
    }

    panic!("standalone config rollout did not reach expected status");
}

async fn wait_for_service_runtime_node(store: &dyn AdminStore, node_id: &str) {
    for _ in 0..200 {
        if store
            .list_service_runtime_nodes()
            .await
            .unwrap()
            .into_iter()
            .any(|node| node.node_id == node_id)
        {
            return;
        }

        sleep(Duration::from_millis(25)).await;
    }

    panic!("service runtime node did not heartbeat into the shared store: {node_id}");
}

struct NativeDynamicLifecycleLogGuard {
    path: PathBuf,
    previous: Option<String>,
}

impl NativeDynamicLifecycleLogGuard {
    fn new() -> Self {
        let mut path = std::env::temp_dir();
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("unix time")
            .as_millis();
        path.push(format!(
            "sdkwork-app-runtime-native-dynamic-lifecycle-{millis}.log"
        ));

        let previous = std::env::var("SDKWORK_NATIVE_MOCK_LIFECYCLE_LOG").ok();
        std::env::set_var("SDKWORK_NATIVE_MOCK_LIFECYCLE_LOG", &path);

        Self { path, previous }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for NativeDynamicLifecycleLogGuard {
    fn drop(&mut self) {
        match self.previous.as_deref() {
            Some(value) => std::env::set_var("SDKWORK_NATIVE_MOCK_LIFECYCLE_LOG", value),
            None => std::env::remove_var("SDKWORK_NATIVE_MOCK_LIFECYCLE_LOG"),
        }
        let _ = std::fs::remove_file(&self.path);
    }
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

fn write_portal_runtime_config(root: &Path, database_url: &str, jwt_secret: &str) {
    fs::write(
        root.join("config.yaml"),
        format!(
            r#"
database_url: "{database_url}"
portal_jwt_signing_secret: "{jwt_secret}"
runtime_snapshot_interval_secs: 0
extension_hot_reload_interval_secs: 0
"#,
        ),
    )
    .unwrap();
}

fn write_admin_pricing_runtime_config(
    root: &Path,
    database_url: &str,
    jwt_secret: &str,
    pricing_lifecycle_sync_interval_secs: u64,
) {
    fs::write(
        root.join("config.yaml"),
        format!(
            r#"
database_url: "{database_url}"
admin_jwt_signing_secret: "{jwt_secret}"
pricing_lifecycle_sync_interval_secs: {pricing_lifecycle_sync_interval_secs}
runtime_snapshot_interval_secs: 0
extension_hot_reload_interval_secs: 0
"#,
        ),
    )
    .unwrap();
}

fn sqlite_url_for_path(path: &Path) -> String {
    let normalized = path.to_string_lossy().replace('\\', "/");
    if normalized.starts_with('/') {
        format!("sqlite://{normalized}")
    } else {
        format!("sqlite:///{normalized}")
    }
}

fn sqlite_path_from_url(url: &str) -> Option<PathBuf> {
    let raw_path = url.strip_prefix("sqlite://")?;
    let normalized_path = raw_path
        .strip_prefix('/')
        .filter(|candidate| has_windows_drive_prefix(candidate))
        .unwrap_or(raw_path);

    Some(PathBuf::from(normalized_path))
}

fn config_path_value(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn has_windows_drive_prefix(path: &str) -> bool {
    let bytes = path.as_bytes();
    bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'/' || bytes[2] == b'\\')
}

fn native_dynamic_manifest(library_path: &Path) -> ExtensionManifest {
    ExtensionManifest::new(
        FIXTURE_EXTENSION_ID,
        ExtensionKind::Provider,
        "0.1.0",
        ExtensionRuntime::NativeDynamic,
    )
    .with_display_name("Native Mock")
    .with_protocol(ExtensionProtocol::OpenAi)
    .with_entrypoint(library_path.to_string_lossy())
    .with_supported_modality(sdkwork_api_extension_core::ExtensionModality::Audio)
    .with_supported_modality(sdkwork_api_extension_core::ExtensionModality::Video)
    .with_supported_modality(sdkwork_api_extension_core::ExtensionModality::File)
    .with_channel_binding("sdkwork.channel.openai")
    .with_permission(ExtensionPermission::NetworkOutbound)
    .with_capability(sdkwork_api_extension_core::CapabilityDescriptor::new(
        "chat.completions.create",
        CompatibilityLevel::Native,
    ))
    .with_capability(sdkwork_api_extension_core::CapabilityDescriptor::new(
        "chat.completions.stream",
        CompatibilityLevel::Native,
    ))
    .with_capability(sdkwork_api_extension_core::CapabilityDescriptor::new(
        "responses.create",
        CompatibilityLevel::Native,
    ))
    .with_capability(sdkwork_api_extension_core::CapabilityDescriptor::new(
        "responses.stream",
        CompatibilityLevel::Native,
    ))
    .with_capability(sdkwork_api_extension_core::CapabilityDescriptor::new(
        "audio.speech.create",
        CompatibilityLevel::Native,
    ))
    .with_capability(sdkwork_api_extension_core::CapabilityDescriptor::new(
        "files.content",
        CompatibilityLevel::Native,
    ))
    .with_capability(sdkwork_api_extension_core::CapabilityDescriptor::new(
        "videos.content",
        CompatibilityLevel::Native,
    ))
}
