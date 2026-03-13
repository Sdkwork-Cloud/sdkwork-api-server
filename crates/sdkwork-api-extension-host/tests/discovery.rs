use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use base64::{engine::general_purpose::STANDARD, Engine as _};
use ed25519_dalek::{Signer, SigningKey};
use sdkwork_api_extension_core::{
    CapabilityDescriptor, CompatibilityLevel, ExtensionHealthContract, ExtensionKind,
    ExtensionManifest, ExtensionPermission, ExtensionProtocol, ExtensionRuntime,
    ExtensionSignature, ExtensionSignatureAlgorithm, ExtensionTrustDeclaration,
};
use sdkwork_api_extension_host::{
    discover_extension_packages, validate_discovered_extension_package,
    verify_discovered_extension_package_trust, ExtensionDiscoveryPolicy, ExtensionTrustState,
    ManifestValidationSeverity,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

#[test]
fn discovers_sdkwork_extension_manifests_from_configured_directories() {
    let root = temp_extension_root("connector-openai");
    let package_dir = root.join("sdkwork-provider-custom-openai");
    fs::create_dir_all(&package_dir).unwrap();
    fs::write(
        package_dir.join("sdkwork-extension.toml"),
        connector_manifest("sdkwork.provider.custom-openai", "connector", "openai"),
    )
    .unwrap();

    let policy = ExtensionDiscoveryPolicy::new(vec![root.clone()])
        .with_connector_extensions(true)
        .with_native_dynamic_extensions(false);

    let packages = discover_extension_packages(&policy).expect("discovered packages");

    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].root_dir, package_dir);
    assert_eq!(
        packages[0].manifest_path,
        package_dir.join("sdkwork-extension.toml")
    );
    assert_eq!(packages[0].manifest.id, "sdkwork.provider.custom-openai");
    assert_eq!(packages[0].manifest.runtime, ExtensionRuntime::Connector);
    assert_eq!(
        packages[0].manifest.protocol,
        Some(ExtensionProtocol::OpenAi)
    );
    let report = validate_discovered_extension_package(&packages[0]);
    assert!(report.valid);
    assert!(report.issues.is_empty());

    cleanup_dir(&root);
}

#[test]
fn discovery_filters_disabled_runtimes() {
    let root = temp_extension_root("native-dynamic");
    let package_dir = root.join("sdkwork-provider-native-openai");
    fs::create_dir_all(&package_dir).unwrap();
    fs::write(
        package_dir.join("sdkwork-extension.toml"),
        connector_manifest("sdkwork.provider.native-openai", "native_dynamic", "openai"),
    )
    .unwrap();

    let policy = ExtensionDiscoveryPolicy::new(vec![root.clone()])
        .with_connector_extensions(true)
        .with_native_dynamic_extensions(false);

    let packages = discover_extension_packages(&policy).expect("discovered packages");
    assert!(packages.is_empty());

    cleanup_dir(&root);
}

#[test]
fn discovery_validation_reports_missing_permissions_and_health_contract() {
    let root = temp_extension_root("validation");
    let package_dir = root.join("sdkwork-provider-validation-openai");
    fs::create_dir_all(&package_dir).unwrap();
    fs::write(
        package_dir.join("sdkwork-extension.toml"),
        incomplete_connector_manifest("sdkwork.provider.validation-openai"),
    )
    .unwrap();

    let policy = ExtensionDiscoveryPolicy::new(vec![root.clone()])
        .with_connector_extensions(true)
        .with_native_dynamic_extensions(false);

    let packages = discover_extension_packages(&policy).expect("discovered packages");
    let report = validate_discovered_extension_package(&packages[0]);

    assert!(!report.valid);
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.code == "missing_permissions"
            && issue.severity == ManifestValidationSeverity::Error));
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.code == "missing_health_contract"
            && issue.severity == ManifestValidationSeverity::Warning));

    cleanup_dir(&root);
}

#[test]
fn trust_verification_accepts_trusted_signed_packages() {
    let root = temp_extension_root("trust-signed");
    let package_dir = root.join("sdkwork-provider-trusted-openai");
    fs::create_dir_all(&package_dir).unwrap();
    fs::write(package_dir.join("connector.ps1"), "Write-Output 'ok'").unwrap();

    let signing_key = SigningKey::from_bytes(&[7_u8; 32]);
    let public_key = STANDARD.encode(signing_key.verifying_key().to_bytes());
    let manifest = signed_connector_manifest(
        &package_dir,
        "sdkwork.provider.trusted-openai",
        "sdkwork",
        &signing_key,
    );
    fs::write(
        package_dir.join("sdkwork-extension.toml"),
        toml::to_string(&manifest).unwrap(),
    )
    .unwrap();

    let policy = ExtensionDiscoveryPolicy::new(vec![root.clone()])
        .with_connector_extensions(true)
        .with_native_dynamic_extensions(false)
        .with_trusted_signer("sdkwork", &public_key)
        .with_required_signatures_for_connector_extensions(true);

    let packages = discover_extension_packages(&policy).expect("discovered packages");
    let trust = verify_discovered_extension_package_trust(&packages[0], &policy);

    assert_eq!(trust.state, ExtensionTrustState::Verified);
    assert_eq!(trust.publisher.as_deref(), Some("sdkwork"));
    assert!(trust.signature_present);
    assert!(trust.signature_verified);
    assert!(trust.trusted_signer);
    assert!(trust.load_allowed);
    assert!(trust.issues.is_empty());

    cleanup_dir(&root);
}

