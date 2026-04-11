use anyhow::{bail, Context, Result};
use argon2::password_hash::PasswordHash;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use sdkwork_api_domain_billing::{
    AccountBenefitLotRecord, AccountCommerceReconciliationStateRecord, AccountHoldAllocationRecord,
    AccountHoldRecord, AccountHoldStatus, AccountLedgerAllocationRecord, AccountLedgerEntryRecord,
    AccountRecord, BillingAccountingMode, BillingEventRecord, PricingPlanRecord,
    PricingRateRecord, QuotaPolicy, RequestSettlementRecord, RequestSettlementStatus,
};
use sdkwork_api_domain_catalog::{
    Channel, ChannelModelRecord, ModelCatalogEntry, ModelPriceRecord, ProviderAccountRecord,
    ProviderModelRecord, ProxyProvider,
};
use sdkwork_api_domain_commerce::{
    CommerceOrderRecord, CommercePaymentAttemptRecord, CommercePaymentEventProcessingStatus,
    CommercePaymentEventRecord,
    CommerceReconciliationItemRecord, CommerceReconciliationRunRecord, CommerceRefundRecord,
    CommerceWebhookInboxRecord, PaymentMethodCredentialBindingRecord, PaymentMethodRecord,
    ProjectMembershipRecord,
};
use sdkwork_api_domain_credential::OfficialProviderConfig;
use sdkwork_api_domain_identity::{
    AdminUserRecord, ApiKeyGroupRecord, GatewayApiKeyRecord, PortalUserRecord,
};
use sdkwork_api_domain_jobs::{
    AsyncJobAssetRecord, AsyncJobAttemptRecord, AsyncJobAttemptStatus, AsyncJobCallbackRecord,
    AsyncJobCallbackStatus, AsyncJobRecord, AsyncJobStatus,
};
use sdkwork_api_domain_marketing::{
    CampaignBudgetRecord, CouponCodeRecord, CouponTemplateRecord, MarketingCampaignRecord,
};
use sdkwork_api_domain_rate_limit::RateLimitPolicy;
use sdkwork_api_domain_routing::{
    CompiledRoutingSnapshotRecord, ProjectRoutingPreferences, ProviderHealthSnapshot,
    RoutingCandidateAssessment, RoutingCandidateHealth, RoutingDecisionLog, RoutingPolicy,
    RoutingProfileRecord, RoutingStrategy,
};
use sdkwork_api_domain_tenant::{Project, Tenant};
use sdkwork_api_domain_usage::{RequestMeterFactRecord, RequestMeterMetricRecord, UsageCaptureStatus};
use sdkwork_api_extension_core::{ExtensionInstallation, ExtensionInstance};
use sdkwork_api_storage_core::{
    ExtensionRuntimeRolloutParticipantRecord, ExtensionRuntimeRolloutRecord,
    ServiceRuntimeNodeRecord, StandaloneConfigRolloutParticipantRecord,
    StandaloneConfigRolloutRecord,
};

use super::profile_manifest_path;

