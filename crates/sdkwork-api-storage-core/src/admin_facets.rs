use super::*;

macro_rules! define_admin_store_facet {
    ($trait_name:ident { $(async fn $name:ident(&self $(, $arg:ident : $ty:ty)*) -> $ret:ty;)+ }) => {
        #[async_trait]
        pub trait $trait_name: Send + Sync {
            $(
                async fn $name(&self, $( $arg : $ty ),*) -> Result<$ret>;
            )+
        }

        #[async_trait]
        impl<T> $trait_name for T
        where
            T: AdminStore + ?Sized,
        {
            $(
                async fn $name(&self, $( $arg : $ty ),*) -> Result<$ret> {
                    AdminStore::$name(self, $( $arg ),*).await
                }
            )+
        }
    };
}

define_admin_store_facet!(IdentityStore {
    async fn insert_portal_user(&self, user: &PortalUserRecord) -> PortalUserRecord;
    async fn list_portal_users(&self) -> Vec<PortalUserRecord>;
    async fn find_portal_user_by_email(&self, email: &str) -> Option<PortalUserRecord>;
    async fn find_portal_user_by_id(&self, user_id: &str) -> Option<PortalUserRecord>;
    async fn delete_portal_user(&self, user_id: &str) -> bool;
    async fn insert_admin_user(&self, user: &AdminUserRecord) -> AdminUserRecord;
    async fn list_admin_users(&self) -> Vec<AdminUserRecord>;
    async fn find_admin_user_by_email(&self, email: &str) -> Option<AdminUserRecord>;
    async fn find_admin_user_by_id(&self, user_id: &str) -> Option<AdminUserRecord>;
    async fn delete_admin_user(&self, user_id: &str) -> bool;
    async fn insert_gateway_api_key(&self, record: &GatewayApiKeyRecord) -> GatewayApiKeyRecord;
    async fn list_gateway_api_keys(&self) -> Vec<GatewayApiKeyRecord>;
    async fn find_gateway_api_key(&self, hashed_key: &str) -> Option<GatewayApiKeyRecord>;
    async fn delete_gateway_api_key(&self, hashed_key: &str) -> bool;
    async fn insert_api_key_group(&self, record: &ApiKeyGroupRecord) -> ApiKeyGroupRecord;
    async fn list_api_key_groups(&self) -> Vec<ApiKeyGroupRecord>;
    async fn find_api_key_group(&self, group_id: &str) -> Option<ApiKeyGroupRecord>;
    async fn delete_api_key_group(&self, group_id: &str) -> bool;
});

define_admin_store_facet!(TenantStore {
    async fn insert_tenant(&self, tenant: &Tenant) -> Tenant;
    async fn list_tenants(&self) -> Vec<Tenant>;
    async fn find_tenant(&self, tenant_id: &str) -> Option<Tenant>;
    async fn delete_tenant(&self, tenant_id: &str) -> bool;
    async fn insert_project(&self, project: &Project) -> Project;
    async fn list_projects(&self) -> Vec<Project>;
    async fn find_project(&self, project_id: &str) -> Option<Project>;
    async fn delete_project(&self, project_id: &str) -> bool;
    async fn upsert_project_membership(&self, membership: &ProjectMembershipRecord) -> ProjectMembershipRecord;
    async fn find_project_membership(&self, project_id: &str) -> Option<ProjectMembershipRecord>;
});

define_admin_store_facet!(CatalogStore {
    async fn insert_channel(&self, channel: &Channel) -> Channel;
    async fn list_channels(&self) -> Vec<Channel>;
    async fn delete_channel(&self, channel_id: &str) -> bool;
    async fn insert_provider(&self, provider: &ProxyProvider) -> ProxyProvider;
    async fn list_providers(&self) -> Vec<ProxyProvider>;
    async fn find_provider(&self, provider_id: &str) -> Option<ProxyProvider>;
    async fn delete_provider(&self, provider_id: &str) -> bool;
    async fn insert_model(&self, model: &ModelCatalogEntry) -> ModelCatalogEntry;
    async fn list_models(&self) -> Vec<ModelCatalogEntry>;
    async fn find_model(&self, external_name: &str) -> Option<ModelCatalogEntry>;
    async fn delete_model(&self, external_name: &str) -> bool;
    async fn delete_model_variant(&self, external_name: &str, provider_id: &str) -> bool;
    async fn insert_channel_model(&self, record: &ChannelModelRecord) -> ChannelModelRecord;
    async fn list_channel_models(&self) -> Vec<ChannelModelRecord>;
    async fn delete_channel_model(&self, channel_id: &str, model_id: &str) -> bool;
    async fn insert_model_price(&self, record: &ModelPriceRecord) -> ModelPriceRecord;
    async fn list_model_prices(&self) -> Vec<ModelPriceRecord>;
    async fn delete_model_price(&self, channel_id: &str, model_id: &str, proxy_provider_id: &str) -> bool;
});

