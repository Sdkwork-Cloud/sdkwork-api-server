use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
pub use sdkwork_api_secret_core::SecretBackendKind;
use sdkwork_api_storage_core::StorageDialect;
use serde::Deserialize;

const SDKWORK_CONFIG_DIR: &str = "SDKWORK_CONFIG_DIR";
const SDKWORK_CONFIG_FILE: &str = "SDKWORK_CONFIG_FILE";
const SDKWORK_GATEWAY_BIND: &str = "SDKWORK_GATEWAY_BIND";
const SDKWORK_ADMIN_BIND: &str = "SDKWORK_ADMIN_BIND";
const SDKWORK_PORTAL_BIND: &str = "SDKWORK_PORTAL_BIND";
const SDKWORK_DATABASE_URL: &str = "SDKWORK_DATABASE_URL";
const SDKWORK_EXTENSION_PATHS: &str = "SDKWORK_EXTENSION_PATHS";
const SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS: &str =
    "SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS";
const SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS: &str =
    "SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS";
const SDKWORK_EXTENSION_HOT_RELOAD_INTERVAL_SECS: &str =
    "SDKWORK_EXTENSION_HOT_RELOAD_INTERVAL_SECS";
const SDKWORK_EXTENSION_TRUSTED_SIGNERS: &str = "SDKWORK_EXTENSION_TRUSTED_SIGNERS";
const SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS: &str =
    "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS";
const SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS: &str =
    "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS";
const SDKWORK_NATIVE_DYNAMIC_SHUTDOWN_DRAIN_TIMEOUT_MS: &str =
    "SDKWORK_NATIVE_DYNAMIC_SHUTDOWN_DRAIN_TIMEOUT_MS";
const SDKWORK_ADMIN_JWT_SIGNING_SECRET: &str = "SDKWORK_ADMIN_JWT_SIGNING_SECRET";
const SDKWORK_PORTAL_JWT_SIGNING_SECRET: &str = "SDKWORK_PORTAL_JWT_SIGNING_SECRET";
const SDKWORK_RUNTIME_SNAPSHOT_INTERVAL_SECS: &str = "SDKWORK_RUNTIME_SNAPSHOT_INTERVAL_SECS";
const SDKWORK_SECRET_BACKEND: &str = "SDKWORK_SECRET_BACKEND";
const SDKWORK_CREDENTIAL_MASTER_KEY: &str = "SDKWORK_CREDENTIAL_MASTER_KEY";
const SDKWORK_CREDENTIAL_LEGACY_MASTER_KEYS: &str = "SDKWORK_CREDENTIAL_LEGACY_MASTER_KEYS";
const SDKWORK_SECRET_LOCAL_FILE: &str = "SDKWORK_SECRET_LOCAL_FILE";
const SDKWORK_SECRET_KEYRING_SERVICE: &str = "SDKWORK_SECRET_KEYRING_SERVICE";

const MANAGED_ENV_KEYS: [&str; 17] = [
    SDKWORK_GATEWAY_BIND,
    SDKWORK_ADMIN_BIND,
    SDKWORK_PORTAL_BIND,
    SDKWORK_DATABASE_URL,
    SDKWORK_EXTENSION_PATHS,
    SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS,
    SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS,
    SDKWORK_EXTENSION_HOT_RELOAD_INTERVAL_SECS,
    SDKWORK_EXTENSION_TRUSTED_SIGNERS,
    SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS,
    SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS,
    SDKWORK_NATIVE_DYNAMIC_SHUTDOWN_DRAIN_TIMEOUT_MS,
    SDKWORK_ADMIN_JWT_SIGNING_SECRET,
    SDKWORK_PORTAL_JWT_SIGNING_SECRET,
    SDKWORK_RUNTIME_SNAPSHOT_INTERVAL_SECS,
    SDKWORK_SECRET_BACKEND,
    SDKWORK_CREDENTIAL_MASTER_KEY,
];

const MANAGED_SECRET_ENV_KEYS: [&str; 3] = [
    SDKWORK_CREDENTIAL_LEGACY_MASTER_KEYS,
    SDKWORK_SECRET_LOCAL_FILE,
    SDKWORK_SECRET_KEYRING_SERVICE,
];

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

    fn discovered_config_candidates(&self) -> [PathBuf; 3] {
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
    pub secret_backend: SecretBackendKind,
    pub credential_master_key: String,
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
}