#[derive(Debug, Clone)]
pub(crate) struct BootstrapProfilePack {
    pub(crate) data_root: PathBuf,
    pub(crate) profile_id: String,
    #[allow(dead_code)]
    pub(crate) description: Option<String>,
    #[allow(dead_code)]
    pub(crate) release_version: Option<String>,
    #[allow(dead_code)]
    pub(crate) update_ids: Vec<String>,
    pub(crate) data: BootstrapDataPack,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct BootstrapDataPack {
    pub(crate) channels: Vec<Channel>,
    pub(crate) providers: Vec<ProxyProvider>,
    pub(crate) official_provider_configs: Vec<OfficialProviderConfig>,
    pub(crate) provider_accounts: Vec<ProviderAccountRecord>,
    pub(crate) models: Vec<ModelCatalogEntry>,
    pub(crate) channel_models: Vec<ChannelModelRecord>,
    pub(crate) provider_models: Vec<ProviderModelRecord>,
    pub(crate) model_prices: Vec<ModelPriceRecord>,
    pub(crate) tenants: Vec<Tenant>,
    pub(crate) projects: Vec<Project>,
    pub(crate) admin_users: Vec<AdminUserRecord>,
    pub(crate) portal_users: Vec<PortalUserRecord>,
    pub(crate) gateway_api_keys: Vec<GatewayApiKeyRecord>,
    pub(crate) extension_installations: Vec<ExtensionInstallation>,
    pub(crate) extension_instances: Vec<ExtensionInstance>,
    pub(crate) service_runtime_nodes: Vec<ServiceRuntimeNodeRecord>,
    pub(crate) extension_runtime_rollouts: Vec<ExtensionRuntimeRolloutRecord>,
    pub(crate) extension_runtime_rollout_participants:
        Vec<ExtensionRuntimeRolloutParticipantRecord>,
    pub(crate) standalone_config_rollouts: Vec<StandaloneConfigRolloutRecord>,
    pub(crate) standalone_config_rollout_participants:
        Vec<StandaloneConfigRolloutParticipantRecord>,
    pub(crate) routing_profiles: Vec<RoutingProfileRecord>,
    pub(crate) routing_policies: Vec<RoutingPolicy>,
    pub(crate) project_preferences: Vec<ProjectRoutingPreferences>,
    pub(crate) api_key_groups: Vec<ApiKeyGroupRecord>,
    pub(crate) compiled_routing_snapshots: Vec<CompiledRoutingSnapshotRecord>,
    pub(crate) routing_decision_logs: Vec<RoutingDecisionLog>,
    pub(crate) provider_health_snapshots: Vec<ProviderHealthSnapshot>,
    pub(crate) billing_events: Vec<BillingEventRecord>,
    pub(crate) quota_policies: Vec<QuotaPolicy>,
    pub(crate) rate_limit_policies: Vec<RateLimitPolicy>,
    pub(crate) pricing_plans: Vec<PricingPlanRecord>,
    pub(crate) pricing_rates: Vec<PricingRateRecord>,
    pub(crate) accounts: Vec<AccountRecord>,
    pub(crate) account_benefit_lots: Vec<AccountBenefitLotRecord>,
    pub(crate) account_holds: Vec<AccountHoldRecord>,
    pub(crate) account_hold_allocations: Vec<AccountHoldAllocationRecord>,
    pub(crate) account_ledger_entries: Vec<AccountLedgerEntryRecord>,
    pub(crate) account_ledger_allocations: Vec<AccountLedgerAllocationRecord>,
    pub(crate) request_meter_facts: Vec<RequestMeterFactRecord>,
    pub(crate) request_meter_metrics: Vec<RequestMeterMetricRecord>,
    pub(crate) request_settlements: Vec<RequestSettlementRecord>,
    pub(crate) account_commerce_reconciliation_states:
        Vec<AccountCommerceReconciliationStateRecord>,
    pub(crate) payment_methods: Vec<PaymentMethodRecord>,
    pub(crate) payment_method_credential_bindings: Vec<PaymentMethodCredentialBindingRecord>,
    pub(crate) project_memberships: Vec<ProjectMembershipRecord>,
    pub(crate) commerce_orders: Vec<CommerceOrderRecord>,
    pub(crate) commerce_payment_attempts: Vec<CommercePaymentAttemptRecord>,
    pub(crate) commerce_payment_events: Vec<CommercePaymentEventRecord>,
    pub(crate) commerce_webhook_inbox_records: Vec<CommerceWebhookInboxRecord>,
    pub(crate) commerce_refunds: Vec<CommerceRefundRecord>,
    pub(crate) commerce_reconciliation_runs: Vec<CommerceReconciliationRunRecord>,
    pub(crate) commerce_reconciliation_items: Vec<CommerceReconciliationItemRecord>,
    pub(crate) async_jobs: Vec<AsyncJobRecord>,
    pub(crate) async_job_attempts: Vec<AsyncJobAttemptRecord>,
    pub(crate) async_job_assets: Vec<AsyncJobAssetRecord>,
    pub(crate) async_job_callbacks: Vec<AsyncJobCallbackRecord>,
    pub(crate) coupon_templates: Vec<CouponTemplateRecord>,
    pub(crate) marketing_campaigns: Vec<MarketingCampaignRecord>,
    pub(crate) campaign_budgets: Vec<CampaignBudgetRecord>,
    pub(crate) coupon_codes: Vec<CouponCodeRecord>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct BootstrapBundleRefs {
    #[serde(default)]
    channels: Vec<String>,
    #[serde(default)]
    providers: Vec<String>,
    #[serde(default)]
    official_provider_configs: Vec<String>,
    #[serde(default)]
    provider_accounts: Vec<String>,
    #[serde(default)]
    models: Vec<String>,
    #[serde(default)]
    channel_models: Vec<String>,
    #[serde(default)]
    provider_models: Vec<String>,
    #[serde(default)]
    model_prices: Vec<String>,
    #[serde(default)]
    tenants: Vec<String>,
    #[serde(default)]
    projects: Vec<String>,
    #[serde(default)]
    identities: Vec<String>,
    #[serde(default)]
    extensions: Vec<String>,
    #[serde(default)]
    service_runtime_nodes: Vec<String>,
    #[serde(default)]
    extension_runtime_rollouts: Vec<String>,
    #[serde(default)]
    standalone_config_rollouts: Vec<String>,
    #[serde(default)]
    routing: Vec<String>,
    #[serde(default)]
    api_key_groups: Vec<String>,
    #[serde(default)]
    observability: Vec<String>,
    #[serde(default)]
    billing: Vec<String>,
    #[serde(default)]
    quota_policies: Vec<String>,
    #[serde(default)]
    pricing: Vec<String>,
    #[serde(default)]
    accounts: Vec<String>,
    #[serde(default)]
    account_benefit_lots: Vec<String>,
    #[serde(default)]
    account_holds: Vec<String>,
    #[serde(default)]
    account_ledger: Vec<String>,
    #[serde(default)]
    request_metering: Vec<String>,
    #[serde(default)]
    request_settlements: Vec<String>,
    #[serde(default)]
    account_reconciliation: Vec<String>,
    #[serde(default)]
    payment_methods: Vec<String>,
    #[serde(default)]
    marketing: Vec<String>,
    #[serde(default)]
    commerce: Vec<String>,
    #[serde(default)]
    jobs: Vec<String>,
}

impl BootstrapBundleRefs {
    fn extend_from(&mut self, other: &Self) {
        self.channels.extend(other.channels.iter().cloned());
        self.providers.extend(other.providers.iter().cloned());
        self.official_provider_configs
            .extend(other.official_provider_configs.iter().cloned());
        self.provider_accounts
            .extend(other.provider_accounts.iter().cloned());
        self.models.extend(other.models.iter().cloned());
        self.channel_models
            .extend(other.channel_models.iter().cloned());
        self.provider_models
            .extend(other.provider_models.iter().cloned());
        self.model_prices.extend(other.model_prices.iter().cloned());
        self.tenants.extend(other.tenants.iter().cloned());
        self.projects.extend(other.projects.iter().cloned());
        self.identities.extend(other.identities.iter().cloned());
        self.extensions.extend(other.extensions.iter().cloned());
        self.service_runtime_nodes
            .extend(other.service_runtime_nodes.iter().cloned());
        self.extension_runtime_rollouts
            .extend(other.extension_runtime_rollouts.iter().cloned());
        self.standalone_config_rollouts
            .extend(other.standalone_config_rollouts.iter().cloned());
        self.routing.extend(other.routing.iter().cloned());
        self.api_key_groups
            .extend(other.api_key_groups.iter().cloned());
        self.observability
            .extend(other.observability.iter().cloned());
        self.billing.extend(other.billing.iter().cloned());
        self.quota_policies
            .extend(other.quota_policies.iter().cloned());
        self.pricing.extend(other.pricing.iter().cloned());
        self.accounts.extend(other.accounts.iter().cloned());
        self.account_benefit_lots
            .extend(other.account_benefit_lots.iter().cloned());
        self.account_holds
            .extend(other.account_holds.iter().cloned());
        self.account_ledger
            .extend(other.account_ledger.iter().cloned());
        self.request_metering
            .extend(other.request_metering.iter().cloned());
        self.request_settlements
            .extend(other.request_settlements.iter().cloned());
        self.account_reconciliation
            .extend(other.account_reconciliation.iter().cloned());
        self.payment_methods
            .extend(other.payment_methods.iter().cloned());
        self.marketing.extend(other.marketing.iter().cloned());
        self.commerce.extend(other.commerce.iter().cloned());
        self.jobs.extend(other.jobs.iter().cloned());
    }
}

#[derive(Debug, Deserialize)]
struct BootstrapProfileManifest {
    profile_id: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    release_version: Option<String>,
    #[serde(default)]
    updates: Vec<String>,
    #[serde(flatten)]
    refs: BootstrapBundleRefs,
}

#[derive(Debug, Deserialize)]
struct BootstrapUpdateManifest {
    update_id: String,
    #[serde(default)]
    #[allow(dead_code)]
    description: Option<String>,
    #[serde(default)]
    release_version: Option<String>,
    #[serde(default)]
    depends_on: Vec<String>,
    #[serde(flatten)]
    refs: BootstrapBundleRefs,
}

#[derive(Debug, Default, Deserialize)]
struct RoutingBundle {
    #[serde(default)]
    profiles: Vec<RoutingProfileRecord>,
    #[serde(default)]
    policies: Vec<RoutingPolicy>,
    #[serde(default)]
    project_preferences: Vec<ProjectRoutingPreferences>,
}

#[derive(Debug, Default, Deserialize)]
struct ExtensionsBundle {
    #[serde(default)]
    installations: Vec<ExtensionInstallation>,
    #[serde(default)]
    instances: Vec<ExtensionInstance>,
}

#[derive(Debug, Default, Deserialize)]
struct IdentitiesBundle {
    #[serde(default)]
    admin_users: Vec<AdminUserRecord>,
    #[serde(default)]
    portal_users: Vec<PortalUserRecord>,
    #[serde(default)]
    gateway_api_keys: Vec<GatewayApiKeyRecord>,
}

#[derive(Debug, Default, Deserialize)]
struct ExtensionRuntimeRolloutsBundle {
    #[serde(default)]
    rollouts: Vec<ExtensionRuntimeRolloutRecord>,
    #[serde(default)]
    participants: Vec<ExtensionRuntimeRolloutParticipantRecord>,
}

#[derive(Debug, Default, Deserialize)]
struct StandaloneConfigRolloutsBundle {
    #[serde(default)]
    rollouts: Vec<StandaloneConfigRolloutRecord>,
    #[serde(default)]
    participants: Vec<StandaloneConfigRolloutParticipantRecord>,
}

#[derive(Debug, Default, Deserialize)]
struct ObservabilityBundle {
    #[serde(default)]
    compiled_routing_snapshots: Vec<CompiledRoutingSnapshotRecord>,
    #[serde(default)]
    routing_decision_logs: Vec<RoutingDecisionLog>,
    #[serde(default)]
    provider_health_snapshots: Vec<ProviderHealthSnapshot>,
}

#[derive(Debug, Default, Deserialize)]
struct BillingBundle {
    #[serde(default)]
    billing_events: Vec<BillingEventRecord>,
}

#[derive(Debug, Default, Deserialize)]
struct QuotaPoliciesBundle {
    #[serde(default)]
    quota_policies: Vec<QuotaPolicy>,
    #[serde(default)]
    rate_limit_policies: Vec<RateLimitPolicy>,
}

#[derive(Debug, Default, Deserialize)]
struct PricingBundle {
    #[serde(default)]
    plans: Vec<PricingPlanRecord>,
    #[serde(default)]
    rates: Vec<PricingRateRecord>,
}

#[derive(Debug, Default, Deserialize)]
struct AccountHoldsBundle {
    #[serde(default)]
    holds: Vec<AccountHoldRecord>,
    #[serde(default)]
    allocations: Vec<AccountHoldAllocationRecord>,
}

#[derive(Debug, Default, Deserialize)]
struct AccountLedgerBundle {
    #[serde(default)]
    entries: Vec<AccountLedgerEntryRecord>,
    #[serde(default)]
    allocations: Vec<AccountLedgerAllocationRecord>,
}

#[derive(Debug, Default, Deserialize)]
struct RequestMeteringBundle {
    #[serde(default)]
    facts: Vec<RequestMeterFactRecord>,
    #[serde(default)]
    metrics: Vec<RequestMeterMetricRecord>,
}

#[derive(Debug, Default, Deserialize)]
struct PaymentMethodsBundle {
    #[serde(default)]
    payment_methods: Vec<PaymentMethodRecord>,
    #[serde(default)]
    credential_bindings: Vec<PaymentMethodCredentialBindingRecord>,
    #[serde(default)]
    project_memberships: Vec<ProjectMembershipRecord>,
}

#[derive(Debug, Default, Deserialize)]
struct CommerceBundle {
    #[serde(default)]
    orders: Vec<CommerceOrderRecord>,
    #[serde(default)]
    payment_attempts: Vec<CommercePaymentAttemptRecord>,
    #[serde(default)]
    payment_events: Vec<CommercePaymentEventRecord>,
    #[serde(default)]
    webhook_inbox_records: Vec<CommerceWebhookInboxRecord>,
    #[serde(default)]
    refunds: Vec<CommerceRefundRecord>,
    #[serde(default)]
    reconciliation_runs: Vec<CommerceReconciliationRunRecord>,
    #[serde(default)]
    reconciliation_items: Vec<CommerceReconciliationItemRecord>,
}

#[derive(Debug, Default, Deserialize)]
struct JobsBundle {
    #[serde(default)]
    jobs: Vec<AsyncJobRecord>,
    #[serde(default)]
    attempts: Vec<AsyncJobAttemptRecord>,
    #[serde(default)]
    assets: Vec<AsyncJobAssetRecord>,
    #[serde(default)]
    callbacks: Vec<AsyncJobCallbackRecord>,
}

#[derive(Debug, Default, Deserialize)]
struct MarketingBundle {
    #[serde(default)]
    coupon_templates: Vec<CouponTemplateRecord>,
    #[serde(default)]
    marketing_campaigns: Vec<MarketingCampaignRecord>,
    #[serde(default)]
    campaign_budgets: Vec<CampaignBudgetRecord>,
    #[serde(default)]
    coupon_codes: Vec<CouponCodeRecord>,
}

pub(crate) fn load_bootstrap_profile_pack(
    data_root: &Path,
    profile_id: &str,
) -> Result<BootstrapProfilePack> {
    let data_root = fs::canonicalize(data_root).with_context(|| {
        format!(
            "failed to canonicalize bootstrap data root {}",
            data_root.display()
        )
    })?;
    let manifest_path = profile_manifest_path(&data_root, profile_id);
    let manifest: BootstrapProfileManifest = read_json_file(&manifest_path)?;

    if manifest.profile_id != profile_id {
        bail!(
            "bootstrap manifest {} declares profile_id={} but expected {}",
            manifest_path.display(),
            manifest.profile_id,
            profile_id
        );
    }

    let mut refs = manifest.refs.clone();
    let updates = load_update_manifests(&data_root, &manifest.updates)?;
    let update_ids = updates
        .iter()
        .map(|update| update.update_id.clone())
        .collect::<Vec<_>>();
    for update in &updates {
        refs.extend_from(&update.refs);
    }

    let mut data = BootstrapDataPack::default();
    data.channels = load_json_array(&data_root, &refs.channels, "channels")?;
    data.providers = load_json_array(&data_root, &refs.providers, "providers")?;
    data.official_provider_configs = load_json_array(
        &data_root,
        &refs.official_provider_configs,
        "official_provider_configs",
    )?;
    data.provider_accounts =
        load_json_array(&data_root, &refs.provider_accounts, "provider_accounts")?;
    data.models = load_json_array(&data_root, &refs.models, "models")?;
    data.channel_models = load_json_array(&data_root, &refs.channel_models, "channel_models")?;
    data.provider_models = load_json_array(&data_root, &refs.provider_models, "provider_models")?;
    data.model_prices = load_json_array(&data_root, &refs.model_prices, "model_prices")?;
    data.tenants = load_json_array(&data_root, &refs.tenants, "tenants")?;
    data.projects = load_json_array(&data_root, &refs.projects, "projects")?;
    let identities = load_identities_bundle(&data_root, &refs.identities)?;
    data.admin_users = identities.admin_users;
    data.portal_users = identities.portal_users;
    data.gateway_api_keys = identities.gateway_api_keys;
    let extensions = load_extensions_bundle(&data_root, &refs.extensions)?;
    data.extension_installations = extensions.installations;
    data.extension_instances = extensions.instances;
    data.service_runtime_nodes = load_json_array(
        &data_root,
        &refs.service_runtime_nodes,
        "service_runtime_nodes",
    )?;
    let extension_runtime_rollouts =
        load_extension_runtime_rollouts_bundle(&data_root, &refs.extension_runtime_rollouts)?;
    data.extension_runtime_rollouts = extension_runtime_rollouts.rollouts;
    data.extension_runtime_rollout_participants = extension_runtime_rollouts.participants;
    let standalone_config_rollouts =
        load_standalone_config_rollouts_bundle(&data_root, &refs.standalone_config_rollouts)?;
    data.standalone_config_rollouts = standalone_config_rollouts.rollouts;
    data.standalone_config_rollout_participants = standalone_config_rollouts.participants;
    data.api_key_groups = load_json_array(&data_root, &refs.api_key_groups, "api_key_groups")?;

    let routing = load_routing_bundle(&data_root, &refs.routing)?;
    data.routing_profiles = routing.profiles;
    data.routing_policies = routing.policies;
    data.project_preferences = routing.project_preferences;

    let observability = load_observability_bundle(&data_root, &refs.observability)?;
    data.compiled_routing_snapshots = observability.compiled_routing_snapshots;
    data.routing_decision_logs = observability.routing_decision_logs;
    data.provider_health_snapshots = observability.provider_health_snapshots;

    let billing = load_billing_bundle(&data_root, &refs.billing)?;
    data.billing_events = billing.billing_events;

    let quota_policies = load_quota_policies_bundle(&data_root, &refs.quota_policies)?;
    data.quota_policies = quota_policies.quota_policies;
    data.rate_limit_policies = quota_policies.rate_limit_policies;

    let pricing = load_pricing_bundle(&data_root, &refs.pricing)?;
    data.pricing_plans = pricing.plans;
    data.pricing_rates = pricing.rates;

    data.accounts = load_json_array(&data_root, &refs.accounts, "accounts")?;
    data.account_benefit_lots = load_json_array(
        &data_root,
        &refs.account_benefit_lots,
        "account_benefit_lots",
    )?;
    let account_holds = load_account_holds_bundle(&data_root, &refs.account_holds)?;
    data.account_holds = account_holds.holds;
    data.account_hold_allocations = account_holds.allocations;
    let account_ledger = load_account_ledger_bundle(&data_root, &refs.account_ledger)?;
    data.account_ledger_entries = account_ledger.entries;
    data.account_ledger_allocations = account_ledger.allocations;
    let request_metering = load_request_metering_bundle(&data_root, &refs.request_metering)?;
    data.request_meter_facts = request_metering.facts;
    data.request_meter_metrics = request_metering.metrics;
    data.request_settlements =
        load_json_array(&data_root, &refs.request_settlements, "request_settlements")?;
    data.account_commerce_reconciliation_states = load_json_array(
        &data_root,
        &refs.account_reconciliation,
        "account_reconciliation",
    )?;

    let payment_methods = load_payment_methods_bundle(&data_root, &refs.payment_methods)?;
    data.payment_methods = payment_methods.payment_methods;
    data.payment_method_credential_bindings = payment_methods.credential_bindings;
    data.project_memberships = payment_methods.project_memberships;

    let marketing = load_marketing_bundle(&data_root, &refs.marketing)?;
    data.coupon_templates = marketing.coupon_templates;
    data.marketing_campaigns = marketing.marketing_campaigns;
    data.campaign_budgets = marketing.campaign_budgets;
    data.coupon_codes = marketing.coupon_codes;

    let commerce = load_commerce_bundle(&data_root, &refs.commerce)?;
    data.commerce_orders = commerce.orders;
    data.commerce_payment_attempts = commerce.payment_attempts;
    data.commerce_payment_events = commerce.payment_events;
    data.commerce_webhook_inbox_records = commerce.webhook_inbox_records;
    data.commerce_refunds = commerce.refunds;
    data.commerce_reconciliation_runs = commerce.reconciliation_runs;
    data.commerce_reconciliation_items = commerce.reconciliation_items;

    let jobs = load_jobs_bundle(&data_root, &refs.jobs)?;
    data.async_jobs = jobs.jobs;
    data.async_job_attempts = jobs.attempts;
    data.async_job_assets = jobs.assets;
    data.async_job_callbacks = jobs.callbacks;
    collapse_bootstrap_data_pack_updates(&mut data);

    let pack = BootstrapProfilePack {
        data_root,
        profile_id: manifest.profile_id,
        description: manifest.description,
        release_version: manifest.release_version,
        update_ids,
        data,
    };
    validate_bootstrap_profile_pack(&pack)?;
    Ok(pack)
}

fn load_update_manifests(
    data_root: &Path,
    relative_paths: &[String],
) -> Result<Vec<BootstrapUpdateManifest>> {
    let mut seen_update_paths = HashSet::new();
    let mut seen_update_ids = HashSet::new();
    let mut loaded_update_ids = HashSet::new();
    let mut manifests = Vec::new();

    for relative_path in relative_paths {
        let path = resolve_data_file_path(data_root, relative_path)?;
        if !seen_update_paths.insert(path.clone()) {
            bail!(
                "bootstrap update manifest {} is referenced more than once",
                path.display()
            );
        }

        let manifest: BootstrapUpdateManifest = read_json_file(&path).with_context(|| {
            format!(
                "failed to parse bootstrap update manifest {}",
                path.display()
            )
        })?;
        let update_id = manifest.update_id.trim();
        if update_id.is_empty() {
            bail!(
                "bootstrap update manifest {} has empty update_id",
                path.display()
            );
        }
        if !seen_update_ids.insert(update_id.to_owned()) {
            bail!(
                "bootstrap update id {} is declared more than once",
                update_id
            );
        }
        if let Some(release_version) = manifest.release_version.as_deref() {
            if release_version.trim().is_empty() {
                bail!(
                    "bootstrap update {} in {} has empty release_version",
                    update_id,
                    path.display()
                );
            }
        }

        let mut seen_dependencies = HashSet::new();
        for dependency in &manifest.depends_on {
            let dependency = dependency.trim();
            if dependency.is_empty() {
                bail!(
                    "bootstrap update {} in {} has empty dependency id",
                    update_id,
                    path.display()
                );
            }
            if !seen_dependencies.insert(dependency.to_owned()) {
                bail!(
                    "bootstrap update {} in {} declares dependency {} more than once",
                    update_id,
                    path.display(),
                    dependency
                );
            }
            if !loaded_update_ids.contains(dependency) {
                bail!(
                    "bootstrap update {} in {} depends on missing or unordered update {}",
                    update_id,
                    path.display(),
                    dependency
                );
            }
        }

        loaded_update_ids.insert(update_id.to_owned());
        manifests.push(manifest);
    }

    Ok(manifests)
}

fn collapse_bootstrap_data_pack_updates(data: &mut BootstrapDataPack) {
    collapse_last_wins(&mut data.channels, |record| record.id.clone());
    collapse_last_wins(&mut data.providers, |record| record.id.clone());
    collapse_last_wins(&mut data.official_provider_configs, |record| {
        record.provider_id.clone()
    });
    collapse_last_wins(&mut data.provider_accounts, |record| {
        record.provider_account_id.clone()
    });
    collapse_last_wins(&mut data.models, |record| {
        format!("{}::{}", record.external_name, record.provider_id)
    });
    collapse_last_wins(&mut data.channel_models, |record| {
        format!("{}::{}", record.channel_id, record.model_id)
    });
    collapse_last_wins(&mut data.provider_models, |record| {
        format!(
            "{}::{}::{}",
            record.proxy_provider_id, record.channel_id, record.model_id
        )
    });
    collapse_last_wins(&mut data.model_prices, |record| {
        format!(
            "{}::{}::{}",
            record.channel_id, record.model_id, record.proxy_provider_id
        )
    });
    collapse_last_wins(&mut data.tenants, |record| record.id.clone());
    collapse_last_wins(&mut data.projects, |record| record.id.clone());
    collapse_last_wins(&mut data.admin_users, |record| record.id.clone());
    collapse_last_wins(&mut data.portal_users, |record| record.id.clone());
    collapse_last_wins(&mut data.gateway_api_keys, |record| {
        record.hashed_key.clone()
    });
    collapse_last_wins(&mut data.extension_installations, |record| {
        record.installation_id.clone()
    });
    collapse_last_wins(&mut data.extension_instances, |record| {
        record.instance_id.clone()
    });
    collapse_last_wins(&mut data.service_runtime_nodes, |record| {
        record.node_id.clone()
    });
    collapse_last_wins(&mut data.extension_runtime_rollouts, |record| {
        record.rollout_id.clone()
    });
    collapse_last_wins(&mut data.extension_runtime_rollout_participants, |record| {
        format!("{}::{}", record.rollout_id, record.node_id)
    });
    collapse_last_wins(&mut data.standalone_config_rollouts, |record| {
        record.rollout_id.clone()
    });
    collapse_last_wins(&mut data.standalone_config_rollout_participants, |record| {
        format!("{}::{}", record.rollout_id, record.node_id)
    });
    collapse_last_wins(&mut data.routing_profiles, |record| {
        record.profile_id.clone()
    });
    collapse_last_wins(&mut data.routing_policies, |record| {
        record.policy_id.clone()
    });
    collapse_last_wins(&mut data.project_preferences, |record| {
        record.project_id.clone()
    });
    collapse_last_wins(&mut data.api_key_groups, |record| record.group_id.clone());
    collapse_last_wins(&mut data.compiled_routing_snapshots, |record| {
        record.snapshot_id.clone()
    });
    collapse_last_wins(&mut data.routing_decision_logs, |record| {
        record.decision_id.clone()
    });
    collapse_last_wins(
        &mut data.provider_health_snapshots,
        provider_health_snapshot_key,
    );
    collapse_last_wins(&mut data.billing_events, |record| record.event_id.clone());
    collapse_last_wins(&mut data.quota_policies, |record| record.policy_id.clone());
    collapse_last_wins(&mut data.rate_limit_policies, |record| {
        record.policy_id.clone()
    });
    collapse_last_wins(&mut data.pricing_plans, |record| {
        record.pricing_plan_id.to_string()
    });
    collapse_last_wins(&mut data.pricing_rates, |record| {
        record.pricing_rate_id.to_string()
    });
    collapse_last_wins(&mut data.accounts, |record| record.account_id.to_string());
    collapse_last_wins(&mut data.account_benefit_lots, |record| {
        record.lot_id.to_string()
    });
    collapse_last_wins(&mut data.account_holds, |record| record.hold_id.to_string());
    collapse_last_wins(&mut data.account_hold_allocations, |record| {
        record.hold_allocation_id.to_string()
    });
    collapse_last_wins(&mut data.account_ledger_entries, |record| {
        record.ledger_entry_id.to_string()
    });
    collapse_last_wins(&mut data.account_ledger_allocations, |record| {
        record.ledger_allocation_id.to_string()
    });
    collapse_last_wins(&mut data.request_meter_facts, |record| {
        record.request_id.to_string()
    });
    collapse_last_wins(&mut data.request_meter_metrics, |record| {
        record.request_metric_id.to_string()
    });
    collapse_last_wins(&mut data.request_settlements, |record| {
        record.request_settlement_id.to_string()
    });
    collapse_last_wins(&mut data.account_commerce_reconciliation_states, |record| {
        format!("{}::{}", record.account_id, record.project_id)
    });
    collapse_last_wins(&mut data.payment_methods, |record| {
        record.payment_method_id.clone()
    });
    collapse_last_wins(&mut data.payment_method_credential_bindings, |record| {
        record.binding_id.clone()
    });
    collapse_last_wins(&mut data.project_memberships, |record| {
        record.membership_id.clone()
    });
    collapse_last_wins(&mut data.commerce_orders, |record| record.order_id.clone());
    collapse_last_wins(&mut data.commerce_payment_attempts, |record| {
        record.payment_attempt_id.clone()
    });
    collapse_last_wins(&mut data.commerce_payment_events, |record| {
        record.payment_event_id.clone()
    });
    collapse_last_wins(&mut data.commerce_webhook_inbox_records, |record| {
        record.webhook_inbox_id.clone()
    });
    collapse_last_wins(&mut data.commerce_refunds, |record| {
        record.refund_id.clone()
    });
    collapse_last_wins(&mut data.commerce_reconciliation_runs, |record| {
        record.reconciliation_run_id.clone()
    });
    collapse_last_wins(&mut data.commerce_reconciliation_items, |record| {
        record.reconciliation_item_id.clone()
    });
    collapse_last_wins(&mut data.async_jobs, |record| record.job_id.clone());
    collapse_last_wins(&mut data.async_job_attempts, |record| {
        record.attempt_id.to_string()
    });
    collapse_last_wins(&mut data.async_job_assets, |record| record.asset_id.clone());
    collapse_last_wins(&mut data.async_job_callbacks, |record| {
        record.callback_id.to_string()
    });
    collapse_last_wins(&mut data.coupon_templates, |record| {
        record.coupon_template_id.clone()
    });
    collapse_last_wins(&mut data.marketing_campaigns, |record| {
        record.marketing_campaign_id.clone()
    });
    collapse_last_wins(&mut data.campaign_budgets, |record| {
        record.campaign_budget_id.clone()
    });
    collapse_last_wins(&mut data.coupon_codes, |record| {
        record.coupon_code_id.clone()
    });
}

fn collapse_last_wins<T, K, F>(records: &mut Vec<T>, key_fn: F)
where
    K: Eq + std::hash::Hash,
    F: Fn(&T) -> K,
{
    let mut positions = HashMap::new();
    let mut deduped = Vec::with_capacity(records.len());
    for record in records.drain(..) {
        let key = key_fn(&record);
        if let Some(existing_index) = positions.get(&key).copied() {
            deduped[existing_index] = record;
        } else {
            positions.insert(key, deduped.len());
            deduped.push(record);
        }
    }
    *records = deduped;
}

fn load_json_array<T>(
    data_root: &Path,
    relative_paths: &[String],
    section_name: &str,
) -> Result<Vec<T>>
where
    T: DeserializeOwned,
{
    let mut records = Vec::new();
    for relative_path in relative_paths {
        let path = resolve_data_file_path(data_root, relative_path)?;
        let mut batch: Vec<T> = read_json_file(&path).with_context(|| {
            format!(
                "failed to parse bootstrap {} file {}",
                section_name,
                path.display()
            )
        })?;
        records.append(&mut batch);
    }
    Ok(records)
}

fn load_routing_bundle(data_root: &Path, relative_paths: &[String]) -> Result<RoutingBundle> {
    let mut merged = RoutingBundle::default();
    for relative_path in relative_paths {
        let path = resolve_data_file_path(data_root, relative_path)?;
        let bundle: RoutingBundle = read_json_file(&path).with_context(|| {
            format!("failed to parse bootstrap routing file {}", path.display())
        })?;
        merged.profiles.extend(bundle.profiles);
        merged.policies.extend(bundle.policies);
        merged
            .project_preferences
            .extend(bundle.project_preferences);
    }
    Ok(merged)
}

fn load_extensions_bundle(data_root: &Path, relative_paths: &[String]) -> Result<ExtensionsBundle> {
    let mut merged = ExtensionsBundle::default();
    for relative_path in relative_paths {
        let path = resolve_data_file_path(data_root, relative_path)?;
        let bundle: ExtensionsBundle = read_json_file(&path).with_context(|| {
            format!(
                "failed to parse bootstrap extensions file {}",
                path.display()
            )
        })?;
        merged.installations.extend(bundle.installations);
        merged.instances.extend(bundle.instances);
    }
    Ok(merged)
}

fn load_identities_bundle(data_root: &Path, relative_paths: &[String]) -> Result<IdentitiesBundle> {
    let mut merged = IdentitiesBundle::default();
    for relative_path in relative_paths {
        let path = resolve_data_file_path(data_root, relative_path)?;
        let bundle: IdentitiesBundle = read_json_file(&path).with_context(|| {
            format!(
                "failed to parse bootstrap identities file {}",
                path.display()
            )
        })?;
        merged.admin_users.extend(bundle.admin_users);
        merged.portal_users.extend(bundle.portal_users);
        merged.gateway_api_keys.extend(bundle.gateway_api_keys);
    }
    Ok(merged)
}

fn load_extension_runtime_rollouts_bundle(
    data_root: &Path,
    relative_paths: &[String],
) -> Result<ExtensionRuntimeRolloutsBundle> {
    let mut merged = ExtensionRuntimeRolloutsBundle::default();
    for relative_path in relative_paths {
        let path = resolve_data_file_path(data_root, relative_path)?;
        let bundle: ExtensionRuntimeRolloutsBundle = read_json_file(&path).with_context(|| {
            format!(
                "failed to parse bootstrap extension runtime rollouts file {}",
                path.display()
            )
        })?;
        merged.rollouts.extend(bundle.rollouts);
        merged.participants.extend(bundle.participants);
    }
    Ok(merged)
}

fn load_standalone_config_rollouts_bundle(
    data_root: &Path,
    relative_paths: &[String],
) -> Result<StandaloneConfigRolloutsBundle> {
    let mut merged = StandaloneConfigRolloutsBundle::default();
    for relative_path in relative_paths {
        let path = resolve_data_file_path(data_root, relative_path)?;
        let bundle: StandaloneConfigRolloutsBundle = read_json_file(&path).with_context(|| {
            format!(
                "failed to parse bootstrap standalone config rollouts file {}",
                path.display()
            )
        })?;
        merged.rollouts.extend(bundle.rollouts);
        merged.participants.extend(bundle.participants);
    }
    Ok(merged)
}

fn load_observability_bundle(
    data_root: &Path,
    relative_paths: &[String],
) -> Result<ObservabilityBundle> {
    let mut merged = ObservabilityBundle::default();
    for relative_path in relative_paths {
        let path = resolve_data_file_path(data_root, relative_path)?;
        let bundle: ObservabilityBundle = read_json_file(&path).with_context(|| {
            format!(
                "failed to parse bootstrap observability file {}",
                path.display()
            )
        })?;
        merged
            .compiled_routing_snapshots
            .extend(bundle.compiled_routing_snapshots);
        merged
            .routing_decision_logs
            .extend(bundle.routing_decision_logs);
        merged
            .provider_health_snapshots
            .extend(bundle.provider_health_snapshots);
    }
    Ok(merged)
}

fn load_billing_bundle(data_root: &Path, relative_paths: &[String]) -> Result<BillingBundle> {
    let mut merged = BillingBundle::default();
    for relative_path in relative_paths {
        let path = resolve_data_file_path(data_root, relative_path)?;
        let bundle: BillingBundle = read_json_file(&path).with_context(|| {
            format!("failed to parse bootstrap billing file {}", path.display())
        })?;
        merged.billing_events.extend(bundle.billing_events);
    }
    Ok(merged)
}

fn load_quota_policies_bundle(
    data_root: &Path,
    relative_paths: &[String],
) -> Result<QuotaPoliciesBundle> {
    let mut merged = QuotaPoliciesBundle::default();
    for relative_path in relative_paths {
        let path = resolve_data_file_path(data_root, relative_path)?;
        let bundle: QuotaPoliciesBundle = read_json_file(&path).with_context(|| {
            format!(
                "failed to parse bootstrap quota_policies file {}",
                path.display()
            )
        })?;
        merged.quota_policies.extend(bundle.quota_policies);
        merged
            .rate_limit_policies
            .extend(bundle.rate_limit_policies);
    }
    Ok(merged)
}

fn load_pricing_bundle(data_root: &Path, relative_paths: &[String]) -> Result<PricingBundle> {
    let mut merged = PricingBundle::default();
    for relative_path in relative_paths {
        let path = resolve_data_file_path(data_root, relative_path)?;
        let bundle: PricingBundle = read_json_file(&path).with_context(|| {
            format!("failed to parse bootstrap pricing file {}", path.display())
        })?;
        merged.plans.extend(bundle.plans);
        merged.rates.extend(bundle.rates);
    }
    Ok(merged)
}

fn load_account_holds_bundle(
    data_root: &Path,
    relative_paths: &[String],
) -> Result<AccountHoldsBundle> {
    let mut merged = AccountHoldsBundle::default();
    for relative_path in relative_paths {
        let path = resolve_data_file_path(data_root, relative_path)?;
        let bundle: AccountHoldsBundle = read_json_file(&path).with_context(|| {
            format!(
                "failed to parse bootstrap account holds file {}",
                path.display()
            )
        })?;
        merged.holds.extend(bundle.holds);
        merged.allocations.extend(bundle.allocations);
    }
    Ok(merged)
}

fn load_account_ledger_bundle(
    data_root: &Path,
    relative_paths: &[String],
) -> Result<AccountLedgerBundle> {
    let mut merged = AccountLedgerBundle::default();
    for relative_path in relative_paths {
        let path = resolve_data_file_path(data_root, relative_path)?;
        let bundle: AccountLedgerBundle = read_json_file(&path).with_context(|| {
            format!(
                "failed to parse bootstrap account ledger file {}",
                path.display()
            )
        })?;
        merged.entries.extend(bundle.entries);
        merged.allocations.extend(bundle.allocations);
    }
    Ok(merged)
}

fn load_request_metering_bundle(
    data_root: &Path,
    relative_paths: &[String],
) -> Result<RequestMeteringBundle> {
    let mut merged = RequestMeteringBundle::default();
    for relative_path in relative_paths {
        let path = resolve_data_file_path(data_root, relative_path)?;
        let bundle: RequestMeteringBundle = read_json_file(&path).with_context(|| {
            format!(
                "failed to parse bootstrap request metering file {}",
                path.display()
            )
        })?;
        merged.facts.extend(bundle.facts);
        merged.metrics.extend(bundle.metrics);
    }
    Ok(merged)
}

fn load_payment_methods_bundle(
    data_root: &Path,
    relative_paths: &[String],
) -> Result<PaymentMethodsBundle> {
    let mut merged = PaymentMethodsBundle::default();
    for relative_path in relative_paths {
        let path = resolve_data_file_path(data_root, relative_path)?;
        let bundle: PaymentMethodsBundle = read_json_file(&path).with_context(|| {
            format!(
                "failed to parse bootstrap payment_methods file {}",
                path.display()
            )
        })?;
        merged.payment_methods.extend(bundle.payment_methods);
        merged
            .credential_bindings
            .extend(bundle.credential_bindings);
        merged
            .project_memberships
            .extend(bundle.project_memberships);
    }
    Ok(merged)
}

fn load_commerce_bundle(data_root: &Path, relative_paths: &[String]) -> Result<CommerceBundle> {
    let mut merged = CommerceBundle::default();
    for relative_path in relative_paths {
        let path = resolve_data_file_path(data_root, relative_path)?;
        let bundle: CommerceBundle = read_json_file(&path).with_context(|| {
            format!("failed to parse bootstrap commerce file {}", path.display())
        })?;
        merged.orders.extend(bundle.orders);
        merged.payment_attempts.extend(bundle.payment_attempts);
        merged.payment_events.extend(bundle.payment_events);
        merged
            .webhook_inbox_records
            .extend(bundle.webhook_inbox_records);
        merged.refunds.extend(bundle.refunds);
        merged
            .reconciliation_runs
            .extend(bundle.reconciliation_runs);
        merged
            .reconciliation_items
            .extend(bundle.reconciliation_items);
    }
    Ok(merged)
}

fn load_jobs_bundle(data_root: &Path, relative_paths: &[String]) -> Result<JobsBundle> {
    let mut merged = JobsBundle::default();
    for relative_path in relative_paths {
        let path = resolve_data_file_path(data_root, relative_path)?;
        let bundle: JobsBundle = read_json_file(&path)
            .with_context(|| format!("failed to parse bootstrap jobs file {}", path.display()))?;
        merged.jobs.extend(bundle.jobs);
        merged.attempts.extend(bundle.attempts);
        merged.assets.extend(bundle.assets);
        merged.callbacks.extend(bundle.callbacks);
    }
    Ok(merged)
}

fn load_marketing_bundle(data_root: &Path, relative_paths: &[String]) -> Result<MarketingBundle> {
    let mut merged = MarketingBundle::default();
    for relative_path in relative_paths {
        let path = resolve_data_file_path(data_root, relative_path)?;
        let bundle: MarketingBundle = read_json_file(&path).with_context(|| {
            format!(
                "failed to parse bootstrap marketing file {}",
                path.display()
            )
        })?;
        merged.coupon_templates.extend(bundle.coupon_templates);
        merged
            .marketing_campaigns
            .extend(bundle.marketing_campaigns);
        merged.campaign_budgets.extend(bundle.campaign_budgets);
        merged.coupon_codes.extend(bundle.coupon_codes);
    }
    Ok(merged)
}

fn read_json_file<T>(path: &Path) -> Result<T>
where
    T: DeserializeOwned,
{
    let bytes = fs::read(path)
        .with_context(|| format!("failed to read bootstrap data file {}", path.display()))?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("failed to decode json from {}", path.display()))
}

fn resolve_data_file_path(data_root: &Path, relative_path: &str) -> Result<PathBuf> {
    let relative_path = relative_path.trim();
    if relative_path.is_empty() {
        bail!("bootstrap data file path must not be empty");
    }

    let relative = Path::new(relative_path);
    if relative.is_absolute() {
        bail!(
            "bootstrap data file path {} must be relative to {}",
            relative_path,
            data_root.display()
        );
    }

    let joined = data_root.join(relative);
    let canonical = fs::canonicalize(&joined)
        .with_context(|| format!("bootstrap data file {} does not exist", joined.display()))?;

    if !canonical.starts_with(data_root) {
        bail!(
            "bootstrap data file {} escapes configured data root {}",
            canonical.display(),
            data_root.display()
        );
    }

    Ok(canonical)
}

fn validate_bootstrap_profile_pack(pack: &BootstrapProfilePack) -> Result<()> {
    let data = &pack.data;

    ensure_unique("channels.id", &data.channels, |record| record.id.clone())?;
    ensure_unique("providers.id", &data.providers, |record| record.id.clone())?;
    ensure_unique(
        "official_provider_configs.provider_id",
        &data.official_provider_configs,
        |record| record.provider_id.clone(),
    )?;
    ensure_unique(
        "provider_accounts.provider_account_id",
        &data.provider_accounts,
        |record| record.provider_account_id.clone(),
    )?;
    ensure_unique("models.external_name+provider_id", &data.models, |record| {
        format!("{}::{}", record.external_name, record.provider_id)
    })?;
    ensure_unique(
        "channel_models.channel_id+model_id",
        &data.channel_models,
        |record| format!("{}::{}", record.channel_id, record.model_id),
    )?;
    ensure_unique(
        "provider_models.proxy_provider_id+channel_id+model_id",
        &data.provider_models,
        |record| {
            format!(
                "{}::{}::{}",
                record.proxy_provider_id, record.channel_id, record.model_id
            )
        },
    )?;
    ensure_unique(
        "model_prices.channel_id+model_id+proxy_provider_id",
        &data.model_prices,
        |record| {
            format!(
                "{}::{}::{}",
                record.channel_id, record.model_id, record.proxy_provider_id
            )
        },
    )?;
    ensure_unique("tenants.id", &data.tenants, |record| record.id.clone())?;
    ensure_unique("projects.id", &data.projects, |record| record.id.clone())?;
    ensure_unique("admin_users.id", &data.admin_users, |record| {
        record.id.clone()
    })?;
    ensure_unique("admin_users.email", &data.admin_users, |record| {
        normalize_identity_email(&record.email)
    })?;
    ensure_unique("portal_users.id", &data.portal_users, |record| {
        record.id.clone()
    })?;
    ensure_unique("portal_users.email", &data.portal_users, |record| {
        normalize_identity_email(&record.email)
    })?;
    ensure_unique(
        "gateway_api_keys.hashed_key",
        &data.gateway_api_keys,
        |record| record.hashed_key.clone(),
    )?;
    ensure_unique(
        "extension_installations.installation_id",
        &data.extension_installations,
        |record| record.installation_id.clone(),
    )?;
    ensure_unique(
        "extension_instances.instance_id",
        &data.extension_instances,
        |record| record.instance_id.clone(),
    )?;
    ensure_unique(
        "service_runtime_nodes.node_id",
        &data.service_runtime_nodes,
        |record| record.node_id.clone(),
    )?;
    ensure_unique(
        "extension_runtime_rollouts.rollout_id",
        &data.extension_runtime_rollouts,
        |record| record.rollout_id.clone(),
    )?;
    ensure_unique(
        "extension_runtime_rollout_participants.rollout_id+node_id",
        &data.extension_runtime_rollout_participants,
        |record| format!("{}::{}", record.rollout_id, record.node_id),
    )?;
    ensure_unique(
        "standalone_config_rollouts.rollout_id",
        &data.standalone_config_rollouts,
        |record| record.rollout_id.clone(),
    )?;
    ensure_unique(
        "standalone_config_rollout_participants.rollout_id+node_id",
        &data.standalone_config_rollout_participants,
        |record| format!("{}::{}", record.rollout_id, record.node_id),
    )?;
    ensure_unique(
        "routing_profiles.profile_id",
        &data.routing_profiles,
        |record| record.profile_id.clone(),
    )?;
    ensure_unique(
        "routing_policies.policy_id",
        &data.routing_policies,
        |record| record.policy_id.clone(),
    )?;
    ensure_unique(
        "project_preferences.project_id",
        &data.project_preferences,
        |record| record.project_id.clone(),
    )?;
    ensure_unique("api_key_groups.group_id", &data.api_key_groups, |record| {
        record.group_id.clone()
    })?;
    ensure_unique(
        "compiled_routing_snapshots.snapshot_id",
        &data.compiled_routing_snapshots,
        |record| record.snapshot_id.clone(),
    )?;
    ensure_unique(
        "routing_decision_logs.decision_id",
        &data.routing_decision_logs,
        |record| record.decision_id.clone(),
    )?;
    ensure_unique(
        "provider_health_snapshots.provider_id+runtime+instance_id",
        &data.provider_health_snapshots,
        |record| provider_health_snapshot_key(record),
    )?;
    ensure_unique("billing_events.event_id", &data.billing_events, |record| {
        record.event_id.clone()
    })?;
    ensure_unique("quota_policies.policy_id", &data.quota_policies, |record| {
        record.policy_id.clone()
    })?;
    ensure_unique(
        "rate_limit_policies.policy_id",
        &data.rate_limit_policies,
        |record| record.policy_id.clone(),
    )?;
    ensure_unique(
        "pricing_plans.pricing_plan_id",
        &data.pricing_plans,
        |record| record.pricing_plan_id.to_string(),
    )?;
    ensure_unique(
        "pricing_plans.plan_code+plan_version",
        &data.pricing_plans,
        |record| format!("{}::{}", record.plan_code, record.plan_version),
    )?;
    ensure_unique(
        "pricing_rates.pricing_rate_id",
        &data.pricing_rates,
        |record| record.pricing_rate_id.to_string(),
    )?;
    ensure_unique("accounts.account_id", &data.accounts, |record| {
        record.account_id.to_string()
    })?;
    ensure_unique(
        "accounts.tenant_id+organization_id+user_id+account_type",
        &data.accounts,
        |record| {
            format!(
                "{}::{}::{}::{:?}",
                record.tenant_id, record.organization_id, record.user_id, record.account_type
            )
        },
    )?;
    ensure_unique(
        "account_benefit_lots.lot_id",
        &data.account_benefit_lots,
        |record| record.lot_id.to_string(),
    )?;
    ensure_unique("account_holds.hold_id", &data.account_holds, |record| {
        record.hold_id.to_string()
    })?;
    ensure_unique(
        "account_hold_allocations.hold_allocation_id",
        &data.account_hold_allocations,
        |record| record.hold_allocation_id.to_string(),
    )?;
    ensure_unique(
        "account_ledger_entries.ledger_entry_id",
        &data.account_ledger_entries,
        |record| record.ledger_entry_id.to_string(),
    )?;
    ensure_unique(
        "account_ledger_allocations.ledger_allocation_id",
        &data.account_ledger_allocations,
        |record| record.ledger_allocation_id.to_string(),
    )?;
    ensure_unique(
        "request_meter_facts.request_id",
        &data.request_meter_facts,
        |record| record.request_id.to_string(),
    )?;
    ensure_unique(
        "request_meter_metrics.request_metric_id",
        &data.request_meter_metrics,
        |record| record.request_metric_id.to_string(),
    )?;
    ensure_unique(
        "request_settlements.request_settlement_id",
        &data.request_settlements,
        |record| record.request_settlement_id.to_string(),
    )?;
    ensure_unique(
        "account_reconciliation.account_id+project_id",
        &data.account_commerce_reconciliation_states,
        |record| format!("{}::{}", record.account_id, record.project_id),
    )?;
    ensure_unique(
        "payment_methods.payment_method_id",
        &data.payment_methods,
        |record| record.payment_method_id.clone(),
    )?;
    ensure_unique(
        "payment_method_credential_bindings.binding_id",
        &data.payment_method_credential_bindings,
        |record| record.binding_id.clone(),
    )?;
    ensure_unique(
        "payment_method_credential_bindings.payment_method_id+usage_kind",
        &data.payment_method_credential_bindings,
        |record| format!("{}::{}", record.payment_method_id, record.usage_kind),
    )?;
    ensure_unique(
        "project_memberships.project_id",
        &data.project_memberships,
        |record| record.project_id.clone(),
    )?;
    ensure_unique(
        "project_memberships.membership_id",
        &data.project_memberships,
        |record| record.membership_id.clone(),
    )?;
    ensure_unique(
        "commerce_orders.order_id",
        &data.commerce_orders,
        |record| record.order_id.clone(),
    )?;
    ensure_unique(
        "commerce_payment_attempts.payment_attempt_id",
        &data.commerce_payment_attempts,
        |record| record.payment_attempt_id.clone(),
    )?;
    ensure_unique(
        "commerce_payment_attempts.idempotency_key",
        &data.commerce_payment_attempts,
        |record| record.idempotency_key.clone(),
    )?;
    ensure_unique(
        "commerce_payment_events.payment_event_id",
        &data.commerce_payment_events,
        |record| record.payment_event_id.clone(),
    )?;
    ensure_unique(
        "commerce_payment_events.dedupe_key",
        &data.commerce_payment_events,
        |record| record.dedupe_key.clone(),
    )?;
    ensure_unique(
        "commerce_webhook_inbox_records.webhook_inbox_id",
        &data.commerce_webhook_inbox_records,
        |record| record.webhook_inbox_id.clone(),
    )?;
    ensure_unique(
        "commerce_webhook_inbox_records.dedupe_key",
        &data.commerce_webhook_inbox_records,
        |record| record.dedupe_key.clone(),
    )?;
    ensure_unique(
        "commerce_refunds.refund_id",
        &data.commerce_refunds,
        |record| record.refund_id.clone(),
    )?;
    ensure_unique(
        "commerce_refunds.idempotency_key",
        &data.commerce_refunds,
        |record| record.idempotency_key.clone(),
    )?;
    ensure_unique(
        "commerce_reconciliation_runs.reconciliation_run_id",
        &data.commerce_reconciliation_runs,
        |record| record.reconciliation_run_id.clone(),
    )?;
    ensure_unique(
        "commerce_reconciliation_items.reconciliation_item_id",
        &data.commerce_reconciliation_items,
        |record| record.reconciliation_item_id.clone(),
    )?;
    ensure_unique("async_jobs.job_id", &data.async_jobs, |record| {
        record.job_id.clone()
    })?;
    ensure_unique(
        "async_job_attempts.attempt_id",
        &data.async_job_attempts,
        |record| record.attempt_id.to_string(),
    )?;
    ensure_unique(
        "async_job_attempts.job_id+attempt_number",
        &data.async_job_attempts,
        |record| format!("{}::{}", record.job_id, record.attempt_number),
    )?;
    ensure_unique(
        "async_job_assets.asset_id",
        &data.async_job_assets,
        |record| record.asset_id.clone(),
    )?;
    ensure_unique(
        "async_job_callbacks.callback_id",
        &data.async_job_callbacks,
        |record| record.callback_id.to_string(),
    )?;
    ensure_unique(
        "coupon_templates.coupon_template_id",
        &data.coupon_templates,
        |record| record.coupon_template_id.clone(),
    )?;
    ensure_unique(
        "coupon_templates.template_key",
        &data.coupon_templates,
        |record| record.template_key.clone(),
    )?;
    ensure_unique(
        "marketing_campaigns.marketing_campaign_id",
        &data.marketing_campaigns,
        |record| record.marketing_campaign_id.clone(),
    )?;
    ensure_unique(
        "campaign_budgets.campaign_budget_id",
        &data.campaign_budgets,
        |record| record.campaign_budget_id.clone(),
    )?;
    ensure_unique(
        "coupon_codes.coupon_code_id",
        &data.coupon_codes,
        |record| record.coupon_code_id.clone(),
    )?;
    ensure_unique("coupon_codes.code_value", &data.coupon_codes, |record| {
        normalize_coupon_code(&record.code_value)
    })?;

    let channel_ids = collect_ids(data.channels.iter().map(|record| record.id.as_str()));
    let provider_ids = collect_ids(data.providers.iter().map(|record| record.id.as_str()));
    let tenant_ids = collect_ids(data.tenants.iter().map(|record| record.id.as_str()));
    let project_ids = collect_ids(data.projects.iter().map(|record| record.id.as_str()));
    let api_key_group_ids = collect_ids(
        data.api_key_groups
            .iter()
            .map(|record| record.group_id.as_str()),
    );
    let extension_installation_ids = collect_ids(
        data.extension_installations
            .iter()
            .map(|record| record.installation_id.as_str()),
    );
    let extension_instance_ids = collect_ids(
        data.extension_instances
            .iter()
            .map(|record| record.instance_id.as_str()),
    );
    let extension_ids = collect_ids(
        data.extension_installations
            .iter()
            .map(|record| record.extension_id.as_str())
            .chain(
                data.extension_instances
                    .iter()
                    .map(|record| record.extension_id.as_str()),
            ),
    );
    let service_runtime_node_ids = collect_ids(
        data.service_runtime_nodes
            .iter()
            .map(|record| record.node_id.as_str()),
    );
    let extension_runtime_rollout_ids = collect_ids(
        data.extension_runtime_rollouts
            .iter()
            .map(|record| record.rollout_id.as_str()),
    );
    let standalone_config_rollout_ids = collect_ids(
        data.standalone_config_rollouts
            .iter()
            .map(|record| record.rollout_id.as_str()),
    );
    let routing_profile_ids = collect_ids(
        data.routing_profiles
            .iter()
            .map(|record| record.profile_id.as_str()),
    );
    let routing_policy_ids = collect_ids(
        data.routing_policies
            .iter()
            .map(|record| record.policy_id.as_str()),
    );
    let compiled_routing_snapshot_ids = collect_ids(
        data.compiled_routing_snapshots
            .iter()
            .map(|record| record.snapshot_id.as_str()),
    );
    let gateway_api_key_hashes = collect_ids(
        data.gateway_api_keys
            .iter()
            .map(|record| record.hashed_key.as_str()),
    );
    let pricing_plan_ids = collect_ids(
        data.pricing_plans
            .iter()
            .map(|record| record.pricing_plan_id.to_string()),
    );
    let active_pricing_plan_ids = collect_ids(
        data.pricing_plans
            .iter()
            .filter(|record| record.status.trim() == "active")
            .map(|record| record.pricing_plan_id.to_string()),
    );
    let pricing_plans_by_id = data
        .pricing_plans
        .iter()
        .map(|record| (record.pricing_plan_id.to_string(), record))
        .collect::<HashMap<_, _>>();
    let account_ids = collect_ids(
        data.accounts
            .iter()
            .map(|record| record.account_id.to_string()),
    );
    let accounts_by_id = data
        .accounts
        .iter()
        .map(|record| (record.account_id, record))
        .collect::<HashMap<_, _>>();
    let payment_method_ids = collect_ids(
        data.payment_methods
            .iter()
            .map(|record| record.payment_method_id.as_str()),
    );
    let coupon_template_ids = collect_ids(
        data.coupon_templates
            .iter()
            .map(|record| record.coupon_template_id.as_str()),
    );
    let commerce_order_ids = collect_ids(
        data.commerce_orders
            .iter()
            .map(|record| record.order_id.as_str()),
    );
    let commerce_payment_attempt_ids = collect_ids(
        data.commerce_payment_attempts
            .iter()
            .map(|record| record.payment_attempt_id.as_str()),
    );
    let commerce_refund_ids = collect_ids(
        data.commerce_refunds
            .iter()
            .map(|record| record.refund_id.as_str()),
    );
    let commerce_reconciliation_run_ids = collect_ids(
        data.commerce_reconciliation_runs
            .iter()
            .map(|record| record.reconciliation_run_id.as_str()),
    );
    let async_job_ids = collect_ids(data.async_jobs.iter().map(|record| record.job_id.as_str()));
    let marketing_campaign_ids = collect_ids(
        data.marketing_campaigns
            .iter()
            .map(|record| record.marketing_campaign_id.as_str()),
    );
    let model_ids = collect_ids(
        data.models
            .iter()
            .map(|record| record.external_name.as_str())
            .chain(
                data.provider_models
                    .iter()
                    .map(|record| record.model_id.as_str()),
            ),
    );
    let model_variant_keys = collect_ids(
        data.models
            .iter()
            .map(|record| format!("{}::{}", record.external_name, record.provider_id)),
    );
    let provider_model_keys = collect_ids(data.provider_models.iter().map(|record| {
        format!(
            "{}::{}::{}",
            record.proxy_provider_id, record.channel_id, record.model_id
        )
    }));
    let channel_model_keys = collect_ids(
        data.channel_models
            .iter()
            .map(|record| format!("{}::{}", record.channel_id, record.model_id)),
    );
    let active_model_price_keys = collect_ids(
        data.model_prices
            .iter()
            .filter(|record| record.is_active)
            .map(|record| {
                format!(
                    "{}::{}::{}",
                    record.proxy_provider_id, record.channel_id, record.model_id
                )
            }),
    );
    let project_tenants = data
        .projects
        .iter()
        .map(|record| (record.id.as_str(), record.tenant_id.as_str()))
        .collect::<HashMap<_, _>>();
    let api_key_groups = data
        .api_key_groups
        .iter()
        .map(|record| (record.group_id.as_str(), record))
        .collect::<HashMap<_, _>>();
    let gateway_api_keys = data
        .gateway_api_keys
        .iter()
        .map(|record| (record.hashed_key.as_str(), record))
        .collect::<HashMap<_, _>>();
    let extension_installations = data
        .extension_installations
        .iter()
        .map(|record| (record.installation_id.as_str(), record))
        .collect::<HashMap<_, _>>();
    let extension_instances = data
        .extension_instances
        .iter()
        .map(|record| (record.instance_id.as_str(), record))
        .collect::<HashMap<_, _>>();
    let service_runtime_nodes = data
        .service_runtime_nodes
        .iter()
        .map(|record| (record.node_id.as_str(), record))
        .collect::<HashMap<_, _>>();
    let extension_runtime_rollouts = data
        .extension_runtime_rollouts
        .iter()
        .map(|record| (record.rollout_id.as_str(), record))
        .collect::<HashMap<_, _>>();
    let standalone_config_rollouts = data
        .standalone_config_rollouts
        .iter()
        .map(|record| (record.rollout_id.as_str(), record))
        .collect::<HashMap<_, _>>();
    let routing_profiles = data
        .routing_profiles
        .iter()
        .map(|record| (record.profile_id.as_str(), record))
        .collect::<HashMap<_, _>>();
    let routing_policies = data
        .routing_policies
        .iter()
        .map(|record| (record.policy_id.as_str(), record))
        .collect::<HashMap<_, _>>();
    let compiled_routing_snapshots = data
        .compiled_routing_snapshots
        .iter()
        .map(|record| (record.snapshot_id.as_str(), record))
        .collect::<HashMap<_, _>>();
    let commerce_orders = data
        .commerce_orders
        .iter()
        .map(|record| (record.order_id.as_str(), record))
        .collect::<HashMap<_, _>>();
    let commerce_payment_attempts = data
        .commerce_payment_attempts
        .iter()
        .map(|record| (record.payment_attempt_id.as_str(), record))
        .collect::<HashMap<_, _>>();
    let commerce_refunds = data
        .commerce_refunds
        .iter()
        .map(|record| (record.refund_id.as_str(), record))
        .collect::<HashMap<_, _>>();
    let commerce_reconciliation_runs = data
        .commerce_reconciliation_runs
        .iter()
        .map(|record| (record.reconciliation_run_id.as_str(), record))
        .collect::<HashMap<_, _>>();
    let payment_methods = data
        .payment_methods
        .iter()
        .map(|record| (record.payment_method_id.as_str(), record))
        .collect::<HashMap<_, _>>();
    let marketing_campaigns = data
        .marketing_campaigns
        .iter()
        .map(|record| (record.marketing_campaign_id.as_str(), record))
        .collect::<HashMap<_, _>>();
    let coupon_codes_by_value = data
        .coupon_codes
        .iter()
        .map(|record| (normalize_coupon_code(&record.code_value), record))
        .collect::<HashMap<_, _>>();
    let async_jobs = data
        .async_jobs
        .iter()
        .map(|record| (record.job_id.as_str(), record))
        .collect::<HashMap<_, _>>();
    let providers = data
        .providers
        .iter()
        .map(|record| (record.id.as_str(), record))
        .collect::<HashMap<_, _>>();
    let allowed_workspace_user_ids = collect_ids(
        data.portal_users
            .iter()
            .map(|record| record.id.as_str())
            .chain(
                data.project_memberships
                    .iter()
                    .map(|record| record.user_id.as_str()),
            ),
    );
    let available_coupon_codes = collect_ids(
        data.coupon_codes
            .iter()
            .map(|record| normalize_coupon_code(&record.code_value)),
    );
    let provider_channels = build_provider_channels(&data.providers);
    let executable_provider_account_provider_ids = collect_ids(
        data.provider_accounts
            .iter()
            .filter(|record| record.enabled)
            .filter(|record| {
                extension_instances
                    .get(record.execution_instance_id.as_str())
                    .is_some_and(|instance| {
                        instance.enabled
                            && extension_installations
                                .get(instance.installation_id.as_str())
                                .is_some_and(|installation| installation.enabled)
                    })
            })
            .map(|record| record.provider_id.as_str()),
    );
    let executable_provider_account_instance_ids = collect_ids(
        data.provider_accounts
            .iter()
            .filter(|record| record.enabled)
            .filter(|record| {
                extension_instances
                    .get(record.execution_instance_id.as_str())
                    .is_some_and(|instance| {
                        instance.enabled
                            && extension_installations
                                .get(instance.installation_id.as_str())
                                .is_some_and(|installation| installation.enabled)
                    })
            })
            .map(|record| record.execution_instance_id.as_str()),
    );
    let executable_provider_account_instance_bindings = collect_ids(
        data.provider_accounts
            .iter()
            .filter(|record| record.enabled)
            .filter(|record| {
                extension_instances
                    .get(record.execution_instance_id.as_str())
                    .is_some_and(|instance| {
                        instance.enabled
                            && extension_installations
                                .get(instance.installation_id.as_str())
                                .is_some_and(|installation| installation.enabled)
                    })
            })
            .map(|record| format!("{}::{}", record.provider_id, record.execution_instance_id)),
    );
    let tenant_accessible_executable_provider_ids = build_tenant_accessible_executable_provider_ids(
        &data.provider_accounts,
        &extension_instances,
        &extension_installations,
    );
    let available_channel_model_variants = build_available_channel_model_variants(
        &data.models,
        &data.provider_models,
        &provider_channels,
    );
    let mut async_job_idempotency_keys = HashSet::new();
    let mut async_job_callback_dedupe_keys = HashSet::new();

    validate_account_kernel_bootstrap_data(
        data,
        &account_ids,
        &channel_ids,
        &provider_ids,
        &project_ids,
        &gateway_api_key_hashes,
        &pricing_plan_ids,
        &active_pricing_plan_ids,
        &pricing_plans_by_id,
        &available_channel_model_variants,
        &provider_channels,
        &commerce_order_ids,
        &executable_provider_account_provider_ids,
    )?;

    for provider in &data.providers {
        ensure_reference(
            "providers.channel_id",
            &provider.id,
            &provider.channel_id,
            &channel_ids,
        )?;
        ensure_unique(
            &format!("providers.channel_bindings.channel_id for {}", provider.id),
            &provider.channel_bindings,
            |record| record.channel_id.clone(),
        )?;
        for binding in &provider.channel_bindings {
            if binding.provider_id != provider.id {
                bail!(
                    "provider {} contains channel binding for mismatched provider {}",
                    provider.id,
                    binding.provider_id
                );
            }
            ensure_reference(
                "providers.channel_bindings.channel_id",
                &provider.id,
                &binding.channel_id,
                &channel_ids,
            )?;
        }
    }

    for record in &data.official_provider_configs {
        ensure_reference(
            "official_provider_configs.provider_id",
            &record.provider_id,
            &record.provider_id,
            &provider_ids,
        )?;
    }

    for record in &data.provider_accounts {
        ensure_non_empty_field(
            "provider_accounts.display_name",
            &record.provider_account_id,
            &record.display_name,
        )?;
        ensure_non_empty_field(
            "provider_accounts.account_kind",
            &record.provider_account_id,
            &record.account_kind,
        )?;
        ensure_non_empty_field(
            "provider_accounts.owner_scope",
            &record.provider_account_id,
            &record.owner_scope,
        )?;
        ensure_reference(
            "provider_accounts.provider_id",
            &record.provider_account_id,
            &record.provider_id,
            &provider_ids,
        )?;
        ensure_reference(
            "provider_accounts.execution_instance_id",
            &record.provider_account_id,
            &record.execution_instance_id,
            &extension_instance_ids,
        )?;
        if let Some(owner_tenant_id) = record.owner_tenant_id.as_deref() {
            ensure_reference(
                "provider_accounts.owner_tenant_id",
                &record.provider_account_id,
                owner_tenant_id,
                &tenant_ids,
            )?;
        }
        if record.owner_scope == "tenant" && record.owner_tenant_id.is_none() {
            bail!(
                "provider account {} uses tenant owner_scope but does not declare owner_tenant_id",
                record.provider_account_id
            );
        }
        if let Some(base_url_override) = record.base_url_override.as_deref() {
            ensure_non_empty_field(
                "provider_accounts.base_url_override",
                &record.provider_account_id,
                base_url_override,
            )?;
        }
        for tag in &record.routing_tags {
            ensure_non_empty_field(
                "provider_accounts.routing_tags",
                &record.provider_account_id,
                tag,
            )?;
        }

        let provider = providers.get(record.provider_id.as_str()).ok_or_else(|| {
            anyhow::anyhow!(
                "bootstrap provider_accounts.provider_id record {} references missing {}",
                record.provider_account_id,
                record.provider_id
            )
        })?;
        let instance = extension_instances
            .get(record.execution_instance_id.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "bootstrap provider_accounts.execution_instance_id record {} references missing {}",
                    record.provider_account_id,
                    record.execution_instance_id
                )
            })?;
        let installation = extension_installations
            .get(instance.installation_id.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "bootstrap provider account {} binds execution instance {} whose installation {} is missing",
                    record.provider_account_id,
                    record.execution_instance_id,
                    instance.installation_id
                )
            })?;
        if provider.extension_id != instance.extension_id {
            bail!(
                "provider account {} binds provider {} to extension instance {} with mismatched extension {}",
                record.provider_account_id,
                record.provider_id,
                record.execution_instance_id,
                instance.extension_id
            );
        }
        if record.enabled && !instance.enabled {
            bail!(
                "provider account {} is enabled but execution instance {} is disabled",
                record.provider_account_id,
                record.execution_instance_id
            );
        }
        if record.enabled && !installation.enabled {
            bail!(
                "provider account {} is enabled but installation {} is disabled",
                record.provider_account_id,
                installation.installation_id
            );
        }
    }

    for record in &data.models {
        ensure_reference(
            "models.provider_id",
            &record.external_name,
            &record.provider_id,
            &provider_ids,
        )?;
    }

    for record in &data.channel_models {
        ensure_reference(
            "channel_models.channel_id",
            &record.model_id,
            &record.channel_id,
            &channel_ids,
        )?;
        let key = format!("{}::{}", record.channel_id, record.model_id);
        if !available_channel_model_variants.contains(&key) {
            bail!(
                "channel model {} on channel {} does not have any compatible model variant",
                record.model_id,
                record.channel_id
            );
        }
    }

    for record in &data.provider_models {
        let provider_model_record_id = format!(
            "{}::{}::{}",
            record.proxy_provider_id, record.channel_id, record.model_id
        );
        ensure_non_empty_field(
            "provider_models.provider_model_id",
            &record.model_id,
            &record.provider_model_id,
        )?;
        ensure_reference(
            "provider_models.proxy_provider_id",
            &record.model_id,
            &record.proxy_provider_id,
            &provider_ids,
        )?;
        ensure_reference(
            "provider_models.channel_id",
            &record.model_id,
            &record.channel_id,
            &channel_ids,
        )?;
        ensure_reference(
            "provider_models.channel_model",
            &record.model_id,
            &format!("{}::{}", record.channel_id, record.model_id),
            &channel_model_keys,
        )?;
        if !provider_channels
            .get(record.proxy_provider_id.as_str())
            .is_some_and(|channels| channels.contains(record.channel_id.as_str()))
        {
            bail!(
                "provider model {} for provider {} references channel {} without a provider channel binding",
                record.model_id,
                record.proxy_provider_id,
                record.channel_id
            );
        }
        if record.is_active {
            ensure_active_model_price_coverage(
                "provider_models.model_id",
                &provider_model_record_id,
                &record.proxy_provider_id,
                &record.channel_id,
                &record.model_id,
                &active_model_price_keys,
            )?;
        }
    }

    for record in &data.model_prices {
        let record_id = format!(
            "{}::{}::{}",
            record.proxy_provider_id, record.channel_id, record.model_id
        );
        ensure_non_empty_field(
            "model_prices.currency_code",
            &record_id,
            &record.currency_code,
        )?;
        ensure_non_empty_field("model_prices.price_unit", &record_id, &record.price_unit)?;
        ensure_non_empty_field(
            "model_prices.price_source_kind",
            &record_id,
            &record.price_source_kind,
        )?;
        ensure_supported_model_price_source_kind(&record_id, &record.price_source_kind)?;
        ensure_finite_non_negative_number(
            "model_prices.input_price",
            &record_id,
            record.input_price,
        )?;
        ensure_finite_non_negative_number(
            "model_prices.output_price",
            &record_id,
            record.output_price,
        )?;
        ensure_finite_non_negative_number(
            "model_prices.cache_read_price",
            &record_id,
            record.cache_read_price,
        )?;
        ensure_finite_non_negative_number(
            "model_prices.cache_write_price",
            &record_id,
            record.cache_write_price,
        )?;
        ensure_finite_non_negative_number(
            "model_prices.request_price",
            &record_id,
            record.request_price,
        )?;
        ensure_unique(
            &format!("model_prices.pricing_tiers.tier_id for {}", record_id),
            &record.pricing_tiers,
            |tier| tier.tier_id.clone(),
        )?;
        ensure_reference(
            "model_prices.channel_id",
            &record.model_id,
            &record.channel_id,
            &channel_ids,
        )?;
        ensure_reference(
            "model_prices.proxy_provider_id",
            &record.model_id,
            &record.proxy_provider_id,
            &provider_ids,
        )?;
        if record.is_active {
            ensure_provider_has_enabled_account(
                "model_prices.proxy_provider_id",
                &record_id,
                &record.proxy_provider_id,
                &executable_provider_account_provider_ids,
            )?;
        }
        ensure_reference(
            "model_prices.channel_model",
            &record.model_id,
            &format!("{}::{}", record.channel_id, record.model_id),
            &channel_model_keys,
        )?;
        if !provider_channels
            .get(record.proxy_provider_id.as_str())
            .is_some_and(|channels| channels.contains(record.channel_id.as_str()))
        {
            bail!(
                "model price {} for provider {} references channel {} without a provider channel binding",
                record.model_id,
                record.proxy_provider_id,
                record.channel_id
            );
        }
        ensure_reference(
            "model_prices.model_variant",
            &record.model_id,
            &format!("{}::{}", record.model_id, record.proxy_provider_id),
            &model_variant_keys,
        )
        .or_else(|_| {
            ensure_reference(
                "model_prices.provider_model",
                &record.model_id,
                &format!(
                    "{}::{}::{}",
                    record.proxy_provider_id, record.channel_id, record.model_id
                ),
                &provider_model_keys,
            )
        })?;
        for tier in &record.pricing_tiers {
            let tier_record_id = format!("{}::{}", record_id, tier.tier_id);
            ensure_non_empty_field(
                "model_prices.pricing_tiers.tier_id",
                &tier_record_id,
                &tier.tier_id,
            )?;
            ensure_non_empty_field(
                "model_prices.pricing_tiers.condition_kind",
                &tier_record_id,
                &tier.condition_kind,
            )?;
            ensure_non_empty_field(
                "model_prices.pricing_tiers.currency_code",
                &tier_record_id,
                &tier.currency_code,
            )?;
            ensure_non_empty_field(
                "model_prices.pricing_tiers.price_unit",
                &tier_record_id,
                &tier.price_unit,
            )?;
            ensure_finite_non_negative_number(
                "model_prices.pricing_tiers.input_price",
                &tier_record_id,
                tier.input_price,
            )?;
            ensure_finite_non_negative_number(
                "model_prices.pricing_tiers.output_price",
                &tier_record_id,
                tier.output_price,
            )?;
            ensure_finite_non_negative_number(
                "model_prices.pricing_tiers.cache_read_price",
                &tier_record_id,
                tier.cache_read_price,
            )?;
            ensure_finite_non_negative_number(
                "model_prices.pricing_tiers.cache_write_price",
                &tier_record_id,
                tier.cache_write_price,
            )?;
            ensure_finite_non_negative_number(
                "model_prices.pricing_tiers.request_price",
                &tier_record_id,
                tier.request_price,
            )?;
            if let (Some(min_input_tokens), Some(max_input_tokens)) =
                (tier.min_input_tokens, tier.max_input_tokens)
            {
                if max_input_tokens < min_input_tokens {
                    bail!(
                        "bootstrap model_prices.pricing_tiers.min_input_tokens record {} exceeds max_input_tokens",
                        tier_record_id
                    );
                }
            }
        }
    }

    let active_provider_model_channels = collect_ids(
        data.provider_models
            .iter()
            .filter(|record| record.is_active)
            .map(|record| format!("{}::{}", record.proxy_provider_id, record.channel_id)),
    );
    let active_model_price_channels = collect_ids(
        data.model_prices
            .iter()
            .filter(|record| record.is_active)
            .map(|record| format!("{}::{}", record.proxy_provider_id, record.channel_id)),
    );
    for provider in &data.providers {
        for binding in &provider.channel_bindings {
            if binding.channel_id == provider.channel_id {
                continue;
            }

            let provider_channel_key = format!("{}::{}", provider.id, binding.channel_id);
            if !active_provider_model_channels.contains(&provider_channel_key) {
                bail!(
                    "provider {} declares non-primary channel binding {} without any active provider model coverage",
                    provider.id,
                    binding.channel_id
                );
            }
            if !active_model_price_channels.contains(&provider_channel_key) {
                bail!(
                    "provider {} declares non-primary channel binding {} without any active model price coverage",
                    provider.id,
                    binding.channel_id
                );
            }
        }
    }

    for record in &data.projects {
        ensure_reference(
            "projects.tenant_id",
            &record.id,
            &record.tenant_id,
            &tenant_ids,
        )?;
    }

    for record in &data.admin_users {
        ensure_valid_identity_email("admin_users.email", &record.id, &record.email)?;
        ensure_identity_password_material(
            "admin_users.password_hash",
            &record.id,
            &record.password_salt,
            &record.password_hash,
        )?;
    }

    for record in &data.portal_users {
        ensure_valid_identity_email("portal_users.email", &record.id, &record.email)?;
        ensure_identity_password_material(
            "portal_users.password_hash",
            &record.id,
            &record.password_salt,
            &record.password_hash,
        )?;
        ensure_reference(
            "portal_users.workspace_tenant_id",
            &record.id,
            &record.workspace_tenant_id,
            &tenant_ids,
        )?;
        ensure_reference(
            "portal_users.workspace_project_id",
            &record.id,
            &record.workspace_project_id,
            &project_ids,
        )?;
        if project_tenants
            .get(record.workspace_project_id.as_str())
            .is_some_and(|tenant_id| *tenant_id != record.workspace_tenant_id.as_str())
        {
            bail!(
                "portal user {} references workspace project {} that belongs to another tenant",
                record.id,
                record.workspace_project_id
            );
        }
    }

    for record in &data.gateway_api_keys {
        ensure_non_empty_field(
            "gateway_api_keys.hashed_key",
            &record.hashed_key,
            &record.hashed_key,
        )?;
        ensure_non_empty_field("gateway_api_keys.label", &record.hashed_key, &record.label)?;
        ensure_reference(
            "gateway_api_keys.tenant_id",
            &record.hashed_key,
            &record.tenant_id,
            &tenant_ids,
        )?;
        ensure_reference(
            "gateway_api_keys.project_id",
            &record.hashed_key,
            &record.project_id,
            &project_ids,
        )?;
        if project_tenants
            .get(record.project_id.as_str())
            .is_some_and(|tenant_id| *tenant_id != record.tenant_id.as_str())
        {
            bail!(
                "gateway api key {} references project {} that belongs to another tenant",
                record.hashed_key,
                record.project_id
            );
        }
        if let Some(group_id) = record.api_key_group_id.as_deref() {
            ensure_reference(
                "gateway_api_keys.api_key_group_id",
                &record.hashed_key,
                group_id,
                &api_key_group_ids,
            )?;
            let group = api_key_groups.get(group_id).ok_or_else(|| {
                anyhow::anyhow!(
                    "bootstrap gateway_api_keys.api_key_group_id record {} references missing {}",
                    record.hashed_key,
                    group_id
                )
            })?;
            if group.tenant_id != record.tenant_id
                || group.project_id != record.project_id
                || group.environment != record.environment
            {
                bail!(
                    "gateway api key {} references api key group {} with mismatched workspace or environment",
                    record.hashed_key,
                    group_id
                );
            }
        }
        if let Some(raw_key) = record.raw_key.as_deref() {
            let raw_key = raw_key.trim();
            ensure_non_empty_field("gateway_api_keys.raw_key", &record.hashed_key, raw_key)?;
            let expected_hash = sha256_hex(raw_key);
            if expected_hash != record.hashed_key {
                bail!(
                    "gateway api key {} raw_key does not match hashed_key",
                    record.hashed_key
                );
            }
        }
    }

    for record in &data.extension_installations {
        ensure_json_object(
            "extension_installations.config",
            &record.installation_id,
            &record.config,
        )?;
    }

    for record in &data.extension_instances {
        ensure_reference(
            "extension_instances.installation_id",
            &record.instance_id,
            &record.installation_id,
            &extension_installation_ids,
        )?;
        let installation = extension_installations
            .get(record.installation_id.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "bootstrap extension_instances.installation_id record {} references missing {}",
                    record.instance_id,
                    record.installation_id
                )
            })?;
        if installation.extension_id != record.extension_id {
            bail!(
                "bootstrap extension instance {} uses extension {} but installation {} targets {}",
                record.instance_id,
                record.extension_id,
                record.installation_id,
                installation.extension_id
            );
        }
        ensure_json_object(
            "extension_instances.config",
            &record.instance_id,
            &record.config,
        )?;
    }

    for record in &data.service_runtime_nodes {
        ensure_non_empty_field(
            "service_runtime_nodes.node_id",
            &record.node_id,
            &record.node_id,
        )?;
        ensure_non_empty_field(
            "service_runtime_nodes.service_kind",
            &record.node_id,
            &record.service_kind,
        )?;
        if record.last_seen_at_ms < record.started_at_ms {
            bail!(
                "service runtime node {} has last_seen_at_ms earlier than started_at_ms",
                record.node_id
            );
        }
    }

    for record in &data.extension_runtime_rollouts {
        ensure_non_empty_field(
            "extension_runtime_rollouts.rollout_id",
            &record.rollout_id,
            &record.rollout_id,
        )?;
        ensure_non_empty_field(
            "extension_runtime_rollouts.scope",
            &record.rollout_id,
            &record.scope,
        )?;
        ensure_non_empty_field(
            "extension_runtime_rollouts.created_by",
            &record.rollout_id,
            &record.created_by,
        )?;
        if record.deadline_at_ms < record.created_at_ms {
            bail!(
                "extension runtime rollout {} has deadline_at_ms earlier than created_at_ms",
                record.rollout_id
            );
        }
        if let Some(requested_extension_id) = record.requested_extension_id.as_deref() {
            ensure_reference(
                "extension_runtime_rollouts.requested_extension_id",
                &record.rollout_id,
                requested_extension_id,
                &extension_ids,
            )?;
        }
        if let Some(requested_instance_id) = record.requested_instance_id.as_deref() {
            ensure_reference(
                "extension_runtime_rollouts.requested_instance_id",
                &record.rollout_id,
                requested_instance_id,
                &extension_instance_ids,
            )?;
        }
        if let Some(resolved_extension_id) = record.resolved_extension_id.as_deref() {
            ensure_reference(
                "extension_runtime_rollouts.resolved_extension_id",
                &record.rollout_id,
                resolved_extension_id,
                &extension_ids,
            )?;
        }

        match record.scope.as_str() {
            "all" => {
                if record.requested_extension_id.is_some()
                    || record.requested_instance_id.is_some()
                    || record.resolved_extension_id.is_some()
                {
                    bail!(
                        "extension runtime rollout {} with scope all must not declare requested or resolved extension targets",
                        record.rollout_id
                    );
                }
            }
            "extension" => {
                let extension_id = record
                    .resolved_extension_id
                    .as_deref()
                    .or(record.requested_extension_id.as_deref())
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "extension runtime rollout {} with scope extension must declare requested_extension_id or resolved_extension_id",
                            record.rollout_id
                        )
                    })?;
                if let Some(requested_extension_id) = record.requested_extension_id.as_deref() {
                    if requested_extension_id != extension_id {
                        bail!(
                            "extension runtime rollout {} declares mismatched requested and resolved extension ids",
                            record.rollout_id
                        );
                    }
                }
                if let Some(resolved_extension_id) = record.resolved_extension_id.as_deref() {
                    if resolved_extension_id != extension_id {
                        bail!(
                            "extension runtime rollout {} declares mismatched requested and resolved extension ids",
                            record.rollout_id
                        );
                    }
                }
                if record.requested_instance_id.is_some() {
                    bail!(
                        "extension runtime rollout {} with scope extension must not declare requested_instance_id",
                        record.rollout_id
                    );
                }
            }
            "instance" => {
                let requested_instance_id = record.requested_instance_id.as_deref().ok_or_else(|| {
                    anyhow::anyhow!(
                        "extension runtime rollout {} with scope instance must declare requested_instance_id",
                        record.rollout_id
                    )
                })?;
                let instance = extension_instances
                    .get(requested_instance_id)
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "bootstrap extension_runtime_rollouts.requested_instance_id record {} references missing {}",
                            record.rollout_id,
                            requested_instance_id
                        )
                    })?;
                if let Some(requested_extension_id) = record.requested_extension_id.as_deref() {
                    if requested_extension_id != instance.extension_id {
                        bail!(
                            "extension runtime rollout {} requested_extension_id does not match instance {}",
                            record.rollout_id,
                            requested_instance_id
                        );
                    }
                }
                if let Some(resolved_extension_id) = record.resolved_extension_id.as_deref() {
                    if resolved_extension_id != instance.extension_id {
                        bail!(
                            "extension runtime rollout {} resolved_extension_id does not match instance {}",
                            record.rollout_id,
                            requested_instance_id
                        );
                    }
                }
                if !executable_provider_account_instance_ids.contains(requested_instance_id) {
                    bail!(
                        "extension runtime rollout {} requested_instance_id {} has no executable provider account binding",
                        record.rollout_id,
                        requested_instance_id
                    );
                }
            }
            other => bail!("unsupported extension runtime rollout scope: {other}"),
        }
    }

    for record in &data.extension_runtime_rollout_participants {
        ensure_reference(
            "extension_runtime_rollout_participants.rollout_id",
            &record.node_id,
            &record.rollout_id,
            &extension_runtime_rollout_ids,
        )?;
        ensure_reference(
            "extension_runtime_rollout_participants.node_id",
            &record.rollout_id,
            &record.node_id,
            &service_runtime_node_ids,
        )?;
        ensure_non_empty_field(
            "extension_runtime_rollout_participants.service_kind",
            &record.node_id,
            &record.service_kind,
        )?;
        ensure_non_empty_field(
            "extension_runtime_rollout_participants.status",
            &record.node_id,
            &record.status,
        )?;
        if let Some(message) = record.message.as_deref() {
            ensure_non_empty_field(
                "extension_runtime_rollout_participants.message",
                &record.node_id,
                message,
            )?;
        }
        let node = service_runtime_nodes
            .get(record.node_id.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "bootstrap extension_runtime_rollout_participants.node_id record {} references missing {}",
                    record.rollout_id,
                    record.node_id
                )
            })?;
        if node.service_kind != record.service_kind {
            bail!(
                "extension runtime rollout participant {} on node {} has mismatched service_kind {}",
                record.rollout_id,
                record.node_id,
                record.service_kind
            );
        }
        let rollout = extension_runtime_rollouts
            .get(record.rollout_id.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "bootstrap extension_runtime_rollout_participants.rollout_id record {} references missing {}",
                    record.node_id,
                    record.rollout_id
                )
            })?;
        if record.updated_at_ms < rollout.created_at_ms {
            bail!(
                "extension runtime rollout participant {} on node {} has updated_at_ms earlier than rollout creation",
                record.rollout_id,
                record.node_id
            );
        }
    }

    for record in &data.standalone_config_rollouts {
        ensure_non_empty_field(
            "standalone_config_rollouts.rollout_id",
            &record.rollout_id,
            &record.rollout_id,
        )?;
        ensure_non_empty_field(
            "standalone_config_rollouts.created_by",
            &record.rollout_id,
            &record.created_by,
        )?;
        if let Some(service_kind) = record.requested_service_kind.as_deref() {
            ensure_non_empty_field(
                "standalone_config_rollouts.requested_service_kind",
                &record.rollout_id,
                service_kind,
            )?;
        }
        if record.deadline_at_ms < record.created_at_ms {
            bail!(
                "standalone config rollout {} has deadline_at_ms earlier than created_at_ms",
                record.rollout_id
            );
        }
    }

    for record in &data.standalone_config_rollout_participants {
        ensure_reference(
            "standalone_config_rollout_participants.rollout_id",
            &record.node_id,
            &record.rollout_id,
            &standalone_config_rollout_ids,
        )?;
        ensure_reference(
            "standalone_config_rollout_participants.node_id",
            &record.rollout_id,
            &record.node_id,
            &service_runtime_node_ids,
        )?;
        ensure_non_empty_field(
            "standalone_config_rollout_participants.service_kind",
            &record.node_id,
            &record.service_kind,
        )?;
        ensure_non_empty_field(
            "standalone_config_rollout_participants.status",
            &record.node_id,
            &record.status,
        )?;
        if let Some(message) = record.message.as_deref() {
            ensure_non_empty_field(
                "standalone_config_rollout_participants.message",
                &record.node_id,
                message,
            )?;
        }
        let node = service_runtime_nodes
            .get(record.node_id.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "bootstrap standalone_config_rollout_participants.node_id record {} references missing {}",
                    record.rollout_id,
                    record.node_id
                )
            })?;
        if node.service_kind != record.service_kind {
            bail!(
                "standalone config rollout participant {} on node {} has mismatched service_kind {}",
                record.rollout_id,
                record.node_id,
                record.service_kind
            );
        }
        let rollout = standalone_config_rollouts
            .get(record.rollout_id.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "bootstrap standalone_config_rollout_participants.rollout_id record {} references missing {}",
                    record.node_id,
                    record.rollout_id
                )
            })?;
        if let Some(requested_service_kind) = rollout.requested_service_kind.as_deref() {
            if requested_service_kind != record.service_kind {
                bail!(
                    "standalone config rollout participant {} on node {} does not match requested_service_kind {}",
                    record.rollout_id,
                    record.node_id,
                    requested_service_kind
                );
            }
        }
        if record.updated_at_ms < rollout.created_at_ms {
            bail!(
                "standalone config rollout participant {} on node {} has updated_at_ms earlier than rollout creation",
                record.rollout_id,
                record.node_id
            );
        }
    }

    for record in &data.routing_profiles {
        ensure_reference(
            "routing_profiles.tenant_id",
            &record.profile_id,
            &record.tenant_id,
            &tenant_ids,
        )?;
        ensure_reference(
            "routing_profiles.project_id",
            &record.profile_id,
            &record.project_id,
            &project_ids,
        )?;
        if project_tenants
            .get(record.project_id.as_str())
            .is_some_and(|tenant_id| *tenant_id != record.tenant_id.as_str())
        {
            bail!(
                "routing profile {} references project {} that belongs to another tenant",
                record.profile_id,
                record.project_id
            );
        }
        let routed_provider_ids = routing_profile_provider_ids(record);
        ensure_provider_list_exists(
            "routing_profiles.providers",
            &record.profile_id,
            routed_provider_ids.clone(),
            &provider_ids,
        )?;
        let tenant_provider_ids = tenant_accessible_provider_ids_for(
            &tenant_accessible_executable_provider_ids,
            &record.tenant_id,
        );
        if let Some(default_provider_id) = record.default_provider_id.as_deref() {
            ensure_provider_has_enabled_account(
                "routing_profiles.default_provider_id",
                &record.profile_id,
                default_provider_id,
                &tenant_provider_ids,
            )?;
        }
        ensure_provider_list_has_enabled_accounts(
            "routing_profiles.providers",
            &record.profile_id,
            &routed_provider_ids,
            &tenant_provider_ids,
        )?;
    }

    for record in &data.routing_policies {
        let routed_provider_ids = record.declared_provider_ids();
        ensure_provider_list_exists(
            "routing_policies.providers",
            &record.policy_id,
            routed_provider_ids.clone(),
            &provider_ids,
        )?;
        if let Some(default_provider_id) = record.default_provider_id.as_deref() {
            ensure_provider_has_enabled_account(
                "routing_policies.default_provider_id",
                &record.policy_id,
                default_provider_id,
                &executable_provider_account_provider_ids,
            )?;
        }
        ensure_provider_list_has_enabled_accounts(
            "routing_policies.providers",
            &record.policy_id,
            &routed_provider_ids,
            &executable_provider_account_provider_ids,
        )?;
        if record.enabled && !routed_provider_ids.is_empty() {
            ensure_routing_policy_has_any_capability_matched_active_model_price_coverage(
                record,
                &routed_provider_ids,
                &data.model_prices,
                &data.models,
                &data.provider_models,
            )?;
        }
    }

    for record in &data.project_preferences {
        ensure_reference(
            "project_preferences.project_id",
            &record.project_id,
            &record.project_id,
            &project_ids,
        )?;
        if !record.preset_id.trim().is_empty() {
            ensure_reference(
                "project_preferences.preset_id",
                &record.project_id,
                &record.preset_id,
                &routing_profile_ids,
            )?;
            let profile = routing_profiles
                .get(record.preset_id.as_str())
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "bootstrap project_preferences.preset_id record {} references missing {}",
                        record.project_id,
                        record.preset_id
                    )
                })?;
            let expected_tenant_id =
                project_tenants
                    .get(record.project_id.as_str())
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "project preferences {} references project {} without a tenant mapping",
                            record.project_id,
                            record.project_id
                        )
                    })?;
            if profile.tenant_id != *expected_tenant_id || profile.project_id != record.project_id {
                bail!(
                    "project preferences {} references routing profile {} from another workspace",
                    record.project_id,
                    record.preset_id
                );
            }
        }
        let routed_provider_ids = project_preference_provider_ids(record);
        ensure_provider_list_exists(
            "project_preferences.providers",
            &record.project_id,
            routed_provider_ids.clone(),
            &provider_ids,
        )?;
        let tenant_provider_ids = tenant_accessible_provider_ids_for(
            &tenant_accessible_executable_provider_ids,
            project_tenants
                .get(record.project_id.as_str())
                .copied()
                .unwrap_or_default(),
        );
        if let Some(default_provider_id) = record.default_provider_id.as_deref() {
            ensure_provider_has_enabled_account(
                "project_preferences.default_provider_id",
                &record.project_id,
                default_provider_id,
                &tenant_provider_ids,
            )?;
        }
        ensure_provider_list_has_enabled_accounts(
            "project_preferences.providers",
            &record.project_id,
            &routed_provider_ids,
            &tenant_provider_ids,
        )?;
    }

    for record in &data.api_key_groups {
        ensure_reference(
            "api_key_groups.tenant_id",
            &record.group_id,
            &record.tenant_id,
            &tenant_ids,
        )?;
        ensure_reference(
            "api_key_groups.project_id",
            &record.group_id,
            &record.project_id,
            &project_ids,
        )?;
        if project_tenants
            .get(record.project_id.as_str())
            .is_some_and(|tenant_id| *tenant_id != record.tenant_id.as_str())
        {
            bail!(
                "api key group {} references project {} that belongs to another tenant",
                record.group_id,
                record.project_id
            );
        }
        if let Some(profile_id) = record.default_routing_profile_id.as_deref() {
            ensure_reference(
                "api_key_groups.default_routing_profile_id",
                &record.group_id,
                profile_id,
                &routing_profile_ids,
            )?;
            let profile = routing_profiles.get(profile_id).ok_or_else(|| {
                anyhow::anyhow!(
                    "bootstrap api_key_groups.default_routing_profile_id record {} references missing {}",
                    record.group_id,
                    profile_id
                )
            })?;
            if profile.tenant_id != record.tenant_id || profile.project_id != record.project_id {
                bail!(
                    "api key group {} references routing profile {} from another workspace",
                    record.group_id,
                    profile_id
                );
            }
        }
        if let Some(mode) = record.default_accounting_mode.as_deref() {
            BillingAccountingMode::from_str(mode).map_err(|error| {
                anyhow::anyhow!(
                    "api key group {} has unsupported default_accounting_mode {}: {}",
                    record.group_id,
                    mode,
                    error
                )
            })?;
        }
    }

    for record in &data.compiled_routing_snapshots {
        let tenant_provider_ids = tenant_accessible_provider_ids_for(
            &tenant_accessible_executable_provider_ids,
            record.tenant_id.as_deref().unwrap_or_default(),
        );
        ensure_compiled_snapshot_temporal_posture(record)?;
        ensure_compiled_snapshot_default_provider_matches_deterministic_priority(record)?;
        ensure_non_empty_field(
            "compiled_routing_snapshots.capability",
            &record.snapshot_id,
            &record.capability,
        )?;
        ensure_non_empty_field(
            "compiled_routing_snapshots.route_key",
            &record.snapshot_id,
            &record.route_key,
        )?;
        ensure_non_empty_field(
            "compiled_routing_snapshots.strategy",
            &record.snapshot_id,
            &record.strategy,
        )?;
        if let Some(tenant_id) = record.tenant_id.as_deref() {
            ensure_reference(
                "compiled_routing_snapshots.tenant_id",
                &record.snapshot_id,
                tenant_id,
                &tenant_ids,
            )?;
        }
        if let Some(project_id) = record.project_id.as_deref() {
            ensure_reference(
                "compiled_routing_snapshots.project_id",
                &record.snapshot_id,
                project_id,
                &project_ids,
            )?;
            if let Some(tenant_id) = record.tenant_id.as_deref() {
                if project_tenants
                    .get(project_id)
                    .is_some_and(|project_tenant_id| *project_tenant_id != tenant_id)
                {
                    bail!(
                        "compiled routing snapshot {} references project {} that belongs to another tenant",
                        record.snapshot_id,
                        project_id
                    );
                }
            }
        }
        if let Some(group_id) = record.api_key_group_id.as_deref() {
            ensure_reference(
                "compiled_routing_snapshots.api_key_group_id",
                &record.snapshot_id,
                group_id,
                &api_key_group_ids,
            )?;
            let group = api_key_groups.get(group_id).ok_or_else(|| {
                anyhow::anyhow!(
                    "bootstrap compiled_routing_snapshots.api_key_group_id record {} references missing {}",
                    record.snapshot_id,
                    group_id
                )
            })?;
            if record.tenant_id.as_deref() != Some(group.tenant_id.as_str())
                || record.project_id.as_deref() != Some(group.project_id.as_str())
            {
                bail!(
                    "compiled routing snapshot {} references api key group {} with mismatched workspace",
                    record.snapshot_id,
                    group_id
                );
            }
        }
        if let Some(policy_id) = record.matched_policy_id.as_deref() {
            ensure_reference(
                "compiled_routing_snapshots.matched_policy_id",
                &record.snapshot_id,
                policy_id,
                &routing_policy_ids,
            )?;
            ensure_compiled_snapshot_matched_policy_is_enabled_and_matches(
                record,
                policy_id,
                &routing_policies,
            )?;
        }
        if let Some(project_id) = record.project_routing_preferences_project_id.as_deref() {
            ensure_reference(
                "compiled_routing_snapshots.project_routing_preferences_project_id",
                &record.snapshot_id,
                project_id,
                &project_ids,
            )?;
            if record.project_id.as_deref() != Some(project_id) {
                bail!(
                    "compiled routing snapshot {} references project routing preferences for project {} but project_id is {:?}",
                    record.snapshot_id,
                    project_id,
                    record.project_id
                );
            }
        }
        if let Some(profile_id) = record.applied_routing_profile_id.as_deref() {
            ensure_reference(
                "compiled_routing_snapshots.applied_routing_profile_id",
                &record.snapshot_id,
                profile_id,
                &routing_profile_ids,
            )?;
            let profile = routing_profiles.get(profile_id).ok_or_else(|| {
                anyhow::anyhow!(
                    "bootstrap compiled_routing_snapshots.applied_routing_profile_id record {} references missing {}",
                    record.snapshot_id,
                    profile_id
                )
            })?;
            if record.tenant_id.as_deref() != Some(profile.tenant_id.as_str())
                || record.project_id.as_deref() != Some(profile.project_id.as_str())
            {
                bail!(
                    "compiled routing snapshot {} references routing profile {} with mismatched workspace",
                    record.snapshot_id,
                    profile_id
                );
            }
            for provider_id in compiled_routing_snapshot_provider_ids(record) {
                ensure_provider_declared_by_routing_profile(
                    "compiled_routing_snapshots.providers",
                    &record.snapshot_id,
                    profile_id,
                    &provider_id,
                    profile,
                )?;
            }
        }
        ensure_provider_list_exists(
            "compiled_routing_snapshots.providers",
            &record.snapshot_id,
            compiled_routing_snapshot_provider_ids(record),
            &provider_ids,
        )?;
        ensure_provider_list_has_enabled_accounts(
            "compiled_routing_snapshots.providers",
            &record.snapshot_id,
            &compiled_routing_snapshot_provider_ids(record),
            &tenant_provider_ids,
        )?;
        if let Some(default_provider_id) = record.default_provider_id.as_deref() {
            ensure_provider_has_active_route_price_coverage(
                "compiled_routing_snapshots.default_provider_id",
                &record.snapshot_id,
                default_provider_id,
                &record.route_key,
                &provider_channels,
                &active_model_price_keys,
            )?;
        }
    }

    for record in &data.routing_decision_logs {
        let tenant_provider_ids = tenant_accessible_provider_ids_for(
            &tenant_accessible_executable_provider_ids,
            record.tenant_id.as_deref().unwrap_or_default(),
        );
        let selected_provider_assessments = collect_selected_provider_assessments(record);
        ensure_selected_provider_assessment_exists(record, &selected_provider_assessments)?;
        ensure_selected_provider_assessment_is_available(record, &selected_provider_assessments)?;
        ensure_non_empty_field(
            "routing_decision_logs.capability",
            &record.decision_id,
            &record.capability,
        )?;
        ensure_non_empty_field(
            "routing_decision_logs.route_key",
            &record.decision_id,
            &record.route_key,
        )?;
        ensure_non_empty_field(
            "routing_decision_logs.strategy",
            &record.decision_id,
            &record.strategy,
        )?;
        if let Some(tenant_id) = record.tenant_id.as_deref() {
            ensure_reference(
                "routing_decision_logs.tenant_id",
                &record.decision_id,
                tenant_id,
                &tenant_ids,
            )?;
        }
        if let Some(project_id) = record.project_id.as_deref() {
            ensure_reference(
                "routing_decision_logs.project_id",
                &record.decision_id,
                project_id,
                &project_ids,
            )?;
            if let Some(tenant_id) = record.tenant_id.as_deref() {
                if project_tenants
                    .get(project_id)
                    .is_some_and(|project_tenant_id| *project_tenant_id != tenant_id)
                {
                    bail!(
                        "routing decision log {} references project {} that belongs to another tenant",
                        record.decision_id,
                        project_id
                    );
                }
            }
        }
        if let Some(group_id) = record.api_key_group_id.as_deref() {
            ensure_reference(
                "routing_decision_logs.api_key_group_id",
                &record.decision_id,
                group_id,
                &api_key_group_ids,
            )?;
            let group = api_key_groups.get(group_id).ok_or_else(|| {
                anyhow::anyhow!(
                    "bootstrap routing_decision_logs.api_key_group_id record {} references missing {}",
                    record.decision_id,
                    group_id
                )
            })?;
            if record.tenant_id.as_deref() != Some(group.tenant_id.as_str())
                || record.project_id.as_deref() != Some(group.project_id.as_str())
            {
                bail!(
                    "routing decision log {} references api key group {} with mismatched workspace",
                    record.decision_id,
                    group_id
                );
            }
        }
        ensure_reference(
            "routing_decision_logs.selected_provider_id",
            &record.decision_id,
            &record.selected_provider_id,
            &provider_ids,
        )?;
        ensure_provider_has_enabled_account(
            "routing_decision_logs.selected_provider_id",
            &record.decision_id,
            &record.selected_provider_id,
            &tenant_provider_ids,
        )?;
        ensure_provider_has_active_route_price_coverage(
            "routing_decision_logs.selected_provider_id",
            &record.decision_id,
            &record.selected_provider_id,
            &record.route_key,
            &provider_channels,
            &active_model_price_keys,
        )?;
        if let Some(policy_id) = record.matched_policy_id.as_deref() {
            ensure_reference(
                "routing_decision_logs.matched_policy_id",
                &record.decision_id,
                policy_id,
                &routing_policy_ids,
            )?;
            ensure_routing_decision_matched_policy_is_enabled_and_matches(
                record,
                policy_id,
                &routing_policies,
            )?;
        }
        let applied_routing_profile = if let Some(profile_id) =
            record.applied_routing_profile_id.as_deref()
        {
            ensure_reference(
                "routing_decision_logs.applied_routing_profile_id",
                &record.decision_id,
                profile_id,
                &routing_profile_ids,
            )?;
            let profile = routing_profiles.get(profile_id).ok_or_else(|| {
                anyhow::anyhow!(
                    "bootstrap routing_decision_logs.applied_routing_profile_id record {} references missing {}",
                    record.decision_id,
                    profile_id
                )
            })?;
            if record.tenant_id.as_deref() != Some(profile.tenant_id.as_str())
                || record.project_id.as_deref() != Some(profile.project_id.as_str())
            {
                bail!(
                    "routing decision log {} references routing profile {} with mismatched workspace",
                    record.decision_id,
                    profile_id
                );
            }
            ensure_provider_declared_by_routing_profile(
                "routing_decision_logs.selected_provider_id",
                &record.decision_id,
                profile_id,
                &record.selected_provider_id,
                profile,
            )?;
            Some((profile_id, profile))
        } else {
            None
        };
        if let Some(snapshot_id) = record.compiled_routing_snapshot_id.as_deref() {
            ensure_reference(
                "routing_decision_logs.compiled_routing_snapshot_id",
                &record.decision_id,
                snapshot_id,
                &compiled_routing_snapshot_ids,
            )?;
            let snapshot = compiled_routing_snapshots.get(snapshot_id).ok_or_else(|| {
                anyhow::anyhow!(
                    "bootstrap routing_decision_logs.compiled_routing_snapshot_id record {} references missing {}",
                    record.decision_id,
                    snapshot_id
                )
            })?;
            if record.tenant_id.as_deref() != snapshot.tenant_id.as_deref()
                || record.project_id.as_deref() != snapshot.project_id.as_deref()
                || record.api_key_group_id.as_deref() != snapshot.api_key_group_id.as_deref()
            {
                bail!(
                    "routing decision log {} references snapshot {} with mismatched workspace context",
                    record.decision_id,
                    snapshot_id
                );
            }
            ensure_routing_decision_matches_compiled_snapshot(record, snapshot_id, snapshot)?;
            ensure_provider_declared_by_compiled_snapshot(
                "routing_decision_logs.selected_provider_id",
                &record.decision_id,
                snapshot_id,
                &record.selected_provider_id,
                snapshot,
            )?;
            ensure_selected_provider_assessment_satisfies_snapshot_health_requirement(
                record,
                snapshot,
                &selected_provider_assessments,
            )?;
        }
        for assessment in &record.assessments {
            ensure_reference(
                "routing_decision_logs.assessments.provider_id",
                &record.decision_id,
                &assessment.provider_id,
                &provider_ids,
            )?;
            ensure_provider_has_enabled_account(
                "routing_decision_logs.assessments.provider_id",
                &record.decision_id,
                &assessment.provider_id,
                &tenant_provider_ids,
            )?;
            if let Some((profile_id, profile)) = applied_routing_profile {
                ensure_provider_declared_by_routing_profile(
                    "routing_decision_logs.assessments.provider_id",
                    &record.decision_id,
                    profile_id,
                    &assessment.provider_id,
                    profile,
                )?;
            }
            if let Some(snapshot_id) = record.compiled_routing_snapshot_id.as_deref() {
                let snapshot = compiled_routing_snapshots.get(snapshot_id).ok_or_else(|| {
                    anyhow::anyhow!(
                        "bootstrap routing_decision_logs.compiled_routing_snapshot_id record {} references missing {}",
                        record.decision_id,
                        snapshot_id
                    )
                })?;
                ensure_provider_declared_by_compiled_snapshot(
                    "routing_decision_logs.assessments.provider_id",
                    &record.decision_id,
                    snapshot_id,
                    &assessment.provider_id,
                    snapshot,
                )?;
            }
        }
    }

    for record in &data.provider_health_snapshots {
        ensure_provider_health_snapshot_runtime_posture(record)?;
        ensure_reference(
            "provider_health_snapshots.provider_id",
            &record.provider_id,
            &record.provider_id,
            &provider_ids,
        )?;
        ensure_non_empty_field(
            "provider_health_snapshots.runtime",
            &record.provider_id,
            &record.runtime,
        )?;
        let provider = providers.get(record.provider_id.as_str()).ok_or_else(|| {
            anyhow::anyhow!(
                "bootstrap provider_health_snapshots.provider_id record {} references missing {}",
                record.provider_id,
                record.provider_id
            )
        })?;
        if provider.extension_id != record.extension_id {
            bail!(
                "provider health snapshot {} uses extension {} but provider {} is bound to {}",
                record.provider_id,
                record.extension_id,
                record.provider_id,
                provider.extension_id
            );
        }
        if let Some(instance_id) = record.instance_id.as_deref() {
            ensure_reference(
                "provider_health_snapshots.instance_id",
                &record.provider_id,
                instance_id,
                &extension_instance_ids,
            )?;
            let instance = extension_instances.get(instance_id).ok_or_else(|| {
                anyhow::anyhow!(
                    "bootstrap provider_health_snapshots.instance_id record {} references missing {}",
                    record.provider_id,
                    instance_id
                )
            })?;
            if instance.extension_id != record.extension_id {
                bail!(
                    "provider health snapshot {} references instance {} with mismatched extension {}",
                    record.provider_id,
                    instance_id,
                    instance.extension_id
                );
            }
            let provider_instance_binding = format!("{}::{}", record.provider_id, instance_id);
            if !executable_provider_account_instance_bindings.contains(&provider_instance_binding) {
                bail!(
                    "provider health snapshot {} references instance {} without any executable provider account binding",
                    record.provider_id,
                    instance_id
                );
            }
        }
    }

    for record in &data.billing_events {
        ensure_non_empty_field(
            "billing_events.capability",
            &record.event_id,
            &record.capability,
        )?;
        ensure_non_empty_field(
            "billing_events.route_key",
            &record.event_id,
            &record.route_key,
        )?;
        ensure_non_empty_field(
            "billing_events.usage_model",
            &record.event_id,
            &record.usage_model,
        )?;
        ensure_non_empty_field(
            "billing_events.operation_kind",
            &record.event_id,
            &record.operation_kind,
        )?;
        ensure_non_empty_field(
            "billing_events.modality",
            &record.event_id,
            &record.modality,
        )?;
        ensure_reference(
            "billing_events.tenant_id",
            &record.event_id,
            &record.tenant_id,
            &tenant_ids,
        )?;
        ensure_reference(
            "billing_events.project_id",
            &record.event_id,
            &record.project_id,
            &project_ids,
        )?;
        if project_tenants
            .get(record.project_id.as_str())
            .is_some_and(|tenant_id| *tenant_id != record.tenant_id.as_str())
        {
            bail!(
                "billing event {} references project {} that belongs to another tenant",
                record.event_id,
                record.project_id
            );
        }
        ensure_reference(
            "billing_events.provider_id",
            &record.event_id,
            &record.provider_id,
            &provider_ids,
        )?;
        let tenant_provider_ids = tenant_accessible_provider_ids_for(
            &tenant_accessible_executable_provider_ids,
            &record.tenant_id,
        );
        ensure_provider_has_enabled_account(
            "billing_events.provider_id",
            &record.event_id,
            &record.provider_id,
            &tenant_provider_ids,
        )?;
        if let Some(channel_id) = record.channel_id.as_deref() {
            ensure_reference(
                "billing_events.channel_id",
                &record.event_id,
                channel_id,
                &channel_ids,
            )?;
            if !provider_channels
                .get(record.provider_id.as_str())
                .is_some_and(|channels| channels.contains(channel_id))
            {
                bail!(
                    "billing event {} references provider {} with channel {} without a provider channel binding",
                    record.event_id,
                    record.provider_id,
                    channel_id
                );
            }
            ensure_reference(
                "billing_events.route_key",
                &record.event_id,
                &format!("{}::{}", channel_id, record.route_key),
                &channel_model_keys,
            )?;
            ensure_active_model_price_coverage(
                "billing_events.route_key",
                &record.event_id,
                &record.provider_id,
                channel_id,
                &record.route_key,
                &active_model_price_keys,
            )?;
        }
        if !billing_event_matches_catalog(record, &model_variant_keys, &data.provider_models) {
            bail!(
                "billing event {} references usage_model {} that is not available on provider {} for route_key {}",
                record.event_id,
                record.usage_model,
                record.provider_id,
                record.route_key
            );
        }
        if let Some(group_id) = record.api_key_group_id.as_deref() {
            ensure_reference(
                "billing_events.api_key_group_id",
                &record.event_id,
                group_id,
                &api_key_group_ids,
            )?;
            let group = api_key_groups.get(group_id).ok_or_else(|| {
                anyhow::anyhow!(
                    "bootstrap billing_events.api_key_group_id record {} references missing {}",
                    record.event_id,
                    group_id
                )
            })?;
            if group.tenant_id != record.tenant_id || group.project_id != record.project_id {
                bail!(
                    "billing event {} references api key group {} with mismatched workspace",
                    record.event_id,
                    group_id
                );
            }
        }
        if let Some(api_key_hash) = record.api_key_hash.as_deref() {
            ensure_reference(
                "billing_events.api_key_hash",
                &record.event_id,
                api_key_hash,
                &gateway_api_key_hashes,
            )?;
            let gateway_api_key = gateway_api_keys.get(api_key_hash).ok_or_else(|| {
                anyhow::anyhow!(
                    "bootstrap billing_events.api_key_hash record {} references missing {}",
                    record.event_id,
                    api_key_hash
                )
            })?;
            if gateway_api_key.tenant_id != record.tenant_id
                || gateway_api_key.project_id != record.project_id
                || gateway_api_key.api_key_group_id.as_deref() != record.api_key_group_id.as_deref()
            {
                let gateway_group = gateway_api_key.api_key_group_id.as_deref().unwrap_or("<none>");
                let billing_group = record.api_key_group_id.as_deref().unwrap_or("<none>");
                bail!(
                    "billing event {} api_key_hash {} resolves to gateway api key metadata with mismatched workspace or api_key_group_id (billing group {}, gateway key group {})",
                    record.event_id,
                    api_key_hash,
                    billing_group,
                    gateway_group
                );
            }
        }
        if let Some(profile_id) = record.applied_routing_profile_id.as_deref() {
            ensure_reference(
                "billing_events.applied_routing_profile_id",
                &record.event_id,
                profile_id,
                &routing_profile_ids,
            )?;
            let profile = routing_profiles.get(profile_id).ok_or_else(|| {
                anyhow::anyhow!(
                    "bootstrap billing_events.applied_routing_profile_id record {} references missing {}",
                    record.event_id,
                    profile_id
                )
            })?;
            if profile.tenant_id != record.tenant_id || profile.project_id != record.project_id {
                bail!(
                    "billing event {} references routing profile {} with mismatched workspace",
                    record.event_id,
                    profile_id
                );
            }
            ensure_provider_declared_by_routing_profile(
                "billing_events.provider_id",
                &record.event_id,
                profile_id,
                &record.provider_id,
                profile,
            )?;
        }
        if let Some(snapshot_id) = record.compiled_routing_snapshot_id.as_deref() {
            ensure_reference(
                "billing_events.compiled_routing_snapshot_id",
                &record.event_id,
                snapshot_id,
                &compiled_routing_snapshot_ids,
            )?;
            let snapshot = compiled_routing_snapshots.get(snapshot_id).ok_or_else(|| {
                anyhow::anyhow!(
                    "bootstrap billing_events.compiled_routing_snapshot_id record {} references missing {}",
                    record.event_id,
                    snapshot_id
                )
            })?;
            if snapshot.tenant_id.as_deref() != Some(record.tenant_id.as_str())
                || snapshot.project_id.as_deref() != Some(record.project_id.as_str())
                || snapshot.api_key_group_id.as_deref() != record.api_key_group_id.as_deref()
            {
                bail!(
                    "billing event {} references routing snapshot {} with mismatched workspace context",
                    record.event_id,
                    snapshot_id
                );
            }
            ensure_billing_event_matches_compiled_snapshot(record, snapshot_id, snapshot)?;
            ensure_provider_declared_by_compiled_snapshot(
                "billing_events.provider_id",
                &record.event_id,
                snapshot_id,
                &record.provider_id,
                snapshot,
            )?;
        }
    }

    for record in &data.quota_policies {
        ensure_reference(
            "quota_policies.project_id",
            &record.policy_id,
            &record.project_id,
            &project_ids,
        )?;
    }

    for record in &data.rate_limit_policies {
        ensure_reference(
            "rate_limit_policies.project_id",
            &record.policy_id,
            &record.project_id,
            &project_ids,
        )?;
    }

    for record in &data.pricing_plans {
        let record_id = record.pricing_plan_id.to_string();
        if let Some(effective_to_ms) = record.effective_to_ms {
            if effective_to_ms < record.effective_from_ms {
                bail!(
                    "pricing plan {} has effective_to_ms earlier than effective_from_ms",
                    record.pricing_plan_id
                );
            }
        }
        ensure_non_empty_field(
            "pricing_plans.currency_code",
            &record_id,
            &record.currency_code,
        )?;
        ensure_non_empty_field(
            "pricing_plans.credit_unit_code",
            &record_id,
            &record.credit_unit_code,
        )?;
        ensure_non_empty_field("pricing_plans.status", &record_id, &record.status)?;
    }

    for record in &data.pricing_rates {
        let record_id = record.pricing_rate_id.to_string();
        let pricing_plan_id = record.pricing_plan_id.to_string();
        ensure_reference(
            "pricing_rates.pricing_plan_id",
            &record_id,
            &pricing_plan_id,
            &pricing_plan_ids,
        )?;
        let pricing_plan = pricing_plans_by_id.get(&pricing_plan_id).ok_or_else(|| {
            anyhow::anyhow!(
                "bootstrap pricing_rates.pricing_plan_id record {} references missing {}",
                record.pricing_rate_id,
                record.pricing_plan_id
            )
        })?;
        if pricing_plan.tenant_id != record.tenant_id
            || pricing_plan.organization_id != record.organization_id
        {
            bail!(
                "pricing rate {} references pricing plan {} with mismatched tenant/organization ownership",
                record.pricing_rate_id,
                record.pricing_plan_id
            );
        }
        if record.model_code.is_some() && record.provider_code.is_none() {
            bail!(
                "pricing rate {} sets model_code without provider_code",
                record.pricing_rate_id
            );
        }
        if record.status.trim() == "active" && !active_pricing_plan_ids.contains(&pricing_plan_id) {
            bail!(
                "pricing rate {} is an active pricing rate but parent pricing plan {} is not active",
                record.pricing_rate_id,
                record.pricing_plan_id
            );
        }
        if let Some(provider_code) = record.provider_code.as_deref() {
            ensure_reference(
                "pricing_rates.provider_code",
                &record_id,
                provider_code,
                &provider_ids,
            )?;
            if record.status.trim() == "active" {
                ensure_provider_has_enabled_account(
                    "pricing_rates.provider_code",
                    &record_id,
                    provider_code,
                    &executable_provider_account_provider_ids,
                )?;
            }
        }
        if let (Some(provider_code), Some(model_code)) =
            (record.provider_code.as_deref(), record.model_code.as_deref())
        {
            if record.status.trim() == "active" {
                ensure_provider_has_active_route_price_coverage(
                    "pricing_rates.model_code",
                    &record_id,
                    provider_code,
                    model_code,
                    &provider_channels,
                    &active_model_price_keys,
                )?;
            }
            if !provider_supports_catalog_model(
                provider_code,
                model_code,
                &data.models,
                &data.provider_models,
            ) {
                bail!(
                    "pricing rate {} references model {} that is not available on provider {}",
                    record.pricing_rate_id,
                    model_code,
                    provider_code
                );
            }
            if let Some(capability_code) = record.capability_code.as_deref() {
                let declared_capability = serde_json::from_value::<
                    sdkwork_api_domain_catalog::ModelCapability,
                >(serde_json::Value::String(capability_code.to_owned()))
                .with_context(|| {
                    format!(
                        "bootstrap pricing_rates.capability_code record {} contains unsupported capability {}",
                        record.pricing_rate_id, capability_code
                    )
                })?;
                if !provider_supports_catalog_model_capability(
                    provider_code,
                    model_code,
                    &declared_capability,
                    &data.models,
                    &data.provider_models,
                ) {
                    bail!(
                        "pricing rate {} declares capability {} not supported by provider model {}::{}",
                        record.pricing_rate_id,
                        capability_code,
                        provider_code,
                        model_code
                    );
                }
            }
        } else if let Some(provider_code) = record.provider_code.as_deref() {
            if record.status.trim() == "active" {
                if let Some(capability_code) = record.capability_code.as_deref() {
                    let declared_capability = serde_json::from_value::<
                        sdkwork_api_domain_catalog::ModelCapability,
                    >(serde_json::Value::String(capability_code.to_owned()))
                    .with_context(|| {
                        format!(
                            "bootstrap pricing_rates.capability_code record {} contains unsupported capability {}",
                            record.pricing_rate_id, capability_code
                        )
                    })?;
                    ensure_provider_has_any_active_model_price_capability_coverage(
                        "pricing_rates.provider_code",
                        &record_id,
                        provider_code,
                        capability_code,
                        &declared_capability,
                        &provider_channels,
                        &data.model_prices,
                        &data.models,
                        &data.provider_models,
                    )?;
                } else {
                    ensure_provider_has_any_active_model_price_coverage(
                        "pricing_rates.provider_code",
                        &record_id,
                        provider_code,
                        &provider_channels,
                        &active_model_price_keys,
                    )?;
                }
            }
        }
    }

    for record in &data.payment_methods {
        serde_json::from_str::<serde_json::Value>(&record.config_json).with_context(|| {
            format!(
                "payment method {} contains invalid config_json",
                record.payment_method_id
            )
        })?;
    }

    for record in &data.payment_method_credential_bindings {
        ensure_reference(
            "payment_method_credential_bindings.payment_method_id",
            &record.binding_id,
            &record.payment_method_id,
            &payment_method_ids,
        )?;
    }

    for record in &data.project_memberships {
        ensure_reference(
            "project_memberships.project_id",
            &record.membership_id,
            &record.project_id,
            &project_ids,
        )?;
    }

    for record in &data.commerce_orders {
        ensure_reference(
            "commerce_orders.project_id",
            &record.order_id,
            &record.project_id,
            &project_ids,
        )?;
        ensure_reference(
            "commerce_orders.user_id",
            &record.order_id,
            &record.user_id,
            &allowed_workspace_user_ids,
        )?;
        ensure_non_empty_field(
            "commerce_orders.target_kind",
            &record.order_id,
            &record.target_kind,
        )?;
        ensure_non_empty_field(
            "commerce_orders.target_id",
            &record.order_id,
            &record.target_id,
        )?;
        ensure_non_empty_field(
            "commerce_orders.target_name",
            &record.order_id,
            &record.target_name,
        )?;
        ensure_non_empty_field(
            "commerce_orders.currency_code",
            &record.order_id,
            &record.currency_code,
        )?;
        let implied_subsidy = record
            .list_price_cents
            .checked_sub(record.payable_price_cents)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "commerce order {} payable_price_cents exceeds list_price_cents",
                    record.order_id
                )
            })?;
        if implied_subsidy != record.subsidy_amount_minor {
            bail!(
                "commerce order {} subsidy_amount_minor does not match list_price_cents minus payable_price_cents",
                record.order_id
            );
        }
        if record.refundable_amount_minor > record.payable_price_cents {
            bail!(
                "commerce order {} refundable_amount_minor exceeds payable_price_cents",
                record.order_id
            );
        }
        if record.refunded_amount_minor > record.refundable_amount_minor {
            bail!(
                "commerce order {} refunded_amount_minor exceeds refundable_amount_minor",
                record.order_id
            );
        }
        let linked_refund_total = data
            .commerce_refunds
            .iter()
            .filter(|refund| refund.order_id == record.order_id)
            .map(|refund| refund.amount_minor)
            .sum::<u64>();
        if linked_refund_total != record.refunded_amount_minor {
            bail!(
                "commerce order {} refunded_amount_minor does not match linked refunds total",
                record.order_id
            );
        }
        ensure_non_empty_field("commerce_orders.status", &record.order_id, &record.status)?;
        ensure_non_empty_field(
            "commerce_orders.settlement_status",
            &record.order_id,
            &record.settlement_status,
        )?;
        ensure_non_empty_field("commerce_orders.source", &record.order_id, &record.source)?;
        if let Some(payment_method_id) = record.payment_method_id.as_deref() {
            ensure_reference(
                "commerce_orders.payment_method_id",
                &record.order_id,
                payment_method_id,
                &payment_method_ids,
            )?;
        }
        if let Some(payment_attempt_id) = record.latest_payment_attempt_id.as_deref() {
            ensure_reference(
                "commerce_orders.latest_payment_attempt_id",
                &record.order_id,
                payment_attempt_id,
                &commerce_payment_attempt_ids,
            )?;
            let attempt = commerce_payment_attempts
                .get(payment_attempt_id)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "bootstrap commerce_orders.latest_payment_attempt_id record {} references missing {}",
                        record.order_id,
                        payment_attempt_id
                    )
                })?;
            if attempt.order_id != record.order_id {
                bail!(
                    "commerce order {} references latest payment attempt {} for another order",
                    record.order_id,
                    payment_attempt_id
                );
            }
            if attempt.amount_minor != record.payable_price_cents {
                bail!(
                    "commerce order {} latest payment attempt {} amount_minor does not match payable_price_cents",
                    record.order_id,
                    payment_attempt_id
                );
            }
        }
        if let Some(marketing_campaign_id) = record.marketing_campaign_id.as_deref() {
            ensure_reference(
                "commerce_orders.marketing_campaign_id",
                &record.order_id,
                marketing_campaign_id,
                &marketing_campaign_ids,
            )?;
        }
        if let Some(coupon_code) = record.applied_coupon_code.as_deref() {
            let normalized_coupon_code = normalize_coupon_code(coupon_code);
            ensure_reference(
                "commerce_orders.applied_coupon_code",
                &record.order_id,
                &normalized_coupon_code,
                &available_coupon_codes,
            )?;
            if let Some(marketing_campaign_id) = record.marketing_campaign_id.as_deref() {
                let campaign = marketing_campaigns.get(marketing_campaign_id).ok_or_else(|| {
                    anyhow::anyhow!(
                        "bootstrap commerce_orders.marketing_campaign_id record {} references missing {}",
                        record.order_id,
                        marketing_campaign_id
                    )
                })?;
                let coupon_code_record = coupon_codes_by_value
                    .get(normalized_coupon_code.as_str())
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "bootstrap commerce_orders.applied_coupon_code record {} references missing {}",
                            record.order_id,
                            normalized_coupon_code
                        )
                    })?;
                if campaign.coupon_template_id != coupon_code_record.coupon_template_id {
                    bail!(
                        "commerce order {} applied coupon code {} resolves to template {} outside marketing campaign {} template {}",
                        record.order_id,
                        coupon_code,
                        coupon_code_record.coupon_template_id,
                        marketing_campaign_id,
                        campaign.coupon_template_id
                    );
                }
            }
        }
        ensure_json_object_from_str(
            "commerce_orders.pricing_snapshot_json",
            &record.order_id,
            &record.pricing_snapshot_json,
        )?;
    }

    for record in &data.commerce_payment_events {
        ensure_reference(
            "commerce_payment_events.order_id",
            &record.payment_event_id,
            &record.order_id,
            &commerce_order_ids,
        )?;
        ensure_reference(
            "commerce_payment_events.project_id",
            &record.payment_event_id,
            &record.project_id,
            &project_ids,
        )?;
        ensure_reference(
            "commerce_payment_events.user_id",
            &record.payment_event_id,
            &record.user_id,
            &allowed_workspace_user_ids,
        )?;
        ensure_non_empty_field(
            "commerce_payment_events.provider",
            &record.payment_event_id,
            &record.provider,
        )?;
        ensure_non_empty_field(
            "commerce_payment_events.dedupe_key",
            &record.payment_event_id,
            &record.dedupe_key,
        )?;
        ensure_non_empty_field(
            "commerce_payment_events.event_type",
            &record.payment_event_id,
            &record.event_type,
        )?;
        if record.processing_status == CommercePaymentEventProcessingStatus::Processed
            && record.processed_at_ms.is_none()
        {
            bail!(
                "commerce payment event {} with processing_status processed must declare processed_at_ms",
                record.payment_event_id
            );
        }
        ensure_json_value_from_str(
            "commerce_payment_events.payload_json",
            &record.payment_event_id,
            &record.payload_json,
        )?;
        if let Some(processed_at_ms) = record.processed_at_ms {
            if processed_at_ms < record.received_at_ms {
                bail!(
                    "commerce payment event {} has processed_at_ms earlier than received_at_ms",
                    record.payment_event_id
                );
            }
        }
        let order = commerce_orders
            .get(record.order_id.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "bootstrap commerce_payment_events.order_id record {} references missing {}",
                    record.payment_event_id,
                    record.order_id
                )
            })?;
        if order.project_id != record.project_id || order.user_id != record.user_id {
            bail!(
                "commerce payment event {} references order {} with mismatched project or user",
                record.payment_event_id,
                record.order_id
            );
        }
        if let Some(payment_method_id) = order.payment_method_id.as_deref() {
            let payment_method = payment_methods.get(payment_method_id).ok_or_else(|| {
                anyhow::anyhow!(
                    "bootstrap commerce_orders.payment_method_id record {} references missing {}",
                    record.order_id,
                    payment_method_id
                )
            })?;
            if payment_method.provider != record.provider {
                bail!(
                    "commerce payment event {} provider does not match order {} payment method {}",
                    record.payment_event_id,
                    record.order_id,
                    payment_method_id
                );
            }
        }
        if let Some(payment_attempt_id) = order.latest_payment_attempt_id.as_deref() {
            let attempt = commerce_payment_attempts
                .get(payment_attempt_id)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "bootstrap commerce_orders.latest_payment_attempt_id record {} references missing {}",
                        record.order_id,
                        payment_attempt_id
                    )
                })?;
            if attempt.provider != record.provider {
                bail!(
                    "commerce payment event {} provider does not match order {} latest payment attempt {}",
                    record.payment_event_id,
                    record.order_id,
                    payment_attempt_id
                );
            }
        }
        if let Some(order_status_after) = record.order_status_after.as_deref() {
            ensure_non_empty_field(
                "commerce_payment_events.order_status_after",
                &record.payment_event_id,
                order_status_after,
            )?;
        }
    }

    for record in &data.commerce_payment_attempts {
        ensure_reference(
            "commerce_payment_attempts.order_id",
            &record.payment_attempt_id,
            &record.order_id,
            &commerce_order_ids,
        )?;
        ensure_reference(
            "commerce_payment_attempts.project_id",
            &record.payment_attempt_id,
            &record.project_id,
            &project_ids,
        )?;
        ensure_reference(
            "commerce_payment_attempts.user_id",
            &record.payment_attempt_id,
            &record.user_id,
            &allowed_workspace_user_ids,
        )?;
        ensure_reference(
            "commerce_payment_attempts.payment_method_id",
            &record.payment_attempt_id,
            &record.payment_method_id,
            &payment_method_ids,
        )?;
        ensure_non_empty_field(
            "commerce_payment_attempts.provider",
            &record.payment_attempt_id,
            &record.provider,
        )?;
        ensure_non_empty_field(
            "commerce_payment_attempts.channel",
            &record.payment_attempt_id,
            &record.channel,
        )?;
        ensure_non_empty_field(
            "commerce_payment_attempts.status",
            &record.payment_attempt_id,
            &record.status,
        )?;
        ensure_non_empty_field(
            "commerce_payment_attempts.idempotency_key",
            &record.payment_attempt_id,
            &record.idempotency_key,
        )?;
        ensure_non_empty_field(
            "commerce_payment_attempts.currency_code",
            &record.payment_attempt_id,
            &record.currency_code,
        )?;
        ensure_json_value_from_str(
            "commerce_payment_attempts.request_payload_json",
            &record.payment_attempt_id,
            &record.request_payload_json,
        )?;
        ensure_json_value_from_str(
            "commerce_payment_attempts.response_payload_json",
            &record.payment_attempt_id,
            &record.response_payload_json,
        )?;
        if record.attempt_sequence == 0 {
            bail!(
                "commerce payment attempt {} must have attempt_sequence >= 1",
                record.payment_attempt_id
            );
        }
        if record.captured_amount_minor > record.amount_minor {
            bail!(
                "commerce payment attempt {} cannot capture more than the original amount",
                record.payment_attempt_id
            );
        }
        if record.refunded_amount_minor > record.captured_amount_minor {
            bail!(
                "commerce payment attempt {} cannot refund more than the captured amount",
                record.payment_attempt_id
            );
        }
        if let Some(expires_at_ms) = record.expires_at_ms {
            if expires_at_ms < record.initiated_at_ms {
                bail!(
                    "commerce payment attempt {} has expires_at_ms earlier than initiated_at_ms",
                    record.payment_attempt_id
                );
            }
        }
        if let Some(completed_at_ms) = record.completed_at_ms {
            if completed_at_ms < record.initiated_at_ms {
                bail!(
                    "commerce payment attempt {} has completed_at_ms earlier than initiated_at_ms",
                    record.payment_attempt_id
                );
            }
        }
        if record.updated_at_ms < record.initiated_at_ms {
            bail!(
                "commerce payment attempt {} has updated_at_ms earlier than initiated_at_ms",
                record.payment_attempt_id
            );
        }
        if record.status == "succeeded" {
            if record.captured_amount_minor != record.amount_minor {
                bail!(
                    "commerce payment attempt {} with status succeeded must capture the full amount",
                    record.payment_attempt_id
                );
            }
            if record.completed_at_ms.is_none() {
                bail!(
                    "commerce payment attempt {} with status succeeded must declare completed_at_ms",
                    record.payment_attempt_id
                );
            }
        }
        let linked_refund_total = data
            .commerce_refunds
            .iter()
            .filter(|refund| {
                refund.payment_attempt_id.as_deref() == Some(record.payment_attempt_id.as_str())
            })
            .map(|refund| refund.amount_minor)
            .sum::<u64>();
        if linked_refund_total != record.refunded_amount_minor {
            bail!(
                "commerce payment attempt {} refunded_amount_minor does not match linked refunds total",
                record.payment_attempt_id
            );
        }
        let order = commerce_orders
            .get(record.order_id.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "bootstrap commerce_payment_attempts.order_id record {} references missing {}",
                    record.payment_attempt_id,
                    record.order_id
                )
            })?;
        if order.project_id != record.project_id || order.user_id != record.user_id {
            bail!(
                "commerce payment attempt {} references order {} with mismatched project or user",
                record.payment_attempt_id,
                record.order_id
            );
        }
        if order.currency_code != record.currency_code {
            bail!(
                "commerce payment attempt {} currency does not match order {}",
                record.payment_attempt_id,
                record.order_id
            );
        }
        if order
            .payment_method_id
            .as_deref()
            .is_some_and(|payment_method_id| payment_method_id != record.payment_method_id)
        {
            bail!(
                "commerce payment attempt {} uses payment method {} that does not match order {}",
                record.payment_attempt_id,
                record.payment_method_id,
                record.order_id
            );
        }
        let payment_method = payment_methods
            .get(record.payment_method_id.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "bootstrap commerce_payment_attempts.payment_method_id record {} references missing {}",
                    record.payment_attempt_id,
                    record.payment_method_id
                )
            })?;
        if payment_method.provider != record.provider || payment_method.channel != record.channel {
            bail!(
                "commerce payment attempt {} is not compatible with payment method {}",
                record.payment_attempt_id,
                record.payment_method_id
            );
        }
    }

    for record in &data.commerce_webhook_inbox_records {
        ensure_non_empty_field(
            "commerce_webhook_inbox_records.provider",
            &record.webhook_inbox_id,
            &record.provider,
        )?;
        ensure_non_empty_field(
            "commerce_webhook_inbox_records.dedupe_key",
            &record.webhook_inbox_id,
            &record.dedupe_key,
        )?;
        ensure_non_empty_field(
            "commerce_webhook_inbox_records.processing_status",
            &record.webhook_inbox_id,
            &record.processing_status,
        )?;
        if record.processing_status == "processed" && record.processed_at_ms.is_none() {
            bail!(
                "commerce webhook inbox {} with processing_status processed must declare processed_at_ms",
                record.webhook_inbox_id
            );
        }
        ensure_json_value_from_str(
            "commerce_webhook_inbox_records.payload_json",
            &record.webhook_inbox_id,
            &record.payload_json,
        )?;
        if let Some(payment_method_id) = record.payment_method_id.as_deref() {
            ensure_reference(
                "commerce_webhook_inbox_records.payment_method_id",
                &record.webhook_inbox_id,
                payment_method_id,
                &payment_method_ids,
            )?;
            let payment_method = payment_methods.get(payment_method_id).ok_or_else(|| {
                anyhow::anyhow!(
                    "bootstrap commerce_webhook_inbox_records.payment_method_id record {} references missing {}",
                    record.webhook_inbox_id,
                    payment_method_id
                )
            })?;
            if payment_method.provider != record.provider {
                bail!(
                    "commerce webhook inbox {} provider does not match payment method {}",
                    record.webhook_inbox_id,
                    payment_method_id
                );
            }
        }
        if let Some(provider_event_id) = record.provider_event_id.as_deref() {
            ensure_non_empty_field(
                "commerce_webhook_inbox_records.provider_event_id",
                &record.webhook_inbox_id,
                provider_event_id,
            )?;
            for payment_event in data.commerce_payment_events.iter().filter(|payment_event| {
                payment_event.provider_event_id.as_deref() == Some(provider_event_id)
            }) {
                if payment_event.provider != record.provider {
                    bail!(
                        "commerce webhook inbox {} provider does not match payment event {} for provider_event_id {}",
                        record.webhook_inbox_id,
                        payment_event.payment_event_id,
                        provider_event_id
                    );
                }
                if payment_event.dedupe_key != record.dedupe_key {
                    bail!(
                        "commerce webhook inbox {} dedupe_key does not match payment event {} for provider_event_id {}",
                        record.webhook_inbox_id,
                        payment_event.payment_event_id,
                        provider_event_id
                    );
                }
                if let Some(webhook_payment_method_id) = record.payment_method_id.as_deref() {
                    let order = commerce_orders
                        .get(payment_event.order_id.as_str())
                        .ok_or_else(|| {
                            anyhow::anyhow!(
                                "bootstrap commerce_payment_events.order_id record {} references missing {}",
                                payment_event.payment_event_id,
                                payment_event.order_id
                            )
                        })?;
                    if order
                        .payment_method_id
                        .as_deref()
                        .is_some_and(|order_payment_method_id| order_payment_method_id != webhook_payment_method_id)
                    {
                        bail!(
                            "commerce webhook inbox {} payment method does not match order {} linked to payment event {}",
                            record.webhook_inbox_id,
                            payment_event.order_id,
                            payment_event.payment_event_id
                        );
                    }
                }
            }
        }
        if let Some(signature_header) = record.signature_header.as_deref() {
            ensure_non_empty_field(
                "commerce_webhook_inbox_records.signature_header",
                &record.webhook_inbox_id,
                signature_header,
            )?;
        }
        if record.last_received_at_ms < record.first_received_at_ms {
            bail!(
                "commerce webhook inbox {} has last_received_at_ms earlier than first_received_at_ms",
                record.webhook_inbox_id
            );
        }
        if let Some(next_retry_at_ms) = record.next_retry_at_ms {
            if next_retry_at_ms < record.last_received_at_ms {
                bail!(
                    "commerce webhook inbox {} has next_retry_at_ms earlier than last_received_at_ms",
                    record.webhook_inbox_id
                );
            }
        }
        if let Some(processed_at_ms) = record.processed_at_ms {
            if processed_at_ms < record.first_received_at_ms {
                bail!(
                    "commerce webhook inbox {} has processed_at_ms earlier than first_received_at_ms",
                    record.webhook_inbox_id
                );
            }
            if processed_at_ms < record.last_received_at_ms {
                bail!(
                    "commerce webhook inbox {} has processed_at_ms earlier than last_received_at_ms",
                    record.webhook_inbox_id
                );
            }
        }
    }

    for record in &data.commerce_refunds {
        ensure_reference(
            "commerce_refunds.order_id",
            &record.refund_id,
            &record.order_id,
            &commerce_order_ids,
        )?;
        ensure_non_empty_field(
            "commerce_refunds.provider",
            &record.refund_id,
            &record.provider,
        )?;
        ensure_non_empty_field(
            "commerce_refunds.idempotency_key",
            &record.refund_id,
            &record.idempotency_key,
        )?;
        ensure_non_empty_field("commerce_refunds.status", &record.refund_id, &record.status)?;
        if record.status == "succeeded" && record.completed_at_ms.is_none() {
            bail!(
                "commerce refund {} with status succeeded must declare completed_at_ms",
                record.refund_id
            );
        }
        ensure_non_empty_field(
            "commerce_refunds.currency_code",
            &record.refund_id,
            &record.currency_code,
        )?;
        ensure_json_value_from_str(
            "commerce_refunds.request_payload_json",
            &record.refund_id,
            &record.request_payload_json,
        )?;
        ensure_json_value_from_str(
            "commerce_refunds.response_payload_json",
            &record.refund_id,
            &record.response_payload_json,
        )?;
        if record.updated_at_ms < record.created_at_ms {
            bail!(
                "commerce refund {} has updated_at_ms earlier than created_at_ms",
                record.refund_id
            );
        }
        if let Some(completed_at_ms) = record.completed_at_ms {
            if completed_at_ms < record.created_at_ms {
                bail!(
                    "commerce refund {} has completed_at_ms earlier than created_at_ms",
                    record.refund_id
                );
            }
        }
        let order = commerce_orders
            .get(record.order_id.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "bootstrap commerce_refunds.order_id record {} references missing {}",
                    record.refund_id,
                    record.order_id
                )
            })?;
        if record.amount_minor > order.payable_price_cents {
            bail!(
                "commerce refund {} cannot exceed order {} payable amount",
                record.refund_id,
                record.order_id
            );
        }
        if order.currency_code != record.currency_code {
            bail!(
                "commerce refund {} currency does not match order {}",
                record.refund_id,
                record.order_id
            );
        }
        if let Some(payment_attempt_id) = record.payment_attempt_id.as_deref() {
            ensure_reference(
                "commerce_refunds.payment_attempt_id",
                &record.refund_id,
                payment_attempt_id,
                &commerce_payment_attempt_ids,
            )?;
            let attempt = commerce_payment_attempts
                .get(payment_attempt_id)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "bootstrap commerce_refunds.payment_attempt_id record {} references missing {}",
                        record.refund_id,
                        payment_attempt_id
                    )
                })?;
            if attempt.order_id != record.order_id {
                bail!(
                    "commerce refund {} references payment attempt {} for another order",
                    record.refund_id,
                    payment_attempt_id
                );
            }
            if attempt.provider != record.provider {
                bail!(
                    "commerce refund {} provider does not match payment attempt {}",
                    record.refund_id,
                    payment_attempt_id
                );
            }
        }
        if let Some(payment_method_id) = record.payment_method_id.as_deref() {
            ensure_reference(
                "commerce_refunds.payment_method_id",
                &record.refund_id,
                payment_method_id,
                &payment_method_ids,
            )?;
            let payment_method = payment_methods.get(payment_method_id).ok_or_else(|| {
                anyhow::anyhow!(
                    "bootstrap commerce_refunds.payment_method_id record {} references missing {}",
                    record.refund_id,
                    payment_method_id
                )
            })?;
            if payment_method.provider != record.provider {
                bail!(
                    "commerce refund {} provider does not match payment method {}",
                    record.refund_id,
                    payment_method_id
                );
            }
            if let Some(payment_attempt_id) = record.payment_attempt_id.as_deref() {
                let attempt = commerce_payment_attempts
                    .get(payment_attempt_id)
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "bootstrap commerce_refunds.payment_attempt_id record {} references missing {}",
                            record.refund_id,
                            payment_attempt_id
                        )
                    })?;
                if attempt.payment_method_id != payment_method_id {
                    bail!(
                        "commerce refund {} payment method does not match payment attempt {}",
                        record.refund_id,
                        payment_attempt_id
                    );
                }
            }
        }
    }

    for record in &data.commerce_reconciliation_runs {
        ensure_non_empty_field(
            "commerce_reconciliation_runs.provider",
            &record.reconciliation_run_id,
            &record.provider,
        )?;
        ensure_non_empty_field(
            "commerce_reconciliation_runs.status",
            &record.reconciliation_run_id,
            &record.status,
        )?;
        if record.status == "completed" && record.completed_at_ms.is_none() {
            bail!(
                "commerce reconciliation run {} with status completed must declare completed_at_ms",
                record.reconciliation_run_id
            );
        }
        ensure_json_object_from_str(
            "commerce_reconciliation_runs.summary_json",
            &record.reconciliation_run_id,
            &record.summary_json,
        )?;
        if record.scope_ended_at_ms < record.scope_started_at_ms {
            bail!(
                "commerce reconciliation run {} has scope_ended_at_ms earlier than scope_started_at_ms",
                record.reconciliation_run_id
            );
        }
        if record.updated_at_ms < record.created_at_ms {
            bail!(
                "commerce reconciliation run {} has updated_at_ms earlier than created_at_ms",
                record.reconciliation_run_id
            );
        }
        if let Some(completed_at_ms) = record.completed_at_ms {
            if completed_at_ms < record.created_at_ms {
                bail!(
                    "commerce reconciliation run {} has completed_at_ms earlier than created_at_ms",
                    record.reconciliation_run_id
                );
            }
        }
        if let Some(payment_method_id) = record.payment_method_id.as_deref() {
            ensure_reference(
                "commerce_reconciliation_runs.payment_method_id",
                &record.reconciliation_run_id,
                payment_method_id,
                &payment_method_ids,
            )?;
            let payment_method = payment_methods.get(payment_method_id).ok_or_else(|| {
                anyhow::anyhow!(
                    "bootstrap commerce_reconciliation_runs.payment_method_id record {} references missing {}",
                    record.reconciliation_run_id,
                    payment_method_id
                )
            })?;
            if payment_method.provider != record.provider {
                bail!(
                    "commerce reconciliation run {} provider does not match payment method {}",
                    record.reconciliation_run_id,
                    payment_method_id
                );
            }
        }
    }

    for record in &data.commerce_reconciliation_items {
        ensure_reference(
            "commerce_reconciliation_items.reconciliation_run_id",
            &record.reconciliation_item_id,
            &record.reconciliation_run_id,
            &commerce_reconciliation_run_ids,
        )?;
        let reconciliation_run = commerce_reconciliation_runs
            .get(record.reconciliation_run_id.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "bootstrap commerce_reconciliation_items.reconciliation_run_id record {} references missing {}",
                    record.reconciliation_item_id,
                    record.reconciliation_run_id
                )
            })?;
        ensure_non_empty_field(
            "commerce_reconciliation_items.discrepancy_type",
            &record.reconciliation_item_id,
            &record.discrepancy_type,
        )?;
        ensure_non_empty_field(
            "commerce_reconciliation_items.status",
            &record.reconciliation_item_id,
            &record.status,
        )?;
        ensure_json_object_from_str(
            "commerce_reconciliation_items.detail_json",
            &record.reconciliation_item_id,
            &record.detail_json,
        )?;
        if record.updated_at_ms < record.created_at_ms {
            bail!(
                "commerce reconciliation item {} has updated_at_ms earlier than created_at_ms",
                record.reconciliation_item_id
            );
        }
        if record.order_id.is_none()
            && record.payment_attempt_id.is_none()
            && record.refund_id.is_none()
            && record.external_reference.is_none()
        {
            bail!(
                "commerce reconciliation item {} must reference at least one business object",
                record.reconciliation_item_id
            );
        }
        if let Some(order_id) = record.order_id.as_deref() {
            ensure_reference(
                "commerce_reconciliation_items.order_id",
                &record.reconciliation_item_id,
                order_id,
                &commerce_order_ids,
            )?;
        }
        if let Some(payment_attempt_id) = record.payment_attempt_id.as_deref() {
            ensure_reference(
                "commerce_reconciliation_items.payment_attempt_id",
                &record.reconciliation_item_id,
                payment_attempt_id,
                &commerce_payment_attempt_ids,
            )?;
            let attempt = commerce_payment_attempts
                .get(payment_attempt_id)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "bootstrap commerce_reconciliation_items.payment_attempt_id record {} references missing {}",
                        record.reconciliation_item_id,
                        payment_attempt_id
                    )
                })?;
            if let Some(order_id) = record.order_id.as_deref() {
                if attempt.order_id != order_id {
                    bail!(
                        "commerce reconciliation item {} references payment attempt {} for another order",
                        record.reconciliation_item_id,
                        payment_attempt_id
                    );
                }
            }
            if attempt.provider != reconciliation_run.provider {
                bail!(
                    "commerce reconciliation item {} references payment attempt {} outside reconciliation run {} provider context",
                    record.reconciliation_item_id,
                    payment_attempt_id,
                    record.reconciliation_run_id
                );
            }
            if reconciliation_run
                .payment_method_id
                .as_deref()
                .is_some_and(|payment_method_id| attempt.payment_method_id != payment_method_id)
            {
                bail!(
                    "commerce reconciliation item {} references payment attempt {} whose payment method does not match reconciliation run {}",
                    record.reconciliation_item_id,
                    payment_attempt_id,
                    record.reconciliation_run_id
                );
            }
            if attempt.updated_at_ms < reconciliation_run.scope_started_at_ms
                || attempt.updated_at_ms > reconciliation_run.scope_ended_at_ms
            {
                bail!(
                    "commerce reconciliation item {} references payment attempt {} outside reconciliation run {} scope",
                    record.reconciliation_item_id,
                    payment_attempt_id,
                    record.reconciliation_run_id
                );
            }
        }
        if let Some(refund_id) = record.refund_id.as_deref() {
            ensure_reference(
                "commerce_reconciliation_items.refund_id",
                &record.reconciliation_item_id,
                refund_id,
                &commerce_refund_ids,
            )?;
            let refund = commerce_refunds.get(refund_id).ok_or_else(|| {
                anyhow::anyhow!(
                    "bootstrap commerce_reconciliation_items.refund_id record {} references missing {}",
                    record.reconciliation_item_id,
                    refund_id
                )
            })?;
            if let Some(order_id) = record.order_id.as_deref() {
                if refund.order_id != order_id {
                    bail!(
                        "commerce reconciliation item {} references refund {} for another order",
                        record.reconciliation_item_id,
                        refund_id
                    );
                }
            }
            if let Some(payment_attempt_id) = record.payment_attempt_id.as_deref() {
                if refund.payment_attempt_id.as_deref() != Some(payment_attempt_id) {
                    bail!(
                        "commerce reconciliation item {} references refund {} that is not linked to payment attempt {}",
                        record.reconciliation_item_id,
                        refund_id,
                        payment_attempt_id
                    );
                }
            }
            if refund.provider != reconciliation_run.provider {
                bail!(
                    "commerce reconciliation item {} references refund {} outside reconciliation run {} provider context",
                    record.reconciliation_item_id,
                    refund_id,
                    record.reconciliation_run_id
                );
            }
            if refund.created_at_ms < reconciliation_run.scope_started_at_ms
                || refund.created_at_ms > reconciliation_run.scope_ended_at_ms
            {
                bail!(
                    "commerce reconciliation item {} references refund {} outside reconciliation run {} scope",
                    record.reconciliation_item_id,
                    refund_id,
                    record.reconciliation_run_id
                );
            }
            if reconciliation_run
                .payment_method_id
                .as_deref()
                .is_some_and(|payment_method_id| refund.payment_method_id.as_deref() != Some(payment_method_id))
            {
                bail!(
                    "commerce reconciliation item {} references refund {} whose payment method does not match reconciliation run {}",
                    record.reconciliation_item_id,
                    refund_id,
                    record.reconciliation_run_id
                );
            }
        }
        if let Some(external_reference) = record.external_reference.as_deref() {
            ensure_non_empty_field(
                "commerce_reconciliation_items.external_reference",
                &record.reconciliation_item_id,
                external_reference,
            )?;
            let mut matched_payment_event = false;
            for payment_event in data
                .commerce_payment_events
                .iter()
                .filter(|payment_event| payment_event.dedupe_key == external_reference)
            {
                matched_payment_event = true;
                if payment_event.provider != reconciliation_run.provider {
                    bail!(
                        "commerce reconciliation item {} external_reference {} resolves to payment event {} outside reconciliation run {} provider context",
                        record.reconciliation_item_id,
                        external_reference,
                        payment_event.payment_event_id,
                        record.reconciliation_run_id
                    );
                }
                if payment_event.received_at_ms < reconciliation_run.scope_started_at_ms
                    || payment_event.received_at_ms > reconciliation_run.scope_ended_at_ms
                {
                    bail!(
                        "commerce reconciliation item {} external_reference {} resolves to payment event {} outside reconciliation run {} scope",
                        record.reconciliation_item_id,
                        external_reference,
                        payment_event.payment_event_id,
                        record.reconciliation_run_id
                    );
                }
            }
            if !matched_payment_event {
                bail!(
                    "commerce reconciliation item {} external_reference {} does not resolve to any payment event",
                    record.reconciliation_item_id,
                    external_reference
                );
            }
        }
    }

    for record in &data.async_jobs {
        ensure_non_empty_field("async_jobs.job_id", &record.job_id, &record.job_id)?;
        ensure_non_empty_field(
            "async_jobs.capability_code",
            &record.job_id,
            &record.capability_code,
        )?;
        ensure_non_empty_field("async_jobs.modality", &record.job_id, &record.modality)?;
        ensure_non_empty_field(
            "async_jobs.operation_kind",
            &record.job_id,
            &record.operation_kind,
        )?;
        if record.progress_percent.is_some_and(|value| value > 100) {
            bail!(
                "bootstrap async_jobs.progress_percent record {} must be between 0 and 100",
                record.job_id
            );
        }
        if record.status == AsyncJobStatus::Succeeded && record.started_at_ms.is_none() {
            bail!(
                "async job {} with status succeeded must declare started_at_ms",
                record.job_id
            );
        }
        if record.status == AsyncJobStatus::Succeeded && record.completed_at_ms.is_none() {
            bail!(
                "async job {} with status succeeded must declare completed_at_ms",
                record.job_id
            );
        }
        if record.updated_at_ms < record.created_at_ms {
            bail!(
                "async job {} has updated_at_ms earlier than created_at_ms",
                record.job_id
            );
        }
        if let Some(started_at_ms) = record.started_at_ms {
            if started_at_ms < record.created_at_ms {
                bail!(
                    "async job {} has started_at_ms earlier than created_at_ms",
                    record.job_id
                );
            }
            if let Some(completed_at_ms) = record.completed_at_ms {
                if completed_at_ms < started_at_ms {
                    bail!(
                        "async job {} has completed_at_ms earlier than started_at_ms",
                        record.job_id
                    );
                }
            }
        }
        if let Some(provider_id) = record.provider_id.as_deref() {
            ensure_reference(
                "async_jobs.provider_id",
                &record.job_id,
                provider_id,
                &provider_ids,
            )?;
            ensure_provider_has_enabled_account(
                "async_jobs.provider_id",
                &record.job_id,
                provider_id,
                &executable_provider_account_provider_ids,
            )?;
        }
        if let Some(account_id) = record.account_id {
            ensure_reference(
                "async_jobs.account_id",
                &record.job_id,
                &account_id.to_string(),
                &account_ids,
            )?;
            let account = accounts_by_id.get(&account_id).ok_or_else(|| {
                anyhow::anyhow!(
                    "bootstrap async_jobs.account_id record {} references missing {}",
                    record.job_id,
                    account_id
                )
            })?;
            if account.tenant_id != record.tenant_id
                || account.organization_id != record.organization_id
                || account.user_id != record.user_id
            {
                bail!(
                    "async job {} must match tenant/organization/user ownership of account {}",
                    record.job_id,
                    account_id
                );
            }
        }
        if let Some(model_code) = record.model_code.as_deref() {
            ensure_reference(
                "async_jobs.model_code",
                &record.job_id,
                model_code,
                &model_ids,
            )?;
            if let Some(provider_id) = record.provider_id.as_deref() {
                ensure_reference(
                    "async_jobs.model_code+provider_id",
                    &record.job_id,
                    &format!("{model_code}::{provider_id}"),
                    &model_variant_keys,
                )?;
                let model = data
                    .models
                    .iter()
                    .find(|model| {
                        model.provider_id == provider_id && model.external_name == model_code
                    })
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "bootstrap async_jobs.model_code record {} references model {} without provider model metadata on provider {}",
                            record.job_id,
                            model_code,
                            provider_id
                        )
                    })?;
                let declared_capability = serde_json::from_value::<
                    sdkwork_api_domain_catalog::ModelCapability,
                >(serde_json::Value::String(record.capability_code.clone()))
                .with_context(|| {
                    format!(
                        "bootstrap async_jobs.capability_code record {} contains unsupported capability {}",
                        record.job_id, record.capability_code
                    )
                })?;
                if !model
                    .capabilities
                    .iter()
                    .any(|capability| capability == &declared_capability)
                {
                    bail!(
                        "async job {} declares capability {} not supported by model {} on provider {}",
                        record.job_id,
                        record.capability_code,
                        model_code,
                        provider_id
                    );
                }
            }
        }
        if let Some(idempotency_key) = record.idempotency_key.as_deref() {
            ensure_non_empty_field(
                "async_jobs.idempotency_key",
                &record.job_id,
                idempotency_key,
            )?;
            if !async_job_idempotency_keys.insert(idempotency_key.to_owned()) {
                bail!(
                    "bootstrap async_jobs.idempotency_key contains duplicate entry {}",
                    idempotency_key
                );
            }
        }
        if let Some(callback_url) = record.callback_url.as_deref() {
            ensure_non_empty_field("async_jobs.callback_url", &record.job_id, callback_url)?;
        }
    }

    for record in &data.async_job_attempts {
        ensure_reference(
            "async_job_attempts.job_id",
            &record.attempt_id.to_string(),
            &record.job_id,
            &async_job_ids,
        )?;
        let job = async_jobs.get(record.job_id.as_str()).ok_or_else(|| {
            anyhow::anyhow!(
                "bootstrap async_job_attempts.job_id record {} references missing {}",
                record.attempt_id,
                record.job_id
            )
        })?;
        ensure_non_empty_field(
            "async_job_attempts.runtime_kind",
            &record.attempt_id.to_string(),
            &record.runtime_kind,
        )?;
        if let Some(endpoint) = record.endpoint.as_deref() {
            ensure_non_empty_field(
                "async_job_attempts.endpoint",
                &record.attempt_id.to_string(),
                endpoint,
            )?;
        }
        if record.updated_at_ms < record.created_at_ms {
            bail!(
                "async job attempt {} has updated_at_ms earlier than created_at_ms",
                record.attempt_id
            );
        }
        if record.status == AsyncJobAttemptStatus::Succeeded && record.claimed_at_ms.is_none() {
            bail!(
                "async job attempt {} with status succeeded must declare claimed_at_ms",
                record.attempt_id
            );
        }
        if record.status == AsyncJobAttemptStatus::Succeeded && record.finished_at_ms.is_none() {
            bail!(
                "async job attempt {} with status succeeded must declare finished_at_ms",
                record.attempt_id
            );
        }
        if record.created_at_ms < job.created_at_ms {
            bail!(
                "async job attempt {} has created_at_ms earlier than parent job {} created_at_ms",
                record.attempt_id,
                job.job_id
            );
        }
        if let Some(attempt_external_job_id) = record.external_job_id.as_deref() {
            ensure_non_empty_field(
                "async_job_attempts.external_job_id",
                &record.attempt_id.to_string(),
                attempt_external_job_id,
            )?;
            if let Some(job_external_job_id) = job.external_job_id.as_deref() {
                if attempt_external_job_id != job_external_job_id {
                    bail!(
                        "async job attempt {} external job id must match parent job {}",
                        record.attempt_id,
                        job.job_id
                    );
                }
            }
        }
        if let Some(claimed_at_ms) = record.claimed_at_ms {
            if claimed_at_ms < record.created_at_ms {
                bail!(
                    "async job attempt {} has claimed_at_ms earlier than created_at_ms",
                    record.attempt_id
                );
            }
            if let Some(finished_at_ms) = record.finished_at_ms {
                if finished_at_ms < claimed_at_ms {
                    bail!(
                        "async job attempt {} has finished_at_ms earlier than claimed_at_ms",
                        record.attempt_id
                    );
                }
            }
        }
        ensure_async_job_attempt_fits_parent_lifecycle(record, job)?;
    }

    for record in &data.async_job_assets {
        ensure_reference(
            "async_job_assets.job_id",
            &record.asset_id,
            &record.job_id,
            &async_job_ids,
        )?;
        ensure_non_empty_field(
            "async_job_assets.asset_kind",
            &record.asset_id,
            &record.asset_kind,
        )?;
        ensure_non_empty_field(
            "async_job_assets.storage_key",
            &record.asset_id,
            &record.storage_key,
        )?;
        let job = async_jobs.get(record.job_id.as_str()).ok_or_else(|| {
            anyhow::anyhow!(
                "bootstrap async_job_assets.job_id record {} references missing {}",
                record.asset_id,
                record.job_id
            )
        })?;
        ensure_async_job_asset_fits_parent_scope(record, job)?;
        if let Some(download_url) = record.download_url.as_deref() {
            ensure_non_empty_field(
                "async_job_assets.download_url",
                &record.asset_id,
                download_url,
            )?;
            if !download_url.contains(&record.job_id) {
                bail!(
                    "async job asset {} download_url must contain parent job {}",
                    record.asset_id,
                    record.job_id
                );
            }
        }
        if let Some(checksum_sha256) = record.checksum_sha256.as_deref() {
            ensure_non_empty_field(
                "async_job_assets.checksum_sha256",
                &record.asset_id,
                checksum_sha256,
            )?;
        }
    }

    for record in &data.async_job_callbacks {
        ensure_reference(
            "async_job_callbacks.job_id",
            &record.callback_id.to_string(),
            &record.job_id,
            &async_job_ids,
        )?;
        ensure_non_empty_field(
            "async_job_callbacks.event_type",
            &record.callback_id.to_string(),
            &record.event_type,
        )?;
        ensure_json_value_from_str(
            "async_job_callbacks.payload_json",
            &record.callback_id.to_string(),
            &record.payload_json,
        )?;
        if let Some(dedupe_key) = record.dedupe_key.as_deref() {
            ensure_non_empty_field(
                "async_job_callbacks.dedupe_key",
                &record.callback_id.to_string(),
                dedupe_key,
            )?;
            if !async_job_callback_dedupe_keys.insert(dedupe_key.to_owned()) {
                bail!(
                    "bootstrap async_job_callbacks.dedupe_key contains duplicate entry {}",
                    dedupe_key
                );
            }
        }
        if record.status == AsyncJobCallbackStatus::Processed && record.processed_at_ms.is_none() {
            bail!(
                "async job callback {} with status processed must declare processed_at_ms",
                record.callback_id
            );
        }
        if let Some(processed_at_ms) = record.processed_at_ms {
            if processed_at_ms < record.received_at_ms {
                bail!(
                    "async job callback {} has processed_at_ms earlier than received_at_ms",
                    record.callback_id
                );
            }
        }
        let job = async_jobs.get(record.job_id.as_str()).ok_or_else(|| {
            anyhow::anyhow!(
                "bootstrap async_job_callbacks.job_id record {} references missing {}",
                record.callback_id,
                record.job_id
            )
        })?;
        let payload =
            serde_json::from_str::<serde_json::Value>(&record.payload_json).with_context(|| {
                format!(
                    "bootstrap async_job_callbacks.payload_json record {} must contain valid json",
                    record.callback_id
                )
            })?;
        if let Some(payload_job_id) = payload.get("job_id").and_then(serde_json::Value::as_str) {
            if payload_job_id != job.job_id {
                bail!(
                    "async job callback {} payload job_id must match parent job {}",
                    record.callback_id,
                    job.job_id
                );
            }
        }
        if let Some(external_job_id) = job.external_job_id.as_deref() {
            if let Some(dedupe_key) = record.dedupe_key.as_deref() {
                if !dedupe_key.contains(external_job_id) && !dedupe_key.contains(&job.job_id) {
                    bail!(
                        "async job callback {} dedupe_key must contain the parent job id or external job id",
                        record.callback_id
                    );
                }
            }
        }
        ensure_async_job_callback_fits_parent_lifecycle(record, job)?;
        ensure_async_job_callback_payload_matches_parent(record, job, &payload)?;
    }

    for record in &data.marketing_campaigns {
        ensure_reference(
            "marketing_campaigns.coupon_template_id",
            &record.marketing_campaign_id,
            &record.coupon_template_id,
            &coupon_template_ids,
        )?;
    }

    for record in &data.campaign_budgets {
        ensure_reference(
            "campaign_budgets.marketing_campaign_id",
            &record.campaign_budget_id,
            &record.marketing_campaign_id,
            &marketing_campaign_ids,
        )?;
    }

    for record in &data.coupon_codes {
        ensure_reference(
            "coupon_codes.coupon_template_id",
            &record.coupon_code_id,
            &record.coupon_template_id,
            &coupon_template_ids,
        )?;
    }

    Ok(())
}

