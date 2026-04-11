use sdkwork_api_config::CacheBackendKind;
use sdkwork_api_config::LocalConfigPaths;
use sdkwork_api_config::RuntimeMode;
use sdkwork_api_config::SecretBackendKind;
use sdkwork_api_config::StandaloneConfig;
use sdkwork_api_config::StandaloneConfigLoader;
use sdkwork_api_storage_core::StorageDialect;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static TEMP_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn defaults_to_server_mode() {
    assert_eq!(RuntimeMode::default(), RuntimeMode::Server);
}

#[test]
fn standalone_defaults_are_local_friendly() {
    let config = StandaloneConfig::default();
    assert_eq!(config.gateway_bind, "127.0.0.1:8080");
    assert_eq!(config.admin_bind, "127.0.0.1:8081");
    assert_eq!(config.database_url, "sqlite://sdkwork-api-server.db");
    assert!(config.extension_paths.is_empty());
    assert!(config.enable_connector_extensions);
    assert!(!config.enable_native_dynamic_extensions);
    assert!(!config.require_signed_connector_extensions);
    assert!(config.require_signed_native_dynamic_extensions);
    assert!(config.extension_trusted_signers.is_empty());
    assert_eq!(config.extension_hot_reload_interval_secs, 0);
    assert_eq!(config.cache_backend, CacheBackendKind::Memory);
    assert!(config.cache_url.is_none());
    assert_eq!(config.secret_backend, SecretBackendKind::DatabaseEncrypted);
    assert_eq!(
        config.admin_jwt_signing_secret,
        "local-dev-admin-jwt-secret"
    );
    assert_eq!(config.storage_dialect().unwrap(), StorageDialect::Sqlite);
}

#[test]
fn infers_postgres_dialect_from_database_url() {
    let config = StandaloneConfig {
        database_url: "postgres://sdkwork:secret@localhost/sdkwork".to_owned(),
        ..StandaloneConfig::default()
    };

    assert_eq!(config.storage_dialect().unwrap(), StorageDialect::Postgres);
}

#[test]
fn supports_three_secret_backend_strategies() {
    assert_eq!(
        SecretBackendKind::DatabaseEncrypted.as_str(),
        "database_encrypted"
    );
    assert_eq!(
        SecretBackendKind::LocalEncryptedFile.as_str(),
        "local_encrypted_file"
    );
    assert_eq!(SecretBackendKind::OsKeyring.as_str(), "os_keyring");
}

#[test]
fn parses_secret_backend_identifiers() {
    assert_eq!(
        SecretBackendKind::parse("database_encrypted").unwrap(),
        SecretBackendKind::DatabaseEncrypted
    );
    assert_eq!(
        SecretBackendKind::parse("local_encrypted_file").unwrap(),
        SecretBackendKind::LocalEncryptedFile
    );
    assert_eq!(
        SecretBackendKind::parse("os_keyring").unwrap(),
        SecretBackendKind::OsKeyring
    );
}

#[test]
fn builds_config_from_pairs() {
    let config = StandaloneConfig::from_pairs([
        ("SDKWORK_GATEWAY_BIND", "0.0.0.0:9000"),
        ("SDKWORK_ADMIN_BIND", "0.0.0.0:9001"),
        (
            "SDKWORK_DATABASE_URL",
            "postgres://sdkwork:secret@localhost/sdkwork",
        ),
        ("SDKWORK_SECRET_BACKEND", "os_keyring"),
        ("SDKWORK_CREDENTIAL_MASTER_KEY", "prod-master-key"),
        ("SDKWORK_ADMIN_JWT_SIGNING_SECRET", "prod-admin-jwt-secret"),
    ])
    .unwrap();

    assert_eq!(config.gateway_bind, "0.0.0.0:9000");
    assert_eq!(config.admin_bind, "0.0.0.0:9001");
    assert_eq!(config.secret_backend, SecretBackendKind::OsKeyring);
    assert_eq!(config.credential_master_key, "prod-master-key");
    assert_eq!(config.admin_jwt_signing_secret, "prod-admin-jwt-secret");
    assert_eq!(config.storage_dialect().unwrap(), StorageDialect::Postgres);
}

#[test]
fn parses_insecure_dev_override_from_pairs_and_reexports_it() {
    let config = StandaloneConfig::from_pairs([(
        "SDKWORK_ALLOW_INSECURE_DEV_DEFAULTS",
        "true",
    )])
    .unwrap();
    let values = config
        .resolved_env_pairs()
        .into_iter()
        .collect::<std::collections::HashMap<_, _>>();

    assert!(config.allow_insecure_dev_defaults);
    assert_eq!(values["SDKWORK_ALLOW_INSECURE_DEV_DEFAULTS"], "true");
}

