import { formatUnits } from 'sdkwork-router-portal-commons/format-core';
import type {
  PortalCommerceCoupon,
  PortalCommerceQuote,
  ProjectBillingSummary,
} from 'sdkwork-router-portal-types';

import type { CouponImpactPreview, CreditsGuardrail, RecommendedCouponOffer } from '../types';

function sortCouponsByImpact(coupons: PortalCommerceCoupon[]): PortalCommerceCoupon[] {
  return coupons
    .slice()
    .sort(
      (left, right) =>
        right.bonus_units - left.bonus_units || left.code.localeCompare(right.code),
    );
}

export function recommendCouponOffer(
  summary: ProjectBillingSummary,
  coupons: PortalCommerceCoupon[],
): PortalCommerceCoupon | null {
  if (!coupons.length) {
    return null;
  }

  const rankedCoupons = sortCouponsByImpact(coupons);

  if (summary.exhausted || (summary.remaining_units ?? 0) < 5_000) {
    return rankedCoupons[0];
  }

  if (summary.used_units === 0) {
    return coupons.find((coupon) => coupon.code === 'WELCOME100') ?? rankedCoupons[0];
  }

  return coupons.find((coupon) => coupon.code === 'TEAMREADY') ?? rankedCoupons[0];
}

export function buildCouponImpactPreview(
  coupon: PortalCommerceCoupon,
  quote: PortalCommerceQuote,
): CouponImpactPreview {
  const projected_remaining_units =
    quote.projected_remaining_units === null || quote.projected_remaining_units === undefined
      ? null
      : quote.projected_remaining_units;

  return {
    coupon,
    quote,
    status:
      projected_remaining_units === null
        ? `${coupon.code} would add ${formatUnits(quote.bonus_units)} bonus units on top of an unlimited quota posture.`
        : `${coupon.code} would increase visible remaining units to ${formatUnits(projected_remaining_units)}.`,
  };
}

export function buildRecommendedCouponOffer(
  summary: ProjectBillingSummary,
  coupons: PortalCommerceCoupon[],
  quote: PortalCommerceQuote,
): RecommendedCouponOffer | null {
  const offer = recommendCouponOffer(summary, coupons);
  if (!offer) {
    return null;
  }

  let rationale = 'This offer is the cleanest fit for the current workspace posture.';

  if (summary.exhausted) {
    rationale =
      'Quota is exhausted, so the portal recommends the highest-impact coupon path before the next launch window.';
  } else if ((summary.remaining_units ?? 0) < 5_000) {
    rationale =
      'Remaining points are low, so the recommended offer prioritizes restoring a safer launch buffer.';
  } else if (summary.used_units === 0) {
    rationale =
      'No usage has been recorded yet, so the portal recommends a first-run offer that lowers the cost of initial experimentation.';
  }

  return {
    offer,
    rationale,
    preview: buildCouponImpactPreview(offer, quote),
  };
}

export function buildRedemptionGuardrails(summary: ProjectBillingSummary): CreditsGuardrail[] {
  const guardrails: CreditsGuardrail[] = [];
  const remainingUnits = summary.remaining_units ?? 0;

  if (summary.exhausted) {
    guardrails.push({
      id: 'restore-before-launch',
      title: 'Restore quota before new traffic is scheduled',
      detail:
        'When visible quota is exhausted, redeeming a coupon should happen before the next test or production launch window.',
      tone: 'warning',
    });
  } else if (remainingUnits < 5_000) {
    guardrails.push({
      id: 'protect-buffer',
      title: 'Treat coupons as a launch buffer, not a surprise fix',
      detail: `Only ${formatUnits(remainingUnits)} units remain. Redeem before runway becomes operationally tight.`,
      tone: 'warning',
    });
  } else {
    guardrails.push({
      id: 'redeem-with-intent',
      title: 'Redeem with a clear demand event in mind',
      detail:
        'Current quota posture is healthy, so coupon usage should align with an onboarding push, load test, or growth moment.',
      tone: 'positive',
    });
  }

  if (summary.used_units === 0) {
    guardrails.push({
      id: 'pair-with-first-request',
      title: 'Pair the first coupon with the first real request',
      detail:
        'The cleanest user experience is to unlock telemetry and bonus points together in the first launch path.',
      tone: 'accent',
    });
  } else {
    guardrails.push({
      id: 'watch-burn-rate',
      title: 'Match redemption to observed consumption',
      detail: `The workspace has already consumed ${formatUnits(summary.used_units)} units, so coupon choice should follow actual burn instead of guesswork.`,
      tone: 'accent',
    });
  }

  guardrails.push({
    id: 'checkout-boundary',
    title: 'Escalate to billing when coupons stop being enough',
    detail:
      'Coupons are a controlled top-up path. Persistent growth should move into recharge packs or subscriptions instead of repeated redemption.',
    tone: 'default',
  });

  return guardrails;
}
