use super::*;

#[async_trait]
pub trait AdminStore: Send + Sync {
    fn dialect(&self) -> StorageDialect;

    fn account_kernel_store(&self) -> Option<&dyn AccountKernelStore> {
        None
    }

    async fn insert_channel(&self, channel: &Channel) -> Result<Channel>;
    async fn list_channels(&self) -> Result<Vec<Channel>>;
    async fn delete_channel(&self, channel_id: &str) -> Result<bool>;

    async fn insert_provider(&self, provider: &ProxyProvider) -> Result<ProxyProvider>;
    async fn list_providers(&self) -> Result<Vec<ProxyProvider>>;
    async fn list_providers_for_model(&self, model: &str) -> Result<Vec<ProxyProvider>> {
        let mut model_provider_ids = self
            .list_provider_models_for_model(model)
            .await?
            .into_iter()
            .map(|entry| entry.proxy_provider_id)
            .collect::<std::collections::HashSet<_>>();
        if model_provider_ids.is_empty() {
            model_provider_ids = self
                .list_models_for_external_name(model)
                .await?
                .into_iter()
                .map(|entry| entry.provider_id)
                .collect::<std::collections::HashSet<_>>();
        }
        Ok(self
            .list_providers()
            .await?
            .into_iter()
            .filter(|provider| {
                model_provider_ids.is_empty() || model_provider_ids.contains(&provider.id)
            })
            .collect())
    }
    async fn find_provider(&self, provider_id: &str) -> Result<Option<ProxyProvider>>;
    async fn delete_provider(&self, provider_id: &str) -> Result<bool>;
    async fn upsert_provider_account(
        &self,
        record: &ProviderAccountRecord,
    ) -> Result<ProviderAccountRecord>;
    async fn list_provider_accounts(&self) -> Result<Vec<ProviderAccountRecord>>;
    async fn list_provider_accounts_for_provider(
        &self,
        provider_id: &str,
    ) -> Result<Vec<ProviderAccountRecord>> {
        Ok(self
            .list_provider_accounts()
            .await?
            .into_iter()
            .filter(|record| record.provider_id == provider_id)
            .collect())
    }
    async fn find_provider_account(
        &self,
        provider_account_id: &str,
    ) -> Result<Option<ProviderAccountRecord>> {
        Ok(self
            .list_provider_accounts()
            .await?
            .into_iter()
            .find(|record| record.provider_account_id == provider_account_id))
    }
    async fn delete_provider_account(&self, provider_account_id: &str) -> Result<bool>;

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
    async fn list_credentials_for_tenant(
        &self,
        tenant_id: &str,
    ) -> Result<Vec<UpstreamCredential>> {
        Ok(self
            .list_credentials()
            .await?
            .into_iter()
            .filter(|credential| credential.tenant_id == tenant_id)
            .collect())
    }
    async fn list_credentials_for_provider(
        &self,
        provider_id: &str,
    ) -> Result<Vec<UpstreamCredential>> {
        Ok(self
            .list_credentials()
            .await?
            .into_iter()
            .filter(|credential| credential.provider_id == provider_id)
            .collect())
    }
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
    async fn upsert_official_provider_config(
        &self,
        config: &OfficialProviderConfig,
    ) -> Result<OfficialProviderConfig>;
    async fn list_official_provider_configs(&self) -> Result<Vec<OfficialProviderConfig>>;
    async fn find_official_provider_config(
        &self,
        provider_id: &str,
    ) -> Result<Option<OfficialProviderConfig>>;