define_admin_store_facet!(CredentialStore {
    async fn insert_credential(&self, credential: &UpstreamCredential) -> UpstreamCredential;
    async fn insert_encrypted_credential(&self, credential: &UpstreamCredential, envelope: &SecretEnvelope) -> UpstreamCredential;
    async fn list_credentials(&self) -> Vec<UpstreamCredential>;
    async fn find_credential(&self, tenant_id: &str, provider_id: &str, key_reference: &str) -> Option<UpstreamCredential>;
    async fn find_credential_envelope(&self, tenant_id: &str, provider_id: &str, key_reference: &str) -> Option<SecretEnvelope>;
    async fn find_provider_credential(&self, tenant_id: &str, provider_id: &str) -> Option<UpstreamCredential>;
    async fn delete_credential(&self, tenant_id: &str, provider_id: &str, key_reference: &str) -> bool;
});

define_admin_store_facet!(RoutingStore {
    async fn insert_routing_policy(&self, policy: &RoutingPolicy) -> RoutingPolicy;
    async fn list_routing_policies(&self) -> Vec<RoutingPolicy>;
    async fn insert_routing_profile(&self, profile: &RoutingProfileRecord) -> RoutingProfileRecord;
    async fn list_routing_profiles(&self) -> Vec<RoutingProfileRecord>;
    async fn find_routing_profile(&self, profile_id: &str) -> Option<RoutingProfileRecord>;
    async fn insert_compiled_routing_snapshot(&self, snapshot: &CompiledRoutingSnapshotRecord) -> CompiledRoutingSnapshotRecord;
    async fn list_compiled_routing_snapshots(&self) -> Vec<CompiledRoutingSnapshotRecord>;
    async fn insert_project_routing_preferences(&self, preferences: &ProjectRoutingPreferences) -> ProjectRoutingPreferences;
    async fn find_project_routing_preferences(&self, project_id: &str) -> Option<ProjectRoutingPreferences>;
    async fn insert_routing_decision_log(&self, log: &RoutingDecisionLog) -> RoutingDecisionLog;
    async fn list_routing_decision_logs(&self) -> Vec<RoutingDecisionLog>;
    async fn insert_provider_health_snapshot(&self, snapshot: &ProviderHealthSnapshot) -> ProviderHealthSnapshot;
    async fn list_provider_health_snapshots(&self) -> Vec<ProviderHealthSnapshot>;
    async fn insert_rate_limit_policy(&self, policy: &RateLimitPolicy) -> RateLimitPolicy;
    async fn list_rate_limit_policies(&self) -> Vec<RateLimitPolicy>;
    async fn list_rate_limit_window_snapshots(&self) -> Vec<RateLimitWindowSnapshot>;
    async fn check_and_consume_rate_limit(&self, policy_id: &str, requested_requests: u64, limit_requests: u64, window_seconds: u64, now_ms: u64) -> RateLimitCheckResult;
});

define_admin_store_facet!(UsageStore {
    async fn insert_usage_record(&self, record: &UsageRecord) -> UsageRecord;
    async fn list_usage_records(&self) -> Vec<UsageRecord>;
});

define_admin_store_facet!(BillingStore {
    async fn insert_billing_event(&self, event: &BillingEventRecord) -> BillingEventRecord;
    async fn list_billing_events(&self) -> Vec<BillingEventRecord>;
    async fn insert_ledger_entry(&self, entry: &LedgerEntry) -> LedgerEntry;
    async fn list_ledger_entries(&self) -> Vec<LedgerEntry>;
    async fn insert_quota_policy(&self, policy: &QuotaPolicy) -> QuotaPolicy;
    async fn list_quota_policies(&self) -> Vec<QuotaPolicy>;
    async fn insert_coupon(&self, coupon: &CouponCampaign) -> CouponCampaign;
    async fn list_coupons(&self) -> Vec<CouponCampaign>;
    async fn find_coupon(&self, coupon_id: &str) -> Option<CouponCampaign>;
    async fn delete_coupon(&self, coupon_id: &str) -> bool;
    async fn insert_commerce_order(&self, order: &CommerceOrderRecord) -> CommerceOrderRecord;
    async fn list_commerce_orders(&self) -> Vec<CommerceOrderRecord>;
    async fn upsert_commerce_payment_event(&self, event: &CommercePaymentEventRecord) -> CommercePaymentEventRecord;
    async fn list_commerce_payment_events(&self) -> Vec<CommercePaymentEventRecord>;
    async fn find_commerce_payment_event_by_dedupe_key(&self, dedupe_key: &str) -> Option<CommercePaymentEventRecord>;
});

