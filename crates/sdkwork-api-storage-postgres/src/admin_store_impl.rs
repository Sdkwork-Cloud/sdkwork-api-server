use super::*;

use sdkwork_api_domain_identity::AdminAuditEventRecord;

#[async_trait]
impl AdminStore for PostgresAdminStore {
    fn dialect(&self) -> StorageDialect {
        StorageDialect::Postgres
    }

    fn account_kernel_store(&self) -> Option<&dyn AccountKernelStore> {
        Some(self)
    }

    async fn insert_channel(&self, channel: &Channel) -> Result<Channel> {
        PostgresAdminStore::insert_channel(self, channel).await
    }

    async fn list_channels(&self) -> Result<Vec<Channel>> {
        PostgresAdminStore::list_channels(self).await
    }

    async fn delete_channel(&self, channel_id: &str) -> Result<bool> {
        PostgresAdminStore::delete_channel(self, channel_id).await
    }

    async fn insert_provider(&self, provider: &ProxyProvider) -> Result<ProxyProvider> {
        PostgresAdminStore::insert_provider(self, provider).await
    }

    async fn list_providers(&self) -> Result<Vec<ProxyProvider>> {
        PostgresAdminStore::list_providers(self).await
    }

    async fn list_providers_for_model(&self, model: &str) -> Result<Vec<ProxyProvider>> {
        PostgresAdminStore::list_providers_for_model(self, model).await
    }

    async fn find_provider(&self, provider_id: &str) -> Result<Option<ProxyProvider>> {
        PostgresAdminStore::find_provider(self, provider_id).await
    }

    async fn delete_provider(&self, provider_id: &str) -> Result<bool> {
        PostgresAdminStore::delete_provider(self, provider_id).await
    }
    async fn upsert_provider_account(
        &self,
        record: &ProviderAccountRecord,
    ) -> Result<ProviderAccountRecord> {
        PostgresAdminStore::upsert_provider_account(self, record).await
    }
    async fn list_provider_accounts(&self) -> Result<Vec<ProviderAccountRecord>> {
        PostgresAdminStore::list_provider_accounts(self).await
    }
    async fn find_provider_account(
        &self,
        provider_account_id: &str,
    ) -> Result<Option<ProviderAccountRecord>> {
        Ok(PostgresAdminStore::list_provider_accounts(self)
            .await?
            .into_iter()
            .find(|record| record.provider_account_id == provider_account_id))
    }
    async fn delete_provider_account(&self, provider_account_id: &str) -> Result<bool> {
        PostgresAdminStore::delete_provider_account(self, provider_account_id).await
    }

    async fn insert_credential(
        &self,
        credential: &UpstreamCredential,
    ) -> Result<UpstreamCredential> {
        PostgresAdminStore::insert_credential(self, credential).await
    }

    async fn insert_encrypted_credential(
        &self,
        credential: &UpstreamCredential,
        envelope: &SecretEnvelope,
    ) -> Result<UpstreamCredential> {
        PostgresAdminStore::insert_encrypted_credential(self, credential, envelope).await
    }

    async fn list_credentials(&self) -> Result<Vec<UpstreamCredential>> {
        PostgresAdminStore::list_credentials(self).await
    }

    async fn list_credentials_for_tenant(
        &self,
        tenant_id: &str,
    ) -> Result<Vec<UpstreamCredential>> {
        PostgresAdminStore::list_credentials_for_tenant(self, tenant_id).await
    }

    async fn list_credentials_for_provider(
        &self,
        provider_id: &str,
    ) -> Result<Vec<UpstreamCredential>> {
        PostgresAdminStore::list_credentials_for_provider(self, provider_id).await
    }

    async fn find_credential(
        &self,
        tenant_id: &str,
        provider_id: &str,
        key_reference: &str,
    ) -> Result<Option<UpstreamCredential>> {
        PostgresAdminStore::find_credential(self, tenant_id, provider_id, key_reference).await
    }

    async fn find_credential_envelope(
        &self,
        tenant_id: &str,
        provider_id: &str,
        key_reference: &str,
    ) -> Result<Option<SecretEnvelope>> {
        PostgresAdminStore::find_credential_envelope(self, tenant_id, provider_id, key_reference)
            .await
    }