#[derive(Debug, Clone)]
pub struct StandaloneConfigLoader {
    local_root: PathBuf,
    values: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandaloneConfigWatchState {
    entries: Vec<StandaloneConfigWatchEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StandaloneConfigWatchEntry {
    path: PathBuf,
    exists: bool,
    is_file: bool,
    len: u64,
    modified_at_ms: Option<u128>,
}

#[derive(Debug, Default, Deserialize)]
struct StandaloneConfigFile {
    gateway_bind: Option<String>,
    admin_bind: Option<String>,
    portal_bind: Option<String>,
    database_url: Option<String>,
    extension_paths: Option<Vec<String>>,
    enable_connector_extensions: Option<bool>,
    enable_native_dynamic_extensions: Option<bool>,
    extension_hot_reload_interval_secs: Option<u64>,
    extension_trusted_signers: Option<HashMap<String, String>>,
    require_signed_connector_extensions: Option<bool>,
    require_signed_native_dynamic_extensions: Option<bool>,
    native_dynamic_shutdown_drain_timeout_ms: Option<u64>,
    admin_jwt_signing_secret: Option<String>,
    portal_jwt_signing_secret: Option<String>,
    runtime_snapshot_interval_secs: Option<u64>,
    secret_backend: Option<String>,
    credential_master_key: Option<String>,
    credential_legacy_master_keys: Option<Vec<String>>,
    secret_local_file: Option<String>,
    secret_keyring_service: Option<String>,
}

impl Default for StandaloneConfig {
    fn default() -> Self {
        Self {
            gateway_bind: "127.0.0.1:8080".to_owned(),
            admin_bind: "127.0.0.1:8081".to_owned(),
            portal_bind: "127.0.0.1:8082".to_owned(),
            database_url: "sqlite://sdkwork-api-server.db".to_owned(),
            extension_paths: Vec::new(),
            enable_connector_extensions: true,
            enable_native_dynamic_extensions: false,
            extension_hot_reload_interval_secs: 0,
            extension_trusted_signers: HashMap::new(),
            require_signed_connector_extensions: false,
            require_signed_native_dynamic_extensions: true,
            native_dynamic_shutdown_drain_timeout_ms: 0,
            admin_jwt_signing_secret: "local-dev-admin-jwt-secret".to_owned(),
            portal_jwt_signing_secret: "local-dev-portal-jwt-secret".to_owned(),
            runtime_snapshot_interval_secs: 0,
            secret_backend: SecretBackendKind::DatabaseEncrypted,
            credential_master_key: "local-dev-master-key".to_owned(),
            credential_legacy_master_keys: Vec::new(),
            secret_local_file: "sdkwork-api-secrets.json".to_owned(),
            secret_keyring_service: "sdkwork-api-server".to_owned(),
        }
    }
}

impl StandaloneConfig {
    pub fn from_env() -> Result<Self> {
        let (_, config) = StandaloneConfigLoader::from_env()?;
        Ok(config)
    }

    pub fn from_local_root_and_pairs<P, I, K, V>(local_root: P, pairs: I) -> Result<Self>
    where
        P: AsRef<Path>,
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let values = collect_pairs(pairs);
        let local_root = absolutize_path(local_root.as_ref())?;
        Self::from_local_root_and_values(local_root, values)
    }

    pub fn from_pairs<I, K, V>(pairs: I) -> Result<Self>
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let values = collect_pairs(pairs);
        let mut config = Self::default();
        config.apply_env_overrides(&values)?;
        Ok(config)
    }

