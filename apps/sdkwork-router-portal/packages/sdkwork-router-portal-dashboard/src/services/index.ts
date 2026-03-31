import {
  formatCurrency,
  formatDateTime,
  formatUnits,
} from 'sdkwork-router-portal-commons/format-core';
import type {
  PortalDashboardSummary,
  PortalRouteKey,
  PortalRoutingDecisionLog,
  PortalRoutingSummary,
  PortalRoutingStrategy,
  UsageRecord,
} from 'sdkwork-router-portal-types';

import type {
  DashboardActivityItem,
  DashboardBreakdownItem,
  DashboardDemandPoint,
  DashboardDistributionPoint,
  DashboardInsight,
  DashboardMetric,
  DashboardModuleItem,
  DashboardRoutingPosture,
  DashboardSeriesPoint,
  DashboardSpendTrendPoint,
  DashboardTrafficTrendPoint,
  DashboardTone,
  PortalDashboardPageViewModel,
} from '../types';

function safeArray<T>(value: T[] | null | undefined): T[] {
  return Array.isArray(value) ? value : [];
}

function normalizeDashboardSummary(snapshot: PortalDashboardSummary): PortalDashboardSummary {
  return {
    ...snapshot,
    usage_summary: {
      ...snapshot.usage_summary,
      projects: safeArray(snapshot.usage_summary.projects),
      providers: safeArray(snapshot.usage_summary.providers),
      models: safeArray(snapshot.usage_summary.models),
    },
    recent_requests: safeArray(snapshot.recent_requests),
  };
}

function normalizeRoutingSummary(
  routingSummary?: PortalRoutingSummary | null,
): PortalRoutingSummary | null {
  if (!routingSummary) {
    return null;
  }

  return {
    ...routingSummary,
    preferences: {
      ...routingSummary.preferences,
      ordered_provider_ids: safeArray(routingSummary.preferences.ordered_provider_ids),
    },
    preview: {
      ...routingSummary.preview,
      candidate_ids: safeArray(routingSummary.preview.candidate_ids),
      assessments: safeArray(routingSummary.preview.assessments),
    },
    provider_options: safeArray(routingSummary.provider_options),
  };
}

function normalizeRoutingLogs(logs?: PortalRoutingDecisionLog[] | null): PortalRoutingDecisionLog[] {
  return safeArray(logs).map((log) => ({
    ...log,
    assessments: safeArray(log.assessments),
  }));
}

function routingStrategyLabel(strategy?: PortalRoutingStrategy | string | null): string {
  switch (strategy) {
    case 'deterministic_priority':
      return 'Predictable order';
    case 'weighted_random':
      return 'Traffic distribution';
    case 'slo_aware':
      return 'Reliability guardrails';
    case 'geo_affinity':
      return 'Regional preference';
    default:
      return 'Adaptive routing';
  }
}

function remainingUnitsLabel(snapshot: PortalDashboardSummary): string {
  if (snapshot.billing_summary.remaining_units === null || snapshot.billing_summary.remaining_units === undefined) {
    return 'Unlimited';
  }

  return formatUnits(snapshot.billing_summary.remaining_units);
}

function insight(
  id: string,
  title: string,
  detail: string,
  tone: DashboardTone,
  route?: PortalRouteKey,
  action_label?: string,
): DashboardInsight {
  return {
    id,
    title,
    detail,
    tone,
    route,
    action_label,
  };
}

function buildInsights(
  snapshot: PortalDashboardSummary,
  routingSummary?: PortalRoutingSummary | null,
): DashboardInsight[] {
  const items: DashboardInsight[] = [];

  if (snapshot.billing_summary.exhausted) {
    items.push(
      insight(
        'quota-exhausted',
        'Quota exhausted',
        'Recharge or move to a higher plan before production traffic resumes.',
        'warning',
        'billing',
        'Review billing',
      ),
    );
  } else if ((snapshot.billing_summary.remaining_units ?? Number.POSITIVE_INFINITY) < 5_000) {
    items.push(
      insight(
        'quota-low',
        'Runway is getting tight',
        `Only ${remainingUnitsLabel(snapshot)} token units remain in the visible quota buffer.`,
        'warning',
        'credits',
        'Review credits',
      ),
    );
  }

  if (snapshot.api_key_count === 0) {
    items.push(
      insight(
        'missing-key',
        'API key setup incomplete',
        'Create at least one project key before inviting clients or teammates.',
        'warning',
        'api-keys',
        'Create key',
      ),
    );
  }

  if (snapshot.usage_summary.total_requests === 0) {
    items.push(
      insight(
        'missing-traffic',
        'Traffic overview is still empty',
        'Send the first request to unlock live model, provider, and spend telemetry.',
        'accent',
        'usage',
        'Open usage',
      ),
    );
  }

  if (routingSummary?.preview.slo_degraded) {
    items.push(
      insight(
        'routing-degraded',
        'Routing is protecting availability',
        'The current preview degraded from the preferred path. Review provider health and fallback behavior.',
        'warning',
        'routing',
        'Open routing',
      ),
    );
  }

  if (!items.length) {
    items.push(
      insight(
        'healthy-workspace',
        'Workspace is ready',
        'Traffic, access, routing, and quota posture are aligned for steady API usage.',
        'positive',
        'usage',
        'Open usage',
      ),
    );
  }

  return items.slice(0, 3);
}