    async fn find_provider_credential(
        &self,
        tenant_id: &str,
        provider_id: &str,
    ) -> Result<Option<UpstreamCredential>> {
        PostgresAdminStore::find_provider_credential(self, tenant_id, provider_id).await
    }

    async fn delete_credential(
        &self,
        tenant_id: &str,
        provider_id: &str,
        key_reference: &str,
    ) -> Result<bool> {
        PostgresAdminStore::delete_credential(self, tenant_id, provider_id, key_reference).await
    }
    async fn upsert_official_provider_config(
        &self,
        config: &OfficialProviderConfig,
    ) -> Result<OfficialProviderConfig> {
        PostgresAdminStore::upsert_official_provider_config(self, config).await
    }
    async fn list_official_provider_configs(&self) -> Result<Vec<OfficialProviderConfig>> {
        PostgresAdminStore::list_official_provider_configs(self).await
    }
    async fn find_official_provider_config(
        &self,
        provider_id: &str,
    ) -> Result<Option<OfficialProviderConfig>> {
        PostgresAdminStore::find_official_provider_config(self, provider_id).await
    }

    async fn insert_model(&self, model: &ModelCatalogEntry) -> Result<ModelCatalogEntry> {
        PostgresAdminStore::insert_model(self, model).await
    }

    async fn list_models(&self) -> Result<Vec<ModelCatalogEntry>> {
        PostgresAdminStore::list_models(self).await
    }

    async fn list_models_for_external_name(
        &self,
        external_name: &str,
    ) -> Result<Vec<ModelCatalogEntry>> {
        PostgresAdminStore::list_models_for_external_name(self, external_name).await
    }

    async fn find_any_model(&self) -> Result<Option<ModelCatalogEntry>> {
        PostgresAdminStore::find_any_model(self).await
    }

    async fn find_model(&self, external_name: &str) -> Result<Option<ModelCatalogEntry>> {
        PostgresAdminStore::find_model(self, external_name).await
    }

    async fn delete_model(&self, external_name: &str) -> Result<bool> {
        PostgresAdminStore::delete_model(self, external_name).await
    }

    async fn delete_model_variant(&self, external_name: &str, provider_id: &str) -> Result<bool> {
        PostgresAdminStore::delete_model_variant(self, external_name, provider_id).await
    }

    async fn insert_channel_model(
        &self,
        record: &ChannelModelRecord,
    ) -> Result<ChannelModelRecord> {
        PostgresAdminStore::insert_channel_model(self, record).await
    }

    async fn list_channel_models(&self) -> Result<Vec<ChannelModelRecord>> {
        PostgresAdminStore::list_channel_models(self).await
    }

    async fn delete_channel_model(&self, channel_id: &str, model_id: &str) -> Result<bool> {
        PostgresAdminStore::delete_channel_model(self, channel_id, model_id).await
    }
    async fn upsert_provider_model(
        &self,
        record: &ProviderModelRecord,
    ) -> Result<ProviderModelRecord> {
        PostgresAdminStore::upsert_provider_model(self, record).await
    }

    async fn list_provider_models(&self) -> Result<Vec<ProviderModelRecord>> {
        PostgresAdminStore::list_provider_models(self).await
    }

    async fn delete_provider_model(
        &self,
        proxy_provider_id: &str,
        channel_id: &str,
        model_id: &str,
    ) -> Result<bool> {
        PostgresAdminStore::delete_provider_model(self, proxy_provider_id, channel_id, model_id)
            .await
    }

    async fn insert_model_price(&self, record: &ModelPriceRecord) -> Result<ModelPriceRecord> {
        PostgresAdminStore::insert_model_price(self, record).await
    }

    async fn list_model_prices(&self) -> Result<Vec<ModelPriceRecord>> {
        PostgresAdminStore::list_model_prices(self).await
    }

    async fn delete_model_price(
        &self,
        channel_id: &str,
        model_id: &str,
        proxy_provider_id: &str,
    ) -> Result<bool> {
        PostgresAdminStore::delete_model_price(self, channel_id, model_id, proxy_provider_id).await
    }

    async fn insert_routing_policy(&self, policy: &RoutingPolicy) -> Result<RoutingPolicy> {
        PostgresAdminStore::insert_routing_policy(self, policy).await
    }

    async fn list_routing_policies(&self) -> Result<Vec<RoutingPolicy>> {
        PostgresAdminStore::list_routing_policies(self).await
    }

