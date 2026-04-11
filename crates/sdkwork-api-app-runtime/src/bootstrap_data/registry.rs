use anyhow::{Context, Result};
use async_trait::async_trait;

use sdkwork_api_app_billing::CommercialBillingAdminKernel;
use sdkwork_api_storage_core::{AccountKernelStore, AdminStore};

use super::manifest::BootstrapProfilePack;

pub(crate) struct BootstrapApplySummary {
    pub(crate) applied_stage_count: usize,
    pub(crate) applied_record_count: usize,
}

pub(crate) async fn apply_bootstrap_profile_pack(
    store: &dyn AdminStore,
    account_kernel: &dyn AccountKernelStore,
    commercial_billing: &dyn CommercialBillingAdminKernel,
    pack: &BootstrapProfilePack,
) -> Result<BootstrapApplySummary> {
    let context = BootstrapStageContext {
        store,
        account_kernel,
        commercial_billing,
    };
    let mut applied_stage_count = 0;
    let mut applied_record_count = 0;

    for stage in ordered_stages() {
        let applied_records = stage
            .apply(&context, pack)
            .await
            .with_context(|| format!("bootstrap stage {} failed", stage.name()))?;
        if applied_records > 0 {
            applied_stage_count += 1;
            applied_record_count += applied_records;
        }
    }

    Ok(BootstrapApplySummary {
        applied_stage_count,
        applied_record_count,
    })
}

struct BootstrapStageContext<'a> {
    store: &'a dyn AdminStore,
    account_kernel: &'a dyn AccountKernelStore,
    commercial_billing: &'a dyn CommercialBillingAdminKernel,
}

#[async_trait]
trait BootstrapStage: Send + Sync {
    fn name(&self) -> &'static str;

    async fn apply(
        &self,
        context: &BootstrapStageContext<'_>,
        pack: &BootstrapProfilePack,
    ) -> Result<usize>;
}

fn ordered_stages() -> Vec<Box<dyn BootstrapStage>> {
    vec![
        Box::new(ChannelStage),
        Box::new(ProviderStage),
        Box::new(ExtensionInstallationStage),
        Box::new(ExtensionInstanceStage),
        Box::new(ServiceRuntimeNodeStage),
        Box::new(ExtensionRuntimeRolloutStage),
        Box::new(ExtensionRuntimeRolloutParticipantStage),
        Box::new(StandaloneConfigRolloutStage),
        Box::new(StandaloneConfigRolloutParticipantStage),
        Box::new(OfficialProviderConfigStage),
        Box::new(ProviderAccountStage),
        Box::new(ModelStage),
        Box::new(ChannelModelStage),
        Box::new(ProviderModelStage),
        Box::new(ModelPriceStage),
        Box::new(TenantStage),
        Box::new(ProjectStage),
        Box::new(AdminUserStage),
        Box::new(PortalUserStage),
        Box::new(RoutingProfileStage),
        Box::new(RoutingPolicyStage),
        Box::new(ProjectRoutingPreferenceStage),
        Box::new(ApiKeyGroupStage),
        Box::new(CompiledRoutingSnapshotStage),
        Box::new(RoutingDecisionLogStage),
        Box::new(ProviderHealthSnapshotStage),
        Box::new(GatewayApiKeyStage),
        Box::new(BillingEventStage),
        Box::new(QuotaPolicyStage),
        Box::new(RateLimitPolicyStage),
        Box::new(PricingPlanStage),
        Box::new(PricingRateStage),
        Box::new(AccountStage),
        Box::new(AccountBenefitLotStage),
        Box::new(AccountHoldStage),
        Box::new(AccountHoldAllocationStage),
        Box::new(AccountLedgerEntryStage),
        Box::new(AccountLedgerAllocationStage),
        Box::new(RequestSettlementStage),
        Box::new(RequestMeterFactStage),
        Box::new(RequestMeterMetricStage),
        Box::new(PaymentMethodStage),
        Box::new(PaymentMethodCredentialBindingStage),
        Box::new(ProjectMembershipStage),
        Box::new(CouponTemplateStage),
        Box::new(MarketingCampaignStage),
        Box::new(CampaignBudgetStage),
        Box::new(CouponCodeStage),
        Box::new(CommerceOrderStage),
        Box::new(CommercePaymentAttemptStage),
        Box::new(CommercePaymentEventStage),
        Box::new(CommerceWebhookInboxStage),
        Box::new(CommerceRefundStage),
        Box::new(CommerceReconciliationRunStage),
        Box::new(CommerceReconciliationItemStage),
        Box::new(AccountCommerceReconciliationStateStage),
        Box::new(AsyncJobStage),
        Box::new(AsyncJobAttemptStage),
        Box::new(AsyncJobAssetStage),
        Box::new(AsyncJobCallbackStage),
    ]
}

