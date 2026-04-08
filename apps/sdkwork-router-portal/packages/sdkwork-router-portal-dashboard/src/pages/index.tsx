import {
  Activity,
  RefreshCw,
  TriangleAlert,
} from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';
import type { ReactNode } from 'react';
import {
  formatCurrency,
  formatDateTime,
  formatUnits,
  usePortalI18n,
} from 'sdkwork-router-portal-commons';
import { Button } from 'sdkwork-router-portal-commons/framework/actions';
import {
  DataTable,
  StatusBadge,
} from 'sdkwork-router-portal-commons/framework/display';
import { EmptyState } from 'sdkwork-router-portal-commons/framework/feedback';
import { ManagementWorkbench } from 'sdkwork-router-portal-commons/framework/workbench';
import {
  WorkspacePanel,
} from 'sdkwork-router-portal-commons/framework/workspace';
import { portalErrorMessage } from 'sdkwork-router-portal-portal-api';

import {
  DashboardDistributionRingChart,
  DashboardBalanceCard,
  DashboardMetricCard,
  DashboardModelDistributionChart,
  DashboardRevenueTrendChart,
  DashboardTokenTrendChart,
} from '../components';
import { loadPortalDashboardSnapshot } from '../repository';
import { buildPortalDashboardViewModel } from '../services';
import type {
  DashboardInsight,
  DashboardStatusVariant,
  PortalDashboardPageProps,
  PortalDashboardPageViewModel,
} from '../types';

const chartPalette = [
  { dotClassName: 'bg-primary-500', sliceClassName: 'text-primary-500' },
  { dotClassName: 'bg-sky-500', sliceClassName: 'text-sky-500' },
  { dotClassName: 'bg-emerald-500', sliceClassName: 'text-emerald-500' },
  { dotClassName: 'bg-amber-500', sliceClassName: 'text-amber-500' },
  { dotClassName: 'bg-rose-500', sliceClassName: 'text-rose-500' },
  { dotClassName: 'bg-cyan-500', sliceClassName: 'text-cyan-500' },
];

type WorkbenchTab = 'requests' | 'routing' | 'modules' | 'actions';
type RequestFilter = 'all' | 'high_spend' | 'latest';

type DashboardWorkbenchRow = {
  id: string;
  primary: ReactNode;
  secondary: ReactNode;
  detail: ReactNode;
  status: ReactNode;
  action: ReactNode;
};

type DashboardWorkbenchConfig = {
  primaryLabel: string;
  secondaryLabel: string;
  detailLabel: string;
  statusLabel: string;
  actionLabel: string;
  emptyTitle: string;
  emptyDetail: string;
  rows: DashboardWorkbenchRow[];
};

type TranslateFn = (text: string, values?: Record<string, string | number>) => string;
type MetricBreakdown = { label: string; value: string };

const EMPTY_METRIC_SUMMARY = {
  revenue: 0,
  request_count: 0,
  used_units: 0,
  average_booked_spend: 0,
};

function clampPercentage(value: number): number {
  return Math.min(100, Math.max(0, value));
}

function formatAverageSpend(value: number, requestCount: number, t: TranslateFn): string {
  return requestCount > 0 ? formatCurrency(value) : t('n/a');
}

function buildMetricBreakdowns(
  t: TranslateFn,
  today: string,
  trailing7d: string,
  currentMonth: string,
): MetricBreakdown[] {
  return [
    { label: t('Today'), value: today },
    { label: t('7 days'), value: trailing7d },
    { label: t('This month'), value: currentMonth },
  ];
}

function DashboardAnalyticsPanel({
  actions,
  children,
  description,
  title,
}: {
  actions?: ReactNode;
  children: ReactNode;
  description: string;
  title: string;
}) {
  return (
    <WorkspacePanel actions={actions} description={description} title={title}>
      {children}
    </WorkspacePanel>
  );
}

function actionTone(index: number): 'primary' | 'secondary' | 'ghost' {
  if (index === 0) {
    return 'primary';
  }

  if (index === 1) {
    return 'secondary';
  }

  return 'ghost';
}

function renderStatusBadge(status: string, statusVariant: DashboardStatusVariant) {
  return <StatusBadge showIcon={false} status={status} variant={statusVariant} />;
}

function requestPosture(
  requestAmount: number,
  averageAmount: number,
  t: TranslateFn,
): {
  status_label: string;
  status_variant: DashboardStatusVariant;
} {
  if (requestAmount >= averageAmount * 1.35 && averageAmount > 0) {
    return {
      status_label: t('High spend'),
      status_variant: 'warning',
    };
  }

  if (requestAmount > 0) {
    return {
      status_label: t('Tracked'),
      status_variant: 'success',
    };
  }

  return {
    status_label: t('Pending'),
    status_variant: 'default',
  };
}

