use anyhow::Result;
use async_trait::async_trait;
use sdkwork_api_domain_billing::{LedgerEntry, QuotaPolicy};
use sdkwork_api_domain_catalog::{
    Channel, ChannelModelRecord, ModelCatalogEntry, ModelPriceRecord, ProxyProvider,
};
use sdkwork_api_domain_coupon::CouponCampaign;
use sdkwork_api_domain_credential::UpstreamCredential;
use sdkwork_api_domain_identity::{AdminUserRecord, GatewayApiKeyRecord, PortalUserRecord};
use sdkwork_api_domain_routing::{
    ProjectRoutingPreferences, ProviderHealthSnapshot, RoutingDecisionLog, RoutingPolicy,
};
use sdkwork_api_domain_tenant::{Project, Tenant};
use sdkwork_api_domain_usage::UsageRecord;
use sdkwork_api_extension_core::{ExtensionInstallation, ExtensionInstance};
use sdkwork_api_secret_core::SecretEnvelope;
use std::sync::{Arc, RwLock};

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

pub struct Reloadable<T: Clone> {
    current: Arc<RwLock<T>>,
}

impl<T: Clone> Reloadable<T> {
    pub fn new(initial: T) -> Self {
        Self {
            current: Arc::new(RwLock::new(initial)),
        }
    }

    pub fn snapshot(&self) -> T {
        self.current
            .read()
            .expect("reloadable value lock poisoned")
            .clone()
    }

    pub fn replace(&self, next: T) {
        *self
            .current
            .write()
            .expect("reloadable value lock poisoned") = next;
    }
}

