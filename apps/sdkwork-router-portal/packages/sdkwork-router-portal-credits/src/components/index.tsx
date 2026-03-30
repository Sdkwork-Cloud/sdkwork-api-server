import { formatUnits, Pill } from 'sdkwork-router-portal-commons';

import type { CouponImpactPreview } from '../types';

export function CouponImpactCard({
  preview,
}: {
  preview: CouponImpactPreview;
}) {
  return (
    <div className="portalx-insight-card">
      <Pill tone="positive">Redemption impact</Pill>
      <strong>{preview.coupon.code}</strong>
      <p>{preview.status}</p>
      <span>{formatUnits(preview.quote.bonus_units)} bonus units</span>
    </div>
  );
}
