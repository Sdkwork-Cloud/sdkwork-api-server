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
use sdkwork_api_domain_commerce::CommerceOrderRecord;
use sdkwork_api_domain_identity::GatewayAuthSubject;
use sdkwork_api_domain_payment::PaymentOrderRecord;
use sdkwork_api_policy_quota::{
    builtin_quota_policy_registry, QuotaPolicyExecutionInput, STRICTEST_LIMIT_QUOTA_POLICY_ID,
};
use sdkwork_api_storage_core::{
    AccountKernelStore, AccountKernelTransaction, AccountKernelTransactionExecutor, AdminStore,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
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
    account_ledger_allocation_id, account_ledger_entry_id, build_commerce_order_credit_scope_json,
    commerce_order_issue_ledger_allocation_id, commerce_order_issue_ledger_entry_id,
    commerce_order_lot_id, commerce_order_refund_ledger_allocation_id,
    commerce_order_refund_ledger_entry_id, commerce_order_source_id, eligible_lots_for_hold,
    ensure_quantity_matches, free_quantity, load_account_ledger_allocations_and_lots,
    load_hold_allocations_and_lots, validate_commerce_order_credit_lot, write_account_ledger_entry,
    ACCOUNTING_EPSILON, HOLD_CREATE_LEDGER_SUFFIX, HOLD_RELEASE_LEDGER_SUFFIX,
    SETTLEMENT_CAPTURE_LEDGER_SUFFIX, SETTLEMENT_RELEASE_LEDGER_SUFFIX,
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

#[derive(Debug, Clone, PartialEq)]
pub struct CommerceOrderAccountGrantResult {
    pub account: AccountRecord,
    pub lot: Option<AccountBenefitLotRecord>,
    pub ledger_entry: Option<AccountLedgerEntryRecord>,
    pub ledger_allocation: Option<AccountLedgerAllocationRecord>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommerceOrderAccountRefundResult {
    pub account: AccountRecord,
    pub lot: Option<AccountBenefitLotRecord>,
    pub ledger_entry: Option<AccountLedgerEntryRecord>,
    pub ledger_allocation: Option<AccountLedgerAllocationRecord>,
    pub reversed_quantity: f64,
}

pub fn commerce_order_account_grant_ledger_entry_id(payment_order: &PaymentOrderRecord) -> u64 {
    stable_u64(
        "account-ledger-order-grant-v1",
        &[
            payment_order.tenant_id.to_string(),
            payment_order.organization_id.to_string(),
            payment_order.payment_order_id.clone(),
        ],
    )
}

pub fn commerce_order_account_refund_ledger_entry_id(
    payment_order: &PaymentOrderRecord,
    refund_order_id: &str,
) -> u64 {
    stable_u64(
        "account-ledger-order-refund-v1",
        &[
            payment_order.tenant_id.to_string(),
            payment_order.organization_id.to_string(),
            refund_order_id.to_owned(),
        ],
    )
}

pub async fn ensure_commerce_order_account_grant<S>(
    store: &S,
    payment_order: &PaymentOrderRecord,
    commerce_order: &CommerceOrderRecord,
    fulfilled_at_ms: u64,
) -> Result<CommerceOrderAccountGrantResult>
where
    S: AccountKernelStore + ?Sized,
{
    ensure!(
        payment_order.commerce_order_id == commerce_order.order_id,
        "payment order {} does not match commerce order {}",
        payment_order.payment_order_id,
        commerce_order.order_id
    );
    ensure!(
        payment_order.project_id == commerce_order.project_id,
        "payment order {} project {} does not match commerce order project {}",
        payment_order.payment_order_id,
        payment_order.project_id,
        commerce_order.project_id
    );

    let account = ensure_primary_account_for_payment_order(store, payment_order).await?;
    let granted_quantity = commerce_order
        .granted_units
        .saturating_add(commerce_order.bonus_units) as f64;
    if granted_quantity <= f64::EPSILON {
        return Ok(CommerceOrderAccountGrantResult {
            account,
            lot: None,
            ledger_entry: None,
            ledger_allocation: None,
        });
    }

    let source_id = stable_u64(
        "commerce-order-source-v1",
        &[
            payment_order.tenant_id.to_string(),
            payment_order.organization_id.to_string(),
            payment_order.commerce_order_id.clone(),
        ],
    );
    let scope_json = Some(
        serde_json::json!({
            "commerce_order_id": commerce_order.order_id,
            "project_id": commerce_order.project_id,
            "target_kind": commerce_order.target_kind,
            "target_id": commerce_order.target_id,
            "source": commerce_order.source,
        })
        .to_string(),
    );
    let acquired_unit_cost = if granted_quantity > f64::EPSILON {
        Some(payment_order.payable_minor as f64 / 100.0 / granted_quantity)
    } else {
        None
    };

    let lot = AccountBenefitLotRecord::new(
        stable_u64(
            "account-lot-order-grant-v1",
            &[
                payment_order.tenant_id.to_string(),
                payment_order.organization_id.to_string(),
                payment_order.payment_order_id.clone(),
            ],
        ),
        payment_order.tenant_id,
        payment_order.organization_id,
        account.account_id,
        payment_order.user_id,
        AccountBenefitType::RequestAllowance,
    )
    .with_source_type(AccountBenefitSourceType::Order)
    .with_source_id(Some(source_id))
    .with_scope_json(scope_json)
    .with_original_quantity(granted_quantity)
    .with_remaining_quantity(granted_quantity)
    .with_priority(0)
    .with_acquired_unit_cost(acquired_unit_cost)
    .with_issued_at_ms(fulfilled_at_ms)
    .with_status(AccountBenefitLotStatus::Active)
    .with_created_at_ms(payment_order.created_at_ms)
    .with_updated_at_ms(fulfilled_at_ms);
    let lot = store.insert_account_benefit_lot(&lot).await?;

    let ledger_entry = AccountLedgerEntryRecord::new(
        commerce_order_account_grant_ledger_entry_id(payment_order),
        payment_order.tenant_id,
        payment_order.organization_id,
        account.account_id,
        payment_order.user_id,
        AccountLedgerEntryType::GrantIssue,
    )
    .with_benefit_type(Some("request_allowance".to_owned()))
    .with_quantity(granted_quantity)
    .with_amount(payment_order.payable_minor as f64 / 100.0)
    .with_created_at_ms(fulfilled_at_ms);
    let ledger_entry = store
        .insert_account_ledger_entry_record(&ledger_entry)
        .await?;

    let ledger_allocation = AccountLedgerAllocationRecord::new(
        stable_u64(
            "account-ledger-alloc-order-grant-v1",
            &[
                payment_order.tenant_id.to_string(),
                payment_order.organization_id.to_string(),
                payment_order.payment_order_id.clone(),
            ],
        ),
        payment_order.tenant_id,
        payment_order.organization_id,
        ledger_entry.ledger_entry_id,
        lot.lot_id,
    )
    .with_quantity_delta(granted_quantity)
    .with_created_at_ms(fulfilled_at_ms);
    let ledger_allocation = store
        .insert_account_ledger_allocation(&ledger_allocation)
        .await?;

    Ok(CommerceOrderAccountGrantResult {
        account,
        lot: Some(lot),
        ledger_entry: Some(ledger_entry),
        ledger_allocation: Some(ledger_allocation),
    })
}

pub async fn reverse_commerce_order_account_grant<S>(
    store: &S,
    payment_order: &PaymentOrderRecord,
    commerce_order: &CommerceOrderRecord,
    refund_order_id: &str,
    refunded_amount_minor: u64,
    reversed_at_ms: u64,
) -> Result<CommerceOrderAccountRefundResult>
where
    S: AccountKernelStore + ?Sized,
{
    ensure!(
        payment_order.commerce_order_id == commerce_order.order_id,
        "payment order {} does not match commerce order {}",
        payment_order.payment_order_id,
        commerce_order.order_id
    );
    ensure!(
        payment_order.project_id == commerce_order.project_id,
        "payment order {} project {} does not match commerce order project {}",
        payment_order.payment_order_id,
        payment_order.project_id,
        commerce_order.project_id
    );
    ensure!(
        matches!(
            commerce_order.target_kind.as_str(),
            "recharge_pack" | "custom_recharge"
        ),
        "account grant reversal does not support {}",
        commerce_order.target_kind
    );

    let account = ensure_primary_account_for_payment_order(store, payment_order).await?;
    let granted_units = commerce_order
        .granted_units
        .saturating_add(commerce_order.bonus_units);
    if granted_units == 0 || refunded_amount_minor == 0 {
        return Ok(CommerceOrderAccountRefundResult {
            account,
            lot: None,
            ledger_entry: None,
            ledger_allocation: None,
            reversed_quantity: 0.0,
        });
    }

    let reversed_units = proportional_refund_units(
        granted_units,
        refunded_amount_minor,
        payment_order.payable_minor,
    )?;
    let reversed_quantity = reversed_units as f64;
    let lot_id = stable_u64(
        "account-lot-order-grant-v1",
        &[
            payment_order.tenant_id.to_string(),
            payment_order.organization_id.to_string(),
            payment_order.payment_order_id.clone(),
        ],
    );

    let ledger_entry = AccountLedgerEntryRecord::new(
        commerce_order_account_refund_ledger_entry_id(payment_order, refund_order_id),
        payment_order.tenant_id,
        payment_order.organization_id,
        account.account_id,
        payment_order.user_id,
        AccountLedgerEntryType::Refund,
    )
    .with_benefit_type(Some("request_allowance".to_owned()))
    .with_quantity(-reversed_quantity)
    .with_amount(-(refunded_amount_minor as f64 / 100.0))
    .with_created_at_ms(reversed_at_ms);

    let ledger_allocation = AccountLedgerAllocationRecord::new(
        stable_u64(
            "account-ledger-alloc-order-refund-v1",
            &[
                payment_order.tenant_id.to_string(),
                payment_order.organization_id.to_string(),
                refund_order_id.to_owned(),
            ],
        ),
        payment_order.tenant_id,
        payment_order.organization_id,
        ledger_entry.ledger_entry_id,
        lot_id,
    )
    .with_quantity_delta(-reversed_quantity)
    .with_created_at_ms(reversed_at_ms);

    store
        .apply_refund_order_account_grant_reversal(
            refund_order_id,
            lot_id,
            reversed_quantity,
            reversed_at_ms,
            &ledger_entry,
            &ledger_allocation,
        )
        .await?;

    Ok(CommerceOrderAccountRefundResult {
        account,
        lot: None,
        ledger_entry: Some(ledger_entry),
        ledger_allocation: Some(ledger_allocation),
        reversed_quantity,
    })
}

async fn ensure_primary_account_for_payment_order<S>(
    store: &S,
    payment_order: &PaymentOrderRecord,
) -> Result<AccountRecord>
where
    S: AccountKernelStore + ?Sized,
{
    if let Some(account) = store
        .find_account_record_by_owner(
            payment_order.tenant_id,
            payment_order.organization_id,
            payment_order.user_id,
            AccountType::Primary,
        )
        .await?
    {
        ensure!(
            account.status == AccountStatus::Active,
            "primary account {} is not active",
            account.account_id
        );
        ensure!(
            account.currency_code == payment_order.currency_code,
            "primary account {} currency {} does not match payment currency {}",
            account.account_id,
            account.currency_code,
            payment_order.currency_code
        );
        return Ok(account);
    }

    let account = AccountRecord::new(
        stable_u64(
            "account-primary-v1",
            &[
                payment_order.tenant_id.to_string(),
                payment_order.organization_id.to_string(),
                payment_order.user_id.to_string(),
                "primary".to_owned(),
            ],
        ),
        payment_order.tenant_id,
        payment_order.organization_id,
        payment_order.user_id,
        AccountType::Primary,
    )
    .with_currency_code(payment_order.currency_code.clone())
    .with_created_at_ms(payment_order.created_at_ms)
    .with_updated_at_ms(payment_order.updated_at_ms);

    store.insert_account_record(&account).await
}

fn stable_u64(namespace: &str, parts: &[String]) -> u64 {
    let mut digest = Sha256::new();
    digest.update(namespace.as_bytes());
    for part in parts {
        digest.update([0_u8]);
        digest.update(part.as_bytes());
    }

    let mut bytes = [0_u8; 8];
    bytes.copy_from_slice(&digest.finalize()[..8]);
    let bounded = u64::from_be_bytes(bytes) & i64::MAX as u64;
    if bounded == 0 {
        1
    } else {
        bounded
    }
}

fn proportional_refund_units(
    granted_units: u64,
    refunded_amount_minor: u64,
    payable_minor: u64,
) -> Result<u64> {
    ensure!(
        payable_minor > 0,
        "payable amount must be positive for refund reversal"
    );
    ensure!(
        refunded_amount_minor <= payable_minor,
        "refunded amount {refunded_amount_minor} exceeds payable amount {payable_minor}"
    );

    let numerator = u128::from(granted_units) * u128::from(refunded_amount_minor);
    let rounded = (numerator + (u128::from(payable_minor) / 2)) / u128::from(payable_minor);
    let reversed_units = u64::try_from(rounded)?;
    ensure!(
        reversed_units > 0,
        "refunded amount {refunded_amount_minor} is too small to reverse any granted quantity"
    );
    Ok(reversed_units.min(granted_units))
}