#[test]
fn parses_http_exposure_controls_from_pairs_and_reexports_them() {
    let config = StandaloneConfig::from_pairs([
        ("SDKWORK_METRICS_BEARER_TOKEN", "prod-metrics-token"),
        (
            "SDKWORK_BROWSER_ALLOWED_ORIGINS",
            "https://console.example.com;https://portal.example.com",
        ),
    ])
    .unwrap();
    let values = config
        .resolved_env_pairs()
        .into_iter()
        .collect::<std::collections::HashMap<_, _>>();

    assert_eq!(config.metrics_bearer_token, "prod-metrics-token");
    assert_eq!(
        config.browser_allowed_origins,
        vec![
            "https://console.example.com".to_owned(),
            "https://portal.example.com".to_owned()
        ]
    );
    assert_eq!(values["SDKWORK_METRICS_BEARER_TOKEN"], "prod-metrics-token");
    assert_eq!(
        values["SDKWORK_BROWSER_ALLOWED_ORIGINS"],
        "https://console.example.com;https://portal.example.com"
    );
}

#[test]
fn parses_bootstrap_data_settings_from_pairs_and_reexports_them() {
    let config = StandaloneConfig::from_pairs([
        ("SDKWORK_BOOTSTRAP_DATA_DIR", "D:/sdkwork/bootstrap"),
        ("SDKWORK_BOOTSTRAP_PROFILE", "dev"),
    ])
    .unwrap();
    let values = config
        .resolved_env_pairs()
        .into_iter()
        .collect::<std::collections::HashMap<_, _>>();

    assert_eq!(
        config.bootstrap_data_dir.as_deref(),
        Some("D:/sdkwork/bootstrap")
    );
    assert_eq!(config.bootstrap_profile, "dev");
    assert_eq!(
        values["SDKWORK_BOOTSTRAP_DATA_DIR"],
        "D:/sdkwork/bootstrap"
    );
    assert_eq!(values["SDKWORK_BOOTSTRAP_PROFILE"], "dev");
}

#[test]
fn non_reloadable_changed_fields_include_insecure_dev_override() {
    let current = StandaloneConfig::default();
    let next = StandaloneConfig {
        allow_insecure_dev_defaults: true,
        ..current.clone()
    };

    let changed = current.non_reloadable_changed_fields(&next);

    assert!(changed.contains(&"allow_insecure_dev_defaults"));
}

#[test]
fn non_reloadable_changed_fields_include_http_exposure_controls() {
    let current = StandaloneConfig::default();
    let next = StandaloneConfig {
        metrics_bearer_token: "rotated-metrics-token".to_owned(),
        browser_allowed_origins: vec!["https://console.example.com".to_owned()],
        ..current.clone()
    };

    let changed = current.non_reloadable_changed_fields(&next);

    assert!(changed.contains(&"metrics_bearer_token"));
    assert!(changed.contains(&"browser_allowed_origins"));
}

#[test]
fn non_reloadable_changed_fields_include_bootstrap_settings() {
    let current = StandaloneConfig::default();
    let next = StandaloneConfig {
        bootstrap_data_dir: Some("D:/sdkwork/bootstrap".to_owned()),
        bootstrap_profile: "dev".to_owned(),
        ..current.clone()
    };

    let changed = current.non_reloadable_changed_fields(&next);

    assert!(changed.contains(&"bootstrap_data_dir"));
    assert!(changed.contains(&"bootstrap_profile"));
}

#[test]
fn builds_secret_runtime_locations_from_pairs() {
    let config = StandaloneConfig::from_pairs([
        ("SDKWORK_SECRET_LOCAL_FILE", "D:/sdkwork/secrets.json"),
        ("SDKWORK_SECRET_KEYRING_SERVICE", "sdkwork-api-server"),
    ])
    .unwrap();

    assert_eq!(config.secret_local_file, "D:/sdkwork/secrets.json");
    assert_eq!(config.secret_keyring_service, "sdkwork-api-server");
}