    async fn insert_model(&self, model: &ModelCatalogEntry) -> Result<ModelCatalogEntry>;
    async fn list_models(&self) -> Result<Vec<ModelCatalogEntry>>;
    async fn list_models_for_external_name(
        &self,
        external_name: &str,
    ) -> Result<Vec<ModelCatalogEntry>> {
        Ok(self
            .list_models()
            .await?
            .into_iter()
            .filter(|model| model.external_name == external_name)
            .collect())
    }
    async fn find_any_model(&self) -> Result<Option<ModelCatalogEntry>> {
        Ok(self.list_models().await?.into_iter().next())
    }
    async fn find_model(&self, external_name: &str) -> Result<Option<ModelCatalogEntry>>;
    async fn delete_model(&self, external_name: &str) -> Result<bool>;
    async fn delete_model_variant(&self, external_name: &str, provider_id: &str) -> Result<bool>;
    async fn insert_channel_model(&self, record: &ChannelModelRecord)
        -> Result<ChannelModelRecord>;
    async fn list_channel_models(&self) -> Result<Vec<ChannelModelRecord>>;
    async fn delete_channel_model(&self, channel_id: &str, model_id: &str) -> Result<bool>;
    async fn upsert_provider_model(
        &self,
        record: &ProviderModelRecord,
    ) -> Result<ProviderModelRecord>;
    async fn list_provider_models(&self) -> Result<Vec<ProviderModelRecord>>;
    async fn list_provider_models_for_model(
        &self,
        model_id: &str,
    ) -> Result<Vec<ProviderModelRecord>> {
        Ok(self
            .list_provider_models()
            .await?
            .into_iter()
            .filter(|record| record.model_id == model_id)
            .collect())
    }
    async fn list_provider_models_for_provider(
        &self,
        proxy_provider_id: &str,
    ) -> Result<Vec<ProviderModelRecord>> {
        Ok(self
            .list_provider_models()
            .await?
            .into_iter()
            .filter(|record| record.proxy_provider_id == proxy_provider_id)
            .collect())
    }
    async fn list_provider_models_for_channel_model(
        &self,
        channel_id: &str,
        model_id: &str,
    ) -> Result<Vec<ProviderModelRecord>> {
        Ok(self
            .list_provider_models()
            .await?
            .into_iter()
            .filter(|record| record.channel_id == channel_id && record.model_id == model_id)
            .collect())
    }
    async fn delete_provider_model(
        &self,
        proxy_provider_id: &str,
        channel_id: &str,
        model_id: &str,
    ) -> Result<bool>;
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
    async fn insert_routing_profile(
        &self,
        profile: &RoutingProfileRecord,
    ) -> Result<RoutingProfileRecord>;
    async fn list_routing_profiles(&self) -> Result<Vec<RoutingProfileRecord>>;
    async fn find_routing_profile(&self, profile_id: &str) -> Result<Option<RoutingProfileRecord>>;
    async fn insert_compiled_routing_snapshot(
        &self,
        snapshot: &CompiledRoutingSnapshotRecord,
    ) -> Result<CompiledRoutingSnapshotRecord>;
    async fn list_compiled_routing_snapshots(&self) -> Result<Vec<CompiledRoutingSnapshotRecord>>;
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
    async fn list_routing_decision_logs_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<RoutingDecisionLog>> {
        Ok(self
            .list_routing_decision_logs()
            .await?
            .into_iter()
            .filter(|log| log.project_id.as_deref() == Some(project_id))
            .collect())
    }
    async fn find_latest_routing_decision_log_for_project(
        &self,
        project_id: &str,
    ) -> Result<Option<RoutingDecisionLog>> {
        Ok(self
            .list_routing_decision_logs_for_project(project_id)
            .await?
            .into_iter()
            .next())
    }
    async fn insert_provider_health_snapshot(
        &self,
        snapshot: &ProviderHealthSnapshot,
    ) -> Result<ProviderHealthSnapshot>;
    async fn list_provider_health_snapshots(&self) -> Result<Vec<ProviderHealthSnapshot>>;

    async fn insert_usage_record(&self, record: &UsageRecord) -> Result<UsageRecord>;
    async fn list_usage_records(&self) -> Result<Vec<UsageRecord>>;
    async fn list_usage_records_for_project(&self, project_id: &str) -> Result<Vec<UsageRecord>> {
        Ok(self
            .list_usage_records()
            .await?
            .into_iter()
            .filter(|record| record.project_id == project_id)
            .collect())
    }
    async fn find_latest_usage_record_for_project(
        &self,
        project_id: &str,
    ) -> Result<Option<UsageRecord>> {
        Ok(self
            .list_usage_records_for_project(project_id)
            .await?
            .into_iter()
            .next())
    }

    async fn insert_billing_event(&self, event: &BillingEventRecord) -> Result<BillingEventRecord>;
    async fn list_billing_events(&self) -> Result<Vec<BillingEventRecord>>;
    async fn insert_ledger_entry(&self, entry: &LedgerEntry) -> Result<LedgerEntry>;
    async fn list_ledger_entries(&self) -> Result<Vec<LedgerEntry>>;
    async fn list_ledger_entries_for_project(&self, project_id: &str) -> Result<Vec<LedgerEntry>> {
        Ok(self
            .list_ledger_entries()
            .await?
            .into_iter()
            .filter(|entry| entry.project_id == project_id)
            .collect())
    }
    async fn insert_quota_policy(&self, policy: &QuotaPolicy) -> Result<QuotaPolicy>;
    async fn list_quota_policies(&self) -> Result<Vec<QuotaPolicy>>;
    async fn list_quota_policies_for_project(&self, project_id: &str) -> Result<Vec<QuotaPolicy>> {
        Ok(self
            .list_quota_policies()
            .await?
            .into_iter()
            .filter(|policy| policy.project_id == project_id)
            .collect())
    }
    async fn delete_quota_policy(&self, policy_id: &str) -> Result<bool>;

    async fn insert_rate_limit_policy(&self, policy: &RateLimitPolicy) -> Result<RateLimitPolicy>;
    async fn list_rate_limit_policies(&self) -> Result<Vec<RateLimitPolicy>>;
    async fn list_rate_limit_policies_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<RateLimitPolicy>> {
        Ok(self
            .list_rate_limit_policies()
            .await?
            .into_iter()
            .filter(|policy| policy.project_id == project_id)
            .collect())
    }
    async fn list_rate_limit_window_snapshots(&self) -> Result<Vec<RateLimitWindowSnapshot>>;
    async fn list_rate_limit_window_snapshots_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<RateLimitWindowSnapshot>> {
        Ok(self
            .list_rate_limit_window_snapshots()
            .await?
            .into_iter()
            .filter(|snapshot| snapshot.project_id == project_id)
            .collect())
    }
    async fn check_and_consume_rate_limit(
        &self,
        policy_id: &str,
        requested_requests: u64,
        limit_requests: u64,
        window_seconds: u64,
        now_ms: u64,
    ) -> Result<RateLimitCheckResult>;

