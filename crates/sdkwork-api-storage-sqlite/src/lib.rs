use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use sdkwork_api_domain_billing::{
    AccountBenefitLotRecord, AccountBenefitLotStatus, AccountBenefitSourceType, AccountBenefitType,
    AccountCommerceReconciliationStateRecord, AccountHoldAllocationRecord, AccountHoldRecord,
    AccountHoldStatus, AccountLedgerAllocationRecord, AccountLedgerEntryRecord,
    AccountLedgerEntryType, AccountRecord, AccountStatus, AccountType, BillingAccountingMode,
    BillingEventRecord, LedgerEntry, PricingPlanRecord,
    PricingRateRecord, QuotaPolicy, RequestSettlementRecord, RequestSettlementStatus,
};
use sdkwork_api_domain_catalog::{
    normalize_provider_extension_id, normalize_provider_protocol_kind,
    CatalogPublicationLifecycleAction, CatalogPublicationLifecycleAuditOutcome,
    CatalogPublicationLifecycleAuditRecord, Channel, ChannelModelRecord, ModelCapability,
    ModelCatalogEntry, ModelPriceRecord, ModelPriceTier, ProviderChannelBinding,
    ProviderAccountRecord, ProviderModelRecord, ProxyProvider,
};
use sdkwork_api_domain_commerce::{
    CommerceOrderRecord, CommercePaymentAttemptRecord, CommercePaymentEventProcessingStatus,
    CommercePaymentEventRecord, CommerceReconciliationItemRecord,
    CommerceReconciliationRunRecord, CommerceRefundRecord,
    CommerceWebhookDeliveryAttemptRecord, CommerceWebhookInboxRecord,
    PaymentMethodCredentialBindingRecord, PaymentMethodRecord, ProjectMembershipRecord,
};
use sdkwork_api_domain_credential::{OfficialProviderConfig, UpstreamCredential};
use sdkwork_api_domain_identity::{
    AdminUserRecord, ApiKeyGroupRecord, CanonicalApiKeyRecord, GatewayApiKeyRecord,
    IdentityBindingRecord, IdentityUserRecord, PortalUserRecord,
};
use sdkwork_api_domain_jobs::{
    AsyncJobAssetRecord, AsyncJobAttemptRecord, AsyncJobAttemptStatus, AsyncJobCallbackRecord,
    AsyncJobCallbackStatus, AsyncJobRecord, AsyncJobStatus,
};
use sdkwork_api_domain_marketing::{
    CampaignBudgetLifecycleAuditRecord, CampaignBudgetRecord, CampaignBudgetStatus,
    CouponCodeLifecycleAuditRecord, CouponCodeRecord, CouponCodeStatus,
    CouponDistributionKind, CouponRedemptionRecord, CouponRedemptionStatus,
    CouponReservationRecord, CouponReservationStatus, CouponRollbackRecord, CouponRollbackStatus,
    CouponRollbackType, CouponTemplateLifecycleAuditRecord, CouponTemplateRecord,
    CouponTemplateStatus, MarketingCampaignLifecycleAuditRecord, MarketingCampaignRecord,
    MarketingCampaignStatus, MarketingOutboxEventRecord, MarketingOutboxEventStatus,
    MarketingSubjectScope,
};
use sdkwork_api_domain_rate_limit::{
    RateLimitCheckResult, RateLimitPolicy, RateLimitWindowSnapshot,
};
use sdkwork_api_domain_routing::{
    CompiledRoutingSnapshotRecord, ProjectRoutingPreferences, ProviderHealthSnapshot,
    RoutingCandidateAssessment, RoutingDecisionLog, RoutingDecisionSource, RoutingPolicy,
    RoutingProfileRecord, RoutingStrategy,
};
use sdkwork_api_domain_tenant::{Project, Tenant};
use sdkwork_api_domain_usage::{
    RequestMeterFactRecord, RequestMeterMetricRecord, RequestStatus, UsageCaptureStatus,
    UsageRecord,
};
use sdkwork_api_extension_core::{ExtensionInstallation, ExtensionInstance, ExtensionRuntime};
use sdkwork_api_secret_core::SecretEnvelope;
use sdkwork_api_storage_core::{
    AccountKernelStore, AccountKernelTransaction, AccountKernelTransactionExecutor, AdminStore,
    AtomicCouponConfirmationCommand, AtomicCouponConfirmationResult, AtomicCouponReleaseCommand,
    AtomicCouponReleaseResult, AtomicCouponReservationCommand, AtomicCouponReservationResult,
    AtomicCouponRollbackCommand, AtomicCouponRollbackCompensationCommand,
    AtomicCouponRollbackCompensationResult, AtomicCouponRollbackResult,
    ExtensionRuntimeRolloutParticipantRecord, ExtensionRuntimeRolloutRecord, IdentityKernelStore,
    MarketingKernelTransaction, MarketingKernelTransactionExecutor, MarketingStore,
    ServiceRuntimeNodeRecord, StandaloneConfigRolloutParticipantRecord,
    StandaloneConfigRolloutRecord, StorageDialect,
};
use serde_json::Value;
use sqlx::{
    sqlite::{SqlitePoolOptions, SqliteRow},
    Row, Sqlite, SqlitePool, Transaction,
};

mod account_kernel_store;
mod account_kernel_transaction;
mod account_support;
mod admin_store_impl;
mod catalog_store;
mod catalog_support;
mod commerce_store;
mod identity_kernel_store;
mod identity_store;
mod jobs_store;
mod marketing_kernel_transaction;
mod marketing_store_impl;
mod marketing_support;
mod migrations;
mod routing_store;
mod runtime_store;
mod sqlite_migration_billing_schema;
mod sqlite_migration_catalog_gateway_compat;
mod sqlite_migration_catalog_gateway_schema;
mod sqlite_migration_commerce_jobs_schema;
mod sqlite_migration_identity_schema;
mod sqlite_migration_legacy_compat;
mod sqlite_migration_marketing_schema;
mod sqlite_migration_routing_schema;
mod sqlite_migration_runtime_schema;
mod sqlite_support;
mod tenant_store;
mod usage_billing_store;

#[cfg(test)]
mod tests;

pub(crate) use account_support::*;
pub(crate) use catalog_support::*;
pub(crate) use marketing_support::*;
pub(crate) use sqlite_migration_billing_schema::apply_sqlite_billing_schema;
pub(crate) use sqlite_migration_catalog_gateway_compat::apply_sqlite_catalog_gateway_compatibility;
pub(crate) use sqlite_migration_catalog_gateway_schema::apply_sqlite_catalog_gateway_schema;
pub(crate) use sqlite_migration_commerce_jobs_schema::apply_sqlite_commerce_jobs_schema;
pub(crate) use sqlite_migration_identity_schema::apply_sqlite_identity_schema;
pub(crate) use sqlite_migration_legacy_compat::apply_sqlite_legacy_compatibility;
pub(crate) use sqlite_migration_marketing_schema::apply_sqlite_marketing_schema;
pub(crate) use sqlite_migration_routing_schema::apply_sqlite_routing_schema;
pub(crate) use sqlite_migration_runtime_schema::apply_sqlite_runtime_schema;
pub(crate) use sqlite_support::*;

pub use migrations::run_migrations;

pub fn dialect_name() -> &'static str {
    "sqlite"
}

#[derive(Debug, Clone)]
pub struct SqliteAdminStore {
    pool: SqlitePool,
}

impl SqliteAdminStore {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}