function buildActionCards(viewModel: PortalDashboardPageViewModel | null): DashboardInsight[] {
  if (!viewModel) {
    return [];
  }

  const deduped = new Map<string, DashboardInsight>();

  for (const item of [...viewModel.quick_actions, ...viewModel.insights]) {
    if (!deduped.has(item.id)) {
      deduped.set(item.id, item);
    }
  }

  return [...deduped.values()].slice(0, 4);
}

export function PortalDashboardPage({
  onNavigate,
  initialSnapshot,
}: PortalDashboardPageProps) {
  const { t } = usePortalI18n();
  const [activeWorkbenchTab, setActiveWorkbenchTab] =
    useState<WorkbenchTab>('requests');
  const [requestFilter, setRequestFilter] = useState<RequestFilter>('all');
  const [viewModel, setViewModel] = useState<PortalDashboardPageViewModel | null>(
    initialSnapshot ? buildPortalDashboardViewModel(initialSnapshot) : null,
  );
  const [status, setStatus] = useState(
    initialSnapshot
      ? t('Refreshing routing and activity data for the current workspace.')
      : t('Loading your workspace overview.'),
  );
  const [isLoading, setIsLoading] = useState(!initialSnapshot);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    setIsLoading(true);

    void loadPortalDashboardSnapshot(initialSnapshot)
      .then((snapshotBundle) => {
        if (cancelled) {
          return;
        }

        setViewModel(
          buildPortalDashboardViewModel(
            snapshotBundle.dashboard,
            snapshotBundle.routing_summary,
            snapshotBundle.routing_logs,
            snapshotBundle.usage_records,
            snapshotBundle.membership,
            Date.now(),
          ),
        );
        setError(null);
        setStatus(t('Live traffic, routing, and spend telemetry are up to date.'));
      })
      .catch((nextError) => {
        if (cancelled) {
          return;
        }

        const message = portalErrorMessage(nextError);
        setError(message);
        setStatus(message);
      })
      .finally(() => {
        if (!cancelled) {
          setIsLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [initialSnapshot, t]);

  const snapshot = viewModel?.snapshot ?? initialSnapshot ?? null;
  const routingEvidence = useMemo(
    () => (viewModel?.activity_feed ?? []).filter((item) => item.route === 'routing'),
    [viewModel],
  );
  const requestAverageAmount = useMemo(() => {
    if (!snapshot?.recent_requests.length) {
      return 0;
    }

    return (
      snapshot.recent_requests.reduce((sum, row) => sum + row.amount, 0) /
      snapshot.recent_requests.length
    );
  }, [snapshot]);
  const requestRows = useMemo(() => {
    if (!snapshot) {
      return [];
    }

    const rows = [...snapshot.recent_requests].sort(
      (left, right) => right.created_at_ms - left.created_at_ms,
    );

    if (requestFilter === 'latest') {
      return rows.slice(0, 6);
    }

    if (requestFilter === 'high_spend') {
      return rows
        .filter((row) => row.amount >= Math.max(requestAverageAmount, 0))
        .sort((left, right) => right.amount - left.amount);
    }

    return rows;
  }, [requestAverageAmount, requestFilter, snapshot]);
  const actionCards = useMemo(() => buildActionCards(viewModel), [viewModel]);
  const balanceSummary = useMemo(() => {
    if (viewModel) {
      return viewModel.balance;
    }

    if (!snapshot) {
      return {
        remaining_units: null,
        quota_limit_units: null,
        used_units: 0,
        utilization_ratio: null,
      };
    }

    const quotaLimitUnits = snapshot.billing_summary.quota_limit_units ?? null;
    const usedUnits = snapshot.billing_summary.used_units;

    return {
      remaining_units: snapshot.billing_summary.remaining_units ?? null,
      quota_limit_units: quotaLimitUnits,
      used_units: usedUnits,
      utilization_ratio:
        quotaLimitUnits && quotaLimitUnits > 0
          ? Math.min(1, Math.max(0, usedUnits / quotaLimitUnits))
          : null,
    };
  }, [snapshot, viewModel]);
  const totals = viewModel?.totals ?? {
    ...EMPTY_METRIC_SUMMARY,
    revenue: snapshot?.billing_summary.booked_amount ?? 0,
    request_count: snapshot?.usage_summary.total_requests ?? 0,
    used_units: snapshot?.billing_summary.used_units ?? 0,
    average_booked_spend:
      snapshot && (snapshot.usage_summary.total_requests ?? 0) > 0
        ? (snapshot.billing_summary.booked_amount ?? 0) / snapshot.usage_summary.total_requests
        : 0,
  };
  const todaySummary = viewModel?.today ?? EMPTY_METRIC_SUMMARY;
  const trailing7dSummary = viewModel?.trailing_7d ?? EMPTY_METRIC_SUMMARY;
  const currentMonthSummary = viewModel?.current_month ?? EMPTY_METRIC_SUMMARY;
  const workbenchConfig = useMemo<DashboardWorkbenchConfig>(() => {
    if (activeWorkbenchTab === 'routing') {
      return {
        primaryLabel: t('Signal'),
        secondaryLabel: t('Timeline'),
        detailLabel: t('Detail'),
        statusLabel: t('Status'),
        actionLabel: t('Action'),
        emptyTitle: t('Preparing routing evidence'),
        emptyDetail: t('Routing evidence will appear once project routing data becomes available.'),
        rows: routingEvidence.map((row) => ({
          id: row.id,
          primary: (
            <span className="font-semibold text-zinc-950 dark:text-zinc-50">{row.title}</span>
          ),
          secondary: row.timestamp_label,
          detail: row.detail,
          status: renderStatusBadge(row.status_label, row.status_variant),
          action:
            row.route && row.action_label ? (
              <Button onClick={() => onNavigate(row.route!)} variant="ghost">
                {row.action_label}
              </Button>
            ) : (
              '-'
            ),
        })),
      };
    }

    if (activeWorkbenchTab === 'modules') {
      return {
        primaryLabel: t('Module'),
        secondaryLabel: t('Status detail'),
        detailLabel: t('Operational detail'),
        statusLabel: t('Status'),
        actionLabel: t('Action'),
        emptyTitle: t('Preparing module posture'),
        emptyDetail: t('Module posture will appear after the dashboard finishes loading.'),
        rows: (viewModel?.modules ?? []).map((row) => ({
          id: row.route,
          primary: (
            <span className="font-semibold text-zinc-950 dark:text-zinc-50">{row.title}</span>
          ),
          secondary: row.status_label,
          detail: row.detail,
          status: renderStatusBadge(row.status_label, row.status_variant),
          action: (
            <Button onClick={() => onNavigate(row.route)} variant="ghost">
              {row.action_label}
            </Button>
          ),
        })),
      };
    }

    if (activeWorkbenchTab === 'actions') {
      return {
        primaryLabel: t('Next action'),
        secondaryLabel: t('Priority'),
        detailLabel: t('Action detail'),
        statusLabel: t('Status'),
        actionLabel: t('Action'),
        emptyTitle: t('Preparing next actions'),
        emptyDetail: t('Next actions will appear when workspace data finishes loading.'),
        rows: actionCards.map((item, index) => ({
          id: item.id,
          primary: (
            <span className="font-semibold text-zinc-950 dark:text-zinc-50">{item.title}</span>
          ),
          secondary: t('Priority #{priority}', { priority: index + 1 }),
          detail: item.detail,
          status: renderStatusBadge(item.status_label, item.status_variant),
          action:
            item.route && item.action_label ? (
              <Button onClick={() => onNavigate(item.route!)} variant={actionTone(index)}>
                {item.action_label}
              </Button>
            ) : (
              '-'
            ),
        })),
      };
    }

    return {
      primaryLabel: t('Request'),
      secondaryLabel: t('Provider'),
      detailLabel: t('Token detail'),
      statusLabel: t('Request posture'),
      actionLabel: t('Booked spend'),
      emptyTitle: t('No recent requests yet'),
      emptyDetail: t(
        'Once gateway requests start flowing through your project, token usage and booked amount will appear here.',
      ),
      rows: requestRows.map((row) => {
        const posture = requestPosture(row.amount, requestAverageAmount, t);

        return {
          id: `${row.project_id}-${row.model}-${row.created_at_ms}`,
          primary: (
            <div className="space-y-1">
              <span className="font-semibold text-zinc-950 dark:text-zinc-50">{row.model}</span>
              <p className="text-xs text-zinc-500 dark:text-zinc-400">
                {formatDateTime(row.created_at_ms)}
              </p>
            </div>
          ),
          secondary: row.provider,
          detail: t('{input} in / {output} out / {total} total', {
            input: formatUnits(row.input_tokens),
            output: formatUnits(row.output_tokens),
            total: formatUnits(row.total_tokens),
          }),
          status: renderStatusBadge(posture.status_label, posture.status_variant),
          action: (
            <span className="font-semibold text-zinc-950 dark:text-zinc-50">
              {formatCurrency(row.amount)}
            </span>
          ),
        };
      }),
    };
  }, [
    actionCards,
    activeWorkbenchTab,
    onNavigate,
    requestAverageAmount,
    requestRows,
    routingEvidence,
    t,
    viewModel,
  ]);
  const routingPosture = viewModel?.routing_posture ?? null;
  const requestFilterLabel =
    requestFilter === 'all'
      ? t('All requests')
      : requestFilter === 'high_spend'
        ? t('High spend')
        : t('Latest first');
  const workbenchTabs = [
    { id: 'requests' as const, label: t('Recent requests') },
    { id: 'routing' as const, label: t('Routing evidence') },
    { id: 'modules' as const, label: t('Module posture') },
    { id: 'actions' as const, label: t('Next actions') },
  ];
  const requestFilterOptions = [
    { id: 'all' as const, label: t('All requests') },
    { id: 'high_spend' as const, label: t('High spend') },
    { id: 'latest' as const, label: t('Latest first') },
  ];
  const workbenchTitle =
    activeWorkbenchTab === 'routing'
      ? t('Routing evidence')
      : activeWorkbenchTab === 'modules'
        ? t('Module posture')
        : activeWorkbenchTab === 'actions'
          ? t('Next actions')
          : t('Recent requests');
  const workbenchDescription =
    activeWorkbenchTab === 'routing'
      ? t('Review routing decisions, latest evidence, and the current provider strategy in one shared workbench.')
      : activeWorkbenchTab === 'modules'
        ? t('Compare module readiness and jump directly into the surface that needs attention.')
        : activeWorkbenchTab === 'actions'
          ? t('Promote the next operational actions from live traffic, billing, and routing signals.')
          : t('Inspect request flow, token usage, and booked spend without leaving the dashboard.');
  const balanceStatusLabel = snapshot?.billing_summary.exhausted ? t('Exhausted') : t('Healthy');
  const quotaRatio = balanceSummary.utilization_ratio === null
    ? null
    : clampPercentage(balanceSummary.utilization_ratio * 100);
  const balanceBreakdowns = buildMetricBreakdowns(
    t,
    formatUnits(todaySummary.used_units),
    formatUnits(trailing7dSummary.used_units),
    formatUnits(currentMonthSummary.used_units),
  );
  const revenueBreakdowns = buildMetricBreakdowns(
    t,
    formatCurrency(todaySummary.revenue),
    formatCurrency(trailing7dSummary.revenue),
    formatCurrency(currentMonthSummary.revenue),
  );
  const requestBreakdowns = buildMetricBreakdowns(
    t,
    formatUnits(todaySummary.request_count),
    formatUnits(trailing7dSummary.request_count),
    formatUnits(currentMonthSummary.request_count),
  );
  const averageSpendBreakdowns = buildMetricBreakdowns(
    t,
    formatAverageSpend(todaySummary.average_booked_spend, todaySummary.request_count, t),
    formatAverageSpend(trailing7dSummary.average_booked_spend, trailing7dSummary.request_count, t),
    formatAverageSpend(currentMonthSummary.average_booked_spend, currentMonthSummary.request_count, t),
  );

  if (error && !snapshot) {
    return (
      <div className="flex h-full items-center justify-center px-6 py-10">
        <div className="max-w-md rounded-[2rem] border border-white/70 bg-white/85 p-8 text-center shadow-[0_20px_60px_rgba(15,23,42,0.08)] backdrop-blur dark:border-white/6 dark:bg-zinc-900/85">
          <div className="mx-auto flex h-14 w-14 items-center justify-center rounded-full bg-rose-500/12 text-rose-500">
            <Activity className="h-6 w-6" />
          </div>
          <h1 className="mt-5 text-2xl font-semibold tracking-tight text-zinc-950 dark:text-zinc-50">
            {t('Dashboard could not be prepared')}
          </h1>
          <p className="mt-3 text-sm leading-6 text-zinc-500 dark:text-zinc-400">
            {error}
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="relative h-full overflow-y-auto">
      <div className="min-h-full px-4 py-5 sm:px-6 lg:px-8 xl:px-10">
        <div className="w-full space-y-6 xl:space-y-8">
          <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
            <DashboardBalanceCard
              balanceValue={
                balanceSummary.remaining_units === null
                  ? t('Unlimited')
                  : formatUnits(balanceSummary.remaining_units)
              }
              description={t('Available workspace balance before the visible quota guardrail is reached.')}
              onRecharge={() => onNavigate('recharge')}
              onRedeem={() => onNavigate('credits')}
              planValue={viewModel?.membership?.plan_name ?? t('No membership')}
              quotaLimitValue={
                balanceSummary.quota_limit_units === null
                  ? t('Unlimited')
                  : formatUnits(balanceSummary.quota_limit_units)
              }
              statusLabel={balanceStatusLabel}
              usedBreakdowns={balanceBreakdowns}
              usedUnitsValue={formatUnits(balanceSummary.used_units)}
              utilizationPercent={quotaRatio}
            />
            <DashboardMetricCard
              breakdowns={revenueBreakdowns}
              description={t('Booked revenue stays aligned with the latest routed demand across the same operating window.')}
              label={t('Revenue')}
              value={formatCurrency(totals.revenue)}
            />
            <DashboardMetricCard
              breakdowns={requestBreakdowns}
              description={t('Request volume keeps the live demand rhythm visible for today, the trailing week, and the current month.')}
              label={t('Total requests')}
              value={formatUnits(totals.request_count)}
            />
            <DashboardMetricCard
              breakdowns={averageSpendBreakdowns}
              description={t('Average booked spend shows current request efficiency without leaving the dashboard overview.')}
              label={t('Average booked spend')}
              value={formatAverageSpend(totals.average_booked_spend, totals.request_count, t)}
            />
          </div>

          <div className="grid gap-6 xl:grid-cols-[1.35fr_0.95fr]">
            <DashboardAnalyticsPanel
              actions={(
                <div className="inline-flex rounded-full border border-emerald-500/20 bg-emerald-500/10 px-3 py-1 text-xs font-semibold uppercase tracking-[0.14em] text-emerald-700 dark:text-emerald-200">
                  {snapshot
                    ? t('{amount} booked', {
                      amount: formatCurrency(snapshot.billing_summary.booked_amount),
                    })
                    : t('Awaiting spend')}
                </div>
              )}
              description={t('Booked amount now follows the shared workspace panel pattern while preserving the existing spend trend telemetry.')}
              title={t('Spend trend')}
            >
              <DashboardRevenueTrendChart
                points={viewModel?.spend_trend_points ?? []}
                title={t('Spend trend')}
                summaryLabel={t('Visible requests')}
                summaryValue={formatUnits(snapshot?.usage_summary.total_requests ?? 0)}
                peakLabel={t('Peak spend')}
                yAxisFormatter={formatCurrency}
              />
            </DashboardAnalyticsPanel>

            <DashboardAnalyticsPanel
              description={t('Provider share is rendered with the same distribution ring and evidence table rhythm as the shared workspace system.')}
              title={t('Provider distribution')}
            >
              <div className="grid gap-5 2xl:grid-cols-[minmax(320px,0.88fr)_1.12fr]">
                {viewModel?.provider_share_series.length ? (
                  <DashboardDistributionRingChart
                    rows={viewModel.provider_share_series.map((row) => ({
                      id: row.name,
                      ...row,
                    }))}
                    sliceClassNames={chartPalette.map((item) => item.sliceClassName)}
                    centerLabel={t('Requests')}
                    centerValue={formatUnits(snapshot?.usage_summary.total_requests ?? 0)}
                    ariaLabel={t('Provider distribution')}
                    valueAccessor={(row) => row.value}
                  />
                ) : (
                  <EmptyState
                    description={t('Provider share appears after the first request passes through the gateway.')}
                    title={t('No provider traffic yet')}
                  />
                )}

                <DataTable
                  columns={[
                    {
                      id: 'provider',
                      header: t('Provider'),
                      cell: (row) => {
                        const palette =
                          chartPalette[
                            (viewModel?.provider_mix.findIndex((item) => item.id === row.id) ?? 0)
                            % chartPalette.length
                          ] ?? chartPalette[0]!;

                        return (
                          <div className="flex items-center gap-3">
                            <span className={`h-2.5 w-2.5 rounded-full ${palette.dotClassName}`} />
                            <div className="font-semibold text-zinc-950 dark:text-zinc-50">
                              {row.label}
                            </div>
                          </div>
                        );
                      },
                    },
                    { id: 'requests', header: t('Requests'), cell: (row) => row.value_label },
                    { id: 'projects', header: t('Projects'), cell: (row) => row.secondary_label },
                    { id: 'share', header: t('Share'), cell: (row) => `${row.share}%` },
                  ]}
                  emptyState={(
                    <div className="mx-auto flex max-w-[28rem] flex-col items-center gap-2 text-center">
                      <strong className="text-base font-semibold text-zinc-950 dark:text-zinc-50">
                        {t('No provider traffic yet')}
                      </strong>
                      <p className="text-sm text-zinc-500 dark:text-zinc-400">
                        {t('Provider share appears after the first request passes through the gateway.')}
                      </p>
                    </div>
                  )}
                  getRowId={(row) => row.id}
                  rows={viewModel?.provider_mix ?? []}
                />
              </div>
            </DashboardAnalyticsPanel>
          </div>
          <div className="grid gap-6 xl:grid-cols-[1.35fr_0.95fr]">
            <DashboardAnalyticsPanel
              actions={(
                <div className="inline-flex rounded-full border border-primary-500/20 bg-primary-500/10 px-3 py-1 text-xs font-semibold uppercase tracking-[0.14em] text-primary-700 dark:text-primary-200">
                  {snapshot
                    ? t('{count} visible requests', {
                      count: formatUnits(snapshot.usage_summary.total_requests),
                    })
                    : t('Awaiting traffic')}
                </div>
              )}
              description={t('Traffic posture now lives on a shared workspace panel while keeping the portal-specific multi-series token telemetry.')}
              title={t('Traffic trend')}
            >
              <DashboardTokenTrendChart
                points={viewModel?.traffic_trend_points ?? []}
                title={t('Traffic trend')}
                summary={t('Total, input, and output tokens stay visible in the same multi-series claw surface.')}
                series={[
                  {
                    key: 'total_tokens',
                    label: t('Total tokens'),
                    dotClassName: 'bg-primary-500',
                    strokeClassName: 'text-primary-500',
                  },
                  {
                    key: 'input_tokens',
                    label: t('Input tokens'),
                    dotClassName: 'bg-sky-500',
                    strokeClassName: 'text-sky-500',
                  },
                  {
                    key: 'output_tokens',
                    label: t('Output tokens'),
                    dotClassName: 'bg-emerald-500',
                    strokeClassName: 'text-emerald-500',
                  },
                ]}
                yAxisFormatter={(value) => formatUnits(value)}
              />
            </DashboardAnalyticsPanel>

            <DashboardAnalyticsPanel
              description={t('Model demand adopts the same ring-chart-and-table pairing that the shared workspace system uses for model breakdown.')}
              title={t('Model distribution')}
            >
              <div className="grid gap-5 2xl:grid-cols-[minmax(320px,0.88fr)_1.12fr]">
                {viewModel?.model_demand_series.length ? (
                  <DashboardModelDistributionChart
                    rows={viewModel.model_demand_series.map((row) => ({
                      id: row.name,
                      ...row,
                    }))}
                    sliceClassNames={chartPalette.map((item) => item.sliceClassName)}
                    centerLabel={t('Models')}
                    centerValue={formatUnits(snapshot?.usage_summary.model_count ?? 0)}
                    ariaLabel={t('Model distribution')}
                    valueAccessor={(row) => row.requests}
                  />
                ) : (
                  <EmptyState
                    description={t('Model demand appears after the first request passes through the gateway.')}
                    title={t('No model demand yet')}
                  />
                )}

                <DataTable
                  columns={[
                    {
                      id: 'model',
                      header: t('Model'),
                      cell: (row) => {
                        const palette =
                          chartPalette[
                            (viewModel?.model_mix.findIndex((item) => item.id === row.id) ?? 0)
                            % chartPalette.length
                          ] ?? chartPalette[0]!;

                        return (
                          <div className="flex items-center gap-3">
                            <span className={`h-2.5 w-2.5 rounded-full ${palette.dotClassName}`} />
                            <div className="font-semibold text-zinc-950 dark:text-zinc-50">
                              {row.label}
                            </div>
                          </div>
                        );
                      },
                    },
                    { id: 'requests', header: t('Requests'), cell: (row) => row.value_label },
                    { id: 'providers', header: t('Providers'), cell: (row) => row.secondary_label },
                    { id: 'share', header: t('Share'), cell: (row) => `${row.share}%` },
                  ]}
                  emptyState={(
                    <div className="mx-auto flex max-w-[28rem] flex-col items-center gap-2 text-center">
                      <strong className="text-base font-semibold text-zinc-950 dark:text-zinc-50">
                        {t('No model demand yet')}
                      </strong>
                      <p className="text-sm text-zinc-500 dark:text-zinc-400">
                        {t('Model demand appears after the first request passes through the gateway.')}
                      </p>
                    </div>
                  )}
                  getRowId={(row) => row.id}
                  rows={viewModel?.model_mix ?? []}
                />
              </div>
            </DashboardAnalyticsPanel>
          </div>
          <ManagementWorkbench
            actions={
              activeWorkbenchTab === 'routing' && routingPosture ? (
                <Button onClick={() => onNavigate('routing')} variant="ghost">
                  {routingPosture.action_label}
                </Button>
              ) : undefined
            }
            description={t('A single shared workbench for requests, routing evidence, module posture, and next actions.')}
            detail={{
              children:
                activeWorkbenchTab === 'routing' ? (
                  routingPosture ? (
                    <div className="grid gap-3 sm:grid-cols-2">
                      <div className="rounded-[1.4rem] border border-zinc-200/70 bg-zinc-50/70 p-4 dark:border-white/6 dark:bg-zinc-950/35">
                        <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                          {t('Strategy')}
                        </div>
                        <div className="mt-2 text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                          {routingPosture.strategy_label}
                        </div>
                      </div>
                      <div className="rounded-[1.4rem] border border-zinc-200/70 bg-zinc-50/70 p-4 dark:border-white/6 dark:bg-zinc-950/35">
                        <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                          {t('Selected provider')}
                        </div>
                        <div className="mt-2 text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                          {routingPosture.selected_provider}
                        </div>
                      </div>
                      <div className="rounded-[1.4rem] border border-zinc-200/70 bg-zinc-50/70 p-4 dark:border-white/6 dark:bg-zinc-950/35">
                        <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                          {t('Preferred region')}
                        </div>
                        <div className="mt-2 text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                          {routingPosture.preferred_region}
                        </div>
                      </div>
                      <div className="rounded-[1.4rem] border border-zinc-200/70 bg-zinc-50/70 p-4 dark:border-white/6 dark:bg-zinc-950/35">
                        <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                          {t('Evidence count')}
                        </div>
                        <div className="mt-2 text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                          {routingPosture.evidence_count}
                        </div>
                      </div>
                    </div>
                  ) : (
                    <EmptyState
                      description={t('Routing posture will appear once project routing data becomes available.')}
                      title={t('Preparing routing evidence')}
                    />
                  )
                ) : activeWorkbenchTab === 'modules' ? (
                  viewModel?.modules.length ? (
                    <div className="space-y-3">
                      {viewModel.modules.slice(0, 4).map((module) => (
                        <div
                          key={module.route}
                          className="rounded-[1.4rem] border border-zinc-200/70 bg-zinc-50/70 p-4 dark:border-white/6 dark:bg-zinc-950/35"
                        >
                          <div className="flex items-center justify-between gap-3">
                            <div className="font-semibold text-zinc-950 dark:text-zinc-50">
                              {module.title}
                            </div>
                            {renderStatusBadge(module.status_label, module.status_variant)}
                          </div>
                          <p className="mt-3 text-sm leading-6 text-zinc-500 dark:text-zinc-400">
                            {module.detail}
                          </p>
                          <div className="mt-3">
                            <Button
                              onClick={() => onNavigate(module.route)}
                              variant="ghost"
                            >
                              {module.action_label}
                            </Button>
                          </div>
                        </div>
                      ))}
                    </div>
                  ) : (
                    <EmptyState
                      description={t('Module posture will appear after the dashboard finishes loading.')}
                      title={t('Preparing module posture')}
                    />
                  )
                ) : activeWorkbenchTab === 'actions' ? (
                  actionCards.length ? (
                    <div className="space-y-3">
                      {actionCards.slice(0, 3).map((item, index) => (
                        <div
                          key={item.id}
                          className="rounded-[1.4rem] border border-zinc-200/70 bg-zinc-50/70 p-4 dark:border-white/6 dark:bg-zinc-950/35"
                        >
                          <div className="flex items-center justify-between gap-3">
                            <div className="font-semibold text-zinc-950 dark:text-zinc-50">
                              {item.title}
                            </div>
                            {renderStatusBadge(`P${index + 1}`, item.status_variant)}
                          </div>
                          <p className="mt-3 text-sm leading-6 text-zinc-500 dark:text-zinc-400">
                            {item.detail}
                          </p>
                          {item.route && item.action_label ? (
                            <div className="mt-3">
                              <Button
                                onClick={() => onNavigate(item.route!)}
                                variant={actionTone(index)}
                              >
                                {item.action_label}
                              </Button>
                            </div>
                          ) : null}
                        </div>
                      ))}
                    </div>
                  ) : (
                    <EmptyState
                      description={t('Next actions will appear when workspace data finishes loading.')}
                      title={t('Preparing next actions')}
                    />
                  )
                ) : (
                  <div className="grid gap-3 sm:grid-cols-2">
                    <div className="rounded-[1.4rem] border border-zinc-200/70 bg-zinc-50/70 p-4 dark:border-white/6 dark:bg-zinc-950/35">
                      <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                        {t('Visible rows')}
                      </div>
                      <div className="mt-2 text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                        {formatUnits(workbenchConfig.rows.length)}
                      </div>
                    </div>
                    <div className="rounded-[1.4rem] border border-zinc-200/70 bg-zinc-50/70 p-4 dark:border-white/6 dark:bg-zinc-950/35">
                      <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                        {t('Average booked spend')}
                      </div>
                      <div className="mt-2 text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                        {formatCurrency(requestAverageAmount)}
                      </div>
                    </div>
                    <div className="rounded-[1.4rem] border border-zinc-200/70 bg-zinc-50/70 p-4 dark:border-white/6 dark:bg-zinc-950/35">
                      <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                        {t('Active filter')}
                      </div>
                      <div className="mt-2 text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                        {requestFilterLabel}
                      </div>
                    </div>
                    <div className="rounded-[1.4rem] border border-zinc-200/70 bg-zinc-50/70 p-4 dark:border-white/6 dark:bg-zinc-950/35">
                      <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                        {t('Visible requests')}
                      </div>
                      <div className="mt-2 text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                        {formatUnits(snapshot?.usage_summary.total_requests ?? 0)}
                      </div>
                    </div>
                  </div>
                ),
              description:
                activeWorkbenchTab === 'routing'
                  ? routingPosture?.latest_reason ??
                    t('Routing evidence will appear once routing data is available.')
                  : activeWorkbenchTab === 'modules'
                    ? t('Each portal module exposes its latest operational posture in the same shared inspector rail.')
                    : activeWorkbenchTab === 'actions'
                      ? t('The highest-leverage actions stay visible beside the workbench table.')
                      : t('Request filters, spend posture, and token flow remain visible beside the workbench table.'),
              eyebrow:
                activeWorkbenchTab === 'routing'
                  ? t('Routing posture')
                  : activeWorkbenchTab === 'modules'
                    ? t('Workspace signal')
                    : activeWorkbenchTab === 'actions'
                      ? t('Priority stack')
                      : t('Request posture'),
              summary:
                activeWorkbenchTab === 'routing' && routingPosture ? (
                  renderStatusBadge(
                    routingPosture.status_label,
                    routingPosture.status_variant,
                  )
                ) : activeWorkbenchTab === 'modules' ? (
                  <span className="inline-flex items-center rounded-full border border-zinc-200/80 bg-white/90 px-3 py-1 text-xs font-semibold uppercase tracking-[0.14em] text-zinc-600 dark:border-white/8 dark:bg-zinc-950/45 dark:text-zinc-300">
                    {t('{count} modules', { count: formatUnits(viewModel?.modules.length ?? 0) })}
                  </span>
                ) : activeWorkbenchTab === 'actions' ? (
                  <span className="inline-flex items-center rounded-full border border-zinc-200/80 bg-white/90 px-3 py-1 text-xs font-semibold uppercase tracking-[0.14em] text-zinc-600 dark:border-white/8 dark:bg-zinc-950/45 dark:text-zinc-300">
                    {t('{count} actions', { count: formatUnits(actionCards.length) })}
                  </span>
                ) : (
                  <span className="inline-flex items-center rounded-full border border-zinc-200/80 bg-white/90 px-3 py-1 text-xs font-semibold uppercase tracking-[0.14em] text-zinc-600 dark:border-white/8 dark:bg-zinc-950/45 dark:text-zinc-300">
                    {requestFilterLabel}
                  </span>
                ),
              title:
                activeWorkbenchTab === 'routing'
                  ? routingPosture?.title ?? t('Preparing routing evidence')
                  : activeWorkbenchTab === 'modules'
                    ? t('Module posture snapshot')
                    : activeWorkbenchTab === 'actions'
                      ? t('Next actions summary')
                      : t('Request posture summary'),
            }}
            detailWidth={360}
            eyebrow={t('Activity workbench')}
            filters={(
              <div className="space-y-4">
                <div
                  className="flex flex-wrap gap-2"
                  data-slot="portal-dashboard-workbench-tabs"
                >
                  {workbenchTabs.map((tab) => {
                    const isActive = activeWorkbenchTab === tab.id;

                    return (
                      <Button
                        key={tab.id}
                        type="button"
                        onClick={() => setActiveWorkbenchTab(tab.id)}
                        className={`h-auto rounded-full px-4 py-2 text-sm font-semibold shadow-none transition-colors ${
                          isActive
                            ? 'bg-zinc-950 text-white dark:bg-zinc-100 dark:text-zinc-950'
                            : 'bg-zinc-100 text-zinc-600 hover:bg-zinc-200 dark:bg-zinc-800 dark:text-zinc-300 dark:hover:bg-zinc-700'
                        }`}
                        variant="ghost"
                      >
                        {tab.label}
                      </Button>
                    );
                  })}
                </div>

                {activeWorkbenchTab === 'requests' ? (
                  <div className="flex flex-wrap gap-2">
                    {requestFilterOptions.map((option) => (
                      <Button
                        key={option.id}
                        type="button"
                        onClick={() => setRequestFilter(option.id)}
                        className={`h-auto rounded-full px-3 py-1.5 text-xs font-semibold uppercase tracking-[0.14em] shadow-none transition-colors ${
                          requestFilter === option.id
                            ? 'bg-primary-500 text-white'
                            : 'bg-zinc-100 text-zinc-600 hover:bg-zinc-200 dark:bg-zinc-800 dark:text-zinc-300 dark:hover:bg-zinc-700'
                        }`}
                        variant="ghost"
                      >
                        {option.label}
                      </Button>
                    ))}
                  </div>
                ) : null}
              </div>
            )}
            main={{
              children: (
                <DataTable
                  columns={[
                    {
                      id: 'primary',
                      header: workbenchConfig.primaryLabel,
                      cell: (row) => row.primary,
                    },
                    {
                      id: 'secondary',
                      header: workbenchConfig.secondaryLabel,
                      cell: (row) => row.secondary,
                    },
                    {
                      id: 'detail',
                      header: workbenchConfig.detailLabel,
                      cell: (row) => row.detail,
                    },
                    {
                      id: 'status',
                      header: workbenchConfig.statusLabel,
                      cell: (row) => row.status,
                    },
                    {
                      id: 'action',
                      header: workbenchConfig.actionLabel,
                      cell: (row) => row.action,
                    },
                  ]}
                  emptyState={(
                    <div className="mx-auto flex max-w-[32rem] flex-col items-center gap-2 text-center">
                      <strong className="text-base font-semibold text-zinc-950 dark:text-zinc-50">
                        {workbenchConfig.emptyTitle}
                      </strong>
                      <p className="text-sm text-zinc-500 dark:text-zinc-400">
                        {workbenchConfig.emptyDetail}
                      </p>
                    </div>
                  )}
                  getRowId={(row) => row.id}
                  rows={workbenchConfig.rows}
                />
              ),
              description: workbenchDescription,
              title: workbenchTitle,
            }}
            title={t('Analytics workbench')}
          />

          {error ? (
            <div className="flex items-center gap-3 rounded-[1.5rem] border border-amber-500/20 bg-amber-500/10 px-4 py-3 text-sm text-amber-800 dark:text-amber-200">
              <TriangleAlert className="h-4 w-4" />
              {error}
            </div>
          ) : null}

          {isLoading ? (
            <div className="flex items-center gap-3 rounded-[1.5rem] border border-primary-500/15 bg-primary-500/10 px-4 py-3 text-sm text-primary-700 dark:text-primary-200">
              <RefreshCw className="h-4 w-4 animate-spin" />
              {status}
            </div>
          ) : null}
        </div>
      </div>
    </div>
  );
}