fn validate_account_kernel_bootstrap_data(
    data: &BootstrapDataPack,
    account_ids: &HashSet<String>,
    channel_ids: &HashSet<String>,
    provider_ids: &HashSet<String>,
    project_ids: &HashSet<String>,
    gateway_api_key_hashes: &HashSet<String>,
    pricing_plan_ids: &HashSet<String>,
    active_pricing_plan_ids: &HashSet<String>,
    pricing_plans_by_id: &HashMap<String, &PricingPlanRecord>,
    available_channel_model_variants: &HashSet<String>,
    provider_channels: &HashMap<String, HashSet<String>>,
    commerce_order_ids: &HashSet<String>,
    executable_provider_account_provider_ids: &HashSet<String>,
) -> Result<()> {
    let epsilon = 1e-6_f64;
    let accounts = data
        .accounts
        .iter()
        .map(|record| (record.account_id, record))
        .collect::<HashMap<_, _>>();
    let account_benefit_lots = data
        .account_benefit_lots
        .iter()
        .map(|record| (record.lot_id, record))
        .collect::<HashMap<_, _>>();
    let account_holds = data
        .account_holds
        .iter()
        .map(|record| (record.hold_id, record))
        .collect::<HashMap<_, _>>();
    let request_meter_facts = data
        .request_meter_facts
        .iter()
        .map(|record| (record.request_id, record))
        .collect::<HashMap<_, _>>();
    let active_model_price_keys = collect_ids(
        data.model_prices
            .iter()
            .filter(|record| record.is_active)
            .map(|record| {
                format!(
                    "{}::{}::{}",
                    record.proxy_provider_id, record.channel_id, record.model_id
                )
            }),
    );
    let mut request_metric_token_summaries = HashMap::<u64, (u64, f64, u64, f64)>::new();
    for record in &data.request_meter_metrics {
        let summary = request_metric_token_summaries
            .entry(record.request_id)
            .or_insert((0, 0.0, 0, 0.0));
        match record.metric_code.as_str() {
            "token.input" => {
                summary.0 += 1;
                summary.1 += record.quantity;
            }
            "token.output" => {
                summary.2 += 1;
                summary.3 += record.quantity;
            }
            _ => {}
        }
    }
    let mut billing_events_by_reference = HashMap::<&str, Vec<&BillingEventRecord>>::new();
    for record in &data.billing_events {
        if let Some(reference_id) = record.reference_id.as_deref() {
            billing_events_by_reference
                .entry(reference_id)
                .or_default()
                .push(record);
        }
    }
    let commerce_orders = data
        .commerce_orders
        .iter()
        .map(|record| (record.order_id.as_str(), record))
        .collect::<HashMap<_, _>>();
    let mut hold_allocation_totals = HashMap::<u64, (f64, f64, f64)>::new();
    let mut ledger_allocation_totals = HashMap::<u64, f64>::new();
    let known_request_ids = collect_ids(
        data.account_holds
            .iter()
            .map(|record| record.request_id.to_string())
            .chain(
                data.request_settlements
                    .iter()
                    .map(|record| record.request_id.to_string()),
            )
            .chain(
                data.request_meter_facts
                    .iter()
                    .map(|record| record.request_id.to_string()),
            ),
    );

    for record in &data.accounts {
        let record_id = record.account_id.to_string();
        ensure_non_empty_field("accounts.currency_code", &record_id, &record.currency_code)?;
        ensure_non_empty_field(
            "accounts.credit_unit_code",
            &record_id,
            &record.credit_unit_code,
        )?;
        if !record.overdraft_limit.is_finite() || record.overdraft_limit < 0.0 {
            bail!(
                "bootstrap accounts.overdraft_limit record {} must be a non-negative finite value",
                record.account_id
            );
        }
        if record.updated_at_ms < record.created_at_ms {
            bail!(
                "account {} has updated_at_ms earlier than created_at_ms",
                record.account_id
            );
        }
    }

    for record in &data.account_benefit_lots {
        let record_id = record.lot_id.to_string();
        ensure_reference(
            "account_benefit_lots.account_id",
            &record_id,
            &record.account_id.to_string(),
            account_ids,
        )?;
        let account = accounts.get(&record.account_id).ok_or_else(|| {
            anyhow::anyhow!(
                "bootstrap account_benefit_lots.account_id record {} references missing {}",
                record.lot_id,
                record.account_id
            )
        })?;
        if account.tenant_id != record.tenant_id
            || account.organization_id != record.organization_id
            || account.user_id != record.user_id
        {
            bail!(
                "account benefit lot {} must match tenant/organization/user ownership of account {}",
                record.lot_id,
                record.account_id
            );
        }
        if !record.original_quantity.is_finite()
            || !record.remaining_quantity.is_finite()
            || !record.held_quantity.is_finite()
            || record.original_quantity < 0.0
            || record.remaining_quantity < 0.0
            || record.held_quantity < 0.0
        {
            bail!(
                "account benefit lot {} quantities must be finite non-negative values",
                record.lot_id
            );
        }
        if record
            .acquired_unit_cost
            .is_some_and(|value| !value.is_finite() || value < 0.0)
        {
            bail!(
                "account benefit lot {} acquired_unit_cost must be a finite non-negative value",
                record.lot_id
            );
        }
        if record.remaining_quantity + epsilon < record.held_quantity {
            bail!(
                "account benefit lot {} cannot hold more quantity than remains",
                record.lot_id
            );
        }
        if record.updated_at_ms < record.created_at_ms {
            bail!(
                "account benefit lot {} has updated_at_ms earlier than created_at_ms",
                record.lot_id
            );
        }
        if let Some(expires_at_ms) = record.expires_at_ms {
            if expires_at_ms < record.issued_at_ms {
                bail!(
                    "account benefit lot {} has expires_at_ms earlier than issued_at_ms",
                    record.lot_id
                );
            }
        }
        if let Some(scope_json) = record.scope_json.as_deref() {
            ensure_json_object_from_str("account_benefit_lots.scope_json", &record_id, scope_json)?;
        }
    }

    for record in &data.account_holds {
        let record_id = record.hold_id.to_string();
        ensure_reference(
            "account_holds.account_id",
            &record_id,
            &record.account_id.to_string(),
            account_ids,
        )?;
        let account = accounts.get(&record.account_id).ok_or_else(|| {
            anyhow::anyhow!(
                "bootstrap account_holds.account_id record {} references missing {}",
                record.hold_id,
                record.account_id
            )
        })?;
        if account.tenant_id != record.tenant_id
            || account.organization_id != record.organization_id
            || account.user_id != record.user_id
        {
            bail!(
                "account hold {} must match tenant/organization/user ownership of account {}",
                record.hold_id,
                record.account_id
            );
        }
        let fact = request_meter_facts.get(&record.request_id).ok_or_else(|| {
            anyhow::anyhow!(
                "bootstrap account_holds.request_id record {} references missing request meter fact {}",
                record.hold_id,
                record.request_id
            )
        })?;
        if fact.tenant_id != record.tenant_id
            || fact.organization_id != record.organization_id
            || fact.user_id != record.user_id
            || fact.account_id != record.account_id
        {
            bail!(
                "account hold {} must match tenant/organization/user/account ownership of request meter fact {}",
                record.hold_id,
                record.request_id
            );
        }
        if !record.estimated_quantity.is_finite()
            || !record.captured_quantity.is_finite()
            || !record.released_quantity.is_finite()
            || record.estimated_quantity < 0.0
            || record.captured_quantity < 0.0
            || record.released_quantity < 0.0
        {
            bail!(
                "account hold {} quantities must be finite non-negative values",
                record.hold_id
            );
        }
        if record.captured_quantity + record.released_quantity > record.estimated_quantity + epsilon
        {
            bail!(
                "account hold {} captured/released quantities exceed the estimated quantity",
                record.hold_id
            );
        }
        if (fact.estimated_credit_hold - record.estimated_quantity).abs() > epsilon {
            bail!(
                "account hold {} estimated_quantity does not match request meter fact {}",
                record.hold_id,
                record.request_id
            );
        }
        match record.status {
            AccountHoldStatus::Held => {
                if record.captured_quantity > epsilon || record.released_quantity > epsilon {
                    bail!(
                        "account hold {} status held must not contain realized quantities",
                        record.hold_id
                    );
                }
                if fact.usage_capture_status
                    != sdkwork_api_domain_usage::UsageCaptureStatus::Estimated
                {
                    bail!(
                        "account hold {} status held requires request meter fact {} usage_capture_status estimated",
                        record.hold_id,
                        record.request_id
                    );
                }
            }
            AccountHoldStatus::PartiallyReleased => {
                if record.captured_quantity <= epsilon {
                    bail!(
                        "account hold {} status partially_released requires captured_quantity",
                        record.hold_id
                    );
                }
                if record.released_quantity <= epsilon {
                    bail!(
                        "account hold {} status partially_released requires released_quantity",
                        record.hold_id
                    );
                }
                if !matches!(
                    fact.usage_capture_status,
                    sdkwork_api_domain_usage::UsageCaptureStatus::Captured
                        | sdkwork_api_domain_usage::UsageCaptureStatus::Reconciled
                ) {
                    bail!(
                        "account hold {} status partially_released requires request meter fact {} usage_capture_status captured or reconciled",
                        record.hold_id,
                        record.request_id
                    );
                }
            }
            _ => {}
        }
        if record.created_at_ms < fact.started_at_ms {
            bail!(
                "account hold {} has created_at_ms earlier than request meter fact {} started_at_ms",
                record.hold_id,
                record.request_id
            );
        }
        if record.status == AccountHoldStatus::PartiallyReleased {
            if let Some(finished_at_ms) = fact.finished_at_ms {
                if record.updated_at_ms < finished_at_ms {
                    bail!(
                        "account hold {} has updated_at_ms earlier than request meter fact {} finished_at_ms",
                        record.hold_id,
                        record.request_id
                    );
                }
            }
        }
        if record.expires_at_ms < record.created_at_ms {
            bail!(
                "account hold {} has expires_at_ms earlier than created_at_ms",
                record.hold_id
            );
        }
        if record.updated_at_ms < record.created_at_ms {
            bail!(
                "account hold {} has updated_at_ms earlier than created_at_ms",
                record.hold_id
            );
        }
    }

    for record in &data.account_hold_allocations {
        let record_id = record.hold_allocation_id.to_string();
        ensure_reference(
            "account_hold_allocations.hold_id",
            &record_id,
            &record.hold_id.to_string(),
            &collect_ids(
                data.account_holds
                    .iter()
                    .map(|hold| hold.hold_id.to_string()),
            ),
        )?;
        ensure_reference(
            "account_hold_allocations.lot_id",
            &record_id,
            &record.lot_id.to_string(),
            &collect_ids(
                data.account_benefit_lots
                    .iter()
                    .map(|lot| lot.lot_id.to_string()),
            ),
        )?;
        let hold = account_holds.get(&record.hold_id).ok_or_else(|| {
            anyhow::anyhow!(
                "bootstrap account_hold_allocations.hold_id record {} references missing {}",
                record.hold_allocation_id,
                record.hold_id
            )
        })?;
        let lot = account_benefit_lots.get(&record.lot_id).ok_or_else(|| {
            anyhow::anyhow!(
                "bootstrap account_hold_allocations.lot_id record {} references missing {}",
                record.hold_allocation_id,
                record.lot_id
            )
        })?;
        if hold.account_id != lot.account_id {
            bail!(
                "account hold allocation {} must reference hold and lot for the same account",
                record.hold_allocation_id
            );
        }
        if hold.tenant_id != record.tenant_id
            || hold.organization_id != record.organization_id
            || lot.tenant_id != record.tenant_id
            || lot.organization_id != record.organization_id
        {
            bail!(
                "account hold allocation {} must preserve tenant and organization ownership",
                record.hold_allocation_id
            );
        }
        if !record.allocated_quantity.is_finite()
            || !record.captured_quantity.is_finite()
            || !record.released_quantity.is_finite()
            || record.allocated_quantity < 0.0
            || record.captured_quantity < 0.0
            || record.released_quantity < 0.0
        {
            bail!(
                "account hold allocation {} quantities must be finite non-negative values",
                record.hold_allocation_id
            );
        }
        if record.captured_quantity + record.released_quantity > record.allocated_quantity + epsilon
        {
            bail!(
                "account hold allocation {} captured/released quantities exceed the allocated quantity",
                record.hold_allocation_id
            );
        }
        if record.updated_at_ms < record.created_at_ms {
            bail!(
                "account hold allocation {} has updated_at_ms earlier than created_at_ms",
                record.hold_allocation_id
            );
        }
        let totals = hold_allocation_totals
            .entry(record.hold_id)
            .or_insert((0.0, 0.0, 0.0));
        totals.0 += record.allocated_quantity;
        totals.1 += record.captured_quantity;
        totals.2 += record.released_quantity;
    }

    for record in &data.account_holds {
        if let Some((allocated, captured, released)) = hold_allocation_totals.get(&record.hold_id) {
            if (allocated - record.estimated_quantity).abs() > epsilon
                || (captured - record.captured_quantity).abs() > epsilon
                || (released - record.released_quantity).abs() > epsilon
            {
                bail!(
                    "account hold {} does not match the totals of its hold allocations",
                    record.hold_id
                );
            }
        }
    }

    for record in &data.account_ledger_entries {
        let record_id = record.ledger_entry_id.to_string();
        ensure_reference(
            "account_ledger_entries.account_id",
            &record_id,
            &record.account_id.to_string(),
            account_ids,
        )?;
        let account = accounts.get(&record.account_id).ok_or_else(|| {
            anyhow::anyhow!(
                "bootstrap account_ledger_entries.account_id record {} references missing {}",
                record.ledger_entry_id,
                record.account_id
            )
        })?;
        if account.tenant_id != record.tenant_id
            || account.organization_id != record.organization_id
            || account.user_id != record.user_id
        {
            bail!(
                "account ledger entry {} must match tenant/organization/user ownership of account {}",
                record.ledger_entry_id,
                record.account_id
            );
        }
        if !record.quantity.is_finite() || !record.amount.is_finite() {
            bail!(
                "account ledger entry {} quantity and amount must be finite values",
                record.ledger_entry_id
            );
        }
        if let Some(request_id) = record.request_id {
            ensure_reference(
                "account_ledger_entries.request_id",
                &record_id,
                &request_id.to_string(),
                &known_request_ids,
            )?;
        }
        if let Some(hold_id) = record.hold_id {
            ensure_reference(
                "account_ledger_entries.hold_id",
                &record_id,
                &hold_id.to_string(),
                &collect_ids(
                    data.account_holds
                        .iter()
                        .map(|hold| hold.hold_id.to_string()),
                ),
            )?;
            let hold = account_holds.get(&hold_id).ok_or_else(|| {
                anyhow::anyhow!(
                    "bootstrap account_ledger_entries.hold_id record {} references missing {}",
                    record.ledger_entry_id,
                    hold_id
                )
            })?;
            if hold.account_id != record.account_id {
                bail!(
                    "account ledger entry {} references hold {} on another account",
                    record.ledger_entry_id,
                    hold_id
                );
            }
        }
        if let Some(benefit_type) = record.benefit_type.as_deref() {
            ensure_non_empty_field(
                "account_ledger_entries.benefit_type",
                &record_id,
                benefit_type,
            )?;
        }
    }

    for record in &data.account_ledger_allocations {
        let record_id = record.ledger_allocation_id.to_string();
        ensure_reference(
            "account_ledger_allocations.ledger_entry_id",
            &record_id,
            &record.ledger_entry_id.to_string(),
            &collect_ids(
                data.account_ledger_entries
                    .iter()
                    .map(|entry| entry.ledger_entry_id.to_string()),
            ),
        )?;
        ensure_reference(
            "account_ledger_allocations.lot_id",
            &record_id,
            &record.lot_id.to_string(),
            &collect_ids(
                data.account_benefit_lots
                    .iter()
                    .map(|lot| lot.lot_id.to_string()),
            ),
        )?;
        let entry = data
            .account_ledger_entries
            .iter()
            .find(|entry| entry.ledger_entry_id == record.ledger_entry_id)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "bootstrap account_ledger_allocations.ledger_entry_id record {} references missing {}",
                    record.ledger_allocation_id,
                    record.ledger_entry_id
                )
            })?;
        let lot = account_benefit_lots.get(&record.lot_id).ok_or_else(|| {
            anyhow::anyhow!(
                "bootstrap account_ledger_allocations.lot_id record {} references missing {}",
                record.ledger_allocation_id,
                record.lot_id
            )
        })?;
        if entry.account_id != lot.account_id {
            bail!(
                "account ledger allocation {} must reference a ledger entry and lot for the same account",
                record.ledger_allocation_id
            );
        }
        if entry.tenant_id != record.tenant_id
            || entry.organization_id != record.organization_id
            || lot.tenant_id != record.tenant_id
            || lot.organization_id != record.organization_id
        {
            bail!(
                "account ledger allocation {} must preserve tenant and organization ownership",
                record.ledger_allocation_id
            );
        }
        if !record.quantity_delta.is_finite() {
            bail!(
                "account ledger allocation {} quantity_delta must be finite",
                record.ledger_allocation_id
            );
        }
        *ledger_allocation_totals
            .entry(record.ledger_entry_id)
            .or_insert(0.0) += record.quantity_delta;
    }

    for record in &data.account_ledger_entries {
        if let Some(quantity_delta) = ledger_allocation_totals.get(&record.ledger_entry_id) {
            if (quantity_delta - record.quantity).abs() > epsilon {
                bail!(
                    "account ledger entry {} does not match the totals of its ledger allocations",
                    record.ledger_entry_id
                );
            }
        }
    }

    for record in &data.request_meter_facts {
        let record_id = record.request_id.to_string();
        ensure_reference(
            "request_meter_facts.account_id",
            &record_id,
            &record.account_id.to_string(),
            account_ids,
        )?;
        let account = accounts.get(&record.account_id).ok_or_else(|| {
            anyhow::anyhow!(
                "bootstrap request_meter_facts.account_id record {} references missing {}",
                record.request_id,
                record.account_id
            )
        })?;
        if account.tenant_id != record.tenant_id
            || account.organization_id != record.organization_id
            || account.user_id != record.user_id
        {
            bail!(
                "request meter fact {} must match tenant/organization/user ownership of account {}",
                record.request_id,
                record.account_id
            );
        }
        ensure_non_empty_field(
            "request_meter_facts.auth_type",
            &record_id,
            &record.auth_type,
        )?;
        ensure_non_empty_field(
            "request_meter_facts.protocol_family",
            &record_id,
            &record.protocol_family,
        )?;
        ensure_non_empty_field(
            "request_meter_facts.capability_code",
            &record_id,
            &record.capability_code,
        )?;
        ensure_non_empty_field(
            "request_meter_facts.channel_code",
            &record_id,
            &record.channel_code,
        )?;
        ensure_non_empty_field(
            "request_meter_facts.model_code",
            &record_id,
            &record.model_code,
        )?;
        ensure_non_empty_field(
            "request_meter_facts.provider_code",
            &record_id,
            &record.provider_code,
        )?;
        ensure_reference(
            "request_meter_facts.channel_code",
            &record_id,
            &record.channel_code,
            channel_ids,
        )?;
        ensure_reference(
            "request_meter_facts.provider_code",
            &record_id,
            &record.provider_code,
            provider_ids,
        )?;
        ensure_provider_has_enabled_account(
            "request_meter_facts.provider_code",
            &record_id,
            &record.provider_code,
            &executable_provider_account_provider_ids,
        )?;
        ensure_active_model_price_coverage(
            "request_meter_facts.model_code",
            &record_id,
            &record.provider_code,
            &record.channel_code,
            &record.model_code,
            &active_model_price_keys,
        )?;
        if let Some(api_key_hash) = record.api_key_hash.as_deref() {
            ensure_reference(
                "request_meter_facts.api_key_hash",
                &record_id,
                api_key_hash,
                gateway_api_key_hashes,
            )?;
        }
        if let Some(cost_pricing_plan_id) = record.cost_pricing_plan_id {
            let pricing_plan_id = cost_pricing_plan_id.to_string();
            ensure_reference(
                "request_meter_facts.cost_pricing_plan_id",
                &record_id,
                &pricing_plan_id,
                pricing_plan_ids,
            )?;
            let pricing_plan = pricing_plans_by_id.get(&pricing_plan_id).ok_or_else(|| {
                anyhow::anyhow!(
                    "bootstrap request_meter_facts.cost_pricing_plan_id record {} references missing {}",
                    record.request_id,
                    pricing_plan_id
                )
            })?;
            if !active_pricing_plan_ids.contains(&pricing_plan_id) {
                bail!(
                    "request meter fact {} references inactive cost pricing plan {}",
                    record.request_id,
                    pricing_plan_id
                );
            }
            ensure_request_meter_fact_pricing_plan_workspace_or_shared(
                "request_meter_facts.cost_pricing_plan_id",
                record,
                pricing_plan,
            )?;
        }
        if let Some(retail_pricing_plan_id) = record.retail_pricing_plan_id {
            let pricing_plan_id = retail_pricing_plan_id.to_string();
            ensure_reference(
                "request_meter_facts.retail_pricing_plan_id",
                &record_id,
                &pricing_plan_id,
                pricing_plan_ids,
            )?;
            let pricing_plan = pricing_plans_by_id.get(&pricing_plan_id).ok_or_else(|| {
                anyhow::anyhow!(
                    "bootstrap request_meter_facts.retail_pricing_plan_id record {} references missing {}",
                    record.request_id,
                    pricing_plan_id
                )
            })?;
            if !active_pricing_plan_ids.contains(&pricing_plan_id) {
                bail!(
                    "request meter fact {} references inactive retail pricing plan {}",
                    record.request_id,
                    pricing_plan_id
                );
            }
            ensure_request_meter_fact_pricing_plan_workspace_or_shared(
                "request_meter_facts.retail_pricing_plan_id",
                record,
                pricing_plan,
            )?;
        }
        let channel_variant_key = format!("{}::{}", record.channel_code, record.model_code);
        if !available_channel_model_variants.contains(&channel_variant_key) {
            bail!(
                "request meter fact {} references missing channel/model variant {}",
                record.request_id,
                channel_variant_key
            );
        }
        let declared_capability = serde_json::from_value::<
            sdkwork_api_domain_catalog::ModelCapability,
        >(serde_json::Value::String(record.capability_code.clone()))
        .with_context(|| {
            format!(
                "bootstrap request_meter_facts.capability_code record {} contains unsupported capability {}",
                record.request_id, record.capability_code
            )
        })?;
        let channel_model = data
            .channel_models
            .iter()
            .find(|model| {
                model.channel_id == record.channel_code && model.model_id == record.model_code
            })
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "bootstrap request_meter_facts.channel_code record {} references missing channel/model variant {}",
                    record.request_id,
                    channel_variant_key
                )
            })?;
        if !channel_model
            .capabilities
            .iter()
            .any(|capability| capability == &declared_capability)
        {
            bail!(
                "request meter fact {} declares capability {} not supported by channel model {}",
                record.request_id,
                record.capability_code,
                channel_variant_key
            );
        }
        let provider_supports_model = data.models.iter().any(|model| {
            model.provider_id == record.provider_code && model.external_name == record.model_code
        }) || data.provider_models.iter().any(|model| {
            model.proxy_provider_id == record.provider_code
                && model.channel_id == record.channel_code
                && model.model_id == record.model_code
                && model.is_active
        });
        if !provider_supports_model {
            bail!(
                "request meter fact {} references model {} that is not available on provider {}",
                record.request_id,
                record.model_code,
                record.provider_code
            );
        }
        let provider_supports_capability = data.models.iter().any(|model| {
            model.provider_id == record.provider_code
                && model.external_name == record.model_code
                && model
                    .capabilities
                    .iter()
                    .any(|capability| capability == &declared_capability)
        }) || data.provider_models.iter().any(|model| {
            model.proxy_provider_id == record.provider_code
                && model.channel_id == record.channel_code
                && model.model_id == record.model_code
                && model.is_active
                && model
                    .capabilities
                    .iter()
                    .any(|capability| capability == &declared_capability)
        });
        if !provider_supports_capability {
            bail!(
                "request meter fact {} declares capability {} not supported by provider model {}::{}::{}",
                record.request_id,
                record.capability_code,
                record.provider_code,
                record.channel_code,
                record.model_code
            );
        }
        if let Some(channels) = provider_channels.get(&record.provider_code) {
            if !channels.contains(&record.channel_code) {
                bail!(
                    "request meter fact {} references provider {} on unsupported channel {}",
                    record.request_id,
                    record.provider_code,
                    record.channel_code
                );
            }
        }
        if !record.estimated_credit_hold.is_finite()
            || record.estimated_credit_hold < 0.0
            || record
                .actual_credit_charge
                .is_some_and(|value: f64| !value.is_finite() || value < 0.0)
            || record
                .actual_provider_cost
                .is_some_and(|value: f64| !value.is_finite() || value < 0.0)
        {
            bail!(
                "request meter fact {} must contain finite non-negative accounting values",
                record.request_id
            );
        }
        if record.updated_at_ms < record.created_at_ms {
            bail!(
                "request meter fact {} has updated_at_ms earlier than created_at_ms",
                record.request_id
            );
        }
        if let Some(finished_at_ms) = record.finished_at_ms {
            if finished_at_ms < record.started_at_ms {
                bail!(
                    "request meter fact {} has finished_at_ms earlier than started_at_ms",
                    record.request_id
                );
            }
        }
        match record.usage_capture_status {
            sdkwork_api_domain_usage::UsageCaptureStatus::Pending => {
                if record.actual_credit_charge.is_some() {
                    bail!(
                        "request meter fact {} usage_capture_status pending must not set actual_credit_charge before capture",
                        record.request_id
                    );
                }
                if record.actual_provider_cost.is_some() {
                    bail!(
                        "request meter fact {} usage_capture_status pending must not set actual_provider_cost before capture",
                        record.request_id
                    );
                }
            }
            sdkwork_api_domain_usage::UsageCaptureStatus::Estimated => {
                if record.actual_credit_charge.is_some() {
                    bail!(
                        "request meter fact {} usage_capture_status estimated must not set actual_credit_charge before capture",
                        record.request_id
                    );
                }
                if record.actual_provider_cost.is_some() {
                    bail!(
                        "request meter fact {} usage_capture_status estimated must not set actual_provider_cost before capture",
                        record.request_id
                    );
                }
            }
            sdkwork_api_domain_usage::UsageCaptureStatus::Captured => {
                if record.actual_credit_charge.is_none() {
                    bail!(
                        "request meter fact {} usage_capture_status captured requires actual_credit_charge",
                        record.request_id
                    );
                }
                if record.actual_provider_cost.is_none() {
                    bail!(
                        "request meter fact {} usage_capture_status captured requires actual_provider_cost",
                        record.request_id
                    );
                }
                if record.finished_at_ms.is_none() {
                    bail!(
                        "request meter fact {} usage_capture_status captured requires finished_at_ms",
                        record.request_id
                    );
                }
            }
            sdkwork_api_domain_usage::UsageCaptureStatus::Reconciled => {
                if record.actual_credit_charge.is_none() {
                    bail!(
                        "request meter fact {} usage_capture_status reconciled requires actual_credit_charge",
                        record.request_id
                    );
                }
                if record.actual_provider_cost.is_none() {
                    bail!(
                        "request meter fact {} usage_capture_status reconciled requires actual_provider_cost",
                        record.request_id
                    );
                }
                if record.finished_at_ms.is_none() {
                    bail!(
                        "request meter fact {} usage_capture_status reconciled requires finished_at_ms",
                        record.request_id
                    );
                }
            }
            sdkwork_api_domain_usage::UsageCaptureStatus::Failed => {}
        }
        if let Some(gateway_request_ref) = record.gateway_request_ref.as_deref() {
            let billing_events = billing_events_by_reference
                .get(gateway_request_ref)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "request meter fact {} gateway_request_ref {} references missing billing event",
                        record.request_id,
                        gateway_request_ref
                    )
                })?;
            if billing_events.len() != 1 {
                let event_ids = billing_events
                    .iter()
                    .map(|event| event.event_id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                bail!(
                    "request meter fact {} gateway_request_ref {} resolves to multiple billing events: {}",
                    record.request_id,
                    gateway_request_ref,
                    event_ids
                );
            }
            let billing_event = billing_events[0];
            if billing_event.provider_id != record.provider_code {
                bail!(
                    "request meter fact {} provider_code {} does not match billing event {} provider_id {}",
                    record.request_id,
                    record.provider_code,
                    billing_event.event_id,
                    billing_event.provider_id
                );
            }
            if billing_event.channel_id.as_deref() != Some(record.channel_code.as_str()) {
                let billing_channel = billing_event.channel_id.as_deref().unwrap_or("<none>");
                bail!(
                    "request meter fact {} channel_code {} does not match billing event {} channel_id {}",
                    record.request_id,
                    record.channel_code,
                    billing_event.event_id,
                    billing_channel
                );
            }
            if billing_event.capability != record.capability_code {
                bail!(
                    "request meter fact {} capability_code {} does not match billing event {} capability {}",
                    record.request_id,
                    record.capability_code,
                    billing_event.event_id,
                    billing_event.capability
                );
            }
            if billing_event.route_key != record.model_code {
                bail!(
                    "request meter fact {} model_code {} does not match billing event {} route_key {}",
                    record.request_id,
                    record.model_code,
                    billing_event.event_id,
                    billing_event.route_key
                );
            }
            if billing_event.api_key_hash.as_deref() != record.api_key_hash.as_deref() {
                let billing_api_key_hash =
                    billing_event.api_key_hash.as_deref().unwrap_or("<none>");
                let request_api_key_hash = record.api_key_hash.as_deref().unwrap_or("<none>");
                bail!(
                    "request meter fact {} api_key_hash {} does not match billing event {} api_key_hash {}",
                    record.request_id,
                    request_api_key_hash,
                    billing_event.event_id,
                    billing_api_key_hash
                );
            }
            if let Some(actual_provider_cost) = record.actual_provider_cost {
                if (billing_event.upstream_cost - actual_provider_cost).abs() > epsilon {
                    bail!(
                        "request meter fact {} actual_provider_cost {} does not match billing event {} upstream_cost {}",
                        record.request_id,
                        actual_provider_cost,
                        billing_event.event_id,
                        billing_event.upstream_cost
                    );
                }
            }
            let (input_metric_count, input_metric_sum, output_metric_count, output_metric_sum) =
                request_metric_token_summaries
                    .get(&record.request_id)
                    .copied()
                    .unwrap_or((0, 0.0, 0, 0.0));
            if billing_event.input_tokens > 0 && input_metric_count == 0 {
                bail!(
                    "request meter fact {} billing event {} input_tokens {} requires token.input metrics",
                    record.request_id,
                    billing_event.event_id,
                    billing_event.input_tokens
                );
            }
            if input_metric_count > 0
                && (input_metric_sum - billing_event.input_tokens as f64).abs() > epsilon
            {
                bail!(
                    "request meter fact {} token.input metric quantity {} does not match billing event {} input_tokens {}",
                    record.request_id,
                    input_metric_sum,
                    billing_event.event_id,
                    billing_event.input_tokens
                );
            }
            if billing_event.output_tokens > 0 && output_metric_count == 0 {
                bail!(
                    "request meter fact {} billing event {} output_tokens {} requires token.output metrics",
                    record.request_id,
                    billing_event.event_id,
                    billing_event.output_tokens
                );
            }
            if output_metric_count > 0
                && (output_metric_sum - billing_event.output_tokens as f64).abs() > epsilon
            {
                bail!(
                    "request meter fact {} token.output metric quantity {} does not match billing event {} output_tokens {}",
                    record.request_id,
                    output_metric_sum,
                    billing_event.event_id,
                    billing_event.output_tokens
                );
            }
        }
    }

    for record in &data.request_meter_metrics {
        let record_id = record.request_metric_id.to_string();
        ensure_reference(
            "request_meter_metrics.request_id",
            &record_id,
            &record.request_id.to_string(),
            &collect_ids(
                data.request_meter_facts
                    .iter()
                    .map(|fact| fact.request_id.to_string()),
            ),
        )?;
        ensure_non_empty_field(
            "request_meter_metrics.metric_code",
            &record_id,
            &record.metric_code,
        )?;
        ensure_non_empty_field(
            "request_meter_metrics.source_kind",
            &record_id,
            &record.source_kind,
        )?;
        ensure_non_empty_field(
            "request_meter_metrics.capture_stage",
            &record_id,
            &record.capture_stage,
        )?;
        if !record.quantity.is_finite() || record.quantity < 0.0 {
            bail!(
                "request meter metric {} quantity must be a finite non-negative value",
                record.request_metric_id
            );
        }
        let fact = request_meter_facts.get(&record.request_id).ok_or_else(|| {
            anyhow::anyhow!(
                "bootstrap request_meter_metrics.request_id record {} references missing {}",
                record.request_metric_id,
                record.request_id
            )
        })?;
        if record.tenant_id != fact.tenant_id || record.organization_id != fact.organization_id {
            bail!(
                "request meter metric {} must match tenant and organization ownership of request meter fact {}",
                record.request_metric_id,
                record.request_id
            );
        }
        match fact.usage_capture_status {
            sdkwork_api_domain_usage::UsageCaptureStatus::Estimated
                if record.capture_stage != "estimate" =>
            {
                bail!(
                    "request meter metric {} capture_stage {} does not match estimated usage_capture_status of request meter fact {}",
                    record.request_metric_id,
                    record.capture_stage,
                    record.request_id
                );
            }
            sdkwork_api_domain_usage::UsageCaptureStatus::Captured
                if record.capture_stage != "final" =>
            {
                bail!(
                    "request meter metric {} capture_stage {} does not match captured usage_capture_status of request meter fact {}",
                    record.request_metric_id,
                    record.capture_stage,
                    record.request_id
                );
            }
            _ => {}
        }
        if record.captured_at_ms < fact.started_at_ms {
            bail!(
                "request meter metric {} has captured_at_ms earlier than the parent request started_at_ms",
                record.request_metric_id
            );
        }
        if let Some(finished_at_ms) = fact.finished_at_ms {
            if record.captured_at_ms > finished_at_ms {
                bail!(
                    "request meter metric {} has captured_at_ms later than the parent request finished_at_ms",
                    record.request_metric_id
                );
            }
        }
        if record.captured_at_ms > fact.updated_at_ms {
            bail!(
                "request meter metric {} has captured_at_ms later than the parent request updated_at_ms",
                record.request_metric_id
            );
        }
    }

    for record in &data.request_settlements {
        let record_id = record.request_settlement_id.to_string();
        ensure_reference(
            "request_settlements.account_id",
            &record_id,
            &record.account_id.to_string(),
            account_ids,
        )?;
        let account = accounts.get(&record.account_id).ok_or_else(|| {
            anyhow::anyhow!(
                "bootstrap request_settlements.account_id record {} references missing {}",
                record.request_settlement_id,
                record.account_id
            )
        })?;
        if account.tenant_id != record.tenant_id
            || account.organization_id != record.organization_id
            || account.user_id != record.user_id
        {
            bail!(
                "request settlement {} must match tenant/organization/user ownership of account {}",
                record.request_settlement_id,
                record.account_id
            );
        }
        if let Some(hold_id) = record.hold_id {
            ensure_reference(
                "request_settlements.hold_id",
                &record_id,
                &hold_id.to_string(),
                &collect_ids(
                    data.account_holds
                        .iter()
                        .map(|hold| hold.hold_id.to_string()),
                ),
            )?;
            let hold = account_holds.get(&hold_id).ok_or_else(|| {
                anyhow::anyhow!(
                    "bootstrap request_settlements.hold_id record {} references missing {}",
                    record.request_settlement_id,
                    hold_id
                )
            })?;
            if hold.account_id != record.account_id || hold.request_id != record.request_id {
                bail!(
                    "request settlement {} must reference a hold for the same account and request",
                    record.request_settlement_id
                );
            }
            if (hold.estimated_quantity - record.estimated_credit_hold).abs() > epsilon {
                bail!(
                    "request settlement {} estimated_credit_hold does not match linked hold {} estimated_quantity",
                    record.request_settlement_id,
                    hold_id
                );
            }
            if (hold.captured_quantity - record.captured_credit_amount).abs() > epsilon {
                bail!(
                    "request settlement {} captured_credit_amount does not match linked hold {} captured_quantity",
                    record.request_settlement_id,
                    hold_id
                );
            }
            if (hold.released_quantity - record.released_credit_amount).abs() > epsilon {
                bail!(
                    "request settlement {} released_credit_amount does not match linked hold {} released_quantity",
                    record.request_settlement_id,
                    hold_id
                );
            }
            if record.created_at_ms < hold.created_at_ms {
                bail!(
                    "request settlement {} has created_at_ms earlier than linked hold {} created_at_ms",
                    record.request_settlement_id,
                    hold_id
                );
            }
            if record.updated_at_ms < hold.updated_at_ms {
                bail!(
                    "request settlement {} has updated_at_ms earlier than linked hold {} updated_at_ms",
                    record.request_settlement_id,
                    hold_id
                );
            }
            if record.settled_at_ms != 0 && record.settled_at_ms < hold.updated_at_ms {
                bail!(
                    "request settlement {} has settled_at_ms earlier than linked hold {} updated_at_ms",
                    record.request_settlement_id,
                    hold_id
                );
            }
            match record.status {
                RequestSettlementStatus::Pending => {
                    if hold.status != AccountHoldStatus::Held {
                        bail!(
                            "request settlement {} status pending requires linked hold {} status held",
                            record.request_settlement_id,
                            hold_id
                        );
                    }
                }
                RequestSettlementStatus::PartiallyReleased => {
                    if hold.status != AccountHoldStatus::PartiallyReleased {
                        bail!(
                            "request settlement {} status partially_released requires linked hold {} status partially_released",
                            record.request_settlement_id,
                            hold_id
                        );
                    }
                }
                _ => {}
            }
        }
        if !record.estimated_credit_hold.is_finite()
            || !record.released_credit_amount.is_finite()
            || !record.captured_credit_amount.is_finite()
            || !record.provider_cost_amount.is_finite()
            || !record.retail_charge_amount.is_finite()
            || !record.shortfall_amount.is_finite()
            || !record.refunded_amount.is_finite()
            || record.estimated_credit_hold < 0.0
            || record.released_credit_amount < 0.0
            || record.captured_credit_amount < 0.0
            || record.provider_cost_amount < 0.0
            || record.retail_charge_amount < 0.0
            || record.shortfall_amount < 0.0
            || record.refunded_amount < 0.0
        {
            bail!(
                "request settlement {} must contain finite non-negative accounting values",
                record.request_settlement_id
            );
        }
        match record.status {
            RequestSettlementStatus::Pending => {
                if record.settled_at_ms != 0 {
                    bail!(
                        "request settlement {} status pending must not set settled_at_ms",
                        record.request_settlement_id
                    );
                }
                if record.released_credit_amount > epsilon
                    || record.captured_credit_amount > epsilon
                    || record.provider_cost_amount > epsilon
                    || record.retail_charge_amount > epsilon
                    || record.shortfall_amount > epsilon
                    || record.refunded_amount > epsilon
                {
                    bail!(
                        "request settlement {} status pending must not contain realized accounting values",
                        record.request_settlement_id
                    );
                }
            }
            RequestSettlementStatus::Captured
            | RequestSettlementStatus::PartiallyReleased
            | RequestSettlementStatus::Released
            | RequestSettlementStatus::Refunded => {
                if record.settled_at_ms == 0 {
                    bail!(
                        "request settlement {} status {:?} requires settled_at_ms",
                        record.request_settlement_id,
                        record.status
                    );
                }
            }
            RequestSettlementStatus::Failed => {}
        }
        if record.status == RequestSettlementStatus::PartiallyReleased {
            if record.released_credit_amount <= epsilon {
                bail!(
                    "request settlement {} status partially_released requires released_credit_amount",
                    record.request_settlement_id
                );
            }
            if record.captured_credit_amount <= epsilon {
                bail!(
                    "request settlement {} status partially_released requires captured_credit_amount",
                    record.request_settlement_id
                );
            }
            if record.refunded_amount > epsilon {
                bail!(
                    "request settlement {} status partially_released must not set refunded_amount",
                    record.request_settlement_id
                );
            }
        }
        if record.captured_credit_amount + record.released_credit_amount
            > record.estimated_credit_hold + epsilon
        {
            bail!(
                "request settlement {} captured and released credit amounts exceed estimated_credit_hold",
                record.request_settlement_id
            );
        }
        if record.refunded_amount > record.captured_credit_amount + epsilon {
            bail!(
                "request settlement {} refunded_amount exceeds captured_credit_amount",
                record.request_settlement_id
            );
        }
        if record.updated_at_ms < record.created_at_ms {
            bail!(
                "request settlement {} has updated_at_ms earlier than created_at_ms",
                record.request_settlement_id
            );
        }
        if record.settled_at_ms != 0 && record.settled_at_ms < record.created_at_ms {
            bail!(
                "request settlement {} has settled_at_ms earlier than created_at_ms",
                record.request_settlement_id
            );
        }
        let fact = request_meter_facts.get(&record.request_id).ok_or_else(|| {
            anyhow::anyhow!(
                "bootstrap request_settlements.request_id record {} references missing request meter fact {}",
                record.request_settlement_id,
                record.request_id
            )
        })?;
        if fact.account_id != record.account_id
            || fact.tenant_id != record.tenant_id
            || fact.organization_id != record.organization_id
            || fact.user_id != record.user_id
        {
            bail!(
                "request settlement {} must match tenant/organization/user/account ownership of request meter fact {}",
                record.request_settlement_id,
                record.request_id
            );
        }
        match record.status {
            RequestSettlementStatus::Pending => {
                if fact.usage_capture_status != UsageCaptureStatus::Estimated {
                    bail!(
                        "request settlement {} status pending requires request meter fact {} usage_capture_status estimated",
                        record.request_settlement_id,
                        record.request_id
                    );
                }
            }
            RequestSettlementStatus::Captured
            | RequestSettlementStatus::PartiallyReleased
            | RequestSettlementStatus::Released
            | RequestSettlementStatus::Refunded => {
                if !matches!(
                    fact.usage_capture_status,
                    UsageCaptureStatus::Captured | UsageCaptureStatus::Reconciled
                ) {
                    bail!(
                        "request settlement {} status {:?} requires request meter fact {} usage_capture_status captured or reconciled",
                        record.request_settlement_id,
                        record.status,
                        record.request_id
                    );
                }
            }
            RequestSettlementStatus::Failed => {}
        }
        if record.created_at_ms < fact.started_at_ms {
            bail!(
                "request settlement {} has created_at_ms earlier than request meter fact {} started_at_ms",
                record.request_settlement_id,
                record.request_id
            );
        }
        if let Some(finished_at_ms) = fact.finished_at_ms {
            if record.settled_at_ms != 0 && record.settled_at_ms < finished_at_ms {
                bail!(
                    "request settlement {} has settled_at_ms earlier than request meter fact {} finished_at_ms",
                    record.request_settlement_id,
                    record.request_id
                );
            }
        }
        if (record.estimated_credit_hold - fact.estimated_credit_hold).abs() > epsilon {
            bail!(
                "request settlement {} estimated_credit_hold does not match request meter fact {}",
                record.request_settlement_id,
                record.request_id
            );
        }
        if (record.captured_credit_amount - fact.actual_credit_charge.unwrap_or(0.0)).abs()
            > epsilon
        {
            bail!(
                "request settlement {} captured_credit_amount does not match request meter fact {}",
                record.request_settlement_id,
                record.request_id
            );
        }
        if (record.provider_cost_amount - fact.actual_provider_cost.unwrap_or(0.0)).abs()
            > epsilon
        {
            bail!(
                "request settlement {} provider_cost_amount does not match request meter fact {}",
                record.request_settlement_id,
                record.request_id
            );
        }
    }

    for record in &data.account_commerce_reconciliation_states {
        let record_id = format!("{}::{}", record.account_id, record.project_id);
        ensure_reference(
            "account_reconciliation.account_id",
            &record_id,
            &record.account_id.to_string(),
            account_ids,
        )?;
        let account = accounts.get(&record.account_id).ok_or_else(|| {
            anyhow::anyhow!(
                "bootstrap account_reconciliation.account_id record {} references missing {}",
                record_id,
                record.account_id
            )
        })?;
        if account.tenant_id != record.tenant_id || account.organization_id != record.organization_id {
            bail!(
                "account reconciliation state {} must match tenant/organization ownership of account {}",
                record_id,
                record.account_id
            );
        }
        ensure_reference(
            "account_reconciliation.project_id",
            &record_id,
            &record.project_id,
            project_ids,
        )?;
        ensure_reference(
            "account_reconciliation.last_order_id",
            &record_id,
            &record.last_order_id,
            commerce_order_ids,
        )?;
        let order = commerce_orders
            .get(record.last_order_id.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!(
                "bootstrap account_reconciliation.last_order_id record {} references missing {}",
                record_id,
                record.last_order_id
            )
            })?;
        if order.project_id != record.project_id {
            bail!(
                "account reconciliation state {} references order {} from another project",
                record_id,
                record.last_order_id
            );
        }
        if order.created_at_ms != record.last_order_created_at_ms {
            bail!(
                "account reconciliation state {} last_order_created_at_ms does not match order {} created_at_ms",
                record_id,
                record.last_order_id
            );
        }
        if order.updated_at_ms != record.last_order_updated_at_ms {
            bail!(
                "account reconciliation state {} last_order_updated_at_ms does not match order {} updated_at_ms",
                record_id,
                record.last_order_id
            );
        }
        if record.last_order_updated_at_ms < record.last_order_created_at_ms {
            bail!(
                "account reconciliation state {} has last_order_updated_at_ms earlier than last_order_created_at_ms",
                record_id
            );
        }
        if record.updated_at_ms < record.last_order_updated_at_ms {
            bail!(
                "account reconciliation state {} has updated_at_ms earlier than last_order_updated_at_ms",
                record_id
            );
        }
    }

    Ok(())
}

