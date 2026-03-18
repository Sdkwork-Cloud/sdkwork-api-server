import type {
  PortalDashboardSummary,
  PortalRouteKey,
  PortalRoutingDecisionLog,
  PortalRoutingSummary,
  UsageRecord,
} from 'sdkwork-router-portal-types';

export type DashboardTone = 'accent' | 'positive' | 'warning' | 'default';

export interface DashboardInsight {
  id: string;
  title: string;
  detail: string;
  tone: DashboardTone;
  route?: PortalRouteKey;
  action_label?: string;
}

export interface DashboardMetric {
  id: string;
  label: string;
  value: string;
  detail: string;
}

export interface DashboardRoutingPosture {
  title: string;
  detail: string;
  strategy_label: string;
  selected_provider: string;
  preferred_region: string;
  evidence_count: string;
  latest_reason: string;
  tone: DashboardTone;
  route: PortalRouteKey;
  action_label: string;
}

export interface DashboardBreakdownItem {
  id: string;
  label: string;
  secondary_label: string;
  value_label: string;
  share: number;
}

export interface DashboardSeriesPoint {
  bucket: string;
  requests: number;
  amount: number;
}

export interface DashboardDistributionPoint {
  name: string;
  value: number;
}

export interface DashboardDemandPoint {
  name: string;
  requests: number;
}

export interface DashboardActivityItem {
  id: string;
  title: string;
  detail: string;
  timestamp_label: string;
  tone: DashboardTone;
  route?: PortalRouteKey;
  action_label?: string;
}

export interface DashboardModuleItem {
  route: PortalRouteKey;
  title: string;
  status_label: string;
  detail: string;
  tone: DashboardTone;
  action_label: string;
}

export interface PortalDashboardSnapshotBundle {
  dashboard: PortalDashboardSummary;
  routing_summary: PortalRoutingSummary;
  routing_logs: PortalRoutingDecisionLog[];
  usage_records: UsageRecord[];
}

export interface PortalDashboardPageProps {
  onNavigate: (route: PortalRouteKey) => void;
  initialSnapshot?: PortalDashboardSummary | null;
}

export interface PortalDashboardPageViewModel {
  snapshot: PortalDashboardSummary;
  insights: DashboardInsight[];
  metrics: DashboardMetric[];
  routing_posture: DashboardRoutingPosture | null;
  quick_actions: DashboardInsight[];
  provider_mix: DashboardBreakdownItem[];
  model_mix: DashboardBreakdownItem[];
  request_volume_series: DashboardSeriesPoint[];
  spend_series: DashboardSeriesPoint[];
  provider_share_series: DashboardDistributionPoint[];
  model_demand_series: DashboardDemandPoint[];
  activity_feed: DashboardActivityItem[];
  modules: DashboardModuleItem[];
}
