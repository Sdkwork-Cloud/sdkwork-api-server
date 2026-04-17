use super::*;

#[derive(Debug, Clone)]
pub struct StandaloneConfigLoader {
    local_root: PathBuf,
    values: HashMap<String, String>,
    overrides: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandaloneConfigWatchState {
    entries: Vec<StandaloneConfigWatchEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StandaloneConfigWatchEntry {
    pub(crate) path: PathBuf,
    pub(crate) exists: bool,
    pub(crate) is_file: bool,
    pub(crate) len: u64,
    pub(crate) modified_at_ms: Option<u128>,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct StandaloneConfigFile {
    pub(crate) gateway_bind: Option<String>,
    pub(crate) admin_bind: Option<String>,
    pub(crate) portal_bind: Option<String>,
    pub(crate) database_url: Option<String>,
    pub(crate) cache_backend: Option<String>,
    pub(crate) cache_url: Option<String>,
    pub(crate) extension_paths: Option<Vec<String>>,
    pub(crate) enable_connector_extensions: Option<bool>,
    pub(crate) enable_native_dynamic_extensions: Option<bool>,
    pub(crate) extension_hot_reload_interval_secs: Option<u64>,
    pub(crate) extension_trusted_signers: Option<HashMap<String, String>>,
    pub(crate) require_signed_connector_extensions: Option<bool>,
    pub(crate) require_signed_native_dynamic_extensions: Option<bool>,
    pub(crate) native_dynamic_shutdown_drain_timeout_ms: Option<u64>,
    pub(crate) admin_jwt_signing_secret: Option<String>,
    pub(crate) portal_jwt_signing_secret: Option<String>,
    pub(crate) bootstrap_data_dir: Option<String>,
    pub(crate) bootstrap_profile: Option<String>,
    pub(crate) official_openai_enabled: Option<bool>,
    pub(crate) official_openai_base_url: Option<String>,
    pub(crate) official_openai_api_key: Option<String>,
    pub(crate) official_anthropic_enabled: Option<bool>,
    pub(crate) official_anthropic_base_url: Option<String>,
    pub(crate) official_anthropic_api_key: Option<String>,
    pub(crate) official_gemini_enabled: Option<bool>,
    pub(crate) official_gemini_base_url: Option<String>,
    pub(crate) official_gemini_api_key: Option<String>,
    pub(crate) runtime_snapshot_interval_secs: Option<u64>,
    pub(crate) pricing_lifecycle_sync_interval_secs: Option<u64>,
    pub(crate) secret_backend: Option<String>,
    pub(crate) credential_master_key: Option<String>,
    pub(crate) allow_insecure_dev_defaults: Option<bool>,
    pub(crate) metrics_bearer_token: Option<String>,
    pub(crate) browser_allowed_origins: Option<Vec<String>>,
    pub(crate) credential_legacy_master_keys: Option<Vec<String>>,
    pub(crate) secret_local_file: Option<String>,
    pub(crate) secret_keyring_service: Option<String>,
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
        Self::from_local_root_and_values(local_root, values, HashMap::new())
    }

    pub fn reload(&self) -> Result<StandaloneConfig> {
        StandaloneConfig::from_local_root_and_values_with_overrides(
            self.local_root.clone(),
            self.values.clone(),
            self.overrides.clone(),
        )
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
        let mut overrides = self.overrides.clone();
        for (key, value) in pairs {
            overrides.insert(key.into(), value.into());
        }
        Self::from_local_root_and_values(self.local_root.clone(), self.values.clone(), overrides)
    }

    pub fn watch_state(&self) -> Result<StandaloneConfigWatchState> {
        StandaloneConfigWatchState::capture(&self.local_root, &self.values)
    }

    fn from_values(values: HashMap<String, String>) -> Result<(Self, StandaloneConfig)> {
        let local_root = resolve_local_root_dir(&values)?;
        Self::from_local_root_and_values(local_root, values, HashMap::new())
    }

    fn from_local_root_and_values(
        local_root: PathBuf,
        values: HashMap<String, String>,
        overrides: HashMap<String, String>,
    ) -> Result<(Self, StandaloneConfig)> {
        let config = StandaloneConfig::from_local_root_and_values_with_overrides(
            local_root.clone(),
            values.clone(),
            overrides.clone(),
        )?;
        Ok((
            Self {
                local_root,
                values,
                overrides,
            },
            config,
        ))
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
