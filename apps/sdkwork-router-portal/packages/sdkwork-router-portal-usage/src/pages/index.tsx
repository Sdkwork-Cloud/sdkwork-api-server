import { useEffect, useMemo, useState } from 'react';
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
  Button,
  DataTable,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  EmptyState,
  formatCurrency,
  formatDateTime,
  formatUnits,
  InlineButton,
  Pill,
  Surface,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from 'sdkwork-router-portal-commons';
import { portalErrorMessage } from 'sdkwork-router-portal-portal-api';
import type { UsageRecord, UsageSummary } from 'sdkwork-router-portal-types';

import { UsageFiltersPanel, UsageHighlights } from '../components';
import { loadUsageWorkbenchData } from '../repository';
import { buildUsageWorkbenchViewModel } from '../services';
import type { PortalUsagePageProps, UsageFilters } from '../types';

const chartPalette = ['#4f8cff', '#30b889', '#f6b73c', '#f56d91', '#9e7bff', '#52c7ea'];

const emptySummary: UsageSummary = {
  total_requests: 0,
  project_count: 0,
  model_count: 0,
  provider_count: 0,
  projects: [],
  providers: [],
  models: [],
};

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

export function PortalUsagePage({ onNavigate }: PortalUsagePageProps) {
  const [summary, setSummary] = useState<UsageSummary>(emptySummary);
  const [records, setRecords] = useState<UsageRecord[]>([]);
  const [filters, setFilters] = useState<UsageFilters>({ model: '', provider: '', date_range: '30d' });
  const [status, setStatus] = useState('Loading request telemetry...');
  const [filterDialogOpen, setFilterDialogOpen] = useState(false);

  useEffect(() => {
    let cancelled = false;

    void loadUsageWorkbenchData()
      .then((data) => {
        if (cancelled) {
          return;
        }

        setSummary(data.summary);
        setRecords(data.records);
        setStatus('Per-call request telemetry is filtered to your workspace project.');
      })
      .catch((error) => {
        if (!cancelled) {
          setStatus(portalErrorMessage(error));
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  const viewModel = useMemo(
    () => buildUsageWorkbenchViewModel(summary, records, filters),
    [filters, records, summary],
  );

  const totalTokens = viewModel.filtered_records.reduce((sum, record) => sum + record.total_tokens, 0);
  const totalInputTokens = viewModel.filtered_records.reduce((sum, record) => sum + record.input_tokens, 0);
  const totalOutputTokens = viewModel.filtered_records.reduce((sum, record) => sum + record.output_tokens, 0);

  return (
    <>
      <Dialog open={filterDialogOpen} onOpenChange={setFilterDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Refine usage view</DialogTitle>
            <DialogDescription>
              Narrow the request slice by model, provider, and time window without keeping
              filters permanently expanded on the page.
            </DialogDescription>
          </DialogHeader>
          <UsageFiltersPanel
            filters={filters}
            modelOptions={viewModel.model_options}
            onChange={setFilters}
            providerOptions={viewModel.provider_options}
          />
          <DialogFooter>
            <Button
              onClick={() => setFilters({ model: '', provider: '', date_range: '30d' })}
              type="button"
              variant="ghost"
            >
              Reset filters
            </Button>
            <Button onClick={() => setFilterDialogOpen(false)} type="button">
              Apply view
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Tabs className="grid gap-6" defaultValue="overview">
        <TabsList className="w-full justify-start overflow-x-auto">
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="request-log">Request log</TabsTrigger>
          <TabsTrigger value="demand-mix">Demand mix</TabsTrigger>
        </TabsList>

        <TabsContent className="space-y-6" value="overview">
          <div className="grid gap-6 xl:grid-cols-2">
            <Surface
              actions={
                <div className="flex flex-wrap gap-2">
                  <Button onClick={() => setFilterDialogOpen(true)} variant="secondary">
                    Refine view
                  </Button>
                  <InlineButton onClick={() => onNavigate('billing')} tone="primary">
                    Review billing
                  </InlineButton>
                </div>
              }
              detail={status}
              title="Request volume"
            >
              {viewModel.request_volume_series.length ? (
                <div className="h-72">
                  <ResponsiveContainer width="100%" height="100%">
                    <AreaChart data={viewModel.request_volume_series}>
                      <defs>
                        <linearGradient id="portalx-usage-volume-fill" x1="0" y1="0" x2="0" y2="1">
                          <stop offset="5%" stopColor="#4f8cff" stopOpacity={0.4} />
                          <stop offset="95%" stopColor="#4f8cff" stopOpacity={0.05} />
                        </linearGradient>
                      </defs>
                      <CartesianGrid stroke="rgba(148, 163, 184, 0.12)" vertical={false} />
                      <XAxis dataKey="bucket" stroke="#94a3b8" tickLine={false} axisLine={false} />
                      <YAxis stroke="#94a3b8" tickLine={false} axisLine={false} allowDecimals={false} />
                      <Tooltip
                        contentStyle={{
                          background: '#020617',
                          border: '1px solid rgba(148, 163, 184, 0.15)',
                          borderRadius: '16px',
                        }}
                        formatter={(value) => formatUnits(asNumber(value))}
                      />
                      <Area
                        type="monotone"
                        dataKey="requests"
                        stroke="#4f8cff"
                        fillOpacity={1}
                        fill="url(#portalx-usage-volume-fill)"
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
            </Surface>

            <Surface
              detail="Input and output token flow from the live usage records."
              title="Token flow"
            >
              {viewModel.request_volume_series.length ? (
                <div className="h-72">
                  <ResponsiveContainer width="100%" height="100%">
                    <BarChart data={viewModel.request_volume_series}>
                      <CartesianGrid stroke="rgba(148, 163, 184, 0.12)" vertical={false} />
                      <XAxis dataKey="bucket" stroke="#94a3b8" tickLine={false} axisLine={false} />
                      <YAxis stroke="#94a3b8" tickLine={false} axisLine={false} />
                      <Tooltip
                        contentStyle={{
                          background: '#020617',
                          border: '1px solid rgba(148, 163, 184, 0.15)',
                          borderRadius: '16px',
                        }}
                        formatter={(value) => formatUnits(asNumber(value))}
                      />
                      <Bar dataKey="input_tokens" stackId="tokens" fill="#30b889" radius={[8, 8, 0, 0]} />
                      <Bar dataKey="output_tokens" stackId="tokens" fill="#4f8cff" radius={[8, 8, 0, 0]} />
                    </BarChart>
                  </ResponsiveContainer>
                </div>
              ) : (
                <EmptyState
                  detail="Token flow appears after the first usage records are written."
                  title="No token flow yet"
                />
              )}
            </Surface>
          </div>

          <UsageHighlights highlights={viewModel.highlights} />

          <div className="portalx-split-grid portalx-split-grid-wide">
            <Surface
              detail="A quick read on which providers, models, and recent calls currently shape this traffic slice."
              title="Traffic profile"
            >
              <ul className="portalx-fact-list">
                {viewModel.traffic_profile.map((item) => (
                  <li key={item.id}>
                    <strong>{item.label}</strong>
                    <span>{item.value}</span>
                    <p>{item.detail}</p>
                  </li>
                ))}
              </ul>
            </Surface>

            <Surface
              detail="Translate raw request rows into a cost and burn read before you jump into billing."
              title="Spend watch"
            >
              <ul className="portalx-fact-list">
                {viewModel.spend_watch.map((item) => (
                  <li key={item.id}>
                    <strong>{item.label}</strong>
                    <span>{item.value}</span>
                    <p>{item.detail}</p>
                  </li>
                ))}
              </ul>
            </Surface>
          </div>

          <Surface
            detail="An evidence layer for concentration, spikes, and day-to-day burn stability."
            title="Request diagnostics"
          >
            <div className="portalx-guardrail-list">
              {viewModel.diagnostics.map((item) => (
                <article className="portalx-guardrail-card" key={item.id}>
                  <Pill tone={item.tone}>{item.title}</Pill>
                  <p>{item.detail}</p>
                </article>
              ))}
            </div>
          </Surface>
        </TabsContent>

        <TabsContent className="space-y-6" value="request-log">
          <Surface
            actions={(
              <InlineButton onClick={() => onNavigate('api-keys')} tone="ghost">
                Manage keys
              </InlineButton>
            )}
            detail="Every request row includes metered units, booked amount, and token breakdown when available."
            title="Request log"
          >
            {viewModel.filtered_records.length ? (
              <DataTable
                columns={[
                  { key: 'model', label: 'Model', render: (row) => row.model },
                  { key: 'provider', label: 'Provider', render: (row) => row.provider },
                  { key: 'input', label: 'Input tokens', render: (row) => formatUnits(row.input_tokens) },
                  { key: 'output', label: 'Output tokens', render: (row) => formatUnits(row.output_tokens) },
                  { key: 'total', label: 'Total tokens', render: (row) => formatUnits(row.total_tokens) },
                  { key: 'units', label: 'Token units', render: (row) => formatUnits(row.units) },
                  { key: 'amount', label: 'Booked', render: (row) => formatCurrency(row.amount) },
                  { key: 'time', label: 'Recorded', render: (row) => formatDateTime(row.created_at_ms) },
                ]}
                empty="No request history recorded yet."
                getKey={(row, index) => `${row.created_at_ms}-${row.model}-${index}`}
                rows={viewModel.filtered_records}
              />
            ) : (
              <EmptyState
                detail="Adjust the filters or run a gateway call from your project and the request list will populate here."
                title="No request history for this slice"
              />
            )}
          </Surface>

          <div className="grid gap-6 xl:grid-cols-2">
            <Surface
              detail="A direct view of token composition across the current request slice."
              title="Token accounting"
            >
              <ul className="portalx-fact-list">
                <li>
                  <strong>Input tokens</strong>
                  <span>{formatUnits(totalInputTokens)}</span>
                  <p>Prompt-side tokens observed in the current slice.</p>
                </li>
                <li>
                  <strong>Output tokens</strong>
                  <span>{formatUnits(totalOutputTokens)}</span>
                  <p>Completion-side tokens observed in the current slice.</p>
                </li>
                <li>
                  <strong>Total tokens</strong>
                  <span>{formatUnits(totalTokens)}</span>
                  <p>Combined token count across all visible requests.</p>
                </li>
              </ul>
            </Surface>

            <Surface
              detail="The request workbench should hand off to the next operational or commercial action."
              title="Connected actions"
            >
              <div className="portalx-checklist-grid">
                <article className="portalx-checklist-card">
                  <strong>Review credits if burn pace is rising</strong>
                  <p>Use the credits view to decide whether a coupon top-up is enough for the next traffic window.</p>
                  <InlineButton onClick={() => onNavigate('credits')} tone="primary">
                    Open credits
                  </InlineButton>
                </article>
                <article className="portalx-checklist-card">
                  <strong>Move into billing for sustained growth</strong>
                  <p>If usage is becoming steady instead of experimental, compare the current burn slice against subscription and recharge paths.</p>
                  <InlineButton onClick={() => onNavigate('billing')} tone="secondary">
                    Review billing
                  </InlineButton>
                </article>
              </div>
            </Surface>
          </div>
        </TabsContent>

        <TabsContent className="space-y-6" value="demand-mix">
          <div className="grid gap-6 xl:grid-cols-2">
            <Surface detail="Top provider paths selected for this filtered workspace slice." title="Provider distribution">
              {viewModel.provider_mix.length ? (
                <div className="grid gap-5 xl:grid-cols-[1fr,280px]">
                  <div className="h-72">
                    <ResponsiveContainer width="100%" height="100%">
                      <PieChart>
                        <Pie
                          data={viewModel.provider_mix}
                          dataKey="requests"
                          innerRadius={70}
                          outerRadius={110}
                          paddingAngle={3}
                        >
                          {viewModel.provider_mix.map((entry, index) => (
                            <Cell fill={chartPalette[index % chartPalette.length]} key={entry.id} />
                          ))}
                        </Pie>
                        <Tooltip
                          contentStyle={{
                            background: '#020617',
                            border: '1px solid rgba(148, 163, 184, 0.15)',
                            borderRadius: '16px',
                          }}
                          formatter={(value) => formatUnits(asNumber(value))}
                        />
                      </PieChart>
                    </ResponsiveContainer>
                  </div>
                  <ul className="portalx-fact-list">
                    {viewModel.provider_mix.map((item) => (
                      <li key={item.id}>
                        <strong>{item.label}</strong>
                        <span>{item.share}% share</span>
                        <p>{formatUnits(item.requests)} requests routed through this provider path.</p>
                      </li>
                    ))}
                  </ul>
                </div>
              ) : (
                <EmptyState
                  detail="Provider routing activity appears after the first requests are recorded."
                  title="No provider data yet"
                />
              )}
            </Surface>

            <Surface detail="Top models ranked by request demand inside the active slice." title="Model distribution">
              {viewModel.model_mix.length ? (
                <div className="h-72">
                  <ResponsiveContainer width="100%" height="100%">
                    <BarChart data={viewModel.model_mix}>
                      <CartesianGrid stroke="rgba(148, 163, 184, 0.12)" vertical={false} />
                      <XAxis dataKey="label" stroke="#94a3b8" tickLine={false} axisLine={false} />
                      <YAxis stroke="#94a3b8" tickLine={false} axisLine={false} allowDecimals={false} />
                      <Tooltip
                        contentStyle={{
                          background: '#020617',
                          border: '1px solid rgba(148, 163, 184, 0.15)',
                          borderRadius: '16px',
                        }}
                        formatter={(value) => formatUnits(asNumber(value))}
                      />
                      <Bar dataKey="requests" fill="#4f8cff" radius={[8, 8, 0, 0]} />
                    </BarChart>
                  </ResponsiveContainer>
                </div>
              ) : (
                <EmptyState
                  detail="Model demand appears after the first gateway requests are recorded."
                  title="No model data yet"
                />
              )}
            </Surface>
          </div>

          <Surface
            detail="Use demand concentration to decide whether routing, pricing, or quota actions should happen next."
            title="Mix actions"
          >
            <div className="portalx-checklist-grid">
              <article className="portalx-checklist-card">
                <strong>Adjust routing when provider concentration is intentional</strong>
                <p>Move into Routing when you want to confirm whether the dominant provider is a policy choice or an accidental concentration pattern.</p>
                <InlineButton onClick={() => onNavigate('routing')} tone="primary">
                  Open routing
                </InlineButton>
              </article>
              <article className="portalx-checklist-card">
                <strong>Keep credentials aligned with the active demand path</strong>
                <p>Use API Keys to verify that environment credentials still match the models and providers carrying production traffic.</p>
                <InlineButton onClick={() => onNavigate('api-keys')} tone="secondary">
                  Manage keys
                </InlineButton>
              </article>
            </div>
          </Surface>
        </TabsContent>
      </Tabs>
    </>
  );
}