#[test]
fn parses_extension_discovery_settings_from_pairs() {
    #[cfg(windows)]
    let expected_extension_paths = vec![
        "D:/sdkwork/extensions".to_owned(),
        "D:/sdkwork/extensions-trusted".to_owned(),
    ];
    #[cfg(not(windows))]
    let expected_extension_paths = vec![
        "/tmp/sdkwork/extensions".to_owned(),
        "/tmp/sdkwork/extensions-trusted".to_owned(),
    ];
    let extension_paths =
        std::env::join_paths(expected_extension_paths.iter().map(PathBuf::from)).unwrap();

    let config = StandaloneConfig::from_pairs([
        (
            "SDKWORK_EXTENSION_PATHS",
            extension_paths.to_string_lossy().as_ref(),
        ),
        ("SDKWORK_EXTENSION_HOT_RELOAD_INTERVAL_SECS", "7"),
        ("SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS", "false"),
        ("SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS", "true"),
        (
            "SDKWORK_EXTENSION_TRUSTED_SIGNERS",
            "sdkwork=ZXhwaWNpdC1wdWJsaWMta2V5;partner=c2Vjb25kLXB1YmxpYy1rZXk=",
        ),
        (
            "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS",
            "true",
        ),
        (
            "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS",
            "false",
        ),
    ])
    .unwrap();

    assert_eq!(config.extension_paths, expected_extension_paths);
    assert_eq!(config.extension_hot_reload_interval_secs, 7);
    assert!(!config.enable_connector_extensions);
    assert!(config.enable_native_dynamic_extensions);
    assert!(config.require_signed_connector_extensions);
    assert!(!config.require_signed_native_dynamic_extensions);
    assert_eq!(
        config.extension_trusted_signers["sdkwork"],
        "ZXhwaWNpdC1wdWJsaWMta2V5"
    );
    assert_eq!(
        config.extension_trusted_signers["partner"],
        "c2Vjb25kLXB1YmxpYy1rZXk="
    );
}

#[test]
fn parses_cache_backend_and_optional_cache_url_from_pairs() {
    let config = StandaloneConfig::from_pairs([
        ("SDKWORK_CACHE_BACKEND", "redis"),
        ("SDKWORK_CACHE_URL", "redis://127.0.0.1:6379/0"),
    ])
    .unwrap();
    let values = config
        .resolved_env_pairs()
        .into_iter()
        .collect::<std::collections::HashMap<_, _>>();

    assert_eq!(config.cache_backend, CacheBackendKind::Redis);
    assert_eq!(
        config.cache_url.as_deref(),
        Some("redis://127.0.0.1:6379/0")
    );
    assert_eq!(values["SDKWORK_CACHE_BACKEND"], "redis");
    assert_eq!(values["SDKWORK_CACHE_URL"], "redis://127.0.0.1:6379/0");
}

#[test]
fn non_reloadable_changed_fields_include_cache_backend_and_cache_url() {
    let current = StandaloneConfig::default();
    let next = StandaloneConfig {
        cache_backend: CacheBackendKind::Redis,
        cache_url: Some("redis://127.0.0.1:6379/7".to_owned()),
        ..current.clone()
    };

    let changed = current.non_reloadable_changed_fields(&next);

    assert!(changed.contains(&"cache_backend"));
    assert!(changed.contains(&"cache_url"));
}

#[test]
fn local_config_paths_use_sdkwork_router_root() {
    let paths = LocalConfigPaths::from_home_dir(PathBuf::from("/tmp/sdkwork-user"));

    assert_eq!(
        paths.root_dir,
        PathBuf::from("/tmp/sdkwork-user/.sdkwork/router")
    );
    assert_eq!(
        paths.primary_config_yaml,
        PathBuf::from("/tmp/sdkwork-user/.sdkwork/router/config.yaml")
    );
    assert_eq!(
        paths.fallback_config_json,
        PathBuf::from("/tmp/sdkwork-user/.sdkwork/router/config.json")
    );
    assert_eq!(
        paths.secret_local_file,
        PathBuf::from("/tmp/sdkwork-user/.sdkwork/router/secrets.json")
    );
    assert_eq!(
        paths.extensions_dir,
        PathBuf::from("/tmp/sdkwork-user/.sdkwork/router/extensions")
    );
}

#[test]
fn uses_local_root_sqlite_defaults_when_no_config_file_exists() {
    let root = temp_config_root("local-defaults");
    let config =
        StandaloneConfig::from_local_root_and_pairs(&root, std::iter::empty::<(&str, &str)>())
            .unwrap();

    assert_eq!(config.gateway_bind, "127.0.0.1:8080");
    assert_eq!(config.admin_bind, "127.0.0.1:8081");
    assert_eq!(config.portal_bind, "127.0.0.1:8082");
    assert_eq!(
        config.database_url,
        sqlite_url_for(root.join("sdkwork-api-server.db"))
    );
    assert_eq!(
        config.secret_local_file,
        root.join("secrets.json").to_string_lossy()
    );
    assert_eq!(
        config.extension_paths,
        vec![root.join("extensions").to_string_lossy().into_owned()]
    );
    assert_eq!(config.storage_dialect().unwrap(), StorageDialect::Sqlite);
}

