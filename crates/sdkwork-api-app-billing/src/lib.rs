use anyhow::{ensure, Result};
use async_trait::async_trait;
use sdkwork_api_app_identity::{
    gateway_auth_subject_from_request_context,
    GatewayRequestContext as IdentityGatewayRequestContext,
};
use sdkwork_api_domain_billing::{
    AccountBenefitLotRecord, AccountBenefitLotStatus, AccountBenefitSourceType, AccountBenefitType,
    AccountCommerceReconciliationStateRecord, AccountHoldAllocationRecord, AccountHoldRecord,
    AccountHoldStatus, AccountLedgerAllocationRecord, AccountLedgerEntryRecord,
    AccountLedgerEntryType, AccountRecord, AccountStatus, AccountType,
    BillingEventAccountingModeSummary, BillingEventCapabilitySummary, BillingEventGroupSummary,
    BillingEventProjectSummary, BillingEventRecord, BillingEventSummary, BillingSummary,
    LedgerEntry, PricingPlanRecord, PricingRateRecord, ProjectBillingSummary,
    RequestSettlementRecord, RequestSettlementStatus,
};
use sdkwork_api_domain_identity::GatewayAuthSubject;
use sdkwork_api_policy_quota::{
    builtin_quota_policy_registry, QuotaPolicyExecutionInput, STRICTEST_LIMIT_QUOTA_POLICY_ID,
};
use sdkwork_api_storage_core::{
    AccountKernelStore, AccountKernelTransaction, AccountKernelTransactionExecutor, AdminStore,
};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use utoipa::ToSchema;

mod account_balance;
mod account_mutations;
mod billing_events;
mod billing_inputs;
mod billing_kernels;
mod billing_summary;
mod billing_support;
mod commerce_credits;
mod pricing_lifecycle;

pub(crate) use billing_support::{
    account_ledger_allocation_id,
    account_ledger_entry_id,
    build_commerce_order_credit_scope_json,
    commerce_order_issue_ledger_allocation_id,
    commerce_order_issue_ledger_entry_id,
    commerce_order_lot_id,
    commerce_order_refund_ledger_allocation_id,
    commerce_order_refund_ledger_entry_id,
    commerce_order_source_id,
    eligible_lots_for_hold,
    ensure_quantity_matches,
    free_quantity,
    load_account_ledger_allocations_and_lots,
    load_hold_allocations_and_lots,
    validate_commerce_order_credit_lot,
    write_account_ledger_entry,
    ACCOUNTING_EPSILON,
    HOLD_CREATE_LEDGER_SUFFIX,
    HOLD_RELEASE_LEDGER_SUFFIX,
    SETTLEMENT_CAPTURE_LEDGER_SUFFIX,
    SETTLEMENT_RELEASE_LEDGER_SUFFIX,
};

pub use account_balance::*;
pub use account_mutations::*;
pub use billing_events::*;
pub use billing_inputs::*;
pub use billing_kernels::*;
pub use billing_summary::*;
pub use commerce_credits::*;
pub use pricing_lifecycle::*;
pub use sdkwork_api_domain_billing::{BillingAccountingMode, QuotaCheckResult, QuotaPolicy};
