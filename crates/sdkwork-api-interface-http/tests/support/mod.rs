use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use ed25519_dalek::SigningKey;
use sdkwork_api_app_identity::{persist_gateway_api_key, upsert_admin_user};
use sdkwork_api_ext_provider_native_mock::FIXTURE_EXTENSION_ID;
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use serde_json::Value;
use sqlx::SqlitePool;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::{SystemTime, UNIX_EPOCH};
use tower::ServiceExt;

#[allow(dead_code)]
pub async fn issue_gateway_api_key(pool: &SqlitePool, tenant_id: &str, project_id: &str) -> String {
    let store = SqliteAdminStore::new(pool.clone());
    persist_gateway_api_key(&store, tenant_id, project_id, "live")
        .await
        .unwrap()
        .plaintext
}

#[allow(dead_code)]
pub async fn issue_admin_token(pool: &SqlitePool, app: Router) -> String {
    let store = SqliteAdminStore::new(pool.clone());
    upsert_admin_user(
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
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    json["token"].as_str().unwrap().to_owned()
}

#[allow(dead_code)]
pub fn sha256_hex_path(path: &Path) -> String {
    use sha2::{Digest, Sha256};

    let digest = Sha256::digest(std::fs::read(path).unwrap());
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        encoded.push_str(&format!("{byte:02x}"));
    }
    encoded
}

#[allow(dead_code)]
pub async fn assert_single_usage_record_and_decision_log(
    admin_app: Router,
    admin_token: &str,
    expected_model: &str,
    expected_provider: &str,
    expected_route_key: &str,
) {
    let usage = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/usage/records")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(usage.status(), StatusCode::OK);
    let usage_json = read_json(usage).await;
    assert_eq!(usage_json.as_array().unwrap().len(), 1);
    assert_eq!(usage_json[0]["model"], expected_model);
    assert_eq!(usage_json[0]["provider"], expected_provider);

    let logs = admin_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/routing/decision-logs")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(logs.status(), StatusCode::OK);
    let logs_json = read_json(logs).await;
    assert_eq!(logs_json.as_array().unwrap().len(), 1);
    assert_eq!(logs_json[0]["route_key"], expected_route_key);
    assert_eq!(logs_json[0]["selected_provider_id"], expected_provider);
}

#[allow(dead_code)]
pub async fn assert_no_usage_records(admin_app: Router, admin_token: &str) {
    let usage = admin_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/usage/records")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(usage.status(), StatusCode::OK);
    let usage_json = read_json(usage).await;
    assert_eq!(usage_json.as_array().unwrap().len(), 0);
}

#[allow(dead_code)]
pub async fn assert_single_decision_log(
    admin_app: Router,
    admin_token: &str,
    expected_route_key: &str,
    expected_provider: &str,
) {
    let logs = admin_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/routing/decision-logs")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(logs.status(), StatusCode::OK);
    let logs_json = read_json(logs).await;
    assert_eq!(logs_json.as_array().unwrap().len(), 1);
    assert_eq!(logs_json[0]["route_key"], expected_route_key);
    assert_eq!(logs_json[0]["selected_provider_id"], expected_provider);
}

#[allow(dead_code)]
pub struct NativeDynamicMockPackage {
    pub extension_root: PathBuf,
    pub library_path: PathBuf,
    _guard: ExtensionEnvGuard,
}

impl Drop for NativeDynamicMockPackage {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.extension_root);
    }
}

#[allow(dead_code)]
pub fn prepare_native_dynamic_mock_package(suffix: &str) -> NativeDynamicMockPackage {
    let extension_root = temp_extension_root(suffix);
    let package_dir = extension_root.join("sdkwork-provider-native-mock");
    fs::create_dir_all(&package_dir).unwrap();

    let signing_key = SigningKey::from_bytes(&[7_u8; 32]);
    let public_key = STANDARD.encode(signing_key.verifying_key().to_bytes());
    let library_path = native_dynamic_fixture_library_path();
    let manifest = native_dynamic_manifest(&library_path);
    let signature = sign_native_dynamic_package(&package_dir, &manifest, &signing_key);
    let manifest = manifest.with_trust(
        sdkwork_api_extension_core::ExtensionTrustDeclaration::signed(
            "sdkwork",
            sdkwork_api_extension_core::ExtensionSignature::new(
                sdkwork_api_extension_core::ExtensionSignatureAlgorithm::Ed25519,
                public_key.clone(),
                signature,
            ),
        ),
    );
    fs::write(
        package_dir.join("sdkwork-extension.toml"),
        toml::to_string(&manifest).unwrap(),
    )
    .unwrap();
    let guard = native_dynamic_env_guard(&extension_root, &public_key);

    NativeDynamicMockPackage {
        extension_root,
        library_path,
        _guard: guard,
    }
}

