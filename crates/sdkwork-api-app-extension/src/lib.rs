use anyhow::Result;
use sdkwork_api_extension_core::{ExtensionInstallation, ExtensionInstance, ExtensionRuntime};
use sdkwork_api_extension_host::{
    discover_extension_packages,
    list_connector_runtime_statuses as host_connector_runtime_statuses,
    list_native_dynamic_runtime_statuses as host_native_dynamic_runtime_statuses,
    validate_discovered_extension_package, verify_discovered_extension_package_trust,
    ConnectorRuntimeStatus, DiscoveredExtensionPackage, ExtensionTrustReport,
    ManifestValidationReport, NativeDynamicRuntimeStatus,
};
use sdkwork_api_storage_core::AdminStore;
use serde::Serialize;
use serde_json::Value;

pub use sdkwork_api_extension_host::ExtensionDiscoveryPolicy;

pub struct PersistExtensionInstanceInput<'a> {
    pub instance_id: &'a str,
    pub installation_id: &'a str,
    pub extension_id: &'a str,
    pub enabled: bool,
    pub base_url: Option<&'a str>,
    pub credential_ref: Option<&'a str>,
    pub config: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DiscoveredExtensionPackageRecord {
    pub root_dir: std::path::PathBuf,
    pub manifest_path: std::path::PathBuf,
    pub distribution_name: String,
    pub crate_name: String,
    pub manifest: sdkwork_api_extension_core::ExtensionManifest,
    pub validation: ManifestValidationReport,
    pub trust: ExtensionTrustReport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExtensionRuntimeStatusRecord {
    pub runtime: String,
    pub extension_id: String,
    pub display_name: String,
    pub instance_id: String,
    pub base_url: Option<String>,
    pub health_url: Option<String>,
    pub process_id: Option<u32>,
    pub library_path: Option<String>,
    pub running: bool,
    pub healthy: bool,
    pub supports_health_check: bool,
    pub supports_shutdown: bool,
    pub message: Option<String>,
}

pub type ConnectorRuntimeStatusRecord = ExtensionRuntimeStatusRecord;

pub async fn list_extension_installations(
    store: &dyn AdminStore,
) -> Result<Vec<ExtensionInstallation>> {
    store.list_extension_installations().await
}

pub async fn persist_extension_installation(
    store: &dyn AdminStore,
    installation_id: &str,
    extension_id: &str,
    runtime: ExtensionRuntime,
    enabled: bool,
    entrypoint: Option<&str>,
    config: Value,
) -> Result<ExtensionInstallation> {
    let mut installation =
        ExtensionInstallation::new(installation_id, extension_id, runtime).with_enabled(enabled);
    if let Some(entrypoint) = entrypoint {
        installation = installation.with_entrypoint(entrypoint);
    }
    installation = installation.with_config(config);
    store.insert_extension_installation(&installation).await
}

pub async fn list_extension_instances(store: &dyn AdminStore) -> Result<Vec<ExtensionInstance>> {
    store.list_extension_instances().await
}

pub fn list_discovered_extension_packages(
    policy: &ExtensionDiscoveryPolicy,
) -> Result<Vec<DiscoveredExtensionPackageRecord>> {
    Ok(discover_extension_packages(policy)?
        .into_iter()
        .map(|package| DiscoveredExtensionPackageRecord::from_package(package, policy))
        .collect())
}

pub fn list_connector_runtime_statuses() -> Result<Vec<ConnectorRuntimeStatusRecord>> {
    Ok(host_connector_runtime_statuses()?
        .into_iter()
        .map(ExtensionRuntimeStatusRecord::from)
        .collect())
}

pub fn list_extension_runtime_statuses() -> Result<Vec<ExtensionRuntimeStatusRecord>> {
    let mut statuses = host_connector_runtime_statuses()?
        .into_iter()
        .map(ExtensionRuntimeStatusRecord::from)
        .collect::<Vec<_>>();
    statuses.extend(
        host_native_dynamic_runtime_statuses()?
            .into_iter()
            .map(ExtensionRuntimeStatusRecord::from),
    );
    statuses.sort_by(|left, right| {
        left.runtime
            .cmp(&right.runtime)
            .then(left.extension_id.cmp(&right.extension_id))
            .then(left.instance_id.cmp(&right.instance_id))
    });
    Ok(statuses)
}

pub async fn persist_extension_instance(
    store: &dyn AdminStore,
    input: PersistExtensionInstanceInput<'_>,
) -> Result<ExtensionInstance> {
    let mut instance =
        ExtensionInstance::new(input.instance_id, input.installation_id, input.extension_id)
            .with_enabled(input.enabled);
    if let Some(base_url) = input.base_url {
        instance = instance.with_base_url(base_url);
    }
    if let Some(credential_ref) = input.credential_ref {
        instance = instance.with_credential_ref(credential_ref);
    }
    instance = instance.with_config(input.config);
    store.insert_extension_instance(&instance).await
}

pub fn configured_extension_discovery_policy_from_env() -> ExtensionDiscoveryPolicy {
    let search_paths = std::env::var_os("SDKWORK_EXTENSION_PATHS")
        .map(|value| std::env::split_paths(&value).collect::<Vec<_>>())
        .unwrap_or_default();

    let mut policy = ExtensionDiscoveryPolicy::new(search_paths)
        .with_connector_extensions(env_flag(
            "SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS",
            true,
        ))
        .with_native_dynamic_extensions(env_flag(
            "SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS",
            false,
        ))
        .with_required_signatures_for_connector_extensions(env_flag(
            "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS",
            false,
        ))
        .with_required_signatures_for_native_dynamic_extensions(env_flag(
            "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS",
            true,
        ));
    for (publisher, public_key) in env_trusted_signers("SDKWORK_EXTENSION_TRUSTED_SIGNERS") {
        policy = policy.with_trusted_signer(publisher, public_key);
    }
    policy
}

impl DiscoveredExtensionPackageRecord {
    fn from_package(value: DiscoveredExtensionPackage, policy: &ExtensionDiscoveryPolicy) -> Self {
        let validation = validate_discovered_extension_package(&value);
        let trust = verify_discovered_extension_package_trust(&value, policy);
        Self {
            root_dir: value.root_dir,
            manifest_path: value.manifest_path,
            distribution_name: value.manifest.distribution_name(),
            crate_name: value.manifest.crate_name(),
            manifest: value.manifest,
            validation,
            trust,
        }
    }
}

impl From<ConnectorRuntimeStatus> for ExtensionRuntimeStatusRecord {
    fn from(value: ConnectorRuntimeStatus) -> Self {
        Self {
            runtime: "connector".to_owned(),
            extension_id: value.extension_id,
            display_name: value.display_name,
            instance_id: value.instance_id,
            base_url: Some(value.base_url),
            health_url: Some(value.health_url),
            process_id: value.process_id,
            library_path: None,
            running: value.running,
            healthy: value.healthy,
            supports_health_check: true,
            supports_shutdown: true,
            message: None,
        }
    }
}

impl From<NativeDynamicRuntimeStatus> for ExtensionRuntimeStatusRecord {
    fn from(value: NativeDynamicRuntimeStatus) -> Self {
        Self {
            runtime: "native_dynamic".to_owned(),
            extension_id: value.extension_id,
            display_name: value.display_name,
            instance_id: String::new(),
            base_url: None,
            health_url: None,
            process_id: None,
            library_path: Some(value.library_path),
            running: value.running,
            healthy: value.healthy,
            supports_health_check: value.supports_health_check,
            supports_shutdown: value.supports_shutdown,
            message: value.message,
        }
    }
}

fn env_flag(key: &str, default: bool) -> bool {
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse::<bool>().ok())
        .unwrap_or(default)
}

fn env_trusted_signers(key: &str) -> Vec<(String, String)> {
    std::env::var(key)
        .ok()
        .map(|value| parse_trusted_signers(&value))
        .unwrap_or_default()
}

fn parse_trusted_signers(value: &str) -> Vec<(String, String)> {
    value
        .split(';')
        .filter_map(|entry| {
            let entry = entry.trim();
            if entry.is_empty() {
                return None;
            }
            let (publisher, public_key) = entry.split_once('=')?;
            let publisher = publisher.trim();
            let public_key = public_key.trim();
            if publisher.is_empty() || public_key.is_empty() {
                return None;
            }
            Some((publisher.to_owned(), public_key.to_owned()))
        })
        .collect()
}
