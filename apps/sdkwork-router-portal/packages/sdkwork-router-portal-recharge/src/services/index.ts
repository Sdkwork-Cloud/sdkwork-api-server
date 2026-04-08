import type {
  BillingEventAccountingModeSummary,
  BillingEventCapabilitySummary,
  BillingEventSummary,
  PortalCommerceMembership,
  PortalCommerceOrder,
  PortalCommerceQuote,
  PortalCustomRechargePolicy,
  PortalRechargeOption,
  ProjectBillingSummary,
} from 'sdkwork-router-portal-types';

import type {
  PortalRechargeFinanceProjection,
  PortalRechargeQuoteSnapshot,
  PortalRechargeSummaryCard,
} from '../types';

type TranslateFn = (text: string, values?: Record<string, string | number>) => string;
export type PortalRechargeAmountValidationResult =
  | 'disabled'
  | 'below_minimum'
  | 'above_maximum'
  | 'step_mismatch';

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

function emptyBillingEventSummary(): BillingEventSummary {
  return {
    total_events: 0,
    project_count: 0,
    group_count: 0,
    capability_count: 0,
    total_request_count: 0,
    total_units: 0,
    total_input_tokens: 0,
    total_output_tokens: 0,
    total_tokens: 0,
    total_image_count: 0,
    total_audio_seconds: 0,
    total_video_seconds: 0,
    total_music_seconds: 0,
    total_upstream_cost: 0,
    total_customer_charge: 0,
    projects: [],
    groups: [],
    capabilities: [],
    accounting_modes: [],
  };
}

function sortAccountingModes(
  items: BillingEventAccountingModeSummary[],
): BillingEventAccountingModeSummary[] {
  return [...items]
    .filter((item) => item.event_count > 0)
    .sort((left, right) =>
      right.total_customer_charge - left.total_customer_charge
      || right.request_count - left.request_count
      || left.accounting_mode.localeCompare(right.accounting_mode),
    );
}

function sortCapabilities(
  items: BillingEventCapabilitySummary[],
): BillingEventCapabilitySummary[] {
  return [...items]
    .filter((item) => item.event_count > 0)
    .sort((left, right) =>
      right.total_customer_charge - left.total_customer_charge
      || right.request_count - left.request_count
      || right.total_tokens - left.total_tokens
      || left.capability.localeCompare(right.capability),
    );
}

export function buildPortalRechargeFinanceProjection(input: {
  membership: PortalCommerceMembership | null;
  billingEventSummary: BillingEventSummary | null | undefined;
}): PortalRechargeFinanceProjection {
  const summary = input.billingEventSummary ?? emptyBillingEventSummary();

  return {
    membership: input.membership,
    leading_accounting_mode: sortAccountingModes(summary.accounting_modes)[0] ?? null,
    leading_capability: sortCapabilities(summary.capabilities)[0] ?? null,
    multimodal_totals: {
      image_count: summary.total_image_count,
      audio_seconds: summary.total_audio_seconds,
      video_seconds: summary.total_video_seconds,
      music_seconds: summary.total_music_seconds,
    },
  };
}

export function buildPortalRechargeSummaryCards(input: {
  quote: PortalCommerceQuote | null;
  rechargeOptions: PortalRechargeOption[];
  summary: ProjectBillingSummary | null;
  orders: PortalCommerceOrder[];
  t: TranslateFn;
}): PortalRechargeSummaryCard[] {
  const { orders, quote, rechargeOptions, summary, t } = input;
  const recommendedOption = rechargeOptions.find((option) => option.recommended) ?? rechargeOptions[0];
  const balanceValue =
    summary?.remaining_units === null || summary?.remaining_units === undefined
      ? t('Unlimited')
      : formatUnits(summary.remaining_units);
  const rechargeOrders = orders.filter((order) => (
    order.target_kind === 'custom_recharge' || order.target_kind === 'recharge_pack'
  ));

  return [
    {
      label: t('Balance'),
      value: balanceValue,
      detail: t('Current available balance before the next quota guardrail is reached.'),
    },
    {
      label: t('Effective ratio'),
      value: quote?.effective_ratio_label ?? recommendedOption?.effective_ratio_label ?? t('n/a'),
      detail: t('Recharge pricing is quoted from the server-managed recharge policy.'),
    },
    {
      label: t('Recharge orders'),
      value: formatUnits(rechargeOrders.length),
      detail: t('Recent top-up orders remain visible together with custom recharge history.'),
    },
  ];
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
  orders: PortalCommerceOrder[],
): PortalCommerceOrder[] {
  return orders
    .filter((order) => order.target_kind === 'custom_recharge' || order.target_kind === 'recharge_pack')
    .slice()
    .sort((left, right) => right.created_at_ms - left.created_at_ms);
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
