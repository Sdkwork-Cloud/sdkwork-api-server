import {
  Activity,
  Coins,
  KeyRound,
  RefreshCw,
  TriangleAlert,
} from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';
import type { ReactNode } from 'react';
import {
  Button,
  DataTable,
  EmptyState,
  formatCurrency,
  formatDateTime,
  formatUnits,
  InlineButton,
} from 'sdkwork-router-portal-commons';
import { portalErrorMessage } from 'sdkwork-router-portal-portal-api';

import {
  DashboardDistributionRingChart,
  DashboardModelDistributionChart,
  DashboardRevenueTrendChart,
  DashboardSectionHeader,
  DashboardStatusPill,
  DashboardSummaryCard,
  DashboardTokenTrendChart,
} from '../components';
import { loadPortalDashboardSnapshot } from '../repository';
import { buildPortalDashboardViewModel } from '../services';
import type {
  DashboardInsight,
  DashboardTone,
  PortalDashboardPageProps,
  PortalDashboardPageViewModel,
} from '../types';

const surfaceClass =
  'rounded-[2rem] border border-[color:var(--portal-border-color)] [background:var(--portal-surface-background)] p-6 shadow-[var(--portal-shadow-soft)] backdrop-blur';
const chartFrameClass =
  'overflow-hidden rounded-[1.5rem] border border-[color:var(--portal-chart-grid)]';

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