fn build_provider_channels(providers: &[ProxyProvider]) -> HashMap<String, HashSet<String>> {
    let mut provider_channels = HashMap::new();
    for provider in providers {
        let channels = provider_channels
            .entry(provider.id.clone())
            .or_insert_with(HashSet::new);
        channels.insert(provider.channel_id.clone());
        for binding in &provider.channel_bindings {
            channels.insert(binding.channel_id.clone());
        }
    }
    provider_channels
}

fn build_available_channel_model_variants(
    models: &[ModelCatalogEntry],
    provider_models: &[ProviderModelRecord],
    provider_channels: &HashMap<String, HashSet<String>>,
) -> HashSet<String> {
    let mut variants = HashSet::new();
    for model in models {
        if let Some(channels) = provider_channels.get(&model.provider_id) {
            for channel_id in channels {
                variants.insert(format!("{}::{}", channel_id, model.external_name));
            }
        }
    }
    for record in provider_models {
        if record.is_active {
            variants.insert(format!("{}::{}", record.channel_id, record.model_id));
        }
    }
    variants
}

fn routing_profile_provider_ids(record: &RoutingProfileRecord) -> Vec<String> {
    let mut provider_ids = record.ordered_provider_ids.clone();
    if let Some(default_provider_id) = record.default_provider_id.as_ref() {
        if !provider_ids
            .iter()
            .any(|provider_id| provider_id == default_provider_id)
        {
            provider_ids.push(default_provider_id.clone());
        }
    }
    provider_ids
}