function buildMetrics(
  snapshot: PortalDashboardSummary,
  routingSummary?: PortalRoutingSummary | null,
): DashboardMetric[] {
  return [
    {
      id: 'requests',
      label: 'Requests',
      value: formatUnits(snapshot.usage_summary.total_requests),
      detail: 'Completed gateway calls recorded in the current workspace.',
    },
    {
      id: 'booked-amount',
      label: 'Booked spend',
      value: formatCurrency(snapshot.billing_summary.booked_amount),
      detail: 'Total booked amount attached to the visible billing summary.',
    },
    {
      id: 'remaining-units',
      label: 'Remaining units',
      value: remainingUnitsLabel(snapshot),
      detail: 'Token-unit runway remaining before the visible quota ceiling is reached.',
    },
    {
      id: 'keys',
      label: 'API keys',
      value: formatUnits(snapshot.api_key_count),
      detail: 'Active key inventory visible inside this portal session.',
    },
    {
      id: 'providers',
      label: 'Providers',
      value: formatUnits(snapshot.usage_summary.provider_count),
      detail: 'Providers that served recent visible traffic.',
    },
    {
      id: 'route',
      label: 'Default route',
      value: routingSummary?.preview.selected_provider_id ?? 'Pending',
      detail: 'Provider currently selected by the routing preview.',
    },
  ];
}

function buildQuickActions(
  snapshot: PortalDashboardSummary,
  routingSummary?: PortalRoutingSummary | null,
): DashboardInsight[] {
  const actions: DashboardInsight[] = [];

  if (snapshot.api_key_count === 0) {
    actions.push(
      insight(
        'action-create-key',
        'Create the first API key',
        'Set up a scoped key so external clients can start sending traffic safely.',
        'warning',
        'api-keys',
        'Create key',
      ),
    );
  }

  if (snapshot.usage_summary.total_requests === 0) {
    actions.push(
      insight(
        'action-start-traffic',
        'Send the first API request',
        'The first real call will populate demand, cost, and provider telemetry across the portal.',
        'accent',
        'usage',
        'Open usage',
      ),
    );
  }

  if (snapshot.billing_summary.exhausted) {
    actions.push(
      insight(
        'action-recover-billing',
        'Restore quota before the next traffic window',
        'Billing recovery is the blocking action before more gateway requests can land.',
        'warning',
        'billing',
        'Review billing',
      ),
    );
  } else if ((snapshot.billing_summary.remaining_units ?? Number.POSITIVE_INFINITY) < 5_000) {
    actions.push(
      insight(
        'action-protect-runway',
        'Top up credits before runway gets tight',
        `Only ${remainingUnitsLabel(snapshot)} token units remain in the visible launch buffer.`,
        'warning',
        'credits',
        'Review credits',
      ),
    );
  }

  if (routingSummary) {
    actions.push(
      insight(
        'action-review-routing',
        'Review the active route',
        'Confirm provider order, fallback posture, and region preference before scaling traffic.',
        routingSummary.preview.slo_degraded ? 'warning' : 'default',
        'routing',
        'Open routing',
      ),
    );
  }

  if (!actions.length) {
    actions.push(
      insight(
        'action-scale',
        'Inspect live usage',
        'With the workspace in a healthy state, usage is the best place to validate growth before widening rollout.',
        'positive',
        'usage',
        'Open usage',
      ),
    );
  }

  return actions.slice(0, 4);
}

