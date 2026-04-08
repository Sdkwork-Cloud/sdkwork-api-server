import { useEffect, useState, type FormEvent } from 'react';
import {
  getCommerceOrderAudit,
  listCommercePaymentMethods,
} from 'sdkwork-router-admin-admin-api';
import {
  commercialPricingChargeUnitLabel,
  commercialPricingDisplayUnit,
  commercialPricingMethodLabel,
  countCurrentlyEffectiveCommercialPricingPlans,
  selectPrimaryCommercialPricingPlan,
  selectPrimaryCommercialPricingRate,
  useAdminWorkbench,
  useAdminI18n,
} from 'sdkwork-router-admin-core';
import type {
  AdminPageProps,
  CommerceOrderAuditRecord,
  PaymentMethodRecord,
} from 'sdkwork-router-admin-types';
import { CommercialOrderAuditDrawer } from './commercialOrderAuditDrawer';
import {
  buildOrderPaymentAuditColumns,
  buildOrderRefundAuditColumns,
  buildRefundTimelineColumns,
  buildSettlementLedgerColumns,
  CommercialDashboardMain,
} from './commercialOverviewSections';
import type { CommercialFact, CommercialSummaryMetric } from './formatters';
import {
  buildCommercialLedgerTimelineRows,
  buildCommercialRefundTimelineRows,
} from './ledgerTimeline';
import {
  hasCommercialOrderAuditLookupValue,
  normalizeCommercialOrderAuditLookupValue,
} from './orderAuditLookup';
import {
  buildCommercialOrderPaymentAuditRows,
  buildCommercialRefundAuditRows,
} from './orderPaymentAudit';
import { PaymentMethodManagerSection } from './paymentMethodManagerSection';
import { PaymentOrderOperationsSection } from './paymentOrderOperationsSection';
import { PaymentReconciliationSection } from './paymentReconciliationSection';
import { PaymentWebhookInboxSection } from './paymentWebhookInboxSection';