    pub fn resolved_env_pairs(&self) -> Vec<(String, String)> {
        let mut pairs = vec![
            (SDKWORK_GATEWAY_BIND.to_owned(), self.gateway_bind.clone()),
            (SDKWORK_ADMIN_BIND.to_owned(), self.admin_bind.clone()),
            (SDKWORK_PORTAL_BIND.to_owned(), self.portal_bind.clone()),
            (SDKWORK_DATABASE_URL.to_owned(), self.database_url.clone()),
            (
                SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS.to_owned(),
                self.enable_connector_extensions.to_string(),
            ),
            (
                SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS.to_owned(),
                self.enable_native_dynamic_extensions.to_string(),
            ),
            (
                SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS.to_owned(),
                self.require_signed_connector_extensions.to_string(),
            ),
            (
                SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS.to_owned(),
                self.require_signed_native_dynamic_extensions.to_string(),
            ),
            (
                SDKWORK_ADMIN_JWT_SIGNING_SECRET.to_owned(),
                self.admin_jwt_signing_secret.clone(),
            ),
            (
                SDKWORK_PORTAL_JWT_SIGNING_SECRET.to_owned(),
                self.portal_jwt_signing_secret.clone(),
            ),
            (
                SDKWORK_RUNTIME_SNAPSHOT_INTERVAL_SECS.to_owned(),
                self.runtime_snapshot_interval_secs.to_string(),
            ),
            (
                SDKWORK_NATIVE_DYNAMIC_SHUTDOWN_DRAIN_TIMEOUT_MS.to_owned(),
                self.native_dynamic_shutdown_drain_timeout_ms.to_string(),
            ),
            (
                SDKWORK_EXTENSION_HOT_RELOAD_INTERVAL_SECS.to_owned(),
                self.extension_hot_reload_interval_secs.to_string(),
            ),
            (
                SDKWORK_SECRET_BACKEND.to_owned(),
                self.secret_backend.as_str().to_owned(),
            ),
            (
                SDKWORK_CREDENTIAL_MASTER_KEY.to_owned(),
                self.credential_master_key.clone(),
            ),
            (
                SDKWORK_SECRET_LOCAL_FILE.to_owned(),
                self.secret_local_file.clone(),
            ),
            (
                SDKWORK_SECRET_KEYRING_SERVICE.to_owned(),
                self.secret_keyring_service.clone(),
            ),
        ];

        if !self.extension_paths.is_empty() {
            pairs.push((
                SDKWORK_EXTENSION_PATHS.to_owned(),
                join_env_paths(&self.extension_paths),
            ));
        }

        if !self.extension_trusted_signers.is_empty() {
            pairs.push((
                SDKWORK_EXTENSION_TRUSTED_SIGNERS.to_owned(),
                trusted_signers_to_env(&self.extension_trusted_signers),
            ));
        }

        if !self.credential_legacy_master_keys.is_empty() {
            pairs.push((
                SDKWORK_CREDENTIAL_LEGACY_MASTER_KEYS.to_owned(),
                join_env_list(&self.credential_legacy_master_keys),
            ));
        }

        pairs
    }

    pub fn apply_to_process_env(&self) {
        for key in MANAGED_ENV_KEYS.into_iter().chain(MANAGED_SECRET_ENV_KEYS) {
            std::env::remove_var(key);
        }

        for (key, value) in self.resolved_env_pairs() {
            std::env::set_var(key, value);
        }
    }

    pub fn runtime_dynamic_config(&self) -> StandaloneRuntimeDynamicConfig {
        StandaloneRuntimeDynamicConfig {
            extension_paths: self.extension_paths.clone(),
            enable_connector_extensions: self.enable_connector_extensions,
            enable_native_dynamic_extensions: self.enable_native_dynamic_extensions,
            extension_hot_reload_interval_secs: self.extension_hot_reload_interval_secs,
            extension_trusted_signers: self.extension_trusted_signers.clone(),
            require_signed_connector_extensions: self.require_signed_connector_extensions,
            require_signed_native_dynamic_extensions: self.require_signed_native_dynamic_extensions,
            native_dynamic_shutdown_drain_timeout_ms: self.native_dynamic_shutdown_drain_timeout_ms,
            runtime_snapshot_interval_secs: self.runtime_snapshot_interval_secs,
        }
    }