    async fn insert_routing_profile(
        &self,
        profile: &RoutingProfileRecord,
    ) -> Result<RoutingProfileRecord> {
        PostgresAdminStore::insert_routing_profile(self, profile).await
    }

    async fn list_routing_profiles(&self) -> Result<Vec<RoutingProfileRecord>> {
        PostgresAdminStore::list_routing_profiles(self).await
    }

    async fn find_routing_profile(&self, profile_id: &str) -> Result<Option<RoutingProfileRecord>> {
        PostgresAdminStore::find_routing_profile(self, profile_id).await
    }

    async fn insert_compiled_routing_snapshot(
        &self,
        snapshot: &CompiledRoutingSnapshotRecord,
    ) -> Result<CompiledRoutingSnapshotRecord> {
        PostgresAdminStore::insert_compiled_routing_snapshot(self, snapshot).await
    }

    async fn list_compiled_routing_snapshots(&self) -> Result<Vec<CompiledRoutingSnapshotRecord>> {
        PostgresAdminStore::list_compiled_routing_snapshots(self).await
    }

    async fn insert_project_routing_preferences(
        &self,
        preferences: &ProjectRoutingPreferences,
    ) -> Result<ProjectRoutingPreferences> {
        PostgresAdminStore::insert_project_routing_preferences(self, preferences).await
    }

