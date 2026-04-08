import {
  cancelPortalCommerceOrder,
  createPortalCommercePaymentAttempt,
  createPortalCommerceOrder,
  getPortalCommercialAccountHistory,
  getPortalBillingEventSummary,
  getPortalBillingEvents,
  getPortalCommerceCheckoutSession,
  getPortalCommerceCatalog,
  getPortalCommerceOrder,
  getPortalCommerceOrderCenter,
  getPortalCommercePaymentAttempt,
  getPortalBillingSummary,
  listPortalCommercialPricingPlans,
  listPortalCommercialPricingRates,
  listPortalCommercePaymentAttempts,
  listPortalCommercePaymentMethods,
  listPortalUsageRecords,
  previewPortalCommerceQuote,
  sendPortalCommercePaymentEvent,
  settlePortalCommerceOrder,
} from 'sdkwork-router-portal-portal-api';
import type {
  CommercePaymentAttemptRecord,
  PaymentMethodRecord,
  PortalCommercePaymentAttemptCreateRequest,
  PortalCommerceCheckoutSession,
  PortalCommerceOrder,
  PortalCommerceOrderCenterEntry,
  PortalCommercePaymentEventRequest,
  PortalCommerceQuote,
} from 'sdkwork-router-portal-types';

import type {
  BillingCheckoutDetail,
  BillingPageData,
} from '../types';
import {
  type BillingPaymentHistorySource,
  buildBillingCheckoutMethods,
  buildBillingPaymentHistory,
  buildBillingRefundHistory,
} from '../services';

function dedupeNonEmptyValues(values: Array<string | null | undefined>): string[] {
  const uniqueValues = new Set<string>();

  for (const value of values) {
    const normalizedValue = value?.trim();
    if (normalizedValue) {
      uniqueValues.add(normalizedValue);
    }
  }

  return [...uniqueValues];
}

function selectBillingPaymentMethod(
  order: PortalCommerceOrder,
  paymentMethods: PaymentMethodRecord[],
  latestPaymentAttempt: CommercePaymentAttemptRecord | null,
): PaymentMethodRecord | null {
  const selectedPaymentMethodId =
    order.payment_method_id ?? latestPaymentAttempt?.payment_method_id ?? null;

  if (selectedPaymentMethodId) {
    return paymentMethods.find((paymentMethod) => (
      paymentMethod.payment_method_id === selectedPaymentMethodId
    )) ?? null;
  }

  if (paymentMethods.length === 1) {
    return paymentMethods[0];
  }

  return null;
}

function sortBillingPaymentAttempts(
  paymentAttempts: CommercePaymentAttemptRecord[] | null | undefined,
): CommercePaymentAttemptRecord[] {
  if (!Array.isArray(paymentAttempts)) {
    return [];
  }

  return paymentAttempts
    .slice()
    .sort((left, right) => (
      right.attempt_sequence - left.attempt_sequence
      || right.updated_at_ms - left.updated_at_ms
      || right.initiated_at_ms - left.initiated_at_ms
    ));
}

function selectLatestBillingPaymentAttempt(
  order: PortalCommerceOrder,
  paymentAttempts: CommercePaymentAttemptRecord[],
): CommercePaymentAttemptRecord | null {
  const sortedAttempts = sortBillingPaymentAttempts(paymentAttempts);

  if (order.latest_payment_attempt_id) {
    return sortedAttempts.find((paymentAttempt) => (
      paymentAttempt.payment_attempt_id === order.latest_payment_attempt_id
    )) ?? null;
  }

  return sortedAttempts[0] ?? null;
}

async function loadFormalBillingHistorySources(
  orderCenterEntries: PortalCommerceOrderCenterEntry[],
): Promise<BillingPaymentHistorySource[]> {
  const orderIds = orderCenterEntries.map((entry) => entry.order.order_id);
  const [formalOrders, paymentMethodsByOrder] = await Promise.all([
    Promise.all(orderIds.map((orderId) => getPortalCommerceOrder(orderId))),
    Promise.all(orderIds.map(async (orderId) => {
      const paymentMethods = await listPortalCommercePaymentMethods(orderId);
      return [orderId, paymentMethods] as const;
    })),
  ]);

  const paymentAttemptIds = dedupeNonEmptyValues(
    formalOrders.map((order) => order.latest_payment_attempt_id),
  );
  const paymentAttempts = paymentAttemptIds.length
    ? await Promise.all(
      paymentAttemptIds.map((paymentAttemptId) => getPortalCommercePaymentAttempt(paymentAttemptId)),
    )
    : [];
  const paymentMethodsByOrderId = new Map(paymentMethodsByOrder);
  const paymentAttemptsById = new Map(
    paymentAttempts.map((paymentAttempt) => [paymentAttempt.payment_attempt_id, paymentAttempt]),
  );

  return orderCenterEntries.map((entry, index) => {
    const order = formalOrders[index];
    const paymentMethods = paymentMethodsByOrderId.get(order.order_id) ?? [];
    const latestPaymentAttempt = order.latest_payment_attempt_id
      ? (paymentAttemptsById.get(order.latest_payment_attempt_id) ?? null)
      : null;

    return {
      order,
      payment_events: entry.payment_events,
      latest_payment_event: entry.latest_payment_event,
      compatibility_checkout_session: entry.checkout_session,
      latest_payment_attempt: latestPaymentAttempt,
      selected_payment_method: selectBillingPaymentMethod(
        order,
        paymentMethods,
        latestPaymentAttempt,
      ),
    };
  });
}

