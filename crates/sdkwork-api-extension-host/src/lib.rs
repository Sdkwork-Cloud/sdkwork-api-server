use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::os::raw::{c_char, c_void};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::time::{Duration, Instant};

use anyhow::{anyhow, Result as AnyhowResult};
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use bytes::Bytes;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use futures_util::stream;
use futures_util::StreamExt;
use libloading::Library;
use sdkwork_api_extension_abi::{
    free_raw_c_string, from_raw_c_str, into_raw_c_string, ExtensionHealthCheckResult,
    ExtensionLifecycleContext, ExtensionLifecycleResult, ProviderInvocation,
    ProviderInvocationResult, ProviderStreamInvocationResult, ProviderStreamWriter,
    SDKWORK_EXTENSION_ABI_VERSION, SDKWORK_EXTENSION_ABI_VERSION_SYMBOL,
    SDKWORK_EXTENSION_FREE_STRING_SYMBOL, SDKWORK_EXTENSION_HEALTH_CHECK_JSON_SYMBOL,
    SDKWORK_EXTENSION_INIT_JSON_SYMBOL, SDKWORK_EXTENSION_MANIFEST_JSON_SYMBOL,
    SDKWORK_EXTENSION_PROVIDER_EXECUTE_JSON_SYMBOL,
    SDKWORK_EXTENSION_PROVIDER_EXECUTE_STREAM_JSON_SYMBOL, SDKWORK_EXTENSION_SHUTDOWN_JSON_SYMBOL,
};
use sdkwork_api_extension_core::{
    ExtensionInstallation, ExtensionInstance, ExtensionManifest, ExtensionRuntime,
    ExtensionSignatureAlgorithm,
};
use sdkwork_api_provider_core::{
    ProviderAdapter, ProviderExecutionAdapter, ProviderOutput, ProviderRequest,
    ProviderRequestOptions, ProviderStreamOutput,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio_stream::wrappers::UnboundedReceiverStream;

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

type AbiVersionFn = unsafe extern "C" fn() -> u32;
type ManifestJsonFn = unsafe extern "C" fn() -> *const c_char;
type ExecuteJsonFn = unsafe extern "C" fn(*const c_char) -> *mut c_char;
type ExecuteStreamJsonFn =
    unsafe extern "C" fn(*const c_char, *const ProviderStreamWriter) -> *mut c_char;
type LifecycleJsonFn = unsafe extern "C" fn(*const c_char) -> *mut c_char;
type FreeStringFn = unsafe extern "C" fn(*mut c_char);
const SDKWORK_NATIVE_DYNAMIC_SHUTDOWN_DRAIN_TIMEOUT_MS: &str =
    "SDKWORK_NATIVE_DYNAMIC_SHUTDOWN_DRAIN_TIMEOUT_MS";

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

fn provider_runtime_aliases(adapter_kind: &str) -> &'static [&'static str] {
    match adapter_kind {
        "openai" => &["openai", "openai-compatible", "custom-openai"],
        "openrouter" => &["openrouter", "openrouter-compatible"],
        "ollama" => &["ollama", "ollama-compatible"],
        _ => &[],
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
    pub extension_id: String,
    pub display_name: String,
    pub base_url: String,
    pub health_url: String,
    pub process_id: Option<u32>,
    pub running: bool,
    pub healthy: bool,
}

#[derive(Debug)]
struct ManagedConnectorProcess {
    child: Child,
    extension_id: String,
    display_name: String,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeDynamicRuntimeStatus {
    pub extension_id: String,
    pub display_name: String,
    pub library_path: String,
    pub running: bool,
    pub healthy: bool,
    pub supports_health_check: bool,
    pub supports_shutdown: bool,
    pub message: Option<String>,
}

struct NativeDynamicRuntime {
    entrypoint: String,
    manifest: ExtensionManifest,
    _library: Library,
    execute_json: ExecuteJsonFn,
    execute_stream_json: Option<ExecuteStreamJsonFn>,
    init_json: Option<LifecycleJsonFn>,
    health_check_json: Option<LifecycleJsonFn>,
    shutdown_json: Option<LifecycleJsonFn>,
    free_string: FreeStringFn,
    lifecycle_state: Mutex<NativeDynamicLifecycleState>,
    lifecycle_drained: Condvar,
}

#[derive(Debug)]
struct NativeDynamicLifecycleState {
    running: bool,
    healthy: bool,
    message: Option<String>,
    shutdown_invoked: bool,
    draining: bool,
    active_invocations: usize,
}

unsafe impl Send for NativeDynamicRuntime {}
unsafe impl Sync for NativeDynamicRuntime {}

struct NativeDynamicProviderAdapter {
    runtime: Arc<NativeDynamicRuntime>,
    base_url: String,
}

struct HostStreamWriterContext {
    sender: UnboundedSender<NativeDynamicStreamEvent>,
}

enum NativeDynamicStreamEvent {
    ContentType(String),
    Chunk(Bytes),
    Finished(ProviderStreamInvocationResult),
}

static CONNECTOR_PROCESS_REGISTRY: OnceLock<Mutex<HashMap<String, ManagedConnectorProcess>>> =
    OnceLock::new();
static NATIVE_DYNAMIC_RUNTIME_REGISTRY: OnceLock<
    Mutex<HashMap<String, Arc<NativeDynamicRuntime>>>,
> = OnceLock::new();

impl NativeDynamicRuntime {
    fn lifecycle_context(&self) -> ExtensionLifecycleContext {
        ExtensionLifecycleContext::new(self.manifest.id.clone(), self.entrypoint.clone())
    }

    fn lock_state(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, NativeDynamicLifecycleState>, ExtensionHostError> {
        self.lifecycle_state.lock().map_err(|_| {
            ExtensionHostError::NativeDynamicRuntimeStatePoisoned {
                entrypoint: self.entrypoint.clone(),
            }
        })
    }

    fn initialize(&self) -> Result<(), ExtensionHostError> {
        let message = if let Some(init_json) = self.init_json {
            let result: ExtensionLifecycleResult = self.invoke_lifecycle_json(init_json, "init")?;
            if !result.success {
                return Err(ExtensionHostError::NativeDynamicLifecycleFailed {
                    entrypoint: self.entrypoint.clone(),
                    phase: "init".to_owned(),
                    message: result
                        .message
                        .unwrap_or_else(|| "plugin reported init failure".to_owned()),
                });
            }
            result.message
        } else {
            Some("native dynamic runtime loaded".to_owned())
        };

        let mut state = self.lock_state()?;
        state.running = true;
        state.healthy = true;
        state.message = message;
        Ok(())
    }

    fn begin_invocation(&self) -> Result<(), ExtensionHostError> {
        let mut state = self.lock_state()?;
        if !state.running {
            return Err(ExtensionHostError::NativeDynamicInvocationFailed {
                entrypoint: self.entrypoint.clone(),
                message: "native dynamic runtime is not running".to_owned(),
            });
        }
        if state.draining || state.shutdown_invoked {
            return Err(ExtensionHostError::NativeDynamicInvocationFailed {
                entrypoint: self.entrypoint.clone(),
                message: "native dynamic runtime is draining for shutdown".to_owned(),
            });
        }
        state.active_invocations += 1;
        Ok(())
    }

    fn finish_invocation(&self) {
        let mut state = self
            .lifecycle_state
            .lock()
            .expect("native dynamic runtime state lock");
        if state.active_invocations > 0 {
            state.active_invocations -= 1;
        }
        if state.active_invocations == 0 {
            self.lifecycle_drained.notify_all();
        }
    }

    fn invocation_guard(&self) -> Result<NativeDynamicInvocationGuard<'_>, ExtensionHostError> {
        self.begin_invocation()?;
        Ok(NativeDynamicInvocationGuard {
            runtime: self,
            finished: false,
        })
    }

    fn ensure_running(&self) -> Result<(), ExtensionHostError> {
        if self.lock_state()?.running {
            Ok(())
        } else {
            Err(ExtensionHostError::NativeDynamicInvocationFailed {
                entrypoint: self.entrypoint.clone(),
                message: "native dynamic runtime is not running".to_owned(),
            })
        }
    }

    fn status(&self) -> Result<NativeDynamicRuntimeStatus, ExtensionHostError> {
        let (running, mut healthy, mut message) = {
            let state = self.lock_state()?;
            (state.running, state.healthy, state.message.clone())
        };

        if running && self.health_check_json.is_some() {
            match self.health_check() {
                Ok(result) => {
                    healthy = result.healthy;
                    message = result.message.clone();
                    let mut state = self.lock_state()?;
                    state.healthy = result.healthy;
                    state.message = result.message.clone();
                }
                Err(error) => {
                    healthy = false;
                    message = Some(error.to_string());
                    let mut state = self.lock_state()?;
                    state.healthy = false;
                    state.message = message.clone();
                }
            }
        }

        Ok(NativeDynamicRuntimeStatus {
            extension_id: self.manifest.id.clone(),
            display_name: self.manifest.display_name.clone(),
            library_path: self.entrypoint.clone(),
            running,
            healthy,
            supports_health_check: self.health_check_json.is_some(),
            supports_shutdown: self.shutdown_json.is_some(),
            message,
        })
    }

    fn health_check(&self) -> Result<ExtensionHealthCheckResult, ExtensionHostError> {
        let Some(health_check_json) = self.health_check_json else {
            let state = self.lock_state()?;
            return Ok(ExtensionHealthCheckResult {
                healthy: state.healthy,
                message: state.message.clone(),
                details: None,
            });
        };

        let _guard = self.invocation_guard()?;
        self.invoke_lifecycle_json(health_check_json, "health_check")
    }

    fn shutdown(&self, drain_timeout_ms: Option<u64>) -> Result<(), ExtensionHostError> {
        let mut state = self.lock_state()?;
        if state.shutdown_invoked {
            return Ok(());
        }
        state.draining = true;
        if let Some(timeout_ms) = drain_timeout_ms {
            let timeout = Duration::from_millis(timeout_ms);
            let started_at = Instant::now();
            while state.active_invocations > 0 {
                let Some(remaining) = timeout.checked_sub(started_at.elapsed()) else {
                    state.draining = false;
                    return Err(ExtensionHostError::NativeDynamicShutdownDrainTimedOut {
                        entrypoint: self.entrypoint.clone(),
                        timeout_ms,
                    });
                };
                let (next_state, wait_result) = self
                    .lifecycle_drained
                    .wait_timeout(state, remaining)
                    .map_err(|_| ExtensionHostError::NativeDynamicRuntimeStatePoisoned {
                        entrypoint: self.entrypoint.clone(),
                    })?;
                state = next_state;
                if wait_result.timed_out() && state.active_invocations > 0 {
                    state.draining = false;
                    return Err(ExtensionHostError::NativeDynamicShutdownDrainTimedOut {
                        entrypoint: self.entrypoint.clone(),
                        timeout_ms,
                    });
                }
            }
        } else {
            while state.active_invocations > 0 {
                state = self.lifecycle_drained.wait(state).map_err(|_| {
                    ExtensionHostError::NativeDynamicRuntimeStatePoisoned {
                        entrypoint: self.entrypoint.clone(),
                    }
                })?;
            }
        }
        drop(state);

        let message = if let Some(shutdown_json) = self.shutdown_json {
            let result: ExtensionLifecycleResult =
                self.invoke_lifecycle_json(shutdown_json, "shutdown")?;
            if !result.success {
                return Err(ExtensionHostError::NativeDynamicLifecycleFailed {
                    entrypoint: self.entrypoint.clone(),
                    phase: "shutdown".to_owned(),
                    message: result
                        .message
                        .unwrap_or_else(|| "plugin reported shutdown failure".to_owned()),
                });
            }
            result.message
        } else {
            Some("native dynamic runtime stopped".to_owned())
        };

        let mut state = self.lock_state()?;
        state.running = false;
        state.healthy = false;
        state.shutdown_invoked = true;
        state.draining = false;
        state.message = message;
        Ok(())
    }

    fn invoke_lifecycle_json<T: DeserializeOwned>(
        &self,
        callback: LifecycleJsonFn,
        phase: &str,
    ) -> Result<T, ExtensionHostError> {
        let payload = serde_json::to_string(&self.lifecycle_context()).map_err(|error| {
            ExtensionHostError::NativeDynamicLifecycleFailed {
                entrypoint: self.entrypoint.clone(),
                phase: phase.to_owned(),
                message: error.to_string(),
            }
        })?;
        let raw_payload = into_raw_c_string(payload);
        let raw_result = unsafe { callback(raw_payload.cast_const()) };
        unsafe { free_raw_c_string(raw_payload) };

        let Some(raw_result_json) = (unsafe { from_raw_c_str(raw_result) }) else {
            return Err(ExtensionHostError::NativeDynamicLifecycleFailed {
                entrypoint: self.entrypoint.clone(),
                phase: phase.to_owned(),
                message: "plugin returned null lifecycle result".to_owned(),
            });
        };
        unsafe { (self.free_string)(raw_result) };

        serde_json::from_str(&raw_result_json).map_err(|error| {
            ExtensionHostError::NativeDynamicLifecycleFailed {
                entrypoint: self.entrypoint.clone(),
                phase: phase.to_owned(),
                message: error.to_string(),
            }
        })
    }
}

struct NativeDynamicInvocationGuard<'a> {
    runtime: &'a NativeDynamicRuntime,
    finished: bool,
}

impl Drop for NativeDynamicInvocationGuard<'_> {
    fn drop(&mut self) {
        if !self.finished {
            self.runtime.finish_invocation();
            self.finished = true;
        }
    }
}

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

