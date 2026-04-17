use super::*;

impl Default for StandaloneConfig {
    fn default() -> Self {
        Self {
            gateway_bind: "127.0.0.1:8080".to_owned(),
            admin_bind: "127.0.0.1:8081".to_owned(),
            portal_bind: "127.0.0.1:8082".to_owned(),
            database_url: "sqlite://sdkwork-api-server.db".to_owned(),
            cache_backend: CacheBackendKind::Memory,
            cache_url: None,
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
            bootstrap_data_dir: None,
            bootstrap_profile: "prod".to_owned(),
            official_openai_enabled: false,
            official_openai_base_url: "https://api.openai.com/v1".to_owned(),
            official_openai_api_key: String::new(),
            official_anthropic_enabled: false,
            official_anthropic_base_url: "https://api.anthropic.com".to_owned(),
            official_anthropic_api_key: String::new(),
            official_gemini_enabled: false,
            official_gemini_base_url: "https://generativelanguage.googleapis.com".to_owned(),
            official_gemini_api_key: String::new(),
            runtime_snapshot_interval_secs: 0,
            pricing_lifecycle_sync_interval_secs: 0,
            secret_backend: SecretBackendKind::DatabaseEncrypted,
            credential_master_key: DEFAULT_CREDENTIAL_MASTER_KEY.to_owned(),
            allow_insecure_dev_defaults: false,
            metrics_bearer_token: DEFAULT_METRICS_BEARER_TOKEN.to_owned(),
            browser_allowed_origins: default_browser_allowed_origins(),
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
                SDKWORK_CACHE_BACKEND.to_owned(),
                self.cache_backend.as_str().to_owned(),
            ),
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
                SDKWORK_BOOTSTRAP_PROFILE.to_owned(),
                self.bootstrap_profile.clone(),
            ),
            (
                SDKWORK_OFFICIAL_OPENAI_ENABLED.to_owned(),
                self.official_openai_enabled.to_string(),
            ),
            (
                SDKWORK_OFFICIAL_OPENAI_BASE_URL.to_owned(),
                self.official_openai_base_url.clone(),
            ),
            (
                SDKWORK_OFFICIAL_ANTHROPIC_ENABLED.to_owned(),
                self.official_anthropic_enabled.to_string(),
            ),
            (
                SDKWORK_OFFICIAL_ANTHROPIC_BASE_URL.to_owned(),
                self.official_anthropic_base_url.clone(),
            ),
            (
                SDKWORK_OFFICIAL_GEMINI_ENABLED.to_owned(),
                self.official_gemini_enabled.to_string(),
            ),
            (
                SDKWORK_OFFICIAL_GEMINI_BASE_URL.to_owned(),
                self.official_gemini_base_url.clone(),
            ),
            (
                SDKWORK_RUNTIME_SNAPSHOT_INTERVAL_SECS.to_owned(),
                self.runtime_snapshot_interval_secs.to_string(),
            ),
            (
                SDKWORK_PRICING_LIFECYCLE_SYNC_INTERVAL_SECS.to_owned(),
                self.pricing_lifecycle_sync_interval_secs.to_string(),
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
                SDKWORK_ALLOW_INSECURE_DEV_DEFAULTS.to_owned(),
                self.allow_insecure_dev_defaults.to_string(),
            ),
            (
                SDKWORK_METRICS_BEARER_TOKEN.to_owned(),
                self.metrics_bearer_token.clone(),
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

        if let Some(cache_url) = &self.cache_url {
            pairs.push((SDKWORK_CACHE_URL.to_owned(), cache_url.clone()));
        }

        if !self.extension_trusted_signers.is_empty() {
            pairs.push((
                SDKWORK_EXTENSION_TRUSTED_SIGNERS.to_owned(),
                trusted_signers_to_env(&self.extension_trusted_signers),
            ));
        }

        if let Some(bootstrap_data_dir) = &self.bootstrap_data_dir {
            pairs.push((
                SDKWORK_BOOTSTRAP_DATA_DIR.to_owned(),
                bootstrap_data_dir.clone(),
            ));
        }

        if !self.credential_legacy_master_keys.is_empty() {
            pairs.push((
                SDKWORK_CREDENTIAL_LEGACY_MASTER_KEYS.to_owned(),
                join_env_list(&self.credential_legacy_master_keys),
            ));
        }

        if !self.official_openai_api_key.is_empty() {
            pairs.push((
                SDKWORK_OFFICIAL_OPENAI_API_KEY.to_owned(),
                self.official_openai_api_key.clone(),
            ));
        }
        if !self.official_anthropic_api_key.is_empty() {
            pairs.push((
                SDKWORK_OFFICIAL_ANTHROPIC_API_KEY.to_owned(),
                self.official_anthropic_api_key.clone(),
            ));
        }
        if !self.official_gemini_api_key.is_empty() {
            pairs.push((
                SDKWORK_OFFICIAL_GEMINI_API_KEY.to_owned(),
                self.official_gemini_api_key.clone(),
            ));
        }

        pairs.push((
            SDKWORK_BROWSER_ALLOWED_ORIGINS.to_owned(),
            join_env_list(&self.browser_allowed_origins),
        ));

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
            pricing_lifecycle_sync_interval_secs: self.pricing_lifecycle_sync_interval_secs,
        }
    }

    pub fn http_exposure_config(&self) -> HttpExposureConfig {
        HttpExposureConfig {
            metrics_bearer_token: self.metrics_bearer_token.clone(),
            browser_allowed_origins: self.browser_allowed_origins.clone(),
        }
    }

    pub fn validate_security_posture(&self) -> Result<()> {
        if self.allow_insecure_dev_defaults {
            return Ok(());
        }

        let externally_bound_services = [
            ("gateway_bind", self.gateway_bind.as_str()),
            ("admin_bind", self.admin_bind.as_str()),
            ("portal_bind", self.portal_bind.as_str()),
        ]
        .into_iter()
        .filter_map(|(field, bind)| (!bind_is_loopback(bind)).then_some(field))
        .collect::<Vec<_>>();

        if externally_bound_services.is_empty() {
            return Ok(());
        }

        let insecure_fields = [
            (
                "admin_jwt_signing_secret",
                self.admin_jwt_signing_secret.as_str(),
                DEFAULT_ADMIN_JWT_SIGNING_SECRET,
            ),
            (
                "portal_jwt_signing_secret",
                self.portal_jwt_signing_secret.as_str(),
                DEFAULT_PORTAL_JWT_SIGNING_SECRET,
            ),
            (
                "credential_master_key",
                self.credential_master_key.as_str(),
                DEFAULT_CREDENTIAL_MASTER_KEY,
            ),
            (
                "metrics_bearer_token",
                self.metrics_bearer_token.as_str(),
                DEFAULT_METRICS_BEARER_TOKEN,
            ),
        ]
        .into_iter()
        .filter_map(|(field, value, default_value)| (value == default_value).then_some(field))
        .collect::<Vec<_>>();

        if insecure_fields.is_empty() {
            return Ok(());
        }

        anyhow::bail!(
            "refusing non-loopback service bindings for {} while built-in development defaults remain configured for {}; rotate these secrets or set {}=true for explicit development-only override",
            externally_bound_services.join(", "),
            insecure_fields.join(", "),
            SDKWORK_ALLOW_INSECURE_DEV_DEFAULTS,
        );
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
        if self.cache_backend != next.cache_backend {
            fields.push("cache_backend");
        }
        if self.cache_url != next.cache_url {
            fields.push("cache_url");
        }
        if self.admin_jwt_signing_secret != next.admin_jwt_signing_secret {
            fields.push("admin_jwt_signing_secret");
        }
        if self.portal_jwt_signing_secret != next.portal_jwt_signing_secret {
            fields.push("portal_jwt_signing_secret");
        }
        if self.bootstrap_data_dir != next.bootstrap_data_dir {
            fields.push("bootstrap_data_dir");
        }
        if self.bootstrap_profile != next.bootstrap_profile {
            fields.push("bootstrap_profile");
        }
        if self.official_openai_enabled != next.official_openai_enabled {
            fields.push("official_openai_enabled");
        }
        if self.official_openai_base_url != next.official_openai_base_url {
            fields.push("official_openai_base_url");
        }
        if self.official_openai_api_key != next.official_openai_api_key {
            fields.push("official_openai_api_key");
        }
        if self.official_anthropic_enabled != next.official_anthropic_enabled {
            fields.push("official_anthropic_enabled");
        }
        if self.official_anthropic_base_url != next.official_anthropic_base_url {
            fields.push("official_anthropic_base_url");
        }
        if self.official_anthropic_api_key != next.official_anthropic_api_key {
            fields.push("official_anthropic_api_key");
        }
        if self.official_gemini_enabled != next.official_gemini_enabled {
            fields.push("official_gemini_enabled");
        }
        if self.official_gemini_base_url != next.official_gemini_base_url {
            fields.push("official_gemini_base_url");
        }
        if self.official_gemini_api_key != next.official_gemini_api_key {
            fields.push("official_gemini_api_key");
        }
        if self.pricing_lifecycle_sync_interval_secs != next.pricing_lifecycle_sync_interval_secs {
            fields.push("pricing_lifecycle_sync_interval_secs");
        }
        if self.secret_backend != next.secret_backend {
            fields.push("secret_backend");
        }
        if self.credential_master_key != next.credential_master_key {
            fields.push("credential_master_key");
        }
        if self.allow_insecure_dev_defaults != next.allow_insecure_dev_defaults {
            fields.push("allow_insecure_dev_defaults");
        }
        if self.metrics_bearer_token != next.metrics_bearer_token {
            fields.push("metrics_bearer_token");
        }
        if self.browser_allowed_origins != next.browser_allowed_origins {
            fields.push("browser_allowed_origins");
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

    pub(crate) fn from_local_root_and_values(
        local_root: PathBuf,
        values: HashMap<String, String>,
    ) -> Result<Self> {
        Self::from_local_root_and_values_with_overrides(local_root, values, HashMap::new())
    }

    pub(crate) fn from_local_root_and_values_with_overrides(
        local_root: PathBuf,
        values: HashMap<String, String>,
        overrides: HashMap<String, String>,
    ) -> Result<Self> {
        let paths = LocalConfigPaths::from_root_dir(local_root);
        let mut config = Self::local_defaults(&paths);

        config.apply_env_overrides(&values)?;

        if let Some(config_file) = resolve_config_file_path(&paths, &values)? {
            let overlay = load_config_file(&config_file)?;
            let base_dir = config_file.parent().unwrap_or(paths.root_dir.as_path());
            config.apply_file_overlay(overlay, base_dir, &config_file)?;
        }

        for overlay_path in resolve_config_overlay_paths(&paths, &values)? {
            let overlay = load_config_file(&overlay_path)?;
            let base_dir = overlay_path.parent().unwrap_or(paths.root_dir.as_path());
            config.apply_file_overlay(overlay, base_dir, &overlay_path)?;
        }

        if !overrides.is_empty() {
            config.apply_env_overrides(&overrides)?;
        }
        Ok(config)
    }

    fn local_defaults(paths: &LocalConfigPaths) -> Self {
        Self {
            gateway_bind: "127.0.0.1:8080".to_owned(),
            admin_bind: "127.0.0.1:8081".to_owned(),
            portal_bind: "127.0.0.1:8082".to_owned(),
            database_url: sqlite_url_for_path(&paths.database_file),
            cache_backend: CacheBackendKind::Memory,
            cache_url: None,
            extension_paths: vec![paths.extensions_dir.to_string_lossy().into_owned()],
            enable_connector_extensions: true,
            enable_native_dynamic_extensions: false,
            extension_hot_reload_interval_secs: 0,
            extension_trusted_signers: HashMap::new(),
            require_signed_connector_extensions: false,
            require_signed_native_dynamic_extensions: true,
            native_dynamic_shutdown_drain_timeout_ms: 0,
            admin_jwt_signing_secret: DEFAULT_ADMIN_JWT_SIGNING_SECRET.to_owned(),
            portal_jwt_signing_secret: DEFAULT_PORTAL_JWT_SIGNING_SECRET.to_owned(),
            bootstrap_data_dir: None,
            bootstrap_profile: "prod".to_owned(),
            official_openai_enabled: false,
            official_openai_base_url: "https://api.openai.com/v1".to_owned(),
            official_openai_api_key: String::new(),
            official_anthropic_enabled: false,
            official_anthropic_base_url: "https://api.anthropic.com".to_owned(),
            official_anthropic_api_key: String::new(),
            official_gemini_enabled: false,
            official_gemini_base_url: "https://generativelanguage.googleapis.com".to_owned(),
            official_gemini_api_key: String::new(),
            runtime_snapshot_interval_secs: 0,
            pricing_lifecycle_sync_interval_secs: 0,
            secret_backend: SecretBackendKind::DatabaseEncrypted,
            credential_master_key: DEFAULT_CREDENTIAL_MASTER_KEY.to_owned(),
            allow_insecure_dev_defaults: false,
            metrics_bearer_token: DEFAULT_METRICS_BEARER_TOKEN.to_owned(),
            browser_allowed_origins: default_browser_allowed_origins(),
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
        if let Some(value) = file.cache_backend {
            self.cache_backend = CacheBackendKind::parse(&value).with_context(|| {
                format!(
                    "invalid cache_backend value in config file {}",
                    config_file.display()
                )
            })?;
        }
        if let Some(value) = file.cache_url {
            self.cache_url = normalize_optional_string(value);
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
        if let Some(value) = file.bootstrap_data_dir {
            self.bootstrap_data_dir =
                normalize_optional_string(normalize_file_path_value(&value, base_dir));
        }
        if let Some(value) = file.bootstrap_profile {
            self.bootstrap_profile = value;
        }
        if let Some(value) = file.official_openai_enabled {
            self.official_openai_enabled = value;
        }
        if let Some(value) = file.official_openai_base_url {
            self.official_openai_base_url = value;
        }
        if let Some(value) = file.official_openai_api_key {
            self.official_openai_api_key = value;
        }
        if let Some(value) = file.official_anthropic_enabled {
            self.official_anthropic_enabled = value;
        }
        if let Some(value) = file.official_anthropic_base_url {
            self.official_anthropic_base_url = value;
        }
        if let Some(value) = file.official_anthropic_api_key {
            self.official_anthropic_api_key = value;
        }
        if let Some(value) = file.official_gemini_enabled {
            self.official_gemini_enabled = value;
        }
        if let Some(value) = file.official_gemini_base_url {
            self.official_gemini_base_url = value;
        }
        if let Some(value) = file.official_gemini_api_key {
            self.official_gemini_api_key = value;
        }
        if let Some(value) = file.runtime_snapshot_interval_secs {
            self.runtime_snapshot_interval_secs = value;
        }
        if let Some(value) = file.pricing_lifecycle_sync_interval_secs {
            self.pricing_lifecycle_sync_interval_secs = value;
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
        if let Some(value) = file.allow_insecure_dev_defaults {
            self.allow_insecure_dev_defaults = value;
        }
        if let Some(value) = file.metrics_bearer_token {
            self.metrics_bearer_token = value;
        }
        if let Some(value) = file.browser_allowed_origins {
            self.browser_allowed_origins = value;
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
        if let Some(value) = values.get(SDKWORK_CACHE_BACKEND) {
            self.cache_backend = CacheBackendKind::parse(value)?;
        }
        if let Some(value) = values.get(SDKWORK_CACHE_URL) {
            self.cache_url = normalize_optional_string(value.clone());
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
        if let Some(value) = values.get(SDKWORK_BOOTSTRAP_DATA_DIR) {
            self.bootstrap_data_dir = normalize_optional_string(value.clone());
        }
        if let Some(value) = values.get(SDKWORK_BOOTSTRAP_PROFILE) {
            self.bootstrap_profile = value.clone();
        }
        self.official_openai_enabled = parse_bool_env(
            values,
            SDKWORK_OFFICIAL_OPENAI_ENABLED,
            self.official_openai_enabled,
        )?;
        if let Some(value) = values.get(SDKWORK_OFFICIAL_OPENAI_BASE_URL) {
            self.official_openai_base_url = value.clone();
        }
        if let Some(value) = values.get(SDKWORK_OFFICIAL_OPENAI_API_KEY) {
            self.official_openai_api_key = value.clone();
        }
        self.official_anthropic_enabled = parse_bool_env(
            values,
            SDKWORK_OFFICIAL_ANTHROPIC_ENABLED,
            self.official_anthropic_enabled,
        )?;
        if let Some(value) = values.get(SDKWORK_OFFICIAL_ANTHROPIC_BASE_URL) {
            self.official_anthropic_base_url = value.clone();
        }
        if let Some(value) = values.get(SDKWORK_OFFICIAL_ANTHROPIC_API_KEY) {
            self.official_anthropic_api_key = value.clone();
        }
        self.official_gemini_enabled = parse_bool_env(
            values,
            SDKWORK_OFFICIAL_GEMINI_ENABLED,
            self.official_gemini_enabled,
        )?;
        if let Some(value) = values.get(SDKWORK_OFFICIAL_GEMINI_BASE_URL) {
            self.official_gemini_base_url = value.clone();
        }
        if let Some(value) = values.get(SDKWORK_OFFICIAL_GEMINI_API_KEY) {
            self.official_gemini_api_key = value.clone();
        }
        self.runtime_snapshot_interval_secs = parse_u64_env(
            values,
            SDKWORK_RUNTIME_SNAPSHOT_INTERVAL_SECS,
            self.runtime_snapshot_interval_secs,
        )?;
        self.pricing_lifecycle_sync_interval_secs = parse_u64_env(
            values,
            SDKWORK_PRICING_LIFECYCLE_SYNC_INTERVAL_SECS,
            self.pricing_lifecycle_sync_interval_secs,
        )?;
        if let Some(value) = values.get(SDKWORK_SECRET_BACKEND) {
            self.secret_backend = SecretBackendKind::parse(value)?;
        }
        if let Some(value) = values.get(SDKWORK_CREDENTIAL_MASTER_KEY) {
            self.credential_master_key = value.clone();
        }
        self.allow_insecure_dev_defaults = parse_bool_env(
            values,
            SDKWORK_ALLOW_INSECURE_DEV_DEFAULTS,
            self.allow_insecure_dev_defaults,
        )?;
        if let Some(value) = values.get(SDKWORK_METRICS_BEARER_TOKEN) {
            self.metrics_bearer_token = value.clone();
        }
        if let Some(value) = values.get(SDKWORK_BROWSER_ALLOWED_ORIGINS) {
            self.browser_allowed_origins =
                parse_string_list_env(value, SDKWORK_BROWSER_ALLOWED_ORIGINS)?;
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
            SDKWORK_PRICING_LIFECYCLE_SYNC_INTERVAL_SECS,
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
        std::env::set_var(
            SDKWORK_PRICING_LIFECYCLE_SYNC_INTERVAL_SECS,
            self.pricing_lifecycle_sync_interval_secs.to_string(),
        );
    }
}