function buildTrafficHeadline(viewModel: PortalDashboardPageViewModel): {
  title: string;
  detail: string;
} {
  const { snapshot, provider_mix: providerMix, model_mix: modelMix } = viewModel;

  if (snapshot.usage_summary.total_requests === 0) {
    return {
      title: 'Waiting for the first API request',
      detail:
        'As soon as real traffic lands, the dashboard will summarize provider concentration, model mix, and booked spend here.',
    };
  }

  const leadProvider = providerMix[0]?.label ?? 'your leading provider';
  const leadModel = modelMix[0]?.label ?? 'your leading model';

  return {
    title: `${leadModel} is driving the current visible demand`,
    detail: `${leadProvider} currently leads ${formatUnits(snapshot.usage_summary.total_requests)} visible requests with ${formatCurrency(snapshot.billing_summary.booked_amount)} in booked spend.`,
  };
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

function requestPosture(requestAmount: number, averageAmount: number): {
  label: string;
  tone: DashboardTone;
} {
  if (requestAmount >= averageAmount * 1.35 && averageAmount > 0) {
    return {
      label: 'High spend',
      tone: 'accent',
    };
  }

  if (requestAmount > 0) {
    return {
      label: 'Tracked',
      tone: 'positive',
    };
  }

  return {
    label: 'Pending',
    tone: 'default',
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
  const [activeWorkbenchTab, setActiveWorkbenchTab] =
    useState<WorkbenchTab>('requests');
  const [requestFilter, setRequestFilter] = useState<RequestFilter>('all');
  const [viewModel, setViewModel] = useState<PortalDashboardPageViewModel | null>(
    initialSnapshot ? buildPortalDashboardViewModel(initialSnapshot) : null,
  );
  const [status, setStatus] = useState(
    initialSnapshot
      ? 'Refreshing routing and activity data for the current workspace.'
      : 'Loading your workspace overview.',
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
          ),
        );
        setError(null);
        setStatus('Live traffic, routing, and spend telemetry are up to date.');
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
  }, [initialSnapshot]);

  const snapshot = viewModel?.snapshot ?? initialSnapshot ?? null;
  const trafficHeadline = viewModel ? buildTrafficHeadline(viewModel) : null;
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
  const workbenchConfig = useMemo<DashboardWorkbenchConfig>(() => {
    if (activeWorkbenchTab === 'routing') {
      return {
        primaryLabel: 'Signal',
        secondaryLabel: 'Timeline',
        detailLabel: 'Detail',
        statusLabel: 'Status',
        actionLabel: 'Action',
        emptyTitle: 'Preparing routing evidence',
        emptyDetail: 'Routing evidence will appear once project routing data becomes available.',
        rows: routingEvidence.map((row) => ({
          id: row.id,
          primary: (
            <span className="font-semibold text-zinc-950 dark:text-zinc-50">{row.title}</span>
          ),
          secondary: row.timestamp_label,
          detail: row.detail,
          status: <DashboardStatusPill label={row.title} tone={row.tone} />,
          action:
            row.route && row.action_label ? (
              <InlineButton onClick={() => onNavigate(row.route!)} tone="ghost">
                {row.action_label}
              </InlineButton>
            ) : (
              '-'
            ),
        })),
      };
    }

    if (activeWorkbenchTab === 'modules') {
      return {
        primaryLabel: 'Module',
        secondaryLabel: 'Status detail',
        detailLabel: 'Operational detail',
        statusLabel: 'Status',
        actionLabel: 'Action',
        emptyTitle: 'Preparing module posture',
        emptyDetail: 'Module posture will appear after the dashboard finishes loading.',
        rows: (viewModel?.modules ?? []).map((row) => ({
          id: row.route,
          primary: (
            <span className="font-semibold text-zinc-950 dark:text-zinc-50">{row.title}</span>
          ),
          secondary: row.status_label,
          detail: row.detail,
          status: <DashboardStatusPill label={row.status_label} tone={row.tone} />,
          action: (
            <InlineButton onClick={() => onNavigate(row.route)} tone="ghost">
              {row.action_label}
            </InlineButton>
          ),
        })),
      };
    }

    if (activeWorkbenchTab === 'actions') {
      return {
        primaryLabel: 'Next action',
        secondaryLabel: 'Priority',
        detailLabel: 'Action detail',
        statusLabel: 'Status',
        actionLabel: 'Action',
        emptyTitle: 'Preparing next actions',
        emptyDetail: 'Next actions will appear when workspace data finishes loading.',
        rows: actionCards.map((item, index) => ({
          id: item.id,
          primary: (
            <span className="font-semibold text-zinc-950 dark:text-zinc-50">{item.title}</span>
          ),
          secondary: `Priority ${index + 1}`,
          detail: item.detail,
          status: <DashboardStatusPill label={item.title} tone={item.tone} />,
          action:
            item.route && item.action_label ? (
              <InlineButton onClick={() => onNavigate(item.route!)} tone={actionTone(index)}>
                {item.action_label}
              </InlineButton>
            ) : (
              '-'
            ),
        })),
      };
    }

    return {
      primaryLabel: 'Request',
      secondaryLabel: 'Provider',
      detailLabel: 'Token detail',
      statusLabel: 'Request posture',
      actionLabel: 'Booked spend',
      emptyTitle: 'No recent requests yet',
      emptyDetail:
        'Once gateway requests start flowing through your project, token usage and booked amount will appear here.',
      rows: requestRows.map((row) => {
        const posture = requestPosture(row.amount, requestAverageAmount);

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
          detail: `${formatUnits(row.input_tokens)} in / ${formatUnits(row.output_tokens)} out / ${formatUnits(row.total_tokens)} total`,
          status: <DashboardStatusPill label={posture.label} tone={posture.tone} />,
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
    viewModel,
  ]);

  if (error && !snapshot) {
    return (
      <div className="flex h-full items-center justify-center px-6 py-10">
        <div className="max-w-md rounded-[2rem] border border-white/70 bg-white/85 p-8 text-center shadow-[0_20px_60px_rgba(15,23,42,0.08)] backdrop-blur dark:border-white/6 dark:bg-zinc-900/85">
          <div className="mx-auto flex h-14 w-14 items-center justify-center rounded-full bg-rose-500/12 text-rose-500">
            <Activity className="h-6 w-6" />
          </div>
          <h1 className="mt-5 text-2xl font-semibold tracking-tight text-zinc-950 dark:text-zinc-50">
            Dashboard could not be prepared
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
          <div className="grid gap-4 xl:grid-cols-3">
            <DashboardSummaryCard
              accent={<Activity className="h-5 w-5 text-primary-500" />}
              changeLabel={
                snapshot?.usage_summary.total_requests
                  ? `${formatUnits(snapshot.usage_summary.total_requests)} visible requests`
                  : 'Awaiting live traffic'
              }
              description="Live request demand, provider spread, and model activity mapped onto the same claw-style summary surface."
              eyebrow="Telemetry"
              title="Traffic posture"
            >
              <div className="grid gap-4">
                <div className="rounded-[1.5rem] bg-primary-500/[0.08] p-4">
                  <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-primary-700 dark:text-primary-200">
                    Visible requests
                  </div>
                  <div className="mt-2 text-3xl font-semibold tracking-tight text-zinc-950 dark:text-zinc-50">
                    {formatUnits(snapshot?.usage_summary.total_requests ?? 0)}
                  </div>
                  <div className="mt-2 text-sm text-zinc-600 dark:text-zinc-300">
                    {trafficHeadline?.detail ??
                      'The first request unlocks trend and distribution signals.'}
                  </div>
                </div>
                <div className="grid gap-3 md:grid-cols-3">
                  <div className="rounded-[1.4rem] bg-zinc-950/[0.03] p-4 dark:bg-white/[0.04]">
                    <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                      Providers
                    </div>
                    <div className="mt-2 text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                      {formatUnits(snapshot?.usage_summary.provider_count ?? 0)}
                    </div>
                  </div>
                  <div className="rounded-[1.4rem] bg-zinc-950/[0.03] p-4 dark:bg-white/[0.04]">
                    <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                      Models
                    </div>
                    <div className="mt-2 text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                      {formatUnits(snapshot?.usage_summary.model_count ?? 0)}
                    </div>
                  </div>
                  <div className="rounded-[1.4rem] bg-zinc-950/[0.03] p-4 dark:bg-white/[0.04]">
                    <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                      Token units
                    </div>
                    <div className="mt-2 text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                      {formatUnits(snapshot?.billing_summary.used_units ?? 0)}
                    </div>
                  </div>
                </div>
              </div>
            </DashboardSummaryCard>

            <DashboardSummaryCard
              accent={<Coins className="h-5 w-5 text-emerald-500" />}
              changeLabel={
                snapshot?.billing_summary.exhausted ? 'Action required' : 'Spend visible'
              }
              description="Booked spend, remaining runway, and quota pressure stay visible without leaving the dashboard workbench."
              eyebrow="Billing"
              title="Cost and quota"
            >
              <div className="grid gap-4">
                <div className="rounded-[1.5rem] bg-emerald-500/[0.08] p-4">
                  <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-emerald-700 dark:text-emerald-200">
                    Booked spend
                  </div>
                  <div className="mt-2 text-3xl font-semibold tracking-tight text-zinc-950 dark:text-zinc-50">
                    {formatCurrency(snapshot?.billing_summary.booked_amount ?? 0)}
                  </div>
                  <div className="mt-2 text-sm text-zinc-600 dark:text-zinc-300">
                    {snapshot?.billing_summary.exhausted
                      ? 'Quota is exhausted and recovery is the next required action.'
                      : 'Booked spend updates in sync with recent request activity.'}
                  </div>
                </div>
                <div className="grid gap-3 md:grid-cols-3">
                  <div className="rounded-[1.4rem] bg-zinc-950/[0.03] p-4 dark:bg-white/[0.04]">
                    <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                      Remaining units
                    </div>
                    <div className="mt-2 text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                      {snapshot?.billing_summary.remaining_units === null
                        ? 'Unlimited'
                        : formatUnits(snapshot?.billing_summary.remaining_units ?? 0)}
                    </div>
                  </div>
                  <div className="rounded-[1.4rem] bg-zinc-950/[0.03] p-4 dark:bg-white/[0.04]">
                    <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                      Ledger entries
                    </div>
                    <div className="mt-2 text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                      {formatUnits(snapshot?.billing_summary.entry_count ?? 0)}
                    </div>
                  </div>
                  <div className="rounded-[1.4rem] bg-zinc-950/[0.03] p-4 dark:bg-white/[0.04]">
                    <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                      Key inventory
                    </div>
                    <div className="mt-2 text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                      {formatUnits(snapshot?.api_key_count ?? 0)}
                    </div>
                  </div>
                </div>
              </div>
            </DashboardSummaryCard>

            <DashboardSummaryCard
              accent={<KeyRound className="h-5 w-5 text-amber-500" />}
              changeLabel={viewModel?.routing_posture?.title ?? 'Checking readiness'}
              description="Route selection, evidence coverage, and workspace access posture are summarized in the same family as claw-studio."
              eyebrow="Workspace"
              title="Workspace readiness"
            >
              <div className="grid gap-4">
                <div className="rounded-[1.5rem] bg-amber-500/[0.08] p-4">
                  <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-amber-700 dark:text-amber-200">
                    Active workspace
                  </div>
                  <div className="mt-2 text-2xl font-semibold tracking-tight text-zinc-950 dark:text-zinc-50">
                    {snapshot?.workspace.project.name ?? 'Preparing workspace'}
                  </div>
                  <div className="mt-2 text-sm text-zinc-600 dark:text-zinc-300">
                    {snapshot?.workspace.user.active
                      ? 'Operator session is active.'
                      : 'Workspace identity is still loading.'}
                  </div>
                </div>
                <div className="grid gap-3 md:grid-cols-3">
                  <div className="rounded-[1.4rem] bg-zinc-950/[0.03] p-4 dark:bg-white/[0.04]">
                    <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                      Default route
                    </div>
                    <div className="mt-2 text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                      {viewModel?.routing_posture?.selected_provider ?? 'Pending'}
                    </div>
                  </div>
                  <div className="rounded-[1.4rem] bg-zinc-950/[0.03] p-4 dark:bg-white/[0.04]">
                    <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                      Evidence count
                    </div>
                    <div className="mt-2 text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                      {viewModel?.routing_posture?.evidence_count ?? '0'}
                    </div>
                  </div>
                  <div className="rounded-[1.4rem] bg-zinc-950/[0.03] p-4 dark:bg-white/[0.04]">
                    <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                      Preferred region
                    </div>
                    <div className="mt-2 text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                      {viewModel?.routing_posture?.preferred_region ?? 'Global'}
                    </div>
                  </div>
                </div>
              </div>
            </DashboardSummaryCard>
          </div>

          <div className="grid gap-6 xl:grid-cols-[1.35fr_0.95fr]">
            <section className={surfaceClass}>
              <DashboardSectionHeader
                eyebrow="Billing"
                title="Spend trend"
                description="Booked amount now follows the same claw-style trend surface used for revenue in claw-studio."
                action={
                  <div className="inline-flex rounded-full border border-emerald-500/20 bg-emerald-500/10 px-3 py-1 text-xs font-semibold uppercase tracking-[0.14em] text-emerald-700 dark:text-emerald-200">
                    {snapshot
                      ? `${formatCurrency(snapshot.billing_summary.booked_amount)} booked`
                      : 'Awaiting spend'}
                  </div>
                }
              />
              <div className="mt-6">
                <DashboardRevenueTrendChart
                  points={viewModel?.spend_trend_points ?? []}
                  title="Spend trend"
                  summaryLabel="Visible requests"
                  summaryValue={formatUnits(snapshot?.usage_summary.total_requests ?? 0)}
                  peakLabel="Peak spend"
                  yAxisFormatter={formatCurrency}
                />
              </div>
            </section>

            <section className={surfaceClass}>
              <DashboardSectionHeader
                eyebrow="Telemetry"
                title="Provider distribution"
                description="Provider share is rendered with the same distribution ring and evidence table rhythm as claw-studio."
              />
              <div className="mt-6 grid gap-5 2xl:grid-cols-[minmax(320px,0.88fr)_1.12fr]">
                {viewModel?.provider_share_series.length ? (
                  <DashboardDistributionRingChart
                    rows={viewModel.provider_share_series.map((row) => ({
                      id: row.name,
                      ...row,
                    }))}
                    sliceClassNames={chartPalette.map((item) => item.sliceClassName)}
                    centerLabel="Requests"
                    centerValue={formatUnits(snapshot?.usage_summary.total_requests ?? 0)}
                    ariaLabel="Provider distribution"
                    valueAccessor={(row) => row.value}
                  />
                ) : (
                  <EmptyState
                    detail="Provider share appears after the first request passes through the gateway."
                    title="No provider traffic yet"
                  />
                )}

                <DataTable
                  columns={[
                    {
                      key: 'provider',
                      label: 'Provider',
                      render: (row) => {
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
                    { key: 'requests', label: 'Requests', render: (row) => row.value_label },
                    { key: 'projects', label: 'Projects', render: (row) => row.secondary_label },
                    { key: 'share', label: 'Share', render: (row) => `${row.share}%` },
                  ]}
                  empty={(
                    <div className="mx-auto flex max-w-[28rem] flex-col items-center gap-2 text-center">
                      <strong className="text-base font-semibold text-zinc-950 dark:text-zinc-50">
                        No provider traffic yet
                      </strong>
                      <p className="text-sm text-zinc-500 dark:text-zinc-400">
                        Provider share appears after the first request passes through the gateway.
                      </p>
                    </div>
                  )}
                  getKey={(row) => row.id}
                  rows={viewModel?.provider_mix ?? []}
                />
              </div>
            </section>
          </div>
          <div className="grid gap-6 xl:grid-cols-[1.35fr_0.95fr]">
            <section className={surfaceClass}>
              <DashboardSectionHeader
                eyebrow="Telemetry"
                title="Traffic trend"
                description="Traffic trend now follows the multi-series claw token-intelligence surface, using Portal token telemetry instead of studio run usage."
                action={
                  <div className="inline-flex rounded-full border border-primary-500/20 bg-primary-500/10 px-3 py-1 text-xs font-semibold uppercase tracking-[0.14em] text-primary-700 dark:text-primary-200">
                    {snapshot
                      ? `${formatUnits(snapshot.usage_summary.total_requests)} visible requests`
                      : 'Awaiting traffic'}
                  </div>
                }
              />
              <div className="mt-6">
                <DashboardTokenTrendChart
                  points={viewModel?.traffic_trend_points ?? []}
                  title="Traffic trend"
                  summary="Total, input, and output tokens stay visible in the same multi-series claw surface."
                  series={[
                    {
                      key: 'total_tokens',
                      label: 'Total tokens',
                      dotClassName: 'bg-primary-500',
                      strokeClassName: 'text-primary-500',
                    },
                    {
                      key: 'input_tokens',
                      label: 'Input tokens',
                      dotClassName: 'bg-sky-500',
                      strokeClassName: 'text-sky-500',
                    },
                    {
                      key: 'output_tokens',
                      label: 'Output tokens',
                      dotClassName: 'bg-emerald-500',
                      strokeClassName: 'text-emerald-500',
                    },
                  ]}
                  yAxisFormatter={(value) => formatUnits(value)}
                />
              </div>
            </section>

            <section className={surfaceClass}>
              <DashboardSectionHeader
                eyebrow="Telemetry"
                title="Model distribution"
                description="Model demand adopts the same ring-chart-and-table pairing that claw-studio uses for model breakdown."
              />
              <div className="mt-6 grid gap-5 2xl:grid-cols-[minmax(320px,0.88fr)_1.12fr]">
                {viewModel?.model_demand_series.length ? (
                  <DashboardModelDistributionChart
                    rows={viewModel.model_demand_series.map((row) => ({
                      id: row.name,
                      ...row,
                    }))}
                    sliceClassNames={chartPalette.map((item) => item.sliceClassName)}
                    centerLabel="Models"
                    centerValue={formatUnits(snapshot?.usage_summary.model_count ?? 0)}
                    ariaLabel="Model distribution"
                    valueAccessor={(row) => row.requests}
                  />
                ) : (
                  <EmptyState
                    detail="Model demand appears after the first request passes through the gateway."
                    title="No model demand yet"
                  />
                )}

                <DataTable
                  columns={[
                    {
                      key: 'model',
                      label: 'Model',
                      render: (row) => {
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
                    { key: 'requests', label: 'Requests', render: (row) => row.value_label },
                    { key: 'providers', label: 'Providers', render: (row) => row.secondary_label },
                    { key: 'share', label: 'Share', render: (row) => `${row.share}%` },
                  ]}
                  empty={(
                    <div className="mx-auto flex max-w-[28rem] flex-col items-center gap-2 text-center">
                      <strong className="text-base font-semibold text-zinc-950 dark:text-zinc-50">
                        No model demand yet
                      </strong>
                      <p className="text-sm text-zinc-500 dark:text-zinc-400">
                        Model demand appears after the first request passes through the gateway.
                      </p>
                    </div>
                  )}
                  getKey={(row) => row.id}
                  rows={viewModel?.model_mix ?? []}
                />
              </div>
            </section>
          </div>
          <section className={surfaceClass}>
            <DashboardSectionHeader
              eyebrow="Activity workbench"
              title="Analytics workbench"
              description="A single claw-style workbench for requests, routing evidence, module posture, and next actions without leaving the right-side Portal canvas."
            />

            <div
              className="mt-6 flex flex-wrap gap-2"
              data-slot="portal-dashboard-workbench-tabs"
            >
              {[
                { id: 'requests' as const, label: 'Recent requests' },
                { id: 'routing' as const, label: 'Routing evidence' },
                { id: 'modules' as const, label: 'Module posture' },
                { id: 'actions' as const, label: 'Next actions' },
              ].map((tab) => {
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

            <div className="mt-6 space-y-4">
              {activeWorkbenchTab === 'requests' ? (
                <div className="flex flex-wrap gap-2">
                  {[
                    { id: 'all' as const, label: 'All requests' },
                  { id: 'high_spend' as const, label: 'High spend' },
                  { id: 'latest' as const, label: 'Latest first' },
                ].map((option) => (
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

              {activeWorkbenchTab === 'routing' && viewModel?.routing_posture ? (
                <>
                  <div className="grid gap-4 lg:grid-cols-4">
                    <div className="rounded-[1.5rem] border border-zinc-200/70 bg-zinc-50/70 p-5 dark:border-white/6 dark:bg-zinc-950/35">
                      <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                        Strategy
                      </div>
                      <div className="mt-2 text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                        {viewModel.routing_posture.strategy_label}
                      </div>
                    </div>
                    <div className="rounded-[1.5rem] border border-zinc-200/70 bg-zinc-50/70 p-5 dark:border-white/6 dark:bg-zinc-950/35">
                      <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                        Selected provider
                      </div>
                      <div className="mt-2 text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                        {viewModel.routing_posture.selected_provider}
                      </div>
                    </div>
                    <div className="rounded-[1.5rem] border border-zinc-200/70 bg-zinc-50/70 p-5 dark:border-white/6 dark:bg-zinc-950/35">
                      <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                        Preferred region
                      </div>
                      <div className="mt-2 text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                        {viewModel.routing_posture.preferred_region}
                      </div>
                    </div>
                    <div className="rounded-[1.5rem] border border-zinc-200/70 bg-zinc-50/70 p-5 dark:border-white/6 dark:bg-zinc-950/35">
                      <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                        Evidence count
                      </div>
                      <div className="mt-2 text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                        {viewModel.routing_posture.evidence_count}
                      </div>
                    </div>
                  </div>

                  <div className="rounded-[1.5rem] border border-zinc-200/70 bg-zinc-50/70 px-4 py-4 dark:border-white/6 dark:bg-zinc-950/35">
                    <div className="flex items-center justify-between gap-3">
                      <DashboardStatusPill
                        label={viewModel.routing_posture.title}
                        tone={viewModel.routing_posture.tone}
                      />
                      <InlineButton onClick={() => onNavigate('routing')} tone="ghost">
                        {viewModel.routing_posture.action_label}
                      </InlineButton>
                    </div>
                    <p className="mt-3 text-sm leading-6 text-zinc-500 dark:text-zinc-400">
                      {viewModel.routing_posture.latest_reason}
                    </p>
                  </div>
                </>
              ) : null}

              <DataTable
                columns={[
                  {
                    key: 'primary',
                    label: workbenchConfig.primaryLabel,
                    render: (row) => row.primary,
                  },
                  {
                    key: 'secondary',
                    label: workbenchConfig.secondaryLabel,
                    render: (row) => row.secondary,
                  },
                  {
                    key: 'detail',
                    label: workbenchConfig.detailLabel,
                    render: (row) => row.detail,
                  },
                  {
                    key: 'status',
                    label: workbenchConfig.statusLabel,
                    render: (row) => row.status,
                  },
                  {
                    key: 'action',
                    label: workbenchConfig.actionLabel,
                    render: (row) => row.action,
                  },
                ]}
                empty={(
                  <div className="mx-auto flex max-w-[32rem] flex-col items-center gap-2 text-center">
                    <strong className="text-base font-semibold text-zinc-950 dark:text-zinc-50">
                      {workbenchConfig.emptyTitle}
                    </strong>
                    <p className="text-sm text-zinc-500 dark:text-zinc-400">
                      {workbenchConfig.emptyDetail}
                    </p>
                  </div>
                )}
                getKey={(row) => row.id}
                rows={workbenchConfig.rows}
              />
            </div>
          </section>

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
