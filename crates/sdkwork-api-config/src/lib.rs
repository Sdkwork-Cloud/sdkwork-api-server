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
    pub credential_master_key: String,
}

impl Default for StandaloneConfig {
    fn default() -> Self {
        Self {
            gateway_bind: "127.0.0.1:8080".to_owned(),
            admin_bind: "127.0.0.1:8081".to_owned(),
            database_url: "sqlite://sdkwork-api-server.db".to_owned(),
            credential_master_key: "local-dev-master-key".to_owned(),
        }
    }
}