    pub fn non_reloadable_changed_fields(&self, next: &Self) -> Vec<&'static str> {
        let mut fields = Vec::new();
        if self.gateway_bind != next.gateway_bind {
            fields.push("gateway_bind");
        }
        if self.admin_bind != next.admin_bind {
            fields.push("admin_bind");
        }
        if self.portal_bind != next.portal_bind {
            fields.push("portal_bind");
        }
        if self.database_url != next.database_url {
            fields.push("database_url");
        }
        if self.admin_jwt_signing_secret != next.admin_jwt_signing_secret {
            fields.push("admin_jwt_signing_secret");
        }
        if self.portal_jwt_signing_secret != next.portal_jwt_signing_secret {
            fields.push("portal_jwt_signing_secret");
        }
        if self.secret_backend != next.secret_backend {
            fields.push("secret_backend");
        }
        if self.credential_master_key != next.credential_master_key {
            fields.push("credential_master_key");
        }
        if self.credential_legacy_master_keys != next.credential_legacy_master_keys {
            fields.push("credential_legacy_master_keys");
        }
        if self.secret_local_file != next.secret_local_file {
            fields.push("secret_local_file");
        }
        if self.secret_keyring_service != next.secret_keyring_service {
            fields.push("secret_keyring_service");
        }
        fields
    }

    pub fn storage_dialect(&self) -> Option<StorageDialect> {
        let database_url = self.database_url.to_ascii_lowercase();

        if database_url.starts_with("sqlite:") {
            Some(StorageDialect::Sqlite)
        } else if database_url.starts_with("postgres://")
            || database_url.starts_with("postgresql://")
        {
            Some(StorageDialect::Postgres)
        } else if database_url.starts_with("mysql://") {
            Some(StorageDialect::Mysql)
        } else if database_url.starts_with("libsql://") || database_url.starts_with("turso://") {
            Some(StorageDialect::Libsql)
        } else {
            None
        }
    }

    fn from_local_root_and_values(
        local_root: PathBuf,
        values: HashMap<String, String>,
    ) -> Result<Self> {
        let paths = LocalConfigPaths::from_root_dir(local_root);
        let mut config = Self::local_defaults(&paths);

        if let Some(config_file) = resolve_config_file_path(&paths, &values)? {
            let overlay = load_config_file(&config_file)?;
            let base_dir = config_file.parent().unwrap_or(paths.root_dir.as_path());
            config.apply_file_overlay(overlay, base_dir, &config_file)?;
        }

        config.apply_env_overrides(&values)?;
        Ok(config)
    }

    fn local_defaults(paths: &LocalConfigPaths) -> Self {
        Self {
            gateway_bind: "127.0.0.1:8080".to_owned(),
            admin_bind: "127.0.0.1:8081".to_owned(),
            portal_bind: "127.0.0.1:8082".to_owned(),
            database_url: sqlite_url_for_path(&paths.database_file),
            extension_paths: vec![paths.extensions_dir.to_string_lossy().into_owned()],
            enable_connector_extensions: true,
            enable_native_dynamic_extensions: false,
            extension_hot_reload_interval_secs: 0,
            extension_trusted_signers: HashMap::new(),
            require_signed_connector_extensions: false,
            require_signed_native_dynamic_extensions: true,
            native_dynamic_shutdown_drain_timeout_ms: 0,
            admin_jwt_signing_secret: "local-dev-admin-jwt-secret".to_owned(),
            portal_jwt_signing_secret: "local-dev-portal-jwt-secret".to_owned(),
            runtime_snapshot_interval_secs: 0,
            secret_backend: SecretBackendKind::DatabaseEncrypted,
            credential_master_key: "local-dev-master-key".to_owned(),
            credential_legacy_master_keys: Vec::new(),
            secret_local_file: paths.secret_local_file.to_string_lossy().into_owned(),
            secret_keyring_service: "sdkwork-api-server".to_owned(),
        }
    }

    fn apply_file_overlay(
        &mut self,
        file: StandaloneConfigFile,
        base_dir: &Path,
        config_file: &Path,
    ) -> Result<()> {
        if let Some(value) = file.gateway_bind {
            self.gateway_bind = value;
        }
        if let Some(value) = file.admin_bind {
            self.admin_bind = value;
        }
        if let Some(value) = file.portal_bind {
            self.portal_bind = value;
        }
        if let Some(value) = file.database_url {
            self.database_url = normalize_database_url(&value, base_dir);
        }
        if let Some(value) = file.extension_paths {
            self.extension_paths = value
                .into_iter()
                .map(|path| normalize_file_path_value(&path, base_dir))
                .collect();
        }
        if let Some(value) = file.enable_connector_extensions {
            self.enable_connector_extensions = value;
        }
        if let Some(value) = file.enable_native_dynamic_extensions {
            self.enable_native_dynamic_extensions = value;
        }
        if let Some(value) = file.extension_hot_reload_interval_secs {
            self.extension_hot_reload_interval_secs = value;
        }
        if let Some(value) = file.extension_trusted_signers {
            self.extension_trusted_signers = value;
        }
        if let Some(value) = file.require_signed_connector_extensions {
            self.require_signed_connector_extensions = value;
        }
        if let Some(value) = file.require_signed_native_dynamic_extensions {
            self.require_signed_native_dynamic_extensions = value;
        }
        if let Some(value) = file.native_dynamic_shutdown_drain_timeout_ms {
            self.native_dynamic_shutdown_drain_timeout_ms = value;
        }
        if let Some(value) = file.admin_jwt_signing_secret {
            self.admin_jwt_signing_secret = value;
        }
        if let Some(value) = file.portal_jwt_signing_secret {
            self.portal_jwt_signing_secret = value;
        }
        if let Some(value) = file.runtime_snapshot_interval_secs {
            self.runtime_snapshot_interval_secs = value;
        }
        if let Some(value) = file.secret_backend {
            self.secret_backend = SecretBackendKind::parse(&value).with_context(|| {
                format!(
                    "invalid secret_backend value in config file {}",
                    config_file.display()
                )
            })?;
        }
        if let Some(value) = file.credential_master_key {
            self.credential_master_key = value;
        }
        if let Some(value) = file.credential_legacy_master_keys {
            self.credential_legacy_master_keys = value;
        }
        if let Some(value) = file.secret_local_file {
            self.secret_local_file = normalize_file_path_value(&value, base_dir);
        }
        if let Some(value) = file.secret_keyring_service {
            self.secret_keyring_service = value;
        }

        Ok(())
    }

    fn apply_env_overrides(&mut self, values: &HashMap<String, String>) -> Result<()> {
        if let Some(value) = values.get(SDKWORK_GATEWAY_BIND) {
            self.gateway_bind = value.clone();
        }
        if let Some(value) = values.get(SDKWORK_ADMIN_BIND) {
            self.admin_bind = value.clone();
        }
        if let Some(value) = values.get(SDKWORK_PORTAL_BIND) {
            self.portal_bind = value.clone();
        }
        if let Some(value) = values.get(SDKWORK_DATABASE_URL) {
            self.database_url = value.clone();
        }
        if let Some(value) = values.get(SDKWORK_EXTENSION_PATHS) {
            self.extension_paths = std::env::split_paths(value)
                .map(|path| path.to_string_lossy().into_owned())
                .collect();
        }
        self.enable_connector_extensions = parse_bool_env(
            values,
            SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS,
            self.enable_connector_extensions,
        )?;
        self.enable_native_dynamic_extensions = parse_bool_env(
            values,
            SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS,
            self.enable_native_dynamic_extensions,
        )?;
        self.extension_hot_reload_interval_secs = parse_u64_env(
            values,
            SDKWORK_EXTENSION_HOT_RELOAD_INTERVAL_SECS,
            self.extension_hot_reload_interval_secs,
        )?;
        if let Some(value) = values.get(SDKWORK_EXTENSION_TRUSTED_SIGNERS) {
            self.extension_trusted_signers =
                parse_trusted_signers(value, SDKWORK_EXTENSION_TRUSTED_SIGNERS)?;
        }
        self.require_signed_connector_extensions = parse_bool_env(
            values,
            SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS,
            self.require_signed_connector_extensions,
        )?;
        self.require_signed_native_dynamic_extensions = parse_bool_env(
            values,
            SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS,
            self.require_signed_native_dynamic_extensions,
        )?;
        self.native_dynamic_shutdown_drain_timeout_ms = parse_u64_env(
            values,
            SDKWORK_NATIVE_DYNAMIC_SHUTDOWN_DRAIN_TIMEOUT_MS,
            self.native_dynamic_shutdown_drain_timeout_ms,
        )?;
        if let Some(value) = values.get(SDKWORK_ADMIN_JWT_SIGNING_SECRET) {
            self.admin_jwt_signing_secret = value.clone();
        }
        if let Some(value) = values.get(SDKWORK_PORTAL_JWT_SIGNING_SECRET) {
            self.portal_jwt_signing_secret = value.clone();
        }
        self.runtime_snapshot_interval_secs = parse_u64_env(
            values,
            SDKWORK_RUNTIME_SNAPSHOT_INTERVAL_SECS,
            self.runtime_snapshot_interval_secs,
        )?;
        if let Some(value) = values.get(SDKWORK_SECRET_BACKEND) {
            self.secret_backend = SecretBackendKind::parse(value)?;
        }
        if let Some(value) = values.get(SDKWORK_CREDENTIAL_MASTER_KEY) {
            self.credential_master_key = value.clone();
        }
        if let Some(value) = values.get(SDKWORK_CREDENTIAL_LEGACY_MASTER_KEYS) {
            self.credential_legacy_master_keys =
                parse_string_list_env(value, SDKWORK_CREDENTIAL_LEGACY_MASTER_KEYS)?;
        }
        if let Some(value) = values.get(SDKWORK_SECRET_LOCAL_FILE) {
            self.secret_local_file = value.clone();
        }
        if let Some(value) = values.get(SDKWORK_SECRET_KEYRING_SERVICE) {
            self.secret_keyring_service = value.clone();
        }

        Ok(())
    }
}

