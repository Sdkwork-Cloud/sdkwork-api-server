use anyhow::{ensure, Context, Result};
use sdkwork_api_app_billing::{
    commerce_order_account_grant_ledger_entry_id, commerce_order_account_refund_ledger_entry_id,
    ensure_commerce_order_account_grant, reverse_commerce_order_account_grant,
    summarize_account_balance, AccountBalanceSnapshot,
};
use sdkwork_api_app_commerce::settle_portal_commerce_order_from_verified_payment;
use sdkwork_api_domain_billing::{
    AccountBenefitLotRecord, AccountLedgerAllocationRecord, AccountLedgerEntryRecord,
    AccountRecord, AccountType,
};
use sdkwork_api_domain_commerce::CommerceOrderRecord;
use sdkwork_api_domain_identity::{IdentityBindingRecord, IdentityUserRecord, PortalUserRecord};
use sdkwork_api_domain_payment::{
    FinanceDirection, FinanceEntryCode, FinanceJournalEntryRecord, FinanceJournalLineRecord,
    PaymentAttemptRecord, PaymentAttemptStatus, PaymentCallbackEventRecord,
    PaymentCallbackProcessingStatus, PaymentChannelPolicyRecord, PaymentGatewayAccountRecord,
    PaymentOrderRecord, PaymentOrderStatus, PaymentProviderCode, PaymentRefundStatus,
    PaymentSessionKind, PaymentSessionRecord, PaymentSessionStatus, PaymentTransactionKind,
    PaymentTransactionRecord, ReconciliationMatchStatus, ReconciliationMatchSummaryRecord,
    RefundOrderRecord, RefundOrderStatus,
};
use sdkwork_api_storage_core::{
    AdminStore, CommercialKernelStore, IdentityKernelStore, PaymentKernelStore,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;

const PORTAL_IDENTITY_BINDING_TYPE: &str = "portal_user";
const PORTAL_IDENTITY_BINDING_ISSUER: &str = "sdkwork-portal";

pub fn service_name() -> &'static str {
    "payment-service"
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaymentSubjectScope {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub user_id: u64,
}

impl PaymentSubjectScope {
    pub fn new(tenant_id: u64, organization_id: u64, user_id: u64) -> Self {
        Self {
            tenant_id,
            organization_id,
            user_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommerceCheckoutMethod {
    pub id: String,
    pub label: String,
    pub detail: String,
    pub action: String,
    pub availability: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommerceCheckoutBridge {
    pub session_status: String,
    pub provider: String,
    pub mode: String,
    pub reference: String,
    pub guidance: String,
    pub methods: Vec<CommerceCheckoutMethod>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnsuredCommercePaymentCheckout {
    pub checkout: CommerceCheckoutBridge,
    pub payment_order_opt: Option<PaymentOrderRecord>,
    pub payment_attempt_opt: Option<PaymentAttemptRecord>,
    pub payment_session_opt: Option<PaymentSessionRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaymentCallbackIntakeRequest {
    pub scope: PaymentSubjectScope,
    pub provider_code: PaymentProviderCode,
    pub gateway_account_id: String,
    pub event_type: String,
    pub event_identity: String,
    pub dedupe_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payment_order_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payment_attempt_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_transaction_id: Option<String>,
    pub signature_status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount_minor: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fee_minor: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub net_amount_minor: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload_json: Option<String>,
    pub received_at_ms: u64,
}

impl PaymentCallbackIntakeRequest {
    pub fn new(
        scope: PaymentSubjectScope,
        provider_code: PaymentProviderCode,
        gateway_account_id: impl Into<String>,
        event_type: impl Into<String>,
        event_identity: impl Into<String>,
        dedupe_key: impl Into<String>,
        received_at_ms: u64,
    ) -> Self {
        Self {
            scope,
            provider_code,
            gateway_account_id: gateway_account_id.into(),
            event_type: event_type.into(),
            event_identity: event_identity.into(),
            dedupe_key: dedupe_key.into(),
            payment_order_id: None,
            payment_attempt_id: None,
            provider_transaction_id: None,
            signature_status: "pending".to_owned(),
            provider_status: None,
            currency_code: None,
            amount_minor: None,
            fee_minor: None,
            net_amount_minor: None,
            payload_json: None,
            received_at_ms,
        }
    }

    pub fn with_payment_order_id(mut self, payment_order_id: Option<String>) -> Self {
        self.payment_order_id = payment_order_id;
        self
    }

    pub fn with_payment_attempt_id(mut self, payment_attempt_id: Option<String>) -> Self {
        self.payment_attempt_id = payment_attempt_id;
        self
    }

    pub fn with_provider_transaction_id(mut self, provider_transaction_id: Option<String>) -> Self {
        self.provider_transaction_id = provider_transaction_id;
        self
    }

    pub fn with_signature_status(mut self, signature_status: impl Into<String>) -> Self {
        self.signature_status = signature_status.into();
        self
    }

    pub fn with_provider_status(mut self, provider_status: Option<String>) -> Self {
        self.provider_status = provider_status;
        self
    }

    pub fn with_currency_code(mut self, currency_code: Option<String>) -> Self {
        self.currency_code = currency_code;
        self
    }

    pub fn with_amount_minor(mut self, amount_minor: Option<u64>) -> Self {
        self.amount_minor = amount_minor;
        self
    }

    pub fn with_fee_minor(mut self, fee_minor: Option<u64>) -> Self {
        self.fee_minor = fee_minor;
        self
    }

    pub fn with_net_amount_minor(mut self, net_amount_minor: Option<u64>) -> Self {
        self.net_amount_minor = net_amount_minor;
        self
    }

    pub fn with_payload_json(mut self, payload_json: Option<String>) -> Self {
        self.payload_json = payload_json;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentCallbackIntakeDisposition {
    Processed,
    Duplicate,
    Ignored,
    RequiresProviderQuery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentCallbackNormalizedOutcome {
    Authorized,
    Settled,
    Failed,
    Canceled,
    Expired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentCallbackIntakeResult {
    pub disposition: PaymentCallbackIntakeDisposition,
    pub normalized_outcome: Option<PaymentCallbackNormalizedOutcome>,
    pub callback_event: PaymentCallbackEventRecord,
    pub payment_order_opt: Option<PaymentOrderRecord>,
    pub payment_attempt_opt: Option<PaymentAttemptRecord>,
    pub payment_session_opt: Option<PaymentSessionRecord>,
    pub payment_transaction_opt: Option<PaymentTransactionRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SettledSaleApplication {
    payment_transaction: PaymentTransactionRecord,
    captured_amount_minor: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PaymentFailoverReplacement {
    payment_attempt: PaymentAttemptRecord,
    payment_session: PaymentSessionRecord,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentAttemptTimelineEntry {
    pub attempt: PaymentAttemptRecord,
    pub sessions: Vec<PaymentSessionRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommerceOrderCenterEntry {
    pub order: CommerceOrderRecord,
    pub payment_order: Option<PaymentOrderRecord>,
    pub payment_attempts: Vec<PaymentAttemptTimelineEntry>,
    pub active_payment_session: Option<PaymentSessionRecord>,
    pub payment_transactions: Vec<PaymentTransactionRecord>,
    pub refunds: Vec<RefundOrderRecord>,
    pub refundable_amount_minor: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PortalAccountHistorySnapshot {
    pub account: Option<AccountRecord>,
    pub balance: Option<AccountBalanceSnapshot>,
    pub lots: Vec<AccountBenefitLotRecord>,
    pub ledger_entries: Vec<AccountLedgerEntryRecord>,
    pub ledger_allocations: Vec<AccountLedgerAllocationRecord>,
    pub refunds: Vec<RefundOrderRecord>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdminPaymentOrderDossier {
    pub payment_order: PaymentOrderRecord,
    pub commerce_order: CommerceOrderRecord,
    pub payment_attempts: Vec<PaymentAttemptRecord>,
    pub payment_sessions: Vec<PaymentSessionRecord>,
    pub payment_callback_events: Vec<PaymentCallbackEventRecord>,
    pub payment_transactions: Vec<PaymentTransactionRecord>,
    pub refund_orders: Vec<RefundOrderRecord>,
    pub reconciliation_lines: Vec<ReconciliationMatchSummaryRecord>,
    pub account: Option<AccountRecord>,
    pub account_ledger_entries: Vec<AccountLedgerEntryRecord>,
    pub account_ledger_allocations: Vec<AccountLedgerAllocationRecord>,
    pub finance_journal_entries: Vec<FinanceJournalEntryRecord>,
    pub finance_journal_lines: Vec<FinanceJournalLineRecord>,
}

pub fn project_commerce_checkout_bridge(
    order: &CommerceOrderRecord,
) -> Result<CommerceCheckoutBridge> {
    ensure!(
        !order.order_id.trim().is_empty(),
        "order_id must not be empty"
    );

    let reference = format!("PAY-{}", normalize_payment_reference(&order.order_id));
    let guidance = guidance_for_order(order);

    if is_zero_payment_checkout(order) {
        return Ok(CommerceCheckoutBridge {
            session_status: "not_required".to_owned(),
            provider: "no_payment_required".to_owned(),
            mode: "instant_fulfillment".to_owned(),
            reference,
            guidance,
            methods: Vec::new(),
        });
    }

    let (session_status, mode, methods) = match order.status.as_str() {
        "pending_payment" => (
            "open",
            "checkout_bridge",
            build_open_checkout_methods(order),
        ),
        "fulfilled" => ("settled", "closed", Vec::new()),
        "canceled" => ("canceled", "closed", Vec::new()),
        "failed" => ("failed", "closed", Vec::new()),
        _ => ("closed", "closed", Vec::new()),
    };

    Ok(CommerceCheckoutBridge {
        session_status: session_status.to_owned(),
        provider: "payment_orchestrator".to_owned(),
        mode: mode.to_owned(),
        reference,
        guidance,
        methods,
    })
}

pub async fn ensure_commerce_payment_checkout<S>(
    store: &S,
    scope: &PaymentSubjectScope,
    order: &CommerceOrderRecord,
    client_kind: &str,
) -> Result<EnsuredCommercePaymentCheckout>
where
    S: PaymentKernelStore + ?Sized,
{
    let checkout = project_commerce_checkout_bridge(order)?;

    if is_zero_payment_checkout(order) {
        return Ok(EnsuredCommercePaymentCheckout {
            checkout,
            payment_order_opt: None,
            payment_attempt_opt: None,
            payment_session_opt: None,
        });
    }

    let payment_order =
        ensure_checkout_payment_order(store, build_payment_order(scope, order)).await?;
    let payment_attempt = ensure_checkout_payment_attempt(
        store,
        &payment_order,
        build_payment_attempt(scope, &payment_order, order, client_kind),
    )
    .await?;
    let payment_session = ensure_checkout_payment_session(
        store,
        &payment_attempt,
        build_payment_session(scope, &payment_attempt, order),
    )
    .await?;

    Ok(EnsuredCommercePaymentCheckout {
        checkout,
        payment_order_opt: Some(payment_order),
        payment_attempt_opt: Some(payment_attempt),
        payment_session_opt: Some(payment_session),
    })
}

async fn ensure_checkout_payment_order<S>(
    store: &S,
    desired: PaymentOrderRecord,
) -> Result<PaymentOrderRecord>
where
    S: PaymentKernelStore + ?Sized,
{
    if let Some(existing) = store
        .find_payment_order_record(&desired.payment_order_id)
        .await?
    {
        let reconciled = reconcile_checkout_payment_order(&existing, desired);
        if reconciled != existing {
            return store.insert_payment_order_record(&reconciled).await;
        }
        return Ok(existing);
    }

    store.insert_payment_order_record(&desired).await
}

async fn ensure_checkout_payment_attempt<S>(
    store: &S,
    payment_order: &PaymentOrderRecord,
    desired: PaymentAttemptRecord,
) -> Result<PaymentAttemptRecord>
where
    S: PaymentKernelStore + ?Sized,
{
    let attempts = store
        .list_payment_attempt_records_for_order(&payment_order.payment_order_id)
        .await?;
    let existing = select_payment_attempt(&attempts, Some(&desired.payment_attempt_id))
        .or_else(|| select_payment_attempt(&attempts, None));

    if let Some(existing) = existing {
        let reconciled = reconcile_checkout_payment_attempt(&existing, desired);
        if reconciled != existing {
            return store.insert_payment_attempt_record(&reconciled).await;
        }
        return Ok(existing);
    }

    store.insert_payment_attempt_record(&desired).await
}

async fn ensure_checkout_payment_session<S>(
    store: &S,
    payment_attempt: &PaymentAttemptRecord,
    desired: PaymentSessionRecord,
) -> Result<PaymentSessionRecord>
where
    S: PaymentKernelStore + ?Sized,
{
    let sessions = store
        .list_payment_session_records_for_attempt(&payment_attempt.payment_attempt_id)
        .await?;
    let existing = sessions
        .iter()
        .find(|session| session.payment_session_id == desired.payment_session_id)
        .cloned()
        .or_else(|| resolve_session_from_records(&sessions));

    if let Some(existing) = existing {
        let reconciled = reconcile_checkout_payment_session(&existing, desired);
        if reconciled != existing {
            return store.insert_payment_session_record(&reconciled).await;
        }
        return Ok(existing);
    }

    store.insert_payment_session_record(&desired).await
}

pub async fn ensure_portal_payment_subject_scope<S>(
    store: &S,
    portal_user_id: &str,
    observed_at_ms: u64,
) -> Result<PaymentSubjectScope>
where
    S: IdentityKernelStore + ?Sized,
{
    let portal_user_id = portal_user_id.trim();
    ensure!(
        !portal_user_id.is_empty(),
        "portal_user_id must not be empty"
    );

    let portal_user = store
        .find_portal_user_by_id(portal_user_id)
        .await?
        .with_context(|| {
            format!("portal user not found for canonical payment bridge: {portal_user_id}")
        })?;

    if let Some(binding) = store
        .find_identity_binding_record(
            PORTAL_IDENTITY_BINDING_TYPE,
            Some(PORTAL_IDENTITY_BINDING_ISSUER),
            Some(&portal_user.id),
        )
        .await?
    {
        upsert_portal_identity_user(
            store,
            &portal_user,
            binding.tenant_id,
            binding.organization_id,
            binding.user_id,
            observed_at_ms,
        )
        .await?;
        return Ok(PaymentSubjectScope::new(
            binding.tenant_id,
            binding.organization_id,
            binding.user_id,
        ));
    }

    let tenant_id = stable_subject_numeric_id("portal_tenant", &portal_user.workspace_tenant_id);
    let organization_id = 0;
    let user_id = stable_subject_numeric_id("portal_user", &portal_user.id);

    upsert_portal_identity_user(
        store,
        &portal_user,
        tenant_id,
        organization_id,
        user_id,
        observed_at_ms,
    )
    .await?;

    let binding = IdentityBindingRecord::new(
        stable_subject_numeric_id("portal_binding", &portal_user.id),
        tenant_id,
        organization_id,
        user_id,
        PORTAL_IDENTITY_BINDING_TYPE,
    )
    .with_issuer(Some(PORTAL_IDENTITY_BINDING_ISSUER.to_owned()))
    .with_subject(Some(portal_user.id.clone()))
    .with_platform(Some("portal".to_owned()))
    .with_owner(Some(portal_user.workspace_project_id.clone()))
    .with_external_ref(Some(portal_user.workspace_tenant_id.clone()))
    .with_created_at_ms(portal_user.created_at_ms)
    .with_updated_at_ms(observed_at_ms);
    store.insert_identity_binding_record(&binding).await?;

    Ok(PaymentSubjectScope::new(
        tenant_id,
        organization_id,
        user_id,
    ))
}

pub async fn ensure_portal_commerce_payment_checkout<S>(
    store: &S,
    portal_user_id: &str,
    order: &CommerceOrderRecord,
    client_kind: &str,
    observed_at_ms: u64,
) -> Result<EnsuredCommercePaymentCheckout>
where
    S: PaymentKernelStore + IdentityKernelStore + ?Sized,
{
    ensure!(
        order.user_id.trim() == portal_user_id.trim(),
        "portal checkout subject mismatch for commerce order {}",
        order.order_id
    );

    let scope = ensure_portal_payment_subject_scope(store, portal_user_id, observed_at_ms).await?;
    ensure_commerce_payment_checkout(store, &scope, order, client_kind).await
}

pub async fn list_project_commerce_order_center<S>(
    store: &S,
    project_id: &str,
) -> Result<Vec<CommerceOrderCenterEntry>>
where
    S: CommercialKernelStore + ?Sized,
{
    ensure!(
        !project_id.trim().is_empty(),
        "project_id must not be empty"
    );

    let mut orders = store.list_commerce_orders_for_project(project_id).await?;
    orders.sort_by(|left, right| {
        right
            .created_at_ms
            .cmp(&left.created_at_ms)
            .then_with(|| right.order_id.cmp(&left.order_id))
    });

    let payment_orders = store
        .list_payment_order_records()
        .await?
        .into_iter()
        .filter(|payment_order| payment_order.project_id == project_id)
        .collect::<Vec<_>>();

    let mut entries = Vec::with_capacity(orders.len());
    for order in orders {
        let payment_order = payment_orders
            .iter()
            .find(|payment_order| payment_order.commerce_order_id == order.order_id)
            .cloned();
        let (
            refunds,
            payment_transactions,
            refundable_amount_minor,
            payment_attempts,
            active_payment_session,
        ) = if let Some(payment_order) = payment_order.as_ref() {
            let mut refunds = store
                .list_refund_order_records_for_payment_order(&payment_order.payment_order_id)
                .await?;
            refunds.sort_by(|left, right| {
                right
                    .created_at_ms
                    .cmp(&left.created_at_ms)
                    .then_with(|| right.refund_order_id.cmp(&left.refund_order_id))
            });

            let mut payment_transactions = store
                .list_payment_transaction_records_for_order(&payment_order.payment_order_id)
                .await?;
            payment_transactions.sort_by(|left, right| {
                right
                    .occurred_at_ms
                    .cmp(&left.occurred_at_ms)
                    .then_with(|| {
                        right
                            .payment_transaction_id
                            .cmp(&left.payment_transaction_id)
                    })
            });

            let payment_attempts =
                load_payment_attempt_timeline_entries(store, &payment_order.payment_order_id)
                    .await?;
            let active_payment_session =
                select_active_payment_session_from_timeline(&payment_attempts);
            let refundable_amount_minor =
                payment_order_remaining_refundable_amount_minor(payment_order, &refunds);
            (
                refunds,
                payment_transactions,
                refundable_amount_minor,
                payment_attempts,
                active_payment_session,
            )
        } else {
            (Vec::new(), Vec::new(), 0, Vec::new(), None)
        };

        entries.push(CommerceOrderCenterEntry {
            order,
            payment_order,
            payment_attempts,
            active_payment_session,
            payment_transactions,
            refunds,
            refundable_amount_minor,
        });
    }

    Ok(entries)
}

async fn load_payment_attempt_timeline_entries<S>(
    store: &S,
    payment_order_id: &str,
) -> Result<Vec<PaymentAttemptTimelineEntry>>
where
    S: PaymentKernelStore + ?Sized,
{
    let mut payment_attempts = store
        .list_payment_attempt_records_for_order(payment_order_id)
        .await?;
    payment_attempts.sort_by(|left, right| {
        right
            .attempt_no
            .cmp(&left.attempt_no)
            .then_with(|| right.payment_attempt_id.cmp(&left.payment_attempt_id))
    });

    let mut timeline = Vec::with_capacity(payment_attempts.len());
    for payment_attempt in payment_attempts {
        let mut sessions = store
            .list_payment_session_records_for_attempt(&payment_attempt.payment_attempt_id)
            .await?;
        sessions.sort_by(|left, right| {
            right
                .created_at_ms
                .cmp(&left.created_at_ms)
                .then_with(|| right.payment_session_id.cmp(&left.payment_session_id))
        });
        timeline.push(PaymentAttemptTimelineEntry {
            attempt: payment_attempt,
            sessions,
        });
    }

    Ok(timeline)
}

fn select_active_payment_session_from_timeline(
    payment_attempts: &[PaymentAttemptTimelineEntry],
) -> Option<PaymentSessionRecord> {
    for payment_attempt in payment_attempts {
        if !payment_attempt_status_is_terminal(payment_attempt.attempt.attempt_status) {
            if let Some(session) = payment_attempt
                .sessions
                .iter()
                .find(|session| !session.session_status.is_terminal())
                .cloned()
                .or_else(|| payment_attempt.sessions.first().cloned())
            {
                return Some(session);
            }
        }
    }

    payment_attempts
        .first()
        .and_then(|payment_attempt| payment_attempt.sessions.first().cloned())
}

fn payment_attempt_status_is_terminal(status: PaymentAttemptStatus) -> bool {
    matches!(
        status,
        PaymentAttemptStatus::Succeeded
            | PaymentAttemptStatus::Failed
            | PaymentAttemptStatus::Expired
            | PaymentAttemptStatus::Canceled
    )
}

pub async fn request_portal_commerce_order_refund(
    store: &dyn CommercialKernelStore,
    portal_user_id: &str,
    project_id: &str,
    order_id: &str,
    refund_reason_code: &str,
    requested_amount_minor: u64,
    requested_at_ms: u64,
) -> Result<RefundOrderRecord> {
    let order =
        load_portal_scoped_commerce_order(store, portal_user_id, project_id, order_id).await?;
    let payment_order =
        find_project_payment_order_for_commerce_order(store, project_id, &order.order_id).await?;
    let scope = PaymentSubjectScope::new(
        payment_order.tenant_id,
        payment_order.organization_id,
        payment_order.user_id,
    );

    let refund_order = request_payment_order_refund(
        store,
        &scope,
        &payment_order.payment_order_id,
        refund_reason_code,
        requested_amount_minor,
        "portal_user",
        portal_user_id,
        requested_at_ms,
    )
    .await?;

    if matches!(refund_order.refund_status, RefundOrderStatus::Requested) {
        return transition_refund_order_to_awaiting_approval(
            store,
            &refund_order.refund_order_id,
            requested_at_ms,
        )
        .await;
    }

    Ok(refund_order)
}

pub async fn list_portal_commerce_order_payment_events(
    store: &dyn CommercialKernelStore,
    portal_user_id: &str,
    project_id: &str,
    order_id: &str,
) -> Result<Vec<PaymentCallbackEventRecord>> {
    let order =
        load_portal_scoped_commerce_order(store, portal_user_id, project_id, order_id).await?;
    let payment_order =
        find_project_payment_order_for_commerce_order(store, project_id, &order.order_id).await?;

    let mut callback_events = store
        .list_payment_callback_event_records()
        .await?
        .into_iter()
        .filter(|callback_event| {
            callback_event.payment_order_id.as_deref()
                == Some(payment_order.payment_order_id.as_str())
        })
        .collect::<Vec<_>>();
    callback_events.sort_by(|left, right| {
        right
            .received_at_ms
            .cmp(&left.received_at_ms)
            .then_with(|| right.callback_event_id.cmp(&left.callback_event_id))
    });
    Ok(callback_events)
}

pub async fn load_portal_account_history<S>(
    store: &S,
    portal_user_id: &str,
    project_id: &str,
    now_ms: u64,
) -> Result<PortalAccountHistorySnapshot>
where
    S: CommercialKernelStore + IdentityKernelStore + ?Sized,
{
    ensure!(
        !portal_user_id.trim().is_empty(),
        "portal_user_id must not be empty"
    );
    ensure!(
        !project_id.trim().is_empty(),
        "project_id must not be empty"
    );

    let scope = ensure_portal_payment_subject_scope(store, portal_user_id, now_ms).await?;
    let mut payment_orders = store
        .list_payment_order_records()
        .await?
        .into_iter()
        .filter(|payment_order| {
            payment_order.project_id == project_id
                && payment_order.tenant_id == scope.tenant_id
                && payment_order.organization_id == scope.organization_id
                && payment_order.user_id == scope.user_id
        })
        .collect::<Vec<_>>();
    payment_orders.sort_by(|left, right| {
        right
            .created_at_ms
            .cmp(&left.created_at_ms)
            .then_with(|| right.payment_order_id.cmp(&left.payment_order_id))
    });

    let mut refunds = Vec::new();
    for payment_order in &payment_orders {
        let mut payment_refunds = store
            .list_refund_order_records_for_payment_order(&payment_order.payment_order_id)
            .await?;
        refunds.append(&mut payment_refunds);
    }
    refunds.sort_by(|left, right| {
        right
            .created_at_ms
            .cmp(&left.created_at_ms)
            .then_with(|| right.refund_order_id.cmp(&left.refund_order_id))
    });

    let Some(account) = store
        .find_account_record_by_owner(
            scope.tenant_id,
            scope.organization_id,
            scope.user_id,
            AccountType::Primary,
        )
        .await?
    else {
        return Ok(PortalAccountHistorySnapshot {
            account: None,
            balance: None,
            lots: Vec::new(),
            ledger_entries: Vec::new(),
            ledger_allocations: Vec::new(),
            refunds,
        });
    };

    let balance = summarize_account_balance(store, account.account_id, now_ms).await?;

    let mut lots = store
        .list_account_benefit_lots()
        .await?
        .into_iter()
        .filter(|lot| lot.account_id == account.account_id)
        .collect::<Vec<_>>();
    lots.sort_by(|left, right| {
        right
            .created_at_ms
            .cmp(&left.created_at_ms)
            .then_with(|| right.lot_id.cmp(&left.lot_id))
    });

    let mut ledger_entries = store
        .list_account_ledger_entry_records()
        .await?
        .into_iter()
        .filter(|entry| entry.account_id == account.account_id)
        .collect::<Vec<_>>();
    ledger_entries.sort_by(|left, right| {
        right
            .created_at_ms
            .cmp(&left.created_at_ms)
            .then_with(|| right.ledger_entry_id.cmp(&left.ledger_entry_id))
    });
    let ledger_entry_ids = ledger_entries
        .iter()
        .map(|entry| entry.ledger_entry_id)
        .collect::<std::collections::BTreeSet<_>>();
    let mut ledger_allocations = store
        .list_account_ledger_allocations()
        .await?
        .into_iter()
        .filter(|allocation| ledger_entry_ids.contains(&allocation.ledger_entry_id))
        .collect::<Vec<_>>();
    ledger_allocations.sort_by(|left, right| {
        right
            .created_at_ms
            .cmp(&left.created_at_ms)
            .then_with(|| right.ledger_allocation_id.cmp(&left.ledger_allocation_id))
    });

    Ok(PortalAccountHistorySnapshot {
        account: Some(account),
        balance: Some(balance),
        lots,
        ledger_entries,
        ledger_allocations,
        refunds,
    })
}

pub async fn load_admin_payment_order_dossier<S>(
    store: &S,
    payment_order_id: &str,
) -> Result<Option<AdminPaymentOrderDossier>>
where
    S: CommercialKernelStore + ?Sized,
{
    ensure!(
        !payment_order_id.trim().is_empty(),
        "payment_order_id must not be empty"
    );

    let Some(payment_order) = store.find_payment_order_record(payment_order_id).await? else {
        return Ok(None);
    };

    let commerce_order = find_commerce_order_for_payment_order(store, &payment_order).await?;

    let mut payment_attempts = store
        .list_payment_attempt_records_for_order(&payment_order.payment_order_id)
        .await?;
    payment_attempts.sort_by(compare_admin_payment_attempts);
    let payment_attempt_ids = payment_attempts
        .iter()
        .map(|attempt| attempt.payment_attempt_id.clone())
        .collect::<HashSet<_>>();

    let mut payment_sessions = Vec::new();
    for payment_attempt in &payment_attempts {
        let mut attempt_sessions = store
            .list_payment_session_records_for_attempt(&payment_attempt.payment_attempt_id)
            .await?;
        payment_sessions.append(&mut attempt_sessions);
    }
    payment_sessions.sort_by(compare_admin_payment_sessions);

    let mut payment_callback_events = store
        .list_payment_callback_event_records()
        .await?
        .into_iter()
        .filter(|event| {
            event.payment_order_id.as_deref() == Some(payment_order_id)
                || event
                    .payment_attempt_id
                    .as_ref()
                    .map(|payment_attempt_id| payment_attempt_ids.contains(payment_attempt_id))
                    .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    payment_callback_events.sort_by(compare_admin_payment_callback_events);

    let mut payment_transactions = store
        .list_payment_transaction_records_for_order(&payment_order.payment_order_id)
        .await?;
    payment_transactions.sort_by(compare_admin_payment_transactions);

    let mut refund_orders = store
        .list_refund_order_records_for_payment_order(&payment_order.payment_order_id)
        .await?;
    refund_orders.sort_by(compare_admin_refund_orders);
    let refund_order_ids = refund_orders
        .iter()
        .map(|refund_order| refund_order.refund_order_id.clone())
        .collect::<HashSet<_>>();

    let mut reconciliation_lines = store
        .list_all_reconciliation_match_summary_records()
        .await?
        .into_iter()
        .filter(|line| {
            line.payment_order_id.as_deref() == Some(payment_order_id)
                || line
                    .refund_order_id
                    .as_ref()
                    .map(|refund_order_id| refund_order_ids.contains(refund_order_id))
                    .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    reconciliation_lines.sort_by(compare_admin_reconciliation_lines);

    let mut account = store
        .find_account_record_by_owner(
            payment_order.tenant_id,
            payment_order.organization_id,
            payment_order.user_id,
            AccountType::Primary,
        )
        .await?;
    let mut account_ledger_entries = Vec::new();
    let mut account_ledger_allocations = Vec::new();
    if let Some(account_record) = account.as_ref() {
        let mut relevant_ledger_entry_ids =
            HashSet::from([commerce_order_account_grant_ledger_entry_id(&payment_order)]);
        relevant_ledger_entry_ids.extend(refund_orders.iter().map(|refund_order| {
            commerce_order_account_refund_ledger_entry_id(
                &payment_order,
                &refund_order.refund_order_id,
            )
        }));

        account_ledger_entries = store
            .list_account_ledger_entry_records()
            .await?
            .into_iter()
            .filter(|entry| {
                entry.account_id == account_record.account_id
                    && relevant_ledger_entry_ids.contains(&entry.ledger_entry_id)
            })
            .collect::<Vec<_>>();
        account_ledger_entries.sort_by(compare_admin_account_ledger_entries);

        let returned_ledger_entry_ids = account_ledger_entries
            .iter()
            .map(|entry| entry.ledger_entry_id)
            .collect::<HashSet<_>>();
        account_ledger_allocations = store
            .list_account_ledger_allocations()
            .await?
            .into_iter()
            .filter(|allocation| returned_ledger_entry_ids.contains(&allocation.ledger_entry_id))
            .collect::<Vec<_>>();
        account_ledger_allocations.sort_by(compare_admin_account_ledger_allocations);
    } else {
        account = None;
    }

    let mut finance_journal_entries = store
        .list_finance_journal_entry_records()
        .await?
        .into_iter()
        .filter(|entry| {
            finance_entry_belongs_to_payment_order(
                entry,
                &payment_order,
                &commerce_order,
                &refund_order_ids,
            )
        })
        .collect::<Vec<_>>();
    finance_journal_entries.sort_by(compare_admin_finance_journal_entries);

    let mut finance_journal_lines = Vec::new();
    for finance_entry in &finance_journal_entries {
        let mut lines = store
            .list_finance_journal_line_records(&finance_entry.finance_journal_entry_id)
            .await?;
        lines.sort_by(compare_admin_finance_journal_lines);
        finance_journal_lines.extend(lines);
    }

    Ok(Some(AdminPaymentOrderDossier {
        payment_order,
        commerce_order,
        payment_attempts,
        payment_sessions,
        payment_callback_events,
        payment_transactions,
        refund_orders,
        reconciliation_lines,
        account,
        account_ledger_entries,
        account_ledger_allocations,
        finance_journal_entries,
        finance_journal_lines,
    }))
}

pub async fn transition_refund_order_to_awaiting_approval<S>(
    store: &S,
    refund_order_id: &str,
    updated_at_ms: u64,
) -> Result<RefundOrderRecord>
where
    S: PaymentKernelStore + ?Sized,
{
    let mut refund_order = store
        .find_refund_order_record(refund_order_id)
        .await?
        .with_context(|| format!("refund order not found: {refund_order_id}"))?;
    if matches!(
        refund_order.refund_status,
        RefundOrderStatus::AwaitingApproval
    ) {
        return Ok(refund_order);
    }
    ensure!(
        matches!(refund_order.refund_status, RefundOrderStatus::Requested),
        "refund order {} status {} cannot transition to awaiting approval",
        refund_order.refund_order_id,
        refund_order.refund_status.as_str()
    );
    refund_order.refund_status = RefundOrderStatus::AwaitingApproval;
    refund_order.approved_amount_minor = None;
    refund_order.updated_at_ms = updated_at_ms.max(refund_order.created_at_ms);
    store.insert_refund_order_record(&refund_order).await
}

pub async fn approve_refund_order_request<S>(
    store: &S,
    refund_order_id: &str,
    approved_amount_minor: Option<u64>,
    approved_at_ms: u64,
) -> Result<RefundOrderRecord>
where
    S: PaymentKernelStore + ?Sized,
{
    let mut refund_order = store
        .find_refund_order_record(refund_order_id)
        .await?
        .with_context(|| format!("refund order not found: {refund_order_id}"))?;
    ensure!(
        matches!(
            refund_order.refund_status,
            RefundOrderStatus::Requested
                | RefundOrderStatus::AwaitingApproval
                | RefundOrderStatus::Approved
        ),
        "refund order {} status {} cannot be approved",
        refund_order.refund_order_id,
        refund_order.refund_status.as_str()
    );

    let approved_amount_minor =
        approved_amount_minor.unwrap_or(refund_order.requested_amount_minor);
    ensure!(
        approved_amount_minor > 0,
        "approved refund amount must be positive"
    );
    ensure!(
        approved_amount_minor <= refund_order.requested_amount_minor,
        "approved refund amount {approved_amount_minor} exceeds requested amount {}",
        refund_order.requested_amount_minor
    );

    let mut payment_order = store
        .find_payment_order_record(&refund_order.payment_order_id)
        .await?
        .with_context(|| {
            format!(
                "payment order not found for refund {}",
                refund_order.refund_order_id
            )
        })?;
    let existing_refunds = store
        .list_refund_order_records_for_payment_order(&payment_order.payment_order_id)
        .await?;
    let reserved_other_amount_minor = existing_refunds
        .iter()
        .filter(|existing| existing.refund_order_id != refund_order.refund_order_id)
        .filter(|existing| {
            !matches!(
                existing.refund_status,
                RefundOrderStatus::Failed | RefundOrderStatus::Canceled
            )
        })
        .map(|existing| {
            existing
                .approved_amount_minor
                .unwrap_or(existing.requested_amount_minor)
                .max(existing.refunded_amount_minor)
        })
        .fold(0_u64, u64::saturating_add);
    let remaining_approvable_amount_minor = payment_order_refund_ceiling_minor(&payment_order)
        .saturating_sub(reserved_other_amount_minor);
    ensure!(
        approved_amount_minor <= remaining_approvable_amount_minor,
        "approved refund amount {approved_amount_minor} exceeds remaining refundable amount {remaining_approvable_amount_minor}"
    );

    if payment_order.refund_status != PaymentRefundStatus::Pending {
        payment_order.refund_status = PaymentRefundStatus::Pending;
        payment_order.updated_at_ms = approved_at_ms;
        payment_order.version = payment_order.version.saturating_add(1);
        store.insert_payment_order_record(&payment_order).await?;
    }

    refund_order.refund_status = RefundOrderStatus::Approved;
    refund_order.approved_amount_minor = Some(approved_amount_minor);
    refund_order.updated_at_ms = approved_at_ms.max(refund_order.created_at_ms);
    store.insert_refund_order_record(&refund_order).await
}

pub async fn start_refund_order_execution<S>(
    store: &S,
    refund_order_id: &str,
    started_at_ms: u64,
) -> Result<RefundOrderRecord>
where
    S: PaymentKernelStore + ?Sized,
{
    let mut refund_order = store
        .find_refund_order_record(refund_order_id)
        .await?
        .with_context(|| format!("refund order not found: {refund_order_id}"))?;
    let mut payment_order = store
        .find_payment_order_record(&refund_order.payment_order_id)
        .await?
        .with_context(|| {
            format!(
                "payment order not found for refund {}",
                refund_order.refund_order_id
            )
        })?;

    if payment_order.refund_status != PaymentRefundStatus::Pending {
        payment_order.refund_status = PaymentRefundStatus::Pending;
        payment_order.updated_at_ms = started_at_ms;
        payment_order.version = payment_order.version.saturating_add(1);
        store.insert_payment_order_record(&payment_order).await?;
    }

    if matches!(refund_order.refund_status, RefundOrderStatus::Processing) {
        return Ok(refund_order);
    }
    ensure!(
        matches!(refund_order.refund_status, RefundOrderStatus::Approved),
        "refund order {} status {} cannot start execution",
        refund_order.refund_order_id,
        refund_order.refund_status.as_str()
    );

    refund_order.refund_status = RefundOrderStatus::Processing;
    refund_order.updated_at_ms = started_at_ms.max(refund_order.created_at_ms);
    store.insert_refund_order_record(&refund_order).await
}

pub async fn cancel_refund_order_request<S>(
    store: &S,
    refund_order_id: &str,
    canceled_at_ms: u64,
) -> Result<RefundOrderRecord>
where
    S: PaymentKernelStore + ?Sized,
{
    let mut refund_order = store
        .find_refund_order_record(refund_order_id)
        .await?
        .with_context(|| format!("refund order not found: {refund_order_id}"))?;
    if matches!(refund_order.refund_status, RefundOrderStatus::Canceled) {
        return Ok(refund_order);
    }
    ensure!(
        matches!(
            refund_order.refund_status,
            RefundOrderStatus::Requested
                | RefundOrderStatus::AwaitingApproval
                | RefundOrderStatus::Approved
        ),
        "refund order {} status {} cannot be canceled",
        refund_order.refund_order_id,
        refund_order.refund_status.as_str()
    );

    let mut payment_order = store
        .find_payment_order_record(&refund_order.payment_order_id)
        .await?
        .with_context(|| {
            format!(
                "payment order not found for refund {}",
                refund_order.refund_order_id
            )
        })?;

    refund_order.refund_status = RefundOrderStatus::Canceled;
    refund_order.updated_at_ms = canceled_at_ms.max(refund_order.created_at_ms);
    let refund_order = store.insert_refund_order_record(&refund_order).await?;

    let mut refunds = store
        .list_refund_order_records_for_payment_order(&payment_order.payment_order_id)
        .await?;
    if let Some(existing) = refunds
        .iter_mut()
        .find(|existing| existing.refund_order_id == refund_order.refund_order_id)
    {
        *existing = refund_order.clone();
    } else {
        refunds.push(refund_order.clone());
    }
    let next_refund_status =
        derive_payment_refund_status(payment_order_refund_ceiling_minor(&payment_order), &refunds);
    if payment_order.refund_status != next_refund_status {
        payment_order.refund_status = next_refund_status;
        payment_order.updated_at_ms = canceled_at_ms;
        payment_order.version = payment_order.version.saturating_add(1);
        store.insert_payment_order_record(&payment_order).await?;
    }

    Ok(refund_order)
}

fn compare_admin_payment_attempts(
    left: &PaymentAttemptRecord,
    right: &PaymentAttemptRecord,
) -> std::cmp::Ordering {
    right
        .attempt_no
        .cmp(&left.attempt_no)
        .then_with(|| right.created_at_ms.cmp(&left.created_at_ms))
        .then_with(|| left.payment_attempt_id.cmp(&right.payment_attempt_id))
}

fn compare_admin_payment_sessions(
    left: &PaymentSessionRecord,
    right: &PaymentSessionRecord,
) -> std::cmp::Ordering {
    right
        .created_at_ms
        .cmp(&left.created_at_ms)
        .then_with(|| right.payment_session_id.cmp(&left.payment_session_id))
}

fn compare_admin_payment_callback_events(
    left: &PaymentCallbackEventRecord,
    right: &PaymentCallbackEventRecord,
) -> std::cmp::Ordering {
    right
        .received_at_ms
        .cmp(&left.received_at_ms)
        .then_with(|| right.callback_event_id.cmp(&left.callback_event_id))
}

fn compare_admin_payment_transactions(
    left: &PaymentTransactionRecord,
    right: &PaymentTransactionRecord,
) -> std::cmp::Ordering {
    right
        .occurred_at_ms
        .cmp(&left.occurred_at_ms)
        .then_with(|| {
            right
                .payment_transaction_id
                .cmp(&left.payment_transaction_id)
        })
}

fn compare_admin_refund_orders(
    left: &RefundOrderRecord,
    right: &RefundOrderRecord,
) -> std::cmp::Ordering {
    right
        .created_at_ms
        .cmp(&left.created_at_ms)
        .then_with(|| right.refund_order_id.cmp(&left.refund_order_id))
}

fn compare_admin_reconciliation_lines(
    left: &ReconciliationMatchSummaryRecord,
    right: &ReconciliationMatchSummaryRecord,
) -> std::cmp::Ordering {
    matches!(left.match_status, ReconciliationMatchStatus::Resolved)
        .cmp(&matches!(
            right.match_status,
            ReconciliationMatchStatus::Resolved
        ))
        .then_with(|| right.updated_at_ms.cmp(&left.updated_at_ms))
        .then_with(|| right.created_at_ms.cmp(&left.created_at_ms))
        .then_with(|| {
            right
                .reconciliation_line_id
                .cmp(&left.reconciliation_line_id)
        })
}

fn compare_admin_account_ledger_entries(
    left: &AccountLedgerEntryRecord,
    right: &AccountLedgerEntryRecord,
) -> std::cmp::Ordering {
    right
        .created_at_ms
        .cmp(&left.created_at_ms)
        .then_with(|| right.ledger_entry_id.cmp(&left.ledger_entry_id))
}

fn compare_admin_account_ledger_allocations(
    left: &AccountLedgerAllocationRecord,
    right: &AccountLedgerAllocationRecord,
) -> std::cmp::Ordering {
    right
        .created_at_ms
        .cmp(&left.created_at_ms)
        .then_with(|| right.ledger_allocation_id.cmp(&left.ledger_allocation_id))
}

fn compare_admin_finance_journal_entries(
    left: &FinanceJournalEntryRecord,
    right: &FinanceJournalEntryRecord,
) -> std::cmp::Ordering {
    right
        .occurred_at_ms
        .cmp(&left.occurred_at_ms)
        .then_with(|| {
            right
                .finance_journal_entry_id
                .cmp(&left.finance_journal_entry_id)
        })
}

fn compare_admin_finance_journal_lines(
    left: &FinanceJournalLineRecord,
    right: &FinanceJournalLineRecord,
) -> std::cmp::Ordering {
    left.line_no.cmp(&right.line_no).then_with(|| {
        left.finance_journal_line_id
            .cmp(&right.finance_journal_line_id)
    })
}

fn finance_entry_belongs_to_payment_order(
    finance_entry: &FinanceJournalEntryRecord,
    payment_order: &PaymentOrderRecord,
    commerce_order: &CommerceOrderRecord,
    refund_order_ids: &HashSet<String>,
) -> bool {
    match finance_entry.source_kind.as_str() {
        "payment_order" => finance_entry.source_id == payment_order.payment_order_id,
        "commerce_order" => finance_entry.source_id == commerce_order.order_id,
        "refund_order" => refund_order_ids.contains(&finance_entry.source_id),
        _ => false,
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn request_payment_order_refund(
    store: &dyn CommercialKernelStore,
    scope: &PaymentSubjectScope,
    payment_order_id: &str,
    refund_reason_code: &str,
    requested_amount_minor: u64,
    requested_by_type: &str,
    requested_by_id: &str,
    requested_at_ms: u64,
) -> Result<RefundOrderRecord> {
    ensure!(
        requested_amount_minor > 0,
        "requested refund amount must be positive"
    );
    ensure!(
        !payment_order_id.trim().is_empty(),
        "payment_order_id must not be empty"
    );
    ensure!(
        !requested_by_type.trim().is_empty(),
        "requested_by_type must not be empty"
    );
    ensure!(
        !requested_by_id.trim().is_empty(),
        "requested_by_id must not be empty"
    );
    let normalized_refund_reason_code = refund_reason_code.trim();
    let normalized_requested_by_type = requested_by_type.trim();
    let normalized_requested_by_id = requested_by_id.trim();

    let mut payment_order = store
        .find_payment_order_record(payment_order_id)
        .await?
        .with_context(|| {
            format!("payment order not found for refund request: {payment_order_id}")
        })?;
    ensure!(
        payment_order.tenant_id == scope.tenant_id
            && payment_order.organization_id == scope.organization_id,
        "payment order {payment_order_id} is outside the current scope"
    );
    if requested_by_type.trim() == "portal_user" {
        ensure!(
            payment_order.user_id == scope.user_id,
            "portal refund requests must be initiated by the payment order owner"
        );
    }
    ensure!(
        payment_order.payment_status.supports_refund(),
        "payment order {} status {} does not support refunds",
        payment_order.payment_order_id,
        payment_order.payment_status.as_str()
    );
    ensure_supported_refund_target_kind(&payment_order.order_kind)?;

    let existing_refunds = store
        .list_refund_order_records_for_payment_order(&payment_order.payment_order_id)
        .await?;
    if let Some(existing) = find_reusable_pending_refund_request(
        &existing_refunds,
        normalized_refund_reason_code,
        normalized_requested_by_type,
        normalized_requested_by_id,
        requested_amount_minor,
    ) {
        if payment_order.refund_status != PaymentRefundStatus::Pending {
            payment_order.refund_status = PaymentRefundStatus::Pending;
            payment_order.updated_at_ms = requested_at_ms;
            payment_order.version = payment_order.version.saturating_add(1);
            store.insert_payment_order_record(&payment_order).await?;
        }
        return Ok(existing);
    }
    let reserved_amount_minor = reserved_refund_amount_minor(&existing_refunds);
    let remaining_amount_minor =
        payment_order_refund_ceiling_minor(&payment_order).saturating_sub(reserved_amount_minor);
    ensure!(
        requested_amount_minor <= remaining_amount_minor,
        "requested refund amount {requested_amount_minor} exceeds remaining refundable amount {remaining_amount_minor}"
    );

    let refund_order = RefundOrderRecord::new(
        refund_order_id(&payment_order.payment_order_id, requested_at_ms),
        payment_order.tenant_id,
        payment_order.organization_id,
        payment_order.payment_order_id.clone(),
        payment_order.commerce_order_id.clone(),
        normalized_refund_reason_code,
        normalized_requested_by_type,
        normalized_requested_by_id,
        payment_order.currency_code.clone(),
        requested_amount_minor,
    )
    .with_approved_amount_minor(Some(requested_amount_minor))
    .with_created_at_ms(requested_at_ms)
    .with_updated_at_ms(requested_at_ms);
    let refund_order = store.insert_refund_order_record(&refund_order).await?;

    if payment_order.refund_status != PaymentRefundStatus::Pending {
        payment_order.refund_status = PaymentRefundStatus::Pending;
        payment_order.updated_at_ms = requested_at_ms;
        payment_order.version = payment_order.version.saturating_add(1);
        store.insert_payment_order_record(&payment_order).await?;
    }

    Ok(refund_order)
}

pub async fn finalize_refund_order_success(
    store: &dyn CommercialKernelStore,
    refund_order_id: &str,
    provider_refund_id: &str,
    refunded_amount_minor: u64,
    finalized_at_ms: u64,
) -> Result<RefundOrderRecord> {
    ensure!(
        !refund_order_id.trim().is_empty(),
        "refund_order_id must not be empty"
    );
    ensure!(
        !provider_refund_id.trim().is_empty(),
        "provider_refund_id must not be empty"
    );
    ensure!(
        refunded_amount_minor > 0,
        "refunded amount must be positive"
    );

    let mut refund_order = store
        .find_refund_order_record(refund_order_id)
        .await?
        .with_context(|| format!("refund order not found: {refund_order_id}"))?;
    if refund_order.refund_status == RefundOrderStatus::Succeeded {
        return Ok(refund_order);
    }
    ensure!(
        !matches!(
            refund_order.refund_status,
            RefundOrderStatus::AwaitingApproval
                | RefundOrderStatus::Approved
                | RefundOrderStatus::Failed
                | RefundOrderStatus::Canceled
                | RefundOrderStatus::PartiallySucceeded
        ),
        "refund order {} status {} cannot be finalized as success",
        refund_order.refund_order_id,
        refund_order.refund_status.as_str()
    );

    let approved_amount_minor = refund_order
        .approved_amount_minor
        .unwrap_or(refund_order.requested_amount_minor);
    ensure!(
        refunded_amount_minor <= approved_amount_minor,
        "refunded amount {refunded_amount_minor} exceeds approved amount {approved_amount_minor}"
    );

    let mut payment_order = store
        .find_payment_order_record(&refund_order.payment_order_id)
        .await?
        .with_context(|| {
            format!(
                "payment order not found for refund {}",
                refund_order.refund_order_id
            )
        })?;
    ensure!(
        payment_order.payment_status.supports_refund(),
        "payment order {} status {} does not support refunds",
        payment_order.payment_order_id,
        payment_order.payment_status.as_str()
    );
    let commerce_order = find_commerce_order_for_payment_order(store, &payment_order).await?;
    ensure_supported_refund_target_kind(&commerce_order.target_kind)?;

    if refund_order.refund_status != RefundOrderStatus::Processing {
        refund_order.refund_status = RefundOrderStatus::Processing;
        refund_order.updated_at_ms = finalized_at_ms;
        refund_order = store.insert_refund_order_record(&refund_order).await?;
    }

    let account_refund = reverse_commerce_order_account_grant(
        store,
        &payment_order,
        &commerce_order,
        &refund_order.refund_order_id,
        refunded_amount_minor,
        finalized_at_ms,
    )
    .await?;
    let reversed_units = account_refund.reversed_quantity as u64;
    if reversed_units > 0 {
        store
            .apply_refund_order_quota_reversal(
                &refund_order.refund_order_id,
                &payment_order.project_id,
                &commerce_order.target_kind,
                reversed_units,
            )
            .await?;
    }

    ensure_refund_payment_transaction_record(
        store,
        &refund_order,
        &payment_order,
        provider_refund_id,
        refunded_amount_minor,
        finalized_at_ms,
    )
    .await?;

    let finance_entry = build_refund_finance_journal_entry_record(
        &refund_order,
        &payment_order.currency_code,
        finalized_at_ms,
    );
    store
        .insert_finance_journal_entry_record(&finance_entry)
        .await?;
    for line in build_refund_finance_journal_line_records(
        &finance_entry,
        &refund_order,
        refunded_amount_minor,
    ) {
        store.insert_finance_journal_line_record(&line).await?;
    }

    refund_order.refunded_amount_minor = refunded_amount_minor;
    refund_order.refund_status = if refunded_amount_minor < approved_amount_minor {
        RefundOrderStatus::PartiallySucceeded
    } else {
        RefundOrderStatus::Succeeded
    };
    refund_order.updated_at_ms = finalized_at_ms;
    refund_order = store.insert_refund_order_record(&refund_order).await?;

    let mut refunds = store
        .list_refund_order_records_for_payment_order(&payment_order.payment_order_id)
        .await?;
    if let Some(existing) = refunds
        .iter_mut()
        .find(|existing| existing.refund_order_id == refund_order.refund_order_id)
    {
        *existing = refund_order.clone();
    } else {
        refunds.push(refund_order.clone());
    }
    payment_order.refund_status =
        derive_payment_refund_status(payment_order_refund_ceiling_minor(&payment_order), &refunds);
    payment_order.updated_at_ms = finalized_at_ms;
    payment_order.version = payment_order.version.saturating_add(1);
    store.insert_payment_order_record(&payment_order).await?;

    Ok(refund_order)
}

pub async fn ingest_payment_callback(
    store: &dyn CommercialKernelStore,
    request: &PaymentCallbackIntakeRequest,
) -> Result<PaymentCallbackIntakeResult> {
    validate_payment_callback_request(request)?;

    let raw_event = build_callback_event_record(request);

    if !signature_status_allows_processing(&request.signature_status) {
        let ignored_event = raw_event
            .with_processing_status(PaymentCallbackProcessingStatus::Ignored)
            .with_processed_at_ms(Some(request.received_at_ms));
        let callback_event =
            persist_callback_event_or_load_duplicate(store, request, &ignored_event).await?;

        if callback_event.callback_event_id != ignored_event.callback_event_id {
            return build_duplicate_callback_result(store, callback_event).await;
        }

        return Ok(PaymentCallbackIntakeResult {
            disposition: PaymentCallbackIntakeDisposition::Ignored,
            normalized_outcome: None,
            callback_event,
            payment_order_opt: None,
            payment_attempt_opt: None,
            payment_session_opt: None,
            payment_transaction_opt: None,
        });
    }

    let pending_event = match prepare_verified_callback_event(store, request, &raw_event).await? {
        PreparedVerifiedCallbackEvent::Duplicate(callback_event) => {
            return build_duplicate_callback_result(store, callback_event).await;
        }
        PreparedVerifiedCallbackEvent::Continue(callback_event) => callback_event,
    };

    match process_verified_payment_callback(store, request, pending_event.clone()).await {
        Ok(result) => Ok(result),
        Err(error) => {
            let failed_event = failed_callback_event_record(&pending_event, request);
            store
                .insert_payment_callback_event_record(&failed_event)
                .await?;
            Err(error)
        }
    }
}

fn build_payment_order(
    scope: &PaymentSubjectScope,
    order: &CommerceOrderRecord,
) -> PaymentOrderRecord {
    let payment_status = payment_order_status_for(order);
    let mut record = PaymentOrderRecord::new(
        payment_order_id(&order.order_id),
        order.order_id.clone(),
        scope.tenant_id,
        scope.organization_id,
        scope.user_id,
        order.project_id.clone(),
        order.target_kind.clone(),
        order.target_kind.clone(),
        order.target_id.clone(),
        infer_currency_code(order),
        order.list_price_cents,
    )
    .with_payable_minor(order.payable_price_cents)
    .with_captured_amount_minor(if matches!(payment_status, PaymentOrderStatus::Captured) {
        order.payable_price_cents
    } else {
        0
    })
    .with_provider_code(PaymentProviderCode::Unspecified)
    .with_method_code("provider_handoff")
    .with_payment_status(payment_status)
    .with_fulfillment_status(fulfillment_status_for(order))
    .with_created_at_ms(order.created_at_ms)
    .with_updated_at_ms(order.created_at_ms);

    if order.payable_price_cents < order.list_price_cents {
        record = record.with_discount_minor(order.list_price_cents - order.payable_price_cents);
    }

    record
}

fn build_payment_attempt(
    scope: &PaymentSubjectScope,
    payment_order: &PaymentOrderRecord,
    order: &CommerceOrderRecord,
    client_kind: &str,
) -> PaymentAttemptRecord {
    let normalized_client_kind = client_kind.trim();
    PaymentAttemptRecord::new(
        payment_attempt_id(&order.order_id),
        scope.tenant_id,
        scope.organization_id,
        payment_order.payment_order_id.clone(),
        1,
        "gateway_selection_pending",
        PaymentProviderCode::Unspecified,
        "provider_handoff",
        if normalized_client_kind.is_empty() {
            "portal_web"
        } else {
            normalized_client_kind
        },
        payment_order.payment_order_id.clone(),
    )
    .with_attempt_status(payment_attempt_status_for_payment_order(payment_order))
    .with_request_payload_hash(payment_order.payment_order_id.clone())
    .with_created_at_ms(order.created_at_ms)
    .with_updated_at_ms(order.created_at_ms)
}

fn build_payment_session(
    scope: &PaymentSubjectScope,
    payment_attempt: &PaymentAttemptRecord,
    order: &CommerceOrderRecord,
) -> PaymentSessionRecord {
    PaymentSessionRecord::new(
        payment_session_id(&order.order_id),
        scope.tenant_id,
        scope.organization_id,
        payment_attempt.payment_attempt_id.clone(),
        PaymentSessionKind::HostedCheckout,
        payment_session_status_for_payment_order(payment_attempt, order),
    )
    .with_display_reference(Some(format!(
        "PAY-{}",
        normalize_payment_reference(&order.order_id)
    )))
    .with_expires_at_ms(order.created_at_ms.saturating_add(15 * 60 * 1_000))
    .with_created_at_ms(order.created_at_ms)
    .with_updated_at_ms(order.created_at_ms)
}

fn validate_payment_callback_request(request: &PaymentCallbackIntakeRequest) -> Result<()> {
    ensure!(
        !request.gateway_account_id.trim().is_empty(),
        "gateway_account_id must not be empty"
    );
    ensure!(
        !request.event_type.trim().is_empty(),
        "event_type must not be empty"
    );
    ensure!(
        !request.event_identity.trim().is_empty(),
        "event_identity must not be empty"
    );
    ensure!(
        !request.dedupe_key.trim().is_empty(),
        "dedupe_key must not be empty"
    );
    Ok(())
}

fn build_callback_event_record(
    request: &PaymentCallbackIntakeRequest,
) -> PaymentCallbackEventRecord {
    PaymentCallbackEventRecord::new(
        payment_callback_event_id(request),
        request.scope.tenant_id,
        request.scope.organization_id,
        request.provider_code,
        request.gateway_account_id.clone(),
        request.event_type.clone(),
        request.event_identity.clone(),
        request.dedupe_key.clone(),
        request.received_at_ms,
    )
    .with_payment_order_id(request.payment_order_id.clone())
    .with_payment_attempt_id(request.payment_attempt_id.clone())
    .with_provider_transaction_id(request.provider_transaction_id.clone())
    .with_signature_status(request.signature_status.clone())
    .with_processing_status(PaymentCallbackProcessingStatus::Pending)
    .with_payload_json(request.payload_json.clone())
}

enum PreparedVerifiedCallbackEvent {
    Continue(PaymentCallbackEventRecord),
    Duplicate(PaymentCallbackEventRecord),
}

async fn prepare_verified_callback_event<S>(
    store: &S,
    request: &PaymentCallbackIntakeRequest,
    raw_event: &PaymentCallbackEventRecord,
) -> Result<PreparedVerifiedCallbackEvent>
where
    S: PaymentKernelStore + ?Sized,
{
    if let Some(existing) = store
        .find_payment_callback_event_record_by_dedupe_key(
            request.provider_code,
            &request.gateway_account_id,
            &request.dedupe_key,
        )
        .await?
    {
        return prepare_existing_verified_callback_event(store, request, existing).await;
    }

    match store.insert_payment_callback_event_record(raw_event).await {
        Ok(callback_event) => Ok(PreparedVerifiedCallbackEvent::Continue(callback_event)),
        Err(error) => {
            if let Some(existing) = store
                .find_payment_callback_event_record_by_dedupe_key(
                    request.provider_code,
                    &request.gateway_account_id,
                    &request.dedupe_key,
                )
                .await?
            {
                prepare_existing_verified_callback_event(store, request, existing).await
            } else {
                Err(error)
            }
        }
    }
}

async fn prepare_existing_verified_callback_event<S>(
    store: &S,
    request: &PaymentCallbackIntakeRequest,
    existing: PaymentCallbackEventRecord,
) -> Result<PreparedVerifiedCallbackEvent>
where
    S: PaymentKernelStore + ?Sized,
{
    match existing.processing_status {
        PaymentCallbackProcessingStatus::Processed | PaymentCallbackProcessingStatus::Ignored => {
            Ok(PreparedVerifiedCallbackEvent::Duplicate(existing))
        }
        PaymentCallbackProcessingStatus::Pending | PaymentCallbackProcessingStatus::Failed => {
            let retry_event = retriable_callback_event_record(existing, request);
            let callback_event = store
                .insert_payment_callback_event_record(&retry_event)
                .await?;
            Ok(PreparedVerifiedCallbackEvent::Continue(callback_event))
        }
    }
}

async fn process_verified_payment_callback(
    store: &dyn CommercialKernelStore,
    request: &PaymentCallbackIntakeRequest,
    pending_event: PaymentCallbackEventRecord,
) -> Result<PaymentCallbackIntakeResult> {
    let normalized_outcome = normalize_callback_outcome(request);
    if normalized_outcome.is_none() {
        return Ok(PaymentCallbackIntakeResult {
            disposition: PaymentCallbackIntakeDisposition::RequiresProviderQuery,
            normalized_outcome: None,
            callback_event: pending_event,
            payment_order_opt: None,
            payment_attempt_opt: None,
            payment_session_opt: None,
            payment_transaction_opt: None,
        });
    }

    let payment_order_id = request
        .payment_order_id
        .clone()
        .or_else(|| pending_event.payment_order_id.clone())
        .context(
            "payment callback processing requires payment_order_id until provider correlation is wired",
        )?;
    let mut payment_order = store
        .find_payment_order_record(&payment_order_id)
        .await?
        .with_context(|| format!("payment order not found for callback: {payment_order_id}"))?;
    let payment_attempt_records = store
        .list_payment_attempt_records_for_order(&payment_order_id)
        .await?;
    let mut payment_attempt = select_payment_attempt(
        &payment_attempt_records,
        request.payment_attempt_id.as_deref(),
    );
    let mut payment_session = if let Some(attempt) = payment_attempt.as_ref() {
        resolve_payment_session(store, &attempt.payment_attempt_id).await?
    } else {
        None
    };

    let outcome = normalized_outcome.expect("normalized outcome checked above");
    if let Some(attempt) = payment_attempt.as_mut() {
        apply_outcome_to_payment_attempt(attempt, request, outcome);
    }
    if let Some(session) = payment_session.as_mut() {
        apply_outcome_to_payment_session(session, request, outcome);
    }

    let mut failover_replacement = None;
    let payment_transaction = match outcome {
        PaymentCallbackNormalizedOutcome::Authorized => {
            apply_outcome_to_payment_order(&mut payment_order, request, outcome);
            Some(
                ensure_authorization_payment_transaction_record(
                    store,
                    &payment_order,
                    payment_attempt.as_ref(),
                    &pending_event,
                    request,
                )
                .await?,
            )
        }
        PaymentCallbackNormalizedOutcome::Settled => {
            let settled_sale_application = ensure_sale_payment_transaction_record(
                store,
                &payment_order,
                payment_attempt.as_ref(),
                &pending_event,
                request,
            )
            .await?;
            apply_settled_outcome_to_payment_order(
                &mut payment_order,
                request,
                settled_sale_application.captured_amount_minor,
            );
            Some(settled_sale_application.payment_transaction)
        }
        PaymentCallbackNormalizedOutcome::Failed
        | PaymentCallbackNormalizedOutcome::Canceled
        | PaymentCallbackNormalizedOutcome::Expired => {
            if should_attempt_payment_failover(&payment_order, outcome) {
                failover_replacement = prepare_payment_failover_replacement(
                    store,
                    &payment_order,
                    &payment_attempt_records,
                    payment_attempt.as_ref(),
                    request.received_at_ms,
                )
                .await?;
            }

            if let Some(replacement) = failover_replacement.as_ref() {
                reopen_payment_order_for_failover(
                    &mut payment_order,
                    &replacement.payment_attempt,
                    request.received_at_ms,
                );
            } else {
                apply_outcome_to_payment_order(&mut payment_order, request, outcome);
            }
            None
        }
    };

    if let Some(attempt) = payment_attempt.as_ref() {
        store.insert_payment_attempt_record(attempt).await?;
    }
    if let Some(session) = payment_session.as_ref() {
        store.insert_payment_session_record(session).await?;
    }
    if let Some(replacement) = failover_replacement.as_ref() {
        store
            .insert_payment_attempt_record(&replacement.payment_attempt)
            .await?;
        store
            .insert_payment_session_record(&replacement.payment_session)
            .await?;
    }
    store.insert_payment_order_record(&payment_order).await?;

    if should_finalize_settled_payment_fulfillment(&payment_order, outcome) {
        finalize_settled_payment_fulfillment(store, &mut payment_order, request.received_at_ms)
            .await?;
    }

    let active_payment_attempt = failover_replacement
        .as_ref()
        .map(|replacement| &replacement.payment_attempt)
        .or(payment_attempt.as_ref());
    let active_payment_session = failover_replacement
        .as_ref()
        .map(|replacement| replacement.payment_session.clone())
        .or(payment_session);

    let processed_event = processed_callback_event_record(
        pending_event,
        request,
        &payment_order,
        active_payment_attempt,
    );
    let callback_event = store
        .insert_payment_callback_event_record(&processed_event)
        .await?;

    Ok(PaymentCallbackIntakeResult {
        disposition: PaymentCallbackIntakeDisposition::Processed,
        normalized_outcome: Some(outcome),
        callback_event,
        payment_order_opt: Some(payment_order),
        payment_attempt_opt: active_payment_attempt.cloned(),
        payment_session_opt: active_payment_session,
        payment_transaction_opt: payment_transaction,
    })
}

async fn persist_callback_event_or_load_duplicate<S>(
    store: &S,
    request: &PaymentCallbackIntakeRequest,
    event: &PaymentCallbackEventRecord,
) -> Result<PaymentCallbackEventRecord>
where
    S: PaymentKernelStore + ?Sized,
{
    match store.insert_payment_callback_event_record(event).await {
        Ok(record) => Ok(record),
        Err(error) => {
            if let Some(existing) = store
                .find_payment_callback_event_record_by_dedupe_key(
                    request.provider_code,
                    &request.gateway_account_id,
                    &request.dedupe_key,
                )
                .await?
            {
                Ok(existing)
            } else {
                Err(error)
            }
        }
    }
}

async fn finalize_settled_payment_fulfillment(
    store: &dyn CommercialKernelStore,
    payment_order: &mut PaymentOrderRecord,
    fulfilled_at_ms: u64,
) -> Result<()> {
    let commerce_order = find_commerce_order_for_payment_order(store, payment_order).await?;
    let settled_order = settle_portal_commerce_order_from_verified_payment(
        store,
        &commerce_order.user_id,
        &commerce_order.project_id,
        &commerce_order.order_id,
    )
    .await
    .map_err(anyhow::Error::from)?;
    ensure_commerce_order_account_grant(store, payment_order, &settled_order, fulfilled_at_ms)
        .await?;

    if payment_order.fulfillment_status != "fulfilled" {
        payment_order.fulfillment_status = "fulfilled".to_owned();
        payment_order.updated_at_ms = fulfilled_at_ms;
        payment_order.version = payment_order.version.saturating_add(1);
        *payment_order = store.insert_payment_order_record(payment_order).await?;
    }

    Ok(())
}

async fn find_commerce_order_for_payment_order<S>(
    store: &S,
    payment_order: &PaymentOrderRecord,
) -> Result<CommerceOrderRecord>
where
    S: CommercialKernelStore + ?Sized,
{
    store
        .list_commerce_orders_for_project(&payment_order.project_id)
        .await?
        .into_iter()
        .find(|order| order.order_id == payment_order.commerce_order_id)
        .with_context(|| {
            format!(
                "commerce order not found for payment order {}",
                payment_order.payment_order_id
            )
        })
}

fn retriable_callback_event_record(
    mut existing: PaymentCallbackEventRecord,
    request: &PaymentCallbackIntakeRequest,
) -> PaymentCallbackEventRecord {
    existing.payment_order_id = request
        .payment_order_id
        .clone()
        .or(existing.payment_order_id.clone());
    existing.payment_attempt_id = request
        .payment_attempt_id
        .clone()
        .or(existing.payment_attempt_id.clone());
    existing.provider_transaction_id = request
        .provider_transaction_id
        .clone()
        .or(existing.provider_transaction_id.clone());
    existing.signature_status = request.signature_status.clone();
    existing.processing_status = PaymentCallbackProcessingStatus::Pending;
    existing.payload_json = request
        .payload_json
        .clone()
        .or(existing.payload_json.clone());
    existing.processed_at_ms = None;
    existing
}

fn processed_callback_event_record(
    pending_event: PaymentCallbackEventRecord,
    request: &PaymentCallbackIntakeRequest,
    payment_order: &PaymentOrderRecord,
    payment_attempt_opt: Option<&PaymentAttemptRecord>,
) -> PaymentCallbackEventRecord {
    let mut processed_event = pending_event;
    processed_event.payment_order_id = Some(payment_order.payment_order_id.clone());
    processed_event.payment_attempt_id =
        payment_attempt_opt.map(|attempt| attempt.payment_attempt_id.clone());
    processed_event.provider_transaction_id = request
        .provider_transaction_id
        .clone()
        .or(processed_event.provider_transaction_id.clone());
    processed_event.signature_status = request.signature_status.clone();
    processed_event.processing_status = PaymentCallbackProcessingStatus::Processed;
    processed_event.processed_at_ms = Some(request.received_at_ms);
    processed_event
}

fn failed_callback_event_record(
    pending_event: &PaymentCallbackEventRecord,
    request: &PaymentCallbackIntakeRequest,
) -> PaymentCallbackEventRecord {
    let mut failed_event = pending_event.clone();
    failed_event.payment_order_id = request
        .payment_order_id
        .clone()
        .or(failed_event.payment_order_id.clone());
    failed_event.payment_attempt_id = request
        .payment_attempt_id
        .clone()
        .or(failed_event.payment_attempt_id.clone());
    failed_event.provider_transaction_id = request
        .provider_transaction_id
        .clone()
        .or(failed_event.provider_transaction_id.clone());
    failed_event.signature_status = request.signature_status.clone();
    failed_event.processing_status = PaymentCallbackProcessingStatus::Failed;
    failed_event.payload_json = request
        .payload_json
        .clone()
        .or(failed_event.payload_json.clone());
    failed_event.processed_at_ms = Some(request.received_at_ms);
    failed_event
}

async fn build_duplicate_callback_result<S>(
    store: &S,
    callback_event: PaymentCallbackEventRecord,
) -> Result<PaymentCallbackIntakeResult>
where
    S: PaymentKernelStore + ?Sized,
{
    let (payment_order_opt, payment_attempt_opt, payment_session_opt, payment_transaction_opt) =
        hydrate_payment_callback_context(store, &callback_event).await?;

    Ok(PaymentCallbackIntakeResult {
        disposition: PaymentCallbackIntakeDisposition::Duplicate,
        normalized_outcome: normalized_outcome_from_callback_event(&callback_event),
        callback_event,
        payment_order_opt,
        payment_attempt_opt,
        payment_session_opt,
        payment_transaction_opt,
    })
}

async fn hydrate_payment_callback_context<S>(
    store: &S,
    callback_event: &PaymentCallbackEventRecord,
) -> Result<(
    Option<PaymentOrderRecord>,
    Option<PaymentAttemptRecord>,
    Option<PaymentSessionRecord>,
    Option<PaymentTransactionRecord>,
)>
where
    S: PaymentKernelStore + ?Sized,
{
    let payment_order_opt = match callback_event.payment_order_id.as_deref() {
        Some(payment_order_id) => store.find_payment_order_record(payment_order_id).await?,
        None => None,
    };
    let payment_attempt_opt =
        if let Some(payment_order_id) = callback_event.payment_order_id.as_deref() {
            let attempts = store
                .list_payment_attempt_records_for_order(payment_order_id)
                .await?;
            select_payment_attempt(&attempts, callback_event.payment_attempt_id.as_deref())
        } else {
            None
        };
    let payment_session_opt = if let Some(attempt) = payment_attempt_opt.as_ref() {
        resolve_session_from_records(
            &store
                .list_payment_session_records_for_attempt(&attempt.payment_attempt_id)
                .await?,
        )
    } else {
        None
    };
    let payment_transaction_opt = if let Some(payment_order) = payment_order_opt.as_ref() {
        let transactions = store
            .list_payment_transaction_records_for_order(&payment_order.payment_order_id)
            .await?;
        select_callback_payment_transaction(
            &transactions,
            callback_event.provider_transaction_id.as_deref(),
            normalized_outcome_from_callback_event(callback_event),
        )
    } else {
        None
    };

    Ok((
        payment_order_opt,
        payment_attempt_opt,
        payment_session_opt,
        payment_transaction_opt,
    ))
}

async fn resolve_payment_session<S>(
    store: &S,
    payment_attempt_id: &str,
) -> Result<Option<PaymentSessionRecord>>
where
    S: PaymentKernelStore + ?Sized,
{
    let sessions = store
        .list_payment_session_records_for_attempt(payment_attempt_id)
        .await?;
    Ok(resolve_session_from_records(&sessions))
}

fn select_payment_attempt(
    attempts: &[PaymentAttemptRecord],
    payment_attempt_id: Option<&str>,
) -> Option<PaymentAttemptRecord> {
    if let Some(payment_attempt_id) = payment_attempt_id {
        attempts
            .iter()
            .find(|attempt| attempt.payment_attempt_id == payment_attempt_id)
            .cloned()
    } else {
        attempts.first().cloned()
    }
}

fn resolve_session_from_records(sessions: &[PaymentSessionRecord]) -> Option<PaymentSessionRecord> {
    sessions.first().cloned()
}

async fn prepare_payment_failover_replacement(
    store: &dyn CommercialKernelStore,
    payment_order: &PaymentOrderRecord,
    payment_attempt_records: &[PaymentAttemptRecord],
    current_attempt_opt: Option<&PaymentAttemptRecord>,
    reopened_at_ms: u64,
) -> Result<Option<PaymentFailoverReplacement>> {
    let Some(current_attempt) = current_attempt_opt else {
        return Ok(None);
    };

    if let Some(existing_attempt) =
        select_existing_failover_attempt(payment_attempt_records, current_attempt)
    {
        let payment_session = resolve_or_build_failover_session(
            store,
            payment_order,
            &existing_attempt,
            reopened_at_ms,
        )
        .await?;
        return Ok(Some(PaymentFailoverReplacement {
            payment_attempt: existing_attempt,
            payment_session,
        }));
    }

    let Some((gateway_account, channel_policy)) = select_payment_failover_route(
        store,
        payment_order,
        payment_attempt_records,
        current_attempt,
    )
    .await?
    else {
        return Ok(None);
    };

    let next_attempt_no = payment_attempt_records
        .iter()
        .map(|attempt| attempt.attempt_no)
        .max()
        .unwrap_or(current_attempt.attempt_no)
        .saturating_add(1);
    let payment_attempt = build_failover_payment_attempt(
        payment_order,
        current_attempt,
        &gateway_account,
        &channel_policy,
        next_attempt_no,
        reopened_at_ms,
    );
    let payment_session =
        build_failover_payment_session(payment_order, &payment_attempt, reopened_at_ms);

    Ok(Some(PaymentFailoverReplacement {
        payment_attempt,
        payment_session,
    }))
}

async fn select_payment_failover_route(
    store: &dyn CommercialKernelStore,
    payment_order: &PaymentOrderRecord,
    payment_attempt_records: &[PaymentAttemptRecord],
    current_attempt: &PaymentAttemptRecord,
) -> Result<Option<(PaymentGatewayAccountRecord, PaymentChannelPolicyRecord)>> {
    let used_gateway_account_ids =
        used_gateway_account_ids_for_payment_order(payment_attempt_records, current_attempt);

    let mut channel_policies = store.list_payment_channel_policy_records().await?;
    channel_policies.retain(|policy| {
        policy.tenant_id == payment_order.tenant_id
            && policy.organization_id == payment_order.organization_id
            && is_active_payment_route_status(&policy.status)
            && payment_channel_policy_matches_order(
                policy,
                payment_order,
                &current_attempt.client_kind,
            )
    });
    channel_policies.sort_by(|left, right| {
        right
            .priority
            .cmp(&left.priority)
            .then_with(|| left.channel_policy_id.cmp(&right.channel_policy_id))
    });

    let mut gateway_accounts = store.list_payment_gateway_account_records().await?;
    gateway_accounts.retain(|gateway_account| {
        gateway_account.tenant_id == payment_order.tenant_id
            && gateway_account.organization_id == payment_order.organization_id
            && is_active_payment_route_status(&gateway_account.status)
    });
    gateway_accounts.sort_by(|left, right| {
        right
            .priority
            .cmp(&left.priority)
            .then_with(|| left.gateway_account_id.cmp(&right.gateway_account_id))
    });

    for channel_policy in channel_policies {
        if let Some(gateway_account) = gateway_accounts.iter().find(|gateway_account| {
            gateway_account.provider_code == channel_policy.provider_code
                && !used_gateway_account_ids.contains(&gateway_account.gateway_account_id)
        }) {
            return Ok(Some((gateway_account.clone(), channel_policy)));
        }
    }

    Ok(None)
}

async fn resolve_or_build_failover_session(
    store: &dyn CommercialKernelStore,
    payment_order: &PaymentOrderRecord,
    payment_attempt: &PaymentAttemptRecord,
    reopened_at_ms: u64,
) -> Result<PaymentSessionRecord> {
    if let Some(payment_session) =
        resolve_payment_session(store, &payment_attempt.payment_attempt_id).await?
    {
        if !payment_session.session_status.is_terminal() {
            return Ok(payment_session);
        }
    }

    Ok(build_failover_payment_session(
        payment_order,
        payment_attempt,
        reopened_at_ms,
    ))
}

fn select_existing_failover_attempt(
    payment_attempt_records: &[PaymentAttemptRecord],
    current_attempt: &PaymentAttemptRecord,
) -> Option<PaymentAttemptRecord> {
    payment_attempt_records
        .iter()
        .filter(|attempt| {
            attempt.payment_attempt_id != current_attempt.payment_attempt_id
                && attempt.attempt_no > current_attempt.attempt_no
                && matches!(
                    attempt.attempt_status,
                    PaymentAttemptStatus::Initiated
                        | PaymentAttemptStatus::HandoffReady
                        | PaymentAttemptStatus::Processing
                        | PaymentAttemptStatus::Authorized
                )
        })
        .max_by(|left, right| {
            left.attempt_no
                .cmp(&right.attempt_no)
                .then_with(|| left.payment_attempt_id.cmp(&right.payment_attempt_id))
        })
        .cloned()
}

fn used_gateway_account_ids_for_payment_order(
    payment_attempt_records: &[PaymentAttemptRecord],
    current_attempt: &PaymentAttemptRecord,
) -> HashSet<String> {
    let mut used_gateway_account_ids = HashSet::new();
    let mut current_attempt_seen = false;

    for attempt in payment_attempt_records {
        let gateway_account_id = if attempt.payment_attempt_id == current_attempt.payment_attempt_id
        {
            current_attempt_seen = true;
            current_attempt.gateway_account_id.as_str()
        } else {
            attempt.gateway_account_id.as_str()
        };

        if is_concrete_gateway_account_id(gateway_account_id) {
            used_gateway_account_ids.insert(gateway_account_id.to_owned());
        }
    }

    if !current_attempt_seen && is_concrete_gateway_account_id(&current_attempt.gateway_account_id)
    {
        used_gateway_account_ids.insert(current_attempt.gateway_account_id.clone());
    }

    used_gateway_account_ids
}

fn is_concrete_gateway_account_id(gateway_account_id: &str) -> bool {
    let normalized = gateway_account_id.trim();
    !normalized.is_empty() && normalized != "gateway_selection_pending"
}

fn payment_channel_policy_matches_order(
    channel_policy: &PaymentChannelPolicyRecord,
    payment_order: &PaymentOrderRecord,
    client_kind: &str,
) -> bool {
    channel_policy.country_code.trim().is_empty()
        && payment_route_scope_matches(&channel_policy.scene_code, &payment_order.order_kind)
        && payment_route_scope_matches(&channel_policy.currency_code, &payment_order.currency_code)
        && payment_route_scope_matches(&channel_policy.client_kind, client_kind)
}

fn payment_route_scope_matches(expected: &str, actual: &str) -> bool {
    expected.trim().is_empty() || expected.trim().eq_ignore_ascii_case(actual.trim())
}

fn is_active_payment_route_status(status: &str) -> bool {
    status.trim().eq_ignore_ascii_case("active")
}

fn should_attempt_payment_failover(
    payment_order: &PaymentOrderRecord,
    outcome: PaymentCallbackNormalizedOutcome,
) -> bool {
    matches!(
        outcome,
        PaymentCallbackNormalizedOutcome::Failed | PaymentCallbackNormalizedOutcome::Expired
    ) && !matches!(
        payment_order.payment_status,
        PaymentOrderStatus::Authorized
            | PaymentOrderStatus::PartiallyCaptured
            | PaymentOrderStatus::Captured
    )
}

fn reopen_payment_order_for_failover(
    payment_order: &mut PaymentOrderRecord,
    payment_attempt: &PaymentAttemptRecord,
    reopened_at_ms: u64,
) {
    payment_order.provider_code = payment_attempt.provider_code;
    payment_order.method_code = Some(payment_attempt.method_code.clone());
    payment_order.payment_status = PaymentOrderStatus::AwaitingCustomer;
    payment_order.fulfillment_status = "pending".to_owned();
    payment_order.updated_at_ms = reopened_at_ms;
    payment_order.version = payment_order.version.saturating_add(1);
}

fn build_failover_payment_attempt(
    payment_order: &PaymentOrderRecord,
    current_attempt: &PaymentAttemptRecord,
    gateway_account: &PaymentGatewayAccountRecord,
    channel_policy: &PaymentChannelPolicyRecord,
    attempt_no: u32,
    created_at_ms: u64,
) -> PaymentAttemptRecord {
    let client_kind = if current_attempt.client_kind.trim().is_empty() {
        "portal_web"
    } else {
        current_attempt.client_kind.as_str()
    };

    PaymentAttemptRecord::new(
        failover_payment_attempt_id(&payment_order.payment_order_id, attempt_no),
        payment_order.tenant_id,
        payment_order.organization_id,
        payment_order.payment_order_id.clone(),
        attempt_no,
        gateway_account.gateway_account_id.clone(),
        channel_policy.provider_code,
        channel_policy.method_code.clone(),
        client_kind,
        format!("{}:{attempt_no}", payment_order.payment_order_id),
    )
    .with_attempt_status(PaymentAttemptStatus::HandoffReady)
    .with_request_payload_hash(payment_order.payment_order_id.clone())
    .with_created_at_ms(created_at_ms)
    .with_updated_at_ms(created_at_ms)
}

fn build_failover_payment_session(
    payment_order: &PaymentOrderRecord,
    payment_attempt: &PaymentAttemptRecord,
    created_at_ms: u64,
) -> PaymentSessionRecord {
    PaymentSessionRecord::new(
        failover_payment_session_id(&payment_order.payment_order_id, payment_attempt.attempt_no),
        payment_order.tenant_id,
        payment_order.organization_id,
        payment_attempt.payment_attempt_id.clone(),
        payment_session_kind_for_method_code(&payment_attempt.method_code),
        PaymentSessionStatus::Open,
    )
    .with_display_reference(Some(format!(
        "PAY-{}-{}",
        normalize_payment_reference(&payment_order.commerce_order_id),
        payment_attempt.attempt_no
    )))
    .with_expires_at_ms(created_at_ms.saturating_add(15 * 60 * 1_000))
    .with_created_at_ms(created_at_ms)
    .with_updated_at_ms(created_at_ms)
}

fn payment_session_kind_for_method_code(method_code: &str) -> PaymentSessionKind {
    let normalized = method_code.trim().to_ascii_lowercase();
    if normalized.contains("qr") {
        PaymentSessionKind::QrCode
    } else if normalized.contains("redirect") {
        PaymentSessionKind::Redirect
    } else {
        PaymentSessionKind::HostedCheckout
    }
}

fn signature_status_allows_processing(signature_status: &str) -> bool {
    matches!(
        signature_status.trim().to_ascii_lowercase().as_str(),
        "verified" | "trusted" | "trusted_internal" | "verified_test"
    )
}

fn normalize_callback_outcome(
    request: &PaymentCallbackIntakeRequest,
) -> Option<PaymentCallbackNormalizedOutcome> {
    let event_type = request.event_type.trim().to_ascii_lowercase();
    let provider_status = request
        .provider_status
        .as_deref()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();

    if contains_any_token(
        &event_type,
        &[
            "completed",
            "captured",
            "paid",
            "settled",
            "succeeded",
            "success",
        ],
    ) || contains_any_token(
        &provider_status,
        &["captured", "settled", "succeeded", "success"],
    ) {
        return Some(PaymentCallbackNormalizedOutcome::Settled);
    }

    if contains_any_token(
        &event_type,
        &[
            "authorized",
            "authorised",
            "requires_capture",
            "awaiting_capture",
            "capturable",
        ],
    ) || contains_any_token(
        &provider_status,
        &[
            "authorized",
            "authorised",
            "requires_capture",
            "awaiting_capture",
            "capturable",
        ],
    ) {
        return Some(PaymentCallbackNormalizedOutcome::Authorized);
    }

    if contains_any_token(&event_type, &["expired", "timeout"])
        || contains_any_token(&provider_status, &["expired", "timeout"])
    {
        return Some(PaymentCallbackNormalizedOutcome::Expired);
    }

    if contains_any_token(&event_type, &["canceled", "cancelled", "closed", "voided"])
        || contains_any_token(
            &provider_status,
            &["canceled", "cancelled", "closed", "voided"],
        )
    {
        return Some(PaymentCallbackNormalizedOutcome::Canceled);
    }

    if contains_any_token(
        &event_type,
        &["failed", "failure", "declined", "rejected", "error"],
    ) || contains_any_token(
        &provider_status,
        &["failed", "failure", "declined", "rejected", "error"],
    ) {
        return Some(PaymentCallbackNormalizedOutcome::Failed);
    }

    None
}

fn contains_any_token(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}

fn payment_order_status_for_outcome(
    outcome: PaymentCallbackNormalizedOutcome,
) -> PaymentOrderStatus {
    match outcome {
        PaymentCallbackNormalizedOutcome::Authorized => PaymentOrderStatus::Authorized,
        PaymentCallbackNormalizedOutcome::Settled => PaymentOrderStatus::Captured,
        PaymentCallbackNormalizedOutcome::Failed => PaymentOrderStatus::Failed,
        PaymentCallbackNormalizedOutcome::Canceled => PaymentOrderStatus::Canceled,
        PaymentCallbackNormalizedOutcome::Expired => PaymentOrderStatus::Expired,
    }
}

fn payment_attempt_status_for_outcome(
    outcome: PaymentCallbackNormalizedOutcome,
) -> PaymentAttemptStatus {
    match outcome {
        PaymentCallbackNormalizedOutcome::Authorized => PaymentAttemptStatus::Authorized,
        PaymentCallbackNormalizedOutcome::Settled => PaymentAttemptStatus::Succeeded,
        PaymentCallbackNormalizedOutcome::Failed => PaymentAttemptStatus::Failed,
        PaymentCallbackNormalizedOutcome::Canceled => PaymentAttemptStatus::Canceled,
        PaymentCallbackNormalizedOutcome::Expired => PaymentAttemptStatus::Expired,
    }
}

fn payment_session_status_for_outcome(
    outcome: PaymentCallbackNormalizedOutcome,
) -> PaymentSessionStatus {
    match outcome {
        PaymentCallbackNormalizedOutcome::Authorized => PaymentSessionStatus::Authorized,
        PaymentCallbackNormalizedOutcome::Settled => PaymentSessionStatus::Settled,
        PaymentCallbackNormalizedOutcome::Failed => PaymentSessionStatus::Failed,
        PaymentCallbackNormalizedOutcome::Canceled => PaymentSessionStatus::Canceled,
        PaymentCallbackNormalizedOutcome::Expired => PaymentSessionStatus::Expired,
    }
}

fn fulfillment_status_for_outcome(
    payment_order: &PaymentOrderRecord,
    outcome: PaymentCallbackNormalizedOutcome,
) -> String {
    match outcome {
        PaymentCallbackNormalizedOutcome::Authorized => "authorized_pending_capture".to_owned(),
        PaymentCallbackNormalizedOutcome::Settled => "captured_pending_fulfillment".to_owned(),
        PaymentCallbackNormalizedOutcome::Failed => {
            if payment_order.payment_status.supports_refund() {
                payment_order.fulfillment_status.clone()
            } else {
                "failed".to_owned()
            }
        }
        PaymentCallbackNormalizedOutcome::Canceled => {
            if payment_order.payment_status.supports_refund() {
                payment_order.fulfillment_status.clone()
            } else {
                "canceled".to_owned()
            }
        }
        PaymentCallbackNormalizedOutcome::Expired => {
            if payment_order.payment_status.supports_refund() {
                payment_order.fulfillment_status.clone()
            } else {
                "expired".to_owned()
            }
        }
    }
}

fn apply_outcome_to_payment_order(
    payment_order: &mut PaymentOrderRecord,
    request: &PaymentCallbackIntakeRequest,
    outcome: PaymentCallbackNormalizedOutcome,
) {
    debug_assert!(!matches!(
        outcome,
        PaymentCallbackNormalizedOutcome::Settled
    ));
    payment_order.provider_code = request.provider_code;
    payment_order.updated_at_ms = request.received_at_ms;
    payment_order.version = payment_order.version.saturating_add(1);
    payment_order.payment_status = reconcile_payment_order_status(
        payment_order.payment_status,
        payment_order_status_for_outcome(outcome),
    );
    payment_order.fulfillment_status = reconcile_fulfillment_status(
        &payment_order.fulfillment_status,
        &fulfillment_status_for_outcome(payment_order, outcome),
    );
}

fn apply_settled_outcome_to_payment_order(
    payment_order: &mut PaymentOrderRecord,
    request: &PaymentCallbackIntakeRequest,
    captured_amount_minor: u64,
) {
    let captured_amount_minor =
        effective_captured_amount_minor(payment_order).max(captured_amount_minor);
    payment_order.provider_code = request.provider_code;
    payment_order.updated_at_ms = request.received_at_ms;
    payment_order.version = payment_order.version.saturating_add(1);
    payment_order.payment_status = reconcile_payment_order_status(
        payment_order.payment_status,
        payment_order_status_for_settled_capture(payment_order, captured_amount_minor),
    );
    payment_order.captured_amount_minor = captured_amount_minor;
    payment_order.fulfillment_status = reconcile_fulfillment_status(
        &payment_order.fulfillment_status,
        &fulfillment_status_for_settled_capture(payment_order, captured_amount_minor),
    );
}

fn apply_outcome_to_payment_attempt(
    payment_attempt: &mut PaymentAttemptRecord,
    request: &PaymentCallbackIntakeRequest,
    outcome: PaymentCallbackNormalizedOutcome,
) {
    payment_attempt.gateway_account_id = request.gateway_account_id.clone();
    payment_attempt.provider_code = request.provider_code;
    payment_attempt.provider_payment_reference = request.provider_transaction_id.clone();
    payment_attempt.updated_at_ms = request.received_at_ms;
    payment_attempt.attempt_status = reconcile_payment_attempt_status(
        payment_attempt.attempt_status,
        payment_attempt_status_for_outcome(outcome),
    );
}

fn apply_outcome_to_payment_session(
    payment_session: &mut PaymentSessionRecord,
    request: &PaymentCallbackIntakeRequest,
    outcome: PaymentCallbackNormalizedOutcome,
) {
    payment_session.updated_at_ms = request.received_at_ms;
    payment_session.session_status = reconcile_payment_session_status(
        payment_session.session_status,
        payment_session_status_for_outcome(outcome),
    );
}

fn build_payment_transaction_record(
    payment_order: &PaymentOrderRecord,
    payment_attempt_opt: Option<&PaymentAttemptRecord>,
    callback_event: &PaymentCallbackEventRecord,
    request: &PaymentCallbackIntakeRequest,
    amount_minor: u64,
) -> PaymentTransactionRecord {
    let provider_transaction_id = callback_provider_transaction_id(callback_event, request);
    PaymentTransactionRecord::new(
        sale_payment_transaction_id(&payment_order.payment_order_id, &provider_transaction_id),
        payment_order.tenant_id,
        payment_order.organization_id,
        payment_order.payment_order_id.clone(),
        PaymentTransactionKind::Sale,
        request.provider_code,
        provider_transaction_id,
        request
            .currency_code
            .clone()
            .unwrap_or_else(|| payment_order.currency_code.clone()),
        amount_minor,
        request.received_at_ms,
    )
    .with_payment_attempt_id(
        payment_attempt_opt.map(|payment_attempt| payment_attempt.payment_attempt_id.clone()),
    )
    .with_fee_minor(request.fee_minor)
    .with_net_amount_minor(request.net_amount_minor)
    .with_provider_status(
        request
            .provider_status
            .clone()
            .unwrap_or_else(|| "succeeded".to_owned()),
    )
    .with_raw_event_id(Some(callback_event.callback_event_id.clone()))
    .with_created_at_ms(request.received_at_ms)
}

fn build_authorization_payment_transaction_record(
    payment_order: &PaymentOrderRecord,
    payment_attempt_opt: Option<&PaymentAttemptRecord>,
    callback_event: &PaymentCallbackEventRecord,
    request: &PaymentCallbackIntakeRequest,
) -> PaymentTransactionRecord {
    let provider_transaction_id = callback_provider_transaction_id(callback_event, request);
    PaymentTransactionRecord::new(
        authorization_payment_transaction_id(&payment_order.payment_order_id),
        payment_order.tenant_id,
        payment_order.organization_id,
        payment_order.payment_order_id.clone(),
        PaymentTransactionKind::Authorization,
        request.provider_code,
        provider_transaction_id,
        request
            .currency_code
            .clone()
            .unwrap_or_else(|| payment_order.currency_code.clone()),
        request.amount_minor.unwrap_or(payment_order.payable_minor),
        request.received_at_ms,
    )
    .with_payment_attempt_id(
        payment_attempt_opt.map(|payment_attempt| payment_attempt.payment_attempt_id.clone()),
    )
    .with_provider_status(
        request
            .provider_status
            .clone()
            .unwrap_or_else(|| "authorized".to_owned()),
    )
    .with_raw_event_id(Some(callback_event.callback_event_id.clone()))
    .with_created_at_ms(request.received_at_ms)
}

async fn ensure_authorization_payment_transaction_record(
    store: &dyn CommercialKernelStore,
    payment_order: &PaymentOrderRecord,
    payment_attempt_opt: Option<&PaymentAttemptRecord>,
    callback_event: &PaymentCallbackEventRecord,
    request: &PaymentCallbackIntakeRequest,
) -> Result<PaymentTransactionRecord> {
    let transactions = store
        .list_payment_transaction_records_for_order(&payment_order.payment_order_id)
        .await?;
    if let Some(existing) = select_primary_payment_transaction_by_kind(
        &transactions,
        PaymentTransactionKind::Authorization,
    ) {
        return Ok(existing);
    }

    let authorization_transaction = build_authorization_payment_transaction_record(
        payment_order,
        payment_attempt_opt,
        callback_event,
        request,
    );
    store
        .insert_payment_transaction_record(&authorization_transaction)
        .await
}

async fn ensure_sale_payment_transaction_record(
    store: &dyn CommercialKernelStore,
    payment_order: &PaymentOrderRecord,
    payment_attempt_opt: Option<&PaymentAttemptRecord>,
    callback_event: &PaymentCallbackEventRecord,
    request: &PaymentCallbackIntakeRequest,
) -> Result<SettledSaleApplication> {
    let provider_transaction_id = callback_provider_transaction_id(callback_event, request);
    let settled_amount_minor = request.amount_minor.unwrap_or(payment_order.payable_minor);

    let transactions = store
        .list_payment_transaction_records_for_order(&payment_order.payment_order_id)
        .await?;
    let sale_transactions = sale_transactions(&transactions);
    let current_captured_amount_minor = aggregate_captured_amount_minor(&sale_transactions);
    if let Some(existing) = find_sale_transaction_by_provider_transaction_id(
        &sale_transactions,
        &provider_transaction_id,
    ) {
        let accepted_amount_minor = accepted_sale_capture_amount_minor(
            payment_order,
            &sale_transactions,
            Some(&existing),
            settled_amount_minor,
        );
        let reconciled = reconcile_sale_payment_transaction_record(
            &existing,
            payment_order,
            payment_attempt_opt,
            callback_event,
            request,
            accepted_amount_minor,
        );
        if reconciled != existing {
            let payment_transaction = store.insert_payment_transaction_record(&reconciled).await?;
            if settled_amount_minor > payment_transaction.amount_minor {
                persist_sale_capture_amount_capped_reconciliation(
                    store,
                    payment_order,
                    &provider_transaction_id,
                    settled_amount_minor,
                    payment_transaction.amount_minor,
                    request.received_at_ms,
                )
                .await?;
            }
            return Ok(SettledSaleApplication {
                captured_amount_minor: aggregate_captured_amount_minor_with_upsert(
                    &sale_transactions,
                    &payment_transaction,
                ),
                payment_transaction,
            });
        }
        if settled_amount_minor > existing.amount_minor {
            persist_sale_capture_amount_capped_reconciliation(
                store,
                payment_order,
                &provider_transaction_id,
                settled_amount_minor,
                existing.amount_minor,
                request.received_at_ms,
            )
            .await?;
        }
        return Ok(SettledSaleApplication {
            captured_amount_minor: current_captured_amount_minor,
            payment_transaction: existing,
        });
    }

    let accepted_amount_minor = accepted_sale_capture_amount_minor(
        payment_order,
        &sale_transactions,
        None,
        settled_amount_minor,
    );
    if accepted_amount_minor > 0 {
        let sale_transaction = build_payment_transaction_record(
            payment_order,
            payment_attempt_opt,
            callback_event,
            request,
            accepted_amount_minor,
        );
        let payment_transaction = store
            .insert_payment_transaction_record(&sale_transaction)
            .await?;
        if settled_amount_minor > payment_transaction.amount_minor {
            persist_sale_capture_amount_capped_reconciliation(
                store,
                payment_order,
                &provider_transaction_id,
                settled_amount_minor,
                payment_transaction.amount_minor,
                request.received_at_ms,
            )
            .await?;
        }
        return Ok(SettledSaleApplication {
            captured_amount_minor: current_captured_amount_minor
                .saturating_add(payment_transaction.amount_minor),
            payment_transaction,
        });
    }

    let existing =
        select_primary_payment_transaction_by_kind(&transactions, PaymentTransactionKind::Sale)
            .context("sale transaction conflict requires an existing sale transaction")?;
    persist_sale_provider_transaction_conflict(
        store,
        payment_order,
        &existing,
        &provider_transaction_id,
        settled_amount_minor,
        request.received_at_ms,
    )
    .await?;
    Ok(SettledSaleApplication {
        captured_amount_minor: current_captured_amount_minor,
        payment_transaction: existing,
    })
}

async fn persist_sale_provider_transaction_conflict(
    store: &dyn CommercialKernelStore,
    payment_order: &PaymentOrderRecord,
    existing_transaction: &PaymentTransactionRecord,
    conflicting_provider_transaction_id: &str,
    settled_amount_minor: u64,
    settled_at_ms: u64,
) -> Result<ReconciliationMatchSummaryRecord> {
    let reconciliation = ReconciliationMatchSummaryRecord::new(
        payment_conflict_reconciliation_line_id(
            &payment_order.payment_order_id,
            conflicting_provider_transaction_id,
        ),
        payment_order.tenant_id,
        payment_order.organization_id,
        payment_conflict_reconciliation_batch_id(&payment_order.payment_order_id),
        conflicting_provider_transaction_id,
        ReconciliationMatchStatus::MismatchReference,
        settled_amount_minor,
    )
    .with_local_amount_minor(Some(existing_transaction.amount_minor))
    .with_payment_order_id(Some(payment_order.payment_order_id.clone()))
    .with_reason_code(Some("payment_provider_transaction_conflict".to_owned()))
    .with_created_at_ms(settled_at_ms);
    store
        .insert_reconciliation_match_summary_record(&reconciliation)
        .await
}

async fn persist_sale_capture_amount_capped_reconciliation(
    store: &dyn CommercialKernelStore,
    payment_order: &PaymentOrderRecord,
    provider_transaction_id: &str,
    provider_amount_minor: u64,
    local_amount_minor: u64,
    settled_at_ms: u64,
) -> Result<ReconciliationMatchSummaryRecord> {
    let reconciliation = ReconciliationMatchSummaryRecord::new(
        payment_amount_capped_reconciliation_line_id(
            &payment_order.payment_order_id,
            provider_transaction_id,
        ),
        payment_order.tenant_id,
        payment_order.organization_id,
        payment_conflict_reconciliation_batch_id(&payment_order.payment_order_id),
        provider_transaction_id,
        ReconciliationMatchStatus::MismatchAmount,
        provider_amount_minor,
    )
    .with_local_amount_minor(Some(local_amount_minor))
    .with_payment_order_id(Some(payment_order.payment_order_id.clone()))
    .with_reason_code(Some("payment_capture_amount_capped".to_owned()))
    .with_created_at_ms(settled_at_ms);
    store
        .insert_reconciliation_match_summary_record(&reconciliation)
        .await
}

fn select_primary_payment_transaction_by_kind(
    transactions: &[PaymentTransactionRecord],
    transaction_kind: PaymentTransactionKind,
) -> Option<PaymentTransactionRecord> {
    transactions
        .iter()
        .filter(|transaction| transaction.transaction_kind == transaction_kind)
        .cloned()
        .min_by(|left, right| {
            left.occurred_at_ms
                .cmp(&right.occurred_at_ms)
                .then_with(|| {
                    left.payment_transaction_id
                        .cmp(&right.payment_transaction_id)
                })
        })
}

fn select_callback_payment_transaction(
    transactions: &[PaymentTransactionRecord],
    provider_transaction_id_opt: Option<&str>,
    outcome_opt: Option<PaymentCallbackNormalizedOutcome>,
) -> Option<PaymentTransactionRecord> {
    let preferred_transaction_kind = match outcome_opt {
        Some(PaymentCallbackNormalizedOutcome::Authorized) => {
            Some(PaymentTransactionKind::Authorization)
        }
        Some(PaymentCallbackNormalizedOutcome::Settled) => Some(PaymentTransactionKind::Sale),
        Some(PaymentCallbackNormalizedOutcome::Failed)
        | Some(PaymentCallbackNormalizedOutcome::Canceled)
        | Some(PaymentCallbackNormalizedOutcome::Expired)
        | None => None,
    };

    if let Some(provider_transaction_id) = provider_transaction_id_opt {
        let exact_matches = transactions
            .iter()
            .filter(|transaction| transaction.provider_transaction_id == provider_transaction_id)
            .cloned()
            .collect::<Vec<_>>();
        if !exact_matches.is_empty() {
            return preferred_transaction_kind
                .and_then(|transaction_kind| {
                    select_primary_payment_transaction_by_kind(&exact_matches, transaction_kind)
                })
                .or_else(|| exact_matches.into_iter().next());
        }
    }

    preferred_transaction_kind.and_then(|transaction_kind| {
        select_primary_payment_transaction_by_kind(transactions, transaction_kind)
    })
}

async fn ensure_refund_payment_transaction_record(
    store: &dyn CommercialKernelStore,
    refund_order: &RefundOrderRecord,
    payment_order: &PaymentOrderRecord,
    provider_refund_id: &str,
    refunded_amount_minor: u64,
    finalized_at_ms: u64,
) -> Result<PaymentTransactionRecord> {
    let payment_transaction_id = refund_payment_transaction_id(&refund_order.refund_order_id);
    if let Some(existing) = store
        .list_payment_transaction_records_for_order(&payment_order.payment_order_id)
        .await?
        .into_iter()
        .find(|transaction| transaction.payment_transaction_id == payment_transaction_id)
    {
        if existing.provider_transaction_id != provider_refund_id {
            persist_refund_provider_transaction_conflict(
                store,
                refund_order,
                payment_order,
                &existing,
                provider_refund_id,
                refunded_amount_minor,
                finalized_at_ms,
            )
            .await?;
        }
        return Ok(existing);
    }

    let refund_transaction = build_refund_payment_transaction_record(
        refund_order,
        payment_order,
        provider_refund_id,
        refunded_amount_minor,
        finalized_at_ms,
    );
    store
        .insert_payment_transaction_record(&refund_transaction)
        .await
}

async fn persist_refund_provider_transaction_conflict(
    store: &dyn CommercialKernelStore,
    refund_order: &RefundOrderRecord,
    payment_order: &PaymentOrderRecord,
    existing_transaction: &PaymentTransactionRecord,
    conflicting_provider_refund_id: &str,
    refunded_amount_minor: u64,
    finalized_at_ms: u64,
) -> Result<ReconciliationMatchSummaryRecord> {
    let reconciliation = ReconciliationMatchSummaryRecord::new(
        refund_conflict_reconciliation_line_id(
            &refund_order.refund_order_id,
            conflicting_provider_refund_id,
        ),
        payment_order.tenant_id,
        payment_order.organization_id,
        refund_conflict_reconciliation_batch_id(&refund_order.refund_order_id),
        conflicting_provider_refund_id,
        ReconciliationMatchStatus::MismatchReference,
        refunded_amount_minor,
    )
    .with_local_amount_minor(Some(existing_transaction.amount_minor))
    .with_payment_order_id(Some(payment_order.payment_order_id.clone()))
    .with_refund_order_id(Some(refund_order.refund_order_id.clone()))
    .with_reason_code(Some("refund_provider_transaction_conflict".to_owned()))
    .with_created_at_ms(finalized_at_ms);
    store
        .insert_reconciliation_match_summary_record(&reconciliation)
        .await
}

fn build_refund_payment_transaction_record(
    refund_order: &RefundOrderRecord,
    payment_order: &PaymentOrderRecord,
    provider_refund_id: &str,
    refunded_amount_minor: u64,
    finalized_at_ms: u64,
) -> PaymentTransactionRecord {
    PaymentTransactionRecord::new(
        refund_payment_transaction_id(&refund_order.refund_order_id),
        payment_order.tenant_id,
        payment_order.organization_id,
        payment_order.payment_order_id.clone(),
        PaymentTransactionKind::Refund,
        payment_order.provider_code,
        provider_refund_id.to_owned(),
        payment_order.currency_code.clone(),
        refunded_amount_minor,
        finalized_at_ms,
    )
    .with_provider_status("succeeded")
    .with_created_at_ms(finalized_at_ms)
}

fn build_refund_finance_journal_entry_record(
    refund_order: &RefundOrderRecord,
    currency_code: &str,
    finalized_at_ms: u64,
) -> FinanceJournalEntryRecord {
    FinanceJournalEntryRecord::new(
        finance_journal_entry_id(&refund_order.refund_order_id),
        refund_order.tenant_id,
        refund_order.organization_id,
        "refund_order",
        refund_order.refund_order_id.clone(),
        FinanceEntryCode::RefundPayout,
        currency_code.to_owned(),
        finalized_at_ms,
    )
    .with_entry_status("posted")
    .with_created_at_ms(finalized_at_ms)
}

fn build_refund_finance_journal_line_records(
    finance_entry: &FinanceJournalEntryRecord,
    refund_order: &RefundOrderRecord,
    refunded_amount_minor: u64,
) -> Vec<FinanceJournalLineRecord> {
    vec![
        FinanceJournalLineRecord::new(
            finance_journal_line_id(&finance_entry.finance_journal_entry_id, 1),
            finance_entry.tenant_id,
            finance_entry.organization_id,
            finance_entry.finance_journal_entry_id.clone(),
            1,
            "customer_prepaid_liability",
            FinanceDirection::Debit,
            refunded_amount_minor,
        )
        .with_party_type(Some("refund_order".to_owned()))
        .with_party_id(Some(refund_order.refund_order_id.clone())),
        FinanceJournalLineRecord::new(
            finance_journal_line_id(&finance_entry.finance_journal_entry_id, 2),
            finance_entry.tenant_id,
            finance_entry.organization_id,
            finance_entry.finance_journal_entry_id.clone(),
            2,
            "payment_refund_clearing",
            FinanceDirection::Credit,
            refunded_amount_minor,
        )
        .with_party_type(Some("refund_order".to_owned()))
        .with_party_id(Some(refund_order.refund_order_id.clone())),
    ]
}

async fn load_portal_scoped_commerce_order(
    store: &dyn AdminStore,
    portal_user_id: &str,
    project_id: &str,
    order_id: &str,
) -> Result<CommerceOrderRecord> {
    ensure!(
        !portal_user_id.trim().is_empty(),
        "portal_user_id must not be empty"
    );
    ensure!(
        !project_id.trim().is_empty(),
        "project_id must not be empty"
    );
    ensure!(!order_id.trim().is_empty(), "order_id must not be empty");

    let order = store
        .list_commerce_orders_for_project(project_id.trim())
        .await?
        .into_iter()
        .find(|order| order.order_id == order_id.trim())
        .with_context(|| format!("commerce order not found: {}", order_id.trim()))?;
    ensure!(
        order.user_id == portal_user_id.trim(),
        "commerce order {} is outside the current portal user scope",
        order.order_id
    );
    Ok(order)
}

async fn find_project_payment_order_for_commerce_order(
    store: &dyn PaymentKernelStore,
    project_id: &str,
    order_id: &str,
) -> Result<PaymentOrderRecord> {
    ensure!(
        !project_id.trim().is_empty(),
        "project_id must not be empty"
    );
    ensure!(!order_id.trim().is_empty(), "order_id must not be empty");

    store
        .list_payment_order_records()
        .await?
        .into_iter()
        .find(|payment_order| {
            payment_order.project_id == project_id.trim()
                && payment_order.commerce_order_id == order_id.trim()
        })
        .with_context(|| {
            format!(
                "payment order not found for commerce order {}",
                order_id.trim()
            )
        })
}

fn ensure_supported_refund_target_kind(target_kind: &str) -> Result<()> {
    ensure!(
        matches!(target_kind, "recharge_pack" | "custom_recharge"),
        "refunds for {target_kind} are not supported in this tranche"
    );
    Ok(())
}

fn reserved_refund_amount_minor(refunds: &[RefundOrderRecord]) -> u64 {
    refunds
        .iter()
        .filter(|refund| {
            !matches!(
                refund.refund_status,
                RefundOrderStatus::Failed | RefundOrderStatus::Canceled
            )
        })
        .map(|refund| {
            refund
                .approved_amount_minor
                .unwrap_or(refund.requested_amount_minor)
                .max(refund.refunded_amount_minor)
        })
        .fold(0_u64, u64::saturating_add)
}

fn find_reusable_pending_refund_request(
    refunds: &[RefundOrderRecord],
    refund_reason_code: &str,
    requested_by_type: &str,
    requested_by_id: &str,
    requested_amount_minor: u64,
) -> Option<RefundOrderRecord> {
    refunds
        .iter()
        .filter(|refund| {
            matches!(
                refund.refund_status,
                RefundOrderStatus::Requested
                    | RefundOrderStatus::AwaitingApproval
                    | RefundOrderStatus::Approved
                    | RefundOrderStatus::Processing
            ) && refund.refund_reason_code == refund_reason_code
                && refund.requested_by_type == requested_by_type
                && refund.requested_by_id == requested_by_id
                && refund.requested_amount_minor == requested_amount_minor
        })
        .max_by(|left, right| {
            left.created_at_ms
                .cmp(&right.created_at_ms)
                .then_with(|| left.refund_order_id.cmp(&right.refund_order_id))
        })
        .cloned()
}

fn derive_payment_refund_status(
    refundable_ceiling_minor: u64,
    refunds: &[RefundOrderRecord],
) -> PaymentRefundStatus {
    let refunded_amount_minor = refunds
        .iter()
        .map(|refund| refund.refunded_amount_minor)
        .fold(0_u64, u64::saturating_add);

    if refunded_amount_minor == 0 {
        if refunds.iter().any(|refund| {
            matches!(
                refund.refund_status,
                RefundOrderStatus::Requested
                    | RefundOrderStatus::AwaitingApproval
                    | RefundOrderStatus::Approved
                    | RefundOrderStatus::Processing
            )
        }) {
            PaymentRefundStatus::Pending
        } else {
            PaymentRefundStatus::NotRequested
        }
    } else if refunded_amount_minor >= refundable_ceiling_minor {
        PaymentRefundStatus::Refunded
    } else {
        PaymentRefundStatus::PartiallyRefunded
    }
}

fn normalized_outcome_from_callback_event(
    callback_event: &PaymentCallbackEventRecord,
) -> Option<PaymentCallbackNormalizedOutcome> {
    match callback_event.processing_status {
        PaymentCallbackProcessingStatus::Processed => {
            let summary = format!(
                "{} {}",
                callback_event.event_type.to_ascii_lowercase(),
                callback_event
                    .payload_json
                    .as_deref()
                    .unwrap_or_default()
                    .to_ascii_lowercase()
            );
            if contains_any_token(&summary, &["canceled", "cancelled", "voided", "closed"]) {
                Some(PaymentCallbackNormalizedOutcome::Canceled)
            } else if contains_any_token(&summary, &["expired", "timeout"]) {
                Some(PaymentCallbackNormalizedOutcome::Expired)
            } else if contains_any_token(&summary, &["failed", "declined", "rejected", "error"]) {
                Some(PaymentCallbackNormalizedOutcome::Failed)
            } else if contains_any_token(
                &summary,
                &[
                    "authorized",
                    "authorised",
                    "requires_capture",
                    "awaiting_capture",
                    "capturable",
                ],
            ) {
                Some(PaymentCallbackNormalizedOutcome::Authorized)
            } else {
                Some(PaymentCallbackNormalizedOutcome::Settled)
            }
        }
        _ => None,
    }
}

fn is_zero_payment_checkout(order: &CommerceOrderRecord) -> bool {
    order.payable_price_cents == 0 || order.target_kind == "coupon_redemption"
}

fn guidance_for_order(order: &CommerceOrderRecord) -> String {
    match (order.target_kind.as_str(), order.status.as_str()) {
        ("subscription_plan", "pending_payment") => {
            "Canonical payment checkout is prepared. Settle the payment to activate the workspace membership and included monthly units.".to_owned()
        }
        ("recharge_pack", "pending_payment") => {
            "Canonical payment checkout is prepared. Settle the payment to apply the recharge pack and restore workspace quota headroom.".to_owned()
        }
        ("custom_recharge", "pending_payment") => {
            "Canonical payment checkout is prepared. Settle the payment to apply the custom recharge amount and restore workspace quota headroom.".to_owned()
        }
        (_, "fulfilled") if is_zero_payment_checkout(order) => {
            "This order required no external payment and was fulfilled immediately at redemption time.".to_owned()
        }
        (_, "fulfilled") => {
            "This checkout session is closed because the order has already been settled.".to_owned()
        }
        (_, "canceled") => {
            "This checkout session is closed because the order was canceled before settlement.".to_owned()
        }
        (_, "failed") => {
            "This checkout session is closed because the payment flow failed.".to_owned()
        }
        _ => {
            "This checkout session describes how the current order can move through the canonical payment rail.".to_owned()
        }
    }
}

fn build_open_checkout_methods(order: &CommerceOrderRecord) -> Vec<CommerceCheckoutMethod> {
    let mut methods = Vec::new();

    if order.payable_price_cents > 0 {
        methods.push(CommerceCheckoutMethod {
            id: "provider_handoff".to_owned(),
            label: "Provider handoff".to_owned(),
            detail: "Canonical payment order and hosted checkout session are prepared. Wire Stripe, Alipay, WeChat Pay, or other gateways to this handoff.".to_owned(),
            action: "provider_handoff".to_owned(),
            availability: "planned".to_owned(),
        });
    }

    methods.push(CommerceCheckoutMethod {
        id: "cancel_order".to_owned(),
        label: "Cancel checkout".to_owned(),
        detail: "Close the pending order without applying quota or membership side effects."
            .to_owned(),
        action: "cancel_order".to_owned(),
        availability: "available".to_owned(),
    });

    methods
}

fn payment_order_status_for(order: &CommerceOrderRecord) -> PaymentOrderStatus {
    match order.status.as_str() {
        "pending_payment" => PaymentOrderStatus::AwaitingCustomer,
        "fulfilled" => PaymentOrderStatus::Captured,
        "canceled" => PaymentOrderStatus::Canceled,
        "failed" => PaymentOrderStatus::Failed,
        _ => PaymentOrderStatus::Created,
    }
}

fn payment_attempt_status_for_payment_order(
    payment_order: &PaymentOrderRecord,
) -> PaymentAttemptStatus {
    match payment_order.payment_status {
        PaymentOrderStatus::Created => PaymentAttemptStatus::Initiated,
        PaymentOrderStatus::AwaitingCustomer => PaymentAttemptStatus::HandoffReady,
        PaymentOrderStatus::Processing => PaymentAttemptStatus::Processing,
        PaymentOrderStatus::Authorized => PaymentAttemptStatus::Authorized,
        PaymentOrderStatus::PartiallyCaptured => PaymentAttemptStatus::Succeeded,
        PaymentOrderStatus::Captured => PaymentAttemptStatus::Succeeded,
        PaymentOrderStatus::Failed => PaymentAttemptStatus::Failed,
        PaymentOrderStatus::Expired => PaymentAttemptStatus::Expired,
        PaymentOrderStatus::Canceled => PaymentAttemptStatus::Canceled,
    }
}

fn payment_session_status_for_payment_order(
    payment_attempt: &PaymentAttemptRecord,
    order: &CommerceOrderRecord,
) -> PaymentSessionStatus {
    match payment_attempt.attempt_status {
        PaymentAttemptStatus::Authorized => PaymentSessionStatus::Authorized,
        PaymentAttemptStatus::Succeeded => PaymentSessionStatus::Settled,
        PaymentAttemptStatus::Failed => PaymentSessionStatus::Failed,
        PaymentAttemptStatus::Expired => PaymentSessionStatus::Expired,
        PaymentAttemptStatus::Canceled => PaymentSessionStatus::Canceled,
        PaymentAttemptStatus::Initiated
        | PaymentAttemptStatus::HandoffReady
        | PaymentAttemptStatus::Processing => {
            if order.status.as_str() == "fulfilled" {
                PaymentSessionStatus::Settled
            } else if order.status.as_str() == "failed" {
                PaymentSessionStatus::Failed
            } else if order.status.as_str() == "canceled" {
                PaymentSessionStatus::Canceled
            } else {
                PaymentSessionStatus::Open
            }
        }
    }
}

fn fulfillment_status_for(order: &CommerceOrderRecord) -> &'static str {
    match order.status.as_str() {
        "fulfilled" => "fulfilled",
        "canceled" => "canceled",
        "failed" => "failed",
        _ => "pending",
    }
}

fn reconcile_checkout_payment_order(
    existing: &PaymentOrderRecord,
    mut desired: PaymentOrderRecord,
) -> PaymentOrderRecord {
    desired.payment_status =
        reconcile_payment_order_status(existing.payment_status, desired.payment_status);
    desired.captured_amount_minor =
        effective_captured_amount_minor(existing).max(desired.captured_amount_minor);
    desired.fulfillment_status =
        reconcile_fulfillment_status(&existing.fulfillment_status, &desired.fulfillment_status);
    desired.refund_status = existing.refund_status;
    if !matches!(existing.provider_code, PaymentProviderCode::Unspecified) {
        desired.provider_code = existing.provider_code;
    }
    if existing.method_code.is_some() {
        desired.method_code = existing.method_code.clone();
    }
    if existing.quote_snapshot_json.is_some() {
        desired.quote_snapshot_json = existing.quote_snapshot_json.clone();
    }
    if existing.metadata_json.is_some() {
        desired.metadata_json = existing.metadata_json.clone();
    }
    desired.version = existing.version.max(desired.version);
    desired.created_at_ms = existing.created_at_ms.min(desired.created_at_ms);
    desired.updated_at_ms = existing.updated_at_ms.max(desired.updated_at_ms);
    desired
}

fn reconcile_checkout_payment_attempt(
    existing: &PaymentAttemptRecord,
    mut desired: PaymentAttemptRecord,
) -> PaymentAttemptRecord {
    desired.payment_attempt_id = existing.payment_attempt_id.clone();
    desired.payment_order_id = existing.payment_order_id.clone();
    desired.attempt_status =
        reconcile_payment_attempt_status(existing.attempt_status, desired.attempt_status);
    if existing.gateway_account_id != "gateway_selection_pending" {
        desired.gateway_account_id = existing.gateway_account_id.clone();
    }
    if !matches!(existing.provider_code, PaymentProviderCode::Unspecified) {
        desired.provider_code = existing.provider_code;
    }
    if !existing.method_code.trim().is_empty() {
        desired.method_code = existing.method_code.clone();
    }
    if !existing.client_kind.trim().is_empty() {
        desired.client_kind = existing.client_kind.clone();
    }
    if !existing.idempotency_key.trim().is_empty() {
        desired.idempotency_key = existing.idempotency_key.clone();
    }
    if existing.provider_request_id.is_some() {
        desired.provider_request_id = existing.provider_request_id.clone();
    }
    if existing.provider_payment_reference.is_some() {
        desired.provider_payment_reference = existing.provider_payment_reference.clone();
    }
    if !existing.request_payload_hash.trim().is_empty() {
        desired.request_payload_hash = existing.request_payload_hash.clone();
    }
    if existing.expires_at_ms.is_some() {
        desired.expires_at_ms = existing.expires_at_ms;
    }
    desired.created_at_ms = existing.created_at_ms.min(desired.created_at_ms);
    desired.updated_at_ms = existing.updated_at_ms.max(desired.updated_at_ms);
    desired
}

fn reconcile_checkout_payment_session(
    existing: &PaymentSessionRecord,
    mut desired: PaymentSessionRecord,
) -> PaymentSessionRecord {
    desired.payment_session_id = existing.payment_session_id.clone();
    desired.payment_attempt_id = existing.payment_attempt_id.clone();
    desired.session_kind = existing.session_kind;
    desired.session_status =
        reconcile_payment_session_status(existing.session_status, desired.session_status);
    if existing.display_reference.is_some() {
        desired.display_reference = existing.display_reference.clone();
    }
    if existing.qr_payload.is_some() {
        desired.qr_payload = existing.qr_payload.clone();
    }
    if existing.redirect_url.is_some() {
        desired.redirect_url = existing.redirect_url.clone();
    }
    desired.expires_at_ms = existing.expires_at_ms.max(desired.expires_at_ms);
    desired.created_at_ms = existing.created_at_ms.min(desired.created_at_ms);
    desired.updated_at_ms = existing.updated_at_ms.max(desired.updated_at_ms);
    desired
}

fn reconcile_payment_order_status(
    existing: PaymentOrderStatus,
    desired: PaymentOrderStatus,
) -> PaymentOrderStatus {
    reconcile_ranked_status(
        existing,
        desired,
        payment_order_status_rank(existing),
        payment_order_status_rank(desired),
    )
}

fn reconcile_payment_attempt_status(
    existing: PaymentAttemptStatus,
    desired: PaymentAttemptStatus,
) -> PaymentAttemptStatus {
    reconcile_ranked_status(
        existing,
        desired,
        payment_attempt_status_rank(existing),
        payment_attempt_status_rank(desired),
    )
}

fn reconcile_payment_session_status(
    existing: PaymentSessionStatus,
    desired: PaymentSessionStatus,
) -> PaymentSessionStatus {
    reconcile_ranked_status(
        existing,
        desired,
        payment_session_status_rank(existing),
        payment_session_status_rank(desired),
    )
}

fn reconcile_ranked_status<T>(existing: T, desired: T, existing_rank: u8, desired_rank: u8) -> T
where
    T: Copy + Eq,
{
    if desired_rank < existing_rank || (desired_rank == existing_rank && existing != desired) {
        existing
    } else {
        desired
    }
}

fn payment_order_status_rank(status: PaymentOrderStatus) -> u8 {
    match status {
        PaymentOrderStatus::Created => 0,
        PaymentOrderStatus::AwaitingCustomer => 1,
        PaymentOrderStatus::Processing => 2,
        PaymentOrderStatus::Authorized => 3,
        PaymentOrderStatus::PartiallyCaptured => 4,
        PaymentOrderStatus::Captured => 5,
        PaymentOrderStatus::Failed | PaymentOrderStatus::Expired | PaymentOrderStatus::Canceled => {
            4
        }
    }
}

fn payment_attempt_status_rank(status: PaymentAttemptStatus) -> u8 {
    match status {
        PaymentAttemptStatus::Initiated => 0,
        PaymentAttemptStatus::HandoffReady => 1,
        PaymentAttemptStatus::Processing => 2,
        PaymentAttemptStatus::Authorized => 3,
        PaymentAttemptStatus::Succeeded
        | PaymentAttemptStatus::Failed
        | PaymentAttemptStatus::Expired
        | PaymentAttemptStatus::Canceled => 4,
    }
}

fn payment_session_status_rank(status: PaymentSessionStatus) -> u8 {
    match status {
        PaymentSessionStatus::Open => 0,
        PaymentSessionStatus::Authorized => 1,
        PaymentSessionStatus::Settled
        | PaymentSessionStatus::Expired
        | PaymentSessionStatus::Failed
        | PaymentSessionStatus::Canceled => 2,
    }
}

fn reconcile_fulfillment_status(existing: &str, desired: &str) -> String {
    let existing_rank = fulfillment_status_rank(existing);
    let desired_rank = fulfillment_status_rank(desired);
    if desired_rank < existing_rank || (desired_rank == existing_rank && existing != desired) {
        existing.to_owned()
    } else {
        desired.to_owned()
    }
}

fn fulfillment_status_rank(status: &str) -> u8 {
    match status.trim() {
        "pending" => 0,
        "authorized_pending_capture" => 1,
        "partial_capture_pending_review" => 2,
        "captured_pending_fulfillment" => 3,
        "fulfilled" | "failed" | "canceled" | "expired" => 4,
        _ => 0,
    }
}

fn effective_captured_amount_minor(payment_order: &PaymentOrderRecord) -> u64 {
    if payment_order.captured_amount_minor > 0 {
        payment_order.captured_amount_minor
    } else if matches!(payment_order.payment_status, PaymentOrderStatus::Captured) {
        payment_order.payable_minor
    } else {
        0
    }
}

fn payment_order_refund_ceiling_minor(payment_order: &PaymentOrderRecord) -> u64 {
    match payment_order.payment_status {
        PaymentOrderStatus::PartiallyCaptured | PaymentOrderStatus::Captured => {
            effective_captured_amount_minor(payment_order)
        }
        _ => 0,
    }
}

fn payment_order_remaining_refundable_amount_minor(
    payment_order: &PaymentOrderRecord,
    refunds: &[RefundOrderRecord],
) -> u64 {
    if payment_order.payment_status.supports_refund() {
        payment_order_refund_ceiling_minor(payment_order)
            .saturating_sub(reserved_refund_amount_minor(refunds))
    } else {
        0
    }
}

fn should_finalize_settled_payment_fulfillment(
    payment_order: &PaymentOrderRecord,
    outcome: PaymentCallbackNormalizedOutcome,
) -> bool {
    matches!(outcome, PaymentCallbackNormalizedOutcome::Settled)
        && matches!(payment_order.payment_status, PaymentOrderStatus::Captured)
        && payment_order.fulfillment_status != "fulfilled"
}

fn payment_order_status_for_settled_capture(
    payment_order: &PaymentOrderRecord,
    captured_amount_minor: u64,
) -> PaymentOrderStatus {
    if captured_amount_minor < payment_order.payable_minor {
        PaymentOrderStatus::PartiallyCaptured
    } else {
        PaymentOrderStatus::Captured
    }
}

fn fulfillment_status_for_settled_capture(
    payment_order: &PaymentOrderRecord,
    captured_amount_minor: u64,
) -> String {
    if captured_amount_minor < payment_order.payable_minor {
        "partial_capture_pending_review".to_owned()
    } else {
        "captured_pending_fulfillment".to_owned()
    }
}

fn callback_provider_transaction_id(
    callback_event: &PaymentCallbackEventRecord,
    request: &PaymentCallbackIntakeRequest,
) -> String {
    request
        .provider_transaction_id
        .clone()
        .unwrap_or_else(|| callback_event.event_identity.clone())
}

fn sale_transactions(transactions: &[PaymentTransactionRecord]) -> Vec<PaymentTransactionRecord> {
    transactions
        .iter()
        .filter(|transaction| transaction.transaction_kind == PaymentTransactionKind::Sale)
        .cloned()
        .collect()
}

fn find_sale_transaction_by_provider_transaction_id(
    transactions: &[PaymentTransactionRecord],
    provider_transaction_id: &str,
) -> Option<PaymentTransactionRecord> {
    let exact_matches = transactions
        .iter()
        .filter(|transaction| transaction.provider_transaction_id == provider_transaction_id)
        .cloned()
        .collect::<Vec<_>>();
    select_primary_payment_transaction_by_kind(&exact_matches, PaymentTransactionKind::Sale)
}

fn aggregate_captured_amount_minor(transactions: &[PaymentTransactionRecord]) -> u64 {
    transactions
        .iter()
        .filter(|transaction| transaction.transaction_kind == PaymentTransactionKind::Sale)
        .fold(0_u64, |captured_amount_minor, transaction| {
            captured_amount_minor.saturating_add(transaction.amount_minor)
        })
}

fn aggregate_captured_amount_minor_with_upsert(
    transactions: &[PaymentTransactionRecord],
    payment_transaction: &PaymentTransactionRecord,
) -> u64 {
    let replaced_amount_minor = transactions
        .iter()
        .find(|transaction| {
            transaction.payment_transaction_id == payment_transaction.payment_transaction_id
        })
        .map(|transaction| transaction.amount_minor)
        .unwrap_or(0);
    aggregate_captured_amount_minor(transactions)
        .saturating_sub(replaced_amount_minor)
        .saturating_add(payment_transaction.amount_minor)
}

fn accepted_sale_capture_amount_minor(
    payment_order: &PaymentOrderRecord,
    transactions: &[PaymentTransactionRecord],
    existing_transaction_opt: Option<&PaymentTransactionRecord>,
    provider_amount_minor: u64,
) -> u64 {
    let accepted_other_amount_minor = aggregate_captured_amount_minor(transactions).saturating_sub(
        existing_transaction_opt
            .map(|transaction| transaction.amount_minor)
            .unwrap_or(0),
    );
    provider_amount_minor.min(
        payment_order
            .payable_minor
            .saturating_sub(accepted_other_amount_minor),
    )
}

fn reconcile_sale_payment_transaction_record(
    existing: &PaymentTransactionRecord,
    payment_order: &PaymentOrderRecord,
    payment_attempt_opt: Option<&PaymentAttemptRecord>,
    callback_event: &PaymentCallbackEventRecord,
    request: &PaymentCallbackIntakeRequest,
    amount_minor: u64,
) -> PaymentTransactionRecord {
    let built = build_payment_transaction_record(
        payment_order,
        payment_attempt_opt,
        callback_event,
        request,
        amount_minor,
    );
    let mut reconciled = existing.clone();
    reconciled.tenant_id = built.tenant_id;
    reconciled.organization_id = built.organization_id;
    reconciled.payment_order_id = built.payment_order_id;
    reconciled.payment_attempt_id = built
        .payment_attempt_id
        .or_else(|| existing.payment_attempt_id.clone());
    reconciled.transaction_kind = built.transaction_kind;
    reconciled.provider_code = built.provider_code;
    reconciled.currency_code = built.currency_code;
    reconciled.amount_minor = existing.amount_minor.max(built.amount_minor);
    reconciled.fee_minor = built.fee_minor.or(existing.fee_minor);
    reconciled.net_amount_minor = built.net_amount_minor.or(existing.net_amount_minor);
    reconciled.provider_status = built.provider_status;
    reconciled.raw_event_id = built.raw_event_id.or_else(|| existing.raw_event_id.clone());
    reconciled.occurred_at_ms = existing.occurred_at_ms.max(built.occurred_at_ms);
    reconciled.created_at_ms = existing.created_at_ms.min(built.created_at_ms);
    reconciled
}

async fn upsert_portal_identity_user<S>(
    store: &S,
    portal_user: &PortalUserRecord,
    tenant_id: u64,
    organization_id: u64,
    user_id: u64,
    observed_at_ms: u64,
) -> Result<IdentityUserRecord>
where
    S: IdentityKernelStore + ?Sized,
{
    let identity_user = IdentityUserRecord::new(user_id, tenant_id, organization_id)
        .with_external_user_ref(Some(portal_user.id.clone()))
        .with_username(Some(portal_user.id.clone()))
        .with_display_name(Some(portal_user.display_name.clone()))
        .with_email(Some(portal_user.email.clone()))
        .with_created_at_ms(portal_user.created_at_ms)
        .with_updated_at_ms(observed_at_ms);
    store.insert_identity_user_record(&identity_user).await
}

fn stable_subject_numeric_id(namespace: &str, value: &str) -> u64 {
    let mut hasher = Sha256::new();
    hasher.update(namespace.as_bytes());
    hasher.update(b":");
    hasher.update(value.trim().as_bytes());
    let digest = hasher.finalize();
    let mut bytes = [0_u8; 8];
    bytes.copy_from_slice(&digest[..8]);
    (u64::from_be_bytes(bytes) & (i64::MAX as u64)).max(1)
}

fn infer_currency_code(order: &CommerceOrderRecord) -> &'static str {
    for label in [&order.payable_price_label, &order.list_price_label] {
        if label.contains('¥') || label.contains('￥') {
            return "CNY";
        }
    }
    "USD"
}

fn payment_order_id(order_id: &str) -> String {
    format!(
        "payment_order_{}",
        normalize_identifier_component(order_id, false)
    )
}

fn payment_attempt_id(order_id: &str) -> String {
    format!(
        "payment_attempt_{}_1",
        normalize_identifier_component(order_id, false)
    )
}

fn failover_payment_attempt_id(payment_order_id: &str, attempt_no: u32) -> String {
    format!(
        "payment_attempt_{}_{}",
        normalize_identifier_component(payment_order_id, false),
        attempt_no
    )
}

fn payment_session_id(order_id: &str) -> String {
    format!(
        "payment_session_{}_1",
        normalize_identifier_component(order_id, false)
    )
}

fn failover_payment_session_id(payment_order_id: &str, attempt_no: u32) -> String {
    format!(
        "payment_session_{}_{}",
        normalize_identifier_component(payment_order_id, false),
        attempt_no
    )
}

fn payment_callback_event_id(request: &PaymentCallbackIntakeRequest) -> String {
    format!(
        "payment_callback_{}_{}_{}_{}",
        request.provider_code.as_str(),
        normalize_identifier_component(&request.gateway_account_id, false),
        normalize_identifier_component(&request.event_identity, false),
        request.received_at_ms
    )
}

fn refund_order_id(payment_order_id: &str, requested_at_ms: u64) -> String {
    format!(
        "refund_order_{}_{}",
        normalize_identifier_component(payment_order_id, false),
        requested_at_ms
    )
}

fn authorization_payment_transaction_id(payment_order_id: &str) -> String {
    format!(
        "payment_transaction_authorization_{}",
        normalize_identifier_component(payment_order_id, false)
    )
}

fn sale_payment_transaction_id(payment_order_id: &str, provider_transaction_id: &str) -> String {
    format!(
        "payment_transaction_sale_{}_{}",
        normalize_identifier_component(payment_order_id, false),
        normalize_identifier_component(provider_transaction_id, false)
    )
}

fn refund_payment_transaction_id(refund_order_id: &str) -> String {
    format!(
        "payment_transaction_refund_{}",
        normalize_identifier_component(refund_order_id, false)
    )
}

fn refund_conflict_reconciliation_batch_id(refund_order_id: &str) -> String {
    format!("refund_conflict_batch_{refund_order_id}")
}

fn payment_conflict_reconciliation_batch_id(payment_order_id: &str) -> String {
    format!("payment_conflict_batch_{payment_order_id}")
}

fn payment_conflict_reconciliation_line_id(
    payment_order_id: &str,
    provider_transaction_id: &str,
) -> String {
    format!(
        "payment_conflict_line_{}_{}",
        normalize_identifier_component(payment_order_id, false),
        normalize_identifier_component(provider_transaction_id, false)
    )
}

fn payment_amount_capped_reconciliation_line_id(
    payment_order_id: &str,
    provider_transaction_id: &str,
) -> String {
    format!(
        "payment_amount_line_{}_{}",
        normalize_identifier_component(payment_order_id, false),
        normalize_identifier_component(provider_transaction_id, false)
    )
}

fn refund_conflict_reconciliation_line_id(
    refund_order_id: &str,
    provider_refund_id: &str,
) -> String {
    format!(
        "refund_conflict_line_{}_{}",
        normalize_identifier_component(refund_order_id, false),
        normalize_identifier_component(provider_refund_id, false)
    )
}

fn finance_journal_entry_id(refund_order_id: &str) -> String {
    format!(
        "finance_journal_{}",
        normalize_identifier_component(refund_order_id, false)
    )
}

fn finance_journal_line_id(finance_journal_entry_id: &str, line_no: u32) -> String {
    format!(
        "finance_journal_line_{}_{}",
        normalize_identifier_component(finance_journal_entry_id, false),
        line_no
    )
}

fn normalize_identifier_component(value: &str, uppercase: bool) -> String {
    let mut normalized = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            normalized.push(if uppercase {
                ch.to_ascii_uppercase()
            } else {
                ch.to_ascii_lowercase()
            });
        } else {
            normalized.push('_');
        }
    }

    normalized.trim_matches('_').to_owned()
}

fn normalize_payment_reference(order_id: &str) -> String {
    let normalized = normalize_identifier_component(order_id, true);
    normalized.replace('_', "-")
}
