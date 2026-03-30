import type {
  GatewayApiKeyRecord,
  PortalRouteKey,
  UsageRecord,
} from 'sdkwork-router-portal-types';

export interface PortalUsagePageProps {
  onNavigate: (route: PortalRouteKey) => void;
}

export type UsageTimeRange = '24h' | '7d' | '30d' | 'all';

export interface UsageFilters {
  api_key_hash: string;
  channel_id: string;
  model: string;
  time_range: UsageTimeRange;
}

export interface UsageSummarySnapshot {
  total_requests: number;
  total_tokens: number;
  input_tokens: number;
  output_tokens: number;
  actual_amount: number;
  reference_amount: number;
  average_latency_ms: number | null;
}

export interface UsageFilterOption {
  value: string;
  label: string;
}

export interface UsageFilterOptions {
  api_keys: UsageFilterOption[];
  channels: string[];
  models: string[];
}

export interface UsageTableRow extends UsageRecord {
  api_key_label: string;
  channel_label: string;
  latency_ms: number | null;
  reference_amount: number;
}

export interface UsagePaginationState {
  page: number;
  page_size: number;
  total_items: number;
  total_pages: number;
}

export interface PortalUsageViewModel {
  summary: UsageSummarySnapshot;
  filter_options: UsageFilterOptions;
  filtered_records: UsageTableRow[];
  rows: UsageTableRow[];
  pagination: UsagePaginationState;
}

export interface PortalUsageWorkbenchData {
  apiKeys: GatewayApiKeyRecord[];
  records: UsageRecord[];
}

export interface BuildPortalUsageViewModelInput {
  records: UsageRecord[];
  apiKeys: GatewayApiKeyRecord[];
  filters: UsageFilters;
  page: number;
  page_size: number;
}