impl StandaloneRuntimeDynamicConfig {
    pub fn apply_to_process_env(&self) {
        for key in [
            SDKWORK_EXTENSION_PATHS,
            SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS,
            SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS,
            SDKWORK_EXTENSION_HOT_RELOAD_INTERVAL_SECS,
            SDKWORK_EXTENSION_TRUSTED_SIGNERS,
            SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS,
            SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS,
            SDKWORK_RUNTIME_SNAPSHOT_INTERVAL_SECS,
        ] {
            std::env::remove_var(key);
        }

        if !self.extension_paths.is_empty() {
            std::env::set_var(
                SDKWORK_EXTENSION_PATHS,
                join_env_paths(&self.extension_paths),
            );
        }
        std::env::set_var(
            SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS,
            self.enable_connector_extensions.to_string(),
        );
        std::env::set_var(
            SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS,
            self.enable_native_dynamic_extensions.to_string(),
        );
        std::env::set_var(
            SDKWORK_EXTENSION_HOT_RELOAD_INTERVAL_SECS,
            self.extension_hot_reload_interval_secs.to_string(),
        );
        if !self.extension_trusted_signers.is_empty() {
            std::env::set_var(
                SDKWORK_EXTENSION_TRUSTED_SIGNERS,
                trusted_signers_to_env(&self.extension_trusted_signers),
            );
        }
        std::env::set_var(
            SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS,
            self.require_signed_connector_extensions.to_string(),
        );
        std::env::set_var(
            SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS,
            self.require_signed_native_dynamic_extensions.to_string(),
        );
        std::env::set_var(
            SDKWORK_NATIVE_DYNAMIC_SHUTDOWN_DRAIN_TIMEOUT_MS,
            self.native_dynamic_shutdown_drain_timeout_ms.to_string(),
        );
        std::env::set_var(
            SDKWORK_RUNTIME_SNAPSHOT_INTERVAL_SECS,
            self.runtime_snapshot_interval_secs.to_string(),
        );
    }
}

