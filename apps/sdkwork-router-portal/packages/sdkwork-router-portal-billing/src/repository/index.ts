import {
  cancelPortalCommerceOrder,
  createPortalCommerceOrder,
  getPortalCommerceCheckoutSession,
  getPortalCommerceCatalog,
  getPortalCommerceMembership,
  getPortalBillingSummary,
  listPortalCommerceOrders,
  listPortalUsageRecords,
  previewPortalCommerceQuote,
  sendPortalCommercePaymentEvent,
  settlePortalCommerceOrder,
} from 'sdkwork-router-portal-portal-api';
import type { PortalCommerceOrder, PortalCommerceQuote } from 'sdkwork-router-portal-types';
import type {
  PortalCommerceCheckoutSession,
  PortalCommercePaymentEventRequest,
} from 'sdkwork-router-portal-types';

import type { BillingPageData } from '../types';

export async function loadBillingPageData(): Promise<BillingPageData> {
  const [summary, usage_records, catalog, orders, membership] = await Promise.all([
    getPortalBillingSummary(),
    listPortalUsageRecords(),
    getPortalCommerceCatalog(),
    listPortalCommerceOrders(),
    getPortalCommerceMembership(),
  ]);

  return {
    summary,
    usage_records,
    plans: catalog.plans,
    packs: catalog.packs,
    orders,
    membership,
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

export function sendBillingPaymentEvent(
  order_id: string,
  input: PortalCommercePaymentEventRequest,
): Promise<PortalCommerceOrder> {
  return sendPortalCommercePaymentEvent(order_id, input);
}