#[test]
fn loads_yaml_config_before_environment_overrides() {
    let root = temp_config_root("yaml-env-override");
    fs::write(
        root.join("config.yaml"),
        r#"
gateway_bind: "127.0.0.1:18080"
admin_bind: "127.0.0.1:18081"
database_url: "sqlite://router.db"
secret_local_file: "secrets/custom.json"
extension_paths:
  - "extensions/core"
  - "extensions/partner"
"#,
    )
    .unwrap();

    let config = StandaloneConfig::from_local_root_and_pairs(
        &root,
        [("SDKWORK_GATEWAY_BIND", "127.0.0.1:28080")],
    )
    .unwrap();

    assert_eq!(config.gateway_bind, "127.0.0.1:28080");
    assert_eq!(config.admin_bind, "127.0.0.1:18081");
    assert_eq!(
        config.database_url,
        sqlite_url_for(root.join("router.db")).as_str()
    );
    assert_eq!(
        config.secret_local_file,
        root.join("secrets/custom.json").to_string_lossy()
    );
    assert_eq!(
        config.extension_paths,
        vec![
            root.join("extensions/core").to_string_lossy().into_owned(),
            root.join("extensions/partner")
                .to_string_lossy()
                .into_owned(),
        ]
    );
}

#[test]
fn loads_cache_settings_from_config_file_and_allows_env_override_to_clear_cache_url() {
    let root = temp_config_root("cache-config");
    fs::write(
        root.join("config.yaml"),
        r#"
cache_backend: "redis"
cache_url: "redis://cache.internal:6379/2"
"#,
    )
    .unwrap();

    let config = StandaloneConfig::from_local_root_and_pairs(
        &root,
        [
            ("SDKWORK_CACHE_BACKEND", "memory"),
            ("SDKWORK_CACHE_URL", ""),
        ],
    )
    .unwrap();

    assert_eq!(config.cache_backend, CacheBackendKind::Memory);
    assert!(config.cache_url.is_none());
}

#[test]
fn falls_back_to_json_when_yaml_is_absent() {
    let root = temp_config_root("json-fallback");
    fs::write(
        root.join("config.json"),
        r#"{
  "portal_bind": "127.0.0.1:19082",
  "enable_native_dynamic_extensions": true
}"#,
    )
    .unwrap();

    let config =
        StandaloneConfig::from_local_root_and_pairs(&root, std::iter::empty::<(&str, &str)>())
            .unwrap();

    assert_eq!(config.portal_bind, "127.0.0.1:19082");
    assert!(config.enable_native_dynamic_extensions);
}

#[test]
fn loads_insecure_dev_override_from_config_file() {
    let root = temp_config_root("insecure-dev-override");
    fs::write(
        root.join("config.yaml"),
        "allow_insecure_dev_defaults: true\n",
    )
    .unwrap();

    let config =
        StandaloneConfig::from_local_root_and_pairs(&root, std::iter::empty::<(&str, &str)>())
            .unwrap();

    assert!(config.allow_insecure_dev_defaults);
}

#[test]
fn loads_http_exposure_controls_from_config_file() {
    let root = temp_config_root("http-exposure-controls");
    fs::write(
        root.join("config.yaml"),
        r#"
metrics_bearer_token: "config-metrics-token"
browser_allowed_origins:
  - "https://console.example.com"
  - "https://portal.example.com"
"#,
    )
    .unwrap();

    let config =
        StandaloneConfig::from_local_root_and_pairs(&root, std::iter::empty::<(&str, &str)>())
            .unwrap();

    assert_eq!(config.metrics_bearer_token, "config-metrics-token");
    assert_eq!(
        config.browser_allowed_origins,
        vec![
            "https://console.example.com".to_owned(),
            "https://portal.example.com".to_owned()
        ]
    );
}

