import { MetricCard } from 'sdkwork-router-portal-commons';

import type { ApiKeyEnvironmentSummary } from '../types';

export function ApiKeyEnvironmentSummaryGrid({
  summaries,
}: {
  summaries: ApiKeyEnvironmentSummary[];
}) {
  return (
    <div className="portalx-summary-grid">
      {summaries.map((summary) => (
        <MetricCard
          detail={`${summary.active} active key(s) currently visible for this environment.`}
          key={summary.environment}
          label={summary.environment}
          value={`${summary.total}`}
        />
      ))}
    </div>
  );
}