fn compiled_routing_snapshot_provider_ids(record: &CompiledRoutingSnapshotRecord) -> Vec<String> {
    let mut provider_ids = record.ordered_provider_ids.clone();
    if let Some(default_provider_id) = record.default_provider_id.as_ref() {
        if !provider_ids
            .iter()
            .any(|provider_id| provider_id == default_provider_id)
        {
            provider_ids.push(default_provider_id.clone());
        }
    }
    provider_ids
}

fn project_preference_provider_ids(record: &ProjectRoutingPreferences) -> Vec<String> {
    let mut provider_ids = record.ordered_provider_ids.clone();
    if let Some(default_provider_id) = record.default_provider_id.as_ref() {
        if !provider_ids
            .iter()
            .any(|provider_id| provider_id == default_provider_id)
        {
            provider_ids.push(default_provider_id.clone());
        }
    }
    provider_ids
}

fn ensure_unique<T>(label: &str, items: &[T], key_fn: impl Fn(&T) -> String) -> Result<()> {
    let mut seen = HashSet::new();
    for item in items {
        let key = key_fn(item);
        if !seen.insert(key.clone()) {
            bail!("bootstrap {} contains duplicate entry {}", label, key);
        }
    }
    Ok(())
}

fn ensure_reference(
    label: &str,
    record_id: &str,
    referenced_value: &str,
    available_values: &HashSet<String>,
) -> Result<()> {
    if available_values.contains(referenced_value) {
        return Ok(());
    }

    bail!(
        "bootstrap {} record {} references missing {}",
        label,
        record_id,
        referenced_value
    )
}