#[allow(dead_code)]
pub struct ConnectorMockPackage {
    pub extension_root: PathBuf,
    pub extension_id: String,
    _guard: ExtensionEnvGuard,
}

impl Drop for ConnectorMockPackage {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.extension_root);
    }
}

#[allow(dead_code)]
pub fn prepare_connector_mock_package(suffix: &str, extension_id: &str) -> ConnectorMockPackage {
    let extension_root = temp_extension_root(suffix);
    let package_dir = extension_root.join("sdkwork-provider-custom-openai");
    fs::create_dir_all(&package_dir).unwrap();
    fs::write(
        package_dir.join("sdkwork-extension.toml"),
        connector_manifest(extension_id),
    )
    .unwrap();
    let guard = connector_env_guard(&extension_root);

    ConnectorMockPackage {
        extension_root,
        extension_id: extension_id.to_owned(),
        _guard: guard,
    }
}

#[allow(dead_code)]
pub async fn wait_for_http_health(base_url: &str) {
    let health_url = format!("{}/health", base_url.trim_end_matches('/'));
    for _ in 0..40 {
        if let Ok(response) = reqwest::get(&health_url).await {
            if response.status().is_success() {
                return;
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }

    panic!("health endpoint did not become ready: {health_url}");
}

fn temp_extension_root(suffix: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    path.push(format!("sdkwork-interface-http-{suffix}-{millis}"));
    path
}

fn native_dynamic_env_guard(path: &Path, public_key: &str) -> ExtensionEnvGuard {
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

fn connector_env_guard(path: &Path) -> ExtensionEnvGuard {
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
    std::env::set_var("SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS", "true");
    std::env::set_var(
        "SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS",
        "false",
    );
    std::env::set_var(
        "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS",
        "false",
    );
    std::env::set_var(
        "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS",
        "false",
    );
    std::env::remove_var("SDKWORK_EXTENSION_TRUSTED_SIGNERS");

    ExtensionEnvGuard {
        previous_paths,
        previous_connector,
        previous_native,
        previous_connector_signature,
        previous_native_signature,
        previous_trusted_signers,
    }
}

struct ExtensionEnvGuard {
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

fn restore_env_var(key: &str, value: Option<&str>) {
    match value {
        Some(value) => std::env::set_var(key, value),
        None => std::env::remove_var(key),
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

fn native_dynamic_manifest(library_path: &Path) -> sdkwork_api_extension_core::ExtensionManifest {
    sdkwork_api_extension_core::ExtensionManifest::new(
        FIXTURE_EXTENSION_ID,
        sdkwork_api_extension_core::ExtensionKind::Provider,
        "0.1.0",
        sdkwork_api_extension_core::ExtensionRuntime::NativeDynamic,
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

fn connector_manifest(extension_id: &str) -> String {
    format!(
        r#"
api_version = "sdkwork.extension/v1"
id = "{extension_id}"
kind = "provider"
version = "0.1.0"
display_name = "Custom OpenAI Connector"
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

[[capabilities]]
operation = "chat.completions.stream"
compatibility = "relay"
"#
    )
}

fn sign_native_dynamic_package(
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

async fn read_json(response: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[allow(dead_code)]
#[derive(Default)]
struct FakeRedisState {
    databases: HashMap<u32, FakeRedisDatabase>,
}

#[allow(dead_code)]
#[derive(Default)]
struct FakeRedisDatabase {
    strings: HashMap<Vec<u8>, FakeRedisStringValue>,
    sets: HashMap<Vec<u8>, HashSet<Vec<u8>>>,
}

#[allow(dead_code)]
struct FakeRedisStringValue {
    value: Vec<u8>,
    expires_at: Option<std::time::Instant>,
}

#[allow(dead_code)]
pub struct FakeRedisServer {
    address: String,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

#[allow(dead_code)]
impl FakeRedisServer {
    pub fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind fake redis listener");
        listener
            .set_nonblocking(true)
            .expect("set nonblocking listener");
        let address = listener.local_addr().unwrap().to_string();
        let stop = Arc::new(AtomicBool::new(false));
        let state = Arc::new(Mutex::new(FakeRedisState::default()));
        let thread_stop = stop.clone();
        let thread = thread::spawn(move || {
            while !thread_stop.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        stream.set_nonblocking(false).expect("set blocking stream");
                        handle_fake_redis_connection(stream, state.clone());
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(std::time::Duration::from_millis(10));
                    }
                    Err(error) => panic!("fake redis accept failed: {error}"),
                }
            }
        });

        Self {
            address,
            stop,
            thread: Some(thread),
        }
    }

    pub fn url_with_db(&self, db: u32) -> String {
        format!("redis://{}/{db}", self.address)
    }
}

impl Drop for FakeRedisServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        let _ = TcpStream::connect(&self.address);
        if let Some(thread) = self.thread.take() {
            thread.join().expect("join fake redis thread");
        }
    }
}

#[allow(dead_code)]
fn handle_fake_redis_connection(stream: TcpStream, state: Arc<Mutex<FakeRedisState>>) {
    let mut stream = stream;
    stream
        .set_read_timeout(Some(std::time::Duration::from_millis(250)))
        .expect("set read timeout");
    stream
        .set_write_timeout(Some(std::time::Duration::from_millis(250)))
        .expect("set write timeout");
    let mut selected_db = 0_u32;

    loop {
        let command = match read_resp_array(&mut stream) {
            Ok(Some(command)) => command,
            Ok(None) => break,
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::UnexpectedEof
                        | std::io::ErrorKind::ConnectionReset
                        | std::io::ErrorKind::TimedOut
                ) =>
            {
                break;
            }
            Err(error) => panic!("fake redis command read failed: {error}"),
        };
        let response = execute_fake_redis_command(&state, &mut selected_db, &command);
        write_resp_value(&mut stream, response).expect("write fake redis response");
    }
}

#[allow(dead_code)]
fn execute_fake_redis_command(
    state: &Arc<Mutex<FakeRedisState>>,
    selected_db: &mut u32,
    command: &[Vec<u8>],
) -> RespValue {
    let name = String::from_utf8_lossy(&command[0]).to_ascii_uppercase();
    let mut state = state.lock().expect("fake redis state");
    let database = state.databases.entry(*selected_db).or_default();
    purge_expired_strings(database);

    match name.as_str() {
        "PING" => RespValue::Simple("PONG".to_owned()),
        "AUTH" => RespValue::Simple("OK".to_owned()),
        "SELECT" => {
            *selected_db = String::from_utf8_lossy(&command[1]).parse().unwrap();
            RespValue::Simple("OK".to_owned())
        }
        "GET" => {
            let key = &command[1];
            RespValue::Bulk(database.strings.get(key).map(|value| value.value.clone()))
        }
        "SET" => {
            let key = command[1].clone();
            let value = command[2].clone();
            let mut ttl_ms = None;
            let mut nx = false;
            let mut index = 3;
            while index < command.len() {
                match String::from_utf8_lossy(&command[index])
                    .to_ascii_uppercase()
                    .as_str()
                {
                    "PX" => {
                        ttl_ms = Some(
                            String::from_utf8_lossy(&command[index + 1])
                                .parse::<u64>()
                                .unwrap(),
                        );
                        index += 2;
                    }
                    "NX" => {
                        nx = true;
                        index += 1;
                    }
                    other => panic!("unsupported fake redis SET option: {other}"),
                }
            }

            if nx && database.strings.contains_key(&key) {
                return RespValue::Bulk(None);
            }

            database.strings.insert(
                key,
                FakeRedisStringValue {
                    value,
                    expires_at: ttl_ms.map(|ttl_ms| {
                        std::time::Instant::now() + std::time::Duration::from_millis(ttl_ms)
                    }),
                },
            );
            RespValue::Simple("OK".to_owned())
        }
        "DEL" => {
            let mut removed = 0_i64;
            for key in &command[1..] {
                if database.strings.remove(key).is_some() {
                    removed += 1;
                }
                if database.sets.remove(key).is_some() {
                    removed += 1;
                }
            }
            RespValue::Integer(removed)
        }
        "SADD" => {
            let members = database.sets.entry(command[1].clone()).or_default();
            let mut added = 0_i64;
            for member in &command[2..] {
                if members.insert(member.clone()) {
                    added += 1;
                }
            }
            RespValue::Integer(added)
        }
        "SMEMBERS" => RespValue::Array(
            database
                .sets
                .get(&command[1])
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .collect(),
        ),
        "SREM" => {
            let mut removed = 0_i64;
            if let Some(members) = database.sets.get_mut(&command[1]) {
                for member in &command[2..] {
                    if members.remove(member) {
                        removed += 1;
                    }
                }
                if members.is_empty() {
                    database.sets.remove(&command[1]);
                }
            }
            RespValue::Integer(removed)
        }
        other => panic!("unsupported fake redis command: {other}"),
    }
}

#[allow(dead_code)]
fn purge_expired_strings(database: &mut FakeRedisDatabase) {
    let now = std::time::Instant::now();
    database.strings.retain(|_, value| {
        value
            .expires_at
            .map(|expires_at| expires_at > now)
            .unwrap_or(true)
    });
}

#[allow(dead_code)]
fn read_resp_array(stream: &mut TcpStream) -> std::io::Result<Option<Vec<Vec<u8>>>> {
    let mut marker = [0_u8; 1];
    match stream.read_exact(&mut marker) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(error) => return Err(error),
    }
    if marker[0] != b'*' {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "expected RESP array",
        ));
    }
    let count = read_resp_line(stream)?
        .parse::<usize>()
        .expect("array length");
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        let mut bulk_marker = [0_u8; 1];
        stream.read_exact(&mut bulk_marker)?;
        if bulk_marker[0] != b'$' {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "expected RESP bulk string",
            ));
        }
        let length = read_resp_line(stream)?
            .parse::<usize>()
            .expect("bulk string length");
        let mut value = vec![0_u8; length];
        stream.read_exact(&mut value)?;
        let mut crlf = [0_u8; 2];
        stream.read_exact(&mut crlf)?;
        values.push(value);
    }
    Ok(Some(values))
}