    async fn find_project_routing_preferences(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectRoutingPreferences>> {
        PostgresAdminStore::find_project_routing_preferences(self, project_id).await
    }

    async fn insert_routing_decision_log(
        &self,
        log: &RoutingDecisionLog,
    ) -> Result<RoutingDecisionLog> {
        PostgresAdminStore::insert_routing_decision_log(self, log).await
    }

    async fn list_routing_decision_logs(&self) -> Result<Vec<RoutingDecisionLog>> {
        PostgresAdminStore::list_routing_decision_logs(self).await
    }

    async fn list_routing_decision_logs_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<RoutingDecisionLog>> {
        PostgresAdminStore::list_routing_decision_logs_for_project(self, project_id).await
    }

    async fn find_latest_routing_decision_log_for_project(
        &self,
        project_id: &str,
    ) -> Result<Option<RoutingDecisionLog>> {
        PostgresAdminStore::find_latest_routing_decision_log_for_project(self, project_id).await
    }

    async fn insert_provider_health_snapshot(
        &self,
        snapshot: &ProviderHealthSnapshot,
    ) -> Result<ProviderHealthSnapshot> {
        PostgresAdminStore::insert_provider_health_snapshot(self, snapshot).await
    }

    async fn list_provider_health_snapshots(&self) -> Result<Vec<ProviderHealthSnapshot>> {
        PostgresAdminStore::list_provider_health_snapshots(self).await
    }

    async fn insert_usage_record(&self, record: &UsageRecord) -> Result<UsageRecord> {
        PostgresAdminStore::insert_usage_record(self, record).await
    }

    async fn list_usage_records(&self) -> Result<Vec<UsageRecord>> {
        PostgresAdminStore::list_usage_records(self).await
    }

    async fn list_usage_records_for_project(&self, project_id: &str) -> Result<Vec<UsageRecord>> {
        PostgresAdminStore::list_usage_records_for_project(self, project_id).await
    }

    async fn find_latest_usage_record_for_project(
        &self,
        project_id: &str,
    ) -> Result<Option<UsageRecord>> {
        PostgresAdminStore::find_latest_usage_record_for_project(self, project_id).await
    }

    async fn insert_billing_event(&self, event: &BillingEventRecord) -> Result<BillingEventRecord> {
        PostgresAdminStore::insert_billing_event(self, event).await
    }

    async fn list_billing_events(&self) -> Result<Vec<BillingEventRecord>> {
        PostgresAdminStore::list_billing_events(self).await
    }

    async fn insert_ledger_entry(&self, entry: &LedgerEntry) -> Result<LedgerEntry> {
        PostgresAdminStore::insert_ledger_entry(self, entry).await
    }

    async fn list_ledger_entries(&self) -> Result<Vec<LedgerEntry>> {
        PostgresAdminStore::list_ledger_entries(self).await
    }

    async fn list_ledger_entries_for_project(&self, project_id: &str) -> Result<Vec<LedgerEntry>> {
        PostgresAdminStore::list_ledger_entries_for_project(self, project_id).await
    }

    async fn insert_quota_policy(&self, policy: &QuotaPolicy) -> Result<QuotaPolicy> {
        PostgresAdminStore::insert_quota_policy(self, policy).await
    }

    async fn list_quota_policies(&self) -> Result<Vec<QuotaPolicy>> {
        PostgresAdminStore::list_quota_policies(self).await
    }

    async fn list_quota_policies_for_project(&self, project_id: &str) -> Result<Vec<QuotaPolicy>> {
        PostgresAdminStore::list_quota_policies_for_project(self, project_id).await
    }

    async fn delete_quota_policy(&self, policy_id: &str) -> Result<bool> {
        PostgresAdminStore::delete_quota_policy(self, policy_id).await
    }

    async fn insert_rate_limit_policy(&self, policy: &RateLimitPolicy) -> Result<RateLimitPolicy> {
        PostgresAdminStore::insert_rate_limit_policy(self, policy).await
    }

    async fn list_rate_limit_policies(&self) -> Result<Vec<RateLimitPolicy>> {
        PostgresAdminStore::list_rate_limit_policies(self).await
    }

    async fn list_rate_limit_window_snapshots(&self) -> Result<Vec<RateLimitWindowSnapshot>> {
        PostgresAdminStore::list_rate_limit_window_snapshots(self).await
    }

    async fn list_rate_limit_policies_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<RateLimitPolicy>> {
        PostgresAdminStore::list_rate_limit_policies_for_project(self, project_id).await
    }

    async fn check_and_consume_rate_limit(
        &self,
        policy_id: &str,
        requested_requests: u64,
        limit_requests: u64,
        window_seconds: u64,
        now_ms: u64,
    ) -> Result<RateLimitCheckResult> {
        PostgresAdminStore::check_and_consume_rate_limit(
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
        PostgresAdminStore::insert_tenant(self, tenant).await
    }

    async fn list_tenants(&self) -> Result<Vec<Tenant>> {
        PostgresAdminStore::list_tenants(self).await
    }

    async fn find_tenant(&self, tenant_id: &str) -> Result<Option<Tenant>> {
        PostgresAdminStore::find_tenant(self, tenant_id).await
    }

    async fn delete_tenant(&self, tenant_id: &str) -> Result<bool> {
        PostgresAdminStore::delete_tenant(self, tenant_id).await
    }

    async fn insert_project(&self, project: &Project) -> Result<Project> {
        PostgresAdminStore::insert_project(self, project).await
    }

    async fn list_projects(&self) -> Result<Vec<Project>> {
        PostgresAdminStore::list_projects(self).await
    }

    async fn find_project(&self, project_id: &str) -> Result<Option<Project>> {
        PostgresAdminStore::find_project(self, project_id).await
    }

    async fn delete_project(&self, project_id: &str) -> Result<bool> {
        PostgresAdminStore::delete_project(self, project_id).await
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
        PostgresAdminStore::insert_catalog_publication_lifecycle_audit_record(self, record).await
    }
    async fn list_catalog_publication_lifecycle_audit_records(
        &self,
    ) -> Result<Vec<CatalogPublicationLifecycleAuditRecord>> {
        PostgresAdminStore::list_catalog_publication_lifecycle_audit_records(self).await
    }

    async fn insert_commerce_order(
        &self,
        order: &CommerceOrderRecord,
    ) -> Result<CommerceOrderRecord> {
        PostgresAdminStore::insert_commerce_order(self, order).await
    }

    async fn list_commerce_orders(&self) -> Result<Vec<CommerceOrderRecord>> {
        PostgresAdminStore::list_commerce_orders(self).await
    }

    async fn list_recent_commerce_orders(&self, limit: usize) -> Result<Vec<CommerceOrderRecord>> {
        PostgresAdminStore::list_recent_commerce_orders(self, limit).await
    }

    async fn list_commerce_orders_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<CommerceOrderRecord>> {
        PostgresAdminStore::list_commerce_orders_for_project(self, project_id).await
    }

    async fn list_commerce_orders_for_project_after(
        &self,
        project_id: &str,
        last_order_updated_at_ms: u64,
        last_order_created_at_ms: u64,
        last_order_id: &str,
    ) -> Result<Vec<CommerceOrderRecord>> {
        PostgresAdminStore::list_commerce_orders_for_project_after(
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
        PostgresAdminStore::upsert_commerce_payment_event(self, event).await
    }

    async fn list_commerce_payment_events(&self) -> Result<Vec<CommercePaymentEventRecord>> {
        PostgresAdminStore::list_commerce_payment_events(self).await
    }

    async fn find_commerce_payment_event_by_dedupe_key(
        &self,
        dedupe_key: &str,
    ) -> Result<Option<CommercePaymentEventRecord>> {
        PostgresAdminStore::find_commerce_payment_event_by_dedupe_key(self, dedupe_key).await
    }

    async fn upsert_payment_method(
        &self,
        payment_method: &PaymentMethodRecord,
    ) -> Result<PaymentMethodRecord> {
        PostgresAdminStore::upsert_payment_method(self, payment_method).await
    }

    async fn list_payment_methods(&self) -> Result<Vec<PaymentMethodRecord>> {
        PostgresAdminStore::list_payment_methods(self).await
    }

    async fn find_payment_method(
        &self,
        payment_method_id: &str,
    ) -> Result<Option<PaymentMethodRecord>> {
        PostgresAdminStore::find_payment_method(self, payment_method_id).await
    }

    async fn delete_payment_method(&self, payment_method_id: &str) -> Result<bool> {
        PostgresAdminStore::delete_payment_method(self, payment_method_id).await
    }

    async fn upsert_payment_method_credential_binding(
        &self,
        binding: &PaymentMethodCredentialBindingRecord,
    ) -> Result<PaymentMethodCredentialBindingRecord> {
        PostgresAdminStore::upsert_payment_method_credential_binding(self, binding).await
    }

    async fn list_payment_method_credential_bindings(
        &self,
        payment_method_id: &str,
    ) -> Result<Vec<PaymentMethodCredentialBindingRecord>> {
        PostgresAdminStore::list_payment_method_credential_bindings(self, payment_method_id).await
    }

    async fn delete_payment_method_credential_binding(
        &self,
        payment_method_id: &str,
        binding_id: &str,
    ) -> Result<bool> {
        PostgresAdminStore::delete_payment_method_credential_binding(
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
        PostgresAdminStore::upsert_commerce_payment_attempt(self, attempt).await
    }

    async fn list_commerce_payment_attempts(&self) -> Result<Vec<CommercePaymentAttemptRecord>> {
        PostgresAdminStore::list_commerce_payment_attempts(self).await
    }

    async fn find_commerce_payment_attempt(
        &self,
        payment_attempt_id: &str,
    ) -> Result<Option<CommercePaymentAttemptRecord>> {
        PostgresAdminStore::find_commerce_payment_attempt(self, payment_attempt_id).await
    }

    async fn find_commerce_payment_attempt_by_idempotency_key(
        &self,
        idempotency_key: &str,
    ) -> Result<Option<CommercePaymentAttemptRecord>> {
        PostgresAdminStore::find_commerce_payment_attempt_by_idempotency_key(self, idempotency_key)
            .await
    }

    async fn list_commerce_payment_attempts_for_order(
        &self,
        order_id: &str,
    ) -> Result<Vec<CommercePaymentAttemptRecord>> {
        PostgresAdminStore::list_commerce_payment_attempts_for_order(self, order_id).await
    }

    async fn upsert_commerce_webhook_inbox(
        &self,
        record: &CommerceWebhookInboxRecord,
    ) -> Result<CommerceWebhookInboxRecord> {
        PostgresAdminStore::upsert_commerce_webhook_inbox(self, record).await
    }

    async fn list_commerce_webhook_inbox_records(&self) -> Result<Vec<CommerceWebhookInboxRecord>> {
        PostgresAdminStore::list_commerce_webhook_inbox_records(self).await
    }

    async fn find_commerce_webhook_inbox_by_dedupe_key(
        &self,
        dedupe_key: &str,
    ) -> Result<Option<CommerceWebhookInboxRecord>> {
        PostgresAdminStore::find_commerce_webhook_inbox_by_dedupe_key(self, dedupe_key).await
    }

    async fn insert_commerce_webhook_delivery_attempt(
        &self,
        record: &CommerceWebhookDeliveryAttemptRecord,
    ) -> Result<CommerceWebhookDeliveryAttemptRecord> {
        PostgresAdminStore::insert_commerce_webhook_delivery_attempt(self, record).await
    }

    async fn list_commerce_webhook_delivery_attempts(
        &self,
        webhook_inbox_id: &str,
    ) -> Result<Vec<CommerceWebhookDeliveryAttemptRecord>> {
        PostgresAdminStore::list_commerce_webhook_delivery_attempts(self, webhook_inbox_id).await
    }

    async fn upsert_commerce_refund(
        &self,
        refund: &CommerceRefundRecord,
    ) -> Result<CommerceRefundRecord> {
        PostgresAdminStore::upsert_commerce_refund(self, refund).await
    }

    async fn list_commerce_refunds(&self) -> Result<Vec<CommerceRefundRecord>> {
        PostgresAdminStore::list_commerce_refunds(self).await
    }

    async fn find_commerce_refund(&self, refund_id: &str) -> Result<Option<CommerceRefundRecord>> {
        PostgresAdminStore::find_commerce_refund(self, refund_id).await
    }

    async fn find_commerce_refund_by_idempotency_key(
        &self,
        idempotency_key: &str,
    ) -> Result<Option<CommerceRefundRecord>> {
        PostgresAdminStore::find_commerce_refund_by_idempotency_key(self, idempotency_key).await
    }

    async fn list_commerce_refunds_for_order(
        &self,
        order_id: &str,
    ) -> Result<Vec<CommerceRefundRecord>> {
        PostgresAdminStore::list_commerce_refunds_for_order(self, order_id).await
    }

    async fn insert_commerce_reconciliation_run(
        &self,
        record: &CommerceReconciliationRunRecord,
    ) -> Result<CommerceReconciliationRunRecord> {
        PostgresAdminStore::insert_commerce_reconciliation_run(self, record).await
    }

    async fn list_commerce_reconciliation_runs(
        &self,
    ) -> Result<Vec<CommerceReconciliationRunRecord>> {
        PostgresAdminStore::list_commerce_reconciliation_runs(self).await
    }

    async fn insert_commerce_reconciliation_item(
        &self,
        record: &CommerceReconciliationItemRecord,
    ) -> Result<CommerceReconciliationItemRecord> {
        PostgresAdminStore::insert_commerce_reconciliation_item(self, record).await
    }

    async fn list_commerce_reconciliation_items(
        &self,
        reconciliation_run_id: &str,
    ) -> Result<Vec<CommerceReconciliationItemRecord>> {
        PostgresAdminStore::list_commerce_reconciliation_items(self, reconciliation_run_id).await
    }

    async fn insert_async_job(&self, record: &AsyncJobRecord) -> Result<AsyncJobRecord> {
        PostgresAdminStore::insert_async_job(self, record).await
    }

    async fn list_async_jobs(&self) -> Result<Vec<AsyncJobRecord>> {
        PostgresAdminStore::list_async_jobs(self).await
    }

    async fn find_async_job(&self, job_id: &str) -> Result<Option<AsyncJobRecord>> {
        PostgresAdminStore::find_async_job(self, job_id).await
    }

    async fn insert_async_job_attempt(
        &self,
        record: &AsyncJobAttemptRecord,
    ) -> Result<AsyncJobAttemptRecord> {
        PostgresAdminStore::insert_async_job_attempt(self, record).await
    }

    async fn list_async_job_attempts(&self, job_id: &str) -> Result<Vec<AsyncJobAttemptRecord>> {
        PostgresAdminStore::list_async_job_attempts(self, job_id).await
    }

    async fn insert_async_job_asset(
        &self,
        record: &AsyncJobAssetRecord,
    ) -> Result<AsyncJobAssetRecord> {
        PostgresAdminStore::insert_async_job_asset(self, record).await
    }

    async fn list_async_job_assets(&self, job_id: &str) -> Result<Vec<AsyncJobAssetRecord>> {
        PostgresAdminStore::list_async_job_assets(self, job_id).await
    }

    async fn insert_async_job_callback(
        &self,
        record: &AsyncJobCallbackRecord,
    ) -> Result<AsyncJobCallbackRecord> {
        PostgresAdminStore::insert_async_job_callback(self, record).await
    }

    async fn list_async_job_callbacks(&self, job_id: &str) -> Result<Vec<AsyncJobCallbackRecord>> {
        PostgresAdminStore::list_async_job_callbacks(self, job_id).await
    }

    async fn upsert_project_membership(
        &self,
        membership: &ProjectMembershipRecord,
    ) -> Result<ProjectMembershipRecord> {
        PostgresAdminStore::upsert_project_membership(self, membership).await
    }

    async fn find_project_membership(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectMembershipRecord>> {
        PostgresAdminStore::find_project_membership(self, project_id).await
    }

    async fn delete_project_membership(&self, project_id: &str) -> Result<bool> {
        PostgresAdminStore::delete_project_membership(self, project_id).await
    }

    async fn insert_portal_user(&self, user: &PortalUserRecord) -> Result<PortalUserRecord> {
        PostgresAdminStore::insert_portal_user(self, user).await
    }

    async fn list_portal_users(&self) -> Result<Vec<PortalUserRecord>> {
        PostgresAdminStore::list_portal_users(self).await
    }

    async fn find_portal_user_by_email(&self, email: &str) -> Result<Option<PortalUserRecord>> {
        PostgresAdminStore::find_portal_user_by_email(self, email).await
    }

    async fn find_portal_user_by_id(&self, user_id: &str) -> Result<Option<PortalUserRecord>> {
        PostgresAdminStore::find_portal_user_by_id(self, user_id).await
    }

    async fn delete_portal_user(&self, user_id: &str) -> Result<bool> {
        PostgresAdminStore::delete_portal_user(self, user_id).await
    }

    async fn insert_admin_user(&self, user: &AdminUserRecord) -> Result<AdminUserRecord> {
        PostgresAdminStore::insert_admin_user(self, user).await
    }

    async fn list_admin_users(&self) -> Result<Vec<AdminUserRecord>> {
        PostgresAdminStore::list_admin_users(self).await
    }

    async fn find_admin_user_by_email(&self, email: &str) -> Result<Option<AdminUserRecord>> {
        PostgresAdminStore::find_admin_user_by_email(self, email).await
    }

    async fn find_admin_user_by_id(&self, user_id: &str) -> Result<Option<AdminUserRecord>> {
        PostgresAdminStore::find_admin_user_by_id(self, user_id).await
    }

    async fn delete_admin_user(&self, user_id: &str) -> Result<bool> {
        PostgresAdminStore::delete_admin_user(self, user_id).await
    }
    async fn insert_admin_audit_event(
        &self,
        record: &AdminAuditEventRecord,
    ) -> Result<AdminAuditEventRecord> {
        PostgresAdminStore::insert_admin_audit_event(self, record).await
    }
    async fn list_admin_audit_events(&self) -> Result<Vec<AdminAuditEventRecord>> {
        PostgresAdminStore::list_admin_audit_events(self).await
    }

    async fn insert_gateway_api_key(
        &self,
        record: &GatewayApiKeyRecord,
    ) -> Result<GatewayApiKeyRecord> {
        PostgresAdminStore::insert_gateway_api_key(self, record).await
    }

    async fn list_gateway_api_keys(&self) -> Result<Vec<GatewayApiKeyRecord>> {
        PostgresAdminStore::list_gateway_api_keys(self).await
    }

    async fn find_gateway_api_key(&self, hashed_key: &str) -> Result<Option<GatewayApiKeyRecord>> {
        PostgresAdminStore::find_gateway_api_key(self, hashed_key).await
    }

    async fn delete_gateway_api_key(&self, hashed_key: &str) -> Result<bool> {
        PostgresAdminStore::delete_gateway_api_key(self, hashed_key).await
    }

    async fn insert_api_key_group(&self, record: &ApiKeyGroupRecord) -> Result<ApiKeyGroupRecord> {
        PostgresAdminStore::insert_api_key_group(self, record).await
    }

    async fn list_api_key_groups(&self) -> Result<Vec<ApiKeyGroupRecord>> {
        PostgresAdminStore::list_api_key_groups(self).await
    }

    async fn find_api_key_group(&self, group_id: &str) -> Result<Option<ApiKeyGroupRecord>> {
        PostgresAdminStore::find_api_key_group(self, group_id).await
    }

    async fn delete_api_key_group(&self, group_id: &str) -> Result<bool> {
        PostgresAdminStore::delete_api_key_group(self, group_id).await
    }

    async fn insert_extension_installation(
        &self,
        installation: &ExtensionInstallation,
    ) -> Result<ExtensionInstallation> {
        PostgresAdminStore::insert_extension_installation(self, installation).await
    }

    async fn list_extension_installations(&self) -> Result<Vec<ExtensionInstallation>> {
        PostgresAdminStore::list_extension_installations(self).await
    }

    async fn insert_extension_instance(
        &self,
        instance: &ExtensionInstance,
    ) -> Result<ExtensionInstance> {
        PostgresAdminStore::insert_extension_instance(self, instance).await
    }

    async fn list_extension_instances(&self) -> Result<Vec<ExtensionInstance>> {
        PostgresAdminStore::list_extension_instances(self).await
    }

    async fn upsert_service_runtime_node(
        &self,
        record: &ServiceRuntimeNodeRecord,
    ) -> Result<ServiceRuntimeNodeRecord> {
        PostgresAdminStore::upsert_service_runtime_node(self, record).await
    }

    async fn list_service_runtime_nodes(&self) -> Result<Vec<ServiceRuntimeNodeRecord>> {
        PostgresAdminStore::list_service_runtime_nodes(self).await
    }

    async fn insert_extension_runtime_rollout(
        &self,
        rollout: &ExtensionRuntimeRolloutRecord,
    ) -> Result<ExtensionRuntimeRolloutRecord> {
        PostgresAdminStore::insert_extension_runtime_rollout(self, rollout).await
    }

    async fn find_extension_runtime_rollout(
        &self,
        rollout_id: &str,
    ) -> Result<Option<ExtensionRuntimeRolloutRecord>> {
        PostgresAdminStore::find_extension_runtime_rollout(self, rollout_id).await
    }

    async fn list_extension_runtime_rollouts(&self) -> Result<Vec<ExtensionRuntimeRolloutRecord>> {
        PostgresAdminStore::list_extension_runtime_rollouts(self).await
    }

    async fn insert_extension_runtime_rollout_participant(
        &self,
        participant: &ExtensionRuntimeRolloutParticipantRecord,
    ) -> Result<ExtensionRuntimeRolloutParticipantRecord> {
        PostgresAdminStore::insert_extension_runtime_rollout_participant(self, participant).await
    }

    async fn list_extension_runtime_rollout_participants(
        &self,
        rollout_id: &str,
    ) -> Result<Vec<ExtensionRuntimeRolloutParticipantRecord>> {
        PostgresAdminStore::list_extension_runtime_rollout_participants(self, rollout_id).await
    }

    async fn list_pending_extension_runtime_rollout_participants_for_node(
        &self,
        node_id: &str,
    ) -> Result<Vec<ExtensionRuntimeRolloutParticipantRecord>> {
        PostgresAdminStore::list_pending_extension_runtime_rollout_participants_for_node(
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
        PostgresAdminStore::transition_extension_runtime_rollout_participant(
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
        PostgresAdminStore::insert_standalone_config_rollout(self, rollout).await
    }

    async fn find_standalone_config_rollout(
        &self,
        rollout_id: &str,
    ) -> Result<Option<StandaloneConfigRolloutRecord>> {
        PostgresAdminStore::find_standalone_config_rollout(self, rollout_id).await
    }

    async fn list_standalone_config_rollouts(&self) -> Result<Vec<StandaloneConfigRolloutRecord>> {
        PostgresAdminStore::list_standalone_config_rollouts(self).await
    }

    async fn insert_standalone_config_rollout_participant(
        &self,
        participant: &StandaloneConfigRolloutParticipantRecord,
    ) -> Result<StandaloneConfigRolloutParticipantRecord> {
        PostgresAdminStore::insert_standalone_config_rollout_participant(self, participant).await
    }

    async fn list_standalone_config_rollout_participants(
        &self,
        rollout_id: &str,
    ) -> Result<Vec<StandaloneConfigRolloutParticipantRecord>> {
        PostgresAdminStore::list_standalone_config_rollout_participants(self, rollout_id).await
    }

    async fn list_pending_standalone_config_rollout_participants_for_node(
        &self,
        node_id: &str,
    ) -> Result<Vec<StandaloneConfigRolloutParticipantRecord>> {
        PostgresAdminStore::list_pending_standalone_config_rollout_participants_for_node(
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
        PostgresAdminStore::transition_standalone_config_rollout_participant(
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