function buildRoutingPosture(
  routingSummary?: PortalRoutingSummary | null,
  routingLogs: PortalRoutingDecisionLog[] = [],
): DashboardRoutingPosture | null {
  if (!routingSummary) {
    return null;
  }

  const latestLog = [...routingLogs].sort((left, right) => right.created_at_ms - left.created_at_ms)[0];
  const strategyLabel = routingStrategyLabel(routingSummary.preferences.strategy);
  const preferredRegion = routingSummary.preferences.preferred_region
    ?? routingSummary.preview.requested_region
    ?? 'Global';

  if (routingSummary.preview.slo_degraded) {
    return {
      title: 'Fallback protection is active',
      detail: 'The current preview degraded from the preferred path. Review provider health and hard constraints.',
      strategy_label: strategyLabel,
      selected_provider: routingSummary.preview.selected_provider_id,
      preferred_region: preferredRegion,
      evidence_count: formatUnits(routingLogs.length),
      latest_reason:
        latestLog?.selection_reason
        ?? routingSummary.preview.selection_reason
        ?? 'A fallback provider was selected to protect availability.',
      tone: 'warning',
      route: 'routing',
      action_label: 'Open routing',
    };
  }

  if (!routingLogs.length) {
    return {
      title: 'Routing is configured and waiting for traffic',
      detail: 'The project has a default routing posture, but no recent evidence has been recorded yet.',
      strategy_label: strategyLabel,
      selected_provider: routingSummary.preview.selected_provider_id,
      preferred_region: preferredRegion,
      evidence_count: '0',
      latest_reason:
        routingSummary.preview.selection_reason
        ?? 'Run a preview or send live traffic to capture the first route decision.',
      tone: 'accent',
      route: 'routing',
      action_label: 'Run preview',
    };
  }

  return {
    title: 'Routing is healthy',
    detail: `The latest routing evidence selected ${latestLog?.selected_provider_id ?? routingSummary.preview.selected_provider_id}.`,
    strategy_label: strategyLabel,
    selected_provider: routingSummary.preview.selected_provider_id,
    preferred_region: preferredRegion,
    evidence_count: formatUnits(routingLogs.length),
    latest_reason:
      latestLog?.selection_reason
      ?? routingSummary.preview.selection_reason
      ?? 'Routing evidence is available for review in the routing workbench.',
    tone: 'positive',
    route: 'routing',
    action_label: 'Open routing',
  };
}

function buildProviderMix(snapshot: PortalDashboardSummary): DashboardBreakdownItem[] {
  const totalRequests = snapshot.usage_summary.total_requests || 1;

  return [...snapshot.usage_summary.providers]
    .sort((left, right) => right.request_count - left.request_count)
    .slice(0, 5)
    .map((provider) => ({
      id: provider.provider,
      label: provider.provider,
      secondary_label: `${formatUnits(provider.project_count)} project${provider.project_count === 1 ? '' : 's'}`,
      value_label: `${formatUnits(provider.request_count)} requests`,
      share: Math.max(6, Math.round((provider.request_count / totalRequests) * 100)),
    }));
}

function buildModelMix(snapshot: PortalDashboardSummary): DashboardBreakdownItem[] {
  const totalRequests = snapshot.usage_summary.total_requests || 1;

  return [...snapshot.usage_summary.models]
    .sort((left, right) => right.request_count - left.request_count)
    .slice(0, 5)
    .map((model) => ({
      id: model.model,
      label: model.model,
      secondary_label: `${formatUnits(model.provider_count)} provider${model.provider_count === 1 ? '' : 's'}`,
      value_label: `${formatUnits(model.request_count)} requests`,
      share: Math.max(6, Math.round((model.request_count / totalRequests) * 100)),
    }));
}

function seriesBucketLabel(timestamp: number): string {
  return new Intl.DateTimeFormat('en-US', {
    month: 'short',
    day: 'numeric',
  }).format(new Date(timestamp));
}

function seriesBucketKey(timestamp: number): string {
  return new Intl.DateTimeFormat('en-CA', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
  }).format(new Date(timestamp));
}

function buildTrafficTrendPoints(
  snapshot: PortalDashboardSummary,
  usageRecords: UsageRecord[],
): DashboardTrafficTrendPoint[] {
  const records = usageRecords.length ? usageRecords : snapshot.recent_requests;
  const grouped = new Map<string, DashboardTrafficTrendPoint>();

  for (const record of records) {
    const label = seriesBucketLabel(record.created_at_ms);
    const bucketKey = seriesBucketKey(record.created_at_ms);
    const current = grouped.get(bucketKey) ?? {
      label,
      bucket_key: bucketKey,
      request_count: 0,
      amount: 0,
      total_tokens: 0,
      input_tokens: 0,
      output_tokens: 0,
    };
    current.request_count += 1;
    current.amount += record.amount;
    current.total_tokens += record.total_tokens;
    current.input_tokens += record.input_tokens;
    current.output_tokens += record.output_tokens;
    grouped.set(bucketKey, current);
  }

  return [...grouped.values()]
    .sort((left, right) => left.bucket_key.localeCompare(right.bucket_key))
    .slice(-7);
}

