import {
  getPortalDashboard,
  getPortalRoutingSummary,
  listPortalUsageRecords,
  listPortalRoutingDecisionLogs,
} from 'sdkwork-router-portal-portal-api';
import type { PortalDashboardSummary } from 'sdkwork-router-portal-types';

import type { PortalDashboardSnapshotBundle } from '../types';

export async function loadPortalDashboardSnapshot(
  initialDashboard?: PortalDashboardSummary | null,
): Promise<PortalDashboardSnapshotBundle> {
  const [dashboard, routing_summary, routing_logs, usage_records] = await Promise.all([
    initialDashboard ? Promise.resolve(initialDashboard) : getPortalDashboard(),
    getPortalRoutingSummary(),
    listPortalRoutingDecisionLogs(),
    listPortalUsageRecords(),
  ]);

  return {
    dashboard,
    routing_summary,
    routing_logs,
    usage_records,
  };
}
