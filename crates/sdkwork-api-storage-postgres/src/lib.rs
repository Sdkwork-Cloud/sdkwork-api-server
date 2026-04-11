use anyhow::{anyhow, Result};
use async_trait::async_trait;
use sdkwork_api_domain_billing::{
    AccountBenefitLotRecord, AccountBenefitLotStatus, AccountBenefitSourceType, AccountBenefitType,
    AccountCommerceReconciliationStateRecord, AccountHoldAllocationRecord, AccountHoldRecord,
    AccountHoldStatus, AccountLedgerAllocationRecord, AccountLedgerEntryRecord,
    AccountLedgerEntryType, AccountRecord, AccountStatus, AccountType, BillingAccountingMode,
    BillingEventRecord, LedgerEntry, PricingPlanOwnershipScope, PricingPlanRecord,
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
    CommercePaymentEventRecord, CommerceReconciliationItemRecord, CommerceReconciliationRunRecord,
    CommerceRefundRecord, CommerceWebhookDeliveryAttemptRecord, CommerceWebhookInboxRecord,
    PaymentMethodCredentialBindingRecord, PaymentMethodRecord, ProjectMembershipRecord,
};
use sdkwork_api_domain_credential::{OfficialProviderConfig, UpstreamCredential};
use sdkwork_api_domain_identity::{
    AdminUserRecord, ApiKeyGroupRecord, GatewayApiKeyRecord, PortalUserRecord,
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
    ExtensionRuntimeRolloutParticipantRecord, ExtensionRuntimeRolloutRecord,
    MarketingKernelTransaction, MarketingKernelTransactionExecutor, MarketingStore,
    ServiceRuntimeNodeRecord, StandaloneConfigRolloutParticipantRecord,
    StandaloneConfigRolloutRecord, StorageDialect,
};
use serde_json::Value;
use sqlx::{
    postgres::{PgPoolOptions, PgRow},
    PgPool, Postgres, Row, Transaction,
};
use std::collections::HashMap;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

mod account_kernel_store;
mod account_kernel_transaction;
mod account_support;
mod admin_store_impl;
mod catalog_store;
mod commerce_finance_store;
mod commerce_order_store;
mod commerce_payment_store;
mod commerce_store_mappers;
mod gateway_store;
mod identity_store;
mod jobs_store;
mod marketing_store_impl;
mod migrations;
mod postgres_migration_billing_schema;
mod postgres_migration_catalog_gateway_schema;
mod postgres_migration_commerce_jobs_schema;
mod postgres_migration_compat;
mod postgres_migration_identity_schema;
mod postgres_migration_marketing_schema;
mod postgres_migration_routing_schema;
mod postgres_migration_runtime_schema;
mod postgres_migration_seed;
mod postgres_support;
mod routing_store;
mod runtime_store;
mod tenant_store;
mod usage_billing_store;

pub(crate) use account_support::*;
pub(crate) use postgres_migration_billing_schema::apply_postgres_billing_schema;
pub(crate) use postgres_migration_catalog_gateway_schema::apply_postgres_catalog_gateway_schema;
pub(crate) use postgres_migration_commerce_jobs_schema::apply_postgres_commerce_jobs_schema;
pub(crate) use postgres_migration_compat::{
    apply_postgres_legacy_table_compatibility, migrate_postgres_legacy_catalog_records,
    recreate_postgres_legacy_compatibility_views,
};
pub(crate) use postgres_migration_identity_schema::apply_postgres_identity_schema;
pub(crate) use postgres_migration_marketing_schema::apply_postgres_marketing_schema;
pub(crate) use postgres_migration_routing_schema::apply_postgres_routing_schema;
pub(crate) use postgres_migration_runtime_schema::apply_postgres_runtime_schema;
pub(crate) use postgres_migration_seed::seed_postgres_builtin_channels;
pub(crate) use postgres_support::*;

pub use migrations::run_migrations;

pub fn dialect_name() -> &'static str {
    "postgres"
}

pub struct PostgresAdminStore {
    pool: PgPool,
}

impl PostgresAdminStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