pub fn load_native_dynamic_library_manifest(
    entrypoint: &Path,
) -> Result<ExtensionManifest, ExtensionHostError> {
    let (_, manifest) = load_native_dynamic_runtime(entrypoint)?;
    Ok(manifest)
}

pub fn load_native_dynamic_provider_adapter(
    entrypoint: &Path,
    base_url: impl Into<String>,
) -> Result<Box<dyn ProviderExecutionAdapter>, ExtensionHostError> {
    let (runtime, _) = load_or_reuse_native_dynamic_runtime(entrypoint)?;
    Ok(Box::new(NativeDynamicProviderAdapter {
        runtime,
        base_url: base_url.into(),
    }))
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
            extension_id: load_plan.extension_id.clone(),
            display_name: load_plan.display_name.clone(),
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
                extension_id: load_plan.extension_id.clone(),
                display_name: load_plan.display_name.clone(),
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
        extension_id: load_plan.extension_id.clone(),
        display_name: load_plan.display_name.clone(),
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

pub fn shutdown_connector_runtimes_for_extension(
    extension_id: &str,
) -> Result<(), ExtensionHostError> {
    let processes = {
        let mut registry = connector_process_registry()?;
        let instance_ids = registry
            .iter()
            .filter(|(_, process)| process.extension_id == extension_id)
            .map(|(instance_id, _)| instance_id.clone())
            .collect::<Vec<_>>();
        instance_ids
            .into_iter()
            .filter_map(|instance_id| {
                registry
                    .remove(&instance_id)
                    .map(|process| (instance_id, process))
            })
            .collect::<Vec<_>>()
    };

    for (instance_id, mut process) in processes {
        kill_child(&instance_id, &mut process.child)?;
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
                    process.extension_id.clone(),
                    process.display_name.clone(),
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
        .map(
            |(instance_id, extension_id, display_name, base_url, health_url, process_id)| {
                Ok(ConnectorRuntimeStatus {
                    instance_id,
                    extension_id,
                    display_name,
                    base_url,
                    healthy: probe_http_health(&health_url)?,
                    health_url,
                    process_id,
                    running: true,
                })
            },
        )
        .collect::<Result<Vec<_>, ExtensionHostError>>()?;
    statuses.sort_by(|left, right| left.instance_id.cmp(&right.instance_id));
    Ok(statuses)
}

pub fn list_native_dynamic_runtime_statuses(
) -> Result<Vec<NativeDynamicRuntimeStatus>, ExtensionHostError> {
    let runtimes = {
        let registry = native_dynamic_runtime_registry()?;
        registry.values().cloned().collect::<Vec<_>>()
    };

    let mut statuses = runtimes
        .into_iter()
        .map(|runtime| runtime.status())
        .collect::<Result<Vec<_>, ExtensionHostError>>()?;
    statuses.sort_by(|left, right| left.extension_id.cmp(&right.extension_id));
    Ok(statuses)
}

pub fn shutdown_all_native_dynamic_runtimes() -> Result<(), ExtensionHostError> {
    let drain_timeout_ms = configured_native_dynamic_shutdown_drain_timeout_ms()?;
    let runtimes = {
        let mut registry = native_dynamic_runtime_registry()?;
        registry.drain().collect::<Vec<_>>()
    };

    shutdown_native_dynamic_runtime_entries(runtimes, drain_timeout_ms)
}

pub fn shutdown_native_dynamic_runtimes_for_extension(
    extension_id: &str,
) -> Result<(), ExtensionHostError> {
    let drain_timeout_ms = configured_native_dynamic_shutdown_drain_timeout_ms()?;
    let runtimes = {
        let mut registry = native_dynamic_runtime_registry()?;
        let entrypoints = registry
            .iter()
            .filter(|(_, runtime)| runtime.manifest.id == extension_id)
            .map(|(entrypoint, _)| entrypoint.clone())
            .collect::<Vec<_>>();
        entrypoints
            .into_iter()
            .filter_map(|entrypoint| {
                registry
                    .remove(&entrypoint)
                    .map(|runtime| (entrypoint, runtime))
            })
            .collect::<Vec<_>>()
    };

    shutdown_native_dynamic_runtime_entries(runtimes, drain_timeout_ms)
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
        let aliases = provider_runtime_aliases(&factory.adapter_kind)
            .iter()
            .map(|alias| (*alias).to_owned())
            .collect::<Vec<_>>();
        self.register_builtin_manifest(factory.manifest);
        self.provider_factories
            .insert(extension_id.clone(), factory.factory);
        self.provider_aliases
            .insert(factory.adapter_kind, extension_id.clone());
        for alias in aliases {
            self.provider_aliases.insert(alias, extension_id.clone());
        }
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
        let adapter_kind = adapter_kind.into();
        let aliases = provider_runtime_aliases(&adapter_kind)
            .iter()
            .map(|alias| (*alias).to_owned())
            .collect::<Vec<_>>();
        self.package_roots
            .insert(extension_id.clone(), package.root_dir.clone());
        self.manifests
            .insert(extension_id.clone(), package.manifest);
        self.provider_factories
            .insert(extension_id.clone(), Arc::new(factory));
        self.provider_aliases
            .insert(adapter_kind, extension_id.clone());
        for alias in aliases {
            self.provider_aliases.insert(alias, extension_id.clone());
        }
    }

    pub fn register_discovered_native_dynamic_provider(
        &mut self,
        package: DiscoveredExtensionPackage,
    ) -> Result<(), ExtensionHostError> {
        let extension_id = package.manifest.id.clone();
        let entrypoint = package.manifest.entrypoint.as_deref().ok_or(
            ExtensionHostError::ManifestReadFailed {
                path: package.manifest_path.display().to_string(),
                message: "native dynamic extension manifest has no entrypoint".to_owned(),
            },
        )?;
        let library_path = resolve_entrypoint(entrypoint, Some(&package.root_dir));
        let (runtime, library_manifest) = load_or_reuse_native_dynamic_runtime(&library_path)?;
        ensure_native_dynamic_manifest_matches(
            &package.manifest,
            &library_manifest,
            &library_path,
        )?;

        self.package_roots
            .insert(extension_id.clone(), package.root_dir.clone());
        self.manifests
            .insert(extension_id.clone(), package.manifest);
        self.provider_factories.insert(
            extension_id,
            Arc::new(move |base_url| {
                Box::new(NativeDynamicProviderAdapter {
                    runtime: runtime.clone(),
                    base_url,
                })
            }),
        );
        Ok(())
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

impl ProviderAdapter for NativeDynamicProviderAdapter {
    fn id(&self) -> &'static str {
        "native_dynamic"
    }
}

#[async_trait]
impl ProviderExecutionAdapter for NativeDynamicProviderAdapter {
    async fn execute(
        &self,
        api_key: &str,
        request: ProviderRequest<'_>,
    ) -> AnyhowResult<ProviderOutput> {
        let invocation = provider_invocation_from_request(request, api_key, &self.base_url)?;
        if invocation.expects_stream {
            let stream =
                execute_native_dynamic_stream_invocation(Arc::clone(&self.runtime), &invocation)
                    .await?;
            return Ok(ProviderOutput::Stream(stream));
        }
        let result = execute_native_dynamic_invocation(&self.runtime, &invocation)?;
        match result {
            ProviderInvocationResult::Json { body } => Ok(ProviderOutput::Json(body)),
            ProviderInvocationResult::Unsupported { message } => Err(anyhow!(
                "{}",
                message.unwrap_or_else(
                    || "native dynamic provider reported unsupported operation".to_owned()
                )
            )),
            ProviderInvocationResult::Error { message } => Err(anyhow!("{message}")),
        }
    }

    async fn execute_with_options(
        &self,
        api_key: &str,
        request: ProviderRequest<'_>,
        options: &ProviderRequestOptions,
    ) -> AnyhowResult<ProviderOutput> {
        let invocation = provider_invocation_from_request_with_options(
            request,
            api_key,
            &self.base_url,
            options,
        )?;
        if invocation.expects_stream {
            let stream =
                execute_native_dynamic_stream_invocation(Arc::clone(&self.runtime), &invocation)
                    .await?;
            return Ok(ProviderOutput::Stream(stream));
        }
        let result = execute_native_dynamic_invocation(&self.runtime, &invocation)?;
        match result {
            ProviderInvocationResult::Json { body } => Ok(ProviderOutput::Json(body)),
            ProviderInvocationResult::Unsupported { message } => Err(anyhow!(
                "{}",
                message.unwrap_or_else(
                    || "native dynamic provider reported unsupported operation".to_owned()
                )
            )),
            ProviderInvocationResult::Error { message } => Err(anyhow!("{message}")),
        }
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
    NativeDynamicLibraryLoadFailed {
        entrypoint: String,
        message: String,
    },
    NativeDynamicSymbolMissing {
        entrypoint: String,
        symbol: String,
        message: String,
    },
    NativeDynamicAbiVersionUnsupported {
        entrypoint: String,
        actual_version: u32,
    },
    NativeDynamicManifestExportMissing {
        entrypoint: String,
    },
    NativeDynamicManifestMismatch {
        entrypoint: String,
        message: String,
    },
    NativeDynamicInvocationSerializeFailed {
        operation: String,
        message: String,
    },
    NativeDynamicInvocationFailed {
        entrypoint: String,
        message: String,
    },
    NativeDynamicResponseParseFailed {
        entrypoint: String,
        message: String,
    },
    NativeDynamicLifecycleFailed {
        entrypoint: String,
        phase: String,
        message: String,
    },
    NativeDynamicShutdownDrainTimedOut {
        entrypoint: String,
        timeout_ms: u64,
    },
    NativeDynamicShutdownDrainTimeoutInvalid {
        value: String,
    },
    NativeDynamicRuntimeStatePoisoned {
        entrypoint: String,
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
            Self::NativeDynamicLibraryLoadFailed {
                entrypoint,
                message,
            } => write!(
                f,
                "failed to load native dynamic extension library {}: {}",
                entrypoint, message
            ),
            Self::NativeDynamicSymbolMissing {
                entrypoint,
                symbol,
                message,
            } => write!(
                f,
                "native dynamic extension library {} is missing symbol {}: {}",
                entrypoint, symbol, message
            ),
            Self::NativeDynamicAbiVersionUnsupported {
                entrypoint,
                actual_version,
            } => write!(
                f,
                "native dynamic extension library {} reported unsupported ABI version {}",
                entrypoint, actual_version
            ),
            Self::NativeDynamicManifestExportMissing { entrypoint } => write!(
                f,
                "native dynamic extension library {} returned no manifest export",
                entrypoint
            ),
            Self::NativeDynamicManifestMismatch {
                entrypoint,
                message,
            } => write!(
                f,
                "native dynamic extension library {} manifest does not match package manifest: {}",
                entrypoint, message
            ),
            Self::NativeDynamicInvocationSerializeFailed { operation, message } => write!(
                f,
                "failed to serialize native dynamic provider invocation {}: {}",
                operation, message
            ),
            Self::NativeDynamicInvocationFailed {
                entrypoint,
                message,
            } => write!(
                f,
                "native dynamic extension library {} failed during invocation: {}",
                entrypoint, message
            ),
            Self::NativeDynamicResponseParseFailed {
                entrypoint,
                message,
            } => write!(
                f,
                "native dynamic extension library {} returned invalid response payload: {}",
                entrypoint, message
            ),
            Self::NativeDynamicLifecycleFailed {
                entrypoint,
                phase,
                message,
            } => write!(
                f,
                "native dynamic extension library {} failed during {}: {}",
                entrypoint, phase, message
            ),
            Self::NativeDynamicShutdownDrainTimedOut {
                entrypoint,
                timeout_ms,
            } => write!(
                f,
                "native dynamic extension library {} drain timed out after {}ms; runtime kept running",
                entrypoint, timeout_ms
            ),
            Self::NativeDynamicShutdownDrainTimeoutInvalid { value } => write!(
                f,
                "native dynamic shutdown drain timeout is invalid: {}",
                value
            ),
            Self::NativeDynamicRuntimeStatePoisoned { entrypoint } => write!(
                f,
                "native dynamic extension library {} runtime state is poisoned",
                entrypoint
            ),
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

fn load_native_dynamic_runtime(
    entrypoint: &Path,
) -> Result<(Arc<NativeDynamicRuntime>, ExtensionManifest), ExtensionHostError> {
    unsafe {
        let library = load_native_dynamic_library(entrypoint)?;
        let abi_version = load_native_dynamic_symbol::<AbiVersionFn>(
            &library,
            SDKWORK_EXTENSION_ABI_VERSION_SYMBOL,
            entrypoint,
        )?;
        let manifest_json = load_native_dynamic_symbol::<ManifestJsonFn>(
            &library,
            SDKWORK_EXTENSION_MANIFEST_JSON_SYMBOL,
            entrypoint,
        )?;
        let execute_json = load_native_dynamic_symbol::<ExecuteJsonFn>(
            &library,
            SDKWORK_EXTENSION_PROVIDER_EXECUTE_JSON_SYMBOL,
            entrypoint,
        )?;
        let execute_stream_json = try_load_native_dynamic_symbol::<ExecuteStreamJsonFn>(
            &library,
            SDKWORK_EXTENSION_PROVIDER_EXECUTE_STREAM_JSON_SYMBOL,
        );
        let init_json = try_load_native_dynamic_symbol::<LifecycleJsonFn>(
            &library,
            SDKWORK_EXTENSION_INIT_JSON_SYMBOL,
        );
        let health_check_json = try_load_native_dynamic_symbol::<LifecycleJsonFn>(
            &library,
            SDKWORK_EXTENSION_HEALTH_CHECK_JSON_SYMBOL,
        );
        let shutdown_json = try_load_native_dynamic_symbol::<LifecycleJsonFn>(
            &library,
            SDKWORK_EXTENSION_SHUTDOWN_JSON_SYMBOL,
        );
        let free_string = load_native_dynamic_symbol::<FreeStringFn>(
            &library,
            SDKWORK_EXTENSION_FREE_STRING_SYMBOL,
            entrypoint,
        )?;

        let actual_version = abi_version();
        if actual_version != SDKWORK_EXTENSION_ABI_VERSION {
            return Err(ExtensionHostError::NativeDynamicAbiVersionUnsupported {
                entrypoint: entrypoint.display().to_string(),
                actual_version,
            });
        }

        let manifest_ptr = manifest_json();
        let Some(manifest_json) = from_raw_c_str(manifest_ptr) else {
            return Err(ExtensionHostError::NativeDynamicManifestExportMissing {
                entrypoint: entrypoint.display().to_string(),
            });
        };
        let manifest: ExtensionManifest =
            serde_json::from_str(&manifest_json).map_err(|error| {
                ExtensionHostError::ManifestParseFailed {
                    path: entrypoint.display().to_string(),
                    message: error.to_string(),
                }
            })?;

        let runtime = Arc::new(NativeDynamicRuntime {
            entrypoint: entrypoint.display().to_string(),
            manifest: manifest.clone(),
            _library: library,
            execute_json,
            execute_stream_json,
            init_json,
            health_check_json,
            shutdown_json,
            free_string,
            lifecycle_state: Mutex::new(NativeDynamicLifecycleState {
                running: true,
                healthy: true,
                message: None,
                shutdown_invoked: false,
                draining: false,
                active_invocations: 0,
            }),
            lifecycle_drained: Condvar::new(),
        });
        runtime.initialize()?;

        let mut registry = native_dynamic_runtime_registry()?;
        registry.insert(runtime.entrypoint.clone(), Arc::clone(&runtime));

        Ok((runtime, manifest))
    }
}

fn load_or_reuse_native_dynamic_runtime(
    entrypoint: &Path,
) -> Result<(Arc<NativeDynamicRuntime>, ExtensionManifest), ExtensionHostError> {
    let entrypoint = entrypoint.display().to_string();
    if let Some(runtime) = native_dynamic_runtime_registry()?.get(&entrypoint).cloned() {
        return Ok((runtime.clone(), runtime.manifest.clone()));
    }

    load_native_dynamic_runtime(Path::new(&entrypoint))
}

fn configured_native_dynamic_shutdown_drain_timeout_ms() -> Result<Option<u64>, ExtensionHostError>
{
    let Some(value) = std::env::var_os(SDKWORK_NATIVE_DYNAMIC_SHUTDOWN_DRAIN_TIMEOUT_MS) else {
        return Ok(None);
    };
    let value = value.to_string_lossy().trim().to_owned();
    if value.is_empty() {
        return Ok(None);
    }
    let timeout_ms = value.parse::<u64>().map_err(|_| {
        ExtensionHostError::NativeDynamicShutdownDrainTimeoutInvalid {
            value: value.clone(),
        }
    })?;
    if timeout_ms == 0 {
        Ok(None)
    } else {
        Ok(Some(timeout_ms))
    }
}

fn shutdown_native_dynamic_runtime_entries(
    runtimes: Vec<(String, Arc<NativeDynamicRuntime>)>,
    drain_timeout_ms: Option<u64>,
) -> Result<(), ExtensionHostError> {
    for (index, (entrypoint, runtime)) in runtimes.iter().enumerate() {
        if let Err(error) = runtime.shutdown(drain_timeout_ms) {
            if matches!(
                error,
                ExtensionHostError::NativeDynamicShutdownDrainTimedOut { .. }
            ) {
                let mut registry = native_dynamic_runtime_registry()?;
                registry.insert(entrypoint.clone(), Arc::clone(runtime));
                for (pending_entrypoint, pending_runtime) in runtimes.iter().skip(index + 1) {
                    registry.insert(pending_entrypoint.clone(), Arc::clone(pending_runtime));
                }
            }
            return Err(error);
        }
    }

    Ok(())
}

#[cfg(windows)]
fn load_native_dynamic_library(entrypoint: &Path) -> Result<Library, ExtensionHostError> {
    static WINDOWS_LIBRARY_LOAD_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let lock = WINDOWS_LIBRARY_LOAD_LOCK.get_or_init(|| Mutex::new(()));
    let _guard = lock.lock().expect("windows library load lock");

    if let Some(parent) = entrypoint.parent() {
        prepend_library_directory_to_path(parent, entrypoint)?;
    }

    unsafe {
        Library::new(entrypoint).map_err(|error| {
            ExtensionHostError::NativeDynamicLibraryLoadFailed {
                entrypoint: entrypoint.display().to_string(),
                message: error.to_string(),
            }
        })
    }
}

#[cfg(not(windows))]
fn load_native_dynamic_library(entrypoint: &Path) -> Result<Library, ExtensionHostError> {
    unsafe {
        Library::new(entrypoint).map_err(|error| {
            ExtensionHostError::NativeDynamicLibraryLoadFailed {
                entrypoint: entrypoint.display().to_string(),
                message: error.to_string(),
            }
        })
    }
}

#[cfg(windows)]
fn prepend_library_directory_to_path(
    directory: &Path,
    entrypoint: &Path,
) -> Result<(), ExtensionHostError> {
    let mut paths = std::env::var_os("PATH")
        .map(|value| std::env::split_paths(&value).collect::<Vec<_>>())
        .unwrap_or_default();

    if paths.iter().any(|path| path == directory) {
        return Ok(());
    }

    paths.insert(0, directory.to_path_buf());
    let joined = std::env::join_paths(paths).map_err(|error| {
        ExtensionHostError::NativeDynamicLibraryLoadFailed {
            entrypoint: entrypoint.display().to_string(),
            message: format!("failed to extend PATH for native dynamic library: {error}"),
        }
    })?;
    std::env::set_var("PATH", joined);
    Ok(())
}

unsafe fn load_native_dynamic_symbol<T: Copy>(
    library: &Library,
    symbol: &[u8],
    entrypoint: &Path,
) -> Result<T, ExtensionHostError> {
    library
        .get::<T>(symbol)
        .map(|loaded| *loaded)
        .map_err(|error| ExtensionHostError::NativeDynamicSymbolMissing {
            entrypoint: entrypoint.display().to_string(),
            symbol: String::from_utf8_lossy(symbol)
                .trim_end_matches('\0')
                .to_owned(),
            message: error.to_string(),
        })
}

unsafe fn try_load_native_dynamic_symbol<T: Copy>(library: &Library, symbol: &[u8]) -> Option<T> {
    library.get::<T>(symbol).ok().map(|loaded| *loaded)
}

fn ensure_native_dynamic_manifest_matches(
    package_manifest: &ExtensionManifest,
    library_manifest: &ExtensionManifest,
    entrypoint: &Path,
) -> Result<(), ExtensionHostError> {
    let same = package_manifest.api_version == library_manifest.api_version
        && package_manifest.id == library_manifest.id
        && package_manifest.kind == library_manifest.kind
        && package_manifest.version == library_manifest.version
        && package_manifest.display_name == library_manifest.display_name
        && package_manifest.runtime == library_manifest.runtime
        && package_manifest.protocol == library_manifest.protocol
        && package_manifest.config_schema == library_manifest.config_schema
        && package_manifest.credential_schema == library_manifest.credential_schema
        && package_manifest.permissions == library_manifest.permissions
        && package_manifest.channel_bindings == library_manifest.channel_bindings
        && package_manifest.capabilities == library_manifest.capabilities;

    if same {
        Ok(())
    } else {
        Err(ExtensionHostError::NativeDynamicManifestMismatch {
            entrypoint: entrypoint.display().to_string(),
            message: format!(
                "package manifest {} does not match library-exported manifest {}",
                package_manifest.id, library_manifest.id
            ),
        })
    }
}

fn execute_native_dynamic_invocation(
    runtime: &NativeDynamicRuntime,
    invocation: &ProviderInvocation,
) -> Result<ProviderInvocationResult, ExtensionHostError> {
    runtime.ensure_running()?;
    let _guard = runtime.invocation_guard()?;
    let payload = serde_json::to_string(invocation).map_err(|error| {
        ExtensionHostError::NativeDynamicInvocationSerializeFailed {
            operation: invocation.operation.clone(),
            message: error.to_string(),
        }
    })?;
    let raw_payload = into_raw_c_string(payload);
    let raw_result = unsafe { (runtime.execute_json)(raw_payload.cast_const()) };
    unsafe { free_raw_c_string(raw_payload) };
    let Some(raw_result_json) = (unsafe { from_raw_c_str(raw_result) }) else {
        return Err(ExtensionHostError::NativeDynamicResponseParseFailed {
            entrypoint: runtime.entrypoint.clone(),
            message: "plugin returned null result".to_owned(),
        });
    };
    unsafe { (runtime.free_string)(raw_result) };

    serde_json::from_str(&raw_result_json).map_err(|error| {
        ExtensionHostError::NativeDynamicResponseParseFailed {
            entrypoint: runtime.entrypoint.clone(),
            message: error.to_string(),
        }
    })
}

async fn execute_native_dynamic_stream_invocation(
    runtime: Arc<NativeDynamicRuntime>,
    invocation: &ProviderInvocation,
) -> Result<ProviderStreamOutput, ExtensionHostError> {
    runtime.ensure_running()?;
    let Some(execute_stream_json) = runtime.execute_stream_json else {
        return Err(ExtensionHostError::NativeDynamicInvocationFailed {
            entrypoint: runtime.entrypoint.clone(),
            message: "plugin does not export stream execution".to_owned(),
        });
    };

    let payload = serde_json::to_string(invocation).map_err(|error| {
        ExtensionHostError::NativeDynamicInvocationSerializeFailed {
            operation: invocation.operation.clone(),
            message: error.to_string(),
        }
    })?;
    runtime.begin_invocation()?;

    let (event_sender, mut event_receiver) = mpsc::unbounded_channel();
    let runtime_for_thread = Arc::clone(&runtime);
    std::thread::spawn(move || {
        struct NativeDynamicOwnedInvocationGuard {
            runtime: Arc<NativeDynamicRuntime>,
        }

        impl Drop for NativeDynamicOwnedInvocationGuard {
            fn drop(&mut self) {
                self.runtime.finish_invocation();
            }
        }

        let _guard = NativeDynamicOwnedInvocationGuard {
            runtime: Arc::clone(&runtime_for_thread),
        };
        let raw_payload = into_raw_c_string(payload);
        let mut writer_context = Box::new(HostStreamWriterContext {
            sender: event_sender.clone(),
        });
        let writer = ProviderStreamWriter {
            context: (&mut *writer_context as *mut HostStreamWriterContext).cast::<c_void>(),
            set_content_type: Some(host_stream_writer_set_content_type),
            write_chunk: Some(host_stream_writer_write_chunk),
        };

        let raw_result =
            unsafe { execute_stream_json(raw_payload.cast_const(), &writer as *const _) };
        unsafe { free_raw_c_string(raw_payload) };

        let result = if raw_result.is_null() {
            ProviderStreamInvocationResult::error("plugin returned null stream result")
        } else {
            let decoded = unsafe { from_raw_c_str(raw_result) };
            unsafe { (runtime_for_thread.free_string)(raw_result) };
            match decoded {
                Some(raw_result_json) => {
                    serde_json::from_str(&raw_result_json).unwrap_or_else(|error| {
                        ProviderStreamInvocationResult::error(format!(
                            "invalid stream result payload: {error}"
                        ))
                    })
                }
                None => {
                    ProviderStreamInvocationResult::error("plugin returned invalid stream result")
                }
            }
        };

        let _ = event_sender.send(NativeDynamicStreamEvent::Finished(result));
    });

    let mut content_type = None;
    let mut prefix_chunk = None;
    match event_receiver.recv().await {
        Some(NativeDynamicStreamEvent::ContentType(value)) => {
            content_type = Some(value);
        }
        Some(NativeDynamicStreamEvent::Chunk(value)) => {
            prefix_chunk = Some(value);
        }
        Some(NativeDynamicStreamEvent::Finished(result)) => {
            return match result {
                ProviderStreamInvocationResult::Streamed { content_type } => {
                    Ok(ProviderStreamOutput::new(content_type, stream::empty()))
                }
                ProviderStreamInvocationResult::Unsupported { message } => {
                    Err(ExtensionHostError::NativeDynamicInvocationFailed {
                        entrypoint: runtime.entrypoint.clone(),
                        message: message.unwrap_or_else(|| {
                            "native dynamic provider reported unsupported stream operation"
                                .to_owned()
                        }),
                    })
                }
                ProviderStreamInvocationResult::Error { message } => {
                    Err(ExtensionHostError::NativeDynamicInvocationFailed {
                        entrypoint: runtime.entrypoint.clone(),
                        message,
                    })
                }
            };
        }
        None => {
            return Err(ExtensionHostError::NativeDynamicInvocationFailed {
                entrypoint: runtime.entrypoint.clone(),
                message: "plugin closed stream without producing metadata or chunks".to_owned(),
            });
        }
    }

    let content_type = content_type.unwrap_or_else(|| "application/octet-stream".to_owned());
    let trailing_stream = UnboundedReceiverStream::new(event_receiver).filter_map(|event| async {
        match event {
            NativeDynamicStreamEvent::ContentType(_) => None,
            NativeDynamicStreamEvent::Chunk(bytes) => Some(Ok(bytes)),
            NativeDynamicStreamEvent::Finished(ProviderStreamInvocationResult::Streamed {
                ..
            }) => None,
            NativeDynamicStreamEvent::Finished(ProviderStreamInvocationResult::Unsupported {
                message,
            }) => Some(Err(io::Error::other(message.unwrap_or_else(|| {
                "native dynamic provider reported unsupported stream operation".to_owned()
            })))),
            NativeDynamicStreamEvent::Finished(ProviderStreamInvocationResult::Error {
                message,
            }) => Some(Err(io::Error::other(message))),
        }
    });

    let body_stream = stream::iter(prefix_chunk.into_iter().map(Ok)).chain(trailing_stream);

    Ok(ProviderStreamOutput::new(content_type, body_stream))
}

unsafe extern "C" fn host_stream_writer_set_content_type(
    context: *mut c_void,
    content_type: *const c_char,
) -> bool {
    if context.is_null() {
        return false;
    }
    let Some(content_type) = from_raw_c_str(content_type) else {
        return false;
    };
    let context = &*(context.cast::<HostStreamWriterContext>());
    context
        .sender
        .send(NativeDynamicStreamEvent::ContentType(content_type))
        .is_ok()
}

unsafe extern "C" fn host_stream_writer_write_chunk(
    context: *mut c_void,
    chunk_ptr: *const u8,
    chunk_len: usize,
) -> bool {
    if context.is_null() || chunk_ptr.is_null() {
        return false;
    }
    let chunk = std::slice::from_raw_parts(chunk_ptr, chunk_len);
    let context = &*(context.cast::<HostStreamWriterContext>());
    context
        .sender
        .send(NativeDynamicStreamEvent::Chunk(Bytes::copy_from_slice(
            chunk,
        )))
        .is_ok()
}

fn serialize_json_body<T: Serialize>(
    body: &T,
    operation: &str,
) -> Result<Value, ExtensionHostError> {
    serde_json::to_value(body).map_err(|error| {
        ExtensionHostError::NativeDynamicInvocationSerializeFailed {
            operation: operation.to_owned(),
            message: error.to_string(),
        }
    })
}

fn provider_invocation_from_request(
    request: ProviderRequest<'_>,
    api_key: &str,
    base_url: &str,
) -> Result<ProviderInvocation, ExtensionHostError> {
    let options = ProviderRequestOptions::default();
    provider_invocation_from_request_with_options(request, api_key, base_url, &options)
}

fn provider_invocation_from_request_with_options(
    request: ProviderRequest<'_>,
    api_key: &str,
    base_url: &str,
    options: &ProviderRequestOptions,
) -> Result<ProviderInvocation, ExtensionHostError> {
    macro_rules! invocation_with_body {
        ($operation:expr, [$($param:expr),*], $body:expr, $expects_stream:expr) => {
            ProviderInvocation::new(
                $operation,
                api_key,
                base_url,
                vec![$($param.to_owned()),*],
                serialize_json_body($body, $operation)?,
                $expects_stream,
            )
            .with_headers(options.headers().clone())
        };
    }

    macro_rules! invocation_without_body {
        ($operation:expr, [$($param:expr),*], $expects_stream:expr) => {
            ProviderInvocation::new(
                $operation,
                api_key,
                base_url,
                vec![$($param.to_owned()),*],
                Value::Null,
                $expects_stream,
            )
            .with_headers(options.headers().clone())
        };
    }

    Ok(match request {
        ProviderRequest::ModelsList => invocation_without_body!("models.list", [], false),
        ProviderRequest::ModelsRetrieve(model_id) => {
            invocation_without_body!("models.retrieve", [model_id], false)
        }
        ProviderRequest::ChatCompletions(body) => {
            invocation_with_body!("chat.completions.create", [], body, false)
        }
        ProviderRequest::ChatCompletionsStream(body) => {
            invocation_with_body!("chat.completions.create", [], body, true)
        }
        ProviderRequest::ChatCompletionsList => {
            invocation_without_body!("chat.completions.list", [], false)
        }
        ProviderRequest::ChatCompletionsRetrieve(completion_id) => {
            invocation_without_body!("chat.completions.retrieve", [completion_id], false)
        }
        ProviderRequest::ChatCompletionsUpdate(completion_id, body) => {
            invocation_with_body!("chat.completions.update", [completion_id], body, false)
        }
        ProviderRequest::ChatCompletionsDelete(completion_id) => {
            invocation_without_body!("chat.completions.delete", [completion_id], false)
        }
        ProviderRequest::ChatCompletionsMessagesList(completion_id) => {
            invocation_without_body!("chat.completions.messages.list", [completion_id], false)
        }
        ProviderRequest::Completions(body) => {
            invocation_with_body!("completions.create", [], body, false)
        }
        ProviderRequest::ModelsDelete(model_id) => {
            invocation_without_body!("models.delete", [model_id], false)
        }
        ProviderRequest::Threads(body) => invocation_with_body!("threads.create", [], body, false),
        ProviderRequest::ThreadsRetrieve(thread_id) => {
            invocation_without_body!("threads.retrieve", [thread_id], false)
        }
        ProviderRequest::ThreadsUpdate(thread_id, body) => {
            invocation_with_body!("threads.update", [thread_id], body, false)
        }
        ProviderRequest::ThreadsDelete(thread_id) => {
            invocation_without_body!("threads.delete", [thread_id], false)
        }
        ProviderRequest::ThreadMessages(thread_id, body) => {
            invocation_with_body!("threads.messages.create", [thread_id], body, false)
        }
        ProviderRequest::ThreadMessagesList(thread_id) => {
            invocation_without_body!("threads.messages.list", [thread_id], false)
        }
        ProviderRequest::ThreadMessagesRetrieve(thread_id, message_id) => {
            invocation_without_body!("threads.messages.retrieve", [thread_id, message_id], false)
        }
        ProviderRequest::ThreadMessagesUpdate(thread_id, message_id, body) => {
            invocation_with_body!(
                "threads.messages.update",
                [thread_id, message_id],
                body,
                false
            )
        }
        ProviderRequest::ThreadMessagesDelete(thread_id, message_id) => {
            invocation_without_body!("threads.messages.delete", [thread_id, message_id], false)
        }
        ProviderRequest::ThreadRuns(thread_id, body) => {
            invocation_with_body!("threads.runs.create", [thread_id], body, false)
        }
        ProviderRequest::ThreadRunsList(thread_id) => {
            invocation_without_body!("threads.runs.list", [thread_id], false)
        }
        ProviderRequest::ThreadRunsRetrieve(thread_id, run_id) => {
            invocation_without_body!("threads.runs.retrieve", [thread_id, run_id], false)
        }
        ProviderRequest::ThreadRunsUpdate(thread_id, run_id, body) => {
            invocation_with_body!("threads.runs.update", [thread_id, run_id], body, false)
        }
        ProviderRequest::ThreadRunsCancel(thread_id, run_id) => {
            invocation_without_body!("threads.runs.cancel", [thread_id, run_id], false)
        }
        ProviderRequest::ThreadRunsSubmitToolOutputs(thread_id, run_id, body) => {
            invocation_with_body!(
                "threads.runs.submit_tool_outputs",
                [thread_id, run_id],
                body,
                false
            )
        }
        ProviderRequest::ThreadRunStepsList(thread_id, run_id) => {
            invocation_without_body!("threads.runs.steps.list", [thread_id, run_id], false)
        }
        ProviderRequest::ThreadRunStepsRetrieve(thread_id, run_id, step_id) => {
            invocation_without_body!(
                "threads.runs.steps.retrieve",
                [thread_id, run_id, step_id],
                false
            )
        }
        ProviderRequest::ThreadsRuns(body) => {
            invocation_with_body!("threads.runs.create_on_thread", [], body, false)
        }
        ProviderRequest::Conversations(body) => {
            invocation_with_body!("conversations.create", [], body, false)
        }
        ProviderRequest::ConversationsList => {
            invocation_without_body!("conversations.list", [], false)
        }
        ProviderRequest::ConversationsRetrieve(conversation_id) => {
            invocation_without_body!("conversations.retrieve", [conversation_id], false)
        }
        ProviderRequest::ConversationsUpdate(conversation_id, body) => {
            invocation_with_body!("conversations.update", [conversation_id], body, false)
        }
        ProviderRequest::ConversationsDelete(conversation_id) => {
            invocation_without_body!("conversations.delete", [conversation_id], false)
        }
        ProviderRequest::ConversationItems(conversation_id, body) => {
            invocation_with_body!("conversations.items.create", [conversation_id], body, false)
        }
        ProviderRequest::ConversationItemsList(conversation_id) => {
            invocation_without_body!("conversations.items.list", [conversation_id], false)
        }
        ProviderRequest::ConversationItemsRetrieve(conversation_id, item_id) => {
            invocation_without_body!(
                "conversations.items.retrieve",
                [conversation_id, item_id],
                false
            )
        }
        ProviderRequest::ConversationItemsDelete(conversation_id, item_id) => {
            invocation_without_body!(
                "conversations.items.delete",
                [conversation_id, item_id],
                false
            )
        }
        ProviderRequest::Responses(body) => {
            invocation_with_body!("responses.create", [], body, false)
        }
        ProviderRequest::ResponsesStream(body) => {
            invocation_with_body!("responses.create", [], body, true)
        }
        ProviderRequest::ResponsesInputTokens(body) => {
            invocation_with_body!("responses.input_tokens.count", [], body, false)
        }
        ProviderRequest::ResponsesRetrieve(response_id) => {
            invocation_without_body!("responses.retrieve", [response_id], false)
        }
        ProviderRequest::ResponsesDelete(response_id) => {
            invocation_without_body!("responses.delete", [response_id], false)
        }
        ProviderRequest::ResponsesInputItemsList(response_id) => {
            invocation_without_body!("responses.input_items.list", [response_id], false)
        }
        ProviderRequest::ResponsesCancel(response_id) => {
            invocation_without_body!("responses.cancel", [response_id], false)
        }
        ProviderRequest::ResponsesCompact(body) => {
            invocation_with_body!("responses.compact", [], body, false)
        }
        ProviderRequest::Embeddings(body) => {
            invocation_with_body!("embeddings.create", [], body, false)
        }
        ProviderRequest::Moderations(body) => {
            invocation_with_body!("moderations.create", [], body, false)
        }
        ProviderRequest::ImagesGenerations(body) => {
            invocation_with_body!("images.generate", [], body, false)
        }
        ProviderRequest::ImagesEdits(body) => {
            invocation_with_body!("images.edit", [], body, false)
        }
        ProviderRequest::ImagesVariations(body) => {
            invocation_with_body!("images.variation", [], body, false)
        }
        ProviderRequest::AudioTranscriptions(body) => {
            invocation_with_body!("audio.transcriptions.create", [], body, false)
        }
        ProviderRequest::AudioTranslations(body) => {
            invocation_with_body!("audio.translations.create", [], body, false)
        }
        ProviderRequest::AudioSpeech(body) => {
            invocation_with_body!("audio.speech.create", [], body, true)
        }
        ProviderRequest::AudioVoicesList => {
            invocation_without_body!("audio.voices.list", [], false)
        }
        ProviderRequest::AudioVoiceConsents(body) => {
            invocation_with_body!("audio.voice_consents.create", [], body, false)
        }
        ProviderRequest::Containers(body) => {
            invocation_with_body!("containers.create", [], body, false)
        }
        ProviderRequest::ContainersList => invocation_without_body!("containers.list", [], false),
        ProviderRequest::ContainersRetrieve(container_id) => {
            invocation_without_body!("containers.retrieve", [container_id], false)
        }
        ProviderRequest::ContainersDelete(container_id) => {
            invocation_without_body!("containers.delete", [container_id], false)
        }
        ProviderRequest::ContainerFiles(container_id, body) => {
            invocation_with_body!("containers.files.create", [container_id], body, false)
        }
        ProviderRequest::ContainerFilesList(container_id) => {
            invocation_without_body!("containers.files.list", [container_id], false)
        }
        ProviderRequest::ContainerFilesRetrieve(container_id, file_id) => {
            invocation_without_body!("containers.files.retrieve", [container_id, file_id], false)
        }
        ProviderRequest::ContainerFilesDelete(container_id, file_id) => {
            invocation_without_body!("containers.files.delete", [container_id, file_id], false)
        }
        ProviderRequest::ContainerFilesContent(container_id, file_id) => {
            invocation_without_body!(
                "containers.files.content.retrieve",
                [container_id, file_id],
                true
            )
        }
        ProviderRequest::Files(body) => invocation_with_body!("files.create", [], body, false),
        ProviderRequest::FilesList => invocation_without_body!("files.list", [], false),
        ProviderRequest::FilesRetrieve(file_id) => {
            invocation_without_body!("files.retrieve", [file_id], false)
        }
        ProviderRequest::FilesDelete(file_id) => {
            invocation_without_body!("files.delete", [file_id], false)
        }
        ProviderRequest::FilesContent(file_id) => {
            invocation_without_body!("files.content", [file_id], true)
        }
        ProviderRequest::Uploads(body) => invocation_with_body!("uploads.create", [], body, false),
        ProviderRequest::UploadParts(body) => {
            invocation_with_body!("uploads.parts.create", [], body, false)
        }
        ProviderRequest::UploadComplete(body) => {
            invocation_with_body!("uploads.complete", [&body.upload_id], body, false)
        }
        ProviderRequest::UploadCancel(upload_id) => {
            invocation_without_body!("uploads.cancel", [upload_id], false)
        }
        ProviderRequest::FineTuningJobs(body) => {
            invocation_with_body!("fine_tuning.jobs.create", [], body, false)
        }
        ProviderRequest::FineTuningJobsList => {
            invocation_without_body!("fine_tuning.jobs.list", [], false)
        }
        ProviderRequest::FineTuningJobsRetrieve(job_id) => {
            invocation_without_body!("fine_tuning.jobs.retrieve", [job_id], false)
        }
        ProviderRequest::FineTuningJobsCancel(job_id) => {
            invocation_without_body!("fine_tuning.jobs.cancel", [job_id], false)
        }
        ProviderRequest::FineTuningJobsEvents(job_id) => {
            invocation_without_body!("fine_tuning.jobs.events.list", [job_id], false)
        }
        ProviderRequest::FineTuningJobsCheckpoints(job_id) => {
            invocation_without_body!("fine_tuning.jobs.checkpoints.list", [job_id], false)
        }
        ProviderRequest::FineTuningJobsPause(job_id) => {
            invocation_without_body!("fine_tuning.jobs.pause", [job_id], false)
        }
        ProviderRequest::FineTuningJobsResume(job_id) => {
            invocation_without_body!("fine_tuning.jobs.resume", [job_id], false)
        }
        ProviderRequest::FineTuningCheckpointPermissions(checkpoint_id, body) => {
            invocation_with_body!(
                "fine_tuning.checkpoints.permissions.create",
                [checkpoint_id],
                body,
                false
            )
        }
        ProviderRequest::FineTuningCheckpointPermissionsList(checkpoint_id) => {
            invocation_without_body!(
                "fine_tuning.checkpoints.permissions.list",
                [checkpoint_id],
                false
            )
        }
        ProviderRequest::FineTuningCheckpointPermissionsDelete(checkpoint_id, permission_id) => {
            invocation_without_body!(
                "fine_tuning.checkpoints.permissions.delete",
                [checkpoint_id, permission_id],
                false
            )
        }
        ProviderRequest::Assistants(body) => {
            invocation_with_body!("assistants.create", [], body, false)
        }
        ProviderRequest::AssistantsList => {
            invocation_without_body!("assistants.list", [], false)
        }
        ProviderRequest::AssistantsRetrieve(assistant_id) => {
            invocation_without_body!("assistants.retrieve", [assistant_id], false)
        }
        ProviderRequest::AssistantsUpdate(assistant_id, body) => {
            invocation_with_body!("assistants.update", [assistant_id], body, false)
        }
        ProviderRequest::AssistantsDelete(assistant_id) => {
            invocation_without_body!("assistants.delete", [assistant_id], false)
        }
        ProviderRequest::RealtimeSessions(body) => {
            invocation_with_body!("realtime.sessions.create", [], body, false)
        }
        ProviderRequest::Evals(body) => invocation_with_body!("evals.create", [], body, false),
        ProviderRequest::EvalsList => invocation_without_body!("evals.list", [], false),
        ProviderRequest::EvalsRetrieve(eval_id) => {
            invocation_without_body!("evals.retrieve", [eval_id], false)
        }
        ProviderRequest::EvalsUpdate(eval_id, body) => {
            invocation_with_body!("evals.update", [eval_id], body, false)
        }
        ProviderRequest::EvalsDelete(eval_id) => {
            invocation_without_body!("evals.delete", [eval_id], false)
        }
        ProviderRequest::EvalRunsList(eval_id) => {
            invocation_without_body!("evals.runs.list", [eval_id], false)
        }
        ProviderRequest::EvalRuns(eval_id, body) => {
            invocation_with_body!("evals.runs.create", [eval_id], body, false)
        }
        ProviderRequest::EvalRunsRetrieve(eval_id, run_id) => {
            invocation_without_body!("evals.runs.retrieve", [eval_id, run_id], false)
        }
        ProviderRequest::EvalRunsDelete(eval_id, run_id) => {
            invocation_without_body!("evals.runs.delete", [eval_id, run_id], false)
        }
        ProviderRequest::EvalRunsCancel(eval_id, run_id) => {
            invocation_without_body!("evals.runs.cancel", [eval_id, run_id], false)
        }
        ProviderRequest::EvalRunOutputItemsList(eval_id, run_id) => {
            invocation_without_body!("evals.runs.output_items.list", [eval_id, run_id], false)
        }
        ProviderRequest::EvalRunOutputItemsRetrieve(eval_id, run_id, output_item_id) => {
            invocation_without_body!(
                "evals.runs.output_items.retrieve",
                [eval_id, run_id, output_item_id],
                false
            )
        }
        ProviderRequest::Batches(body) => invocation_with_body!("batches.create", [], body, false),
        ProviderRequest::BatchesList => invocation_without_body!("batches.list", [], false),
        ProviderRequest::BatchesRetrieve(batch_id) => {
            invocation_without_body!("batches.retrieve", [batch_id], false)
        }
        ProviderRequest::BatchesCancel(batch_id) => {
            invocation_without_body!("batches.cancel", [batch_id], false)
        }
        ProviderRequest::VectorStores(body) => {
            invocation_with_body!("vector_stores.create", [], body, false)
        }
        ProviderRequest::VectorStoresList => {
            invocation_without_body!("vector_stores.list", [], false)
        }
        ProviderRequest::VectorStoresRetrieve(vector_store_id) => {
            invocation_without_body!("vector_stores.retrieve", [vector_store_id], false)
        }
        ProviderRequest::VectorStoresUpdate(vector_store_id, body) => {
            invocation_with_body!("vector_stores.update", [vector_store_id], body, false)
        }
        ProviderRequest::VectorStoresDelete(vector_store_id) => {
            invocation_without_body!("vector_stores.delete", [vector_store_id], false)
        }
        ProviderRequest::VectorStoresSearch(vector_store_id, body) => {
            invocation_with_body!("vector_stores.search", [vector_store_id], body, false)
        }
        ProviderRequest::VectorStoreFiles(vector_store_id, body) => {
            invocation_with_body!("vector_stores.files.create", [vector_store_id], body, false)
        }
        ProviderRequest::VectorStoreFilesList(vector_store_id) => {
            invocation_without_body!("vector_stores.files.list", [vector_store_id], false)
        }
        ProviderRequest::VectorStoreFilesRetrieve(vector_store_id, file_id) => {
            invocation_without_body!(
                "vector_stores.files.retrieve",
                [vector_store_id, file_id],
                false
            )
        }
        ProviderRequest::VectorStoreFilesDelete(vector_store_id, file_id) => {
            invocation_without_body!(
                "vector_stores.files.delete",
                [vector_store_id, file_id],
                false
            )
        }
        ProviderRequest::VectorStoreFileBatches(vector_store_id, body) => {
            invocation_with_body!(
                "vector_stores.file_batches.create",
                [vector_store_id],
                body,
                false
            )
        }
        ProviderRequest::VectorStoreFileBatchesRetrieve(vector_store_id, batch_id) => {
            invocation_without_body!(
                "vector_stores.file_batches.retrieve",
                [vector_store_id, batch_id],
                false
            )
        }
        ProviderRequest::VectorStoreFileBatchesCancel(vector_store_id, batch_id) => {
            invocation_without_body!(
                "vector_stores.file_batches.cancel",
                [vector_store_id, batch_id],
                false
            )
        }
        ProviderRequest::VectorStoreFileBatchesListFiles(vector_store_id, batch_id) => {
            invocation_without_body!(
                "vector_stores.file_batches.files.list",
                [vector_store_id, batch_id],
                false
            )
        }
        ProviderRequest::Videos(body) => invocation_with_body!("videos.create", [], body, false),
        ProviderRequest::VideosList => invocation_without_body!("videos.list", [], false),
        ProviderRequest::VideosRetrieve(video_id) => {
            invocation_without_body!("videos.retrieve", [video_id], false)
        }
        ProviderRequest::VideosDelete(video_id) => {
            invocation_without_body!("videos.delete", [video_id], false)
        }
        ProviderRequest::VideosContent(video_id) => {
            invocation_without_body!("videos.content", [video_id], true)
        }
        ProviderRequest::VideosRemix(video_id, body) => {
            invocation_with_body!("videos.remix", [video_id], body, false)
        }
        ProviderRequest::VideoCharactersCreate(body) => {
            invocation_with_body!("videos.characters.create", [], body, false)
        }
        ProviderRequest::VideoCharactersList(video_id) => {
            invocation_without_body!("videos.characters.list", [video_id], false)
        }
        ProviderRequest::VideoCharactersRetrieve(video_id, character_id) => {
            invocation_without_body!(
                "videos.characters.retrieve",
                [video_id, character_id],
                false
            )
        }
        ProviderRequest::VideoCharactersUpdate(video_id, character_id, body) => {
            invocation_with_body!(
                "videos.characters.update",
                [video_id, character_id],
                body,
                false
            )
        }
        ProviderRequest::VideoCharactersCanonicalRetrieve(character_id) => {
            invocation_without_body!("videos.characters.retrieve", [character_id], false)
        }
        ProviderRequest::VideosEdits(body) => {
            invocation_with_body!("videos.edits.create", [], body, false)
        }
        ProviderRequest::VideosExtensions(body) => {
            invocation_with_body!("videos.extensions.create", [], body, false)
        }
        ProviderRequest::VideosExtend(video_id, body) => {
            invocation_with_body!("videos.extend", [video_id], body, false)
        }
        ProviderRequest::Webhooks(body) => {
            invocation_with_body!("webhooks.create", [], body, false)
        }
        ProviderRequest::WebhooksList => invocation_without_body!("webhooks.list", [], false),
        ProviderRequest::WebhooksRetrieve(webhook_id) => {
            invocation_without_body!("webhooks.retrieve", [webhook_id], false)
        }
        ProviderRequest::WebhooksUpdate(webhook_id, body) => {
            invocation_with_body!("webhooks.update", [webhook_id], body, false)
        }
        ProviderRequest::WebhooksDelete(webhook_id) => {
            invocation_without_body!("webhooks.delete", [webhook_id], false)
        }
    })
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

fn native_dynamic_runtime_registry() -> Result<
    std::sync::MutexGuard<'static, HashMap<String, Arc<NativeDynamicRuntime>>>,
    ExtensionHostError,
> {
    NATIVE_DYNAMIC_RUNTIME_REGISTRY
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .map_err(|_| ExtensionHostError::NativeDynamicRuntimeStatePoisoned {
            entrypoint: "native_dynamic_runtime_registry".to_owned(),
        })
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
                extension_id: process.extension_id.clone(),
                display_name: process.display_name.clone(),
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

#[cfg(test)]
mod tests {
    use super::provider_invocation_from_request;
    use super::provider_invocation_from_request_with_options;
    use sdkwork_api_contract_openai::audio::CreateSpeechRequest;
    use sdkwork_api_contract_openai::responses::CreateResponseRequest;
    use sdkwork_api_contract_openai::uploads::CompleteUploadRequest;
    use sdkwork_api_provider_core::{ProviderRequest, ProviderRequestOptions};

    #[test]
    fn upload_complete_invocation_preserves_upload_id_as_path_param() {
        let request = CompleteUploadRequest::new("upload_123", ["part_1", "part_2"]);

        let invocation = provider_invocation_from_request(
            ProviderRequest::UploadComplete(&request),
            "sk-native",
            "https://example.com/v1",
        )
        .expect("provider invocation");

        assert_eq!(invocation.operation, "uploads.complete");
        assert_eq!(invocation.path_params, vec!["upload_123".to_owned()]);
        assert_eq!(
            invocation.body["part_ids"],
            serde_json::json!(["part_1", "part_2"])
        );
    }

    #[test]
    fn responses_stream_invocation_marks_stream_expectation() {
        let request = CreateResponseRequest {
            model: "gpt-4.1".to_owned(),
            input: serde_json::Value::String("hello".to_owned()),
            stream: Some(true),
        };

        let invocation = provider_invocation_from_request(
            ProviderRequest::ResponsesStream(&request),
            "sk-native",
            "https://example.com/v1",
        )
        .expect("provider invocation");

        assert_eq!(invocation.operation, "responses.create");
        assert!(invocation.expects_stream);
    }

    #[test]
    fn audio_speech_invocation_marks_stream_expectation() {
        let mut request = CreateSpeechRequest::new("gpt-4o-mini-tts", "nova", "hello");
        request.response_format = Some("mp3".to_owned());

        let invocation = provider_invocation_from_request(
            ProviderRequest::AudioSpeech(&request),
            "sk-native",
            "https://example.com/v1",
        )
        .expect("provider invocation");

        assert_eq!(invocation.operation, "audio.speech.create");
        assert!(invocation.expects_stream);
    }

    #[test]
    fn files_content_invocation_marks_stream_expectation() {
        let invocation = provider_invocation_from_request(
            ProviderRequest::FilesContent("file_1"),
            "sk-native",
            "https://example.com/v1",
        )
        .expect("provider invocation");

        assert_eq!(invocation.operation, "files.content");
        assert!(invocation.expects_stream);
    }

    #[test]
    fn videos_content_invocation_marks_stream_expectation() {
        let invocation = provider_invocation_from_request(
            ProviderRequest::VideosContent("video_1"),
            "sk-native",
            "https://example.com/v1",
        )
        .expect("provider invocation");

        assert_eq!(invocation.operation, "videos.content");
        assert!(invocation.expects_stream);
    }

    #[test]
    fn provider_invocation_preserves_compatibility_headers_when_requested() {
        let request = CreateResponseRequest {
            model: "gpt-4.1".to_owned(),
            input: serde_json::Value::String("hello".to_owned()),
            stream: None,
        };
        let options = ProviderRequestOptions::new()
            .with_header("anthropic-version", "2023-06-01")
            .with_header("anthropic-beta", "tools-2024-04-04");

        let invocation = provider_invocation_from_request_with_options(
            ProviderRequest::Responses(&request),
            "sk-native",
            "https://example.com/v1",
            &options,
        )
        .expect("provider invocation");

        assert_eq!(
            invocation
                .headers
                .get("anthropic-version")
                .map(String::as_str),
            Some("2023-06-01")
        );
        assert_eq!(
            invocation.headers.get("anthropic-beta").map(String::as_str),
            Some("tools-2024-04-04")
        );
    }
}
