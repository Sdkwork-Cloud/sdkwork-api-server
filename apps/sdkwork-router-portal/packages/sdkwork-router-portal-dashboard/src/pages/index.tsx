import { useEffect, useState } from 'react';
import {
  Area,
  AreaChart,
  Bar,
  BarChart,
  CartesianGrid,
  Cell,
  Pie,
  PieChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts';
import {
  DataTable,
  EmptyState,
  formatCurrency,
  formatDateTime,
  formatUnits,
  InlineButton,
  MetricCard,
  Pill,
  Surface,
} from 'sdkwork-router-portal-commons';
import { portalErrorMessage } from 'sdkwork-router-portal-portal-api';

import { DashboardBreakdownList, DashboardInsights } from '../components';
import { loadPortalDashboardSnapshot } from '../repository';
import { buildPortalDashboardViewModel } from '../services';
import type { DashboardInsight, PortalDashboardPageProps, PortalDashboardPageViewModel } from '../types';

const chartPalette = [
  'rgb(var(--portal-accent-rgb))',
  '#30b889',
  '#f6b73c',
  '#f56d91',
  '#9e7bff',
];

function asNumber(value: string | number | readonly (string | number)[] | undefined): number {
  if (Array.isArray(value)) {
    return asNumber(value[0]);
  }

  if (typeof value === 'number') {
    return value;
  }

  if (typeof value === 'string') {
    const parsed = Number(value);
    return Number.isFinite(parsed) ? parsed : 0;
  }

  return 0;
}

function buildTrafficHeadline(viewModel: PortalDashboardPageViewModel): {
  title: string;
  detail: string;
} {
  const { snapshot, provider_mix: providerMix, model_mix: modelMix } = viewModel;

  if (snapshot.usage_summary.total_requests === 0) {
    return {
      title: 'Waiting for the first API request',
      detail: 'As soon as real traffic lands, the dashboard will summarize provider concentration, model mix, and booked spend here.',
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

function activityTone(item: DashboardInsight | { tone: 'accent' | 'positive' | 'warning' | 'default' }) {
  return item.tone;
}

export function PortalDashboardPage({ onNavigate, initialSnapshot }: PortalDashboardPageProps) {
  const [viewModel, setViewModel] = useState<PortalDashboardPageViewModel | null>(
    initialSnapshot ? buildPortalDashboardViewModel(initialSnapshot) : null,
  );
  const [status, setStatus] = useState(
    initialSnapshot
      ? 'Refreshing routing and activity data for the current workspace.'
      : 'Loading your workspace overview.',
  );

  useEffect(() => {
    let cancelled = false;

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
        setStatus('Live traffic, routing, and spend telemetry are up to date.');
      })
      .catch((error) => {
        if (!cancelled) {
          setStatus(portalErrorMessage(error));
        }
      });

    return () => {
      cancelled = true;
    };
  }, [initialSnapshot]);

  const snapshot = viewModel?.snapshot ?? initialSnapshot ?? null;
  const trafficHeadline = viewModel ? buildTrafficHeadline(viewModel) : null;

  return (
    <>
      <div className="portalx-status-row portalx-dashboard-status-row">
        <Pill tone={viewModel?.insights[0]?.tone ?? 'accent'}>{snapshot ? 'Live overview' : 'Loading'}</Pill>
        <span className="portalx-status-copy">{status}</span>
        {snapshot ? <Pill tone="default">Tenant {snapshot.workspace.tenant.name}</Pill> : null}
        <InlineButton onClick={() => onNavigate('routing')} tone="secondary">
          Open routing
        </InlineButton>
        <InlineButton onClick={() => onNavigate('api-keys')} tone="primary">
          Manage keys
        </InlineButton>
      </div>

      {viewModel?.insights.length ? <DashboardInsights insights={viewModel.insights} onNavigate={onNavigate} /> : null}

      <div className="portalx-metric-grid portalx-metric-grid-dense">
        {(viewModel?.metrics ?? []).map((metric) => (
          <MetricCard detail={metric.detail} key={metric.id} label={metric.label} value={metric.value} />
        ))}
      </div>

      <div className="portalx-dashboard-grid">
        <div className="portalx-dashboard-main">
          <Surface
            actions={
              <InlineButton onClick={() => onNavigate('usage')} tone="ghost">
                Open usage
              </InlineButton>
            }
            detail="Requests, spend, and demand concentration across the visible workspace window."
            title="Traffic overview"
          >
            {viewModel && snapshot ? (
              <div className="grid gap-6">
                <div className="portalx-dashboard-summary-callout">
                  <strong>{trafficHeadline?.title}</strong>
                  <p>{trafficHeadline?.detail}</p>
                </div>

                <div className="grid gap-4 xl:grid-cols-2">
                  <section className="portal-shell-chart-surface">
                    <div className="mb-4 flex items-start justify-between gap-3">
                      <div>
                        <h3 className="portal-shell-info-title">Request volume</h3>
                        <p className="portal-shell-info-copy text-sm">Visible request count grouped by recent activity buckets.</p>
                      </div>
                      <Pill tone="accent">{formatUnits(snapshot.usage_summary.total_requests)} requests</Pill>
                    </div>
                    {viewModel.request_volume_series.length ? (
                      <div className="h-72">
                        <ResponsiveContainer width="100%" height="100%">
                          <AreaChart data={viewModel.request_volume_series}>
                            <defs>
                              <linearGradient id="portalx-requests-fill" x1="0" y1="0" x2="0" y2="1">
                                <stop offset="5%" stopColor="rgb(var(--portal-accent-rgb))" stopOpacity={0.4} />
                                <stop offset="95%" stopColor="rgb(var(--portal-accent-rgb))" stopOpacity={0} />
                              </linearGradient>
                            </defs>
                            <CartesianGrid stroke="var(--portal-chart-grid)" vertical={false} />
                            <XAxis dataKey="bucket" stroke="var(--portal-chart-axis)" tickLine={false} axisLine={false} />
                            <YAxis stroke="var(--portal-chart-axis)" tickLine={false} axisLine={false} allowDecimals={false} />
                            <Tooltip
                              contentStyle={{
                                background: 'var(--portal-chart-tooltip-background)',
                                border: '1px solid var(--portal-chart-tooltip-border)',
                                borderRadius: '16px',
                                color: 'var(--portal-text-primary)',
                              }}
                            />
                            <Area
                              type="monotone"
                              dataKey="requests"
                              stroke="rgb(var(--portal-accent-rgb))"
                              fillOpacity={1}
                              fill="url(#portalx-requests-fill)"
                              strokeWidth={2}
                            />
                          </AreaChart>
                        </ResponsiveContainer>
                      </div>
                    ) : (
                      <EmptyState
                        detail="Request volume appears after the first API traffic is recorded."
                        title="No request trend yet"
                      />
                    )}
                  </section>

                  <section className="portal-shell-chart-surface">
                    <div className="mb-4 flex items-start justify-between gap-3">
                      <div>
                        <h3 className="portal-shell-info-title">Booked spend trend</h3>
                        <p className="portal-shell-info-copy text-sm">Recent booked amount grouped by the same activity buckets.</p>
                      </div>
                      <Pill tone="positive">{formatCurrency(snapshot.billing_summary.booked_amount)}</Pill>
                    </div>
                    {viewModel.spend_series.length ? (
                      <div className="h-72">
                        <ResponsiveContainer width="100%" height="100%">
                          <AreaChart data={viewModel.spend_series}>
                            <defs>
                              <linearGradient id="portalx-spend-fill" x1="0" y1="0" x2="0" y2="1">
                                <stop offset="5%" stopColor="#30b889" stopOpacity={0.35} />
                                <stop offset="95%" stopColor="#30b889" stopOpacity={0} />
                              </linearGradient>
                            </defs>
                            <CartesianGrid stroke="var(--portal-chart-grid)" vertical={false} />
                            <XAxis dataKey="bucket" stroke="var(--portal-chart-axis)" tickLine={false} axisLine={false} />
                            <YAxis stroke="var(--portal-chart-axis)" tickLine={false} axisLine={false} />
                            <Tooltip
                              formatter={(value) => formatCurrency(asNumber(value))}
                              contentStyle={{
                                background: 'var(--portal-chart-tooltip-background)',
                                border: '1px solid var(--portal-chart-tooltip-border)',
                                borderRadius: '16px',
                                color: 'var(--portal-text-primary)',
                              }}
                            />
                            <Area
                              type="monotone"
                              dataKey="amount"
                              stroke="#30b889"
                              fillOpacity={1}
                              fill="url(#portalx-spend-fill)"
                              strokeWidth={2}
                            />
                          </AreaChart>
                        </ResponsiveContainer>
                      </div>
                    ) : (
                      <EmptyState
                        detail="Spend trend appears after requests start booking cost."
                        title="No spend trend yet"
                      />
                    )}
                  </section>
                </div>

                <div className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_minmax(0,1.1fr)]">
                  <section className="portal-shell-chart-surface">
                    <div className="mb-4">
                      <h3 className="portal-shell-info-title">Provider share</h3>
                      <p className="portal-shell-info-copy text-sm">How current demand is distributed across provider paths.</p>
                    </div>
                    {viewModel.provider_share_series.length ? (
                      <div className="grid gap-4 lg:grid-cols-[minmax(200px,0.9fr)_minmax(0,1fr)]">
                        <div className="h-64">
                          <ResponsiveContainer width="100%" height="100%">
                            <PieChart>
                              <Pie
                                data={viewModel.provider_share_series}
                                dataKey="value"
                                nameKey="name"
                                innerRadius={54}
                                outerRadius={86}
                                paddingAngle={3}
                              >
                                {viewModel.provider_share_series.map((entry, index) => (
                                  <Cell fill={chartPalette[index % chartPalette.length]} key={entry.name} />
                                ))}
                              </Pie>
                              <Tooltip
                                formatter={(value) => `${formatUnits(asNumber(value))} requests`}
                                contentStyle={{
                                  background: 'var(--portal-chart-tooltip-background)',
                                  border: '1px solid var(--portal-chart-tooltip-border)',
                                  borderRadius: '16px',
                                  color: 'var(--portal-text-primary)',
                                }}
                              />
                            </PieChart>
                          </ResponsiveContainer>
                        </div>
                        <DashboardBreakdownList
                          emptyDetail="Provider share appears after the first request passes through the gateway."
                          emptyTitle="No provider traffic yet"
                          items={viewModel.provider_mix}
                        />
                      </div>
                    ) : (
                      <EmptyState
                        detail="Provider share appears after the first request passes through the gateway."
                        title="No provider traffic yet"
                      />
                    )}
                  </section>

                  <section className="portal-shell-chart-surface">
                    <div className="mb-4 flex items-start justify-between gap-3">
                      <div>
                        <h3 className="portal-shell-info-title">Model demand</h3>
                        <p className="portal-shell-info-copy text-sm">Top models by visible request count in the current workspace window.</p>
                      </div>
                      <Pill tone="default">{formatUnits(snapshot.usage_summary.model_count)} models</Pill>
                    </div>
                    {viewModel.model_demand_series.length ? (
                      <div className="grid gap-4">
                        <div className="h-64">
                          <ResponsiveContainer width="100%" height="100%">
                            <BarChart data={viewModel.model_demand_series} layout="vertical" margin={{ left: 12 }}>
                              <CartesianGrid stroke="var(--portal-chart-grid)" horizontal={false} />
                              <XAxis type="number" stroke="var(--portal-chart-axis)" tickLine={false} axisLine={false} allowDecimals={false} />
                              <YAxis
                                type="category"
                                dataKey="name"
                                width={110}
                                stroke="var(--portal-chart-axis)"
                                tickLine={false}
                                axisLine={false}
                              />
                              <Tooltip
                                formatter={(value) => `${formatUnits(asNumber(value))} requests`}
                                contentStyle={{
                                  background: 'var(--portal-chart-tooltip-background)',
                                  border: '1px solid var(--portal-chart-tooltip-border)',
                                  borderRadius: '16px',
                                  color: 'var(--portal-text-primary)',
                                }}
                              />
                              <Bar dataKey="requests" fill="rgb(var(--portal-accent-rgb))" radius={[0, 10, 10, 0]} />
                            </BarChart>
                          </ResponsiveContainer>
                        </div>
                        <DashboardBreakdownList
                          emptyDetail="Model demand appears after the first request passes through the gateway."
                          emptyTitle="No model demand yet"
                          items={viewModel.model_mix}
                        />
                      </div>
                    ) : (
                      <EmptyState
                        detail="Model demand appears after the first request passes through the gateway."
                        title="No model demand yet"
                      />
                    )}
                  </section>
                </div>
              </div>
            ) : (
              <EmptyState
                detail="Traffic overview will appear once workspace telemetry becomes available."
                title="Preparing traffic overview"
              />
            )}
          </Surface>

          <Surface
            actions={
              <InlineButton onClick={() => onNavigate('usage')} tone="ghost">
                Open usage workbench
              </InlineButton>
            }
            detail="Recent API calls with provider, model, token units, and booked amount."
            title="Recent requests"
          >
            {snapshot?.recent_requests.length ? (
              <DataTable
                columns={[
                  {
                    key: 'model',
                    label: 'Model',
                    render: (row) => row.model,
                  },
                  {
                    key: 'provider',
                    label: 'Provider',
                    render: (row) => row.provider,
                  },
                  {
                    key: 'units',
                    label: 'Token Units',
                    render: (row) => formatUnits(row.units),
                  },
                  {
                    key: 'amount',
                    label: 'Booked',
                    render: (row) => formatCurrency(row.amount),
                  },
                  {
                    key: 'created',
                    label: 'Recorded',
                    render: (row) => formatDateTime(row.created_at_ms),
                  },
                ]}
                empty="No request telemetry recorded for this project yet."
                getKey={(row, index) => `${row.project_id}-${row.model}-${row.created_at_ms}-${index}`}
                rows={snapshot.recent_requests}
              />
            ) : (
              <EmptyState
                detail="Once gateway requests start flowing through your project, per-call token-unit usage will appear here."
                title="No recent requests yet"
              />
            )}
          </Surface>
        </div>

        <div className="portalx-dashboard-side">
          <Surface
            actions={
              <InlineButton
                onClick={() => onNavigate(viewModel?.routing_posture?.route ?? 'routing')}
                tone="ghost"
              >
                {viewModel?.routing_posture?.action_label ?? 'Open routing'}
              </InlineButton>
            }
            detail="Default route, region preference, and live decision evidence in one place."
            title="Routing posture"
          >
            {viewModel?.routing_posture ? (
              <>
                <div className="portalx-status-row">
                  <Pill tone={viewModel.routing_posture.tone}>{viewModel.routing_posture.title}</Pill>
                </div>
                <div className="portalx-summary-grid">
                  <article className="portalx-summary-card">
                    <span>Strategy</span>
                    <strong>{viewModel.routing_posture.strategy_label}</strong>
                    <p>Current project routing strategy.</p>
                  </article>
                  <article className="portalx-summary-card">
                    <span>Selected provider</span>
                    <strong>{viewModel.routing_posture.selected_provider}</strong>
                    <p>Provider currently selected by the route preview.</p>
                  </article>
                  <article className="portalx-summary-card">
                    <span>Preferred region</span>
                    <strong>{viewModel.routing_posture.preferred_region}</strong>
                    <p>Region signal influencing the current route.</p>
                  </article>
                  <article className="portalx-summary-card">
                    <span>Evidence count</span>
                    <strong>{viewModel.routing_posture.evidence_count}</strong>
                    <p>Captured preview and live route decisions.</p>
                  </article>
                </div>
                <div className="portalx-note">
                  <strong>Latest signal</strong>
                  <span>{viewModel.routing_posture.latest_reason}</span>
                </div>
              </>
            ) : (
              <EmptyState
                detail="Routing posture will appear once project routing data becomes available."
                title="Preparing routing posture"
              />
            )}
          </Surface>

          <Surface
            detail="The fastest way to move the workspace forward without scanning every module manually."
            title="Quick actions"
          >
            {viewModel ? (
              <ol className="portalx-queue-list">
                {viewModel.quick_actions.map((item, index) => (
                  <li className="portalx-queue-card" key={item.id}>
                    <div className="portalx-status-row">
                      <Pill tone={item.tone}>{item.title}</Pill>
                    </div>
                    <p>{item.detail}</p>
                    {item.route && item.action_label ? (
                      <InlineButton onClick={() => onNavigate(item.route!)} tone={actionTone(index)}>
                        {item.action_label}
                      </InlineButton>
                    ) : null}
                  </li>
                ))}
              </ol>
            ) : (
              <EmptyState detail="Quick actions will appear when workspace data finishes loading." title="Preparing actions" />
            )}
          </Surface>

          <Surface
            detail="Every core portal area stays one click away with a visible health state."
            title="Workspace modules"
          >
            {viewModel ? (
              <div className="portalx-module-grid">
                {viewModel.modules.map((item) => (
                  <button
                    className="portalx-module-card"
                    key={item.route}
                    onClick={() => onNavigate(item.route)}
                    type="button"
                  >
                    <div className="portalx-status-row">
                      <strong>{item.title}</strong>
                      <Pill tone={item.tone}>{item.status_label}</Pill>
                    </div>
                    <p>{item.detail}</p>
                    <small>{item.action_label}</small>
                  </button>
                ))}
              </div>
            ) : (
              <EmptyState detail="Workspace modules will appear after the dashboard finishes loading." title="Preparing modules" />
            )}
          </Surface>

          <Surface
            detail="A compact timeline of the latest request and routing evidence across the workspace."
            title="Recent activity"
          >
            {viewModel?.activity_feed.length ? (
              <ol className="portalx-queue-list">
                {viewModel.activity_feed.map((item) => (
                  <li className="portalx-queue-card" key={item.id}>
                    <div className="portalx-status-row">
                      <Pill tone={activityTone(item)}>{item.title}</Pill>
                      <span>{item.timestamp_label}</span>
                    </div>
                    <p>{item.detail}</p>
                    {item.route && item.action_label ? (
                      <InlineButton onClick={() => onNavigate(item.route!)} tone="ghost">
                        {item.action_label}
                      </InlineButton>
                    ) : null}
                  </li>
                ))}
              </ol>
            ) : (
              <EmptyState
                detail="Recent activity appears once the workspace records request or routing evidence."
                title="No activity yet"
              />
            )}
          </Surface>
        </div>
      </div>
    </>
  );
}
