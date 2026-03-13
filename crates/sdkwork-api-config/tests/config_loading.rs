use sdkwork_api_config::RuntimeMode;
use sdkwork_api_config::SecretBackendKind;
use sdkwork_api_config::StandaloneConfig;
use sdkwork_api_storage_core::StorageDialect;

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
    let extension_paths =
        std::env::join_paths(["D:/sdkwork/extensions", "D:/sdkwork/extensions-trusted"]).unwrap();

    let config = StandaloneConfig::from_pairs([
        (
            "SDKWORK_EXTENSION_PATHS",
            extension_paths.to_string_lossy().as_ref(),
        ),
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

    assert_eq!(
        config.extension_paths,
        vec![
            "D:/sdkwork/extensions".to_owned(),
            "D:/sdkwork/extensions-trusted".to_owned()
        ]
    );
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