struct ChannelStage;
struct ProviderStage;
struct ExtensionInstallationStage;
struct ExtensionInstanceStage;
struct ServiceRuntimeNodeStage;
struct ExtensionRuntimeRolloutStage;
struct ExtensionRuntimeRolloutParticipantStage;
struct StandaloneConfigRolloutStage;
struct StandaloneConfigRolloutParticipantStage;
struct OfficialProviderConfigStage;
struct ProviderAccountStage;
struct ModelStage;
struct ChannelModelStage;
struct ProviderModelStage;
struct ModelPriceStage;
struct TenantStage;
struct ProjectStage;
struct AdminUserStage;
struct PortalUserStage;
struct RoutingProfileStage;
struct RoutingPolicyStage;
struct ProjectRoutingPreferenceStage;
struct ApiKeyGroupStage;
struct CompiledRoutingSnapshotStage;
struct RoutingDecisionLogStage;
struct ProviderHealthSnapshotStage;
struct GatewayApiKeyStage;
struct BillingEventStage;
struct QuotaPolicyStage;
struct RateLimitPolicyStage;
struct PricingPlanStage;
struct PricingRateStage;
struct AccountStage;
struct AccountBenefitLotStage;
struct AccountHoldStage;
struct AccountHoldAllocationStage;
struct AccountLedgerEntryStage;
struct AccountLedgerAllocationStage;
struct RequestSettlementStage;
struct RequestMeterFactStage;
struct RequestMeterMetricStage;
struct PaymentMethodStage;
struct PaymentMethodCredentialBindingStage;
struct ProjectMembershipStage;
struct CouponTemplateStage;
struct MarketingCampaignStage;
struct CampaignBudgetStage;
struct CouponCodeStage;
struct CommerceOrderStage;
struct CommercePaymentAttemptStage;
struct CommercePaymentEventStage;
struct CommerceWebhookInboxStage;
struct CommerceRefundStage;
struct CommerceReconciliationRunStage;
struct CommerceReconciliationItemStage;
struct AccountCommerceReconciliationStateStage;
struct AsyncJobStage;
struct AsyncJobAttemptStage;
struct AsyncJobAssetStage;
struct AsyncJobCallbackStage;

macro_rules! impl_store_stage {
    ($stage_ty:ident, $name:literal, $field:ident, $method:ident) => {
        #[async_trait]
        impl BootstrapStage for $stage_ty {
            fn name(&self) -> &'static str {
                $name
            }

            async fn apply(
                &self,
                context: &BootstrapStageContext<'_>,
                pack: &BootstrapProfilePack,
            ) -> Result<usize> {
                for record in &pack.data.$field {
                    context.store.$method(record).await?;
                }
                Ok(pack.data.$field.len())
            }
        }
    };
}

macro_rules! impl_account_kernel_stage {
    ($stage_ty:ident, $name:literal, $field:ident, $method:ident) => {
        #[async_trait]
        impl BootstrapStage for $stage_ty {
            fn name(&self) -> &'static str {
                $name
            }

            async fn apply(
                &self,
                context: &BootstrapStageContext<'_>,
                pack: &BootstrapProfilePack,
            ) -> Result<usize> {
                for record in &pack.data.$field {
                    context.account_kernel.$method(record).await?;
                }
                Ok(pack.data.$field.len())
            }
        }
    };
}

