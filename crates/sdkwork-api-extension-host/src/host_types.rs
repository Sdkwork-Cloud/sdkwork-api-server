use super::*;


#[derive(Debug, Clone)]
pub struct BuiltinExtensionFactory {
    pub(crate) manifest: ExtensionManifest,
}

impl BuiltinExtensionFactory {
    pub fn new(manifest: ExtensionManifest) -> Self {
        Self { manifest }
    }
}

pub(crate) type ProviderFactory =
    Arc<dyn Fn(String) -> Box<dyn ProviderExecutionAdapter> + Send + Sync + 'static>;

pub(crate) type AbiVersionFn = unsafe extern "C" fn() -> u32;
pub(crate) type ManifestJsonFn = unsafe extern "C" fn() -> *const c_char;
pub(crate) type ExecuteJsonFn = unsafe extern "C" fn(*const c_char) -> *mut c_char;
pub(crate) type ExecuteStreamJsonFn =
    unsafe extern "C" fn(*const c_char, *const ProviderStreamWriter) -> *mut c_char;
pub(crate) type LifecycleJsonFn = unsafe extern "C" fn(*const c_char) -> *mut c_char;
pub(crate) type FreeStringFn = unsafe extern "C" fn(*mut c_char);
pub(crate) const SDKWORK_NATIVE_DYNAMIC_SHUTDOWN_DRAIN_TIMEOUT_MS: &str =
    "SDKWORK_NATIVE_DYNAMIC_SHUTDOWN_DRAIN_TIMEOUT_MS";

#[derive(Clone)]
pub struct BuiltinProviderExtensionFactory {
    pub(crate) manifest: ExtensionManifest,
    pub(crate) adapter_kind: String,
    pub(crate) factory: ProviderFactory,
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

pub(crate) fn provider_runtime_aliases(adapter_kind: &str) -> &'static [&'static str] {
    match adapter_kind {
        "openai" => &["openai", "openai-compatible", "custom-openai"],
        "openrouter" => &["openrouter", "openrouter-compatible"],
        "ollama" => &["ollama", "ollama-compatible"],
        _ => &[],
    }
}

#[derive(Default, Clone)]
pub struct ExtensionHost {
    pub(crate) manifests: HashMap<String, ExtensionManifest>,
    pub(crate) package_roots: HashMap<String, PathBuf>,
    pub(crate) provider_factories: HashMap<String, ProviderFactory>,
    pub(crate) provider_aliases: HashMap<String, String>,
    pub(crate) installations: HashMap<String, ExtensionInstallation>,
    pub(crate) instances_by_id: HashMap<String, ExtensionInstance>,
    pub(crate) instances_by_extension: HashMap<String, Vec<ExtensionInstance>>,
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

    pub(crate) fn allows_runtime(&self, runtime: &ExtensionRuntime) -> bool {
        match runtime {
            ExtensionRuntime::Builtin => true,
            ExtensionRuntime::Connector => self.enable_connector_extensions,
            ExtensionRuntime::NativeDynamic => self.enable_native_dynamic_extensions,
        }
    }

    pub(crate) fn requires_signature(&self, runtime: &ExtensionRuntime) -> bool {
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
pub(crate) struct ManagedConnectorProcess {
    pub(crate) child: Child,
    pub(crate) extension_id: String,
    pub(crate) display_name: String,
    pub(crate) base_url: String,
    pub(crate) health_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConnectorLaunchConfig {
    pub(crate) entrypoint: PathBuf,
    pub(crate) args: Vec<String>,
    pub(crate) environment: HashMap<String, String>,
    pub(crate) working_directory: Option<PathBuf>,
    pub(crate) health_url: String,
    pub(crate) startup_timeout: Duration,
    pub(crate) startup_poll_interval: Duration,
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

pub(crate) struct NativeDynamicRuntime {
    pub(crate) entrypoint: String,
    pub(crate) manifest: ExtensionManifest,
    pub(crate) _library: Library,
    pub(crate) execute_json: ExecuteJsonFn,
    pub(crate) execute_stream_json: Option<ExecuteStreamJsonFn>,
    pub(crate) init_json: Option<LifecycleJsonFn>,
    pub(crate) health_check_json: Option<LifecycleJsonFn>,
    pub(crate) shutdown_json: Option<LifecycleJsonFn>,
    pub(crate) free_string: FreeStringFn,
    pub(crate) lifecycle_state: Mutex<NativeDynamicLifecycleState>,
    pub(crate) lifecycle_drained: Condvar,
}

#[derive(Debug)]
pub(crate) struct NativeDynamicLifecycleState {
    pub(crate) running: bool,
    pub(crate) healthy: bool,
    pub(crate) message: Option<String>,
    pub(crate) shutdown_invoked: bool,
    pub(crate) draining: bool,
    pub(crate) active_invocations: usize,
}

unsafe impl Send for NativeDynamicRuntime {}
unsafe impl Sync for NativeDynamicRuntime {}

pub(crate) struct NativeDynamicProviderAdapter {
    pub(crate) runtime: Arc<NativeDynamicRuntime>,
    pub(crate) base_url: String,
}

pub(crate) struct HostStreamWriterContext {
    pub(crate) sender: UnboundedSender<NativeDynamicStreamEvent>,
}

pub(crate) enum NativeDynamicStreamEvent {
    ContentType(String),
    Chunk(Bytes),
    Finished(ProviderStreamInvocationResult),
}

pub(crate) static CONNECTOR_PROCESS_REGISTRY: OnceLock<Mutex<HashMap<String, ManagedConnectorProcess>>> =
    OnceLock::new();
pub(crate) static NATIVE_DYNAMIC_RUNTIME_REGISTRY: OnceLock<
    Mutex<HashMap<String, Arc<NativeDynamicRuntime>>>,
> = OnceLock::new();
