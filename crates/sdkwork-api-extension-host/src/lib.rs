use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use base64::{engine::general_purpose::STANDARD, Engine as _};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use sdkwork_api_extension_core::{
    ExtensionInstallation, ExtensionInstance, ExtensionManifest, ExtensionRuntime,
    ExtensionSignatureAlgorithm,
};
use sdkwork_api_provider_core::ProviderExecutionAdapter;
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct BuiltinExtensionFactory {
    manifest: ExtensionManifest,
}

impl BuiltinExtensionFactory {
    pub fn new(manifest: ExtensionManifest) -> Self {
        Self { manifest }
    }
}

type ProviderFactory =
    Arc<dyn Fn(String) -> Box<dyn ProviderExecutionAdapter> + Send + Sync + 'static>;

#[derive(Clone)]
pub struct BuiltinProviderExtensionFactory {
    manifest: ExtensionManifest,
    adapter_kind: String,
    factory: ProviderFactory,
}

impl BuiltinProviderExtensionFactory {
    pub fn new<F>(manifest: ExtensionManifest, adapter_kind: impl Into<String>, factory: F) -> Self
    where
        F: Fn(String) -> Box<dyn ProviderExecutionAdapter> + Send + Sync + 'static,
    {
        Self {
            manifest,
            adapter_kind: adapter_kind.into(),
            factory: Arc::new(factory),
        }
    }
}

