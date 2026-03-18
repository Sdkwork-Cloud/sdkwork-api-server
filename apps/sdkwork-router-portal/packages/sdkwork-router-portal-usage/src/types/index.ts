import type { PortalRouteKey, UsageRecord, UsageSummary } from 'sdkwork-router-portal-types';

export interface PortalUsagePageProps {
  onNavigate: (route: PortalRouteKey) => void;
}

export type UsageDateRange = '24h' | '7d' | '30d' | 'all';

export interface UsageFilters {
  model: string;
  provider: string;
  date_range: UsageDateRange;
}

export interface UsageHighlight {
  id: string;
  label: string;
  value: string;
  detail: string;
}

export interface UsageProfileItem {
  id: string;
  label: string;
  value: string;
  detail: string;
}

export interface UsageTrendPoint {
  bucket: string;
  requests: number;
  units: number;
  amount: number;
  input_tokens: number;
  output_tokens: number;
  total_tokens: number;
}

export interface UsageMixPoint {
  id: string;
  label: string;
  requests: number;
  units: number;
  amount: number;
  share: number;
}

export interface UsageDiagnostic {
  id: string;
  title: string;
  detail: string;
  tone: 'accent' | 'positive' | 'warning' | 'default';
}

export interface UsageWorkbenchViewModel {
  summary: UsageSummary;
  filtered_records: UsageRecord[];
  total_units: number;
  total_amount: number;
  model_options: string[];
  provider_options: string[];
  highlights: UsageHighlight[];
  traffic_profile: UsageProfileItem[];
  spend_watch: UsageProfileItem[];
  request_volume_series: UsageTrendPoint[];
  provider_mix: UsageMixPoint[];
  model_mix: UsageMixPoint[];
  diagnostics: UsageDiagnostic[];
}