fn ensure_provider_list_exists(
    label: &str,
    record_id: &str,
    provider_ids: Vec<String>,
    available_provider_ids: &HashSet<String>,
) -> Result<()> {
    for provider_id in provider_ids {
        ensure_reference(label, record_id, &provider_id, available_provider_ids)?;
    }
    Ok(())
}

fn ensure_provider_has_enabled_account(
    label: &str,
    record_id: &str,
    provider_id: &str,
    enabled_provider_account_provider_ids: &HashSet<String>,
) -> Result<()> {
    if enabled_provider_account_provider_ids.contains(provider_id) {
        return Ok(());
    }

    bail!(
        "bootstrap {} record {} references provider {} without any enabled provider account",
        label,
        record_id,
        provider_id
    )
}

fn ensure_active_model_price_coverage(
    label: &str,
    record_id: &str,
    provider_id: &str,
    channel_id: &str,
    model_id: &str,
    active_model_price_keys: &HashSet<String>,
) -> Result<()> {
    let price_key = format!("{}::{}::{}", provider_id, channel_id, model_id);
    if active_model_price_keys.contains(&price_key) {
        return Ok(());
    }

    bail!(
        "bootstrap {} record {} references provider {} channel {} model {} without active model price coverage",
        label,
        record_id,
        provider_id,
        channel_id,
        model_id
    )
}