impl StandaloneConfigLoader {
    pub fn from_env() -> Result<(Self, StandaloneConfig)> {
        Self::from_values(collect_pairs(std::env::vars()))
    }

    pub fn from_local_root_and_pairs<P, I, K, V>(
        local_root: P,
        pairs: I,
    ) -> Result<(Self, StandaloneConfig)>
    where
        P: AsRef<Path>,
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let values = collect_pairs(pairs);
        let local_root = absolutize_path(local_root.as_ref())?;
        Self::from_local_root_and_values(local_root, values)
    }

    pub fn reload(&self) -> Result<StandaloneConfig> {
        StandaloneConfig::from_local_root_and_values(self.local_root.clone(), self.values.clone())
    }

    pub fn local_root(&self) -> &Path {
        &self.local_root
    }

    pub fn with_overrides<I, K, V>(&self, pairs: I) -> Result<(Self, StandaloneConfig)>
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let mut values = self.values.clone();
        for (key, value) in pairs {
            values.insert(key.into(), value.into());
        }
        Self::from_local_root_and_values(self.local_root.clone(), values)
    }

    pub fn watch_state(&self) -> Result<StandaloneConfigWatchState> {
        StandaloneConfigWatchState::capture(&self.local_root, &self.values)
    }

    fn from_values(values: HashMap<String, String>) -> Result<(Self, StandaloneConfig)> {
        let local_root = resolve_local_root_dir(&values)?;
        Self::from_local_root_and_values(local_root, values)
    }

    fn from_local_root_and_values(
        local_root: PathBuf,
        values: HashMap<String, String>,
    ) -> Result<(Self, StandaloneConfig)> {
        let config =
            StandaloneConfig::from_local_root_and_values(local_root.clone(), values.clone())?;
        Ok((Self { local_root, values }, config))
    }
}

impl StandaloneConfigWatchState {
    fn capture(local_root: &Path, values: &HashMap<String, String>) -> Result<Self> {
        let paths = LocalConfigPaths::from_root_dir(local_root.to_path_buf());
        let watch_paths = config_watch_paths(&paths, values)?;
        let mut entries = watch_paths
            .into_iter()
            .map(|path| capture_watch_entry(path.as_path()))
            .collect::<Result<Vec<_>>>()?;
        entries.sort_by(|left, right| left.path.cmp(&right.path));
        Ok(Self { entries })
    }
}

fn collect_pairs<I, K, V>(pairs: I) -> HashMap<String, String>
where
    I: IntoIterator<Item = (K, V)>,
    K: Into<String>,
    V: Into<String>,
{
    pairs
        .into_iter()
        .map(|(key, value)| (key.into(), value.into()))
        .collect()
}

fn resolve_local_root_dir(values: &HashMap<String, String>) -> Result<PathBuf> {
    let home_dir = resolve_home_dir(values).ok();

    match values.get(SDKWORK_CONFIG_DIR) {
        Some(path) if !path.trim().is_empty() => {
            let expanded = expand_home_prefix(path, home_dir.as_deref());
            absolutize_path(&expanded)
        }
        _ => {
            let home_dir = home_dir.ok_or_else(|| {
                anyhow::anyhow!(
                    "unable to resolve a home directory for default config root ~/.sdkwork/router"
                )
            })?;
            Ok(LocalConfigPaths::from_home_dir(home_dir).root_dir)
        }
    }
}