#[allow(dead_code)]
fn read_resp_line(stream: &mut TcpStream) -> std::io::Result<String> {
    let mut bytes = Vec::new();
    loop {
        let mut byte = [0_u8; 1];
        stream.read_exact(&mut byte)?;
        if byte[0] == b'\r' {
            let mut newline = [0_u8; 1];
            stream.read_exact(&mut newline)?;
            if newline[0] != b'\n' {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "invalid RESP line ending",
                ));
            }
            break;
        }
        bytes.push(byte[0]);
    }
    Ok(String::from_utf8(bytes).expect("utf8 resp line"))
}

#[allow(dead_code)]
enum RespValue {
    Simple(String),
    Integer(i64),
    Bulk(Option<Vec<u8>>),
    Array(Vec<Vec<u8>>),
}

#[allow(dead_code)]
fn write_resp_value(stream: &mut TcpStream, value: RespValue) -> std::io::Result<()> {
    match value {
        RespValue::Simple(value) => write!(stream, "+{value}\r\n")?,
        RespValue::Integer(value) => write!(stream, ":{value}\r\n")?,
        RespValue::Bulk(Some(value)) => {
            write!(stream, "${}\r\n", value.len())?;
            stream.write_all(&value)?;
            stream.write_all(b"\r\n")?;
        }
        RespValue::Bulk(None) => write!(stream, "$-1\r\n")?,
        RespValue::Array(values) => {
            write!(stream, "*{}\r\n", values.len())?;
            for value in values {
                write!(stream, "${}\r\n", value.len())?;
                stream.write_all(&value)?;
                stream.write_all(b"\r\n")?;
            }
        }
    }
    stream.flush()?;
    Ok(())
}
