use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::routing::post;
use axum::Router;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use ed25519_dalek::SigningKey;
use sdkwork_api_ext_provider_native_mock::FIXTURE_EXTENSION_ID;
use sqlx::SqlitePool;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tower::ServiceExt;

mod support;

#[tokio::test]
async fn chat_stream_route_accepts_requests() {
    let app = sdkwork_api_interface_http::gateway_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("content-type", "application/json")
                .body(Body::from("{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"hi\"}],\"stream\":true}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

async fn read_body(response: axum::response::Response) -> String {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}

async fn memory_pool() -> SqlitePool {
    sdkwork_api_storage_sqlite::run_migrations("sqlite::memory:")
        .await
        .unwrap()
}

#[derive(Clone, Default)]
struct UpstreamCaptureState {
    authorization: Arc<Mutex<Option<String>>>,
}

#[tokio::test]
async fn stateful_chat_stream_route_relays_to_openai_compatible_provider() {
    let upstream_state = UpstreamCaptureState::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = Router::new()
        .route("/v1/chat/completions", post(upstream_chat_stream_handler))
        .with_state(upstream_state.clone());

    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, "tenant-1", "project-1").await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let _ = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/channels")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"id\":\"openai\",\"name\":\"OpenAI\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    let provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
.header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-openai-official\",\"channel_id\":\"openai\",\"adapter_kind\":\"openai\",\"base_url\":\"http://{address}\",\"display_name\":\"OpenAI Official\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(provider.status(), StatusCode::CREATED);

    let credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
.header("content-type", "application/json")
                .body(Body::from(
                    "{\"tenant_id\":\"tenant-1\",\"provider_id\":\"provider-openai-official\",\"key_reference\":\"cred-openai\",\"secret_value\":\"sk-upstream-openai\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(credential.status(), StatusCode::CREATED);

    let model = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-openai-official\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(model.status(), StatusCode::CREATED);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"relay me\"}],\"stream\":true}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("text/event-stream")
    );
    let body = read_body(response).await;
    assert!(body.contains("chatcmpl_stream_upstream"));
    assert!(body.contains("[DONE]"));
    assert_eq!(
        upstream_state.authorization.lock().unwrap().as_deref(),
        Some("Bearer sk-upstream-openai")
    );
}

#[tokio::test]
async fn stateful_chat_stream_route_relays_to_native_dynamic_provider() {
    let extension_root = temp_extension_root("native-dynamic-stream");
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
    let _guard = native_dynamic_env_guard(&extension_root, &public_key);

    let pool = memory_pool().await;
    let api_key = support::issue_gateway_api_key(&pool, "tenant-1", "project-1").await;
    let admin_app = sdkwork_api_interface_admin::admin_router_with_pool(pool.clone());
    let admin_token = support::issue_admin_token(admin_app.clone()).await;
    let gateway_app = sdkwork_api_interface_http::gateway_router_with_pool(pool);

    let _ = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/channels")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from("{\"id\":\"openai\",\"name\":\"OpenAI\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    let provider = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/providers")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    "{{\"id\":\"provider-native-mock\",\"channel_id\":\"openai\",\"adapter_kind\":\"native-dynamic\",\"base_url\":\"https://native-dynamic.invalid/v1\",\"display_name\":\"Native Mock\",\"extension_id\":\"{FIXTURE_EXTENSION_ID}\"}}"
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(provider.status(), StatusCode::CREATED);

    let credential = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/credentials")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"tenant_id\":\"tenant-1\",\"provider_id\":\"provider-native-mock\",\"key_reference\":\"cred-native-mock\",\"secret_value\":\"sk-native\"}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(credential.status(), StatusCode::CREATED);

    let model = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/models")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"external_name\":\"gpt-4.1\",\"provider_id\":\"provider-native-mock\",\"capabilities\":[\"chat_completions\"],\"streaming\":true}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(model.status(), StatusCode::CREATED);

    let installation = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/installations")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "installation_id": "native-mock-installation",
                        "extension_id": FIXTURE_EXTENSION_ID,
                        "runtime": "native_dynamic",
                        "enabled": true,
                        "entrypoint": library_path.to_string_lossy(),
                        "config": {}
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(installation.status(), StatusCode::CREATED);

    let instance = admin_app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/extensions/instances")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "instance_id": "provider-native-mock",
                        "installation_id": "native-mock-installation",
                        "extension_id": FIXTURE_EXTENSION_ID,
                        "enabled": true,
                        "base_url": "https://native-dynamic.invalid/v1",
                        "credential_ref": "cred-native-mock",
                        "config": {}
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(instance.status(), StatusCode::CREATED);

    let response = gateway_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("authorization", format!("Bearer {api_key}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    "{\"model\":\"gpt-4.1\",\"messages\":[{\"role\":\"user\",\"content\":\"relay me\"}],\"stream\":true}",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("text/event-stream")
    );
    let body = read_body(response).await;
    assert!(body.contains("chatcmpl_native_dynamic_stream"));
    assert!(body.contains("[DONE]"));

    cleanup_dir(&extension_root);
}

async fn upstream_chat_stream_handler(
    State(state): State<UpstreamCaptureState>,
    headers: axum::http::HeaderMap,
) -> ([(axum::http::header::HeaderName, &'static str); 1], String) {
    *state.authorization.lock().unwrap() = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    (
        [(axum::http::header::CONTENT_TYPE, "text/event-stream")],
        "data: {\"id\":\"chatcmpl_stream_upstream\",\"object\":\"chat.completion.chunk\"}\n\ndata: [DONE]\n\n".to_owned(),
    )
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

fn cleanup_dir(path: &Path) {
    let _ = fs::remove_dir_all(path);
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
}

fn sign_native_dynamic_package(
    package_dir: &Path,
    manifest: &sdkwork_api_extension_core::ExtensionManifest,
    signing_key: &SigningKey,
) -> String {
    use ed25519_dalek::Signer;
    use sha2::{Digest, Sha256};

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
            sha256: format!("{:x}", Sha256::digest(std::fs::read(&path).unwrap())),
        })
        .collect::<Vec<_>>();

    let payload = serde_json::to_vec(&PackageSignaturePayload { manifest, files }).unwrap();
    let signature = signing_key.sign(&payload);
    STANDARD.encode(signature.to_bytes())
}
