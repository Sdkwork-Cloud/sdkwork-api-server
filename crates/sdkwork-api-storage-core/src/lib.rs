use anyhow::Result;
use async_trait::async_trait;
use sdkwork_api_domain_billing::LedgerEntry;
use sdkwork_api_domain_catalog::{Channel, ModelCatalogEntry, ProxyProvider};
use sdkwork_api_domain_credential::UpstreamCredential;
use sdkwork_api_domain_identity::GatewayApiKeyRecord;
use sdkwork_api_domain_routing::{ProviderHealthSnapshot, RoutingPolicy};
use sdkwork_api_domain_tenant::{Project, Tenant};
use sdkwork_api_domain_usage::UsageRecord;
use sdkwork_api_extension_core::{ExtensionInstallation, ExtensionInstance};
use sdkwork_api_secret_core::SecretEnvelope;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageDialect {
    Sqlite,
    Postgres,
    Mysql,
    Libsql,
}

impl StorageDialect {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sqlite => "sqlite",
            Self::Postgres => "postgres",
            Self::Mysql => "mysql",
            Self::Libsql => "libsql",
        }
    }
}

#[async_trait]
pub trait AdminStore: Send + Sync {
    fn dialect(&self) -> StorageDialect;

    async fn insert_channel(&self, channel: &Channel) -> Result<Channel>;
    async fn list_channels(&self) -> Result<Vec<Channel>>;

    async fn insert_provider(&self, provider: &ProxyProvider) -> Result<ProxyProvider>;
    async fn list_providers(&self) -> Result<Vec<ProxyProvider>>;
    async fn find_provider(&self, provider_id: &str) -> Result<Option<ProxyProvider>>;

    async fn insert_credential(
        &self,
        credential: &UpstreamCredential,
    ) -> Result<UpstreamCredential>;
    async fn insert_encrypted_credential(
        &self,
        credential: &UpstreamCredential,
        envelope: &SecretEnvelope,
    ) -> Result<UpstreamCredential>;
    async fn list_credentials(&self) -> Result<Vec<UpstreamCredential>>;
    async fn find_credential(
        &self,
        tenant_id: &str,
        provider_id: &str,
        key_reference: &str,
    ) -> Result<Option<UpstreamCredential>>;
    async fn find_credential_envelope(
        &self,
        tenant_id: &str,
        provider_id: &str,
        key_reference: &str,
    ) -> Result<Option<SecretEnvelope>>;
    async fn find_provider_credential(
        &self,
        tenant_id: &str,
        provider_id: &str,
    ) -> Result<Option<UpstreamCredential>>;

    async fn insert_model(&self, model: &ModelCatalogEntry) -> Result<ModelCatalogEntry>;
    async fn list_models(&self) -> Result<Vec<ModelCatalogEntry>>;
    async fn find_model(&self, external_name: &str) -> Result<Option<ModelCatalogEntry>>;
    async fn delete_model(&self, external_name: &str) -> Result<bool>;

    async fn insert_routing_policy(&self, policy: &RoutingPolicy) -> Result<RoutingPolicy>;
    async fn list_routing_policies(&self) -> Result<Vec<RoutingPolicy>>;
    async fn insert_provider_health_snapshot(
        &self,
        snapshot: &ProviderHealthSnapshot,
    ) -> Result<ProviderHealthSnapshot>;
    async fn list_provider_health_snapshots(&self) -> Result<Vec<ProviderHealthSnapshot>>;

    async fn insert_usage_record(&self, record: &UsageRecord) -> Result<UsageRecord>;
    async fn list_usage_records(&self) -> Result<Vec<UsageRecord>>;

    async fn insert_ledger_entry(&self, entry: &LedgerEntry) -> Result<LedgerEntry>;
    async fn list_ledger_entries(&self) -> Result<Vec<LedgerEntry>>;

    async fn insert_tenant(&self, tenant: &Tenant) -> Result<Tenant>;
    async fn list_tenants(&self) -> Result<Vec<Tenant>>;

    async fn insert_project(&self, project: &Project) -> Result<Project>;
    async fn list_projects(&self) -> Result<Vec<Project>>;

    async fn insert_gateway_api_key(
        &self,
        record: &GatewayApiKeyRecord,
    ) -> Result<GatewayApiKeyRecord>;
    async fn list_gateway_api_keys(&self) -> Result<Vec<GatewayApiKeyRecord>>;
    async fn find_gateway_api_key(&self, hashed_key: &str) -> Result<Option<GatewayApiKeyRecord>>;

    async fn insert_extension_installation(
        &self,
        installation: &ExtensionInstallation,
    ) -> Result<ExtensionInstallation>;
    async fn list_extension_installations(&self) -> Result<Vec<ExtensionInstallation>>;

    async fn insert_extension_instance(
        &self,
        instance: &ExtensionInstance,
    ) -> Result<ExtensionInstance>;
    async fn list_extension_instances(&self) -> Result<Vec<ExtensionInstance>>;
}
