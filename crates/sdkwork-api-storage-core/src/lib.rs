use anyhow::Result;
use async_trait::async_trait;
use sdkwork_api_domain_billing::{
    AccountBenefitLotRecord, AccountCommerceReconciliationStateRecord, AccountHoldAllocationRecord,
    AccountHoldRecord, AccountLedgerAllocationRecord, AccountLedgerEntryRecord, AccountRecord,
    AccountType, BillingEventRecord, LedgerEntry, PricingPlanRecord, PricingRateRecord,
    QuotaPolicy, RequestSettlementRecord,
};
use sdkwork_api_domain_catalog::{
    CatalogPublicationLifecycleAuditRecord, Channel, ChannelModelRecord, ModelCatalogEntry,
    ModelPriceRecord, ProviderAccountRecord, ProviderModelRecord, ProxyProvider,
};
use sdkwork_api_domain_commerce::{
    CommerceOrderRecord, CommercePaymentAttemptRecord, CommercePaymentEventRecord,
    CommerceReconciliationItemRecord, CommerceReconciliationRunRecord, CommerceRefundRecord,
    CommerceWebhookDeliveryAttemptRecord, CommerceWebhookInboxRecord,
    PaymentMethodCredentialBindingRecord, PaymentMethodRecord, ProjectMembershipRecord,
};
use sdkwork_api_domain_credential::{OfficialProviderConfig, UpstreamCredential};
use sdkwork_api_domain_identity::{
    AdminUserRecord, ApiKeyGroupRecord, CanonicalApiKeyRecord, GatewayApiKeyRecord,
    IdentityBindingRecord, IdentityUserRecord, PortalUserRecord,
};
use sdkwork_api_domain_jobs::{
    AsyncJobAssetRecord, AsyncJobAttemptRecord, AsyncJobCallbackRecord, AsyncJobRecord,
};
use sdkwork_api_domain_marketing::{
    CampaignBudgetLifecycleAuditRecord, CampaignBudgetRecord, CouponCodeLifecycleAuditRecord,
    CouponCodeRecord, CouponRedemptionRecord, CouponReservationRecord, CouponRollbackRecord,
    CouponRollbackStatus, CouponTemplateLifecycleAuditRecord, CouponTemplateRecord,
    MarketingCampaignLifecycleAuditRecord, MarketingCampaignRecord, MarketingOutboxEventRecord,
};
use sdkwork_api_domain_rate_limit::{
    RateLimitCheckResult, RateLimitPolicy, RateLimitWindowSnapshot,
};
use sdkwork_api_domain_routing::{
    CompiledRoutingSnapshotRecord, ProjectRoutingPreferences, ProviderHealthSnapshot,
    RoutingDecisionLog, RoutingPolicy, RoutingProfileRecord,
};
use sdkwork_api_domain_tenant::{Project, Tenant};
use sdkwork_api_domain_usage::{RequestMeterFactRecord, RequestMeterMetricRecord, UsageRecord};
use sdkwork_api_extension_core::{ExtensionInstallation, ExtensionInstance};
use sdkwork_api_secret_core::SecretEnvelope;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, RwLock};

mod types;
mod admin_facets;
mod admin_store;
mod kernel_support;
mod identity_kernel_store;
mod account_kernel_store;
mod marketing_store;
mod account_transaction;

pub use account_kernel_store::*;
pub use account_transaction::*;
pub use admin_facets::*;
pub use admin_store::*;
pub use identity_kernel_store::*;
pub use marketing_store::*;
pub use types::*;

pub(crate) use kernel_support::*;