export async function loadBillingPageData(): Promise<BillingPageData> {
  const [
    summary,
    usage_records,
    billing_event_summary,
    billing_events,
    catalog,
    order_center,
    commercial_history,
    commercial_pricing_plans,
    commercial_pricing_rates,
  ] = await Promise.all([
    getPortalBillingSummary(),
    listPortalUsageRecords(),
    getPortalBillingEventSummary(),
    getPortalBillingEvents(),
    getPortalCommerceCatalog(),
    getPortalCommerceOrderCenter(),
    getPortalCommercialAccountHistory(),
    listPortalCommercialPricingPlans(),
    listPortalCommercialPricingRates(),
  ]);

  const commercial_account = {
    account: commercial_history.account,
    available_balance: commercial_history.balance.available_balance,
    held_balance: commercial_history.balance.held_balance,
    consumed_balance: commercial_history.balance.consumed_balance,
    grant_balance: commercial_history.balance.grant_balance,
    active_lot_count: commercial_history.balance.active_lot_count,
  };
  const orderCenterEntries = order_center.orders;
  const billingHistorySources = await loadFormalBillingHistorySources(orderCenterEntries);

  return {
    summary,
    usage_records,
    billing_events,
    billing_event_summary,
    plans: catalog.plans,
    packs: catalog.packs,
    orders: billingHistorySources.map((source) => source.order),
    payment_history: buildBillingPaymentHistory(billingHistorySources),
    refund_history: buildBillingRefundHistory(billingHistorySources),
    payment_simulation_enabled: order_center.payment_simulation_enabled,
    membership: order_center.membership,
    commercial_reconciliation: order_center.reconciliation,
    commercial_account,
    commercial_balance: commercial_history.balance,
    commercial_benefit_lots: commercial_history.benefit_lots,
    commercial_holds: commercial_history.holds,
    commercial_request_settlements: commercial_history.request_settlements,
    commercial_pricing_plans,
    commercial_pricing_rates,
  };
}

export function previewBillingCheckout(input: {
  target_kind: 'subscription_plan' | 'recharge_pack';
  target_id: string;
  coupon_code?: string | null;
  current_remaining_units?: number | null;
}): Promise<PortalCommerceQuote> {
  return previewPortalCommerceQuote(input);
}

export function createBillingOrder(input: {
  target_kind: 'subscription_plan' | 'recharge_pack';
  target_id: string;
  coupon_code?: string | null;
}): Promise<PortalCommerceOrder> {
  return createPortalCommerceOrder(input);
}

export function settleBillingOrder(order_id: string): Promise<PortalCommerceOrder> {
  return settlePortalCommerceOrder(order_id);
}

export function cancelBillingOrder(order_id: string): Promise<PortalCommerceOrder> {
  return cancelPortalCommerceOrder(order_id);
}

export function getBillingCheckoutSession(
  order_id: string,
): Promise<PortalCommerceCheckoutSession> {
  return getPortalCommerceCheckoutSession(order_id);
}

export async function getBillingCheckoutDetail(
  order_id: string,
): Promise<BillingCheckoutDetail> {
  const [order, payment_methods, payment_attempts, checkout_session] = await Promise.all([
    getPortalCommerceOrder(order_id),
    listPortalCommercePaymentMethods(order_id),
    listPortalCommercePaymentAttempts(order_id),
    getPortalCommerceCheckoutSession(order_id),
  ]);
  const sortedPaymentAttempts = sortBillingPaymentAttempts(payment_attempts);
  const latest_payment_attempt = selectLatestBillingPaymentAttempt(order, sortedPaymentAttempts);
  const selected_payment_method = selectBillingPaymentMethod(
    order,
    payment_methods,
    latest_payment_attempt,
  );

  return {
    order,
    checkout_session,
    checkout_methods: buildBillingCheckoutMethods({
      order,
      checkout_session,
      payment_methods,
      latest_payment_attempt,
      selected_payment_method,
    }),
    payment_attempts: sortedPaymentAttempts,
    payment_methods,
    latest_payment_attempt,
    selected_payment_method,
  };
}

export function getBillingCommercialAccountHistory() {
  return getPortalCommercialAccountHistory();
}

export function getBillingOrderCenter() {
  return getPortalCommerceOrderCenter();
}

export function createBillingPaymentAttempt(
  order_id: string,
  input: PortalCommercePaymentAttemptCreateRequest,
): Promise<CommercePaymentAttemptRecord> {
  return createPortalCommercePaymentAttempt(order_id, input);
}

export function sendBillingPaymentEvent(
  order_id: string,
  input: PortalCommercePaymentEventRequest,
): Promise<PortalCommerceOrder> {
  return sendPortalCommercePaymentEvent(order_id, input);
}