fn resolve_home_dir(values: &HashMap<String, String>) -> Result<PathBuf> {
    if let Some(path) = values.get("HOME").filter(|value| !value.trim().is_empty()) {
        return Ok(PathBuf::from(path));
    }
    if let Some(path) = values
        .get("USERPROFILE")
        .filter(|value| !value.trim().is_empty())
    {
        return Ok(PathBuf::from(path));
    }

    let home_drive = values
        .get("HOMEDRIVE")
        .map(String::as_str)
        .unwrap_or_default();
    let home_path = values
        .get("HOMEPATH")
        .map(String::as_str)
        .unwrap_or_default();
    if !home_drive.is_empty() && !home_path.is_empty() {
        return Ok(PathBuf::from(format!("{home_drive}{home_path}")));
    }

    Err(anyhow::anyhow!("home directory is not available"))
}

fn resolve_config_file_path(
    paths: &LocalConfigPaths,
    values: &HashMap<String, String>,
) -> Result<Option<PathBuf>> {
    if let Some(resolved) = resolve_requested_config_file_path(paths, values)? {
        if !resolved.is_file() {
            anyhow::bail!(
                "configured config file does not exist: {}",
                resolved.display()
            );
        }
        return Ok(Some(resolved));
    }

    for candidate in paths.discovered_config_candidates() {
        if candidate.is_file() {
            return Ok(Some(candidate));
        }
    }

    Ok(None)
}

fn config_watch_paths(
    paths: &LocalConfigPaths,
    values: &HashMap<String, String>,
) -> Result<Vec<PathBuf>> {
    if let Some(path) = resolve_requested_config_file_path(paths, values)? {
        return Ok(vec![path]);
    }

    Ok(paths.discovered_config_candidates().to_vec())
}

fn resolve_requested_config_file_path(
    paths: &LocalConfigPaths,
    values: &HashMap<String, String>,
) -> Result<Option<PathBuf>> {
    let home_dir = resolve_home_dir(values).ok();

    values
        .get(SDKWORK_CONFIG_FILE)
        .filter(|value| !value.trim().is_empty())
        .map(|path| {
            let expanded = expand_home_prefix(path, home_dir.as_deref());
            let resolved = if expanded.is_absolute() {
                expanded
            } else {
                paths.root_dir.join(expanded)
            };
            absolutize_path(&resolved)
        })
        .transpose()
}

fn load_config_file(path: &Path) -> Result<StandaloneConfigFile> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read config file {}", path.display()))?;
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase());

    match extension.as_deref() {
        Some("yaml") | Some("yml") => serde_yaml::from_str(&content)
            .with_context(|| format!("failed to parse YAML config file {}", path.display())),
        Some("json") => serde_json::from_str(&content)
            .with_context(|| format!("failed to parse JSON config file {}", path.display())),
        Some(other) => Err(anyhow::anyhow!(
            "unsupported config file extension {other} for {}",
            path.display()
        )),
        None => Err(anyhow::anyhow!(
            "config file {} does not have a supported extension",
            path.display()
        )),
    }
}

fn parse_bool_env(values: &HashMap<String, String>, key: &str, default: bool) -> Result<bool> {
    match values.get(key) {
        Some(value) => value
            .parse::<bool>()
            .map_err(|error| anyhow::anyhow!("invalid boolean for {key}: {error}")),
        None => Ok(default),
    }
}

fn parse_u64_env(values: &HashMap<String, String>, key: &str, default: u64) -> Result<u64> {
    match values.get(key) {
        Some(value) => value
            .parse::<u64>()
            .map_err(|error| anyhow::anyhow!("invalid unsigned integer for {key}: {error}")),
        None => Ok(default),
    }
}

fn parse_trusted_signers(value: &str, key: &str) -> Result<HashMap<String, String>> {
    let mut trusted_signers = HashMap::new();
    for entry in value.split(';') {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }
        let (publisher, public_key) = entry
            .split_once('=')
            .ok_or_else(|| anyhow::anyhow!("invalid signer entry for {key}: {entry}"))?;
        let publisher = publisher.trim();
        let public_key = public_key.trim();
        if publisher.is_empty() || public_key.is_empty() {
            return Err(anyhow::anyhow!("invalid signer entry for {key}: {entry}"));
        }
        if trusted_signers
            .insert(publisher.to_owned(), public_key.to_owned())
            .is_some()
        {
            return Err(anyhow::anyhow!(
                "duplicate signer entry for {key}: {publisher}"
            ));
        }
    }
    Ok(trusted_signers)
}

fn trusted_signers_to_env(trusted_signers: &HashMap<String, String>) -> String {
    let mut entries = trusted_signers.iter().collect::<Vec<_>>();
    entries.sort_by(|left, right| left.0.cmp(right.0));
    entries
        .into_iter()
        .map(|(publisher, public_key)| format!("{publisher}={public_key}"))
        .collect::<Vec<_>>()
        .join(";")
}

