use super::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RuntimeMode {
    #[default]
    Server,
    Embedded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalConfigPaths {
    pub root_dir: PathBuf,
    pub primary_config_yaml: PathBuf,
    pub secondary_config_yml: PathBuf,
    pub fallback_config_json: PathBuf,
    pub database_file: PathBuf,
    pub secret_local_file: PathBuf,
    pub extensions_dir: PathBuf,
}

impl LocalConfigPaths {
    pub fn from_home_dir(home_dir: PathBuf) -> Self {
        Self::from_root_dir(home_dir.join(".sdkwork").join("router"))
    }

    pub fn from_root_dir(root_dir: PathBuf) -> Self {
        Self {
            primary_config_yaml: root_dir.join("config.yaml"),
            secondary_config_yml: root_dir.join("config.yml"),
            fallback_config_json: root_dir.join("config.json"),
            database_file: root_dir.join("sdkwork-api-server.db"),
            secret_local_file: root_dir.join("secrets.json"),
            extensions_dir: root_dir.join("extensions"),
            root_dir,
        }
    }

    pub(crate) fn discovered_config_candidates(&self) -> [PathBuf; 3] {
        [
            self.primary_config_yaml.clone(),
            self.secondary_config_yml.clone(),
            self.fallback_config_json.clone(),
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandaloneConfig {
    pub gateway_bind: String,
    pub admin_bind: String,
    pub portal_bind: String,
    pub database_url: String,
    pub cache_backend: CacheBackendKind,
    pub cache_url: Option<String>,
    pub extension_paths: Vec<String>,
    pub enable_connector_extensions: bool,
    pub enable_native_dynamic_extensions: bool,
    pub extension_hot_reload_interval_secs: u64,
    pub extension_trusted_signers: HashMap<String, String>,
    pub require_signed_connector_extensions: bool,
    pub require_signed_native_dynamic_extensions: bool,
    pub native_dynamic_shutdown_drain_timeout_ms: u64,
    pub admin_jwt_signing_secret: String,
    pub portal_jwt_signing_secret: String,
    pub runtime_snapshot_interval_secs: u64,
    pub pricing_lifecycle_sync_interval_secs: u64,
    pub secret_backend: SecretBackendKind,
    pub credential_master_key: String,
    pub allow_insecure_dev_defaults: bool,
    pub metrics_bearer_token: String,
    pub browser_allowed_origins: Vec<String>,
    pub credential_legacy_master_keys: Vec<String>,
    pub secret_local_file: String,
    pub secret_keyring_service: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandaloneRuntimeDynamicConfig {
    pub extension_paths: Vec<String>,
    pub enable_connector_extensions: bool,
    pub enable_native_dynamic_extensions: bool,
    pub extension_hot_reload_interval_secs: u64,
    pub extension_trusted_signers: HashMap<String, String>,
    pub require_signed_connector_extensions: bool,
    pub require_signed_native_dynamic_extensions: bool,
    pub native_dynamic_shutdown_drain_timeout_ms: u64,
    pub runtime_snapshot_interval_secs: u64,
    pub pricing_lifecycle_sync_interval_secs: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpExposureConfig {
    pub metrics_bearer_token: String,
    pub browser_allowed_origins: Vec<String>,
}