#[test]
fn trust_verification_blocks_unsigned_connector_packages_when_required() {
    let root = temp_extension_root("trust-unsigned");
    let package_dir = root.join("sdkwork-provider-unsigned-openai");
    fs::create_dir_all(&package_dir).unwrap();
    fs::write(
        package_dir.join("sdkwork-extension.toml"),
        connector_manifest("sdkwork.provider.unsigned-openai", "connector", "openai"),
    )
    .unwrap();

    let policy = ExtensionDiscoveryPolicy::new(vec![root.clone()])
        .with_connector_extensions(true)
        .with_native_dynamic_extensions(false)
        .with_required_signatures_for_connector_extensions(true);

    let packages = discover_extension_packages(&policy).expect("discovered packages");
    let trust = verify_discovered_extension_package_trust(&packages[0], &policy);

    assert_eq!(trust.state, ExtensionTrustState::Unsigned);
    assert!(!trust.signature_present);
    assert!(!trust.signature_verified);
    assert!(!trust.trusted_signer);
    assert!(!trust.load_allowed);
    assert!(trust
        .issues
        .iter()
        .any(|issue| issue.code == "unsigned_package"));

    cleanup_dir(&root);
}

fn connector_manifest(extension_id: &str, runtime: &str, protocol: &str) -> String {
    format!(
        r#"
api_version = "sdkwork.extension/v1"
id = "{extension_id}"
kind = "provider"
version = "0.1.0"
display_name = "Custom OpenAI"
runtime = "{runtime}"
protocol = "{protocol}"
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
    )
}

fn signed_connector_manifest(
    package_dir: &Path,
    extension_id: &str,
    publisher: &str,
    signing_key: &SigningKey,
) -> ExtensionManifest {
    let manifest = base_connector_manifest(extension_id);
    let signature = sign_package(package_dir, &manifest, signing_key);
    manifest.with_trust(ExtensionTrustDeclaration::signed(
        publisher,
        ExtensionSignature::new(
            ExtensionSignatureAlgorithm::Ed25519,
            signature.public_key,
            signature.signature,
        ),
    ))
}

fn incomplete_connector_manifest(extension_id: &str) -> String {
    format!(
        r#"
api_version = "sdkwork.extension/v1"
id = "{extension_id}"
kind = "provider"
version = "0.1.0"
display_name = "Validation OpenAI"
runtime = "connector"
protocol = "openai"
entrypoint = "bin/sdkwork-provider-validation-openai"
channel_bindings = ["sdkwork.channel.openai"]

[[capabilities]]
operation = "chat.completions.create"
compatibility = "relay"
"#
    )
}

fn base_connector_manifest(extension_id: &str) -> ExtensionManifest {
    ExtensionManifest::new(
        extension_id,
        ExtensionKind::Provider,
        "0.1.0",
        ExtensionRuntime::Connector,
    )
    .with_display_name("Trusted OpenAI")
    .with_protocol(ExtensionProtocol::OpenAi)
    .with_entrypoint("connector.ps1")
    .with_channel_binding("sdkwork.channel.openai")
    .with_permission(ExtensionPermission::NetworkOutbound)
    .with_permission(ExtensionPermission::SpawnProcess)
    .with_health_contract(ExtensionHealthContract::new("/health", 30))
    .with_capability(CapabilityDescriptor::new(
        "chat.completions.create",
        CompatibilityLevel::Relay,
    ))
}

fn sign_package(
    package_dir: &Path,
    manifest: &ExtensionManifest,
    signing_key: &SigningKey,
) -> SignedManifestMaterial {
    let payload = serde_json::to_vec(&PackageSignaturePayload {
        manifest,
        files: collect_package_file_digests(package_dir),
    })
    .unwrap();
    let signature = signing_key.sign(&payload);
    SignedManifestMaterial {
        public_key: STANDARD.encode(signing_key.verifying_key().to_bytes()),
        signature: STANDARD.encode(signature.to_bytes()),
    }
}

fn collect_package_file_digests(root: &Path) -> Vec<PackageFileDigest> {
    let mut files = Vec::new();
    collect_package_file_digests_in(root, root, &mut files);
    files.sort_by(|left, right| left.path.cmp(&right.path));
    files
}

fn collect_package_file_digests_in(
    root: &Path,
    current: &Path,
    files: &mut Vec<PackageFileDigest>,
) {
    let entries = fs::read_dir(current).unwrap();
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            collect_package_file_digests_in(root, &path, files);
            continue;
        }

        if path.file_name().and_then(|name| name.to_str()) == Some("sdkwork-extension.toml") {
            continue;
        }

        let bytes = fs::read(&path).unwrap();
        let digest = Sha256::digest(bytes);
        let relative = path
            .strip_prefix(root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        files.push(PackageFileDigest {
            path: relative,
            sha256: format!("{digest:x}"),
        });
    }
}

#[derive(Debug)]
struct SignedManifestMaterial {
    public_key: String,
    signature: String,
}

#[derive(Serialize)]
struct PackageSignaturePayload<'a> {
    manifest: &'a ExtensionManifest,
    files: Vec<PackageFileDigest>,
}

#[derive(Serialize)]
struct PackageFileDigest {
    path: String,
    sha256: String,
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
