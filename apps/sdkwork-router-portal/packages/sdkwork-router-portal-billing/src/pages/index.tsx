import { useDeferredValue, useEffect, useState } from 'react';
import type { FormEvent, ReactNode } from 'react';

import {
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  DataTable,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  EmptyState,
  FormField,
  InlineButton,
  Input,
  Pill,
  Select,
  ToolbarField,
  ToolbarInline,
  ToolbarSearchField,
  formatCurrency,
  formatDateTime,
  formatUnits,
  usePortalI18n,
} from 'sdkwork-router-portal-commons';
import { portalErrorMessage } from 'sdkwork-router-portal-portal-api';
import type {
  PortalCommerceCheckoutSession,
  PortalCommerceMembership,
  PortalCommerceOrder,
  PortalCommercePaymentEventType,
  ProjectBillingSummary,
  RechargePack,
  SubscriptionPlan,
  UsageRecord,
} from 'sdkwork-router-portal-types';

import { BillingRecommendationCard } from '../components';
import {
  cancelBillingOrder,
  createBillingOrder,
  getBillingCheckoutSession,
  loadBillingPageData,
  previewBillingCheckout,
  sendBillingPaymentEvent,
  settleBillingOrder,
} from '../repository';
import {
  isRecommendedPack,
  isRecommendedPlan,
  recommendBillingChange,
} from '../services';
import type {
  BillingCheckoutPreview,
  BillingPageData,
  PortalBillingPageProps,
} from '../types';

const emptySummary: ProjectBillingSummary = {
  project_id: '',
  entry_count: 0,
  used_units: 0,
  booked_amount: 0,
  exhausted: false,
};

type BillingSelection =
  | {
      kind: 'subscription_plan';
      target: SubscriptionPlan;
    }
  | {
      kind: 'recharge_pack';
      target: RechargePack;
    };

type OrderWorkbenchLane = 'all' | 'pending_payment' | 'failed' | 'timeline';

function orderMatchesSearch(order: PortalCommerceOrder, search: string): boolean {
  if (!search) {
    return true;
  }

  const haystack = [
    order.order_id,
    order.target_name,
    order.target_kind,
    order.status,
    order.applied_coupon_code ?? '',
    order.payable_price_label,
  ]
    .join(' ')
    .toLowerCase();

  return haystack.includes(search);
}

function selectionLabel(selection: BillingSelection | null): string | null {
  if (!selection) {
    return null;
  }

  return selection.kind === 'subscription_plan'
    ? selection.target.name
    : selection.target.label;
}

function isPendingPaymentOrder(order: PortalCommerceOrder): boolean {
  return order.status === 'pending_payment';
}

function matchesOrderLane(
  order: PortalCommerceOrder,
  lane: OrderWorkbenchLane,
): boolean {
  switch (lane) {
    case 'pending_payment':
      return isPendingPaymentOrder(order);
    case 'failed':
      return order.status === 'failed';
    case 'timeline':
      return !isPendingPaymentOrder(order);
    default:
      return true;
  }
}

function orderWorkbenchDetail(lane: OrderWorkbenchLane): string {
  switch (lane) {
    case 'pending_payment':
      return 'Pending payment queue keeps unpaid or unfulfilled orders visible until the workspace settles or cancels them.';
    case 'failed':
      return 'Failed payment isolates checkout attempts that need coupon, payment rail, or provider callback review.';
    case 'timeline':
      return 'Order timeline shows completed or closed outcomes after checkout attempts resolve.';
    default:
      return 'Switch between pending payment queue, failed payment, and order timeline without leaving the main order table.';
  }
}

function checkoutModeLabel(session: PortalCommerceCheckoutSession | null): string {
  switch (session?.mode) {
    case 'operator_settlement':
      return 'Operator settlement';
    case 'instant_fulfillment':
      return 'Instant fulfillment';
    default:
      return 'Closed checkout';
  }
}

function hasProviderHandoff(session: PortalCommerceCheckoutSession | null): boolean {
  return Boolean(session?.methods.some((method) => method.action === 'provider_handoff'));
}

function orderStatusTone(
  status: PortalCommerceOrder['status'],
): 'default' | 'accent' | 'positive' | 'warning' {
  switch (status) {
    case 'fulfilled':
      return 'positive';
    case 'failed':
      return 'warning';
    case 'pending_payment':
      return 'accent';
    default:
      return 'default';
  }
}

