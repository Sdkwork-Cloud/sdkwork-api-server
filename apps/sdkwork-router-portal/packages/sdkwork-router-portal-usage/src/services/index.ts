import type { GatewayApiKeyRecord, UsageRecord } from 'sdkwork-router-portal-types';

import type {
  BuildPortalUsageViewModelInput,
  PortalUsageViewModel,
  UsageFilterOption,
  UsageFilters,
  UsageTableRow,
  UsageTimeRange,
} from '../types';

function compareUsageRecords(left: UsageRecord, right: UsageRecord): number {
  return (
    right.created_at_ms - left.created_at_ms
    || left.model.localeCompare(right.model)
    || left.provider.localeCompare(right.provider)
  );
}

function timeRangeCutoff(timeRange: UsageTimeRange): number | null {
  const now = Date.now();

  switch (timeRange) {
    case '24h':
      return now - 24 * 60 * 60 * 1000;
    case '7d':
      return now - 7 * 24 * 60 * 60 * 1000;
    case '30d':
      return now - 30 * 24 * 60 * 60 * 1000;
    case 'all':
      return null;
  }
}

function sortedUnique(values: string[]): string[] {
  return [...new Set(values.filter((value) => value.trim().length > 0))].sort((left, right) =>
    left.localeCompare(right),
  );
}

function shortenHash(value: string): string {
  if (value.length <= 12) {
    return value;
  }

  return `${value.slice(0, 6)}...${value.slice(-4)}`;
}

function buildApiKeyOption(
  hashedKey: string,
  apiKeyByHash: Map<string, GatewayApiKeyRecord>,
): UsageFilterOption {
  const record = apiKeyByHash.get(hashedKey);
  const label = record?.label?.trim() || shortenHash(hashedKey);

  return {
    value: hashedKey,
    label,
  };
}

function buildUsageTableRow(
  record: UsageRecord,
  apiKeyByHash: Map<string, GatewayApiKeyRecord>,
): UsageTableRow {
  const apiKeyHash = record.api_key_hash?.trim() ?? '';
  const apiKeyLabel = apiKeyHash
    ? buildApiKeyOption(apiKeyHash, apiKeyByHash).label
    : 'Workspace token';
  const channelLabel = record.channel_id?.trim() || 'Unassigned';

  return {
    ...record,
    api_key_label: apiKeyLabel,
    channel_label: channelLabel,
    latency_ms: record.latency_ms ?? null,
    reference_amount: record.reference_amount ?? record.amount,
  };
}

function matchesUsageFilters(record: UsageRecord, filters: UsageFilters): boolean {
  const cutoff = timeRangeCutoff(filters.time_range);
  const apiKeyFilter = filters.api_key_hash.trim();
  const channelFilter = filters.channel_id.trim();
  const modelFilter = filters.model.trim();

  if (apiKeyFilter && apiKeyFilter !== 'all' && (record.api_key_hash ?? '') !== apiKeyFilter) {
    return false;
  }

  if (channelFilter && channelFilter !== 'all' && (record.channel_id ?? '') !== channelFilter) {
    return false;
  }

  if (modelFilter && modelFilter !== 'all' && record.model !== modelFilter) {
    return false;
  }

  if (cutoff !== null && record.created_at_ms < cutoff) {
    return false;
  }

  return true;
}

export function buildPortalUsageViewModel(
  input: BuildPortalUsageViewModelInput,
): PortalUsageViewModel {
  const { apiKeys, filters, page, page_size, records } = input;
  const apiKeyByHash = new Map(apiKeys.map((record) => [record.hashed_key, record]));
  const allApiKeyHashes = sortedUnique([
    ...apiKeys.map((record) => record.hashed_key),
    ...records.map((record) => record.api_key_hash ?? ''),
  ]);
  const channels = sortedUnique(records.map((record) => record.channel_id ?? ''));
  const models = sortedUnique(records.map((record) => record.model));
  const filtered_records = records
    .filter((record) => matchesUsageFilters(record, filters))
    .sort(compareUsageRecords)
    .map((record) => buildUsageTableRow(record, apiKeyByHash));
  const total_items = filtered_records.length;
  const total_pages = Math.max(1, Math.ceil(total_items / page_size));
  const safe_page = Math.min(Math.max(page, 1), total_pages);
  const page_start = (safe_page - 1) * page_size;
  const rows = filtered_records.slice(page_start, page_start + page_size);
  const latencyValues = filtered_records
    .map((record) => record.latency_ms)
    .filter((value): value is number => value !== null && value > 0);

  return {
    summary: {
      total_requests: filtered_records.length,
      total_tokens: filtered_records.reduce((sum, record) => sum + record.total_tokens, 0),
      input_tokens: filtered_records.reduce((sum, record) => sum + record.input_tokens, 0),
      output_tokens: filtered_records.reduce((sum, record) => sum + record.output_tokens, 0),
      actual_amount: filtered_records.reduce((sum, record) => sum + record.amount, 0),
      reference_amount: filtered_records.reduce(
        (sum, record) => sum + record.reference_amount,
        0,
      ),
      average_latency_ms: latencyValues.length
        ? Math.round(
            latencyValues.reduce((sum, value) => sum + value, 0) / latencyValues.length,
          )
        : null,
    },
    filter_options: {
      api_keys: [
        { value: 'all', label: 'All API keys' },
        ...allApiKeyHashes.map((hashedKey) => buildApiKeyOption(hashedKey, apiKeyByHash)),
      ],
      channels: ['all', ...channels],
      models: ['all', ...models],
    },
    filtered_records,
    rows,
    pagination: {
      page: safe_page,
      page_size,
      total_items,
      total_pages,
    },
  };
}
