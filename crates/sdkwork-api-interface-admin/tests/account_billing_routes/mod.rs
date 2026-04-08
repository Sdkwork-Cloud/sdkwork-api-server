use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use sdkwork_api_domain_billing::{
    AccountBenefitLotRecord, AccountBenefitLotStatus, AccountBenefitSourceType, AccountBenefitType,
    AccountHoldRecord, AccountHoldStatus, AccountLedgerAllocationRecord, AccountLedgerEntryRecord,
    AccountLedgerEntryType, AccountRecord, AccountStatus, AccountType, PricingPlanRecord,
    PricingRateRecord, RequestSettlementRecord, RequestSettlementStatus,
};
use sdkwork_api_domain_commerce::{
    CommerceOrderRecord, CommercePaymentEventProcessingStatus, CommercePaymentEventRecord,
};
use sdkwork_api_domain_marketing::{
    CouponBenefitSpec, CouponCodeRecord, CouponCodeStatus, CouponDistributionKind,
    CouponRedemptionRecord, CouponRedemptionStatus, CouponReservationRecord,
    CouponReservationStatus, CouponRestrictionSpec, CouponRollbackRecord, CouponRollbackStatus,
    CouponRollbackType, CouponTemplateRecord, CouponTemplateStatus, MarketingBenefitKind,
    MarketingCampaignRecord, MarketingCampaignStatus, MarketingStackingPolicy,
    MarketingSubjectScope,
};
use sdkwork_api_storage_core::{AccountKernelStore, AdminStore};
use sdkwork_api_storage_sqlite::SqliteAdminStore;
use serde_json::Value;
use sqlx::SqlitePool;
use tower::ServiceExt;

mod billing_views;
mod commerce_audit;
mod commerce_mutations;
mod pricing_crud;
mod pricing_lifecycle;
mod support;

use support::*;