function buildRequestVolumeSeries(
  snapshot: PortalDashboardSummary,
  usageRecords: UsageRecord[],
): DashboardSeriesPoint[] {
  return buildTrafficTrendPoints(snapshot, usageRecords).map((point) => ({
    bucket: point.label,
    requests: point.request_count,
    amount: Number(point.amount.toFixed(2)),
  }));
}

function buildSpendTrendPoints(
  snapshot: PortalDashboardSummary,
  usageRecords: UsageRecord[],
): DashboardSpendTrendPoint[] {
  return buildTrafficTrendPoints(snapshot, usageRecords).map((point) => ({
    label: point.label,
    bucket_key: point.bucket_key,
    amount: Number(point.amount.toFixed(2)),
    requests: point.request_count,
  }));
}

function buildSpendSeries(
  snapshot: PortalDashboardSummary,
  usageRecords: UsageRecord[],
): DashboardSeriesPoint[] {
  return buildSpendTrendPoints(snapshot, usageRecords).map((point) => ({
    bucket: point.label,
    requests: point.requests,
    amount: point.amount,
  }));
}

function buildProviderShareSeries(snapshot: PortalDashboardSummary): DashboardDistributionPoint[] {
  return [...snapshot.usage_summary.providers]
    .sort((left, right) => right.request_count - left.request_count)
    .slice(0, 5)
    .map((provider) => ({
      name: provider.provider,
      value: provider.request_count,
    }));
}

function buildModelDemandSeries(snapshot: PortalDashboardSummary): DashboardDemandPoint[] {
  return [...snapshot.usage_summary.models]
    .sort((left, right) => right.request_count - left.request_count)
    .slice(0, 5)
    .map((model) => ({
      name: model.model,
      requests: model.request_count,
    }));
}

function buildActivityFeed(
  snapshot: PortalDashboardSummary,
  routingLogs: PortalRoutingDecisionLog[] = [],
): DashboardActivityItem[] {
  const requestItems = snapshot.recent_requests.map((request) => ({
    id: `request-${request.project_id}-${request.created_at_ms}-${request.model}`,
    title: `${request.model} via ${request.provider}`,
    detail: `${formatUnits(request.units)} token units booked for ${formatCurrency(request.amount)}.`,
    timestamp_label: formatDateTime(request.created_at_ms),
    timestamp_ms: request.created_at_ms,
    tone: 'default' as DashboardTone,
    route: 'usage' as PortalRouteKey,
    action_label: 'Open usage',
  }));

  const routingItems = routingLogs.map((log) => ({
    id: `routing-${log.decision_id}`,
    title: `Route selected ${log.selected_provider_id}`,
    detail: `${routingStrategyLabel(log.strategy)}${log.selection_reason ? ` · ${log.selection_reason}` : ''}`,
    timestamp_label: formatDateTime(log.created_at_ms),
    timestamp_ms: log.created_at_ms,
    tone: log.slo_degraded ? ('warning' as DashboardTone) : ('positive' as DashboardTone),
    route: 'routing' as PortalRouteKey,
    action_label: 'Open routing',
  }));

  return [...requestItems, ...routingItems]
    .sort((left, right) => right.timestamp_ms - left.timestamp_ms)
    .slice(0, 6)
    .map(({ timestamp_ms: _timestampMs, ...item }) => item);
}

