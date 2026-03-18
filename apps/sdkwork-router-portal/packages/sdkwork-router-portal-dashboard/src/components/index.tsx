import { EmptyState, InlineButton, Pill } from 'sdkwork-router-portal-commons';
import type { PortalRouteKey } from 'sdkwork-router-portal-types';

import type { DashboardBreakdownItem, DashboardInsight } from '../types';

export function DashboardInsights({
  insights,
  onNavigate,
}: {
  insights: DashboardInsight[];
  onNavigate: (route: PortalRouteKey) => void;
}) {
  return (
    <div className="portalx-insight-grid">
      {insights.map((insight) => (
        <article className="portalx-insight-card" key={insight.id}>
          <Pill tone={insight.tone}>{insight.title}</Pill>
          <p>{insight.detail}</p>
          {insight.route && insight.action_label ? (
            <InlineButton onClick={() => onNavigate(insight.route!)} tone="ghost">
              {insight.action_label}
            </InlineButton>
          ) : null}
        </article>
      ))}
    </div>
  );
}

export function DashboardBreakdownList({
  items,
  emptyTitle,
  emptyDetail,
}: {
  items: DashboardBreakdownItem[];
  emptyTitle: string;
  emptyDetail: string;
}) {
  if (!items.length) {
    return <EmptyState detail={emptyDetail} title={emptyTitle} />;
  }

  return (
    <div className="portalx-dashboard-breakdown-list">
      {items.map((item) => (
        <article className="portalx-dashboard-breakdown-row" key={item.id}>
          <div className="portalx-dashboard-breakdown-meta">
            <div>
              <strong>{item.label}</strong>
              <span>{item.secondary_label}</span>
            </div>
            <strong>{item.value_label}</strong>
          </div>
          <div className="portalx-dashboard-breakdown-track">
            <span className="portalx-dashboard-breakdown-fill" style={{ width: `${Math.min(item.share, 100)}%` }} />
          </div>
        </article>
      ))}
    </div>
  );
}