fn ensure_provider_has_active_route_price_coverage(
    label: &str,
    record_id: &str,
    provider_id: &str,
    route_key: &str,
    provider_channels: &HashMap<String, HashSet<String>>,
    active_model_price_keys: &HashSet<String>,
) -> Result<()> {
    if let Some(channels) = provider_channels.get(provider_id) {
        let mut sorted_channels = channels.iter().collect::<Vec<_>>();
        sorted_channels.sort_unstable();
        for channel_id in &sorted_channels {
            let price_key = format!("{}::{}::{}", provider_id, channel_id, route_key);
            if active_model_price_keys.contains(&price_key) {
                return Ok(());
            }
        }

        let bound_channels = sorted_channels
            .into_iter()
            .map(|channel_id| channel_id.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        bail!(
            "bootstrap {} record {} references provider {} route_key {} without active model price coverage on any bound channel [{}]",
            label,
            record_id,
            provider_id,
            route_key,
            bound_channels
        );
    }

    bail!(
        "bootstrap {} record {} references provider {} route_key {} without any provider channel bindings",
        label,
        record_id,
        provider_id,
        route_key
    )
}

fn ensure_provider_has_any_active_model_price_coverage(
    label: &str,
    record_id: &str,
    provider_id: &str,
    provider_channels: &HashMap<String, HashSet<String>>,
    active_model_price_keys: &HashSet<String>,
) -> Result<()> {
    if let Some(channels) = provider_channels.get(provider_id) {
        let mut sorted_channels = channels.iter().collect::<Vec<_>>();
        sorted_channels.sort_unstable();
        for channel_id in &sorted_channels {
            let prefix = format!("{}::{}::", provider_id, channel_id);
            if active_model_price_keys
                .iter()
                .any(|price_key| price_key.starts_with(&prefix))
            {
                return Ok(());
            }
        }

        let bound_channels = sorted_channels
            .into_iter()
            .map(|channel_id| channel_id.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        bail!(
            "bootstrap {} record {} references provider {} without active model price coverage on any bound channel [{}]",
            label,
            record_id,
            provider_id,
            bound_channels
        );
    }

    bail!(
        "bootstrap {} record {} references provider {} without any provider channel bindings",
        label,
        record_id,
        provider_id
    )
}

fn ensure_provider_has_any_active_model_price_capability_coverage(
    label: &str,
    record_id: &str,
    provider_id: &str,
    capability_code: &str,
    capability: &sdkwork_api_domain_catalog::ModelCapability,
    provider_channels: &HashMap<String, HashSet<String>>,
    model_prices: &[ModelPriceRecord],
    models: &[ModelCatalogEntry],
    provider_models: &[ProviderModelRecord],
) -> Result<()> {
    if let Some(channels) = provider_channels.get(provider_id) {
        let mut sorted_channels = channels.iter().collect::<Vec<_>>();
        sorted_channels.sort_unstable();
        for channel_id in &sorted_channels {
            if model_prices.iter().any(|price| {
                price.is_active
                    && price.proxy_provider_id == provider_id
                    && price.channel_id == channel_id.as_str()
                    && provider_channel_supports_catalog_model_capability(
                        provider_id,
                        channel_id.as_str(),
                        &price.model_id,
                        capability,
                        models,
                        provider_models,
                    )
            }) {
                return Ok(());
            }
        }

        let bound_channels = sorted_channels
            .into_iter()
            .map(|channel_id| channel_id.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        bail!(
            "bootstrap {} record {} references provider {} capability {} without any active model price coverage on a capability-matched model across bound channels [{}]",
            label,
            record_id,
            provider_id,
            capability_code,
            bound_channels
        );
    }

    bail!(
        "bootstrap {} record {} references provider {} capability {} without any provider channel bindings",
        label,
        record_id,
        provider_id,
        capability_code
    )
}

fn ensure_routing_policy_has_any_capability_matched_active_model_price_coverage(
    record: &RoutingPolicy,
    provider_ids: &[String],
    model_prices: &[ModelPriceRecord],
    models: &[ModelCatalogEntry],
    provider_models: &[ProviderModelRecord],
) -> Result<()> {
    let declared_capability =
        serde_json::from_value::<sdkwork_api_domain_catalog::ModelCapability>(
            serde_json::Value::String(record.capability.clone()),
        )
        .map_err(|_| {
            anyhow::anyhow!(
                "bootstrap routing_policies.capability record {} contains unsupported capability {}",
                record.policy_id,
                record.capability
            )
        })?;
    let has_supported_provider = provider_ids.iter().any(|provider_id| {
        model_prices
            .iter()
            .filter(|price| {
                price.is_active
                    && price.proxy_provider_id == *provider_id
                    && record.matches(record.capability.as_str(), &price.model_id)
            })
            .any(|price| {
                models.iter().any(|model| {
                    model.provider_id == *provider_id
                        && model.external_name == price.model_id
                        && model
                            .capabilities
                            .iter()
                            .any(|capability| capability == &declared_capability)
                }) || provider_models.iter().any(|provider_model| {
                    provider_model.is_active
                        && provider_model.proxy_provider_id == *provider_id
                        && provider_model.model_id == price.model_id
                        && provider_model
                            .capabilities
                            .iter()
                            .any(|capability| capability == &declared_capability)
                })
            })
    });

    if has_supported_provider {
        return Ok(());
    }

    bail!(
        "routing policy {} capability {} model_pattern {} does not have any declared provider with capability-matched active model price coverage [{}]",
        record.policy_id,
        record.capability,
        record.model_pattern,
        provider_ids.join(", ")
    )
}

fn ensure_compiled_snapshot_matched_policy_is_enabled_and_matches(
    record: &CompiledRoutingSnapshotRecord,
    policy_id: &str,
    routing_policies: &HashMap<&str, &RoutingPolicy>,
) -> Result<()> {
    let policy = routing_policies.get(policy_id).ok_or_else(|| {
        anyhow::anyhow!(
            "bootstrap compiled_routing_snapshots.matched_policy_id record {} references missing {}",
            record.snapshot_id,
            policy_id
        )
    })?;

    if !policy.enabled {
        bail!(
            "compiled routing snapshot {} references disabled matched policy {}",
            record.snapshot_id,
            policy.policy_id
        );
    }

    if !policy.matches(&record.capability, &record.route_key) {
        bail!(
            "compiled routing snapshot {} matched policy {} capability {} model_pattern {} does not match snapshot capability {} route_key {}",
            record.snapshot_id,
            policy.policy_id,
            policy.capability,
            policy.model_pattern,
            record.capability,
            record.route_key
        );
    }

    Ok(())
}

fn ensure_routing_decision_matched_policy_is_enabled_and_matches(
    record: &RoutingDecisionLog,
    policy_id: &str,
    routing_policies: &HashMap<&str, &RoutingPolicy>,
) -> Result<()> {
    let policy = routing_policies.get(policy_id).ok_or_else(|| {
        anyhow::anyhow!(
            "bootstrap routing_decision_logs.matched_policy_id record {} references missing {}",
            record.decision_id,
            policy_id
        )
    })?;

    if !policy.enabled {
        bail!(
            "routing decision log {} references disabled matched policy {}",
            record.decision_id,
            policy.policy_id
        );
    }

    if !policy.matches(&record.capability, &record.route_key) {
        bail!(
            "routing decision log {} matched policy {} capability {} model_pattern {} does not match decision capability {} route_key {}",
            record.decision_id,
            policy.policy_id,
            policy.capability,
            policy.model_pattern,
            record.capability,
            record.route_key
        );
    }

    Ok(())
}

fn ensure_compiled_snapshot_temporal_posture(record: &CompiledRoutingSnapshotRecord) -> Result<()> {
    if record.updated_at_ms >= record.created_at_ms {
        return Ok(());
    }

    bail!(
        "compiled routing snapshot {} has updated_at_ms {} earlier than created_at_ms {}",
        record.snapshot_id,
        record.updated_at_ms,
        record.created_at_ms
    )
}

fn ensure_compiled_snapshot_default_provider_matches_deterministic_priority(
    record: &CompiledRoutingSnapshotRecord,
) -> Result<()> {
    if record.strategy != RoutingStrategy::DeterministicPriority.as_str() {
        return Ok(());
    }

    let Some(default_provider_id) = record.default_provider_id.as_deref() else {
        return Ok(());
    };

    let Some(first_provider_id) = record.ordered_provider_ids.first() else {
        return Ok(());
    };

    if default_provider_id == first_provider_id {
        return Ok(());
    }

    bail!(
        "compiled routing snapshot {} strategy {} requires default_provider_id {} to match first ordered provider {}",
        record.snapshot_id,
        record.strategy,
        default_provider_id,
        first_provider_id
    )
}

fn collect_selected_provider_assessments<'a>(
    record: &'a RoutingDecisionLog,
) -> Vec<&'a RoutingCandidateAssessment> {
    record
        .assessments
        .iter()
        .filter(|assessment| assessment.provider_id == record.selected_provider_id)
        .collect()
}

fn ensure_selected_provider_assessment_exists(
    record: &RoutingDecisionLog,
    selected_provider_assessments: &[&RoutingCandidateAssessment],
) -> Result<()> {
    if selected_provider_assessments.len() == 1 {
        return Ok(());
    }

    let qualifier = if selected_provider_assessments.is_empty() {
        "missing"
    } else {
        "duplicated"
    };

    bail!(
        "routing decision log {} has {} assessment evidence for selected provider {}",
        record.decision_id,
        qualifier,
        record.selected_provider_id
    )
}

fn ensure_selected_provider_assessment_is_available(
    record: &RoutingDecisionLog,
    selected_provider_assessments: &[&RoutingCandidateAssessment],
) -> Result<()> {
    let Some(selected_provider_assessment) = selected_provider_assessments.first() else {
        return Ok(());
    };

    if selected_provider_assessment.available {
        return Ok(());
    }

    bail!(
        "routing decision log {} selected provider {} assessment is not available",
        record.decision_id,
        record.selected_provider_id
    )
}

fn ensure_selected_provider_assessment_satisfies_snapshot_health_requirement(
    record: &RoutingDecisionLog,
    snapshot: &CompiledRoutingSnapshotRecord,
    selected_provider_assessments: &[&RoutingCandidateAssessment],
) -> Result<()> {
    if !snapshot.require_healthy {
        return Ok(());
    }

    let Some(selected_provider_assessment) = selected_provider_assessments.first() else {
        return Ok(());
    };

    if selected_provider_assessment.health == RoutingCandidateHealth::Healthy {
        return Ok(());
    }

    bail!(
        "routing decision log {} selected provider {} assessment health {:?} does not satisfy require_healthy compiled routing snapshot {}",
        record.decision_id,
        record.selected_provider_id,
        selected_provider_assessment.health,
        snapshot.snapshot_id
    )
}

fn ensure_provider_list_has_enabled_accounts(
    label: &str,
    record_id: &str,
    provider_ids: &[String],
    enabled_provider_account_provider_ids: &HashSet<String>,
) -> Result<()> {
    for provider_id in provider_ids {
        ensure_provider_has_enabled_account(
            label,
            record_id,
            provider_id,
            enabled_provider_account_provider_ids,
        )?;
    }
    Ok(())
}

fn ensure_provider_declared_by_compiled_snapshot(
    label: &str,
    record_id: &str,
    snapshot_id: &str,
    provider_id: &str,
    snapshot: &CompiledRoutingSnapshotRecord,
) -> Result<()> {
    let snapshot_provider_ids = compiled_routing_snapshot_provider_ids(snapshot);
    if snapshot_provider_ids
        .iter()
        .any(|snapshot_provider_id| snapshot_provider_id == provider_id)
    {
        return Ok(());
    }

    bail!(
        "bootstrap {} record {} references provider {} outside compiled routing snapshot {}",
        label,
        record_id,
        provider_id,
        snapshot_id
    )
}

fn ensure_provider_declared_by_routing_profile(
    label: &str,
    record_id: &str,
    profile_id: &str,
    provider_id: &str,
    profile: &RoutingProfileRecord,
) -> Result<()> {
    let profile_provider_ids = routing_profile_provider_ids(profile);
    if profile_provider_ids
        .iter()
        .any(|profile_provider_id| profile_provider_id == provider_id)
    {
        return Ok(());
    }

    bail!(
        "bootstrap {} record {} references provider {} outside applied routing profile {}",
        label,
        record_id,
        provider_id,
        profile_id
    )
}

fn ensure_provider_health_snapshot_runtime_posture(record: &ProviderHealthSnapshot) -> Result<()> {
    if record.runtime == "builtin" && record.instance_id.is_none() {
        bail!(
            "provider health snapshot {} runtime builtin requires instance_id evidence",
            record.provider_id
        );
    }

    if record.runtime == "passthrough" && record.instance_id.is_some() {
        bail!(
            "provider health snapshot {} runtime passthrough must not reference instance_id",
            record.provider_id
        );
    }

    if record.healthy && !record.running {
        bail!(
            "provider health snapshot {} cannot be healthy while running = false",
            record.provider_id
        );
    }

    if record.healthy
        && record
            .message
            .as_deref()
            .is_none_or(|message| message.trim().is_empty())
    {
        bail!(
            "provider health snapshot {} healthy state requires a message",
            record.provider_id
        );
    }

    Ok(())
}

fn ensure_async_job_attempt_fits_parent_lifecycle(
    record: &AsyncJobAttemptRecord,
    job: &AsyncJobRecord,
) -> Result<()> {
    if let Some(job_provider_id) = job.provider_id.as_deref() {
        let normalized_job_provider = normalize_async_job_payload_provider(job_provider_id);
        if record.runtime_kind != normalized_job_provider {
            bail!(
                "async job attempt {} runtime_kind {} must match parent job {} provider {} ({})",
                record.attempt_id,
                record.runtime_kind,
                job.job_id,
                job_provider_id,
                normalized_job_provider
            );
        }
    }

    if let (Some(claimed_at_ms), Some(job_started_at_ms)) = (record.claimed_at_ms, job.started_at_ms)
    {
        if claimed_at_ms < job_started_at_ms {
            bail!(
                "async job attempt {} has claimed_at_ms earlier than parent job {} started_at_ms",
                record.attempt_id,
                job.job_id
            );
        }
    }

    if let (Some(finished_at_ms), Some(job_completed_at_ms)) =
        (record.finished_at_ms, job.completed_at_ms)
    {
        if finished_at_ms > job_completed_at_ms {
            bail!(
                "async job attempt {} has finished_at_ms later than parent job {} completed_at_ms",
                record.attempt_id,
                job.job_id
            );
        }
    }

    Ok(())
}

fn ensure_async_job_asset_fits_parent_scope(
    record: &AsyncJobAssetRecord,
    job: &AsyncJobRecord,
) -> Result<()> {
    if let (Some(mime_type), Some(expected_mime_type)) = (
        record.mime_type.as_deref(),
        expected_async_job_asset_mime_type(record.asset_kind.as_str()),
    ) {
        if mime_type != expected_mime_type {
            bail!(
                "async job asset {} mime_type {} must match asset_kind {} ({})",
                record.asset_id,
                mime_type,
                record.asset_kind,
                expected_mime_type
            );
        }
    }

    if record.created_at_ms < job.created_at_ms {
        bail!(
            "async job asset {} has created_at_ms earlier than parent job {} created_at_ms",
            record.asset_id,
            job.job_id
        );
    }

    if !record.storage_key.contains(&record.job_id) {
        bail!(
            "async job asset {} storage_key must contain parent job {}",
            record.asset_id,
            record.job_id
        );
    }

    let expected_prefix = format!("tenant-{}/jobs/", job.tenant_id);
    if !record.storage_key.starts_with(&expected_prefix) {
        bail!(
            "async job asset {} storage_key must stay within parent tenant scope {}",
            record.asset_id,
            expected_prefix
        );
    }

    if let (Some(storage_leaf), Some(expected_extension)) = (
        async_job_path_leaf(record.storage_key.as_str()),
        expected_async_job_asset_extension(record.asset_kind.as_str()),
    ) {
        if !storage_leaf.ends_with(expected_extension) {
            bail!(
                "async job asset {} storage_key leaf {} must match asset_kind {} ({})",
                record.asset_id,
                storage_leaf,
                record.asset_kind,
                expected_extension
            );
        }
    }

    if let (Some(storage_leaf), Some(download_leaf)) = (
        async_job_path_leaf(record.storage_key.as_str()),
        record
            .download_url
            .as_deref()
            .and_then(async_job_path_leaf),
    ) {
        if download_leaf != storage_leaf {
            bail!(
                "async job asset {} download_url leaf {} must match storage_key leaf {} for parent job {}",
                record.asset_id,
                download_leaf,
                storage_leaf,
                job.job_id
            );
        }
    }

    Ok(())
}

fn ensure_async_job_callback_fits_parent_lifecycle(
    record: &AsyncJobCallbackRecord,
    job: &AsyncJobRecord,
) -> Result<()> {
    if record.received_at_ms < job.created_at_ms {
        bail!(
            "async job callback {} has received_at_ms earlier than parent job {} created_at_ms",
            record.callback_id,
            job.job_id
        );
    }

    if record.event_type == "job.completed" {
        if let Some(job_completed_at_ms) = job.completed_at_ms {
            if record.received_at_ms < job_completed_at_ms {
                bail!(
                    "async job callback {} event_type job.completed has received_at_ms earlier than parent job {} completed_at_ms",
                    record.callback_id,
                    job.job_id
                );
            }
        }
    }

    if job.completed_at_ms.is_some() && record.event_type != "job.completed" {
        bail!(
            "async job callback {} event_type {} must be job.completed for parent job {}",
            record.callback_id,
            record.event_type,
            job.job_id
        );
    }

    if record.event_type == "job.completed" {
        if let Some(dedupe_key) = record.dedupe_key.as_deref() {
            if !dedupe_key.ends_with(":completed") {
                bail!(
                    "async job callback {} dedupe_key {} must end with :completed for event_type job.completed",
                    record.callback_id,
                    dedupe_key
                );
            }
        }
    }

    Ok(())
}

fn ensure_async_job_callback_payload_matches_parent(
    record: &AsyncJobCallbackRecord,
    job: &AsyncJobRecord,
    payload: &serde_json::Value,
) -> Result<()> {
    if let Some(payload_status) = payload.get("status").and_then(serde_json::Value::as_str) {
        if payload_status != job.status.as_str() {
            bail!(
                "async job callback {} payload status {} must match parent job {} status {}",
                record.callback_id,
                payload_status,
                job.job_id,
                job.status.as_str()
            );
        }
    }

    if let Some(payload_provider) = payload.get("provider").and_then(serde_json::Value::as_str) {
        if let Some(job_provider_id) = job.provider_id.as_deref() {
            let normalized_job_provider = normalize_async_job_payload_provider(job_provider_id);
            if payload_provider != job_provider_id && payload_provider != normalized_job_provider {
                bail!(
                    "async job callback {} payload provider {} must match parent job {} provider {} ({})",
                    record.callback_id,
                    payload_provider,
                    job.job_id,
                    job_provider_id,
                    normalized_job_provider
                );
            }
        }
    }

    Ok(())
}

fn normalize_async_job_payload_provider(provider_id: &str) -> String {
    let without_prefix = provider_id.strip_prefix("provider-").unwrap_or(provider_id);
    for suffix in ["-official", "-main", "-local"] {
        if let Some(stripped) = without_prefix.strip_suffix(suffix) {
            return stripped.to_owned();
        }
    }
    without_prefix.to_owned()
}

fn expected_async_job_asset_mime_type(asset_kind: &str) -> Option<&'static str> {
    match asset_kind {
        "json" => Some("application/json"),
        "markdown" => Some("text/markdown"),
        _ => None,
    }
}

