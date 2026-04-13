use super::*;

use sdkwork_api_domain_identity::AdminAuditEventRecord;

#[async_trait]
impl AdminStore for SqliteAdminStore {
    fn dialect(&self) -> StorageDialect {
        StorageDialect::Sqlite
    }

    fn account_kernel_store(&self) -> Option<&dyn AccountKernelStore> {
        Some(self)
    }

    async fn insert_channel(&self, channel: &Channel) -> Result<Channel> {
        SqliteAdminStore::insert_channel(self, channel).await
    }
    async fn list_channels(&self) -> Result<Vec<Channel>> {
        SqliteAdminStore::list_channels(self).await
    }
    async fn delete_channel(&self, channel_id: &str) -> Result<bool> {
        SqliteAdminStore::delete_channel(self, channel_id).await
    }
    async fn insert_provider(&self, provider: &ProxyProvider) -> Result<ProxyProvider> {
        SqliteAdminStore::insert_provider(self, provider).await
    }
    async fn list_providers(&self) -> Result<Vec<ProxyProvider>> {
        SqliteAdminStore::list_providers(self).await
    }
    async fn list_providers_for_model(&self, model: &str) -> Result<Vec<ProxyProvider>> {
        SqliteAdminStore::list_providers_for_model(self, model).await
    }
    async fn find_provider(&self, provider_id: &str) -> Result<Option<ProxyProvider>> {
        SqliteAdminStore::find_provider(self, provider_id).await
    }
    async fn delete_provider(&self, provider_id: &str) -> Result<bool> {
        SqliteAdminStore::delete_provider(self, provider_id).await
    }
    async fn upsert_provider_account(
        &self,
        record: &ProviderAccountRecord,
    ) -> Result<ProviderAccountRecord> {
        SqliteAdminStore::upsert_provider_account(self, record).await
    }
    async fn list_provider_accounts(&self) -> Result<Vec<ProviderAccountRecord>> {
        SqliteAdminStore::list_provider_accounts(self).await
    }
    async fn find_provider_account(
        &self,
        provider_account_id: &str,
    ) -> Result<Option<ProviderAccountRecord>> {
        Ok(SqliteAdminStore::list_provider_accounts(self)
            .await?
            .into_iter()
            .find(|record| record.provider_account_id == provider_account_id))
    }
    async fn delete_provider_account(&self, provider_account_id: &str) -> Result<bool> {
        SqliteAdminStore::delete_provider_account(self, provider_account_id).await
    }
    async fn insert_credential(
        &self,
        credential: &UpstreamCredential,
    ) -> Result<UpstreamCredential> {
        SqliteAdminStore::insert_credential(self, credential).await
    }
    async fn insert_encrypted_credential(
        &self,
        credential: &UpstreamCredential,
        envelope: &SecretEnvelope,
    ) -> Result<UpstreamCredential> {
        SqliteAdminStore::insert_encrypted_credential(self, credential, envelope).await
    }
    async fn list_credentials(&self) -> Result<Vec<UpstreamCredential>> {
        SqliteAdminStore::list_credentials(self).await
    }
    async fn list_credentials_for_tenant(
        &self,
        tenant_id: &str,
    ) -> Result<Vec<UpstreamCredential>> {
        SqliteAdminStore::list_credentials_for_tenant(self, tenant_id).await
    }
    async fn list_credentials_for_provider(
        &self,
        provider_id: &str,
    ) -> Result<Vec<UpstreamCredential>> {
        SqliteAdminStore::list_credentials_for_provider(self, provider_id).await
    }
    async fn find_credential(
        &self,
        tenant_id: &str,
        provider_id: &str,
        key_reference: &str,
    ) -> Result<Option<UpstreamCredential>> {
        SqliteAdminStore::find_credential(self, tenant_id, provider_id, key_reference).await
    }
    async fn find_credential_envelope(
        &self,
        tenant_id: &str,
        provider_id: &str,
        key_reference: &str,
    ) -> Result<Option<SecretEnvelope>> {
        SqliteAdminStore::find_credential_envelope(self, tenant_id, provider_id, key_reference)
            .await
    }
    async fn find_provider_credential(
        &self,
        tenant_id: &str,
        provider_id: &str,
    ) -> Result<Option<UpstreamCredential>> {
        SqliteAdminStore::find_provider_credential(self, tenant_id, provider_id).await
    }
    async fn delete_credential(
        &self,
        tenant_id: &str,
        provider_id: &str,
        key_reference: &str,
    ) -> Result<bool> {
        SqliteAdminStore::delete_credential(self, tenant_id, provider_id, key_reference).await
    }
    async fn upsert_official_provider_config(
        &self,
        config: &OfficialProviderConfig,
    ) -> Result<OfficialProviderConfig> {
        SqliteAdminStore::upsert_official_provider_config(self, config).await
    }
    async fn list_official_provider_configs(&self) -> Result<Vec<OfficialProviderConfig>> {
        SqliteAdminStore::list_official_provider_configs(self).await
    }
    async fn find_official_provider_config(
        &self,
        provider_id: &str,
    ) -> Result<Option<OfficialProviderConfig>> {
        SqliteAdminStore::find_official_provider_config(self, provider_id).await
    }
    async fn insert_model(&self, model: &ModelCatalogEntry) -> Result<ModelCatalogEntry> {
        SqliteAdminStore::insert_model(self, model).await
    }
    async fn list_models(&self) -> Result<Vec<ModelCatalogEntry>> {
        SqliteAdminStore::list_models(self).await
    }
    async fn list_models_for_external_name(
        &self,
        external_name: &str,
    ) -> Result<Vec<ModelCatalogEntry>> {
        SqliteAdminStore::list_models_for_external_name(self, external_name).await
    }
    async fn find_any_model(&self) -> Result<Option<ModelCatalogEntry>> {
        SqliteAdminStore::find_any_model(self).await
    }
    async fn find_model(&self, external_name: &str) -> Result<Option<ModelCatalogEntry>> {
        SqliteAdminStore::find_model(self, external_name).await
    }
    async fn delete_model(&self, external_name: &str) -> Result<bool> {
        SqliteAdminStore::delete_model(self, external_name).await
    }
    async fn delete_model_variant(&self, external_name: &str, provider_id: &str) -> Result<bool> {
        SqliteAdminStore::delete_model_variant(self, external_name, provider_id).await
    }
    async fn insert_channel_model(
        &self,
        record: &ChannelModelRecord,
    ) -> Result<ChannelModelRecord> {
        SqliteAdminStore::insert_channel_model(self, record).await
    }
    async fn list_channel_models(&self) -> Result<Vec<ChannelModelRecord>> {
        SqliteAdminStore::list_channel_models(self).await
    }
    async fn delete_channel_model(&self, channel_id: &str, model_id: &str) -> Result<bool> {
        SqliteAdminStore::delete_channel_model(self, channel_id, model_id).await
    }
    async fn upsert_provider_model(
        &self,
        record: &ProviderModelRecord,
    ) -> Result<ProviderModelRecord> {
        SqliteAdminStore::upsert_provider_model(self, record).await
    }
    async fn list_provider_models(&self) -> Result<Vec<ProviderModelRecord>> {
        SqliteAdminStore::list_provider_models(self).await
    }
    async fn delete_provider_model(
        &self,
        proxy_provider_id: &str,
        channel_id: &str,
        model_id: &str,
    ) -> Result<bool> {
        SqliteAdminStore::delete_provider_model(self, proxy_provider_id, channel_id, model_id).await
    }
    async fn insert_model_price(&self, record: &ModelPriceRecord) -> Result<ModelPriceRecord> {
        SqliteAdminStore::insert_model_price(self, record).await
    }
    async fn list_model_prices(&self) -> Result<Vec<ModelPriceRecord>> {
        SqliteAdminStore::list_model_prices(self).await
    }
    async fn delete_model_price(
        &self,
        channel_id: &str,
        model_id: &str,
        proxy_provider_id: &str,
    ) -> Result<bool> {
        SqliteAdminStore::delete_model_price(self, channel_id, model_id, proxy_provider_id).await
    }
    async fn insert_routing_policy(&self, policy: &RoutingPolicy) -> Result<RoutingPolicy> {
        SqliteAdminStore::insert_routing_policy(self, policy).await
    }
    async fn list_routing_policies(&self) -> Result<Vec<RoutingPolicy>> {
        SqliteAdminStore::list_routing_policies(self).await
    }
    async fn insert_routing_profile(
        &self,
        profile: &RoutingProfileRecord,
    ) -> Result<RoutingProfileRecord> {
        SqliteAdminStore::insert_routing_profile(self, profile).await
    }
    async fn list_routing_profiles(&self) -> Result<Vec<RoutingProfileRecord>> {
        SqliteAdminStore::list_routing_profiles(self).await
    }
    async fn find_routing_profile(&self, profile_id: &str) -> Result<Option<RoutingProfileRecord>> {
        SqliteAdminStore::find_routing_profile(self, profile_id).await
    }
    async fn insert_compiled_routing_snapshot(
        &self,
        snapshot: &CompiledRoutingSnapshotRecord,
    ) -> Result<CompiledRoutingSnapshotRecord> {
        SqliteAdminStore::insert_compiled_routing_snapshot(self, snapshot).await
    }
    async fn list_compiled_routing_snapshots(&self) -> Result<Vec<CompiledRoutingSnapshotRecord>> {
        SqliteAdminStore::list_compiled_routing_snapshots(self).await
    }
    async fn insert_project_routing_preferences(
        &self,
        preferences: &ProjectRoutingPreferences,
    ) -> Result<ProjectRoutingPreferences> {
        SqliteAdminStore::insert_project_routing_preferences(self, preferences).await
    }
    async fn find_project_routing_preferences(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectRoutingPreferences>> {
        SqliteAdminStore::find_project_routing_preferences(self, project_id).await
    }
    async fn insert_routing_decision_log(
        &self,
        log: &RoutingDecisionLog,
    ) -> Result<RoutingDecisionLog> {
        SqliteAdminStore::insert_routing_decision_log(self, log).await
    }
    async fn list_routing_decision_logs(&self) -> Result<Vec<RoutingDecisionLog>> {
        SqliteAdminStore::list_routing_decision_logs(self).await
    }
    async fn list_routing_decision_logs_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<RoutingDecisionLog>> {
        SqliteAdminStore::list_routing_decision_logs_for_project(self, project_id).await
    }
    async fn find_latest_routing_decision_log_for_project(
        &self,
        project_id: &str,
    ) -> Result<Option<RoutingDecisionLog>> {
        SqliteAdminStore::find_latest_routing_decision_log_for_project(self, project_id).await
    }
    async fn insert_provider_health_snapshot(
        &self,
        snapshot: &ProviderHealthSnapshot,
    ) -> Result<ProviderHealthSnapshot> {
        SqliteAdminStore::insert_provider_health_snapshot(self, snapshot).await
    }
    async fn list_provider_health_snapshots(&self) -> Result<Vec<ProviderHealthSnapshot>> {
        SqliteAdminStore::list_provider_health_snapshots(self).await
    }
    async fn insert_usage_record(&self, record: &UsageRecord) -> Result<UsageRecord> {
        SqliteAdminStore::insert_usage_record(self, record).await
    }
    async fn list_usage_records(&self) -> Result<Vec<UsageRecord>> {
        SqliteAdminStore::list_usage_records(self).await
    }
    async fn list_usage_records_for_project(&self, project_id: &str) -> Result<Vec<UsageRecord>> {
        SqliteAdminStore::list_usage_records_for_project(self, project_id).await
    }
    async fn find_latest_usage_record_for_project(
        &self,
        project_id: &str,
    ) -> Result<Option<UsageRecord>> {
        SqliteAdminStore::find_latest_usage_record_for_project(self, project_id).await
    }
    async fn insert_billing_event(&self, event: &BillingEventRecord) -> Result<BillingEventRecord> {
        SqliteAdminStore::insert_billing_event(self, event).await
    }
    async fn list_billing_events(&self) -> Result<Vec<BillingEventRecord>> {
        SqliteAdminStore::list_billing_events(self).await
    }
    async fn insert_ledger_entry(&self, entry: &LedgerEntry) -> Result<LedgerEntry> {
        SqliteAdminStore::insert_ledger_entry(self, entry).await
    }
    async fn list_ledger_entries(&self) -> Result<Vec<LedgerEntry>> {
        SqliteAdminStore::list_ledger_entries(self).await
    }
    async fn list_ledger_entries_for_project(&self, project_id: &str) -> Result<Vec<LedgerEntry>> {
        SqliteAdminStore::list_ledger_entries_for_project(self, project_id).await
    }
    async fn insert_quota_policy(&self, policy: &QuotaPolicy) -> Result<QuotaPolicy> {
        SqliteAdminStore::insert_quota_policy(self, policy).await
    }
    async fn list_quota_policies(&self) -> Result<Vec<QuotaPolicy>> {
        SqliteAdminStore::list_quota_policies(self).await
    }
    async fn list_quota_policies_for_project(&self, project_id: &str) -> Result<Vec<QuotaPolicy>> {
        SqliteAdminStore::list_quota_policies_for_project(self, project_id).await
    }
    async fn delete_quota_policy(&self, policy_id: &str) -> Result<bool> {
        SqliteAdminStore::delete_quota_policy(self, policy_id).await
    }
    async fn insert_rate_limit_policy(&self, policy: &RateLimitPolicy) -> Result<RateLimitPolicy> {
        SqliteAdminStore::insert_rate_limit_policy(self, policy).await
    }
    async fn list_rate_limit_policies(&self) -> Result<Vec<RateLimitPolicy>> {
        SqliteAdminStore::list_rate_limit_policies(self).await
    }
    async fn list_rate_limit_window_snapshots(&self) -> Result<Vec<RateLimitWindowSnapshot>> {
        SqliteAdminStore::list_rate_limit_window_snapshots(self).await
    }
    async fn list_rate_limit_policies_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<RateLimitPolicy>> {
        SqliteAdminStore::list_rate_limit_policies_for_project(self, project_id).await
    }
    async fn check_and_consume_rate_limit(
        &self,
        policy_id: &str,
        requested_requests: u64,
        limit_requests: u64,
        window_seconds: u64,
        now_ms: u64,
    ) -> Result<RateLimitCheckResult> {
        SqliteAdminStore::check_and_consume_rate_limit(
            self,
            policy_id,
            requested_requests,
            limit_requests,
            window_seconds,
            now_ms,
        )
        .await
    }
    async fn insert_tenant(&self, tenant: &Tenant) -> Result<Tenant> {
        SqliteAdminStore::insert_tenant(self, tenant).await
    }
    async fn list_tenants(&self) -> Result<Vec<Tenant>> {
        SqliteAdminStore::list_tenants(self).await
    }
    async fn find_tenant(&self, tenant_id: &str) -> Result<Option<Tenant>> {
        SqliteAdminStore::find_tenant(self, tenant_id).await
    }
    async fn delete_tenant(&self, tenant_id: &str) -> Result<bool> {
        SqliteAdminStore::delete_tenant(self, tenant_id).await
    }
    async fn insert_project(&self, project: &Project) -> Result<Project> {
        SqliteAdminStore::insert_project(self, project).await
    }
    async fn list_projects(&self) -> Result<Vec<Project>> {
        SqliteAdminStore::list_projects(self).await
    }
    async fn find_project(&self, project_id: &str) -> Result<Option<Project>> {
        SqliteAdminStore::find_project(self, project_id).await
    }
    async fn delete_project(&self, project_id: &str) -> Result<bool> {
        SqliteAdminStore::delete_project(self, project_id).await
    }
    async fn insert_coupon_template_record(
        &self,
        record: &CouponTemplateRecord,
    ) -> Result<CouponTemplateRecord> {
        <Self as MarketingStore>::insert_coupon_template_record(self, record).await
    }
    async fn list_coupon_template_records(&self) -> Result<Vec<CouponTemplateRecord>> {
        <Self as MarketingStore>::list_coupon_template_records(self).await
    }
    async fn find_coupon_template_record(
        &self,
        coupon_template_id: &str,
    ) -> Result<Option<CouponTemplateRecord>> {
        <Self as MarketingStore>::find_coupon_template_record(self, coupon_template_id).await
    }
    async fn find_coupon_template_record_by_template_key(
        &self,
        template_key: &str,
    ) -> Result<Option<CouponTemplateRecord>> {
        <Self as MarketingStore>::find_coupon_template_record_by_template_key(self, template_key)
            .await
    }
    async fn insert_coupon_template_lifecycle_audit_record(
        &self,
        record: &CouponTemplateLifecycleAuditRecord,
    ) -> Result<CouponTemplateLifecycleAuditRecord> {
        <Self as MarketingStore>::insert_coupon_template_lifecycle_audit_record(self, record).await
    }
    async fn list_coupon_template_lifecycle_audit_records(
        &self,
    ) -> Result<Vec<CouponTemplateLifecycleAuditRecord>> {
        <Self as MarketingStore>::list_coupon_template_lifecycle_audit_records(self).await
    }
    async fn list_coupon_template_lifecycle_audit_records_for_template(
        &self,
        coupon_template_id: &str,
    ) -> Result<Vec<CouponTemplateLifecycleAuditRecord>> {
        <Self as MarketingStore>::list_coupon_template_lifecycle_audit_records_for_template(
            self,
            coupon_template_id,
        )
        .await
    }
    async fn insert_marketing_campaign_record(
        &self,
        record: &MarketingCampaignRecord,
    ) -> Result<MarketingCampaignRecord> {
        <Self as MarketingStore>::insert_marketing_campaign_record(self, record).await
    }
    async fn list_marketing_campaign_records(&self) -> Result<Vec<MarketingCampaignRecord>> {
        <Self as MarketingStore>::list_marketing_campaign_records(self).await
    }

    async fn find_marketing_campaign_record(
        &self,
        marketing_campaign_id: &str,
    ) -> Result<Option<MarketingCampaignRecord>> {
        <Self as MarketingStore>::find_marketing_campaign_record(self, marketing_campaign_id).await
    }
    async fn list_marketing_campaign_records_for_template(
        &self,
        coupon_template_id: &str,
    ) -> Result<Vec<MarketingCampaignRecord>> {
        <Self as MarketingStore>::list_marketing_campaign_records_for_template(
            self,
            coupon_template_id,
        )
        .await
    }
    async fn insert_marketing_campaign_lifecycle_audit_record(
        &self,
        record: &MarketingCampaignLifecycleAuditRecord,
    ) -> Result<MarketingCampaignLifecycleAuditRecord> {
        <Self as MarketingStore>::insert_marketing_campaign_lifecycle_audit_record(self, record)
            .await
    }
    async fn list_marketing_campaign_lifecycle_audit_records(
        &self,
    ) -> Result<Vec<MarketingCampaignLifecycleAuditRecord>> {
        <Self as MarketingStore>::list_marketing_campaign_lifecycle_audit_records(self).await
    }
    async fn list_marketing_campaign_lifecycle_audit_records_for_campaign(
        &self,
        marketing_campaign_id: &str,
    ) -> Result<Vec<MarketingCampaignLifecycleAuditRecord>> {
        <Self as MarketingStore>::list_marketing_campaign_lifecycle_audit_records_for_campaign(
            self,
            marketing_campaign_id,
        )
        .await
    }
    async fn insert_campaign_budget_record(
        &self,
        record: &CampaignBudgetRecord,
    ) -> Result<CampaignBudgetRecord> {
        <Self as MarketingStore>::insert_campaign_budget_record(self, record).await
    }
    async fn list_campaign_budget_records(&self) -> Result<Vec<CampaignBudgetRecord>> {
        <Self as MarketingStore>::list_campaign_budget_records(self).await
    }
    async fn find_campaign_budget_record(
        &self,
        campaign_budget_id: &str,
    ) -> Result<Option<CampaignBudgetRecord>> {
        <Self as MarketingStore>::find_campaign_budget_record(self, campaign_budget_id).await
    }
    async fn list_campaign_budget_records_for_campaign(
        &self,
        marketing_campaign_id: &str,
    ) -> Result<Vec<CampaignBudgetRecord>> {
        <Self as MarketingStore>::list_campaign_budget_records_for_campaign(
            self,
            marketing_campaign_id,
        )
        .await
    }
    async fn insert_campaign_budget_lifecycle_audit_record(
        &self,
        record: &CampaignBudgetLifecycleAuditRecord,
    ) -> Result<CampaignBudgetLifecycleAuditRecord> {
        <Self as MarketingStore>::insert_campaign_budget_lifecycle_audit_record(self, record).await
    }
    async fn list_campaign_budget_lifecycle_audit_records(
        &self,
    ) -> Result<Vec<CampaignBudgetLifecycleAuditRecord>> {
        <Self as MarketingStore>::list_campaign_budget_lifecycle_audit_records(self).await
    }
    async fn list_campaign_budget_lifecycle_audit_records_for_budget(
        &self,
        campaign_budget_id: &str,
    ) -> Result<Vec<CampaignBudgetLifecycleAuditRecord>> {
        <Self as MarketingStore>::list_campaign_budget_lifecycle_audit_records_for_budget(
            self,
            campaign_budget_id,
        )
        .await
    }
    async fn insert_coupon_code_record(
        &self,
        record: &CouponCodeRecord,
    ) -> Result<CouponCodeRecord> {
        <Self as MarketingStore>::insert_coupon_code_record(self, record).await
    }
    async fn list_coupon_code_records(&self) -> Result<Vec<CouponCodeRecord>> {
        <Self as MarketingStore>::list_coupon_code_records(self).await
    }
    async fn find_coupon_code_record(
        &self,
        coupon_code_id: &str,
    ) -> Result<Option<CouponCodeRecord>> {
        <Self as MarketingStore>::find_coupon_code_record(self, coupon_code_id).await
    }
    async fn find_coupon_code_record_by_value(
        &self,
        code_value: &str,
    ) -> Result<Option<CouponCodeRecord>> {
        <Self as MarketingStore>::find_coupon_code_record_by_value(self, code_value).await
    }
    async fn list_redeemable_coupon_code_records_at(
        &self,
        now_ms: u64,
    ) -> Result<Vec<CouponCodeRecord>> {
        <Self as MarketingStore>::list_redeemable_coupon_code_records_at(self, now_ms).await
    }
    async fn insert_coupon_code_lifecycle_audit_record(
        &self,
        record: &CouponCodeLifecycleAuditRecord,
    ) -> Result<CouponCodeLifecycleAuditRecord> {
        <Self as MarketingStore>::insert_coupon_code_lifecycle_audit_record(self, record).await
    }
    async fn list_coupon_code_lifecycle_audit_records(
        &self,
    ) -> Result<Vec<CouponCodeLifecycleAuditRecord>> {
        <Self as MarketingStore>::list_coupon_code_lifecycle_audit_records(self).await
    }
    async fn list_coupon_code_lifecycle_audit_records_for_code(
        &self,
        coupon_code_id: &str,
    ) -> Result<Vec<CouponCodeLifecycleAuditRecord>> {
        <Self as MarketingStore>::list_coupon_code_lifecycle_audit_records_for_code(
            self,
            coupon_code_id,
        )
        .await
    }
    async fn insert_coupon_reservation_record(
        &self,
        record: &CouponReservationRecord,
    ) -> Result<CouponReservationRecord> {
        <Self as MarketingStore>::insert_coupon_reservation_record(self, record).await
    }
    async fn list_coupon_reservation_records(&self) -> Result<Vec<CouponReservationRecord>> {
        <Self as MarketingStore>::list_coupon_reservation_records(self).await
    }
    async fn find_coupon_reservation_record(
        &self,
        coupon_reservation_id: &str,
    ) -> Result<Option<CouponReservationRecord>> {
        <Self as MarketingStore>::find_coupon_reservation_record(self, coupon_reservation_id).await
    }
    async fn list_active_coupon_reservation_records_at(
        &self,
        now_ms: u64,
    ) -> Result<Vec<CouponReservationRecord>> {
        <Self as MarketingStore>::list_active_coupon_reservation_records_at(self, now_ms).await
    }
    async fn insert_coupon_redemption_record(
        &self,
        record: &CouponRedemptionRecord,
    ) -> Result<CouponRedemptionRecord> {
        <Self as MarketingStore>::insert_coupon_redemption_record(self, record).await
    }
    async fn list_coupon_redemption_records(&self) -> Result<Vec<CouponRedemptionRecord>> {
        <Self as MarketingStore>::list_coupon_redemption_records(self).await
    }
    async fn find_coupon_redemption_record(
        &self,
        coupon_redemption_id: &str,
    ) -> Result<Option<CouponRedemptionRecord>> {
        <Self as MarketingStore>::find_coupon_redemption_record(self, coupon_redemption_id).await
    }
    async fn insert_coupon_rollback_record(
        &self,
        record: &CouponRollbackRecord,
    ) -> Result<CouponRollbackRecord> {
        <Self as MarketingStore>::insert_coupon_rollback_record(self, record).await
    }
    async fn list_coupon_rollback_records(&self) -> Result<Vec<CouponRollbackRecord>> {
        <Self as MarketingStore>::list_coupon_rollback_records(self).await
    }
    async fn insert_marketing_outbox_event_record(
        &self,
        record: &MarketingOutboxEventRecord,
    ) -> Result<MarketingOutboxEventRecord> {
        <Self as MarketingStore>::insert_marketing_outbox_event_record(self, record).await
    }
    async fn list_marketing_outbox_event_records(&self) -> Result<Vec<MarketingOutboxEventRecord>> {
        <Self as MarketingStore>::list_marketing_outbox_event_records(self).await
    }
    async fn reserve_coupon_redemption_atomic(
        &self,
        command: &AtomicCouponReservationCommand,
    ) -> Result<AtomicCouponReservationResult> {
        sdkwork_api_storage_core::execute_atomic_coupon_reservation(self, command).await
    }
    async fn confirm_coupon_redemption_atomic(
        &self,
        command: &AtomicCouponConfirmationCommand,
    ) -> Result<AtomicCouponConfirmationResult> {
        sdkwork_api_storage_core::execute_atomic_coupon_confirmation(self, command).await
    }
    async fn release_coupon_reservation_atomic(
        &self,
        command: &AtomicCouponReleaseCommand,
    ) -> Result<AtomicCouponReleaseResult> {
        sdkwork_api_storage_core::execute_atomic_coupon_release(self, command).await
    }
    async fn rollback_coupon_redemption_atomic(
        &self,
        command: &AtomicCouponRollbackCommand,
    ) -> Result<AtomicCouponRollbackResult> {
        sdkwork_api_storage_core::execute_atomic_coupon_rollback(self, command).await
    }
    async fn compensate_coupon_rollback_atomic(
        &self,
        command: &AtomicCouponRollbackCompensationCommand,
    ) -> Result<AtomicCouponRollbackCompensationResult> {
        sdkwork_api_storage_core::execute_atomic_coupon_rollback_compensation(self, command).await
    }
    async fn insert_catalog_publication_lifecycle_audit_record(
        &self,
        record: &CatalogPublicationLifecycleAuditRecord,
    ) -> Result<CatalogPublicationLifecycleAuditRecord> {
        SqliteAdminStore::insert_catalog_publication_lifecycle_audit_record(self, record).await
    }
    async fn list_catalog_publication_lifecycle_audit_records(
        &self,
    ) -> Result<Vec<CatalogPublicationLifecycleAuditRecord>> {
        SqliteAdminStore::list_catalog_publication_lifecycle_audit_records(self).await
    }
    async fn insert_commerce_order(
        &self,
        order: &CommerceOrderRecord,
    ) -> Result<CommerceOrderRecord> {
        SqliteAdminStore::insert_commerce_order(self, order).await
    }
    async fn list_commerce_orders(&self) -> Result<Vec<CommerceOrderRecord>> {
        SqliteAdminStore::list_commerce_orders(self).await
    }
    async fn list_recent_commerce_orders(&self, limit: usize) -> Result<Vec<CommerceOrderRecord>> {
        SqliteAdminStore::list_recent_commerce_orders(self, limit).await
    }
    async fn list_commerce_orders_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<CommerceOrderRecord>> {
        SqliteAdminStore::list_commerce_orders_for_project(self, project_id).await
    }
    async fn list_commerce_orders_for_project_after(
        &self,
        project_id: &str,
        last_order_updated_at_ms: u64,
        last_order_created_at_ms: u64,
        last_order_id: &str,
    ) -> Result<Vec<CommerceOrderRecord>> {
        SqliteAdminStore::list_commerce_orders_for_project_after(
            self,
            project_id,
            last_order_updated_at_ms,
            last_order_created_at_ms,
            last_order_id,
        )
        .await
    }
    async fn upsert_commerce_payment_event(
        &self,
        event: &CommercePaymentEventRecord,
    ) -> Result<CommercePaymentEventRecord> {
        SqliteAdminStore::upsert_commerce_payment_event(self, event).await
    }
    async fn list_commerce_payment_events(&self) -> Result<Vec<CommercePaymentEventRecord>> {
        SqliteAdminStore::list_commerce_payment_events(self).await
    }
    async fn find_commerce_payment_event_by_dedupe_key(
        &self,
        dedupe_key: &str,
    ) -> Result<Option<CommercePaymentEventRecord>> {
        SqliteAdminStore::find_commerce_payment_event_by_dedupe_key(self, dedupe_key).await
    }
    async fn upsert_payment_method(
        &self,
        payment_method: &PaymentMethodRecord,
    ) -> Result<PaymentMethodRecord> {
        SqliteAdminStore::upsert_payment_method(self, payment_method).await
    }
    async fn list_payment_methods(&self) -> Result<Vec<PaymentMethodRecord>> {
        SqliteAdminStore::list_payment_methods(self).await
    }
    async fn find_payment_method(
        &self,
        payment_method_id: &str,
    ) -> Result<Option<PaymentMethodRecord>> {
        SqliteAdminStore::find_payment_method(self, payment_method_id).await
    }
    async fn delete_payment_method(&self, payment_method_id: &str) -> Result<bool> {
        SqliteAdminStore::delete_payment_method(self, payment_method_id).await
    }
    async fn upsert_payment_method_credential_binding(
        &self,
        binding: &PaymentMethodCredentialBindingRecord,
    ) -> Result<PaymentMethodCredentialBindingRecord> {
        SqliteAdminStore::upsert_payment_method_credential_binding(self, binding).await
    }
    async fn list_payment_method_credential_bindings(
        &self,
        payment_method_id: &str,
    ) -> Result<Vec<PaymentMethodCredentialBindingRecord>> {
        SqliteAdminStore::list_payment_method_credential_bindings(self, payment_method_id).await
    }
    async fn delete_payment_method_credential_binding(
        &self,
        payment_method_id: &str,
        binding_id: &str,
    ) -> Result<bool> {
        SqliteAdminStore::delete_payment_method_credential_binding(
            self,
            payment_method_id,
            binding_id,
        )
        .await
    }
    async fn upsert_commerce_payment_attempt(
        &self,
        attempt: &CommercePaymentAttemptRecord,
    ) -> Result<CommercePaymentAttemptRecord> {
        SqliteAdminStore::upsert_commerce_payment_attempt(self, attempt).await
    }
    async fn list_commerce_payment_attempts(&self) -> Result<Vec<CommercePaymentAttemptRecord>> {
        SqliteAdminStore::list_commerce_payment_attempts(self).await
    }
    async fn find_commerce_payment_attempt(
        &self,
        payment_attempt_id: &str,
    ) -> Result<Option<CommercePaymentAttemptRecord>> {
        SqliteAdminStore::find_commerce_payment_attempt(self, payment_attempt_id).await
    }
    async fn find_commerce_payment_attempt_by_idempotency_key(
        &self,
        idempotency_key: &str,
    ) -> Result<Option<CommercePaymentAttemptRecord>> {
        SqliteAdminStore::find_commerce_payment_attempt_by_idempotency_key(self, idempotency_key)
            .await
    }
    async fn list_commerce_payment_attempts_for_order(
        &self,
        order_id: &str,
    ) -> Result<Vec<CommercePaymentAttemptRecord>> {
        SqliteAdminStore::list_commerce_payment_attempts_for_order(self, order_id).await
    }
    async fn upsert_commerce_webhook_inbox(
        &self,
        record: &CommerceWebhookInboxRecord,
    ) -> Result<CommerceWebhookInboxRecord> {
        SqliteAdminStore::upsert_commerce_webhook_inbox(self, record).await
    }
    async fn list_commerce_webhook_inbox_records(&self) -> Result<Vec<CommerceWebhookInboxRecord>> {
        SqliteAdminStore::list_commerce_webhook_inbox_records(self).await
    }
    async fn find_commerce_webhook_inbox_by_dedupe_key(
        &self,
        dedupe_key: &str,
    ) -> Result<Option<CommerceWebhookInboxRecord>> {
        SqliteAdminStore::find_commerce_webhook_inbox_by_dedupe_key(self, dedupe_key).await
    }
    async fn insert_commerce_webhook_delivery_attempt(
        &self,
        record: &CommerceWebhookDeliveryAttemptRecord,
    ) -> Result<CommerceWebhookDeliveryAttemptRecord> {
        SqliteAdminStore::insert_commerce_webhook_delivery_attempt(self, record).await
    }
    async fn list_commerce_webhook_delivery_attempts(
        &self,
        webhook_inbox_id: &str,
    ) -> Result<Vec<CommerceWebhookDeliveryAttemptRecord>> {
        SqliteAdminStore::list_commerce_webhook_delivery_attempts(self, webhook_inbox_id).await
    }
    async fn upsert_commerce_refund(
        &self,
        refund: &CommerceRefundRecord,
    ) -> Result<CommerceRefundRecord> {
        SqliteAdminStore::upsert_commerce_refund(self, refund).await
    }
    async fn list_commerce_refunds(&self) -> Result<Vec<CommerceRefundRecord>> {
        SqliteAdminStore::list_commerce_refunds(self).await
    }
    async fn find_commerce_refund(&self, refund_id: &str) -> Result<Option<CommerceRefundRecord>> {
        SqliteAdminStore::find_commerce_refund(self, refund_id).await
    }
    async fn find_commerce_refund_by_idempotency_key(
        &self,
        idempotency_key: &str,
    ) -> Result<Option<CommerceRefundRecord>> {
        SqliteAdminStore::find_commerce_refund_by_idempotency_key(self, idempotency_key).await
    }
    async fn list_commerce_refunds_for_order(
        &self,
        order_id: &str,
    ) -> Result<Vec<CommerceRefundRecord>> {
        SqliteAdminStore::list_commerce_refunds_for_order(self, order_id).await
    }
    async fn insert_commerce_reconciliation_run(
        &self,
        record: &CommerceReconciliationRunRecord,
    ) -> Result<CommerceReconciliationRunRecord> {
        SqliteAdminStore::insert_commerce_reconciliation_run(self, record).await
    }
    async fn list_commerce_reconciliation_runs(
        &self,
    ) -> Result<Vec<CommerceReconciliationRunRecord>> {
        SqliteAdminStore::list_commerce_reconciliation_runs(self).await
    }
    async fn insert_commerce_reconciliation_item(
        &self,
        record: &CommerceReconciliationItemRecord,
    ) -> Result<CommerceReconciliationItemRecord> {
        SqliteAdminStore::insert_commerce_reconciliation_item(self, record).await
    }
    async fn list_commerce_reconciliation_items(
        &self,
        reconciliation_run_id: &str,
    ) -> Result<Vec<CommerceReconciliationItemRecord>> {
        SqliteAdminStore::list_commerce_reconciliation_items(self, reconciliation_run_id).await
    }
    async fn insert_async_job(&self, record: &AsyncJobRecord) -> Result<AsyncJobRecord> {
        SqliteAdminStore::insert_async_job(self, record).await
    }
    async fn list_async_jobs(&self) -> Result<Vec<AsyncJobRecord>> {
        SqliteAdminStore::list_async_jobs(self).await
    }
    async fn find_async_job(&self, job_id: &str) -> Result<Option<AsyncJobRecord>> {
        SqliteAdminStore::find_async_job(self, job_id).await
    }
    async fn insert_async_job_attempt(
        &self,
        record: &AsyncJobAttemptRecord,
    ) -> Result<AsyncJobAttemptRecord> {
        SqliteAdminStore::insert_async_job_attempt(self, record).await
    }
    async fn list_async_job_attempts(&self, job_id: &str) -> Result<Vec<AsyncJobAttemptRecord>> {
        SqliteAdminStore::list_async_job_attempts(self, job_id).await
    }
    async fn insert_async_job_asset(
        &self,
        record: &AsyncJobAssetRecord,
    ) -> Result<AsyncJobAssetRecord> {
        SqliteAdminStore::insert_async_job_asset(self, record).await
    }
    async fn list_async_job_assets(&self, job_id: &str) -> Result<Vec<AsyncJobAssetRecord>> {
        SqliteAdminStore::list_async_job_assets(self, job_id).await
    }
    async fn insert_async_job_callback(
        &self,
        record: &AsyncJobCallbackRecord,
    ) -> Result<AsyncJobCallbackRecord> {
        SqliteAdminStore::insert_async_job_callback(self, record).await
    }
    async fn list_async_job_callbacks(&self, job_id: &str) -> Result<Vec<AsyncJobCallbackRecord>> {
        SqliteAdminStore::list_async_job_callbacks(self, job_id).await
    }
    async fn upsert_project_membership(
        &self,
        membership: &ProjectMembershipRecord,
    ) -> Result<ProjectMembershipRecord> {
        SqliteAdminStore::upsert_project_membership(self, membership).await
    }
    async fn find_project_membership(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectMembershipRecord>> {
        SqliteAdminStore::find_project_membership(self, project_id).await
    }
    async fn delete_project_membership(&self, project_id: &str) -> Result<bool> {
        SqliteAdminStore::delete_project_membership(self, project_id).await
    }
    async fn insert_portal_user(&self, user: &PortalUserRecord) -> Result<PortalUserRecord> {
        SqliteAdminStore::insert_portal_user(self, user).await
    }
    async fn list_portal_users(&self) -> Result<Vec<PortalUserRecord>> {
        SqliteAdminStore::list_portal_users(self).await
    }
    async fn find_portal_user_by_email(&self, email: &str) -> Result<Option<PortalUserRecord>> {
        SqliteAdminStore::find_portal_user_by_email(self, email).await
    }
    async fn find_portal_user_by_id(&self, user_id: &str) -> Result<Option<PortalUserRecord>> {
        SqliteAdminStore::find_portal_user_by_id(self, user_id).await
    }
    async fn delete_portal_user(&self, user_id: &str) -> Result<bool> {
        SqliteAdminStore::delete_portal_user(self, user_id).await
    }
    async fn insert_admin_user(&self, user: &AdminUserRecord) -> Result<AdminUserRecord> {
        SqliteAdminStore::insert_admin_user(self, user).await
    }
    async fn list_admin_users(&self) -> Result<Vec<AdminUserRecord>> {
        SqliteAdminStore::list_admin_users(self).await
    }
    async fn find_admin_user_by_email(&self, email: &str) -> Result<Option<AdminUserRecord>> {
        SqliteAdminStore::find_admin_user_by_email(self, email).await
    }
    async fn find_admin_user_by_id(&self, user_id: &str) -> Result<Option<AdminUserRecord>> {
        SqliteAdminStore::find_admin_user_by_id(self, user_id).await
    }
    async fn delete_admin_user(&self, user_id: &str) -> Result<bool> {
        SqliteAdminStore::delete_admin_user(self, user_id).await
    }
    async fn insert_admin_audit_event(
        &self,
        record: &AdminAuditEventRecord,
    ) -> Result<AdminAuditEventRecord> {
        SqliteAdminStore::insert_admin_audit_event(self, record).await
    }
    async fn list_admin_audit_events(&self) -> Result<Vec<AdminAuditEventRecord>> {
        SqliteAdminStore::list_admin_audit_events(self).await
    }
    async fn insert_gateway_api_key(
        &self,
        record: &GatewayApiKeyRecord,
    ) -> Result<GatewayApiKeyRecord> {
        SqliteAdminStore::insert_gateway_api_key(self, record).await
    }
    async fn list_gateway_api_keys(&self) -> Result<Vec<GatewayApiKeyRecord>> {
        SqliteAdminStore::list_gateway_api_keys(self).await
    }
    async fn find_gateway_api_key(&self, hashed_key: &str) -> Result<Option<GatewayApiKeyRecord>> {
        SqliteAdminStore::find_gateway_api_key(self, hashed_key).await
    }
    async fn delete_gateway_api_key(&self, hashed_key: &str) -> Result<bool> {
        SqliteAdminStore::delete_gateway_api_key(self, hashed_key).await
    }
    async fn insert_api_key_group(&self, record: &ApiKeyGroupRecord) -> Result<ApiKeyGroupRecord> {
        SqliteAdminStore::insert_api_key_group(self, record).await
    }
    async fn list_api_key_groups(&self) -> Result<Vec<ApiKeyGroupRecord>> {
        SqliteAdminStore::list_api_key_groups(self).await
    }
    async fn find_api_key_group(&self, group_id: &str) -> Result<Option<ApiKeyGroupRecord>> {
        SqliteAdminStore::find_api_key_group(self, group_id).await
    }
    async fn delete_api_key_group(&self, group_id: &str) -> Result<bool> {
        SqliteAdminStore::delete_api_key_group(self, group_id).await
    }
    async fn insert_extension_installation(
        &self,
        installation: &ExtensionInstallation,
    ) -> Result<ExtensionInstallation> {
        SqliteAdminStore::insert_extension_installation(self, installation).await
    }
    async fn list_extension_installations(&self) -> Result<Vec<ExtensionInstallation>> {
        SqliteAdminStore::list_extension_installations(self).await
    }
    async fn insert_extension_instance(
        &self,
        instance: &ExtensionInstance,
    ) -> Result<ExtensionInstance> {
        SqliteAdminStore::insert_extension_instance(self, instance).await
    }
    async fn list_extension_instances(&self) -> Result<Vec<ExtensionInstance>> {
        SqliteAdminStore::list_extension_instances(self).await
    }
    async fn upsert_service_runtime_node(
        &self,
        record: &ServiceRuntimeNodeRecord,
    ) -> Result<ServiceRuntimeNodeRecord> {
        SqliteAdminStore::upsert_service_runtime_node(self, record).await
    }
    async fn list_service_runtime_nodes(&self) -> Result<Vec<ServiceRuntimeNodeRecord>> {
        SqliteAdminStore::list_service_runtime_nodes(self).await
    }
    async fn insert_extension_runtime_rollout(
        &self,
        rollout: &ExtensionRuntimeRolloutRecord,
    ) -> Result<ExtensionRuntimeRolloutRecord> {
        SqliteAdminStore::insert_extension_runtime_rollout(self, rollout).await
    }
    async fn find_extension_runtime_rollout(
        &self,
        rollout_id: &str,
    ) -> Result<Option<ExtensionRuntimeRolloutRecord>> {
        SqliteAdminStore::find_extension_runtime_rollout(self, rollout_id).await
    }
    async fn list_extension_runtime_rollouts(&self) -> Result<Vec<ExtensionRuntimeRolloutRecord>> {
        SqliteAdminStore::list_extension_runtime_rollouts(self).await
    }
    async fn insert_extension_runtime_rollout_participant(
        &self,
        participant: &ExtensionRuntimeRolloutParticipantRecord,
    ) -> Result<ExtensionRuntimeRolloutParticipantRecord> {
        SqliteAdminStore::insert_extension_runtime_rollout_participant(self, participant).await
    }
    async fn list_extension_runtime_rollout_participants(
        &self,
        rollout_id: &str,
    ) -> Result<Vec<ExtensionRuntimeRolloutParticipantRecord>> {
        SqliteAdminStore::list_extension_runtime_rollout_participants(self, rollout_id).await
    }
    async fn list_pending_extension_runtime_rollout_participants_for_node(
        &self,
        node_id: &str,
    ) -> Result<Vec<ExtensionRuntimeRolloutParticipantRecord>> {
        SqliteAdminStore::list_pending_extension_runtime_rollout_participants_for_node(
            self, node_id,
        )
        .await
    }
    async fn transition_extension_runtime_rollout_participant(
        &self,
        rollout_id: &str,
        node_id: &str,
        from_status: &str,
        to_status: &str,
        message: Option<&str>,
        updated_at_ms: u64,
    ) -> Result<bool> {
        SqliteAdminStore::transition_extension_runtime_rollout_participant(
            self,
            rollout_id,
            node_id,
            from_status,
            to_status,
            message,
            updated_at_ms,
        )
        .await
    }
    async fn insert_standalone_config_rollout(
        &self,
        rollout: &StandaloneConfigRolloutRecord,
    ) -> Result<StandaloneConfigRolloutRecord> {
        SqliteAdminStore::insert_standalone_config_rollout(self, rollout).await
    }
    async fn find_standalone_config_rollout(
        &self,
        rollout_id: &str,
    ) -> Result<Option<StandaloneConfigRolloutRecord>> {
        SqliteAdminStore::find_standalone_config_rollout(self, rollout_id).await
    }
    async fn list_standalone_config_rollouts(&self) -> Result<Vec<StandaloneConfigRolloutRecord>> {
        SqliteAdminStore::list_standalone_config_rollouts(self).await
    }
    async fn insert_standalone_config_rollout_participant(
        &self,
        participant: &StandaloneConfigRolloutParticipantRecord,
    ) -> Result<StandaloneConfigRolloutParticipantRecord> {
        SqliteAdminStore::insert_standalone_config_rollout_participant(self, participant).await
    }
    async fn list_standalone_config_rollout_participants(
        &self,
        rollout_id: &str,
    ) -> Result<Vec<StandaloneConfigRolloutParticipantRecord>> {
        SqliteAdminStore::list_standalone_config_rollout_participants(self, rollout_id).await
    }
    async fn list_pending_standalone_config_rollout_participants_for_node(
        &self,
        node_id: &str,
    ) -> Result<Vec<StandaloneConfigRolloutParticipantRecord>> {
        SqliteAdminStore::list_pending_standalone_config_rollout_participants_for_node(
            self, node_id,
        )
        .await
    }
    async fn transition_standalone_config_rollout_participant(
        &self,
        rollout_id: &str,
        node_id: &str,
        from_status: &str,
        to_status: &str,
        message: Option<&str>,
        updated_at_ms: u64,
    ) -> Result<bool> {
        SqliteAdminStore::transition_standalone_config_rollout_participant(
            self,
            rollout_id,
            node_id,
            from_status,
            to_status,
            message,
            updated_at_ms,
        )
        .await
    }
}