#[test]
fn loads_bootstrap_settings_from_config_file_and_allows_env_override() {
    let root = temp_config_root("bootstrap-settings");
    fs::write(
        root.join("config.yaml"),
        r#"
bootstrap_data_dir: "bootstrap-data"
bootstrap_profile: "prod"
"#,
    )
    .unwrap();

    let config = StandaloneConfig::from_local_root_and_pairs(
        &root,
        [
            ("SDKWORK_BOOTSTRAP_DATA_DIR", "D:/sdkwork/bootstrap"),
            ("SDKWORK_BOOTSTRAP_PROFILE", "dev"),
        ],
    )
    .unwrap();

    assert_eq!(
        config.bootstrap_data_dir.as_deref(),
        Some("D:/sdkwork/bootstrap")
    );
    assert_eq!(config.bootstrap_profile, "dev");
}

#[test]
fn exports_resolved_config_back_to_sdkwork_environment_pairs() {
    let root = temp_config_root("resolved-env");
    fs::write(
        root.join("config.yaml"),
        "admin_bind: \"127.0.0.1:19081\"\n",
    )
    .unwrap();

    let config =
        StandaloneConfig::from_local_root_and_pairs(&root, std::iter::empty::<(&str, &str)>())
            .unwrap();
    let pairs = config.resolved_env_pairs();
    let values = pairs
        .into_iter()
        .collect::<std::collections::HashMap<_, _>>();

    assert_eq!(values["SDKWORK_ADMIN_BIND"], "127.0.0.1:19081");
    assert_eq!(
        values["SDKWORK_SECRET_LOCAL_FILE"],
        root.join("secrets.json").to_string_lossy()
    );
    assert_eq!(
        values["SDKWORK_EXTENSION_PATHS"],
        root.join("extensions").to_string_lossy()
    );
}

#[test]
fn standalone_config_loader_reloads_from_original_inputs_after_resolved_env_export() {
    let root = temp_config_root("loader-reload");
    fs::write(
        root.join("config.yaml"),
        r#"
admin_bind: "127.0.0.1:19081"
extension_hot_reload_interval_secs: 1
"#,
    )
    .unwrap();

    let (loader, initial) = StandaloneConfigLoader::from_local_root_and_pairs(
        &root,
        [("SDKWORK_GATEWAY_BIND", "127.0.0.1:29080")],
    )
    .unwrap();
    assert_eq!(initial.extension_hot_reload_interval_secs, 1);

    fs::write(
        root.join("config.yaml"),
        r#"
admin_bind: "127.0.0.1:19081"
extension_hot_reload_interval_secs: 7
"#,
    )
    .unwrap();

    let stale =
        StandaloneConfig::from_local_root_and_pairs(&root, initial.resolved_env_pairs()).unwrap();
    assert_eq!(stale.extension_hot_reload_interval_secs, 1);

    let reloaded = loader.reload().unwrap();
    assert_eq!(reloaded.gateway_bind, "127.0.0.1:29080");
    assert_eq!(reloaded.extension_hot_reload_interval_secs, 7);
}

#[test]
fn standalone_config_loader_watch_state_changes_when_config_file_is_created() {
    let root = temp_config_root("loader-watch-state");
    let (loader, _initial) = StandaloneConfigLoader::from_local_root_and_pairs(
        &root,
        std::iter::empty::<(&str, &str)>(),
    )
    .unwrap();
    let before = loader.watch_state().unwrap();

    fs::write(
        root.join("config.yaml"),
        "portal_bind: \"127.0.0.1:19082\"\n",
    )
    .unwrap();

    let after = loader.watch_state().unwrap();
    assert_ne!(before, after);
}

#[test]
fn standalone_config_loader_with_overrides_preserves_requested_config_file() {
    let root = temp_config_root("loader-with-overrides");
    let config_dir = root.join("configs");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(
        config_dir.join("custom.yaml"),
        r#"
admin_bind: "127.0.0.1:19081"
portal_bind: "127.0.0.1:19082"
"#,
    )
    .unwrap();

    let (loader, initial) = StandaloneConfigLoader::from_local_root_and_pairs(
        &root,
        [
            ("SDKWORK_CONFIG_FILE", "configs/custom.yaml"),
            ("SDKWORK_GATEWAY_BIND", "127.0.0.1:29080"),
        ],
    )
    .unwrap();
    assert_eq!(initial.admin_bind, "127.0.0.1:19081");
    assert_eq!(initial.gateway_bind, "127.0.0.1:29080");

    let (overridden_loader, overridden) = loader
        .with_overrides([("SDKWORK_GATEWAY_BIND", "127.0.0.1:39080")])
        .unwrap();

    assert_eq!(overridden.gateway_bind, "127.0.0.1:39080");
    assert_eq!(overridden.admin_bind, "127.0.0.1:19081");
    assert_eq!(overridden.portal_bind, "127.0.0.1:19082");
    assert_eq!(
        overridden_loader.reload().unwrap().gateway_bind,
        "127.0.0.1:39080"
    );
    assert_eq!(
        overridden_loader.reload().unwrap().admin_bind,
        "127.0.0.1:19081"
    );
}