#[derive(Default, Clone)]
pub struct ExtensionHost {
    manifests: HashMap<String, ExtensionManifest>,
    package_roots: HashMap<String, PathBuf>,
    provider_factories: HashMap<String, ProviderFactory>,
    provider_aliases: HashMap<String, String>,
    installations: HashMap<String, ExtensionInstallation>,
    instances_by_id: HashMap<String, ExtensionInstance>,
    instances_by_extension: HashMap<String, Vec<ExtensionInstance>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExtensionLoadPlan {
    pub instance_id: String,
    pub installation_id: String,
    pub extension_id: String,
    pub enabled: bool,
    pub runtime: ExtensionRuntime,
    pub display_name: String,
    pub entrypoint: Option<String>,
    pub base_url: Option<String>,
    pub credential_ref: Option<String>,
    pub config_schema: Option<String>,
    pub credential_schema: Option<String>,
    pub package_root: Option<PathBuf>,
    pub config: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionDiscoveryPolicy {
    pub search_paths: Vec<PathBuf>,
    pub enable_connector_extensions: bool,
    pub enable_native_dynamic_extensions: bool,
    pub trusted_signers: HashMap<String, String>,
    pub require_signed_connector_extensions: bool,
    pub require_signed_native_dynamic_extensions: bool,
}

impl ExtensionDiscoveryPolicy {
    pub fn new(search_paths: Vec<PathBuf>) -> Self {
        Self {
            search_paths,
            enable_connector_extensions: true,
            enable_native_dynamic_extensions: false,
            trusted_signers: HashMap::new(),
            require_signed_connector_extensions: false,
            require_signed_native_dynamic_extensions: true,
        }
    }

    pub fn with_connector_extensions(mut self, enabled: bool) -> Self {
        self.enable_connector_extensions = enabled;
        self
    }

    pub fn with_native_dynamic_extensions(mut self, enabled: bool) -> Self {
        self.enable_native_dynamic_extensions = enabled;
        self
    }

    pub fn with_trusted_signer(
        mut self,
        publisher: impl Into<String>,
        public_key: impl Into<String>,
    ) -> Self {
        self.trusted_signers
            .insert(publisher.into(), public_key.into());
        self
    }

    pub fn with_required_signatures_for_connector_extensions(mut self, enabled: bool) -> Self {
        self.require_signed_connector_extensions = enabled;
        self
    }

    pub fn with_required_signatures_for_native_dynamic_extensions(mut self, enabled: bool) -> Self {
        self.require_signed_native_dynamic_extensions = enabled;
        self
    }

    fn allows_runtime(&self, runtime: &ExtensionRuntime) -> bool {
        match runtime {
            ExtensionRuntime::Builtin => true,
            ExtensionRuntime::Connector => self.enable_connector_extensions,
            ExtensionRuntime::NativeDynamic => self.enable_native_dynamic_extensions,
        }
    }

    fn requires_signature(&self, runtime: &ExtensionRuntime) -> bool {
        match runtime {
            ExtensionRuntime::Builtin => false,
            ExtensionRuntime::Connector => self.require_signed_connector_extensions,
            ExtensionRuntime::NativeDynamic => self.require_signed_native_dynamic_extensions,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredExtensionPackage {
    pub root_dir: PathBuf,
    pub manifest_path: PathBuf,
    pub manifest: ExtensionManifest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ManifestValidationSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ManifestValidationIssue {
    pub severity: ManifestValidationSeverity,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ManifestValidationReport {
    pub valid: bool,
    pub issues: Vec<ManifestValidationIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionTrustState {
    Verified,
    Unsigned,
    UntrustedSigner,
    InvalidSignature,
    VerificationFailed,
}

impl ExtensionTrustState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Verified => "verified",
            Self::Unsigned => "unsigned",
            Self::UntrustedSigner => "untrusted_signer",
            Self::InvalidSignature => "invalid_signature",
            Self::VerificationFailed => "verification_failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExtensionTrustIssue {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExtensionTrustReport {
    pub state: ExtensionTrustState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,
    pub signature_present: bool,
    pub signature_verified: bool,
    pub trusted_signer: bool,
    pub load_allowed: bool,
    pub issues: Vec<ExtensionTrustIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectorRuntimeStatus {
    pub instance_id: String,
    pub base_url: String,
    pub health_url: String,
    pub process_id: Option<u32>,
    pub running: bool,
    pub healthy: bool,
}

#[derive(Debug)]
struct ManagedConnectorProcess {
    child: Child,
    base_url: String,
    health_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ConnectorLaunchConfig {
    entrypoint: PathBuf,
    args: Vec<String>,
    environment: HashMap<String, String>,
    working_directory: Option<PathBuf>,
    health_url: String,
    startup_timeout: Duration,
    startup_poll_interval: Duration,
}

static CONNECTOR_PROCESS_REGISTRY: OnceLock<Mutex<HashMap<String, ManagedConnectorProcess>>> =
    OnceLock::new();

pub fn discover_extension_packages(
    policy: &ExtensionDiscoveryPolicy,
) -> Result<Vec<DiscoveredExtensionPackage>, ExtensionHostError> {
    let mut packages = Vec::new();
    for search_path in &policy.search_paths {
        discover_in_path(search_path, policy, &mut packages)?;
    }
    packages.sort_by(|left, right| left.manifest_path.cmp(&right.manifest_path));
    Ok(packages)
}

pub fn validate_discovered_extension_package(
    package: &DiscoveredExtensionPackage,
) -> ManifestValidationReport {
    validate_extension_manifest(&package.manifest)
}

pub fn verify_discovered_extension_package_trust(
    package: &DiscoveredExtensionPackage,
    policy: &ExtensionDiscoveryPolicy,
) -> ExtensionTrustReport {
    let requires_signature = policy.requires_signature(&package.manifest.runtime);
    let Some(trust) = package.manifest.trust.as_ref() else {
        return ExtensionTrustReport {
            state: ExtensionTrustState::Unsigned,
            publisher: None,
            signature_present: false,
            signature_verified: false,
            trusted_signer: false,
            load_allowed: !requires_signature,
            issues: vec![ExtensionTrustIssue {
                code: "unsigned_package".to_owned(),
                message: if requires_signature {
                    "extension package must be signed by a trusted publisher for this runtime"
                        .to_owned()
                } else {
                    "extension package does not declare trust metadata".to_owned()
                },
            }],
        };
    };

    let payload = match package_signature_payload(package) {
        Ok(payload) => payload,
        Err(message) => {
            return ExtensionTrustReport {
                state: ExtensionTrustState::VerificationFailed,
                publisher: Some(trust.publisher.clone()),
                signature_present: true,
                signature_verified: false,
                trusted_signer: false,
                load_allowed: false,
                issues: vec![ExtensionTrustIssue {
                    code: "package_payload_unreadable".to_owned(),
                    message: message.to_string(),
                }],
            }
        }
    };

    if let Err(message) = verify_signature_bytes(
        &payload,
        trust.signature.algorithm.clone(),
        &trust.signature.public_key,
        &trust.signature.signature,
    ) {
        return ExtensionTrustReport {
            state: ExtensionTrustState::InvalidSignature,
            publisher: Some(trust.publisher.clone()),
            signature_present: true,
            signature_verified: false,
            trusted_signer: false,
            load_allowed: false,
            issues: vec![ExtensionTrustIssue {
                code: "invalid_signature".to_owned(),
                message,
            }],
        };
    }

    let trusted_signer = policy
        .trusted_signers
        .get(&trust.publisher)
        .map(|expected_public_key| {
            public_keys_match(expected_public_key, &trust.signature.public_key)
        })
        .unwrap_or(false);
    if !trusted_signer {
        return ExtensionTrustReport {
            state: ExtensionTrustState::UntrustedSigner,
            publisher: Some(trust.publisher.clone()),
            signature_present: true,
            signature_verified: true,
            trusted_signer: false,
            load_allowed: false,
            issues: vec![ExtensionTrustIssue {
                code: "untrusted_signer".to_owned(),
                message: format!(
                    "publisher {} is not trusted by the current extension trust policy",
                    trust.publisher
                ),
            }],
        };
    }

    ExtensionTrustReport {
        state: ExtensionTrustState::Verified,
        publisher: Some(trust.publisher.clone()),
        signature_present: true,
        signature_verified: true,
        trusted_signer: true,
        load_allowed: true,
        issues: Vec::new(),
    }
}

pub fn ensure_connector_runtime_started(
    load_plan: &ExtensionLoadPlan,
    base_url: &str,
) -> Result<ConnectorRuntimeStatus, ExtensionHostError> {
    if load_plan.runtime != ExtensionRuntime::Connector {
        return Err(ExtensionHostError::ConnectorRuntimeUnsupported {
            instance_id: load_plan.instance_id.clone(),
            runtime: load_plan.runtime.clone(),
        });
    }

    let launch_config = ConnectorLaunchConfig::from_load_plan(load_plan, base_url)?;

    if let Some(status) = running_connector_status(&load_plan.instance_id)? {
        if status.base_url == base_url
            && status.health_url == launch_config.health_url
            && probe_http_health(&launch_config.health_url)?
        {
            return Ok(ConnectorRuntimeStatus {
                healthy: true,
                ..status
            });
        }
        shutdown_connector_runtime(&load_plan.instance_id)?;
    }

    if probe_http_health(&launch_config.health_url)? {
        return Ok(ConnectorRuntimeStatus {
            instance_id: load_plan.instance_id.clone(),
            base_url: base_url.to_owned(),
            health_url: launch_config.health_url,
            process_id: None,
            running: true,
            healthy: true,
        });
    }

    let mut command = Command::new(&launch_config.entrypoint);
    command.args(&launch_config.args);
    command.stdin(Stdio::null());
    command.stdout(Stdio::null());
    command.stderr(Stdio::null());
    if let Some(working_directory) = &launch_config.working_directory {
        command.current_dir(working_directory);
    }
    command.env(
        "SDKWORK_EXTENSION_INSTANCE_ID",
        load_plan.instance_id.as_str(),
    );
    command.env(
        "SDKWORK_EXTENSION_INSTALLATION_ID",
        load_plan.installation_id.as_str(),
    );
    command.env("SDKWORK_EXTENSION_ID", load_plan.extension_id.as_str());
    command.env("SDKWORK_EXTENSION_BASE_URL", base_url);
    if let Some(credential_ref) = &load_plan.credential_ref {
        command.env("SDKWORK_EXTENSION_CREDENTIAL_REF", credential_ref);
    }
    command.env(
        "SDKWORK_EXTENSION_CONFIG_JSON",
        load_plan.config.to_string(),
    );
    for (key, value) in &launch_config.environment {
        command.env(key, value);
    }

    let child =
        command
            .spawn()
            .map_err(|error| ExtensionHostError::ConnectorRuntimeSpawnFailed {
                instance_id: load_plan.instance_id.clone(),
                entrypoint: launch_config.entrypoint.display().to_string(),
                message: error.to_string(),
            })?;

    let process_id = Some(child.id());
    {
        let mut registry = connector_process_registry()?;
        registry.insert(
            load_plan.instance_id.clone(),
            ManagedConnectorProcess {
                child,
                base_url: base_url.to_owned(),
                health_url: launch_config.health_url.clone(),
            },
        );
    }

    wait_for_connector_health(
        &load_plan.instance_id,
        &launch_config.health_url,
        launch_config.startup_timeout,
        launch_config.startup_poll_interval,
    )?;

    Ok(ConnectorRuntimeStatus {
        instance_id: load_plan.instance_id.clone(),
        base_url: base_url.to_owned(),
        health_url: launch_config.health_url,
        process_id,
        running: true,
        healthy: true,
    })
}

pub fn shutdown_connector_runtime(instance_id: &str) -> Result<(), ExtensionHostError> {
    let mut registry = connector_process_registry()?;
    if let Some(mut process) = registry.remove(instance_id) {
        kill_child(instance_id, &mut process.child)?;
    }
    Ok(())
}

pub fn shutdown_all_connector_runtimes() -> Result<(), ExtensionHostError> {
    let mut registry = connector_process_registry()?;
    let instance_ids = registry.keys().cloned().collect::<Vec<_>>();
    for instance_id in instance_ids {
        if let Some(mut process) = registry.remove(&instance_id) {
            kill_child(&instance_id, &mut process.child)?;
        }
    }
    Ok(())
}

pub fn list_connector_runtime_statuses() -> Result<Vec<ConnectorRuntimeStatus>, ExtensionHostError>
{
    let snapshots = {
        let mut registry = connector_process_registry()?;
        let mut exited_instance_ids = Vec::new();
        let mut snapshots = Vec::new();

        for (instance_id, process) in registry.iter_mut() {
            match process.child.try_wait() {
                Ok(None) => snapshots.push((
                    instance_id.clone(),
                    process.base_url.clone(),
                    process.health_url.clone(),
                    Some(process.child.id()),
                )),
                Ok(Some(_)) => exited_instance_ids.push(instance_id.clone()),
                Err(error) => {
                    return Err(ExtensionHostError::ConnectorRuntimeShutdownFailed {
                        instance_id: instance_id.clone(),
                        message: error.to_string(),
                    })
                }
            }
        }

        for instance_id in exited_instance_ids {
            registry.remove(&instance_id);
        }

        snapshots
    };

    let mut statuses = snapshots
        .into_iter()
        .map(|(instance_id, base_url, health_url, process_id)| {
            Ok(ConnectorRuntimeStatus {
                instance_id,
                base_url,
                healthy: probe_http_health(&health_url)?,
                health_url,
                process_id,
                running: true,
            })
        })
        .collect::<Result<Vec<_>, ExtensionHostError>>()?;
    statuses.sort_by(|left, right| left.instance_id.cmp(&right.instance_id));
    Ok(statuses)
}

pub fn validate_extension_manifest(manifest: &ExtensionManifest) -> ManifestValidationReport {
    let mut issues = Vec::new();

    if manifest.permissions.is_empty() {
        issues.push(ManifestValidationIssue {
            severity: ManifestValidationSeverity::Error,
            code: "missing_permissions".to_owned(),
            message: "extension manifest must declare explicit permissions".to_owned(),
        });
    }

    if manifest.channel_bindings.is_empty() {
        issues.push(ManifestValidationIssue {
            severity: ManifestValidationSeverity::Error,
            code: "missing_channel_bindings".to_owned(),
            message: "extension manifest must declare at least one channel binding".to_owned(),
        });
    }

    if manifest.capabilities.is_empty() {
        issues.push(ManifestValidationIssue {
            severity: ManifestValidationSeverity::Error,
            code: "missing_capabilities".to_owned(),
            message: "extension manifest must declare at least one capability".to_owned(),
        });
    }

    if matches!(
        manifest.runtime,
        ExtensionRuntime::Connector | ExtensionRuntime::NativeDynamic
    ) && manifest.entrypoint.is_none()
    {
        issues.push(ManifestValidationIssue {
            severity: ManifestValidationSeverity::Error,
            code: "missing_entrypoint".to_owned(),
            message: "runtime-backed extension manifest must declare an entrypoint".to_owned(),
        });
    }

    if manifest.runtime == ExtensionRuntime::Connector && manifest.health.is_none() {
        issues.push(ManifestValidationIssue {
            severity: ManifestValidationSeverity::Warning,
            code: "missing_health_contract".to_owned(),
            message: "connector extensions should declare an explicit health contract".to_owned(),
        });
    }

    let valid = !issues
        .iter()
        .any(|issue| issue.severity == ManifestValidationSeverity::Error);
    ManifestValidationReport { valid, issues }
}

impl ExtensionHost {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_builtin(&mut self, factory: BuiltinExtensionFactory) {
        self.register_builtin_manifest(factory.manifest);
    }

    pub fn register_builtin_manifest(&mut self, manifest: ExtensionManifest) {
        self.manifests.insert(manifest.id.clone(), manifest);
    }

    pub fn register_discovered_manifest(&mut self, package: DiscoveredExtensionPackage) {
        self.package_roots
            .insert(package.manifest.id.clone(), package.root_dir.clone());
        self.manifests
            .insert(package.manifest.id.clone(), package.manifest);
    }

    pub fn register_builtin_provider(&mut self, factory: BuiltinProviderExtensionFactory) {
        let extension_id = factory.manifest.id.clone();
        self.register_builtin_manifest(factory.manifest);
        self.provider_factories
            .insert(extension_id.clone(), factory.factory);
        self.provider_aliases
            .insert(factory.adapter_kind, extension_id);
    }

    pub fn register_discovered_provider<F>(
        &mut self,
        package: DiscoveredExtensionPackage,
        adapter_kind: impl Into<String>,
        factory: F,
    ) where
        F: Fn(String) -> Box<dyn ProviderExecutionAdapter> + Send + Sync + 'static,
    {
        let extension_id = package.manifest.id.clone();
        self.package_roots
            .insert(extension_id.clone(), package.root_dir.clone());
        self.manifests
            .insert(extension_id.clone(), package.manifest);
        self.provider_factories
            .insert(extension_id.clone(), Arc::new(factory));
        self.provider_aliases
            .insert(adapter_kind.into(), extension_id);
    }

    pub fn manifest(&self, id: &str) -> Option<&ExtensionManifest> {
        self.manifests.get(id)
    }

    pub fn install(
        &mut self,
        installation: ExtensionInstallation,
    ) -> Result<(), ExtensionHostError> {
        if !self.manifests.contains_key(&installation.extension_id) {
            return Err(ExtensionHostError::ManifestNotFound {
                extension_id: installation.extension_id,
            });
        }

        self.installations
            .insert(installation.installation_id.clone(), installation);
        Ok(())
    }

    pub fn installations(&self) -> Vec<ExtensionInstallation> {
        self.installations.values().cloned().collect()
    }

    pub fn mount_instance(
        &mut self,
        instance: ExtensionInstance,
    ) -> Result<(), ExtensionHostError> {
        let Some(installation) = self.installations.get(&instance.installation_id) else {
            return Err(ExtensionHostError::InstallationNotFound {
                installation_id: instance.installation_id,
            });
        };

        if installation.extension_id != instance.extension_id {
            return Err(ExtensionHostError::InstallationExtensionMismatch {
                installation_id: installation.installation_id.clone(),
                installation_extension_id: installation.extension_id.clone(),
                instance_extension_id: instance.extension_id,
            });
        }

        let instances = self
            .instances_by_extension
            .entry(installation.extension_id.clone())
            .or_default();
        if let Some(existing) = instances
            .iter_mut()
            .find(|existing| existing.instance_id == instance.instance_id)
        {
            *existing = instance.clone();
        } else {
            instances.push(instance.clone());
        }

        self.instances_by_id
            .insert(instance.instance_id.clone(), instance);
        Ok(())
    }

    pub fn instances(&self, extension_id: &str) -> Vec<ExtensionInstance> {
        self.instances_by_extension
            .get(extension_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn load_plan(&self, instance_id: &str) -> Result<ExtensionLoadPlan, ExtensionHostError> {
        let Some(instance) = self.instances_by_id.get(instance_id) else {
            return Err(ExtensionHostError::InstanceNotFound {
                instance_id: instance_id.to_owned(),
            });
        };
        let Some(installation) = self.installations.get(&instance.installation_id) else {
            return Err(ExtensionHostError::InstallationNotFound {
                installation_id: instance.installation_id.clone(),
            });
        };
        let Some(manifest) = self.manifests.get(&instance.extension_id) else {
            return Err(ExtensionHostError::ManifestNotFound {
                extension_id: instance.extension_id.clone(),
            });
        };

        if installation.runtime != manifest.runtime {
            return Err(ExtensionHostError::RuntimeMismatch {
                extension_id: manifest.id.clone(),
                manifest_runtime: manifest.runtime.clone(),
                installation_runtime: installation.runtime.clone(),
            });
        }

        Ok(ExtensionLoadPlan {
            instance_id: instance.instance_id.clone(),
            installation_id: installation.installation_id.clone(),
            extension_id: manifest.id.clone(),
            enabled: installation.enabled && instance.enabled,
            runtime: installation.runtime.clone(),
            display_name: manifest.display_name.clone(),
            entrypoint: installation
                .entrypoint
                .clone()
                .or_else(|| manifest.entrypoint.clone()),
            base_url: instance.base_url.clone(),
            credential_ref: instance.credential_ref.clone(),
            config_schema: manifest.config_schema.clone(),
            credential_schema: manifest.credential_schema.clone(),
            package_root: self.package_roots.get(&instance.extension_id).cloned(),
            config: merge_config(&installation.config, &instance.config),
        })
    }

    pub fn resolve_provider(
        &self,
        runtime_key: &str,
        base_url: impl Into<String>,
    ) -> Option<Box<dyn ProviderExecutionAdapter>> {
        let base_url = base_url.into();
        self.provider_factories
            .get(runtime_key)
            .or_else(|| {
                self.provider_aliases
                    .get(runtime_key)
                    .and_then(|extension_id| self.provider_factories.get(extension_id))
            })
            .map(|factory| factory(base_url))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtensionHostError {
    ManifestNotFound {
        extension_id: String,
    },
    InstallationNotFound {
        installation_id: String,
    },
    InstanceNotFound {
        instance_id: String,
    },
    InstallationExtensionMismatch {
        installation_id: String,
        installation_extension_id: String,
        instance_extension_id: String,
    },
    RuntimeMismatch {
        extension_id: String,
        manifest_runtime: ExtensionRuntime,
        installation_runtime: ExtensionRuntime,
    },
    ManifestReadFailed {
        path: String,
        message: String,
    },
    ManifestParseFailed {
        path: String,
        message: String,
    },
    ConnectorRuntimeUnsupported {
        instance_id: String,
        runtime: ExtensionRuntime,
    },
    ConnectorRuntimeEntrypointMissing {
        instance_id: String,
    },
    ConnectorRuntimeBaseUrlInvalid {
        instance_id: String,
        base_url: String,
    },
    ConnectorRuntimeSpawnFailed {
        instance_id: String,
        entrypoint: String,
        message: String,
    },
    ConnectorRuntimeStatePoisoned,
    ConnectorRuntimeExited {
        instance_id: String,
        status: Option<i32>,
    },
    ConnectorRuntimeHealthTimedOut {
        instance_id: String,
        health_url: String,
        timeout_ms: u64,
    },
    ConnectorRuntimeShutdownFailed {
        instance_id: String,
        message: String,
    },
}

impl fmt::Display for ExtensionHostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ManifestNotFound { extension_id } => {
                write!(f, "extension manifest not found: {extension_id}")
            }
            Self::InstallationNotFound { installation_id } => {
                write!(f, "extension installation not found: {installation_id}")
            }
            Self::InstanceNotFound { instance_id } => {
                write!(f, "extension instance not found: {instance_id}")
            }
            Self::InstallationExtensionMismatch {
                installation_id,
                installation_extension_id,
                instance_extension_id,
            } => write!(
                f,
                "extension instance references {} but installation {} is bound to {}",
                instance_extension_id, installation_id, installation_extension_id
            ),
            Self::RuntimeMismatch {
                extension_id,
                manifest_runtime,
                installation_runtime,
            } => write!(
                f,
                "extension {} manifest runtime {:?} does not match installation runtime {:?}",
                extension_id, manifest_runtime, installation_runtime
            ),
            Self::ManifestReadFailed { path, message } => {
                write!(f, "failed to read extension manifest {}: {}", path, message)
            }
            Self::ManifestParseFailed { path, message } => {
                write!(
                    f,
                    "failed to parse extension manifest {}: {}",
                    path, message
                )
            }
            Self::ConnectorRuntimeUnsupported {
                instance_id,
                runtime,
            } => write!(
                f,
                "extension instance {} uses unsupported connector runtime {:?}",
                instance_id, runtime
            ),
            Self::ConnectorRuntimeEntrypointMissing { instance_id } => write!(
                f,
                "connector extension instance {} has no executable entrypoint",
                instance_id
            ),
            Self::ConnectorRuntimeBaseUrlInvalid {
                instance_id,
                base_url,
            } => write!(
                f,
                "connector extension instance {} has invalid base url {}",
                instance_id, base_url
            ),
            Self::ConnectorRuntimeSpawnFailed {
                instance_id,
                entrypoint,
                message,
            } => write!(
                f,
                "failed to spawn connector extension instance {} from {}: {}",
                instance_id, entrypoint, message
            ),
            Self::ConnectorRuntimeStatePoisoned => {
                write!(f, "connector runtime state is poisoned")
            }
            Self::ConnectorRuntimeExited {
                instance_id,
                status,
            } => write!(
                f,
                "connector extension instance {} exited before becoming ready with status {:?}",
                instance_id, status
            ),
            Self::ConnectorRuntimeHealthTimedOut {
                instance_id,
                health_url,
                timeout_ms,
            } => write!(
                f,
                "connector extension instance {} did not become healthy at {} within {}ms",
                instance_id, health_url, timeout_ms
            ),
            Self::ConnectorRuntimeShutdownFailed {
                instance_id,
                message,
            } => write!(
                f,
                "failed to shut down connector extension instance {}: {}",
                instance_id, message
            ),
        }
    }
}

impl std::error::Error for ExtensionHostError {}

fn merge_config(base: &Value, overlay: &Value) -> Value {
    match (base, overlay) {
        (Value::Object(base_map), Value::Object(overlay_map)) => {
            let mut merged = base_map.clone();
            for (key, overlay_value) in overlay_map {
                let value = match merged.get(key) {
                    Some(base_value) => merge_config(base_value, overlay_value),
                    None => overlay_value.clone(),
                };
                merged.insert(key.clone(), value);
            }
            Value::Object(merged)
        }
        (_, overlay) => overlay.clone(),
    }
}

#[derive(Serialize)]
struct PackageSignaturePayload {
    manifest: ExtensionManifest,
    files: Vec<PackageFileDigest>,
}

#[derive(Serialize)]
struct PackageFileDigest {
    path: String,
    sha256: String,
}

fn package_signature_payload(
    package: &DiscoveredExtensionPackage,
) -> Result<Vec<u8>, ExtensionHostError> {
    let mut manifest = package.manifest.clone();
    manifest.trust = None;
    let payload = PackageSignaturePayload {
        manifest,
        files: collect_package_file_digests(&package.root_dir)?,
    };
    serde_json::to_vec(&payload).map_err(|error| ExtensionHostError::ManifestReadFailed {
        path: package.manifest_path.display().to_string(),
        message: error.to_string(),
    })
}

fn collect_package_file_digests(root: &Path) -> Result<Vec<PackageFileDigest>, ExtensionHostError> {
    let mut files = Vec::new();
    collect_package_file_digests_in(root, root, &mut files)?;
    files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(files)
}

fn collect_package_file_digests_in(
    root: &Path,
    current: &Path,
    files: &mut Vec<PackageFileDigest>,
) -> Result<(), ExtensionHostError> {
    let entries =
        fs::read_dir(current).map_err(|error| ExtensionHostError::ManifestReadFailed {
            path: current.display().to_string(),
            message: error.to_string(),
        })?;
    for entry in entries {
        let entry = entry.map_err(|error| ExtensionHostError::ManifestReadFailed {
            path: current.display().to_string(),
            message: error.to_string(),
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_package_file_digests_in(root, &path, files)?;
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) == Some("sdkwork-extension.toml") {
            continue;
        }

        let bytes = fs::read(&path).map_err(|error| ExtensionHostError::ManifestReadFailed {
            path: path.display().to_string(),
            message: error.to_string(),
        })?;
        let relative_path = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        files.push(PackageFileDigest {
            path: relative_path,
            sha256: format!("{:x}", Sha256::digest(bytes)),
        });
    }
    Ok(())
}

fn verify_signature_bytes(
    payload: &[u8],
    algorithm: ExtensionSignatureAlgorithm,
    public_key: &str,
    signature: &str,
) -> Result<(), String> {
    match algorithm {
        ExtensionSignatureAlgorithm::Ed25519 => {
            let public_key_bytes = decode_fixed_base64::<32>(public_key, "public key")?;
            let verifying_key =
                VerifyingKey::from_bytes(&public_key_bytes).map_err(|error| error.to_string())?;
            let signature_bytes = decode_fixed_base64::<64>(signature, "signature")?;
            let signature = Signature::from_bytes(&signature_bytes);
            verifying_key
                .verify(payload, &signature)
                .map_err(|error| error.to_string())
        }
    }
}

fn public_keys_match(expected_public_key: &str, actual_public_key: &str) -> bool {
    match (
        STANDARD.decode(expected_public_key),
        STANDARD.decode(actual_public_key),
    ) {
        (Ok(expected), Ok(actual)) => expected == actual,
        _ => expected_public_key == actual_public_key,
    }
}

fn decode_fixed_base64<const N: usize>(value: &str, label: &str) -> Result<[u8; N], String> {
    let decoded = STANDARD
        .decode(value)
        .map_err(|error| format!("invalid {label} encoding: {error}"))?;
    decoded
        .try_into()
        .map_err(|_| format!("invalid {label} length"))
}

fn discover_in_path(
    path: &Path,
    policy: &ExtensionDiscoveryPolicy,
    packages: &mut Vec<DiscoveredExtensionPackage>,
) -> Result<(), ExtensionHostError> {
    if !path.exists() {
        return Ok(());
    }

    if path.is_file() {
        if path.file_name().and_then(|name| name.to_str()) == Some("sdkwork-extension.toml") {
            let manifest = parse_manifest(path)?;
            if policy.allows_runtime(&manifest.runtime) {
                let root_dir = path.parent().unwrap_or(path).to_path_buf();
                packages.push(DiscoveredExtensionPackage {
                    root_dir,
                    manifest_path: path.to_path_buf(),
                    manifest,
                });
            }
        }
        return Ok(());
    }

    let entries = fs::read_dir(path).map_err(|error| ExtensionHostError::ManifestReadFailed {
        path: path.display().to_string(),
        message: error.to_string(),
    })?;

    for entry in entries {
        let entry = entry.map_err(|error| ExtensionHostError::ManifestReadFailed {
            path: path.display().to_string(),
            message: error.to_string(),
        })?;
        discover_in_path(&entry.path(), policy, packages)?;
    }

    Ok(())
}

fn parse_manifest(path: &Path) -> Result<ExtensionManifest, ExtensionHostError> {
    let manifest_text =
        fs::read_to_string(path).map_err(|error| ExtensionHostError::ManifestReadFailed {
            path: path.display().to_string(),
            message: error.to_string(),
        })?;

    toml::from_str(&manifest_text).map_err(|error| ExtensionHostError::ManifestParseFailed {
        path: path.display().to_string(),
        message: error.to_string(),
    })
}

impl ConnectorLaunchConfig {
    fn from_load_plan(
        load_plan: &ExtensionLoadPlan,
        base_url: &str,
    ) -> Result<Self, ExtensionHostError> {
        let Some(entrypoint) = load_plan.entrypoint.as_deref() else {
            return Err(ExtensionHostError::ConnectorRuntimeEntrypointMissing {
                instance_id: load_plan.instance_id.clone(),
            });
        };

        let entrypoint = resolve_entrypoint(entrypoint, load_plan.package_root.as_deref());
        let args = config_string_array(&load_plan.config, "command_args");
        let environment = config_string_map(&load_plan.config, "environment");
        let working_directory = config_path(
            &load_plan.config,
            "working_directory",
            load_plan.package_root.as_deref(),
        )
        .or_else(|| load_plan.package_root.clone());
        let health_path =
            config_string(&load_plan.config, "health_path").unwrap_or_else(|| "/health".to_owned());
        let health_url = join_health_url(base_url, &health_path).ok_or_else(|| {
            ExtensionHostError::ConnectorRuntimeBaseUrlInvalid {
                instance_id: load_plan.instance_id.clone(),
                base_url: base_url.to_owned(),
            }
        })?;

        Ok(Self {
            entrypoint,
            args,
            environment,
            working_directory,
            health_url,
            startup_timeout: Duration::from_millis(config_u64(
                &load_plan.config,
                "startup_timeout_ms",
                5_000,
            )),
            startup_poll_interval: Duration::from_millis(config_u64(
                &load_plan.config,
                "startup_poll_interval_ms",
                50,
            )),
        })
    }
}

fn connector_process_registry() -> Result<
    std::sync::MutexGuard<'static, HashMap<String, ManagedConnectorProcess>>,
    ExtensionHostError,
> {
    CONNECTOR_PROCESS_REGISTRY
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .map_err(|_| ExtensionHostError::ConnectorRuntimeStatePoisoned)
}

fn running_connector_status(
    instance_id: &str,
) -> Result<Option<ConnectorRuntimeStatus>, ExtensionHostError> {
    let mut registry = connector_process_registry()?;
    let mut should_remove = false;
    let status = match registry.get_mut(instance_id) {
        Some(process) => match process.child.try_wait() {
            Ok(None) => Some(ConnectorRuntimeStatus {
                instance_id: instance_id.to_owned(),
                base_url: process.base_url.clone(),
                health_url: process.health_url.clone(),
                process_id: Some(process.child.id()),
                running: true,
                healthy: false,
            }),
            Ok(Some(_)) => {
                should_remove = true;
                None
            }
            Err(error) => {
                return Err(ExtensionHostError::ConnectorRuntimeShutdownFailed {
                    instance_id: instance_id.to_owned(),
                    message: error.to_string(),
                })
            }
        },
        None => None,
    };

    if should_remove {
        registry.remove(instance_id);
    }

    Ok(status)
}

fn wait_for_connector_health(
    instance_id: &str,
    health_url: &str,
    timeout: Duration,
    poll_interval: Duration,
) -> Result<(), ExtensionHostError> {
    let start = Instant::now();
    loop {
        if connector_process_exited(instance_id)? {
            return Err(ExtensionHostError::ConnectorRuntimeExited {
                instance_id: instance_id.to_owned(),
                status: None,
            });
        }

        if probe_http_health(health_url)? {
            return Ok(());
        }

        if start.elapsed() >= timeout {
            return Err(ExtensionHostError::ConnectorRuntimeHealthTimedOut {
                instance_id: instance_id.to_owned(),
                health_url: health_url.to_owned(),
                timeout_ms: timeout.as_millis() as u64,
            });
        }

        std::thread::sleep(poll_interval);
    }
}

fn connector_process_exited(instance_id: &str) -> Result<bool, ExtensionHostError> {
    let mut registry = connector_process_registry()?;
    let mut should_remove = false;
    let exited = match registry.get_mut(instance_id) {
        Some(process) => match process.child.try_wait() {
            Ok(Some(_)) => {
                should_remove = true;
                true
            }
            Ok(None) => false,
            Err(error) => {
                return Err(ExtensionHostError::ConnectorRuntimeShutdownFailed {
                    instance_id: instance_id.to_owned(),
                    message: error.to_string(),
                })
            }
        },
        None => true,
    };

    if should_remove {
        registry.remove(instance_id);
    }

    Ok(exited)
}

fn kill_child(instance_id: &str, child: &mut Child) -> Result<(), ExtensionHostError> {
    match child.try_wait() {
        Ok(Some(_)) => Ok(()),
        Ok(None) => {
            child
                .kill()
                .map_err(|error| ExtensionHostError::ConnectorRuntimeShutdownFailed {
                    instance_id: instance_id.to_owned(),
                    message: error.to_string(),
                })?;
            let _ = child.wait();
            Ok(())
        }
        Err(error) => Err(ExtensionHostError::ConnectorRuntimeShutdownFailed {
            instance_id: instance_id.to_owned(),
            message: error.to_string(),
        }),
    }
}

fn resolve_entrypoint(entrypoint: &str, package_root: Option<&Path>) -> PathBuf {
    let path = PathBuf::from(entrypoint);
    if path.is_absolute() {
        return path;
    }

    if entrypoint.contains('\\') || entrypoint.contains('/') || entrypoint.starts_with('.') {
        if let Some(package_root) = package_root {
            return package_root.join(path);
        }
    }

    path
}

fn config_string(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn config_string_array(value: &Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn config_string_map(value: &Value, key: &str) -> HashMap<String, String> {
    value
        .get(key)
        .and_then(Value::as_object)
        .map(|items| {
            items
                .iter()
                .filter_map(|(key, value)| {
                    value.as_str().map(|value| (key.clone(), value.to_owned()))
                })
                .collect::<HashMap<_, _>>()
        })
        .unwrap_or_default()
}

fn config_u64(value: &Value, key: &str, default: u64) -> u64 {
    value.get(key).and_then(Value::as_u64).unwrap_or(default)
}

fn config_path(value: &Value, key: &str, package_root: Option<&Path>) -> Option<PathBuf> {
    let raw = config_string(value, key)?;
    let path = PathBuf::from(&raw);
    if path.is_absolute() {
        Some(path)
    } else {
        package_root.map(|package_root| package_root.join(path))
    }
}

fn join_health_url(base_url: &str, health_path: &str) -> Option<String> {
    let normalized_path = if health_path.starts_with('/') {
        health_path.to_owned()
    } else {
        format!("/{health_path}")
    };
    let base_url = base_url.trim_end_matches('/');
    if base_url.starts_with("http://") {
        Some(format!("{base_url}{normalized_path}"))
    } else {
        None
    }
}

fn probe_http_health(health_url: &str) -> Result<bool, ExtensionHostError> {
    let (address, path) = parse_http_health_url(health_url)?;
    let mut stream = match TcpStream::connect_timeout(&address, Duration::from_millis(250)) {
        Ok(stream) => stream,
        Err(_) => return Ok(false),
    };
    let _ = stream.set_read_timeout(Some(Duration::from_millis(250)));
    let _ = stream.set_write_timeout(Some(Duration::from_millis(250)));

    let request = format!(
        "GET {path} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        address
    );
    if stream.write_all(request.as_bytes()).is_err() {
        return Ok(false);
    }

    let mut response = Vec::new();
    let mut buffer = [0_u8; 1024];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(bytes_read) => {
                response.extend_from_slice(&buffer[..bytes_read]);
                if response.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }
            Err(_) => return Ok(false),
        }
    }

    let response = String::from_utf8_lossy(&response);
    Ok(response.starts_with("HTTP/1.1 200")
        || response.starts_with("HTTP/1.0 200")
        || response.starts_with("HTTP/2 200"))
}

fn parse_http_health_url(health_url: &str) -> Result<(SocketAddr, String), ExtensionHostError> {
    let without_scheme = health_url.strip_prefix("http://").ok_or_else(|| {
        ExtensionHostError::ConnectorRuntimeBaseUrlInvalid {
            instance_id: String::new(),
            base_url: health_url.to_owned(),
        }
    })?;
    let (host_port, path) = match without_scheme.split_once('/') {
        Some((host_port, path)) => (host_port, format!("/{path}")),
        None => (without_scheme, "/".to_owned()),
    };
    let (host, port) = match host_port.rsplit_once(':') {
        Some((host, port)) => (
            host,
            port.parse::<u16>().map_err(|_| {
                ExtensionHostError::ConnectorRuntimeBaseUrlInvalid {
                    instance_id: String::new(),
                    base_url: health_url.to_owned(),
                }
            })?,
        ),
        None => (host_port, 80),
    };

    let address = (host, port)
        .to_socket_addrs()
        .map_err(|_| ExtensionHostError::ConnectorRuntimeBaseUrlInvalid {
            instance_id: String::new(),
            base_url: health_url.to_owned(),
        })?
        .next()
        .ok_or_else(|| ExtensionHostError::ConnectorRuntimeBaseUrlInvalid {
            instance_id: String::new(),
            base_url: health_url.to_owned(),
        })?;

    Ok((address, path))
}
