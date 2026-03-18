import { formatCurrency, formatDateTime, formatUnits } from 'sdkwork-router-portal-commons';
import type { UsageRecord, UsageSummary } from 'sdkwork-router-portal-types';

import type {
  UsageDateRange,
  UsageDiagnostic,
  UsageFilters,
  UsageHighlight,
  UsageMixPoint,
  UsageProfileItem,
  UsageTrendPoint,
  UsageWorkbenchViewModel,
} from '../types';

function sortedUnique(values: string[]): string[] {
  return [...new Set(values)].sort((left, right) => left.localeCompare(right));
}

function dateRangeCutoff(dateRange: UsageDateRange): number | null {
  const now = Date.now();

  switch (dateRange) {
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

function filteredUsageRecords(records: UsageRecord[], filters: UsageFilters): UsageRecord[] {
  const cutoff = dateRangeCutoff(filters.date_range);

  return records.filter((record) => {
    if (filters.model && record.model !== filters.model) {
      return false;
    }
    if (filters.provider && record.provider !== filters.provider) {
      return false;
    }
    if (cutoff !== null && record.created_at_ms < cutoff) {
      return false;
    }
    return true;
  });
}

function usageBucketKey(record: UsageRecord): string {
  if (!record.created_at_ms) {
    return 'pending';
  }

  return new Date(record.created_at_ms).toISOString().slice(0, 10);
}

function usageBucketLabel(record: UsageRecord): string {
  if (!record.created_at_ms) {
    return 'Pending';
  }

  return new Intl.DateTimeFormat('en-US', {
    month: 'short',
    day: 'numeric',
  }).format(new Date(record.created_at_ms));
}

function buildUsageTrend(records: UsageRecord[]): UsageTrendPoint[] {
  const trend = new Map<string, UsageTrendPoint>();
  const ordered = [...records].sort((left, right) => left.created_at_ms - right.created_at_ms);

  for (const record of ordered) {
    const key = usageBucketKey(record);
    const existing = trend.get(key) ?? {
      bucket: usageBucketLabel(record),
      requests: 0,
      units: 0,
      amount: 0,
      input_tokens: 0,
      output_tokens: 0,
      total_tokens: 0,
    };

    existing.requests += 1;
    existing.units += record.units;
    existing.amount += record.amount;
    existing.input_tokens += record.input_tokens;
    existing.output_tokens += record.output_tokens;
    existing.total_tokens += record.total_tokens;
    trend.set(key, existing);
  }

  return [...trend.values()];
}

function buildUsageMix(
  records: UsageRecord[],
  mode: 'provider' | 'model',
): UsageMixPoint[] {
  const mix = new Map<string, UsageMixPoint>();

  for (const record of records) {
    const label = mode === 'provider' ? record.provider : record.model;
    const existing = mix.get(label) ?? {
      id: `${mode}-${label}`,
      label,
      requests: 0,
      units: 0,
      amount: 0,
      share: 0,
    };

    existing.requests += 1;
    existing.units += record.units;
    existing.amount += record.amount;
    mix.set(label, existing);
  }

  const totalRequests = records.length || 1;

  return [...mix.values()]
    .map((entry) => ({
      ...entry,
      share: Math.round((entry.requests / totalRequests) * 100),
    }))
    .sort((left, right) => right.requests - left.requests)
    .slice(0, 6);
}

function calculateTimelineDays(records: UsageRecord[]): number | null {
  const timestamped = records
    .map((record) => record.created_at_ms)
    .filter((timestamp) => timestamp > 0)
    .sort((left, right) => left - right);

  if (!timestamped.length) {
    return null;
  }

  const earliest = timestamped[0];
  const latest = timestamped[timestamped.length - 1];
  return Math.max(1, Math.ceil((latest - earliest) / (24 * 60 * 60 * 1000)) + 1);
}

function calculateDailyBurn(records: UsageRecord[]): number | null {
  if (!records.length) {
    return null;
  }

  const totalUnits = records.reduce((sum, record) => sum + record.units, 0);
  const timelineDays = calculateTimelineDays(records);

  if (!timelineDays) {
    return null;
  }

  return Math.max(1, Math.round(totalUnits / timelineDays));
}

function buildUsageHighlights(records: UsageRecord[]): UsageHighlight[] {
  const totalUnits = records.reduce((sum, record) => sum + record.units, 0);
  const totalAmount = records.reduce((sum, record) => sum + record.amount, 0);
  const totalTokens = records.reduce((sum, record) => sum + record.total_tokens, 0);
  const heaviestRequest = [...records].sort((left, right) => right.units - left.units)[0];

  return [
    {
      id: 'filtered-requests',
      label: 'Filtered requests',
      value: formatUnits(records.length),
      detail: 'Requests visible after the current workspace filters are applied.',
    },
    {
      id: 'token-units',
      label: 'Token units',
      value: formatUnits(totalUnits),
      detail: 'Metered token-unit volume inside the current filtered slice.',
    },
    {
      id: 'total-tokens',
      label: 'Total tokens',
      value: formatUnits(totalTokens),
      detail: 'Raw input plus output token count returned by the live usage records.',
    },
    {
      id: 'booked',
      label: 'Booked amount',
      value: formatCurrency(totalAmount),
      detail: heaviestRequest
        ? `Largest visible request: ${formatUnits(heaviestRequest.units)} units via ${heaviestRequest.provider}.`
        : 'Booked spend stays empty until the first request lands.',
    },
  ];
}

function buildTrafficProfile(
  records: UsageRecord[],
  providerMix: UsageMixPoint[],
  modelMix: UsageMixPoint[],
): UsageProfileItem[] {
  const latestRequest = [...records].sort((left, right) => right.created_at_ms - left.created_at_ms)[0];

  return [
    {
      id: 'primary-provider',
      label: 'Primary provider',
      value: providerMix[0]?.label ?? 'Waiting for traffic',
      detail: providerMix[0]
        ? `${providerMix[0].share}% of the visible request slice currently routes through this provider path.`
        : 'Provider preference appears after the first request lands.',
    },
    {
      id: 'primary-model',
      label: 'Primary model',
      value: modelMix[0]?.label ?? 'Waiting for traffic',
      detail: modelMix[0]
        ? `${modelMix[0].share}% of the visible request slice currently targets this model.`
        : 'Model concentration appears once request telemetry is present.',
    },
    {
      id: 'latest-request',
      label: 'Latest request',
      value: latestRequest ? formatDateTime(latestRequest.created_at_ms) : 'Pending',
      detail: latestRequest
        ? `${latestRequest.model} via ${latestRequest.provider}.`
        : 'The latest-call marker appears after the first request is recorded.',
    },
  ];
}

function buildSpendWatch(records: UsageRecord[]): UsageProfileItem[] {
  const totalUnits = records.reduce((sum, record) => sum + record.units, 0);
  const totalAmount = records.reduce((sum, record) => sum + record.amount, 0);
  const dailyBurn = calculateDailyBurn(records);
  const averageAmount = records.length ? totalAmount / records.length : 0;
  const peakRequest = [...records].sort((left, right) => right.amount - left.amount)[0];

  return [
    {
      id: 'booked-amount',
      label: 'Visible booked amount',
      value: formatCurrency(totalAmount),
      detail: 'Booked amount associated with the currently filtered request slice.',
    },
    {
      id: 'avg-booked',
      label: 'Average booked per request',
      value: formatCurrency(averageAmount),
      detail: 'A quick proxy for how expensive each visible request currently is.',
    },
    {
      id: 'daily-burn',
      label: 'Observed burn pace',
      value: dailyBurn === null ? 'Needs more data' : `${formatUnits(dailyBurn)} / day`,
      detail: dailyBurn === null
        ? 'The portal needs more timestamped usage records before it can infer daily burn.'
        : 'Derived from the visible usage timeline rather than a fixed 30-day assumption.',
    },
    {
      id: 'largest-request',
      label: 'Largest request cost',
      value: peakRequest ? formatCurrency(peakRequest.amount) : '$0.00',
      detail: peakRequest
        ? `${formatUnits(peakRequest.total_tokens)} total tokens in the most expensive visible request.`
        : 'Peak cost appears once traffic is recorded.',
    },
  ];
}

function normalizedConcentration(counts: number[]): number {
  if (!counts.length) {
    return 0;
  }

  if (counts.length === 1) {
    return 100;
  }

  const total = counts.reduce((sum, count) => sum + count, 0);
  if (!total) {
    return 0;
  }

  const hhi = counts.reduce((sum, count) => {
    const share = count / total;
    return sum + share * share;
  }, 0);
  const minimum = 1 / counts.length;
  return Math.round(((hhi - minimum) / (1 - minimum)) * 100);
}

function coefficientOfVariation(values: number[]): number {
  if (values.length <= 1) {
    return 0;
  }

  const mean = values.reduce((sum, value) => sum + value, 0) / values.length;
  if (!mean) {
    return 0;
  }

  const variance = values.reduce((sum, value) => sum + (value - mean) ** 2, 0) / values.length;
  return Math.sqrt(variance) / mean;
}

function buildDiagnostics(
  records: UsageRecord[],
  providerMix: UsageMixPoint[],
  modelMix: UsageMixPoint[],
  requestVolumeSeries: UsageTrendPoint[],
): UsageDiagnostic[] {
  if (!records.length) {
    return [
      {
        id: 'first-request',
        title: 'Send the first request to unlock diagnostics',
        detail: 'Traffic profile, spend watch, and anomaly signals all depend on the first visible request slice.',
        tone: 'accent',
      },
    ];
  }

  const diagnostics: UsageDiagnostic[] = [];
  const providerConcentration = normalizedConcentration(providerMix.map((item) => item.requests));
  const modelConcentration = normalizedConcentration(modelMix.map((item) => item.requests));
  const dailyVolatility = coefficientOfVariation(requestVolumeSeries.map((item) => item.units));
  const totalUnits = records.reduce((sum, record) => sum + record.units, 0);
  const averageUnits = totalUnits / records.length;
  const peakRequest = [...records].sort((left, right) => right.units - left.units)[0];

  if (providerConcentration >= 55 && providerMix[0]) {
    diagnostics.push({
      id: 'provider-concentration',
      title: 'Provider concentration is high',
      detail: `${providerMix[0].label} carries ${providerMix[0].share}% of the visible request slice. The portal flags concentration using a normalized HHI score of ${providerConcentration}.`,
      tone: 'warning',
    });
  }

  if (modelConcentration >= 55 && modelMix[0]) {
    diagnostics.push({
      id: 'model-concentration',
      title: 'Model mix is narrowly concentrated',
      detail: `${modelMix[0].label} dominates the active slice. The current model concentration index is ${modelConcentration}.`,
      tone: 'accent',
    });
  }

  if (peakRequest && peakRequest.units > averageUnits * 3) {
    diagnostics.push({
      id: 'token-spike',
      title: 'Token spikes are visible in the request mix',
      detail: `The heaviest visible request used ${formatUnits(peakRequest.units)} token units, more than 3x the current average. Investigate prompt or payload size before launch.`,
      tone: 'warning',
    });
  }

  if (dailyVolatility >= 0.85) {
    diagnostics.push({
      id: 'daily-volatility',
      title: 'Daily burn is volatile',
      detail: 'The recent usage buckets swing sharply from day to day, so billing decisions should leave more headroom than a steady-state workload would require.',
      tone: 'accent',
    });
  }

  if (!diagnostics.length) {
    diagnostics.push({
      id: 'healthy-slice',
      title: 'Current request slice looks stable',
      detail: 'No obvious provider concentration, token-spike, or day-to-day burn volatility stands out in the active workspace view.',
      tone: 'positive',
    });
  }

  return diagnostics;
}

export function buildUsageWorkbenchViewModel(
  summary: UsageSummary,
  records: UsageRecord[],
  filters: UsageFilters,
): UsageWorkbenchViewModel {
  const filtered_records = filteredUsageRecords(records, filters);
  const request_volume_series = buildUsageTrend(filtered_records);
  const provider_mix = buildUsageMix(filtered_records, 'provider');
  const model_mix = buildUsageMix(filtered_records, 'model');

  return {
    summary,
    filtered_records,
    total_units: filtered_records.reduce((sum, record) => sum + record.units, 0),
    total_amount: filtered_records.reduce((sum, record) => sum + record.amount, 0),
    model_options: sortedUnique(records.map((record) => record.model)),
    provider_options: sortedUnique(records.map((record) => record.provider)),
    highlights: buildUsageHighlights(filtered_records),
    traffic_profile: buildTrafficProfile(filtered_records, provider_mix, model_mix),
    spend_watch: buildSpendWatch(filtered_records),
    request_volume_series,
    provider_mix,
    model_mix,
    diagnostics: buildDiagnostics(filtered_records, provider_mix, model_mix, request_volume_series),
  };
}