export function PortalBillingPage({ onNavigate }: PortalBillingPageProps) {
  const { t } = usePortalI18n();
  const [summary, setSummary] = useState<ProjectBillingSummary>(emptySummary);
  const [usageRecords, setUsageRecords] = useState<UsageRecord[]>([]);
  const [plans, setPlans] = useState<SubscriptionPlan[]>([]);
  const [packs, setPacks] = useState<RechargePack[]>([]);
  const [orders, setOrders] = useState<PortalCommerceOrder[]>([]);
  const [membership, setMembership] = useState<PortalCommerceMembership | null>(null);
  const [status, setStatus] = useState('Loading billing posture...');
  const [searchQuery, setSearchQuery] = useState('');
  const [orderLane, setOrderLane] = useState<OrderWorkbenchLane>('all');
  const [checkoutOpen, setCheckoutOpen] = useState(false);
  const [checkoutSelection, setCheckoutSelection] = useState<BillingSelection | null>(null);
  const [couponCode, setCouponCode] = useState('');
  const [checkoutPreview, setCheckoutPreview] = useState<BillingCheckoutPreview | null>(null);
  const [checkoutStatus, setCheckoutStatus] = useState(
    'Choose a plan or recharge path to price the next checkout.',
  );
  const [previewLoading, setPreviewLoading] = useState(false);
  const [orderLoading, setOrderLoading] = useState(false);
  const [queueActionOrderId, setQueueActionOrderId] = useState<string | null>(null);
  const [queueActionType, setQueueActionType] = useState<'settle' | 'cancel' | null>(null);
  const [checkoutSession, setCheckoutSession] = useState<PortalCommerceCheckoutSession | null>(null);
  const [checkoutSessionOrderId, setCheckoutSessionOrderId] = useState<string | null>(null);
  const [providerEventOrderId, setProviderEventOrderId] = useState<string | null>(null);
  const [providerEventType, setProviderEventType] = useState<PortalCommercePaymentEventType | null>(
    null,
  );
  const [checkoutSessionStatus, setCheckoutSessionStatus] = useState(
    'Open session from Pending payment queue to inspect the payment rail.',
  );
  const [checkoutSessionLoading, setCheckoutSessionLoading] = useState(false);
  const deferredSearch = useDeferredValue(searchQuery.trim().toLowerCase());

  const recommendation = recommendBillingChange(summary, plans, packs, usageRecords);

  useEffect(() => {
    let cancelled = false;

    void loadBillingPageData()
      .then((data) => {
        if (cancelled) {
          return;
        }

        applyBillingPageData(data);
        setStatus(
          'Billing posture now combines live quota evidence, checkout state, and the payment lifecycle timeline.',
        );
      })
      .catch((error) => {
        if (!cancelled) {
          setStatus(portalErrorMessage(error));
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  function applyBillingPageData(data: BillingPageData) {
    setSummary(data.summary);
    setUsageRecords(data.usage_records);
    setPlans(data.plans);
    setPacks(data.packs);
    setOrders(data.orders);
    setMembership(data.membership);
  }

  useEffect(() => {
    if (checkoutSessionOrderId && orders.some((order) => order.order_id === checkoutSessionOrderId)) {
      return;
    }

    const nextPendingOrder = orders.find((order) => isPendingPaymentOrder(order));
    if (!nextPendingOrder) {
      setCheckoutSessionOrderId(null);
      setCheckoutSession(null);
      setCheckoutSessionStatus('Open session from Pending payment queue to inspect the payment rail.');
      return;
    }

    void loadCheckoutSession(nextPendingOrder.order_id);
  }, [orders, checkoutSessionOrderId]);

  async function refreshBillingPage(nextStatus?: string): Promise<void> {
    const data = await loadBillingPageData();
    applyBillingPageData(data);
    if (nextStatus) {
      setStatus(nextStatus);
    }
  }

  function currentRemainingUnits(): number | null {
    return summary.remaining_units ?? null;
  }

  async function loadCheckoutSession(orderId: string): Promise<void> {
    setCheckoutSessionOrderId(orderId);
    setCheckoutSessionLoading(true);
    setCheckoutSessionStatus(`Loading checkout session for ${orderId}...`);

    try {
      const session = await getBillingCheckoutSession(orderId);
      setCheckoutSession(session);
      setCheckoutSessionStatus(
        `${session.reference} maps this order into the current payment rail with ${checkoutModeLabel(session)} semantics.`,
      );
    } catch (error) {
      setCheckoutSession(null);
      setCheckoutSessionStatus(portalErrorMessage(error));
    } finally {
      setCheckoutSessionLoading(false);
    }
  }

  async function loadCheckoutPreview(
    selection: BillingSelection,
    nextCouponCode = couponCode,
  ): Promise<void> {
    setPreviewLoading(true);
    setCheckoutStatus(`Loading live checkout pricing for ${selection.target.id}...`);
    setCheckoutPreview(null);

    try {
      const quote = await previewBillingCheckout({
        target_kind: selection.kind,
        target_id: selection.target.id,
        coupon_code: nextCouponCode.trim() ? nextCouponCode.trim().toUpperCase() : null,
        current_remaining_units: currentRemainingUnits(),
      });
      setCheckoutPreview(quote);
      setCheckoutStatus(
        `${quote.target_name} is priced by the live commerce quote service and ready to create as a pending payment order.`,
      );
    } catch (error) {
      setCheckoutStatus(portalErrorMessage(error));
    } finally {
      setPreviewLoading(false);
    }
  }

  function openCheckout(selection: BillingSelection) {
    setCheckoutSelection(selection);
    setCheckoutOpen(true);
    setCouponCode('');
    setCheckoutPreview(null);
    void loadCheckoutPreview(selection, '');
  }

  async function handleCheckoutPreviewRefresh(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!checkoutSelection) {
      return;
    }

    await loadCheckoutPreview(checkoutSelection);
  }

  async function placeOrder() {
    if (!checkoutSelection) {
      return;
    }

    setOrderLoading(true);
    setCheckoutStatus(`Creating a checkout order for ${checkoutSelection.target.id}...`);

    try {
      const order = await createBillingOrder({
        target_kind: checkoutSelection.kind,
        target_id: checkoutSelection.target.id,
        coupon_code: couponCode.trim() ? couponCode.trim().toUpperCase() : null,
      });
      await refreshBillingPage(
        `${order.target_name} was queued in Pending payment queue. Settle it before quota or membership changes are applied.`,
      );
      await loadCheckoutSession(order.order_id);
      setCheckoutOpen(false);
      setCheckoutPreview(null);
      setCouponCode('');
      setCheckoutSelection(null);
    } catch (error) {
      setCheckoutStatus(portalErrorMessage(error));
    } finally {
      setOrderLoading(false);
    }
  }

  async function handleQueueAction(
    order: PortalCommerceOrder,
    action: 'settle' | 'cancel',
  ): Promise<void> {
    setQueueActionOrderId(order.order_id);
    setQueueActionType(action);
    setStatus(
      action === 'settle'
        ? `Settling ${order.target_name} into active workspace quota...`
        : `Canceling ${order.target_name} before fulfillment is applied...`,
    );

    try {
      const nextOrder =
        action === 'settle'
          ? await settleBillingOrder(order.order_id)
          : await cancelBillingOrder(order.order_id);
      await refreshBillingPage(
        action === 'settle'
          ? `${nextOrder.target_name} was settled and moved into Order timeline.`
          : `${nextOrder.target_name} was canceled and left out of quota fulfillment.`,
      );
      await loadCheckoutSession(nextOrder.order_id);
    } catch (error) {
      setStatus(portalErrorMessage(error));
    } finally {
      setQueueActionOrderId(null);
      setQueueActionType(null);
    }
  }

  async function handleProviderEvent(eventType: PortalCommercePaymentEventType): Promise<void> {
    if (!checkoutSessionOrderId) {
      return;
    }

    setProviderEventOrderId(checkoutSessionOrderId);
    setProviderEventType(eventType);
    setStatus(
      eventType === 'settled'
        ? `Replaying provider settlement for ${checkoutSessionOrderId}...`
        : eventType === 'failed'
          ? `Replaying provider failure for ${checkoutSessionOrderId}...`
          : `Replaying provider cancellation for ${checkoutSessionOrderId}...`,
    );

    try {
      const nextOrder = await sendBillingPaymentEvent(checkoutSessionOrderId, {
        event_type: eventType,
      });
      await refreshBillingPage(
        eventType === 'settled'
          ? `${nextOrder.target_name} was settled through the provider callback seam.`
          : eventType === 'failed'
            ? `${nextOrder.target_name} was marked failed and left out of fulfillment.`
            : `${nextOrder.target_name} was canceled through the provider callback seam.`,
      );
      await loadCheckoutSession(nextOrder.order_id);
    } catch (error) {
      setStatus(portalErrorMessage(error));
    } finally {
      setProviderEventOrderId(null);
      setProviderEventType(null);
    }
  }

  const remainingUnitsLabel =
    summary.remaining_units === null || summary.remaining_units === undefined
      ? 'Unlimited'
      : formatUnits(summary.remaining_units);
  const searchedOrders = orders.filter((order) => orderMatchesSearch(order, deferredSearch));
  const pendingOrders = orders.filter((order) => isPendingPaymentOrder(order));
  const failedOrders = orders.filter((order) => order.status === 'failed');
  const timelineOrders = orders.filter((order) => !isPendingPaymentOrder(order));
  const visibleOrders = searchedOrders.filter((order) => matchesOrderLane(order, orderLane));
  const pendingPaymentCount = orders.filter((order) => isPendingPaymentOrder(order)).length;
  const failedPaymentCount = failedOrders.length;
  const timelineOrderCount = timelineOrders.length;
  const selectedTargetLabel = selectionLabel(checkoutSelection);
  const orderWorkbenchCopy = orderWorkbenchDetail(orderLane);
  const orderEmptyTitle =
    orderLane === 'pending_payment'
      ? t('No pending payment orders for this slice')
      : orderLane === 'failed'
        ? t('No failed payment orders for this slice')
        : orderLane === 'timeline'
          ? t('No timeline orders for this slice')
          : t('No orders for this slice');
  const orderEmptyDetail = orders.length
    ? orderLane === 'pending_payment'
      ? 'Adjust the search or switch Order lane to reveal a different pending checkout.'
      : orderLane === 'failed'
        ? 'No failed payment orders match the current search or lane selection.'
        : orderLane === 'timeline'
          ? 'Adjust the search or switch Order lane to reveal a different settled or canceled order.'
          : 'Adjust the search or switch Order lane to reveal a different checkout.'
    : 'Create the first subscription or recharge checkout and it will appear here.';

  return (
    <>
      <Dialog open={checkoutOpen} onOpenChange={setCheckoutOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('Checkout preview')}</DialogTitle>
            <DialogDescription>{checkoutStatus}</DialogDescription>
          </DialogHeader>

          <form className="grid gap-4" onSubmit={(event) => void handleCheckoutPreviewRefresh(event)}>
            <div className="grid gap-4 md:grid-cols-2">
              <div className="rounded-[28px] border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/50">
                <p className="text-xs font-semibold uppercase tracking-[0.22em] text-zinc-500 dark:text-zinc-400">
                  Selected offer
                </p>
                <h3 className="mt-3 text-xl font-semibold text-zinc-950 dark:text-zinc-50">
                  {selectedTargetLabel ?? 'Checkout preview'}
                </h3>
                <p className="mt-2 text-sm text-zinc-600 dark:text-zinc-300">
                  {checkoutPreview
                    ? `${checkoutPreview.target_kind} · ${checkoutPreview.payable_price_label}`
                    : 'Preview the live quote before creating a checkout.'}
                </p>
                <div className="mt-4 flex flex-wrap gap-2">
                  <Pill tone="accent">{checkoutSelection?.kind ?? 'commerce'}</Pill>
                  {checkoutPreview?.applied_coupon ? (
                    <Pill tone="warning">{checkoutPreview.applied_coupon.code}</Pill>
                  ) : null}
                </div>
              </div>

              <div className="rounded-[28px] border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/50">
                <p className="text-xs font-semibold uppercase tracking-[0.22em] text-zinc-500 dark:text-zinc-400">
                  Order impact
                </p>
                <div className="mt-3 grid gap-3 text-sm text-zinc-600 dark:text-zinc-300">
                  <div className="flex items-center justify-between gap-3">
                    <span>Payable price</span>
                    <strong className="text-zinc-950 dark:text-zinc-50">
                      {checkoutPreview?.payable_price_label ?? 'Pending'}
                    </strong>
                  </div>
                  <div className="flex items-center justify-between gap-3">
                    <span>Granted units</span>
                    <strong className="text-zinc-950 dark:text-zinc-50">
                      {checkoutPreview ? formatUnits(checkoutPreview.granted_units) : 'Pending'}
                    </strong>
                  </div>
                  <div className="flex items-center justify-between gap-3">
                    <span>Projected remaining</span>
                    <strong className="text-zinc-950 dark:text-zinc-50">
                      {checkoutPreview?.projected_remaining_units === null
                      || checkoutPreview?.projected_remaining_units === undefined
                        ? remainingUnitsLabel
                        : formatUnits(checkoutPreview.projected_remaining_units)}
                    </strong>
                  </div>
                </div>
              </div>
            </div>

            <FormField
              hint="Optional coupon codes are priced by the backend quote service before checkout creation."
              label={t('Coupon code')}
            >
              <Input
                onChange={(event) => {
                  setCouponCode(event.target.value);
                  setCheckoutPreview(null);
                }}
                placeholder={t('SPRING20')}
                value={couponCode}
              />
            </FormField>

            <DialogFooter>
              <Button onClick={() => setCheckoutOpen(false)} type="button" variant="ghost">
                {t('Close')}
              </Button>
              <Button onClick={() => onNavigate('credits')} type="button" variant="secondary">
                {t('Open credits')}
              </Button>
              <Button disabled={previewLoading || orderLoading} type="submit" variant="secondary">
                {previewLoading ? t('Loading preview...') : t('Refresh preview')}
              </Button>
              <Button
                disabled={!checkoutPreview || previewLoading || orderLoading}
                onClick={() => void placeOrder()}
                type="button"
              >
                {orderLoading ? t('Creating checkout...') : t('Create checkout')}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      <div className="grid gap-4">
        <section
          data-slot="portal-billing-toolbar"
          className="rounded-[28px] border border-zinc-200/80 bg-white/92 p-4 shadow-[0_18px_48px_rgba(15,23,42,0.08)] backdrop-blur dark:border-zinc-800/80 dark:bg-zinc-950/70 sm:p-5"
        >
          <ToolbarInline>
            <ToolbarSearchField
              label={t('Search order lifecycle')}
              value={searchQuery}
              onChange={(event) => setSearchQuery(event.target.value)}
              placeholder={t('Search order lifecycle')}
              className="min-w-[15rem] flex-[0_1_20rem]"
            />
            <ToolbarField label={t('Order lane')} className="min-w-[12rem] shrink-0">
              <Select
                value={orderLane}
                onChange={(event) => setOrderLane(event.target.value as OrderWorkbenchLane)}
              >
                <option value="all">{t('All orders')}</option>
                <option value="pending_payment">{t('Pending payment queue')}</option>
                <option value="failed">{t('Failed payment')}</option>
                <option value="timeline">{t('Order timeline')}</option>
              </Select>
            </ToolbarField>
            <div className="ml-auto flex shrink-0 items-center gap-2.5 whitespace-nowrap">
              <Button type="button" onClick={() => onNavigate('credits')}>
                {t('Open credits')}
              </Button>
              <InlineButton onClick={() => onNavigate('usage')} tone="secondary">
                {t('Open usage')}
              </InlineButton>
              <InlineButton onClick={() => onNavigate('account')} tone="secondary">
                {t('Open account')}
              </InlineButton>
            </div>
          </ToolbarInline>
        </section>

        <section className="grid gap-4 xl:grid-cols-[1.2fr_0.8fr]">
          <Card className="rounded-[32px] border-zinc-200/80 bg-white/92 shadow-[0_18px_48px_rgba(15,23,42,0.08)] dark:border-zinc-800/80 dark:bg-zinc-950/70">
            <CardHeader>
              <CardTitle>{t('Decision support')}</CardTitle>
              <CardDescription>{status}</CardDescription>
            </CardHeader>
            <CardContent className="grid gap-4">
              <BillingRecommendationCard recommendation={recommendation} />
              <div className="grid gap-3 md:grid-cols-2">
                <DecisionCard
                  detail="Live quota still available to the current workspace."
                  label="Remaining units"
                  value={remainingUnitsLabel}
                />
                <DecisionCard
                  detail="Total token units already consumed."
                  label="Used units"
                  value={formatUnits(summary.used_units)}
                />
                <DecisionCard
                  detail="Live amount visible in the billing summary."
                  label="Booked amount"
                  value={formatCurrency(summary.booked_amount)}
                />
                <DecisionCard
                  detail="Open checkout orders that still need settlement before quota changes apply."
                  label="Pending payment"
                  value={String(pendingPaymentCount)}
                />
                <DecisionCard
                  detail="Payment attempts that closed on the failure path and need a fresh checkout decision."
                  label="Failed payment"
                  value={String(failedPaymentCount)}
                />
              </div>
            </CardContent>
          </Card>

          <div className="grid gap-4">
            <InfoPanel
              detail={
                membership
                  ? `${membership.plan_name} is the active workspace membership and defines the current subscription entitlement baseline.`
                  : 'No active membership is recorded yet. Settle a subscription order to activate monthly entitlement posture.'
              }
              title="Active membership"
            >
              <div className="grid gap-3 text-sm text-zinc-600 dark:text-zinc-300">
                <div className="flex items-center justify-between gap-3">
                  <span>Plan</span>
                  <strong className="text-zinc-950 dark:text-zinc-50">
                    {membership?.plan_name ?? 'No membership'}
                  </strong>
                </div>
                <div className="flex items-center justify-between gap-3">
                  <span>Cadence</span>
                  <strong className="text-zinc-950 dark:text-zinc-50">
                    {membership?.cadence ?? 'n/a'}
                  </strong>
                </div>
                <div className="flex items-center justify-between gap-3">
                  <span>Included units</span>
                  <strong className="text-zinc-950 dark:text-zinc-50">
                    {membership ? formatUnits(membership.included_units) : 'n/a'}
                  </strong>
                </div>
                <div className="flex items-center justify-between gap-3">
                  <span>Status</span>
                  <strong className="text-zinc-950 dark:text-zinc-50">
                    {membership?.status ?? 'inactive'}
                  </strong>
                </div>
              </div>
            </InfoPanel>

            <InfoPanel
              detail={recommendation.runway.detail}
              title="Estimated runway"
            >
              <div className="grid gap-3 text-sm text-zinc-600 dark:text-zinc-300">
                <div className="flex items-center justify-between gap-3">
                  <span>Projected coverage</span>
                  <strong className="text-zinc-950 dark:text-zinc-50">
                    {recommendation.runway.label}
                  </strong>
                </div>
                <div className="flex items-center justify-between gap-3">
                  <span>Daily burn</span>
                  <strong className="text-zinc-950 dark:text-zinc-50">
                    {recommendation.runway.daily_units
                      ? `${formatUnits(recommendation.runway.daily_units)} / day`
                      : 'Needs data'}
                  </strong>
                </div>
                <div className="flex items-center justify-between gap-3">
                  <span>Quota posture</span>
                  <strong className="text-zinc-950 dark:text-zinc-50">
                    {summary.exhausted ? 'Exhausted' : 'Active'}
                  </strong>
                </div>
              </div>
            </InfoPanel>

            <InfoPanel
              detail={recommendation.bundle.detail}
              title="Recommended bundle"
            >
              <div className="grid gap-3 text-sm text-zinc-600 dark:text-zinc-300">
                <div className="flex items-center justify-between gap-3">
                  <span>Subscription</span>
                  <strong className="text-zinc-950 dark:text-zinc-50">
                    {recommendation.plan?.name ?? 'None'}
                  </strong>
                </div>
                <div className="flex items-center justify-between gap-3">
                  <span>Recharge buffer</span>
                  <strong className="text-zinc-950 dark:text-zinc-50">
                    {recommendation.pack?.label ?? 'Optional'}
                  </strong>
                </div>
                <div className="flex items-center justify-between gap-3">
                  <span>Bundle posture</span>
                  <strong className="text-zinc-950 dark:text-zinc-50">
                    {recommendation.bundle.title}
                  </strong>
                </div>
              </div>
            </InfoPanel>
          </div>
        </section>

        <section className="grid gap-4 xl:grid-cols-2">
          <CatalogPanel
            detail="Choose the monthly posture that best matches expected gateway demand."
            title="Plan catalog"
          >
            <div className="grid gap-3">
              {plans.map((plan) => (
                <article
                  key={plan.id}
                  className="rounded-[28px] border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/50"
                >
                  <div className="flex flex-wrap items-start justify-between gap-3">
                    <div className="grid gap-2">
                      <div className="flex flex-wrap gap-2">
                        <Pill tone={isRecommendedPlan(plan, recommendation) ? 'positive' : 'default'}>
                          {plan.name}
                        </Pill>
                        <Pill tone="accent">{plan.price_label}</Pill>
                      </div>
                      <strong className="text-lg text-zinc-950 dark:text-zinc-50">
                        {formatUnits(plan.included_units)} included units
                      </strong>
                      <p className="text-sm text-zinc-600 dark:text-zinc-300">{plan.highlight}</p>
                    </div>
                    <Button
                      type="button"
                      onClick={() =>
                        openCheckout({
                          kind: 'subscription_plan',
                          target: plan,
                        })
                      }
                    >
                      {plan.cta}
                    </Button>
                  </div>
                </article>
              ))}
            </div>
          </CatalogPanel>

          <CatalogPanel
            detail="Use top-ups to restore headroom without changing the base plan."
            title="Recharge packs"
          >
            <div className="grid gap-3">
              {packs.map((pack) => (
                <article
                  key={pack.id}
                  className="rounded-[28px] border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/50"
                >
                  <div className="flex flex-wrap items-start justify-between gap-3">
                    <div className="grid gap-2">
                      <div className="flex flex-wrap gap-2">
                        <Pill tone={isRecommendedPack(pack, recommendation) ? 'warning' : 'default'}>
                          {pack.label}
                        </Pill>
                        <Pill tone="accent">{pack.price_label}</Pill>
                      </div>
                      <strong className="text-lg text-zinc-950 dark:text-zinc-50">
                        {formatUnits(pack.points)} units
                      </strong>
                      <p className="text-sm text-zinc-600 dark:text-zinc-300">{pack.note}</p>
                    </div>
                    <Button
                      type="button"
                      onClick={() =>
                        openCheckout({
                          kind: 'recharge_pack',
                          target: pack,
                        })
                      }
                    >
                      {t('Create checkout')}
                    </Button>
                  </div>
                </article>
              ))}
            </div>
          </CatalogPanel>
        </section>

        <section>
          <Card className="rounded-[32px] border-zinc-200/80 bg-white/92 shadow-[0_18px_48px_rgba(15,23,42,0.08)] dark:border-zinc-800/80 dark:bg-zinc-950/70">
            <CardHeader>
              <CardTitle>{t('Order workbench')}</CardTitle>
              <CardDescription>{orderWorkbenchCopy}</CardDescription>
            </CardHeader>
            <CardContent className="grid gap-4">
              <div className="flex flex-wrap gap-2">
                <Pill tone={orderLane === 'all' ? 'accent' : 'default'}>
                  {orders.length} all orders
                </Pill>
                <Pill tone={orderLane === 'pending_payment' ? 'accent' : 'default'}>
                  {pendingPaymentCount} Pending payment queue
                </Pill>
                <Pill tone={orderLane === 'failed' ? 'warning' : 'default'}>
                  {failedPaymentCount} Failed payment
                </Pill>
                <Pill tone={orderLane === 'timeline' ? 'positive' : 'default'}>
                  {timelineOrderCount} Order timeline
                </Pill>
              </div>

              <DataTable
                columns={[
                  {
                    key: 'offer',
                    label: 'Offer',
                    render: (row) => row.target_name,
                  },
                  {
                    key: 'kind',
                    label: 'Kind',
                    render: (row) => row.target_kind,
                  },
                  {
                    key: 'coupon',
                    label: 'Coupon',
                    render: (row) => row.applied_coupon_code ?? 'None',
                  },
                  {
                    key: 'payable',
                    label: 'Payable',
                    render: (row) => row.payable_price_label,
                  },
                  {
                    key: 'units',
                    label: 'Granted units',
                    render: (row) => formatUnits(row.granted_units + row.bonus_units),
                  },
                  {
                    key: 'status',
                    label: 'Status',
                    render: (row) => <Pill tone={orderStatusTone(row.status)}>{row.status}</Pill>,
                  },
                  {
                    key: 'time',
                    label: 'Created',
                    render: (row) => formatDateTime(row.created_at_ms),
                  },
                  {
                    key: 'actions',
                    label: 'Actions',
                    render: (row) => (
                      <div className="flex flex-wrap gap-2">
                        <InlineButton
                          disabled={checkoutSessionLoading}
                          onClick={() => void loadCheckoutSession(row.order_id)}
                          tone="secondary"
                        >
                          {checkoutSessionLoading && checkoutSessionOrderId === row.order_id
                            ? t('Loading session...')
                            : t('Open session')}
                        </InlineButton>
                        {isPendingPaymentOrder(row) ? (
                          <>
                            <InlineButton
                              disabled={queueActionOrderId !== null}
                              onClick={() => void handleQueueAction(row, 'settle')}
                              tone="primary"
                            >
                              {queueActionOrderId === row.order_id && queueActionType === 'settle'
                                ? t('Settling...')
                                : t('Settle order')}
                            </InlineButton>
                            <InlineButton
                              disabled={queueActionOrderId !== null}
                              onClick={() => void handleQueueAction(row, 'cancel')}
                              tone="secondary"
                            >
                              {queueActionOrderId === row.order_id && queueActionType === 'cancel'
                                ? t('Canceling...')
                                : t('Cancel order')}
                            </InlineButton>
                          </>
                        ) : null}
                      </div>
                    ),
                  },
                ]}
                empty={(
                  <div className="mx-auto flex max-w-[32rem] flex-col items-center gap-2 text-center">
                    <strong className="text-base font-semibold text-zinc-950 dark:text-zinc-50">
                      {orderEmptyTitle}
                    </strong>
                    <p className="text-sm text-zinc-500 dark:text-zinc-400">
                      {orderEmptyDetail}
                    </p>
                  </div>
                )}
                getKey={(row) => row.order_id}
                rows={visibleOrders}
              />
            </CardContent>
          </Card>
        </section>

        <section className="grid gap-4 xl:grid-cols-[0.95fr_1.05fr]">
          <InfoPanel
            detail={checkoutSessionStatus}
            title="Checkout session"
          >
            {checkoutSession ? (
              <div className="grid gap-4">
                <div className="grid gap-3 md:grid-cols-2 text-sm text-zinc-600 dark:text-zinc-300">
                  <div className="flex items-center justify-between gap-3">
                    <span>Reference</span>
                    <strong className="text-zinc-950 dark:text-zinc-50">
                      {checkoutSession.reference}
                    </strong>
                  </div>
                  <div className="flex items-center justify-between gap-3">
                    <span>Payment rail</span>
                    <strong className="text-zinc-950 dark:text-zinc-50">
                      {checkoutSession.provider}
                    </strong>
                  </div>
                  <div className="flex items-center justify-between gap-3">
                    <span>Checkout mode</span>
                    <strong className="text-zinc-950 dark:text-zinc-50">
                      {checkoutModeLabel(checkoutSession)}
                    </strong>
                  </div>
                  <div className="flex items-center justify-between gap-3">
                    <span>Session status</span>
                    <strong className="text-zinc-950 dark:text-zinc-50">
                      {checkoutSession.session_status}
                    </strong>
                  </div>
                </div>

                <div className="rounded-[28px] border border-zinc-200 bg-zinc-50/90 p-5 dark:border-zinc-800 dark:bg-zinc-900/80">
                  <p className="text-xs font-semibold uppercase tracking-[0.22em] text-zinc-500 dark:text-zinc-400">
                    Guidance
                  </p>
                  <p className="mt-3 text-sm text-zinc-600 dark:text-zinc-300">
                    {checkoutSession.guidance}
                  </p>
                </div>

                <div className="grid gap-3">
                  {checkoutSession.methods.length ? (
                    checkoutSession.methods.map((method) => (
                      <article
                        key={method.id}
                        className="rounded-[28px] border border-zinc-200 bg-zinc-50/80 p-5 dark:border-zinc-800 dark:bg-zinc-900/50"
                      >
                        <div className="flex flex-wrap items-start justify-between gap-3">
                          <div className="grid gap-2">
                            <div className="flex flex-wrap gap-2">
                              <Pill tone="accent">{method.label}</Pill>
                              <Pill tone={method.availability === 'available' ? 'positive' : 'warning'}>
                                {method.availability}
                              </Pill>
                            </div>
                            <p className="text-sm text-zinc-600 dark:text-zinc-300">{method.detail}</p>
                          </div>
                          <strong className="text-sm text-zinc-950 dark:text-zinc-50">
                            {method.action}
                          </strong>
                        </div>
                      </article>
                    ))
                  ) : (
                    <EmptyState
                      detail="This checkout session is already closed, so there are no remaining payment actions."
                      title={t('No checkout methods remain')}
                    />
                  )}
                </div>

                {hasProviderHandoff(checkoutSession) ? (
                  <div className="rounded-[28px] border border-zinc-200 bg-zinc-50/90 p-5 dark:border-zinc-800 dark:bg-zinc-900/80">
                    <div className="flex flex-wrap items-start justify-between gap-3">
                      <div className="grid gap-2">
                        <p className="text-xs font-semibold uppercase tracking-[0.22em] text-zinc-500 dark:text-zinc-400">
                          Provider callbacks
                        </p>
                        <p className="text-sm text-zinc-600 dark:text-zinc-300">
                          Simulate hosted payment callbacks so server mode can rehearse settlement,
                          failure, and cancellation without wiring a live PSP SDK.
                        </p>
                      </div>
                      <Pill tone="warning">Webhook seam</Pill>
                    </div>
                    <div className="mt-4 flex flex-wrap gap-2">
                      <InlineButton
                        disabled={providerEventOrderId !== null}
                        onClick={() => void handleProviderEvent('settled')}
                        tone="primary"
                      >
                        {providerEventOrderId === checkoutSessionOrderId
                        && providerEventType === 'settled'
                          ? t('Applying settlement...')
                          : t('Simulate provider settlement')}
                      </InlineButton>
                      <InlineButton
                        disabled={providerEventOrderId !== null}
                        onClick={() => void handleProviderEvent('failed')}
                        tone="secondary"
                      >
                        {providerEventOrderId === checkoutSessionOrderId
                        && providerEventType === 'failed'
                          ? t('Applying failure...')
                          : t('Simulate provider failure')}
                      </InlineButton>
                      <InlineButton
                        disabled={providerEventOrderId !== null}
                        onClick={() => void handleProviderEvent('canceled')}
                        tone="secondary"
                      >
                        {providerEventOrderId === checkoutSessionOrderId
                        && providerEventType === 'canceled'
                          ? t('Applying cancel...')
                          : t('Simulate provider cancel')}
                      </InlineButton>
                    </div>
                  </div>
                ) : null}
              </div>
            ) : (
              <EmptyState
                detail="Open session from Pending payment queue to inspect the provider-neutral checkout seam for the selected order."
                title={t('No checkout session selected')}
              />
            )}
          </InfoPanel>

          <InfoPanel
            detail="Provider-neutral checkout seams keep local desktop mode and server-hosted payment providers aligned under one payment rail."
            title="Payment rail"
          >
            <div className="grid gap-3 text-sm text-zinc-600 dark:text-zinc-300">
              <div className="flex items-center justify-between gap-3">
                <span>Local desktop mode</span>
                <strong className="text-zinc-950 dark:text-zinc-50">
                  Operator settlement
                </strong>
              </div>
              <div className="flex items-center justify-between gap-3">
                <span>Server mode seam</span>
                <strong className="text-zinc-950 dark:text-zinc-50">
                  Provider handoff
                </strong>
              </div>
              <div className="flex items-center justify-between gap-3">
                <span>Current selected reference</span>
                <strong className="text-zinc-950 dark:text-zinc-50">
                  {checkoutSession?.reference ?? 'Awaiting pending order'}
                </strong>
              </div>
              <div className="flex items-center justify-between gap-3">
                <span>Payable price</span>
                <strong className="text-zinc-950 dark:text-zinc-50">
                  {checkoutSession?.payable_price_label ?? 'n/a'}
                </strong>
              </div>
            </div>
          </InfoPanel>
        </section>
      </div>
    </>
  );
}

function DecisionCard({
  label,
  value,
  detail,
}: {
  label: string;
  value: string;
  detail: string;
}) {
  return (
    <article className="rounded-[28px] border border-zinc-200 bg-zinc-50/90 p-5 dark:border-zinc-800 dark:bg-zinc-900/80">
      <p className="text-xs font-semibold uppercase tracking-[0.22em] text-zinc-500 dark:text-zinc-400">
        {label}
      </p>
      <strong className="mt-3 block text-2xl text-zinc-950 dark:text-zinc-50">{value}</strong>
      <p className="mt-2 text-sm text-zinc-600 dark:text-zinc-300">{detail}</p>
    </article>
  );
}

function InfoPanel({
  title,
  detail,
  children,
}: {
  title: string;
  detail: string;
  children: ReactNode;
}) {
  return (
    <Card className="rounded-[32px] border-zinc-200/80 bg-white/92 shadow-[0_18px_48px_rgba(15,23,42,0.08)] dark:border-zinc-800/80 dark:bg-zinc-950/70">
      <CardHeader>
        <CardTitle>{title}</CardTitle>
        <CardDescription>{detail}</CardDescription>
      </CardHeader>
      <CardContent>{children}</CardContent>
    </Card>
  );
}

function CatalogPanel({
  title,
  detail,
  children,
}: {
  title: string;
  detail: string;
  children: ReactNode;
}) {
  return (
    <Card className="rounded-[32px] border-zinc-200/80 bg-white/92 shadow-[0_18px_48px_rgba(15,23,42,0.08)] dark:border-zinc-800/80 dark:bg-zinc-950/70">
      <CardHeader>
        <CardTitle>{title}</CardTitle>
        <CardDescription>{detail}</CardDescription>
      </CardHeader>
      <CardContent>{children}</CardContent>
    </Card>
  );
}
