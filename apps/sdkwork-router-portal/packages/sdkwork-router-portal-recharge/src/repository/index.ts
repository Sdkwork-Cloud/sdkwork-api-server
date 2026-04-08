import {
  createPortalCommerceOrder,
  getPortalBillingSummary,
  getPortalCommerceCatalog,
  listPortalCommerceOrders,
  previewPortalCommerceQuote,
} from 'sdkwork-router-portal-portal-api';
import type { PortalCommerceOrder, PortalCommerceQuote } from 'sdkwork-router-portal-types';

import type { PortalRechargePageData } from '../types';

export async function loadPortalRechargePageData(): Promise<PortalRechargePageData> {
  const [summary, catalog, orders] = await Promise.all([
    getPortalBillingSummary(),
    getPortalCommerceCatalog(),
    listPortalCommerceOrders(),
  ]);

  return {
    summary,
    rechargeOptions: Array.isArray(catalog.recharge_options) ? catalog.recharge_options : [],
    customRechargePolicy: catalog.custom_recharge_policy ?? null,
    orders: Array.isArray(orders) ? orders : [],
  };
}

export function previewPortalRechargeQuote(input: {
  amount_cents: number;
  current_remaining_units?: number | null;
}): Promise<PortalCommerceQuote> {
  return previewPortalCommerceQuote({
    target_kind: 'custom_recharge',
    target_id: 'custom',
    custom_amount_cents: input.amount_cents,
    current_remaining_units: input.current_remaining_units,
  });
}

export function createPortalRechargeOrder(input: {
  amount_cents: number;
}): Promise<PortalCommerceOrder> {
  return createPortalCommerceOrder({
    target_kind: 'custom_recharge',
    target_id: 'custom',
    custom_amount_cents: input.amount_cents,
  });
}