impl<T: Clone> Clone for Reloadable<T> {
    fn clone(&self) -> Self {
        Self {
            current: Arc::clone(&self.current),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceRuntimeNodeRecord {
    pub node_id: String,
    pub service_kind: String,
    pub started_at_ms: u64,
    pub last_seen_at_ms: u64,
}

impl ServiceRuntimeNodeRecord {
    pub fn new(
        node_id: impl Into<String>,
        service_kind: impl Into<String>,
        started_at_ms: u64,
    ) -> Self {
        Self {
            node_id: node_id.into(),
            service_kind: service_kind.into(),
            started_at_ms,
            last_seen_at_ms: started_at_ms,
        }
    }

    pub fn with_last_seen_at_ms(mut self, last_seen_at_ms: u64) -> Self {
        self.last_seen_at_ms = last_seen_at_ms;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionRuntimeRolloutRecord {
    pub rollout_id: String,
    pub scope: String,
    pub requested_extension_id: Option<String>,
    pub requested_instance_id: Option<String>,
    pub resolved_extension_id: Option<String>,
    pub created_by: String,
    pub created_at_ms: u64,
    pub deadline_at_ms: u64,
}

impl ExtensionRuntimeRolloutRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        rollout_id: impl Into<String>,
        scope: impl Into<String>,
        requested_extension_id: Option<String>,
        requested_instance_id: Option<String>,
        resolved_extension_id: Option<String>,
        created_by: impl Into<String>,
        created_at_ms: u64,
        deadline_at_ms: u64,
    ) -> Self {
        Self {
            rollout_id: rollout_id.into(),
            scope: scope.into(),
            requested_extension_id,
            requested_instance_id,
            resolved_extension_id,
            created_by: created_by.into(),
            created_at_ms,
            deadline_at_ms,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionRuntimeRolloutParticipantRecord {
    pub rollout_id: String,
    pub node_id: String,
    pub service_kind: String,
    pub status: String,
    pub message: Option<String>,
    pub updated_at_ms: u64,
}

impl ExtensionRuntimeRolloutParticipantRecord {
    pub fn new(
        rollout_id: impl Into<String>,
        node_id: impl Into<String>,
        service_kind: impl Into<String>,
        status: impl Into<String>,
        updated_at_ms: u64,
    ) -> Self {
        Self {
            rollout_id: rollout_id.into(),
            node_id: node_id.into(),
            service_kind: service_kind.into(),
            status: status.into(),
            message: None,
            updated_at_ms,
        }
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandaloneConfigRolloutRecord {
    pub rollout_id: String,
    pub requested_service_kind: Option<String>,
    pub created_by: String,
    pub created_at_ms: u64,
    pub deadline_at_ms: u64,
}

impl StandaloneConfigRolloutRecord {
    pub fn new(
        rollout_id: impl Into<String>,
        requested_service_kind: Option<String>,
        created_by: impl Into<String>,
        created_at_ms: u64,
        deadline_at_ms: u64,
    ) -> Self {
        Self {
            rollout_id: rollout_id.into(),
            requested_service_kind,
            created_by: created_by.into(),
            created_at_ms,
            deadline_at_ms,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandaloneConfigRolloutParticipantRecord {
    pub rollout_id: String,
    pub node_id: String,
    pub service_kind: String,
    pub status: String,
    pub message: Option<String>,
    pub updated_at_ms: u64,
}

impl StandaloneConfigRolloutParticipantRecord {
    pub fn new(
        rollout_id: impl Into<String>,
        node_id: impl Into<String>,
        service_kind: impl Into<String>,
        status: impl Into<String>,
        updated_at_ms: u64,
    ) -> Self {
        Self {
            rollout_id: rollout_id.into(),
            node_id: node_id.into(),
            service_kind: service_kind.into(),
            status: status.into(),
            message: None,
            updated_at_ms,
        }
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }
}

#[async_trait]
pub trait AdminStore: Send + Sync {
    fn dialect(&self) -> StorageDialect;

    async fn insert_channel(&self, channel: &Channel) -> Result<Channel>;
    async fn list_channels(&self) -> Result<Vec<Channel>>;
    async fn delete_channel(&self, channel_id: &str) -> Result<bool>;

    async fn insert_provider(&self, provider: &ProxyProvider) -> Result<ProxyProvider>;
    async fn list_providers(&self) -> Result<Vec<ProxyProvider>>;
    async fn find_provider(&self, provider_id: &str) -> Result<Option<ProxyProvider>>;
    async fn delete_provider(&self, provider_id: &str) -> Result<bool>;

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
    async fn delete_credential(
        &self,
        tenant_id: &str,
        provider_id: &str,
        key_reference: &str,
    ) -> Result<bool>;

    async fn insert_model(&self, model: &ModelCatalogEntry) -> Result<ModelCatalogEntry>;
    async fn list_models(&self) -> Result<Vec<ModelCatalogEntry>>;
    async fn find_model(&self, external_name: &str) -> Result<Option<ModelCatalogEntry>>;
    async fn delete_model(&self, external_name: &str) -> Result<bool>;
    async fn delete_model_variant(&self, external_name: &str, provider_id: &str) -> Result<bool>;
    async fn insert_channel_model(&self, record: &ChannelModelRecord) -> Result<ChannelModelRecord>;
    async fn list_channel_models(&self) -> Result<Vec<ChannelModelRecord>>;
    async fn delete_channel_model(&self, channel_id: &str, model_id: &str) -> Result<bool>;
    async fn insert_model_price(&self, record: &ModelPriceRecord) -> Result<ModelPriceRecord>;
    async fn list_model_prices(&self) -> Result<Vec<ModelPriceRecord>>;
    async fn delete_model_price(
        &self,
        channel_id: &str,
        model_id: &str,
        proxy_provider_id: &str,
    ) -> Result<bool>;

    async fn insert_routing_policy(&self, policy: &RoutingPolicy) -> Result<RoutingPolicy>;
    async fn list_routing_policies(&self) -> Result<Vec<RoutingPolicy>>;
    async fn insert_project_routing_preferences(
        &self,
        preferences: &ProjectRoutingPreferences,
    ) -> Result<ProjectRoutingPreferences>;
    async fn find_project_routing_preferences(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectRoutingPreferences>>;
    async fn insert_routing_decision_log(
        &self,
        log: &RoutingDecisionLog,
    ) -> Result<RoutingDecisionLog>;
    async fn list_routing_decision_logs(&self) -> Result<Vec<RoutingDecisionLog>>;
    async fn insert_provider_health_snapshot(
        &self,
        snapshot: &ProviderHealthSnapshot,
    ) -> Result<ProviderHealthSnapshot>;
    async fn list_provider_health_snapshots(&self) -> Result<Vec<ProviderHealthSnapshot>>;

    async fn insert_usage_record(&self, record: &UsageRecord) -> Result<UsageRecord>;
    async fn list_usage_records(&self) -> Result<Vec<UsageRecord>>;

    async fn insert_ledger_entry(&self, entry: &LedgerEntry) -> Result<LedgerEntry>;
    async fn list_ledger_entries(&self) -> Result<Vec<LedgerEntry>>;
    async fn insert_quota_policy(&self, policy: &QuotaPolicy) -> Result<QuotaPolicy>;
    async fn list_quota_policies(&self) -> Result<Vec<QuotaPolicy>>;

    async fn insert_tenant(&self, tenant: &Tenant) -> Result<Tenant>;
    async fn list_tenants(&self) -> Result<Vec<Tenant>>;
    async fn find_tenant(&self, tenant_id: &str) -> Result<Option<Tenant>>;
    async fn delete_tenant(&self, tenant_id: &str) -> Result<bool>;

    async fn insert_project(&self, project: &Project) -> Result<Project>;
    async fn list_projects(&self) -> Result<Vec<Project>>;
    async fn find_project(&self, project_id: &str) -> Result<Option<Project>>;
    async fn delete_project(&self, project_id: &str) -> Result<bool>;

    async fn insert_coupon(&self, coupon: &CouponCampaign) -> Result<CouponCampaign>;
    async fn list_coupons(&self) -> Result<Vec<CouponCampaign>>;
    async fn find_coupon(&self, coupon_id: &str) -> Result<Option<CouponCampaign>>;
    async fn delete_coupon(&self, coupon_id: &str) -> Result<bool>;

    async fn insert_portal_user(&self, user: &PortalUserRecord) -> Result<PortalUserRecord>;
    async fn list_portal_users(&self) -> Result<Vec<PortalUserRecord>>;
    async fn find_portal_user_by_email(&self, email: &str) -> Result<Option<PortalUserRecord>>;
    async fn find_portal_user_by_id(&self, user_id: &str) -> Result<Option<PortalUserRecord>>;
    async fn delete_portal_user(&self, user_id: &str) -> Result<bool>;
    async fn insert_admin_user(&self, user: &AdminUserRecord) -> Result<AdminUserRecord>;
    async fn list_admin_users(&self) -> Result<Vec<AdminUserRecord>>;
    async fn find_admin_user_by_email(&self, email: &str) -> Result<Option<AdminUserRecord>>;
    async fn find_admin_user_by_id(&self, user_id: &str) -> Result<Option<AdminUserRecord>>;
    async fn delete_admin_user(&self, user_id: &str) -> Result<bool>;

    async fn insert_gateway_api_key(
        &self,
        record: &GatewayApiKeyRecord,
    ) -> Result<GatewayApiKeyRecord>;
    async fn list_gateway_api_keys(&self) -> Result<Vec<GatewayApiKeyRecord>>;
    async fn find_gateway_api_key(&self, hashed_key: &str) -> Result<Option<GatewayApiKeyRecord>>;
    async fn delete_gateway_api_key(&self, hashed_key: &str) -> Result<bool>;

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

    async fn upsert_service_runtime_node(
        &self,
        record: &ServiceRuntimeNodeRecord,
    ) -> Result<ServiceRuntimeNodeRecord>;
    async fn list_service_runtime_nodes(&self) -> Result<Vec<ServiceRuntimeNodeRecord>>;

    async fn insert_extension_runtime_rollout(
        &self,
        rollout: &ExtensionRuntimeRolloutRecord,
    ) -> Result<ExtensionRuntimeRolloutRecord>;
    async fn find_extension_runtime_rollout(
        &self,
        rollout_id: &str,
    ) -> Result<Option<ExtensionRuntimeRolloutRecord>>;
    async fn list_extension_runtime_rollouts(&self) -> Result<Vec<ExtensionRuntimeRolloutRecord>>;

    async fn insert_extension_runtime_rollout_participant(
        &self,
        participant: &ExtensionRuntimeRolloutParticipantRecord,
    ) -> Result<ExtensionRuntimeRolloutParticipantRecord>;
    async fn list_extension_runtime_rollout_participants(
        &self,
        rollout_id: &str,
    ) -> Result<Vec<ExtensionRuntimeRolloutParticipantRecord>>;
    async fn list_pending_extension_runtime_rollout_participants_for_node(
        &self,
        node_id: &str,
    ) -> Result<Vec<ExtensionRuntimeRolloutParticipantRecord>>;
    async fn transition_extension_runtime_rollout_participant(
        &self,
        rollout_id: &str,
        node_id: &str,
        from_status: &str,
        to_status: &str,
        message: Option<&str>,
        updated_at_ms: u64,
    ) -> Result<bool>;

    async fn insert_standalone_config_rollout(
        &self,
        rollout: &StandaloneConfigRolloutRecord,
    ) -> Result<StandaloneConfigRolloutRecord>;
    async fn find_standalone_config_rollout(
        &self,
        rollout_id: &str,
    ) -> Result<Option<StandaloneConfigRolloutRecord>>;
    async fn list_standalone_config_rollouts(&self) -> Result<Vec<StandaloneConfigRolloutRecord>>;

    async fn insert_standalone_config_rollout_participant(
        &self,
        participant: &StandaloneConfigRolloutParticipantRecord,
    ) -> Result<StandaloneConfigRolloutParticipantRecord>;
    async fn list_standalone_config_rollout_participants(
        &self,
        rollout_id: &str,
    ) -> Result<Vec<StandaloneConfigRolloutParticipantRecord>>;
    async fn list_pending_standalone_config_rollout_participants_for_node(
        &self,
        node_id: &str,
    ) -> Result<Vec<StandaloneConfigRolloutParticipantRecord>>;
    async fn transition_standalone_config_rollout_participant(
        &self,
        rollout_id: &str,
        node_id: &str,
        from_status: &str,
        to_status: &str,
        message: Option<&str>,
        updated_at_ms: u64,
    ) -> Result<bool>;
}
