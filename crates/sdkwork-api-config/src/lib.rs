use std::collections::HashMap;

use anyhow::Result;
pub use sdkwork_api_secret_core::SecretBackendKind;
use sdkwork_api_storage_core::StorageDialect;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RuntimeMode {
    #[default]
    Server,
    Embedded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandaloneConfig {
    pub gateway_bind: String,
    pub admin_bind: String,
    pub database_url: String,
    pub extension_paths: Vec<String>,
    pub enable_connector_extensions: bool,
    pub enable_native_dynamic_extensions: bool,
    pub extension_trusted_signers: HashMap<String, String>,
    pub require_signed_connector_extensions: bool,
    pub require_signed_native_dynamic_extensions: bool,
    pub admin_jwt_signing_secret: String,
    pub secret_backend: SecretBackendKind,
    pub credential_master_key: String,
    pub secret_local_file: String,
    pub secret_keyring_service: String,
}

impl Default for StandaloneConfig {
    fn default() -> Self {
        Self {
            gateway_bind: "127.0.0.1:8080".to_owned(),
            admin_bind: "127.0.0.1:8081".to_owned(),
            database_url: "sqlite://sdkwork-api-server.db".to_owned(),
            extension_paths: Vec::new(),
            enable_connector_extensions: true,
            enable_native_dynamic_extensions: false,
            extension_trusted_signers: HashMap::new(),
            require_signed_connector_extensions: false,
            require_signed_native_dynamic_extensions: true,
            admin_jwt_signing_secret: "local-dev-admin-jwt-secret".to_owned(),
            secret_backend: SecretBackendKind::DatabaseEncrypted,
            credential_master_key: "local-dev-master-key".to_owned(),
            secret_local_file: "sdkwork-api-secrets.json".to_owned(),
            secret_keyring_service: "sdkwork-api-server".to_owned(),
        }
    }
}

impl StandaloneConfig {
    pub fn from_env() -> Result<Self> {
        let pairs = std::env::vars();
        Self::from_pairs(pairs)
    }

    pub fn from_pairs<I, K, V>(pairs: I) -> Result<Self>
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let values: HashMap<String, String> = pairs
            .into_iter()
            .map(|(key, value)| (key.into(), value.into()))
            .collect();

        let default = Self::default();
        let secret_backend = match values.get("SDKWORK_SECRET_BACKEND") {
            Some(value) => SecretBackendKind::parse(value)?,
            None => default.secret_backend,
        };
        let extension_trusted_signers = parse_trusted_signers_env(
            &values,
            "SDKWORK_EXTENSION_TRUSTED_SIGNERS",
            &default.extension_trusted_signers,
        )?;

        Ok(Self {
            gateway_bind: values
                .get("SDKWORK_GATEWAY_BIND")
                .cloned()
                .unwrap_or(default.gateway_bind),
            admin_bind: values
                .get("SDKWORK_ADMIN_BIND")
                .cloned()
                .unwrap_or(default.admin_bind),
            database_url: values
                .get("SDKWORK_DATABASE_URL")
                .cloned()
                .unwrap_or(default.database_url),
            extension_paths: values
                .get("SDKWORK_EXTENSION_PATHS")
                .map(|value| {
                    std::env::split_paths(value)
                        .map(|path| path.to_string_lossy().into_owned())
                        .collect()
                })
                .unwrap_or(default.extension_paths),
            enable_connector_extensions: parse_bool_env(
                &values,
                "SDKWORK_EXTENSION_ENABLE_CONNECTOR_EXTENSIONS",
                default.enable_connector_extensions,
            )?,
            enable_native_dynamic_extensions: parse_bool_env(
                &values,
                "SDKWORK_EXTENSION_ENABLE_NATIVE_DYNAMIC_EXTENSIONS",
                default.enable_native_dynamic_extensions,
            )?,
            extension_trusted_signers,
            require_signed_connector_extensions: parse_bool_env(
                &values,
                "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_CONNECTOR_EXTENSIONS",
                default.require_signed_connector_extensions,
            )?,
            require_signed_native_dynamic_extensions: parse_bool_env(
                &values,
                "SDKWORK_EXTENSION_REQUIRE_SIGNATURE_FOR_NATIVE_DYNAMIC_EXTENSIONS",
                default.require_signed_native_dynamic_extensions,
            )?,
            admin_jwt_signing_secret: values
                .get("SDKWORK_ADMIN_JWT_SIGNING_SECRET")
                .cloned()
                .unwrap_or(default.admin_jwt_signing_secret),
            secret_backend,
            credential_master_key: values
                .get("SDKWORK_CREDENTIAL_MASTER_KEY")
                .cloned()
                .unwrap_or(default.credential_master_key),
            secret_local_file: values
                .get("SDKWORK_SECRET_LOCAL_FILE")
                .cloned()
                .unwrap_or(default.secret_local_file),
            secret_keyring_service: values
                .get("SDKWORK_SECRET_KEYRING_SERVICE")
                .cloned()
                .unwrap_or(default.secret_keyring_service),
        })
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
}

fn parse_bool_env(values: &HashMap<String, String>, key: &str, default: bool) -> Result<bool> {
    match values.get(key) {
        Some(value) => value
            .parse::<bool>()
            .map_err(|error| anyhow::anyhow!("invalid boolean for {key}: {error}")),
        None => Ok(default),
    }
}

fn parse_trusted_signers_env(
    values: &HashMap<String, String>,
    key: &str,
    default: &HashMap<String, String>,
) -> Result<HashMap<String, String>> {
    match values.get(key) {
        Some(value) => parse_trusted_signers(value, key),
        None => Ok(default.clone()),
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
