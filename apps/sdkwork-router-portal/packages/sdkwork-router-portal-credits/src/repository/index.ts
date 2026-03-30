import {
  createPortalCommerceOrder,
  getPortalCommerceCatalog,
  getPortalBillingSummary,
  listPortalBillingLedger,
  previewPortalCommerceQuote,
} from 'sdkwork-router-portal-portal-api';
import type {
  LedgerEntry,
  PortalCommerceOrder,
  PortalCommerceQuote,
  PortalCommerceCoupon,
  ProjectBillingSummary,
} from 'sdkwork-router-portal-types';

export async function loadCreditsPageData(): Promise<{
  summary: ProjectBillingSummary;
  ledger: LedgerEntry[];
  coupons: PortalCommerceCoupon[];
}> {
  const [summary, ledger, catalog] = await Promise.all([
    getPortalBillingSummary(),
    listPortalBillingLedger(),
    getPortalCommerceCatalog(),
  ]);

  return {
    summary,
    ledger,
    coupons: catalog.coupons.filter((coupon) => coupon.bonus_units > 0),
  };
}

export function previewCreditsCouponRedemption(input: {
  target_id: string;
  current_remaining_units?: number | null;
}): Promise<PortalCommerceQuote> {
  return previewPortalCommerceQuote({
    target_kind: 'coupon_redemption',
    target_id: input.target_id,
    current_remaining_units: input.current_remaining_units,
  });
}

export function createCreditsCouponRedemption(input: {
  target_id: string;
}): Promise<PortalCommerceOrder> {
  return createPortalCommerceOrder({
    target_kind: 'coupon_redemption',
    target_id: input.target_id,
  });
}