export function CommercialPage({ snapshot }: AdminPageProps) {
  const { formatCurrency, formatNumber, t } = useAdminI18n();
  const { refreshWorkspace } = useAdminWorkbench();

  const activeAccounts = snapshot.commercialAccounts.filter(
    (record) => record.account.status === 'active',
  ).length;
  const suspendedAccounts = snapshot.commercialAccounts.filter(
    (record) => record.account.status === 'suspended',
  ).length;
  const availableBalance = snapshot.commercialAccounts.reduce(
    (sum, record) => sum + record.available_balance,
    0,
  );
  const heldBalance = snapshot.commercialAccounts.reduce(
    (sum, record) => sum + record.held_balance,
    0,
  );
  const openHolds = snapshot.commercialAccountHolds.filter(
    (hold) =>
      hold.status === 'held'
      || hold.status === 'captured'
      || hold.status === 'partially_released',
  );
  const latestSettlements = [...snapshot.commercialRequestSettlements]
    .sort((left, right) => right.settled_at_ms - left.settled_at_ms)
    .slice(0, 5);
  const activePricingPlans = countCurrentlyEffectiveCommercialPricingPlans(
    snapshot.commercialPricingPlans,
  );
  const pricedMetrics = new Set(
    snapshot.commercialPricingRates.map((rate) => rate.metric_code),
  );
  const primaryPricingPlan = selectPrimaryCommercialPricingPlan(
    snapshot.commercialPricingPlans,
  );
  const primaryPricingRate = selectPrimaryCommercialPricingRate(
    snapshot.commercialPricingRates,
    primaryPricingPlan,
  );
  const commercialLedgerTimeline = buildCommercialLedgerTimelineRows(
    snapshot.commercialAccountLedger,
    snapshot.commercialRequestSettlements,
  );
  const recentLedgerTimeline = commercialLedgerTimeline.slice(0, 8);
  const refundTimelineRows = buildCommercialRefundTimelineRows(
    commercialLedgerTimeline,
  ).slice(0, 6);
  const commercialOrderPaymentAuditRows = buildCommercialOrderPaymentAuditRows(
    snapshot.commerceOrders,
    snapshot.commercePaymentEvents,
  );
  const recentOrderPaymentAuditRows = commercialOrderPaymentAuditRows.slice(0, 8);
  const refundAuditRows = buildCommercialRefundAuditRows(
    commercialOrderPaymentAuditRows,
  ).slice(0, 6);
  const rejectedPaymentEvents = snapshot.commercePaymentEvents.filter((event) =>
    event.processing_status === 'rejected'
    || event.processing_status === 'failed',
  ).length;
  const commercialPageCopyContract = {
    account: {
      moduleDescription: t('Commercial accounts, settlement explorer, and pricing governance now live as a first-class admin module.'),
      panelDescription: t('Account posture keeps status, held balance, and admission readiness visible in one surface.'),
    },
    surface: {
      title: t('Commercial control plane'),
      overview: t('Operators can audit commercial accounts, request settlement posture, and pricing governance without leaving a dedicated module.'),
      lookupLabel: t('Find order audit'),
      lookupHint: t(
        'Order audit detail opens the full order, payment, and coupon evidence stream for the selected order.',
      ),
      settlementExplorerPanel: t('Settlement explorer highlights open holds, captured requests, and correction posture from canonical settlement records.'),
      settlementLedgerTitle: t('Settlement ledger'),
      settlementLedgerDescription: t('Settlement ledger keeps capture and refund entries linked to request settlements so operators can audit credits, retail charge, and final correction posture without leaving the commercial module.'),
      settlementLedgerEmptyDescription: t('Settlement ledger entries will appear here once commercial account history begins landing for the selected control-plane slice.'),
      settlementLedgerEmptyTitle: t('No settlement ledger entries yet'),
      refundTimelineTitle: t('Refund timeline'),
      refundTimelineDescription: t('Refund timeline isolates correction entries so support and finance can verify credited quantity, linked request, and refund cost posture at a glance.'),
      refundTimelineEmptyDescription: t('Refund activity will appear here once commercial refunds are posted into the account ledger history.'),
      refundTimelineEmptyTitle: t('No refunds recorded yet'),
      orderPaymentAuditLongDescription: t('Order payment audit keeps recent commercial orders linked to payment callbacks, provider evidence, and operator-visible processing posture without loading unbounded order history into the commercial module.'),
      orderPaymentAuditEmptyDescription: t('Recent commerce orders will appear here once checkout, webhook, and settlement evidence starts landing in the commercial audit stream.'),
      orderPaymentAuditEmptyTitle: t('No order payment evidence yet'),
      orderRefundAuditTitle: t('Order refund audit'),
      orderRefundAuditDescription: t('Order refund audit keeps explicit refund callbacks and refunded-order fallback evidence visible so operators can spot missing callback closure before it becomes a reconciliation blind spot.'),
      orderRefundAuditEmptyDescription: t('Refund audit rows will appear here once commercial orders begin entering explicit refund or refunded-order-state correction flows.'),
      orderRefundAuditEmptyTitle: t('No refund evidence yet'),
      latestSettlementsTitle: t('Latest settlements'),
      latestSettlementsRailDescription: t('The right rail keeps the most recent commercial settlement evidence in view for rapid operator triage.'),
      latestSettlementsEmptyTitle: t('No settlement evidence yet'),
      latestSettlementsEmptyDescription: t('Latest settlements will appear here once request settlement records start landing from the canonical commercial kernel.'),
      pricingGovernancePanel: t('Pricing governance keeps commercial plan activation and metric-rate coverage visible for operator review.'),
    },
    audit: {
      detailTitle: t('Order audit detail'),
      loadingSelectedOrder: t('Loading selected order'),
      loadingEvidenceTitle: t('Loading order audit evidence'),
      loadingEvidenceDescription: t('Payment, coupon, and campaign evidence is being loaded for the selected order.'),
      detailUnavailable: t('Order audit detail unavailable'),
      detailDescription: t('Commercial order, checkout, and coupon evidence stay bundled here so operators can reconstruct fulfillment and refund posture without switching modules.'),
      footerDescription: t('Order audit detail keeps payment callbacks and coupon lifecycle evidence scoped to the selected order so reconciliation triage stays deterministic.'),
      paymentTimelineTitle: t('Payment evidence timeline'),
      paymentTimelineDescription: t('Provider callbacks remain ordered here so operators can verify settlement, rejection, and refund sequencing for the selected order.'),
      noPaymentEvidenceRecorded: t('No payment evidence has been recorded for this order yet.'),
      couponEvidenceTitle: t('Coupon evidence chain'),
      couponEvidenceDescription: t('Reservation, redemption, rollback, code, template, and campaign evidence stays attached so discount posture can be audited together with payment callbacks.'),
      couponRollbackTitle: t('Coupon rollback timeline'),
      couponRollbackDescription: t('Rollback evidence confirms whether coupon subsidy and inventory were restored during refund handling.'),
      noCouponRollbackEvidence: t('No coupon rollback evidence has been recorded for this order.'),
      noCouponApplied: t('No coupon applied'),
      reservation: t('Reservation'),
      redemption: t('Redemption'),
      rollbackCount: t('Rollback count'),
      noReservationEvidence: t('No reservation evidence'),
      noRedemptionEvidence: t('No redemption evidence'),
      noCouponCodeEvidence: t('No coupon code evidence'),
      couponTemplate: t('Coupon template'),
      noTemplateEvidence: t('No template evidence'),
      marketingCampaign: t('Marketing campaign'),
      noCampaignEvidence: t('No campaign evidence'),
      restoredBudget: t('Restored budget'),
      restoredInventory: t('Restored inventory'),
      targetKind: t('Target kind'),
      listPrice: t('List price'),
      payablePrice: t('Payable price'),
      grantedUnits: t('Granted units'),
      bonusUnits: t('Bonus units'),
      orderStatusAfter: t('Order status after'),
      paymentEventId: t('Payment event id'),
      dedupeKey: t('Dedupe key'),
      loading: t('Loading'),
      noPaymentEvidence: t('No payment evidence'),
    },
    detail: {
      account: t('Account'),
      accountId: (id: number | string) => t('Account #{id}', { id }),
      request: t('Request'),
      requestId: (id: number | string) => t('Request #{id}', { id }),
      holdId: (id: number | string) => t('Hold #{id}', { id }),
      order: t('Order'),
      orderId: (id: number | string) => t('Order #{id}', { id }),
      investigation: t('Investigation'),
      viewOrderAudit: t('View order audit'),
      entry: t('Entry'),
      credits: t('Credits'),
      settlement: t('Settlement'),
      retailCharge: t('Retail charge'),
      refundCredits: t('Refund credits'),
      providerCost: t('Provider cost'),
      event: t('Event'),
      processing: t('Processing'),
      refundState: t('Refund state'),
      noLinkedRequest: t('No linked request'),
      noLinkedHold: t('No linked hold'),
      noProviderEventId: t('No provider event id'),
      noDerivedOrderStatus: t('No derived order status'),
      pendingEvidence: t('Pending evidence'),
      unlinked: t('Unlinked'),
      na: t('n/a'),
      retailChargeAmount: (amount: string) => t('Retail charge: {amount}', { amount }),
      providerCostAmount: (amount: string) => t('Provider cost: {amount}', { amount }),
      capturedCreditsCount: (count: string) => t('Captured credits: {count}', { count }),
    },
  };
  const orderAuditLookupLabel = commercialPageCopyContract.surface.lookupLabel;
  const orderAuditLookupHint = commercialPageCopyContract.surface.lookupHint;

  const [orderAuditLookupValue, setOrderAuditLookupValue] = useState('');
  const [orderAuditLookupError, setOrderAuditLookupError] = useState<string | null>(null);
  const [selectedOrderAuditId, setSelectedOrderAuditId] = useState<string | null>(null);
  const [selectedOrderAudit, setSelectedOrderAudit] =
    useState<CommerceOrderAuditRecord | null>(null);
  const [isOrderAuditLoading, setIsOrderAuditLoading] = useState(false);
  const [orderAuditError, setOrderAuditError] = useState<string | null>(null);
  const [paymentMethods, setPaymentMethods] = useState<PaymentMethodRecord[]>([]);
  const [paymentMethodsLoading, setPaymentMethodsLoading] = useState(false);
  const [paymentMethodsError, setPaymentMethodsError] = useState<string | null>(null);

  const selectedOrderFromSnapshot = selectedOrderAuditId
    ? snapshot.commerceOrders.find((order) => order.order_id === selectedOrderAuditId) ?? null
    : null;
  const selectedOrderRecord = selectedOrderAudit?.order ?? selectedOrderFromSnapshot ?? null;
  const selectedOrderPaymentEvents = selectedOrderAudit?.payment_events
    ?? (selectedOrderAuditId
      ? snapshot.commercePaymentEvents.filter((event) => event.order_id === selectedOrderAuditId)
      : []);
  const orderAuditOpen = selectedOrderAuditId != null;

  useEffect(() => {
    if (!selectedOrderAuditId) {
      setSelectedOrderAudit(null);
      setOrderAuditError(null);
      setIsOrderAuditLoading(false);
      return;
    }

    let cancelled = false;
    setSelectedOrderAudit(null);
    setOrderAuditError(null);
    setIsOrderAuditLoading(true);

    getCommerceOrderAudit(selectedOrderAuditId)
      .then((audit) => {
        if (!cancelled) {
          setSelectedOrderAudit(audit);
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setOrderAuditError(
            error instanceof Error ? error.message : 'Failed to load order audit detail.',
          );
        }
      })
      .finally(() => {
        if (!cancelled) {
          setIsOrderAuditLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [selectedOrderAuditId]);

  useEffect(() => {
    let cancelled = false;
    setPaymentMethodsLoading(true);
    setPaymentMethodsError(null);

    listCommercePaymentMethods()
      .then((records) => {
        if (!cancelled) {
          setPaymentMethods(records);
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setPaymentMethodsError(
            error instanceof Error
              ? error.message
              : 'Failed to load payment method inventory.',
          );
        }
      })
      .finally(() => {
        if (!cancelled) {
          setPaymentMethodsLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  function openOrderAudit(orderId: string) {
    const normalizedOrderId = normalizeCommercialOrderAuditLookupValue(orderId);
    setOrderAuditLookupValue(normalizedOrderId);
    setOrderAuditLookupError(null);
    setSelectedOrderAuditId(normalizedOrderId);
  }

  async function refreshPaymentMethods() {
    setPaymentMethodsLoading(true);
    setPaymentMethodsError(null);
    try {
      const records = await listCommercePaymentMethods();
      setPaymentMethods(records);
    } catch (error) {
      setPaymentMethodsError(
        error instanceof Error ? error.message : 'Failed to load payment method inventory.',
      );
    } finally {
      setPaymentMethodsLoading(false);
    }
  }

  function handleOrderAuditOpenChange(open: boolean) {
    if (!open) {
      setSelectedOrderAuditId(null);
    }
  }

  function handleOrderAuditLookupSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    if (!hasCommercialOrderAuditLookupValue(orderAuditLookupValue)) {
      setOrderAuditLookupError(t('Enter an order id to open order audit detail.'));
      return;
    }

    openOrderAudit(orderAuditLookupValue);
  }

  const summaryCards: CommercialSummaryMetric[] = [
    {
      label: t('Commercial accounts'),
      value: formatNumber(snapshot.commercialAccounts.length),
      description: t('Canonical payable accounts currently discoverable by the commercial control plane.'),
    },
    {
      label: t('Available balance'),
      value: formatNumber(availableBalance),
      description: t('Spendable credit still available across the commercial account inventory.'),
    },
    {
      label: t('Settlement explorer'),
      value: formatNumber(snapshot.commercialRequestSettlements.length),
      description: t('Captured, released, and refunded request settlements ready for operator investigation.'),
    },
    {
      label: t('Order payment audit'),
      value: formatNumber(snapshot.commerceOrders.length),
      description: t('Recent commerce orders stay linked to provider callbacks and operator-visible payment evidence.'),
    },
    {
      label: t('Pricing governance'),
      value: formatNumber(snapshot.commercialPricingRates.length),
      description: t('Live metric-rate rows currently shaping canonical commercial charging.'),
    },
  ];

  const accountFacts: CommercialFact[] = [
    {
      label: t('Active accounts'),
      value: formatNumber(activeAccounts),
      detail: t('Accounts currently able to receive holds and settlement capture.'),
      tone: 'success',
    },
    {
      label: t('Suspended accounts'),
      value: formatNumber(suspendedAccounts),
      detail: t('Accounts blocked from new commercial admission until operator review.'),
      tone: suspendedAccounts > 0 ? 'warning' : 'secondary',
    },
    {
      label: t('Held balance'),
      value: formatNumber(heldBalance),
      detail: t('Credit currently reserved by request admission and pending settlement flows.'),
      tone: heldBalance > 0 ? 'warning' : 'secondary',
    },
  ];

  const settlementFacts: CommercialFact[] = [
    {
      label: t('Open holds'),
      value: formatNumber(openHolds.length),
      detail: t('Commercial holds that still need capture, release, expiry, or operator intervention.'),
      tone: openHolds.length > 0 ? 'warning' : 'secondary',
    },
    {
      label: t('Captured settlements'),
      value: formatNumber(
        snapshot.commercialRequestSettlements.filter((record) => record.status === 'captured').length,
      ),
      detail: t('Settlements already converted into captured commercial liability evidence.'),
      tone: 'success',
    },
    {
      label: t('Refunded settlements'),
      value: formatNumber(
        snapshot.commercialRequestSettlements.filter((record) => record.status === 'refunded').length,
      ),
      detail: t('Refund posture keeps correction flows visible inside the settlement explorer.'),
      tone: 'secondary',
    },
    {
      label: t('Rejected callbacks'),
      value: formatNumber(rejectedPaymentEvents),
      detail: t('Rejected or failed provider callbacks stay visible before they drift into silent payment reconciliation gaps.'),
      tone: rejectedPaymentEvents > 0 ? 'warning' : 'secondary',
    },
  ];

  const pricingFacts: CommercialFact[] = [
    {
      label: t('Active pricing plans'),
      value: formatNumber(activePricingPlans),
      detail: t('Commercial pricing plans that are active and currently effective in the control plane.'),
      tone: activePricingPlans > 0 ? 'success' : 'warning',
    },
    {
      label: t('Priced metrics'),
      value: formatNumber(pricedMetrics.size),
      detail: t('Distinct metric codes already governed by canonical pricing rates.'),
      tone: 'secondary',
    },
    {
      label: t('Primary plan'),
      value: primaryPricingPlan?.display_name ?? t('No active plan'),
      detail: t('The first active pricing plan remains the quickest operator reference point.'),
      tone: primaryPricingPlan ? 'success' : 'warning',
    },
    {
      label: t('Charge unit'),
      value: commercialPricingChargeUnitLabel(primaryPricingRate?.charge_unit, t),
      detail: t('Primary metered unit keeps settlement granularity explicit for operator review.'),
      tone: primaryPricingRate ? 'success' : 'secondary',
    },
    {
      label: t('Billing method'),
      value: commercialPricingMethodLabel(primaryPricingRate?.pricing_method, t),
      detail: t('Settlement method shows whether the primary rate charges per unit, flat, or step-based.'),
      tone: primaryPricingRate ? 'success' : 'secondary',
    },
    {
      label: t('Price unit'),
      value: commercialPricingDisplayUnit(primaryPricingRate, t),
      detail: t('Display unit makes the commercial rate readable for token and multimodal pricing review.'),
      tone: primaryPricingRate ? 'success' : 'secondary',
    },
  ];

  const settlementLedgerColumns = buildSettlementLedgerColumns(
    formatNumber,
    formatCurrency,
    t,
  );
  const refundTimelineColumns = buildRefundTimelineColumns(
    formatNumber,
    formatCurrency,
    t,
  );
  const orderPaymentAuditColumns = buildOrderPaymentAuditColumns(openOrderAudit, t);
  const orderRefundAuditColumns = buildOrderRefundAuditColumns(openOrderAudit, t);

  return (
    <>
      <CommercialDashboardMain
        accountFacts={accountFacts}
        formatCurrency={formatCurrency}
        formatNumber={formatNumber}
        latestSettlements={latestSettlements}
        onOrderAuditLookupChange={(value) => {
          setOrderAuditLookupValue(value);
          if (orderAuditLookupError) {
            setOrderAuditLookupError(null);
          }
        }}
        onOrderAuditLookupSubmit={handleOrderAuditLookupSubmit}
        orderAuditLookupError={orderAuditLookupError}
        orderAuditLookupHint={orderAuditLookupHint}
        orderAuditLookupLabel={orderAuditLookupLabel}
        orderAuditLookupValue={orderAuditLookupValue}
        orderPaymentAuditColumns={orderPaymentAuditColumns}
        orderRefundAuditColumns={orderRefundAuditColumns}
        pricingFacts={pricingFacts}
        recentLedgerTimeline={recentLedgerTimeline}
        recentOrderPaymentAuditRows={recentOrderPaymentAuditRows}
        refundAuditRows={refundAuditRows}
        refundTimelineColumns={refundTimelineColumns}
        refundTimelineRows={refundTimelineRows}
        settlementFacts={settlementFacts}
        settlementLedgerColumns={settlementLedgerColumns}
        supplementarySections={(
          <div className="space-y-6">
            <PaymentMethodManagerSection
              credentials={snapshot.credentials}
              onRefresh={refreshPaymentMethods}
              paymentMethods={paymentMethods}
              paymentMethodsError={paymentMethodsError}
              paymentMethodsLoading={paymentMethodsLoading}
            />
            <PaymentOrderOperationsSection
              onOpenOrderAudit={openOrderAudit}
              onRefreshWorkspace={refreshWorkspace}
              orders={snapshot.commerceOrders}
            />
            <PaymentWebhookInboxSection />
            <PaymentReconciliationSection paymentMethods={paymentMethods} />
          </div>
        )}
        summaryCards={summaryCards}
      />
      <CommercialOrderAuditDrawer
        error={orderAuditError}
        isLoading={isOrderAuditLoading}
        onOpenChange={handleOrderAuditOpenChange}
        open={orderAuditOpen}
        selectedOrderAudit={selectedOrderAudit}
        selectedOrderPaymentEvents={selectedOrderPaymentEvents}
        selectedOrderRecord={selectedOrderRecord}
      />
    </>
  );
}