define_admin_store_facet!(JobStore {
    async fn insert_async_job(&self, record: &AsyncJobRecord) -> AsyncJobRecord;
    async fn list_async_jobs(&self) -> Vec<AsyncJobRecord>;
    async fn find_async_job(&self, job_id: &str) -> Option<AsyncJobRecord>;
    async fn insert_async_job_attempt(&self, record: &AsyncJobAttemptRecord) -> AsyncJobAttemptRecord;
    async fn list_async_job_attempts(&self, job_id: &str) -> Vec<AsyncJobAttemptRecord>;
    async fn insert_async_job_asset(&self, record: &AsyncJobAssetRecord) -> AsyncJobAssetRecord;
    async fn list_async_job_assets(&self, job_id: &str) -> Vec<AsyncJobAssetRecord>;
    async fn insert_async_job_callback(&self, record: &AsyncJobCallbackRecord) -> AsyncJobCallbackRecord;
    async fn list_async_job_callbacks(&self, job_id: &str) -> Vec<AsyncJobCallbackRecord>;
});

define_admin_store_facet!(ExtensionStore {
    async fn insert_extension_installation(&self, installation: &ExtensionInstallation) -> ExtensionInstallation;
    async fn list_extension_installations(&self) -> Vec<ExtensionInstallation>;
    async fn insert_extension_instance(&self, instance: &ExtensionInstance) -> ExtensionInstance;
    async fn list_extension_instances(&self) -> Vec<ExtensionInstance>;
    async fn upsert_service_runtime_node(&self, record: &ServiceRuntimeNodeRecord) -> ServiceRuntimeNodeRecord;
    async fn list_service_runtime_nodes(&self) -> Vec<ServiceRuntimeNodeRecord>;
    async fn insert_extension_runtime_rollout(&self, rollout: &ExtensionRuntimeRolloutRecord) -> ExtensionRuntimeRolloutRecord;
    async fn find_extension_runtime_rollout(&self, rollout_id: &str) -> Option<ExtensionRuntimeRolloutRecord>;
    async fn list_extension_runtime_rollouts(&self) -> Vec<ExtensionRuntimeRolloutRecord>;
    async fn insert_extension_runtime_rollout_participant(&self, participant: &ExtensionRuntimeRolloutParticipantRecord) -> ExtensionRuntimeRolloutParticipantRecord;
    async fn list_extension_runtime_rollout_participants(&self, rollout_id: &str) -> Vec<ExtensionRuntimeRolloutParticipantRecord>;
    async fn list_pending_extension_runtime_rollout_participants_for_node(&self, node_id: &str) -> Vec<ExtensionRuntimeRolloutParticipantRecord>;
    async fn transition_extension_runtime_rollout_participant(&self, rollout_id: &str, node_id: &str, from_status: &str, to_status: &str, message: Option<&str>, updated_at_ms: u64) -> bool;
    async fn insert_standalone_config_rollout(&self, rollout: &StandaloneConfigRolloutRecord) -> StandaloneConfigRolloutRecord;
    async fn find_standalone_config_rollout(&self, rollout_id: &str) -> Option<StandaloneConfigRolloutRecord>;
    async fn list_standalone_config_rollouts(&self) -> Vec<StandaloneConfigRolloutRecord>;
    async fn insert_standalone_config_rollout_participant(&self, participant: &StandaloneConfigRolloutParticipantRecord) -> StandaloneConfigRolloutParticipantRecord;
    async fn list_standalone_config_rollout_participants(&self, rollout_id: &str) -> Vec<StandaloneConfigRolloutParticipantRecord>;
    async fn list_pending_standalone_config_rollout_participants_for_node(&self, node_id: &str) -> Vec<StandaloneConfigRolloutParticipantRecord>;
    async fn transition_standalone_config_rollout_participant(&self, rollout_id: &str, node_id: &str, from_status: &str, to_status: &str, message: Option<&str>, updated_at_ms: u64) -> bool;
});
