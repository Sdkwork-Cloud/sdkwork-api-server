import type {
  PortalCommerceOrder,
  PortalCommerceQuote,
  PortalCustomRechargePolicy,
  PortalRechargeOption,
  ProjectBillingSummary,
} from 'sdkwork-router-portal-types';

import type {
  PortalRechargeQuoteSnapshot,
} from '../types';

type TranslateFn = (text: string, values?: Record<string, string | number>) => string;
export type PortalRechargeAmountValidationResult =
  | 'disabled'
  | 'below_minimum'
  | 'above_maximum'
  | 'step_mismatch';

export interface PortalRechargeOptionMerchandising {
  badge: string;
  intentLabel: string;
  supportLabel: string;
}

export interface PortalRechargePendingPaymentSpotlight {
  headline: string;
  detail: string;
  latestOrderLabel: string;
  ctaLabel: string;
  count: number;
  latestOrder: PortalCommerceOrder;
}

const rechargeCurrencyFormatter = new Intl.NumberFormat('en-US', {
  style: 'currency',
  currency: 'USD',
  minimumFractionDigits: 2,
  maximumFractionDigits: 2,
});
const rechargeUnitsFormatter = new Intl.NumberFormat('en-US');

function formatCurrency(amount: number): string {
  return rechargeCurrencyFormatter.format(amount);
}

function formatUnits(units: number): string {
  return rechargeUnitsFormatter.format(units);
}

export function buildPortalRechargeQuoteSnapshot(input: {
  customRechargePolicy: PortalCustomRechargePolicy | null;
  quote: PortalCommerceQuote | null;
  summary: ProjectBillingSummary | null;
  t: TranslateFn;
}): PortalRechargeQuoteSnapshot | null {
  const { customRechargePolicy, quote, summary, t } = input;
  if (!quote) {
    return null;
  }

  const projectedBalance =
    quote.projected_remaining_units === null || quote.projected_remaining_units === undefined
      ? summary?.remaining_units ?? null
      : quote.projected_remaining_units;
  const projectedBalanceLabel =
    projectedBalance === null || projectedBalance === undefined
      ? t('Unlimited')
      : formatUnits(projectedBalance);

  return {
    amountLabel: formatCurrency((quote.amount_cents ?? quote.list_price_cents) / 100),
    projectedBalanceLabel,
    grantedUnitsLabel: formatUnits(quote.granted_units + quote.bonus_units),
    effectiveRatioLabel:
      quote.effective_ratio_label
      ?? customRechargePolicy?.rules[0]?.effective_ratio_label
      ?? t('n/a'),
    pricingRuleLabel:
      quote.pricing_rule_label
      ?? t('Tiered custom recharge'),
  };
}

export function buildPortalRechargeHistoryRows(
  orders: PortalCommerceOrder[] | null | undefined,
): PortalCommerceOrder[] {
  return (orders ?? [])
    .filter((order) => order.target_kind === 'custom_recharge' || order.target_kind === 'recharge_pack')
    .slice()
    .sort((left, right) => right.created_at_ms - left.created_at_ms);
}

export function buildPortalRechargeOptionMerchandising(input: {
  option: PortalRechargeOption;
  options: PortalRechargeOption[];
  t: TranslateFn;
}): PortalRechargeOptionMerchandising {
  const { option, options, t } = input;
  const rankedOptions = (options ?? [])
    .slice()
    .sort((left, right) => left.amount_cents - right.amount_cents);
  const optionIndex = rankedOptions.findIndex((candidate) => candidate.id === option.id);
  const lastIndex = rankedOptions.length - 1;

  if (option.recommended) {
    return {
      badge: t('Recommended default'),
      intentLabel: t('Best fit for steady usage'),
      supportLabel: t('The safest default when you want clean value and low decision friction.'),
    };
  }

  if (optionIndex <= 0) {
    return {
      badge: t('Quick coverage'),
      intentLabel: t('Best for immediate runway'),
      supportLabel: t('Keep service continuity covered without overfunding the workspace.'),
    };
  }

  if (optionIndex === lastIndex) {
    return {
      badge: t('Reserve build'),
      intentLabel: t('Built for scale planning'),
      supportLabel: t('Use the larger reserve when you want longer runway and fewer manual top-ups.'),
    };
  }

  return {
    badge: t('Planned growth'),
    intentLabel: t('Ready for the next usage step'),
    supportLabel: option.note?.trim() || t('A balanced top-up when you want more headroom without jumping to a larger reserve.'),
  };
}

export function buildPortalRechargePendingPaymentSpotlight(input: {
  orders: PortalCommerceOrder[] | null | undefined;
  t: TranslateFn;
}): PortalRechargePendingPaymentSpotlight | null {
  const { orders, t } = input;
  const pendingOrders = buildPortalRechargeHistoryRows(orders)
    .filter((order) => order.status === 'pending_payment');

  if (pendingOrders.length === 0) {
    return null;
  }

  return {
    headline: t('Pending settlement queue'),
    detail: t('{count} orders waiting for payment completion in billing.', {
      count: pendingOrders.length,
    }),
    latestOrderLabel: t('Latest pending order'),
    ctaLabel: t('Open billing to complete payment'),
    count: pendingOrders.length,
    latestOrder: pendingOrders[0],
  };
}

export function validatePortalRechargeAmount(
  amountCents: number,
  policy: PortalCustomRechargePolicy | null,
): PortalRechargeAmountValidationResult | null {
  if (!policy) {
    return null;
  }

  if (!policy.enabled) {
    return 'disabled';
  }

  if (amountCents < policy.min_amount_cents) {
    return 'below_minimum';
  }

  if (amountCents > policy.max_amount_cents) {
    return 'above_maximum';
  }

  if (policy.step_amount_cents > 0) {
    const delta = amountCents - policy.min_amount_cents;
    if (delta % policy.step_amount_cents !== 0) {
      return 'step_mismatch';
    }
  }

  return null;
}

export function parseRechargeAmountInput(value: string): number | null {
  const normalized = value.trim();
  if (!normalized) {
    return null;
  }

  const amount = Number(normalized);
  if (!Number.isFinite(amount) || amount <= 0) {
    return null;
  }

  return Math.round(amount * 100);
}

export function formatRechargeAmountInput(amountCents: number | null): string {
  if (amountCents === null || amountCents === undefined || amountCents <= 0) {
    return '';
  }

  return (amountCents / 100).toString();
}
