use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use sdkwork_api_extension_core::{ExtensionProtocol, ExtensionRuntime};
use sdkwork_api_extension_host::{discover_extension_packages, ExtensionDiscoveryPolicy};

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

[[capabilities]]
operation = "chat.completions.create"
compatibility = "relay"
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
