import { formatUnits } from 'sdkwork-router-portal-commons/format-core';
import type {
  ProjectBillingSummary,
  RechargePack,
  SubscriptionPlan,
  UsageRecord,
} from 'sdkwork-router-portal-types';

import type { BillingRecommendation } from '../types';

function buildDailyUsageSeries(usageRecords: UsageRecord[]): number[] {
  const daily = new Map<string, number>();

  for (const record of usageRecords) {
    if (!record.created_at_ms) {
      continue;
    }

    const key = new Date(record.created_at_ms).toISOString().slice(0, 10);
    daily.set(key, (daily.get(key) ?? 0) + record.units);
  }

  return [...daily.entries()]
    .sort((left, right) => left[0].localeCompare(right[0]))
    .map(([, units]) => units);
}

function exponentialMovingAverage(values: number[], alpha = 0.45): number | null {
  if (!values.length) {
    return null;
  }

  let smoothed = values[0];
  for (let index = 1; index < values.length; index += 1) {
    smoothed = alpha * values[index] + (1 - alpha) * smoothed;
  }

  return smoothed;
}

function estimateDailyUnits(
  summary: ProjectBillingSummary,
  usageRecords: UsageRecord[],
): number | null {
  const smoothedDailyUnits = exponentialMovingAverage(buildDailyUsageSeries(usageRecords));
  if (smoothedDailyUnits && Number.isFinite(smoothedDailyUnits)) {
    return Math.max(1, Math.round(smoothedDailyUnits));
  }

  if (summary.used_units <= 0) {
    return null;
  }

  return Math.max(1, Math.ceil(summary.used_units / 30));
}

function buildRunway(
  summary: ProjectBillingSummary,
  usageRecords: UsageRecord[],
): BillingRecommendation['runway'] {
  const daily_units = estimateDailyUnits(summary, usageRecords);

  if (summary.exhausted) {
    return {
      label: '0 days',
      detail: 'Visible quota is already exhausted, so the workspace needs an immediate recharge or plan change before additional traffic is expected.',
      projected_days: 0,
      daily_units,
    };
  }

  if (summary.remaining_units === null || summary.remaining_units === undefined) {
    return {
      label: 'Unlimited',
      detail: 'The current billing summary exposes no visible quota ceiling, so the portal treats runway as unlimited for this workspace.',
      projected_days: null,
      daily_units,
    };
  }

  if (!daily_units) {
    return {
      label: 'Needs first traffic signal',
      detail: 'There is not enough recorded usage yet to project a meaningful burn pace. Send live traffic, then revisit billing decisions.',
      projected_days: null,
      daily_units: null,
    };
  }

  const projected_days = Math.floor(summary.remaining_units / daily_units);
  const label = projected_days < 1 ? '< 1 day' : `${projected_days} days`;

  return {
    label,
    detail: `Estimated from an exponentially smoothed burn pace of ${formatUnits(daily_units)} token units per day.`,
    projected_days,
    daily_units,
  };
}

function buildRecommendedBundle(
  summary: ProjectBillingSummary,
  plan: SubscriptionPlan | null,
  pack: RechargePack | null,
): BillingRecommendation['bundle'] {
  if (!plan && !pack) {
    return {
      title: 'Billing catalog unavailable',
      detail: 'The portal could not build a plan-plus-pack recommendation from the current seed catalog.',
    };
  }

  if (summary.exhausted) {
    return {
      title: `${plan?.name ?? 'Subscription'} + ${pack?.label ?? 'Recharge pack'}`,
      detail: 'The workspace needs both immediate runway recovery and a steadier monthly posture, so the portal recommends a plan and a recharge together.',
    };
  }

  if ((summary.remaining_units ?? 0) < 10_000) {
    return {
      title: `${plan?.name ?? 'Subscription'} with ${pack?.label ?? 'Recharge pack'} as buffer`,
      detail: 'Current quota is still active, but remaining headroom is tight enough that a plan-plus-buffer path is the lowest-friction next move.',
    };
  }

  return {
    title: `${plan?.name ?? 'Subscription'} as the next growth step`,
    detail: 'The workspace is stable today, so the recommended bundle focuses on the cleanest subscription path while keeping the top-up pack available only if demand spikes.',
  };
}

export function recommendBillingChange(
  summary: ProjectBillingSummary,
  plans: SubscriptionPlan[],
  packs: RechargePack[],
  usageRecords: UsageRecord[] = [],
): BillingRecommendation {
  const runway = buildRunway(summary, usageRecords);
  const projectedMonthlyUnits = runway.daily_units
    ? runway.daily_units * 30
    : Math.max(summary.used_units, 1);
  const recommendedPlan = plans.length
    ? (plans.find((plan) => plan.included_units >= projectedMonthlyUnits) ?? plans[plans.length - 1])
    : null;
  const recommendedPack = packs.length
    ? (packs.find((pack) => pack.points >= Math.max(10_000, Math.round(projectedMonthlyUnits / 4))) ??
      packs[packs.length - 1])
    : null;
  const bundle = buildRecommendedBundle(summary, recommendedPlan, recommendedPack);

  if (summary.exhausted && recommendedPlan && recommendedPack) {
    return {
      title: 'Quota is exhausted',
      detail: `Move to ${recommendedPlan.name} or add ${recommendedPack.label} to restore headroom immediately.`,
      plan: recommendedPlan,
      pack: recommendedPack,
      runway,
      bundle,
    };
  }

  if ((summary.remaining_units ?? 0) < 10_000 && recommendedPlan && recommendedPack) {
    return {
      title: 'Headroom is getting tight',
      detail: `Add ${recommendedPack.label} for near-term coverage, or move to ${recommendedPlan.name} for a steadier monthly posture.`,
      plan: recommendedPlan,
      pack: recommendedPack,
      runway,
      bundle,
    };
  }

  return {
    title: recommendedPlan ? 'Current workspace is stable' : 'Billing catalog unavailable',
    detail: recommendedPlan
      ? `Based on a projected monthly demand of ${formatUnits(projectedMonthlyUnits)} units, ${recommendedPlan.name} is the cleanest next subscription step when traffic grows.`
      : 'The portal could not load a live commerce catalog for this workspace.',
    plan: recommendedPlan,
    pack: recommendedPack,
    runway,
    bundle,
  };
}

export function isRecommendedPlan(
  plan: SubscriptionPlan,
  recommendation: BillingRecommendation,
): boolean {
  return recommendation.plan?.id === plan.id;
}

export function isRecommendedPack(
  pack: RechargePack,
  recommendation: BillingRecommendation,
): boolean {
  return recommendation.pack?.id === pack.id;
}