function buildModules(
  snapshot: PortalDashboardSummary,
  routingSummary?: PortalRoutingSummary | null,
): DashboardModuleItem[] {
  const creditsTone: DashboardTone = snapshot.billing_summary.exhausted
    ? 'warning'
    : (snapshot.billing_summary.remaining_units ?? Number.POSITIVE_INFINITY) < 5_000
      ? 'accent'
      : 'positive';

  return [
    {
      route: 'routing',
      title: 'Routing',
      status_label: routingSummary?.preview.slo_degraded ? 'Review' : 'Ready',
      detail: routingSummary
        ? `${routingStrategyLabel(routingSummary.preferences.strategy)} across ${formatUnits(routingSummary.provider_options.length)} providers.`
        : 'Routing preview data is still loading.',
      tone: routingSummary?.preview.slo_degraded ? 'warning' : 'positive',
      action_label: 'Open routing',
    },
    {
      route: 'api-keys',
      title: 'API Keys',
      status_label: snapshot.api_key_count > 0 ? 'Ready' : 'Setup',
      detail: snapshot.api_key_count > 0
        ? `${formatUnits(snapshot.api_key_count)} visible project keys.`
        : 'No project key is visible yet.',
      tone: snapshot.api_key_count > 0 ? 'positive' : 'warning',
      action_label: 'Manage keys',
    },
    {
      route: 'usage',
      title: 'Usage',
      status_label: snapshot.usage_summary.total_requests > 0 ? 'Live' : 'Quiet',
      detail: snapshot.usage_summary.total_requests > 0
        ? `${formatUnits(snapshot.usage_summary.total_requests)} requests across ${formatUnits(snapshot.usage_summary.model_count)} models.`
        : 'The first request will unlock live telemetry.',
      tone: snapshot.usage_summary.total_requests > 0 ? 'positive' : 'accent',
      action_label: 'Open usage',
    },
    {
      route: 'user',
      title: 'User',
      status_label: snapshot.workspace.user.active ? 'Healthy' : 'Review',
      detail: 'Personal profile, session identity, and security controls.',
      tone: snapshot.workspace.user.active ? 'positive' : 'warning',
      action_label: 'Open user',
    },
    {
      route: 'credits',
      title: 'Credits',
      status_label: snapshot.billing_summary.exhausted ? 'Exhausted' : creditsTone === 'accent' ? 'Watch' : 'Healthy',
      detail: snapshot.billing_summary.exhausted
        ? 'Quota is exhausted and requires immediate recovery.'
        : `${remainingUnitsLabel(snapshot)} token units remain in the visible balance.`,
      tone: creditsTone,
      action_label: 'Open credits',
    },
    {
      route: 'billing',
      title: 'Billing',
      status_label: snapshot.billing_summary.exhausted ? 'Action' : 'Ready',
      detail: `Current booked amount is ${formatCurrency(snapshot.billing_summary.booked_amount)}.`,
      tone: snapshot.billing_summary.exhausted ? 'warning' : 'default',
      action_label: 'Open billing',
    },
    {
      route: 'account',
      title: 'Account',
      status_label: snapshot.billing_summary.booked_amount > 0 ? 'Active' : 'Ready',
      detail: 'Cash balance, ledger visibility, and payment-side posture.',
      tone: snapshot.billing_summary.booked_amount > 0 ? 'default' : 'positive',
      action_label: 'Open account',
    },
  ];
}

export function buildPortalDashboardViewModel(
  snapshot: PortalDashboardSummary,
  routingSummary?: PortalRoutingSummary | null,
  routingLogs: PortalRoutingDecisionLog[] = [],
  usageRecords: UsageRecord[] = [],
): PortalDashboardPageViewModel {
  const normalizedSnapshot = normalizeDashboardSummary(snapshot);
  const normalizedRoutingSummary = normalizeRoutingSummary(routingSummary);
  const normalizedRoutingLogs = normalizeRoutingLogs(routingLogs);
  const normalizedUsageRecords = safeArray(usageRecords);
  const traffic_trend_points = buildTrafficTrendPoints(normalizedSnapshot, normalizedUsageRecords);
  const spend_trend_points = buildSpendTrendPoints(normalizedSnapshot, normalizedUsageRecords);

  return {
    snapshot: normalizedSnapshot,
    insights: buildInsights(normalizedSnapshot, normalizedRoutingSummary),
    metrics: buildMetrics(normalizedSnapshot, normalizedRoutingSummary),
    routing_posture: buildRoutingPosture(normalizedRoutingSummary, normalizedRoutingLogs),
    quick_actions: buildQuickActions(normalizedSnapshot, normalizedRoutingSummary),
    provider_mix: buildProviderMix(normalizedSnapshot),
    model_mix: buildModelMix(normalizedSnapshot),
    request_volume_series: buildRequestVolumeSeries(normalizedSnapshot, normalizedUsageRecords),
    spend_series: buildSpendSeries(normalizedSnapshot, normalizedUsageRecords),
    traffic_trend_points,
    spend_trend_points,
    provider_share_series: buildProviderShareSeries(normalizedSnapshot),
    model_demand_series: buildModelDemandSeries(normalizedSnapshot),
    activity_feed: buildActivityFeed(normalizedSnapshot, normalizedRoutingLogs),
    modules: buildModules(normalizedSnapshot, normalizedRoutingSummary),
  };
}