fn expected_async_job_asset_extension(asset_kind: &str) -> Option<&'static str> {
    match asset_kind {
        "json" => Some(".json"),
        "markdown" => Some(".md"),
        _ => None,
    }
}

fn async_job_path_leaf(path: &str) -> Option<&str> {
    path.split(['?', '#'])
        .next()
        .unwrap_or(path)
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .filter(|leaf| !leaf.is_empty())
}

fn ensure_routing_decision_matches_compiled_snapshot(
    record: &RoutingDecisionLog,
    snapshot_id: &str,
    snapshot: &CompiledRoutingSnapshotRecord,
) -> Result<()> {
    ensure_compiled_snapshot_field_matches(
        "routing decision log",
        &record.decision_id,
        snapshot_id,
        "capability",
        &record.capability.as_str(),
        &snapshot.capability.as_str(),
    )?;
    ensure_compiled_snapshot_field_matches(
        "routing decision log",
        &record.decision_id,
        snapshot_id,
        "route_key",
        &record.route_key.as_str(),
        &snapshot.route_key.as_str(),
    )?;
    ensure_compiled_snapshot_field_matches(
        "routing decision log",
        &record.decision_id,
        snapshot_id,
        "strategy",
        &record.strategy.as_str(),
        &snapshot.strategy.as_str(),
    )?;
    let record_matched_policy_id = record.matched_policy_id.as_deref();
    let snapshot_matched_policy_id = snapshot.matched_policy_id.as_deref();
    ensure_compiled_snapshot_field_matches(
        "routing decision log",
        &record.decision_id,
        snapshot_id,
        "matched_policy_id",
        &record_matched_policy_id,
        &snapshot_matched_policy_id,
    )?;
    let record_applied_routing_profile_id = record.applied_routing_profile_id.as_deref();
    let snapshot_applied_routing_profile_id = snapshot.applied_routing_profile_id.as_deref();
    ensure_compiled_snapshot_field_matches(
        "routing decision log",
        &record.decision_id,
        snapshot_id,
        "applied_routing_profile_id",
        &record_applied_routing_profile_id,
        &snapshot_applied_routing_profile_id,
    )
}

fn ensure_billing_event_matches_compiled_snapshot(
    record: &BillingEventRecord,
    snapshot_id: &str,
    snapshot: &CompiledRoutingSnapshotRecord,
) -> Result<()> {
    ensure_compiled_snapshot_field_matches(
        "billing event",
        &record.event_id,
        snapshot_id,
        "capability",
        &record.capability.as_str(),
        &snapshot.capability.as_str(),
    )?;
    ensure_compiled_snapshot_field_matches(
        "billing event",
        &record.event_id,
        snapshot_id,
        "route_key",
        &record.route_key.as_str(),
        &snapshot.route_key.as_str(),
    )?;
    let record_applied_routing_profile_id = record.applied_routing_profile_id.as_deref();
    let snapshot_applied_routing_profile_id = snapshot.applied_routing_profile_id.as_deref();
    ensure_compiled_snapshot_field_matches(
        "billing event",
        &record.event_id,
        snapshot_id,
        "applied_routing_profile_id",
        &record_applied_routing_profile_id,
        &snapshot_applied_routing_profile_id,
    )
}

fn ensure_compiled_snapshot_field_matches<T>(
    record_kind: &str,
    record_id: &str,
    snapshot_id: &str,
    field_name: &str,
    actual: &T,
    expected: &T,
) -> Result<()>
where
    T: PartialEq + std::fmt::Debug,
{
    if actual == expected {
        return Ok(());
    }

    bail!(
        "{} {} field {} = {:?} does not match compiled routing snapshot {} field {} = {:?}",
        record_kind,
        record_id,
        field_name,
        actual,
        snapshot_id,
        field_name,
        expected
    )
}

fn ensure_request_meter_fact_pricing_plan_workspace_or_shared(
    label: &str,
    record: &RequestMeterFactRecord,
    pricing_plan: &PricingPlanRecord,
) -> Result<()> {
    if pricing_plan.tenant_id == record.tenant_id
        && pricing_plan.organization_id == record.organization_id
    {
        return Ok(());
    }

    if pricing_plan.is_platform_shared() {
        return Ok(());
    }

    bail!(
        "bootstrap {} record {} references pricing plan {} outside request workspace ownership without platform_shared scope",
        label,
        record.request_id,
        pricing_plan.pricing_plan_id
    )
}

fn billing_event_matches_catalog(
    record: &BillingEventRecord,
    model_variant_keys: &HashSet<String>,
    provider_models: &[ProviderModelRecord],
) -> bool {
    if record.route_key == record.usage_model
        && model_variant_keys.contains(&format!("{}::{}", record.usage_model, record.provider_id))
    {
        return true;
    }

    provider_models.iter().any(|provider_model| {
        provider_model.is_active
            && provider_model.proxy_provider_id == record.provider_id
            && record
                .channel_id
                .as_deref()
                .is_none_or(|channel_id| provider_model.channel_id == channel_id)
            && provider_model.model_id == record.route_key
            && (provider_model.provider_model_id == record.usage_model
                || provider_model.model_id == record.usage_model)
    })
}

fn provider_supports_catalog_model(
    provider_id: &str,
    model_id: &str,
    models: &[ModelCatalogEntry],
    provider_models: &[ProviderModelRecord],
) -> bool {
    models
        .iter()
        .any(|model| model.provider_id == provider_id && model.external_name == model_id)
        || provider_models.iter().any(|provider_model| {
            provider_model.is_active
                && provider_model.proxy_provider_id == provider_id
                && provider_model.model_id == model_id
        })
}

fn provider_supports_catalog_model_capability(
    provider_id: &str,
    model_id: &str,
    capability: &sdkwork_api_domain_catalog::ModelCapability,
    models: &[ModelCatalogEntry],
    provider_models: &[ProviderModelRecord],
) -> bool {
    models.iter().any(|model| {
        model.provider_id == provider_id
            && model.external_name == model_id
            && model
                .capabilities
                .iter()
                .any(|model_capability| model_capability == capability)
    }) || provider_models.iter().any(|provider_model| {
        provider_model.is_active
            && provider_model.proxy_provider_id == provider_id
            && provider_model.model_id == model_id
            && provider_model
                .capabilities
                .iter()
                .any(|model_capability| model_capability == capability)
    })
}

fn provider_channel_supports_catalog_model_capability(
    provider_id: &str,
    channel_id: &str,
    model_id: &str,
    capability: &sdkwork_api_domain_catalog::ModelCapability,
    models: &[ModelCatalogEntry],
    provider_models: &[ProviderModelRecord],
) -> bool {
    models.iter().any(|model| {
        model.provider_id == provider_id
            && model.external_name == model_id
            && model
                .capabilities
                .iter()
                .any(|model_capability| model_capability == capability)
    }) || provider_models.iter().any(|provider_model| {
        provider_model.is_active
            && provider_model.proxy_provider_id == provider_id
            && provider_model.channel_id == channel_id
            && provider_model.model_id == model_id
            && provider_model
                .capabilities
                .iter()
                .any(|model_capability| model_capability == capability)
    })
}

fn build_tenant_accessible_executable_provider_ids(
    provider_accounts: &[ProviderAccountRecord],
    extension_instances: &HashMap<&str, &ExtensionInstance>,
    extension_installations: &HashMap<&str, &ExtensionInstallation>,
) -> HashMap<String, HashSet<String>> {
    let mut tenant_accessible_provider_ids = HashMap::<String, HashSet<String>>::new();
    let mut shared_provider_ids = HashSet::<String>::new();

    for record in provider_accounts.iter().filter(|record| record.enabled) {
        let Some(instance) = extension_instances.get(record.execution_instance_id.as_str()) else {
            continue;
        };
        if !instance.enabled {
            continue;
        }
        let Some(installation) = extension_installations.get(instance.installation_id.as_str())
        else {
            continue;
        };
        if !installation.enabled {
            continue;
        }

        if record.owner_scope == "tenant" {
            if let Some(owner_tenant_id) = record.owner_tenant_id.as_deref() {
                tenant_accessible_provider_ids
                    .entry(owner_tenant_id.to_owned())
                    .or_default()
                    .insert(record.provider_id.clone());
            }
            continue;
        }

        shared_provider_ids.insert(record.provider_id.clone());
    }

    tenant_accessible_provider_ids.insert(String::new(), shared_provider_ids);
    tenant_accessible_provider_ids
}

fn tenant_accessible_provider_ids_for(
    tenant_accessible_provider_ids: &HashMap<String, HashSet<String>>,
    tenant_id: &str,
) -> HashSet<String> {
    let mut provider_ids = tenant_accessible_provider_ids
        .get("")
        .cloned()
        .unwrap_or_default();
    if let Some(tenant_specific) = tenant_accessible_provider_ids.get(tenant_id) {
        provider_ids.extend(tenant_specific.iter().cloned());
    }
    provider_ids
}

fn ensure_supported_model_price_source_kind(record_id: &str, value: &str) -> Result<()> {
    match value {
        "official" | "proxy" | "local" | "reference" => Ok(()),
        _ => bail!(
            "bootstrap model_prices.price_source_kind record {} contains unsupported value {}",
            record_id,
            value
        ),
    }
}

fn ensure_finite_non_negative_number(label: &str, record_id: &str, value: f64) -> Result<()> {
    if value.is_finite() && value >= 0.0 {
        return Ok(());
    }

    bail!(
        "bootstrap {} record {} must be a finite non-negative number",
        label,
        record_id
    )
}

fn ensure_json_object(label: &str, record_id: &str, value: &serde_json::Value) -> Result<()> {
    if value.is_object() {
        return Ok(());
    }

    bail!(
        "bootstrap {} record {} must contain a json object",
        label,
        record_id
    )
}

fn ensure_json_object_from_str(label: &str, record_id: &str, value: &str) -> Result<()> {
    let parsed = serde_json::from_str::<serde_json::Value>(value).with_context(|| {
        format!(
            "bootstrap {} record {} must contain valid json",
            label, record_id
        )
    })?;
    ensure_json_object(label, record_id, &parsed)
}

fn ensure_json_value_from_str(label: &str, record_id: &str, value: &str) -> Result<()> {
    serde_json::from_str::<serde_json::Value>(value).with_context(|| {
        format!(
            "bootstrap {} record {} must contain valid json",
            label, record_id
        )
    })?;
    Ok(())
}

fn ensure_non_empty_field(label: &str, record_id: &str, value: &str) -> Result<()> {
    if !value.trim().is_empty() {
        return Ok(());
    }

    bail!("bootstrap {} record {} must not be empty", label, record_id)
}

fn ensure_valid_identity_email(label: &str, record_id: &str, email: &str) -> Result<()> {
    let normalized = normalize_identity_email(email);
    if !normalized.is_empty() && normalized.contains('@') {
        return Ok(());
    }

    bail!(
        "bootstrap {} record {} must contain a valid email",
        label,
        record_id
    )
}

fn ensure_identity_password_material(
    label: &str,
    record_id: &str,
    password_salt: &str,
    password_hash: &str,
) -> Result<()> {
    ensure_non_empty_field("identity.password_salt", record_id, password_salt)?;
    ensure_non_empty_field(label, record_id, password_hash)?;
    PasswordHash::new(password_hash).map_err(|error| {
        anyhow::anyhow!(
            "bootstrap {} record {} contains an invalid password hash: {}",
            label,
            record_id,
            error
        )
    })?;
    Ok(())
}

fn normalize_identity_email(email: &str) -> String {
    email.trim().to_ascii_lowercase()
}

fn sha256_hex(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    let digest = hasher.finalize();
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        encoded.push_str(&format!("{byte:02x}"));
    }
    encoded
}

fn collect_ids<I, S>(values: I) -> HashSet<String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    values.into_iter().map(Into::into).collect()
}

fn normalize_coupon_code(code: &str) -> String {
    code.trim().to_ascii_uppercase()
}

fn provider_health_snapshot_key(record: &ProviderHealthSnapshot) -> String {
    format!(
        "{}::{}::{}",
        record.provider_id,
        record.runtime,
        record.instance_id.as_deref().unwrap_or("")
    )
}