#[test]
fn parses_native_dynamic_shutdown_drain_timeout_from_pairs_and_reload_inputs() {
    let env_config =
        StandaloneConfig::from_pairs([("SDKWORK_NATIVE_DYNAMIC_SHUTDOWN_DRAIN_TIMEOUT_MS", "75")])
            .unwrap();
    let env_values = env_config
        .resolved_env_pairs()
        .into_iter()
        .collect::<std::collections::HashMap<_, _>>();
    assert_eq!(
        env_values["SDKWORK_NATIVE_DYNAMIC_SHUTDOWN_DRAIN_TIMEOUT_MS"],
        "75"
    );

    let root = temp_config_root("native-dynamic-drain-timeout");
    fs::write(
        root.join("config.yaml"),
        "native_dynamic_shutdown_drain_timeout_ms: 25\n",
    )
    .unwrap();

    let (loader, initial) = StandaloneConfigLoader::from_local_root_and_pairs(
        &root,
        std::iter::empty::<(&str, &str)>(),
    )
    .unwrap();
    let initial_values = initial
        .resolved_env_pairs()
        .into_iter()
        .collect::<std::collections::HashMap<_, _>>();
    assert_eq!(
        initial_values["SDKWORK_NATIVE_DYNAMIC_SHUTDOWN_DRAIN_TIMEOUT_MS"],
        "25"
    );

    fs::write(
        root.join("config.yaml"),
        "native_dynamic_shutdown_drain_timeout_ms: 125\n",
    )
    .unwrap();

    let reloaded = loader.reload().unwrap();
    let reloaded_values = reloaded
        .resolved_env_pairs()
        .into_iter()
        .collect::<std::collections::HashMap<_, _>>();
    assert_eq!(
        reloaded_values["SDKWORK_NATIVE_DYNAMIC_SHUTDOWN_DRAIN_TIMEOUT_MS"],
        "125"
    );
}

#[test]
fn parses_credential_legacy_master_keys_from_pairs_and_reload_inputs() {
    let env_config = StandaloneConfig::from_pairs([(
        "SDKWORK_CREDENTIAL_LEGACY_MASTER_KEYS",
        "legacy-key-a;legacy-key-b",
    )])
    .unwrap();
    let env_values = env_config
        .resolved_env_pairs()
        .into_iter()
        .collect::<std::collections::HashMap<_, _>>();
    assert_eq!(
        env_values["SDKWORK_CREDENTIAL_LEGACY_MASTER_KEYS"],
        "legacy-key-a;legacy-key-b"
    );
    assert_eq!(
        env_config.credential_legacy_master_keys,
        vec!["legacy-key-a".to_owned(), "legacy-key-b".to_owned()]
    );

    let root = temp_config_root("credential-legacy-master-keys");
    fs::write(
        root.join("config.yaml"),
        r#"
credential_legacy_master_keys:
  - "legacy-key-one"
  - "legacy-key-two"
"#,
    )
    .unwrap();

    let (loader, initial) = StandaloneConfigLoader::from_local_root_and_pairs(
        &root,
        std::iter::empty::<(&str, &str)>(),
    )
    .unwrap();
    assert_eq!(
        initial.credential_legacy_master_keys,
        vec!["legacy-key-one".to_owned(), "legacy-key-two".to_owned()]
    );

    fs::write(
        root.join("config.yaml"),
        r#"
credential_legacy_master_keys:
  - "legacy-key-three"
"#,
    )
    .unwrap();

    let reloaded = loader.reload().unwrap();
    assert_eq!(
        reloaded.credential_legacy_master_keys,
        vec!["legacy-key-three".to_owned()]
    );
}

fn temp_config_root(label: &str) -> PathBuf {
    let unique = TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("sdkwork-config-tests-{label}-{unique}"));
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    fs::create_dir_all(&root).unwrap();
    root
}

fn sqlite_url_for(path: impl AsRef<Path>) -> String {
    let normalized = path.as_ref().to_string_lossy().replace('\\', "/");
    if normalized.starts_with('/') {
        format!("sqlite://{normalized}")
    } else {
        format!("sqlite:///{normalized}")
    }
}