fn parse_string_list_env(value: &str, key: &str) -> Result<Vec<String>> {
    let mut entries = Vec::new();
    for entry in value.split(';') {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }
        entries.push(entry.to_owned());
    }

    if entries.is_empty() && !value.trim().is_empty() {
        anyhow::bail!("invalid list value for {key}");
    }

    Ok(entries)
}

fn join_env_list(values: &[String]) -> String {
    values.join(";")
}

fn normalize_file_path_value(value: &str, base_dir: &Path) -> String {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path.to_string_lossy().into_owned()
    } else {
        base_dir.join(path).to_string_lossy().into_owned()
    }
}

fn normalize_database_url(value: &str, base_dir: &Path) -> String {
    if !value.to_ascii_lowercase().starts_with("sqlite:") {
        return value.to_owned();
    }
    if value.contains(":memory:") {
        return value.to_owned();
    }

    let query_start = value.find('?').unwrap_or(value.len());
    let (sqlite_part, query) = value.split_at(query_start);
    let raw_path = sqlite_part
        .strip_prefix("sqlite://")
        .or_else(|| sqlite_part.strip_prefix("sqlite:"))
        .unwrap_or(sqlite_part);
    if raw_path.is_empty() {
        return value.to_owned();
    }

    let normalized = raw_path.replace('\\', "/");
    let resolved = if normalized.starts_with('/') || has_windows_drive_prefix(&normalized) {
        sqlite_url_for_normalized_path(&normalized)
    } else {
        sqlite_url_for_path(&base_dir.join(PathBuf::from(normalized)))
    };

    format!("{resolved}{query}")
}

fn sqlite_url_for_path(path: &Path) -> String {
    let normalized = path.to_string_lossy().replace('\\', "/");
    sqlite_url_for_normalized_path(&normalized)
}

fn sqlite_url_for_normalized_path(path: &str) -> String {
    if path.starts_with('/') {
        format!("sqlite://{path}")
    } else {
        format!("sqlite:///{path}")
    }
}

fn has_windows_drive_prefix(path: &str) -> bool {
    let bytes = path.as_bytes();
    bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'/' || bytes[2] == b'\\')
}

fn join_env_paths(paths: &[String]) -> String {
    let joined = std::env::join_paths(paths.iter().map(PathBuf::from));
    match joined {
        Ok(value) => value.to_string_lossy().into_owned(),
        Err(_) => {
            #[cfg(windows)]
            let separator = ";";
            #[cfg(not(windows))]
            let separator = ":";
            paths.join(separator)
        }
    }
}

fn expand_home_prefix(value: &str, home_dir: Option<&Path>) -> PathBuf {
    if value == "~" {
        return home_dir
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from(value));
    }

    if let Some(stripped) = value
        .strip_prefix("~/")
        .or_else(|| value.strip_prefix("~\\"))
    {
        return home_dir
            .map(|home_dir| home_dir.join(stripped))
            .unwrap_or_else(|| PathBuf::from(value));
    }

    PathBuf::from(value)
}

fn absolutize_path(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

fn capture_watch_entry(path: &Path) -> Result<StandaloneConfigWatchEntry> {
    match fs::metadata(path) {
        Ok(metadata) => Ok(StandaloneConfigWatchEntry {
            path: path.to_path_buf(),
            exists: true,
            is_file: metadata.is_file(),
            len: metadata.len(),
            modified_at_ms: metadata.modified().ok().and_then(system_time_to_unix_ms),
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(StandaloneConfigWatchEntry {
                path: path.to_path_buf(),
                exists: false,
                is_file: false,
                len: 0,
                modified_at_ms: None,
            })
        }
        Err(error) => Err(error).with_context(|| {
            format!(
                "failed to capture config watch metadata for {}",
                path.display()
            )
        }),
    }
}

fn system_time_to_unix_ms(value: SystemTime) -> Option<u128> {
    value
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_millis())
}

#[cfg(test)]
mod tests {
    use super::StandaloneConfig;

    #[test]
    fn parses_runtime_snapshot_interval_from_env_pairs() {
        let config =
            StandaloneConfig::from_pairs([("SDKWORK_RUNTIME_SNAPSHOT_INTERVAL_SECS", "30")])
                .unwrap();

        assert_eq!(config.runtime_snapshot_interval_secs, 30);
    }

    #[test]
    fn parses_portal_env_pairs() {
        let config = StandaloneConfig::from_pairs([
            ("SDKWORK_PORTAL_BIND", "127.0.0.1:8082"),
            ("SDKWORK_PORTAL_JWT_SIGNING_SECRET", "portal-secret"),
        ])
        .unwrap();

        assert_eq!(config.portal_bind, "127.0.0.1:8082");
        assert_eq!(config.portal_jwt_signing_secret, "portal-secret");
    }
}