    async fn insert_tenant(&self, tenant: &Tenant) -> Result<Tenant>;
    async fn list_tenants(&self) -> Result<Vec<Tenant>>;
    async fn find_tenant(&self, tenant_id: &str) -> Result<Option<Tenant>>;
    async fn delete_tenant(&self, tenant_id: &str) -> Result<bool>;

    async fn insert_project(&self, project: &Project) -> Result<Project>;
    async fn list_projects(&self) -> Result<Vec<Project>>;
    async fn find_project(&self, project_id: &str) -> Result<Option<Project>>;
    async fn delete_project(&self, project_id: &str) -> Result<bool>;

    async fn insert_coupon_template_record(
        &self,
        _record: &CouponTemplateRecord,
    ) -> Result<CouponTemplateRecord> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "insert_coupon_template_record",
        ))
    }
    async fn list_coupon_template_records(&self) -> Result<Vec<CouponTemplateRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "list_coupon_template_records",
        ))
    }
    async fn find_coupon_template_record(
        &self,
        _coupon_template_id: &str,
    ) -> Result<Option<CouponTemplateRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "find_coupon_template_record",
        ))
    }
    async fn find_coupon_template_record_by_template_key(
        &self,
        template_key: &str,
    ) -> Result<Option<CouponTemplateRecord>> {
        Ok(AdminStore::list_coupon_template_records(self)
            .await?
            .into_iter()
            .find(|record| record.template_key == template_key))
    }
    async fn list_coupon_template_records_for_root(
        &self,
        root_coupon_template_id: &str,
    ) -> Result<Vec<CouponTemplateRecord>> {
        Ok(AdminStore::list_coupon_template_records(self)
            .await?
            .into_iter()
            .filter(|record| {
                record
                    .root_coupon_template_id
                    .as_deref()
                    .unwrap_or(record.coupon_template_id.as_str())
                    == root_coupon_template_id
            })
            .collect())
    }
    async fn insert_coupon_template_lifecycle_audit_record(
        &self,
        _record: &CouponTemplateLifecycleAuditRecord,
    ) -> Result<CouponTemplateLifecycleAuditRecord> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "insert_coupon_template_lifecycle_audit_record",
        ))
    }
    async fn list_coupon_template_lifecycle_audit_records(
        &self,
    ) -> Result<Vec<CouponTemplateLifecycleAuditRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "list_coupon_template_lifecycle_audit_records",
        ))
    }
    async fn list_coupon_template_lifecycle_audit_records_for_template(
        &self,
        coupon_template_id: &str,
    ) -> Result<Vec<CouponTemplateLifecycleAuditRecord>> {
        Ok(
            AdminStore::list_coupon_template_lifecycle_audit_records(self)
                .await?
                .into_iter()
                .filter(|record| record.coupon_template_id == coupon_template_id)
                .collect(),
        )
    }
    async fn insert_marketing_campaign_record(
        &self,
        _record: &MarketingCampaignRecord,
    ) -> Result<MarketingCampaignRecord> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "insert_marketing_campaign_record",
        ))
    }
    async fn list_marketing_campaign_records(&self) -> Result<Vec<MarketingCampaignRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "list_marketing_campaign_records",
        ))
    }
    async fn find_marketing_campaign_record(
        &self,
        marketing_campaign_id: &str,
    ) -> Result<Option<MarketingCampaignRecord>> {
        Ok(AdminStore::list_marketing_campaign_records(self)
            .await?
            .into_iter()
            .find(|record| record.marketing_campaign_id == marketing_campaign_id))
    }
    async fn list_marketing_campaign_records_for_template(
        &self,
        coupon_template_id: &str,
    ) -> Result<Vec<MarketingCampaignRecord>> {
        Ok(AdminStore::list_marketing_campaign_records(self)
            .await?
            .into_iter()
            .filter(|record| record.coupon_template_id == coupon_template_id)
            .collect())
    }
    async fn list_marketing_campaign_records_for_root(
        &self,
        root_marketing_campaign_id: &str,
    ) -> Result<Vec<MarketingCampaignRecord>> {
        Ok(AdminStore::list_marketing_campaign_records(self)
            .await?
            .into_iter()
            .filter(|record| {
                record
                    .root_marketing_campaign_id
                    .as_deref()
                    .unwrap_or(record.marketing_campaign_id.as_str())
                    == root_marketing_campaign_id
            })
            .collect())
    }
    async fn insert_marketing_campaign_lifecycle_audit_record(
        &self,
        _record: &MarketingCampaignLifecycleAuditRecord,
    ) -> Result<MarketingCampaignLifecycleAuditRecord> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "insert_marketing_campaign_lifecycle_audit_record",
        ))
    }
    async fn list_marketing_campaign_lifecycle_audit_records(
        &self,
    ) -> Result<Vec<MarketingCampaignLifecycleAuditRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "list_marketing_campaign_lifecycle_audit_records",
        ))
    }
    async fn list_marketing_campaign_lifecycle_audit_records_for_campaign(
        &self,
        marketing_campaign_id: &str,
    ) -> Result<Vec<MarketingCampaignLifecycleAuditRecord>> {
        Ok(
            AdminStore::list_marketing_campaign_lifecycle_audit_records(self)
                .await?
                .into_iter()
                .filter(|record| record.marketing_campaign_id == marketing_campaign_id)
                .collect(),
        )
    }
    async fn insert_campaign_budget_record(
        &self,
        _record: &CampaignBudgetRecord,
    ) -> Result<CampaignBudgetRecord> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "insert_campaign_budget_record",
        ))
    }
    async fn list_campaign_budget_records(&self) -> Result<Vec<CampaignBudgetRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "list_campaign_budget_records",
        ))
    }
    async fn find_campaign_budget_record(
        &self,
        campaign_budget_id: &str,
    ) -> Result<Option<CampaignBudgetRecord>> {
        Ok(AdminStore::list_campaign_budget_records(self)
            .await?
            .into_iter()
            .find(|record| record.campaign_budget_id == campaign_budget_id))
    }
    async fn list_campaign_budget_records_for_campaign(
        &self,
        marketing_campaign_id: &str,
    ) -> Result<Vec<CampaignBudgetRecord>> {
        Ok(AdminStore::list_campaign_budget_records(self)
            .await?
            .into_iter()
            .filter(|record| record.marketing_campaign_id == marketing_campaign_id)
            .collect())
    }
    async fn insert_campaign_budget_lifecycle_audit_record(
        &self,
        _record: &CampaignBudgetLifecycleAuditRecord,
    ) -> Result<CampaignBudgetLifecycleAuditRecord> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "insert_campaign_budget_lifecycle_audit_record",
        ))
    }
    async fn list_campaign_budget_lifecycle_audit_records(
        &self,
    ) -> Result<Vec<CampaignBudgetLifecycleAuditRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "list_campaign_budget_lifecycle_audit_records",
        ))
    }
    async fn list_campaign_budget_lifecycle_audit_records_for_budget(
        &self,
        campaign_budget_id: &str,
    ) -> Result<Vec<CampaignBudgetLifecycleAuditRecord>> {
        Ok(
            AdminStore::list_campaign_budget_lifecycle_audit_records(self)
                .await?
                .into_iter()
                .filter(|record| record.campaign_budget_id == campaign_budget_id)
                .collect(),
        )
    }
    async fn insert_coupon_code_record(
        &self,
        _record: &CouponCodeRecord,
    ) -> Result<CouponCodeRecord> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "insert_coupon_code_record",
        ))
    }
    async fn list_coupon_code_records(&self) -> Result<Vec<CouponCodeRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "list_coupon_code_records",
        ))
    }
    async fn find_coupon_code_record(
        &self,
        _coupon_code_id: &str,
    ) -> Result<Option<CouponCodeRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "find_coupon_code_record",
        ))
    }
    async fn find_coupon_code_record_by_value(
        &self,
        code_value: &str,
    ) -> Result<Option<CouponCodeRecord>> {
        let normalized_code_value = sdkwork_api_domain_marketing::normalize_coupon_code(code_value);
        Ok(AdminStore::list_coupon_code_records(self)
            .await?
            .into_iter()
            .find(|record| {
                sdkwork_api_domain_marketing::normalize_coupon_code(&record.code_value)
                    == normalized_code_value
            }))
    }
    async fn list_redeemable_coupon_code_records_at(
        &self,
        now_ms: u64,
    ) -> Result<Vec<CouponCodeRecord>> {
        Ok(AdminStore::list_coupon_code_records(self)
            .await?
            .into_iter()
            .filter(|record| record.is_redeemable_at(now_ms))
            .collect())
    }
    async fn list_coupon_code_records_for_ids(
        &self,
        coupon_code_ids: &[String],
    ) -> Result<Vec<CouponCodeRecord>> {
        if coupon_code_ids.is_empty() {
            return Ok(Vec::new());
        }
        let code_ids = coupon_code_ids
            .iter()
            .map(String::as_str)
            .collect::<std::collections::HashSet<_>>();
        Ok(AdminStore::list_coupon_code_records(self)
            .await?
            .into_iter()
            .filter(|record| code_ids.contains(record.coupon_code_id.as_str()))
            .collect())
    }
    async fn list_coupon_code_records_for_claimed_subject(
        &self,
        subject_scope: MarketingSubjectScope,
        subject_id: &str,
    ) -> Result<Vec<CouponCodeRecord>> {
        Ok(AdminStore::list_coupon_code_records(self)
            .await?
            .into_iter()
            .filter(|record| {
                record.claimed_subject_scope == Some(subject_scope)
                    && record.claimed_subject_id.as_deref() == Some(subject_id)
            })
            .collect())
    }
    async fn insert_coupon_code_lifecycle_audit_record(
        &self,
        _record: &CouponCodeLifecycleAuditRecord,
    ) -> Result<CouponCodeLifecycleAuditRecord> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "insert_coupon_code_lifecycle_audit_record",
        ))
    }
    async fn list_coupon_code_lifecycle_audit_records(
        &self,
    ) -> Result<Vec<CouponCodeLifecycleAuditRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "list_coupon_code_lifecycle_audit_records",
        ))
    }
    async fn list_coupon_code_lifecycle_audit_records_for_code(
        &self,
        coupon_code_id: &str,
    ) -> Result<Vec<CouponCodeLifecycleAuditRecord>> {
        Ok(AdminStore::list_coupon_code_lifecycle_audit_records(self)
            .await?
            .into_iter()
            .filter(|record| record.coupon_code_id == coupon_code_id)
            .collect())
    }
    async fn insert_coupon_reservation_record(
        &self,
        _record: &CouponReservationRecord,
    ) -> Result<CouponReservationRecord> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "insert_coupon_reservation_record",
        ))
    }
    async fn list_coupon_reservation_records(&self) -> Result<Vec<CouponReservationRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "list_coupon_reservation_records",
        ))
    }
    async fn find_coupon_reservation_record(
        &self,
        _coupon_reservation_id: &str,
    ) -> Result<Option<CouponReservationRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "find_coupon_reservation_record",
        ))
    }
    async fn list_active_coupon_reservation_records_at(
        &self,
        now_ms: u64,
    ) -> Result<Vec<CouponReservationRecord>> {
        Ok(AdminStore::list_coupon_reservation_records(self)
            .await?
            .into_iter()
            .filter(|record| record.is_active_at(now_ms))
            .collect())
    }
    async fn list_coupon_reservation_records_for_code(
        &self,
        coupon_code_id: &str,
    ) -> Result<Vec<CouponReservationRecord>> {
        Ok(AdminStore::list_coupon_reservation_records(self)
            .await?
            .into_iter()
            .filter(|record| record.coupon_code_id == coupon_code_id)
            .collect())
    }
    async fn list_coupon_reservation_records_for_subject(
        &self,
        subject_scope: MarketingSubjectScope,
        subject_id: &str,
    ) -> Result<Vec<CouponReservationRecord>> {
        Ok(AdminStore::list_coupon_reservation_records(self)
            .await?
            .into_iter()
            .filter(|record| {
                record.subject_scope == subject_scope && record.subject_id == subject_id
            })
            .collect())
    }
    async fn insert_coupon_redemption_record(
        &self,
        _record: &CouponRedemptionRecord,
    ) -> Result<CouponRedemptionRecord> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "insert_coupon_redemption_record",
        ))
    }
    async fn list_coupon_redemption_records(&self) -> Result<Vec<CouponRedemptionRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "list_coupon_redemption_records",
        ))
    }
    async fn find_coupon_redemption_record(
        &self,
        _coupon_redemption_id: &str,
    ) -> Result<Option<CouponRedemptionRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "find_coupon_redemption_record",
        ))
    }
    async fn list_coupon_redemption_records_for_reservation_ids(
        &self,
        coupon_reservation_ids: &[String],
    ) -> Result<Vec<CouponRedemptionRecord>> {
        if coupon_reservation_ids.is_empty() {
            return Ok(Vec::new());
        }
        let reservation_ids = coupon_reservation_ids
            .iter()
            .map(String::as_str)
            .collect::<std::collections::HashSet<_>>();
        Ok(AdminStore::list_coupon_redemption_records(self)
            .await?
            .into_iter()
            .filter(|record| reservation_ids.contains(record.coupon_reservation_id.as_str()))
            .collect())
    }
    async fn insert_coupon_rollback_record(
        &self,
        _record: &CouponRollbackRecord,
    ) -> Result<CouponRollbackRecord> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "insert_coupon_rollback_record",
        ))
    }
    async fn list_coupon_rollback_records(&self) -> Result<Vec<CouponRollbackRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "list_coupon_rollback_records",
        ))
    }
    async fn find_coupon_rollback_record(
        &self,
        coupon_rollback_id: &str,
    ) -> Result<Option<CouponRollbackRecord>> {
        Ok(AdminStore::list_coupon_rollback_records(self)
            .await?
            .into_iter()
            .find(|record| record.coupon_rollback_id == coupon_rollback_id))
    }
    async fn list_coupon_rollback_records_for_redemption(
        &self,
        coupon_redemption_id: &str,
    ) -> Result<Vec<CouponRollbackRecord>> {
        Ok(AdminStore::list_coupon_rollback_records(self)
            .await?
            .into_iter()
            .filter(|record| record.coupon_redemption_id == coupon_redemption_id)
            .collect())
    }
    async fn list_coupon_rollback_records_for_redemption_ids(
        &self,
        coupon_redemption_ids: &[String],
    ) -> Result<Vec<CouponRollbackRecord>> {
        if coupon_redemption_ids.is_empty() {
            return Ok(Vec::new());
        }
        let redemption_ids = coupon_redemption_ids
            .iter()
            .map(String::as_str)
            .collect::<std::collections::HashSet<_>>();
        Ok(AdminStore::list_coupon_rollback_records(self)
            .await?
            .into_iter()
            .filter(|record| redemption_ids.contains(record.coupon_redemption_id.as_str()))
            .collect())
    }
    async fn insert_marketing_outbox_event_record(
        &self,
        _record: &MarketingOutboxEventRecord,
    ) -> Result<MarketingOutboxEventRecord> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "insert_marketing_outbox_event_record",
        ))
    }
    async fn list_marketing_outbox_event_records(&self) -> Result<Vec<MarketingOutboxEventRecord>> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "list_marketing_outbox_event_records",
        ))
    }
    async fn reserve_coupon_redemption_atomic(
        &self,
        _command: &AtomicCouponReservationCommand,
    ) -> Result<AtomicCouponReservationResult> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "reserve_coupon_redemption_atomic",
        ))
    }
    async fn confirm_coupon_redemption_atomic(
        &self,
        _command: &AtomicCouponConfirmationCommand,
    ) -> Result<AtomicCouponConfirmationResult> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "confirm_coupon_redemption_atomic",
        ))
    }
    async fn release_coupon_reservation_atomic(
        &self,
        _command: &AtomicCouponReleaseCommand,
    ) -> Result<AtomicCouponReleaseResult> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "release_coupon_reservation_atomic",
        ))
    }
    async fn rollback_coupon_redemption_atomic(
        &self,
        _command: &AtomicCouponRollbackCommand,
    ) -> Result<AtomicCouponRollbackResult> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "rollback_coupon_redemption_atomic",
        ))
    }
    async fn compensate_coupon_rollback_atomic(
        &self,
        _command: &AtomicCouponRollbackCompensationCommand,
    ) -> Result<AtomicCouponRollbackCompensationResult> {
        Err(unsupported_marketing_kernel_method(
            self.dialect(),
            "compensate_coupon_rollback_atomic",
        ))
    }

    async fn insert_catalog_publication_lifecycle_audit_record(
        &self,
        _record: &CatalogPublicationLifecycleAuditRecord,
    ) -> Result<CatalogPublicationLifecycleAuditRecord> {
        Err(unsupported_commerce_method(
            self.dialect(),
            "insert_catalog_publication_lifecycle_audit_record",
        ))
    }
    async fn list_catalog_publication_lifecycle_audit_records(
        &self,
    ) -> Result<Vec<CatalogPublicationLifecycleAuditRecord>> {
        Err(unsupported_commerce_method(
            self.dialect(),
            "list_catalog_publication_lifecycle_audit_records",
        ))
    }

    async fn insert_commerce_order(
        &self,
        order: &CommerceOrderRecord,
    ) -> Result<CommerceOrderRecord>;
    async fn list_commerce_orders(&self) -> Result<Vec<CommerceOrderRecord>>;
    async fn find_commerce_order(&self, order_id: &str) -> Result<Option<CommerceOrderRecord>> {
        Ok(self
            .list_commerce_orders()
            .await?
            .into_iter()
            .find(|order| order.order_id == order_id))
    }
    async fn list_recent_commerce_orders(&self, limit: usize) -> Result<Vec<CommerceOrderRecord>> {
        let mut orders = self.list_commerce_orders().await?;
        orders.sort_by(|left, right| {
            right
                .updated_at_ms
                .cmp(&left.updated_at_ms)
                .then_with(|| right.created_at_ms.cmp(&left.created_at_ms))
                .then_with(|| right.order_id.cmp(&left.order_id))
        });
        orders.truncate(limit);
        Ok(orders)
    }
    async fn upsert_commerce_payment_event(
        &self,
        event: &CommercePaymentEventRecord,
    ) -> Result<CommercePaymentEventRecord>;
    async fn list_commerce_payment_events(&self) -> Result<Vec<CommercePaymentEventRecord>>;
    async fn find_commerce_payment_event_by_dedupe_key(
        &self,
        dedupe_key: &str,
    ) -> Result<Option<CommercePaymentEventRecord>>;
    async fn list_commerce_orders_for_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<CommerceOrderRecord>> {
        Ok(self
            .list_commerce_orders()
            .await?
            .into_iter()
            .filter(|order| order.project_id == project_id)
            .collect())
    }
    async fn list_commerce_orders_for_project_after(
        &self,
        project_id: &str,
        last_order_updated_at_ms: u64,
        last_order_created_at_ms: u64,
        last_order_id: &str,
    ) -> Result<Vec<CommerceOrderRecord>> {
        Ok(self
            .list_commerce_orders_for_project(project_id)
            .await?
            .into_iter()
            .filter(|order| {
                order.updated_at_ms > last_order_updated_at_ms
                    || (order.updated_at_ms == last_order_updated_at_ms
                        && (order.created_at_ms > last_order_created_at_ms
                            || (order.created_at_ms == last_order_created_at_ms
                                && order.order_id.as_str() > last_order_id)))
            })
            .collect())
    }
    async fn list_commerce_payment_events_for_order(
        &self,
        order_id: &str,
    ) -> Result<Vec<CommercePaymentEventRecord>> {
        Ok(self
            .list_commerce_payment_events()
            .await?
            .into_iter()
            .filter(|event| event.order_id == order_id)
            .collect())
    }
    async fn upsert_payment_method(
        &self,
        _payment_method: &PaymentMethodRecord,
    ) -> Result<PaymentMethodRecord> {
        Err(unsupported_commerce_method(
            self.dialect(),
            "upsert_payment_method",
        ))
    }
    async fn list_payment_methods(&self) -> Result<Vec<PaymentMethodRecord>> {
        Err(unsupported_commerce_method(
            self.dialect(),
            "list_payment_methods",
        ))
    }
    async fn find_payment_method(
        &self,
        payment_method_id: &str,
    ) -> Result<Option<PaymentMethodRecord>> {
        Ok(self
            .list_payment_methods()
            .await?
            .into_iter()
            .find(|method| method.payment_method_id == payment_method_id))
    }
    async fn delete_payment_method(&self, _payment_method_id: &str) -> Result<bool> {
        Err(unsupported_commerce_method(
            self.dialect(),
            "delete_payment_method",
        ))
    }
    async fn upsert_payment_method_credential_binding(
        &self,
        _binding: &PaymentMethodCredentialBindingRecord,
    ) -> Result<PaymentMethodCredentialBindingRecord> {
        Err(unsupported_commerce_method(
            self.dialect(),
            "upsert_payment_method_credential_binding",
        ))
    }
    async fn list_payment_method_credential_bindings(
        &self,
        _payment_method_id: &str,
    ) -> Result<Vec<PaymentMethodCredentialBindingRecord>> {
        Err(unsupported_commerce_method(
            self.dialect(),
            "list_payment_method_credential_bindings",
        ))
    }
    async fn delete_payment_method_credential_binding(
        &self,
        _payment_method_id: &str,
        _binding_id: &str,
    ) -> Result<bool> {
        Err(unsupported_commerce_method(
            self.dialect(),
            "delete_payment_method_credential_binding",
        ))
    }
    async fn upsert_commerce_payment_attempt(
        &self,
        _attempt: &CommercePaymentAttemptRecord,
    ) -> Result<CommercePaymentAttemptRecord> {
        Err(unsupported_commerce_method(
            self.dialect(),
            "upsert_commerce_payment_attempt",
        ))
    }
    async fn list_commerce_payment_attempts(&self) -> Result<Vec<CommercePaymentAttemptRecord>> {
        Err(unsupported_commerce_method(
            self.dialect(),
            "list_commerce_payment_attempts",
        ))
    }
    async fn find_commerce_payment_attempt(
        &self,
        payment_attempt_id: &str,
    ) -> Result<Option<CommercePaymentAttemptRecord>> {
        Ok(self
            .list_commerce_payment_attempts()
            .await?
            .into_iter()
            .find(|attempt| attempt.payment_attempt_id == payment_attempt_id))
    }
    async fn find_commerce_payment_attempt_by_idempotency_key(
        &self,
        idempotency_key: &str,
    ) -> Result<Option<CommercePaymentAttemptRecord>> {
        Ok(self
            .list_commerce_payment_attempts()
            .await?
            .into_iter()
            .find(|attempt| attempt.idempotency_key == idempotency_key))
    }
    async fn list_commerce_payment_attempts_for_order(
        &self,
        order_id: &str,
    ) -> Result<Vec<CommercePaymentAttemptRecord>> {
        Ok(self
            .list_commerce_payment_attempts()
            .await?
            .into_iter()
            .filter(|attempt| attempt.order_id == order_id)
            .collect())
    }
    async fn upsert_commerce_webhook_inbox(
        &self,
        _record: &CommerceWebhookInboxRecord,
    ) -> Result<CommerceWebhookInboxRecord> {
        Err(unsupported_commerce_method(
            self.dialect(),
            "upsert_commerce_webhook_inbox",
        ))
    }
    async fn list_commerce_webhook_inbox_records(&self) -> Result<Vec<CommerceWebhookInboxRecord>> {
        Err(unsupported_commerce_method(
            self.dialect(),
            "list_commerce_webhook_inbox_records",
        ))
    }
    async fn find_commerce_webhook_inbox_by_dedupe_key(
        &self,
        dedupe_key: &str,
    ) -> Result<Option<CommerceWebhookInboxRecord>> {
        Ok(self
            .list_commerce_webhook_inbox_records()
            .await?
            .into_iter()
            .find(|record| record.dedupe_key == dedupe_key))
    }
    async fn insert_commerce_webhook_delivery_attempt(
        &self,
        _record: &CommerceWebhookDeliveryAttemptRecord,
    ) -> Result<CommerceWebhookDeliveryAttemptRecord> {
        Err(unsupported_commerce_method(
            self.dialect(),
            "insert_commerce_webhook_delivery_attempt",
        ))
    }
    async fn list_commerce_webhook_delivery_attempts(
        &self,
        _webhook_inbox_id: &str,
    ) -> Result<Vec<CommerceWebhookDeliveryAttemptRecord>> {
        Err(unsupported_commerce_method(
            self.dialect(),
            "list_commerce_webhook_delivery_attempts",
        ))
    }
    async fn upsert_commerce_refund(
        &self,
        _refund: &CommerceRefundRecord,
    ) -> Result<CommerceRefundRecord> {
        Err(unsupported_commerce_method(
            self.dialect(),
            "upsert_commerce_refund",
        ))
    }
    async fn list_commerce_refunds(&self) -> Result<Vec<CommerceRefundRecord>> {
        Err(unsupported_commerce_method(
            self.dialect(),
            "list_commerce_refunds",
        ))
    }
    async fn find_commerce_refund(&self, refund_id: &str) -> Result<Option<CommerceRefundRecord>> {
        Ok(self
            .list_commerce_refunds()
            .await?
            .into_iter()
            .find(|refund| refund.refund_id == refund_id))
    }
    async fn find_commerce_refund_by_idempotency_key(
        &self,
        idempotency_key: &str,
    ) -> Result<Option<CommerceRefundRecord>> {
        Ok(self
            .list_commerce_refunds()
            .await?
            .into_iter()
            .find(|refund| refund.idempotency_key == idempotency_key))
    }
    async fn list_commerce_refunds_for_order(
        &self,
        order_id: &str,
    ) -> Result<Vec<CommerceRefundRecord>> {
        Ok(self
            .list_commerce_refunds()
            .await?
            .into_iter()
            .filter(|refund| refund.order_id == order_id)
            .collect())
    }
    async fn insert_commerce_reconciliation_run(
        &self,
        _record: &CommerceReconciliationRunRecord,
    ) -> Result<CommerceReconciliationRunRecord> {
        Err(unsupported_commerce_method(
            self.dialect(),
            "insert_commerce_reconciliation_run",
        ))
    }
    async fn list_commerce_reconciliation_runs(
        &self,
    ) -> Result<Vec<CommerceReconciliationRunRecord>> {
        Err(unsupported_commerce_method(
            self.dialect(),
            "list_commerce_reconciliation_runs",
        ))
    }
    async fn insert_commerce_reconciliation_item(
        &self,
        _record: &CommerceReconciliationItemRecord,
    ) -> Result<CommerceReconciliationItemRecord> {
        Err(unsupported_commerce_method(
            self.dialect(),
            "insert_commerce_reconciliation_item",
        ))
    }
    async fn list_commerce_reconciliation_items(
        &self,
        _reconciliation_run_id: &str,
    ) -> Result<Vec<CommerceReconciliationItemRecord>> {
        Err(unsupported_commerce_method(
            self.dialect(),
            "list_commerce_reconciliation_items",
        ))
    }
    async fn insert_async_job(&self, record: &AsyncJobRecord) -> Result<AsyncJobRecord>;
    async fn list_async_jobs(&self) -> Result<Vec<AsyncJobRecord>>;
    async fn find_async_job(&self, job_id: &str) -> Result<Option<AsyncJobRecord>>;
    async fn insert_async_job_attempt(
        &self,
        record: &AsyncJobAttemptRecord,
    ) -> Result<AsyncJobAttemptRecord>;
    async fn list_async_job_attempts(&self, job_id: &str) -> Result<Vec<AsyncJobAttemptRecord>>;
    async fn insert_async_job_asset(
        &self,
        record: &AsyncJobAssetRecord,
    ) -> Result<AsyncJobAssetRecord>;
    async fn list_async_job_assets(&self, job_id: &str) -> Result<Vec<AsyncJobAssetRecord>>;
    async fn insert_async_job_callback(
        &self,
        record: &AsyncJobCallbackRecord,
    ) -> Result<AsyncJobCallbackRecord>;
    async fn list_async_job_callbacks(&self, job_id: &str) -> Result<Vec<AsyncJobCallbackRecord>>;
    async fn upsert_project_membership(
        &self,
        membership: &ProjectMembershipRecord,
    ) -> Result<ProjectMembershipRecord>;
    async fn find_project_membership(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectMembershipRecord>>;
    async fn delete_project_membership(&self, project_id: &str) -> Result<bool>;

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
    async fn insert_admin_audit_event(
        &self,
        record: &AdminAuditEventRecord,
    ) -> Result<AdminAuditEventRecord>;
    async fn list_admin_audit_events(&self) -> Result<Vec<AdminAuditEventRecord>>;

    async fn insert_gateway_api_key(
        &self,
        record: &GatewayApiKeyRecord,
    ) -> Result<GatewayApiKeyRecord>;
    async fn list_gateway_api_keys(&self) -> Result<Vec<GatewayApiKeyRecord>>;
    async fn find_gateway_api_key(&self, hashed_key: &str) -> Result<Option<GatewayApiKeyRecord>>;
    async fn delete_gateway_api_key(&self, hashed_key: &str) -> Result<bool>;
    async fn insert_api_key_group(&self, record: &ApiKeyGroupRecord) -> Result<ApiKeyGroupRecord>;
    async fn list_api_key_groups(&self) -> Result<Vec<ApiKeyGroupRecord>>;
    async fn find_api_key_group(&self, group_id: &str) -> Result<Option<ApiKeyGroupRecord>>;
    async fn delete_api_key_group(&self, group_id: &str) -> Result<bool>;

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