impl_store_stage!(ChannelStage, "channels", channels, insert_channel);
impl_store_stage!(ProviderStage, "providers", providers, insert_provider);
impl_store_stage!(
    ExtensionInstallationStage,
    "extension_installations",
    extension_installations,
    insert_extension_installation
);
impl_store_stage!(
    ExtensionInstanceStage,
    "extension_instances",
    extension_instances,
    insert_extension_instance
);
impl_store_stage!(
    ServiceRuntimeNodeStage,
    "service_runtime_nodes",
    service_runtime_nodes,
    upsert_service_runtime_node
);
impl_store_stage!(
    ExtensionRuntimeRolloutStage,
    "extension_runtime_rollouts",
    extension_runtime_rollouts,
    insert_extension_runtime_rollout
);
impl_store_stage!(
    ExtensionRuntimeRolloutParticipantStage,
    "extension_runtime_rollout_participants",
    extension_runtime_rollout_participants,
    insert_extension_runtime_rollout_participant
);
impl_store_stage!(
    StandaloneConfigRolloutStage,
    "standalone_config_rollouts",
    standalone_config_rollouts,
    insert_standalone_config_rollout
);
impl_store_stage!(
    StandaloneConfigRolloutParticipantStage,
    "standalone_config_rollout_participants",
    standalone_config_rollout_participants,
    insert_standalone_config_rollout_participant
);
impl_store_stage!(
    OfficialProviderConfigStage,
    "official_provider_configs",
    official_provider_configs,
    upsert_official_provider_config
);
impl_store_stage!(
    ProviderAccountStage,
    "provider_accounts",
    provider_accounts,
    upsert_provider_account
);
impl_store_stage!(ModelStage, "models", models, insert_model);
impl_store_stage!(
    ChannelModelStage,
    "channel_models",
    channel_models,
    insert_channel_model
);
impl_store_stage!(
    ProviderModelStage,
    "provider_models",
    provider_models,
    upsert_provider_model
);
impl_store_stage!(
    ModelPriceStage,
    "model_prices",
    model_prices,
    insert_model_price
);
impl_store_stage!(TenantStage, "tenants", tenants, insert_tenant);
impl_store_stage!(ProjectStage, "projects", projects, insert_project);
impl_store_stage!(
    AdminUserStage,
    "admin_users",
    admin_users,
    insert_admin_user
);
impl_store_stage!(
    PortalUserStage,
    "portal_users",
    portal_users,
    insert_portal_user
);
impl_store_stage!(
    RoutingProfileStage,
    "routing_profiles",
    routing_profiles,
    insert_routing_profile
);
impl_store_stage!(
    RoutingPolicyStage,
    "routing_policies",
    routing_policies,
    insert_routing_policy
);
impl_store_stage!(
    ProjectRoutingPreferenceStage,
    "project_routing_preferences",
    project_preferences,
    insert_project_routing_preferences
);
impl_store_stage!(
    ApiKeyGroupStage,
    "api_key_groups",
    api_key_groups,
    insert_api_key_group
);
impl_store_stage!(
    CompiledRoutingSnapshotStage,
    "compiled_routing_snapshots",
    compiled_routing_snapshots,
    insert_compiled_routing_snapshot
);
impl_store_stage!(
    RoutingDecisionLogStage,
    "routing_decision_logs",
    routing_decision_logs,
    insert_routing_decision_log
);
impl_store_stage!(
    ProviderHealthSnapshotStage,
    "provider_health_snapshots",
    provider_health_snapshots,
    insert_provider_health_snapshot
);
impl_store_stage!(
    GatewayApiKeyStage,
    "gateway_api_keys",
    gateway_api_keys,
    insert_gateway_api_key
);
impl_store_stage!(
    BillingEventStage,
    "billing_events",
    billing_events,
    insert_billing_event
);
impl_store_stage!(
    QuotaPolicyStage,
    "quota_policies",
    quota_policies,
    insert_quota_policy
);
impl_store_stage!(
    RateLimitPolicyStage,
    "rate_limit_policies",
    rate_limit_policies,
    insert_rate_limit_policy
);
impl_account_kernel_stage!(AccountStage, "accounts", accounts, insert_account_record);
impl_account_kernel_stage!(
    AccountBenefitLotStage,
    "account_benefit_lots",
    account_benefit_lots,
    insert_account_benefit_lot
);
impl_account_kernel_stage!(
    AccountHoldStage,
    "account_holds",
    account_holds,
    insert_account_hold
);
impl_account_kernel_stage!(
    AccountHoldAllocationStage,
    "account_hold_allocations",
    account_hold_allocations,
    insert_account_hold_allocation
);
impl_account_kernel_stage!(
    AccountLedgerEntryStage,
    "account_ledger_entries",
    account_ledger_entries,
    insert_account_ledger_entry_record
);
impl_account_kernel_stage!(
    AccountLedgerAllocationStage,
    "account_ledger_allocations",
    account_ledger_allocations,
    insert_account_ledger_allocation
);
impl_account_kernel_stage!(
    RequestSettlementStage,
    "request_settlements",
    request_settlements,
    insert_request_settlement_record
);
impl_account_kernel_stage!(
    RequestMeterFactStage,
    "request_meter_facts",
    request_meter_facts,
    insert_request_meter_fact
);
impl_account_kernel_stage!(
    RequestMeterMetricStage,
    "request_meter_metrics",
    request_meter_metrics,
    insert_request_meter_metric
);
impl_store_stage!(
    PaymentMethodStage,
    "payment_methods",
    payment_methods,
    upsert_payment_method
);
impl_store_stage!(
    PaymentMethodCredentialBindingStage,
    "payment_method_credential_bindings",
    payment_method_credential_bindings,
    upsert_payment_method_credential_binding
);
impl_store_stage!(
    ProjectMembershipStage,
    "project_memberships",
    project_memberships,
    upsert_project_membership
);
impl_store_stage!(
    CouponTemplateStage,
    "coupon_templates",
    coupon_templates,
    insert_coupon_template_record
);
impl_store_stage!(
    MarketingCampaignStage,
    "marketing_campaigns",
    marketing_campaigns,
    insert_marketing_campaign_record
);
impl_store_stage!(
    CampaignBudgetStage,
    "campaign_budgets",
    campaign_budgets,
    insert_campaign_budget_record
);
impl_store_stage!(
    CouponCodeStage,
    "coupon_codes",
    coupon_codes,
    insert_coupon_code_record
);
impl_store_stage!(
    CommerceOrderStage,
    "commerce_orders",
    commerce_orders,
    insert_commerce_order
);
impl_store_stage!(
    CommercePaymentAttemptStage,
    "commerce_payment_attempts",
    commerce_payment_attempts,
    upsert_commerce_payment_attempt
);
impl_store_stage!(
    CommercePaymentEventStage,
    "commerce_payment_events",
    commerce_payment_events,
    upsert_commerce_payment_event
);
impl_store_stage!(
    CommerceWebhookInboxStage,
    "commerce_webhook_inbox_records",
    commerce_webhook_inbox_records,
    upsert_commerce_webhook_inbox
);
impl_store_stage!(
    CommerceRefundStage,
    "commerce_refunds",
    commerce_refunds,
    upsert_commerce_refund
);
impl_store_stage!(
    CommerceReconciliationRunStage,
    "commerce_reconciliation_runs",
    commerce_reconciliation_runs,
    insert_commerce_reconciliation_run
);
impl_store_stage!(
    CommerceReconciliationItemStage,
    "commerce_reconciliation_items",
    commerce_reconciliation_items,
    insert_commerce_reconciliation_item
);
impl_account_kernel_stage!(
    AccountCommerceReconciliationStateStage,
    "account_commerce_reconciliation_states",
    account_commerce_reconciliation_states,
    insert_account_commerce_reconciliation_state
);
impl_store_stage!(AsyncJobStage, "async_jobs", async_jobs, insert_async_job);
impl_store_stage!(
    AsyncJobAttemptStage,
    "async_job_attempts",
    async_job_attempts,
    insert_async_job_attempt
);
impl_store_stage!(
    AsyncJobAssetStage,
    "async_job_assets",
    async_job_assets,
    insert_async_job_asset
);
impl_store_stage!(
    AsyncJobCallbackStage,
    "async_job_callbacks",
    async_job_callbacks,
    insert_async_job_callback
);

#[async_trait]
impl BootstrapStage for PricingPlanStage {
    fn name(&self) -> &'static str {
        "pricing_plans"
    }

    async fn apply(
        &self,
        context: &BootstrapStageContext<'_>,
        pack: &BootstrapProfilePack,
    ) -> Result<usize> {
        for record in &pack.data.pricing_plans {
            context
                .commercial_billing
                .insert_pricing_plan_record(record)
                .await?;
        }
        Ok(pack.data.pricing_plans.len())
    }
}

#[async_trait]
impl BootstrapStage for PricingRateStage {
    fn name(&self) -> &'static str {
        "pricing_rates"
    }

    async fn apply(
        &self,
        context: &BootstrapStageContext<'_>,
        pack: &BootstrapProfilePack,
    ) -> Result<usize> {
        for record in &pack.data.pricing_rates {
            context
                .commercial_billing
                .insert_pricing_rate_record(record)
                .await?;
        }
        Ok(pack.data.pricing_rates.len())
    }
}
