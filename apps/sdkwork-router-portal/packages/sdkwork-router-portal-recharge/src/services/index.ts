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

const rechargeCurrencyFormatter = new Intl.NumberFormat('en-US', {
  style: 'currency',
  currency: 'USD',
  minimumFractionDigits: 2,
  maximumFractionDigits: 2,
});
const rechargeUnitsFormatter = new Intl.NumberFormat('en-US');
const FIXED_RECHARGE_PICKER_AMOUNTS = [
  1000,
  5000,
  10000,
  20000,
  50000,
  100000,
  200000,
  500000,
];

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

function resolveRechargeRuleSnapshot(
  amountCents: number,
  policy: PortalCustomRechargePolicy | null,
  presets: PortalRechargeOption[],
) {
  const matchedRule = policy?.rules.find(
    (rule) => amountCents >= rule.min_amount_cents && amountCents <= rule.max_amount_cents,
  );

  if (matchedRule) {
    return {
      grantedUnits: Math.round(amountCents * matchedRule.units_per_cent),
      effectiveRatioLabel: matchedRule.effective_ratio_label || 'n/a',
    };
  }

  const nearestPreset = presets
    .slice()
    .sort(
      (left, right) =>
        Math.abs(left.amount_cents - amountCents) - Math.abs(right.amount_cents - amountCents),
    )[0];

  if (!nearestPreset || nearestPreset.amount_cents <= 0) {
    return {
      grantedUnits: 0,
      effectiveRatioLabel: 'n/a',
    };
  }

  return {
    grantedUnits: Math.round(
      amountCents * (nearestPreset.granted_units / nearestPreset.amount_cents),
    ),
    effectiveRatioLabel: nearestPreset.effective_ratio_label,
  };
}

function resolveRecommendedPickerAmount(
  presets: PortalRechargeOption[],
  policy: PortalCustomRechargePolicy | null,
) {
  const suggestedAmount = policy?.suggested_amount_cents ?? null;
  if (suggestedAmount) {
    return FIXED_RECHARGE_PICKER_AMOUNTS.slice().sort(
      (left, right) => Math.abs(left - suggestedAmount) - Math.abs(right - suggestedAmount),
    )[0] ?? 50000;
  }

  const presetRecommendedAmount = presets.find((option) => option.recommended)?.amount_cents;
  if (presetRecommendedAmount && FIXED_RECHARGE_PICKER_AMOUNTS.includes(presetRecommendedAmount)) {
    return presetRecommendedAmount;
  }

  return 50000;
}

export function buildPortalRechargePickerOptions(
  options: PortalRechargeOption[] | null | undefined,
  policy: PortalCustomRechargePolicy | null,
): PortalRechargeOption[] {
  const presets = (options ?? [])
    .slice()
    .sort((left, right) => left.amount_cents - right.amount_cents);
  const recommendedAmount = resolveRecommendedPickerAmount(presets, policy);

  return FIXED_RECHARGE_PICKER_AMOUNTS.map((amountCents) => {
    const preset = presets.find((option) => option.amount_cents === amountCents);
    if (preset) {
      return {
        ...preset,
        recommended: amountCents === recommendedAmount,
      };
    }

    const derived = resolveRechargeRuleSnapshot(amountCents, policy, presets);
    return {
      id: `derived-${amountCents}`,
      label: formatCurrency(amountCents / 100),
      amount_cents: amountCents,
      amount_label: formatCurrency(amountCents / 100),
      granted_units: derived.grantedUnits,
      effective_ratio_label: derived.effectiveRatioLabel,
      note: '',
      recommended: amountCents === recommendedAmount,
      source: policy?.source ?? presets[0]?.source ?? 'live',
    };
  });
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
