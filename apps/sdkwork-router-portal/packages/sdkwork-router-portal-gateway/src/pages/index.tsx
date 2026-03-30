import { useDeferredValue, useEffect, useState } from 'react';
import type { ReactNode } from 'react';

import {
  DataTable,
  EmptyState,
  formatDateTime,
  formatUnits,
  InlineButton,
  Pill,
  Select,
  ToolbarField,
  ToolbarInline,
  ToolbarSearchField,
} from 'sdkwork-router-portal-commons';

import {
  GatewayLaunchReadinessPanel,
  GatewayModeGrid,
  GatewayPostureGrid,
  GatewayReadinessGrid,
  GatewayRuntimeControlsGrid,
  GatewayTopologyGrid,
  Surface,
} from '../components';
import {
  loadGatewayCommandCenterSnapshot,
  restartGatewayCommandCenterDesktopRuntime,
} from '../repository';
import type { GatewayCommandCenterSnapshot, PortalGatewayPageProps } from '../types';

type GatewayWorkbenchLane =
  | 'compatibility'
  | 'rate-limit-policies'
  | 'rate-limit-windows'
  | 'service-health'
  | 'verification';

type GatewayWorkbenchRow = {
  id: string;
  focus: string;
  subject: ReactNode;
  scope: ReactNode;
  meter: ReactNode;
  status: ReactNode;
  detail: ReactNode;
  searchText: string;
};

type GatewayWorkbenchConfig = {
  laneLabel: string;
  focusOptions: Array<{ value: string; label: string }>;
  subjectLabel: string;
  scopeLabel: string;
  meterLabel: string;
  detailLabel: string;
  detail: string;
  emptyTitle: string;
  emptyDetail: string;
};

const WORKBENCH_LANE_OPTIONS: Array<{ value: GatewayWorkbenchLane; label: string }> = [
  { value: 'service-health', label: 'Service health' },
  { value: 'compatibility', label: 'Compatibility routes' },
  { value: 'rate-limit-policies', label: 'Rate-limit policies' },
  { value: 'rate-limit-windows', label: 'Rate-limit windows' },
  { value: 'verification', label: 'Verification commands' },
];

function includesQuery(query: string, values: Array<string | number | null | undefined>): boolean {
  if (!query) {
    return true;
  }

  return values
    .filter((value) => value !== null && value !== undefined)
    .join(' ')
    .toLowerCase()
    .includes(query);
}

function serviceHealthLabel(status: GatewayCommandCenterSnapshot['serviceHealthChecks'][number]['status']) {
  if (status === 'healthy') {
    return 'Healthy';
  }

  if (status === 'degraded') {
    return 'Degraded';
  }

  return 'Unreachable';
}

function serviceHealthTone(status: GatewayCommandCenterSnapshot['serviceHealthChecks'][number]['status']) {
  return status === 'healthy' ? 'positive' : 'warning';
}

function rateLimitTone(enabled: boolean, exceeded: boolean) {
  if (!enabled || exceeded) {
    return 'warning';
  }

  return 'positive';
}

function rateLimitScopeLabel(input: {
  api_key_hash?: string | null;
  route_key?: string | null;
  model_name?: string | null;
}): string {
  const parts = [
    input.api_key_hash ? `key:${input.api_key_hash.slice(0, 12)}...` : null,
    input.route_key ? `route:${input.route_key}` : null,
    input.model_name ? `model:${input.model_name}` : null,
  ].filter(Boolean);

  return parts.length ? parts.join(' / ') : 'project-wide';
}

function verificationFocus(routeFamily: string): string {
  if (routeFamily.includes('/v1/messages')) {
    return 'anthropic';
  }

  if (routeFamily.includes('generateContent')) {
    return 'gemini';
  }

  return 'openai';
}

function verificationTone(focus: string) {
  if (focus === 'anthropic') {
    return 'accent';
  }

  if (focus === 'gemini') {
    return 'seed';
  }

  return 'positive';
}

function formatLatency(latencyMs?: number | null): string {
  if (latencyMs === null || latencyMs === undefined) {
    return 'No latency sample';
  }

  return `${latencyMs} ms`;
}

function workbenchConfig(
  snapshot: GatewayCommandCenterSnapshot,
  lane: GatewayWorkbenchLane,
): GatewayWorkbenchConfig {
  switch (lane) {
    case 'compatibility':
      return {
        laneLabel: 'Compatibility routes',
        focusOptions: [
          { value: 'all', label: 'All routes' },
          { value: 'direct', label: 'Direct gateway' },
          { value: 'translated', label: 'Translated routes' },
          { value: 'desktop', label: 'Desktop setup' },
        ],
        subjectLabel: 'Tool',
        scopeLabel: 'Route family',
        meterLabel: 'Execution truth',
        detailLabel: 'Operator outcome',
        detail:
          'Compatibility routes keep Codex, Claude Code, Gemini, and OpenClaw onboarding on one shared gateway posture.',
        emptyTitle: 'No compatibility routes in this slice',
        emptyDetail:
          'Adjust the workbench lane or search to reveal a different protocol family.',
      };
    case 'rate-limit-policies':
      return {
        laneLabel: 'Rate-limit policies',
        focusOptions: [
          { value: 'all', label: 'All policies' },
          { value: 'enabled', label: 'Enabled' },
          { value: 'disabled', label: 'Disabled' },
        ],
        subjectLabel: 'Policy',
        scopeLabel: 'Scope',
        meterLabel: 'Limit',
        detailLabel: 'Operator notes',
        detail: `Project rate-limit policy posture was last checked ${formatDateTime(snapshot.rateLimitSnapshot.generated_at_ms)}.`,
        emptyTitle: 'No rate-limit policies in this slice',
        emptyDetail:
          'The workspace does not currently expose a matching project-scoped rate-limit policy.',
      };
    case 'rate-limit-windows':
      return {
        laneLabel: 'Rate-limit windows',
        focusOptions: [
          { value: 'all', label: 'All windows' },
          { value: 'within-limit', label: 'Within limit' },
          { value: 'over-limit', label: 'Over limit' },
          { value: 'disabled', label: 'Disabled' },
        ],
        subjectLabel: 'Window',
        scopeLabel: 'Scope',
        meterLabel: 'Usage',
        detailLabel: 'Window detail',
        detail: `Live rate-limit windows were last checked ${formatDateTime(snapshot.rateLimitSnapshot.generated_at_ms)}.`,
        emptyTitle: 'No live windows in this slice',
        emptyDetail: 'Live rate-limit pressure will appear here once gateway activity is present.',
      };
    case 'verification':
      return {
        laneLabel: 'Verification commands',
        focusOptions: [
          { value: 'all', label: 'All commands' },
          { value: 'openai', label: 'OpenAI-compatible' },
          { value: 'anthropic', label: 'Anthropic Messages' },
          { value: 'gemini', label: 'Gemini' },
        ],
        subjectLabel: 'Check',
        scopeLabel: 'Route family',
        meterLabel: 'Protocol',
        detailLabel: 'Verification command',
        detail:
          'Verification commands turn gateway activation into an executable launch checklist instead of static documentation.',
        emptyTitle: 'No verification commands in this slice',
        emptyDetail: 'Change the focus or search to reveal another verification route family.',
      };
    case 'service-health':
    default:
      return {
        laneLabel: 'Service health',
        focusOptions: [
          { value: 'all', label: 'All services' },
          { value: 'healthy', label: 'Healthy' },
          { value: 'degraded', label: 'Degraded' },
          { value: 'unreachable', label: 'Unreachable' },
        ],
        subjectLabel: 'Service',
        scopeLabel: 'Health route',
        meterLabel: 'Runtime signal',
        detailLabel: 'Operator detail',
        detail: `Live service health was last checked ${formatDateTime(snapshot.runtimeHealth.checkedAtMs)}.`,
        emptyTitle: 'No service health checks in this slice',
        emptyDetail: 'Refresh service health to pull the latest runtime evidence into the command workbench.',
      };
  }
}

function buildWorkbenchRows(
  snapshot: GatewayCommandCenterSnapshot,
  lane: GatewayWorkbenchLane,
): GatewayWorkbenchRow[] {
  switch (lane) {
    case 'compatibility':
      return snapshot.compatibilityRows.map((row) => {
        const focus = row.truth.includes('translated')
          ? 'translated'
          : row.truth.includes('desktop-assisted')
            ? 'desktop'
            : 'direct';

        return {
          id: row.id,
          focus,
          subject: (
            <div className="space-y-1">
              <strong className="text-zinc-950 dark:text-zinc-50">{row.tool}</strong>
              <p className="text-xs text-zinc-500 dark:text-zinc-400">{row.protocol}</p>
            </div>
          ),
          scope: row.routeFamily,
          meter: <Pill tone={focus === 'translated' ? 'accent' : focus === 'desktop' ? 'seed' : 'positive'}>{row.truth}</Pill>,
          status: (
            <Pill tone={focus === 'translated' ? 'accent' : focus === 'desktop' ? 'seed' : 'positive'}>
              {focus === 'translated' ? 'Translated' : focus === 'desktop' ? 'Desktop setup' : 'Direct'}
            </Pill>
          ),
          detail: <p className="max-w-[30rem] leading-6">{row.outcome}</p>,
          searchText: [
            row.tool,
            row.protocol,
            row.routeFamily,
            row.truth,
            row.outcome,
          ]
            .join(' ')
            .toLowerCase(),
        };
      });
    case 'rate-limit-policies':
      return snapshot.rateLimitSnapshot.policies.map((policy) => ({
        id: policy.policy_id,
        focus: policy.enabled ? 'enabled' : 'disabled',
        subject: (
          <div className="space-y-1">
            <strong className="text-zinc-950 dark:text-zinc-50">{policy.policy_id}</strong>
            <p className="text-xs text-zinc-500 dark:text-zinc-400">
              Updated {formatDateTime(policy.updated_at_ms)}
            </p>
          </div>
        ),
        scope: rateLimitScopeLabel(policy),
        meter: `${formatUnits(policy.limit_requests)} req / ${policy.window_seconds}s · burst ${formatUnits(policy.burst_requests || policy.requests_per_window)}`,
        status: <Pill tone={policy.enabled ? 'positive' : 'warning'}>{policy.enabled ? 'Enabled' : 'Disabled'}</Pill>,
        detail: policy.notes ?? 'No operator notes were attached to this policy.',
        searchText: [
          policy.policy_id,
          policy.project_id,
          policy.api_key_hash,
          policy.route_key,
          policy.model_name,
          policy.notes,
        ]
          .filter(Boolean)
          .join(' ')
          .toLowerCase(),
      }));
    case 'rate-limit-windows':
      return snapshot.rateLimitSnapshot.windows.map((window) => {
        const focus = !window.enabled ? 'disabled' : window.exceeded ? 'over-limit' : 'within-limit';

        return {
          id: `${window.policy_id}:${window.window_start_ms}`,
          focus,
          subject: (
            <div className="space-y-1">
              <strong className="text-zinc-950 dark:text-zinc-50">{window.policy_id}</strong>
              <p className="text-xs text-zinc-500 dark:text-zinc-400">
                Started {formatDateTime(window.window_start_ms)}
              </p>
            </div>
          ),
          scope: rateLimitScopeLabel(window),
          meter: `${formatUnits(window.request_count)} / ${formatUnits(window.limit_requests)} · ${formatUnits(window.remaining_requests)} remaining`,
          status: (
            <Pill tone={rateLimitTone(window.enabled, window.exceeded)}>
              {!window.enabled ? 'Disabled' : window.exceeded ? 'Over limit' : 'Within limit'}
            </Pill>
          ),
          detail: `${window.window_seconds}s window · ends ${formatDateTime(window.window_end_ms)}`,
          searchText: [
            window.policy_id,
            window.project_id,
            window.api_key_hash,
            window.route_key,
            window.model_name,
            String(window.request_count),
            String(window.limit_requests),
          ]
            .filter(Boolean)
            .join(' ')
            .toLowerCase(),
        };
      });
    case 'verification':
      return snapshot.verificationSnippets.map((snippet) => {
        const focus = verificationFocus(snippet.routeFamily);

        return {
          id: snippet.id,
          focus,
          subject: (
            <div className="space-y-1">
              <strong className="text-zinc-950 dark:text-zinc-50">{snippet.title}</strong>
              <p className="text-xs text-zinc-500 dark:text-zinc-400">{snapshot.gatewayBaseUrl}</p>
            </div>
          ),
          scope: snippet.routeFamily,
          meter: <Pill tone={verificationTone(focus)}>{focus === 'openai' ? 'OpenAI-compatible' : focus === 'anthropic' ? 'Anthropic Messages' : 'Gemini'}</Pill>,
          status: <Pill tone="seed">Ready to run</Pill>,
          detail: (
            <pre className="max-w-[34rem] overflow-x-auto rounded-2xl bg-zinc-950 p-3 text-xs leading-6 text-zinc-300">
              <code>{snippet.command}</code>
            </pre>
          ),
          searchText: [snippet.title, snippet.routeFamily, snippet.command].join(' ').toLowerCase(),
        };
      });
    case 'service-health':
    default:
      return snapshot.serviceHealthChecks.map((check) => ({
        id: check.id,
        focus: check.status,
        subject: (
          <div className="space-y-1">
            <strong className="text-zinc-950 dark:text-zinc-50">{check.label}</strong>
            <p className="text-xs text-zinc-500 dark:text-zinc-400">
              {snapshot.runtimeHealth.mode} mode
            </p>
          </div>
        ),
        scope: (
          <code className="break-all text-xs text-zinc-600 dark:text-zinc-300">
            {check.healthUrl}
          </code>
        ),
        meter: `HTTP ${check.httpStatus ?? 'n/a'} · ${formatLatency(check.responseTimeMs)}`,
        status: <Pill tone={serviceHealthTone(check.status)}>{serviceHealthLabel(check.status)}</Pill>,
        detail: check.detail,
        searchText: [check.label, check.status, check.healthUrl, check.detail].join(' ').toLowerCase(),
      }));
  }
}

export function PortalGatewayPage({ onNavigate }: PortalGatewayPageProps) {
  const [snapshot, setSnapshot] = useState<GatewayCommandCenterSnapshot | null>(null);
  const [status, setStatus] = useState(
    'Loading the router product command center and current launch posture...',
  );
  const [refreshing, setRefreshing] = useState(false);
  const [restartingRuntime, setRestartingRuntime] = useState(false);
  const [workbenchLane, setWorkbenchLane] = useState<GatewayWorkbenchLane>('service-health');
  const [focusFilter, setFocusFilter] = useState('all');
  const [searchQuery, setSearchQuery] = useState('');
  const deferredSearch = useDeferredValue(searchQuery.trim().toLowerCase());

  useEffect(() => {
    let cancelled = false;

    const loadSnapshot = async () => {
      try {
        const nextSnapshot = await loadGatewayCommandCenterSnapshot();
        if (cancelled) {
          return;
        }

        setSnapshot(nextSnapshot);
        setStatus(
          'The portal now exposes compatibility, deployment modes, runtime evidence, and commercial runway as one operator-facing product surface.',
        );
      } catch (error) {
        if (cancelled) {
          return;
        }

        setStatus(
          error instanceof Error
            ? error.message
            : 'The command center could not load the current gateway posture.',
        );
      }
    };

    void loadSnapshot();

    return () => {
      cancelled = true;
    };
  }, []);

  const refreshCommandCenter = async (nextStatus: string) => {
    setRefreshing(true);
    setStatus(nextStatus);

    try {
      const nextSnapshot = await loadGatewayCommandCenterSnapshot();
      setSnapshot(nextSnapshot);
      setStatus(
        'The command center is showing the latest compatibility, runtime, and commercial posture.',
      );
    } catch (error) {
      setStatus(
        error instanceof Error
          ? error.message
          : 'The command center could not refresh the current gateway posture.',
      );
    } finally {
      setRefreshing(false);
    }
  };

  const handleRuntimeControl = async (action: 'restart-desktop-runtime') => {
    if (action !== 'restart-desktop-runtime') {
      return;
    }

    setRestartingRuntime(true);
    setStatus('Restarting the embedded desktop runtime and refreshing live service posture...');

    try {
      const nextSnapshot = await restartGatewayCommandCenterDesktopRuntime();
      setSnapshot(nextSnapshot);
      setStatus(
        'Desktop runtime restarted successfully and the command center has been refreshed with the latest service posture.',
      );
    } catch (error) {
      setStatus(
        error instanceof Error
          ? error.message
          : 'Desktop runtime restart failed before the command center could refresh.',
      );
    } finally {
      setRestartingRuntime(false);
    }
  };

  if (!snapshot) {
    return (
      <Surface detail={status} title="Gateway posture">
        <EmptyState
          detail="The command center will appear once the portal finishes assembling the product-facing router view."
          title="Preparing gateway command center"
        />
      </Surface>
    );
  }

  const config = workbenchConfig(snapshot, workbenchLane);
  const allRows = buildWorkbenchRows(snapshot, workbenchLane);
  const visibleRows = allRows.filter(
    (row) =>
      (focusFilter === 'all' || row.focus === focusFilter)
      && includesQuery(deferredSearch, [row.searchText]),
  );
  const focusLabel =
    config.focusOptions.find((option) => option.value === focusFilter)?.label ?? 'All';

  return (
    <div className="grid gap-4">
      <Surface
        detail={status}
        actions={(
          <div className="flex flex-wrap gap-2">
            <InlineButton
              disabled={refreshing || restartingRuntime}
              onClick={() => {
                void refreshCommandCenter('Refreshing the full command center posture...');
              }}
              tone="secondary"
            >
              {refreshing ? 'Refreshing command center...' : 'Refresh command center'}
            </InlineButton>
          </div>
        )}
        title="Gateway posture"
      >
        <GatewayPostureGrid cards={snapshot.postureCards} />
      </Surface>

      <Surface
        detail={`${snapshot.launchReadiness.detail} Critical blockers and watchpoints stay visible before launch traffic expands.`}
        title="Launch readiness"
      >
        <GatewayLaunchReadinessPanel readiness={snapshot.launchReadiness} />
      </Surface>

      <Surface
        detail={config.detail}
        actions={(
          <div className="flex flex-wrap gap-2">
            <InlineButton
              disabled={refreshing || restartingRuntime}
              onClick={() => {
                void refreshCommandCenter('Refreshing service health and gateway evidence...');
              }}
              tone="secondary"
            >
              {refreshing ? 'Refreshing service health...' : 'Refresh service health'}
            </InlineButton>
          </div>
        )}
        title="Command workbench"
      >
        <div className="grid gap-4">
          <div className="flex flex-wrap items-center gap-3 text-sm text-zinc-500 dark:text-zinc-400">
            <Pill tone="seed">{config.laneLabel}</Pill>
            <span>{`${formatUnits(visibleRows.length)} of ${formatUnits(allRows.length)} rows visible`}</span>
            <span>{`Focus: ${focusLabel}`}</span>
          </div>

          <p className="text-sm leading-6 text-zinc-600 dark:text-zinc-300">
            Verification commands cover <code>/api/v1/models</code>, <code>/v1/messages</code>,
            and <code>generateContent</code> so each compatibility family can be checked from one
            workbench.
          </p>

          <ToolbarInline
            data-slot="portal-gateway-filter-bar"
          >
            <ToolbarSearchField
              label="Search gateway evidence"
              value={searchQuery}
              onChange={(event) => setSearchQuery(event.target.value)}
              placeholder="Search gateway evidence"
              className="min-w-[15rem] flex-[0_1_20rem]"
            />
            <ToolbarField label="Workbench lane" className="min-w-[12rem] shrink-0">
              <Select
                value={workbenchLane}
                onChange={(event) => {
                  setWorkbenchLane(event.target.value as GatewayWorkbenchLane);
                  setFocusFilter('all');
                }}
              >
                {WORKBENCH_LANE_OPTIONS.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </Select>
            </ToolbarField>

            <ToolbarField label="Operational focus" className="min-w-[12rem] shrink-0">
              <Select
                value={focusFilter}
                onChange={(event) => setFocusFilter(event.target.value)}
              >
                {config.focusOptions.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </Select>
            </ToolbarField>

            <div className="ml-auto flex shrink-0 items-center gap-2.5 whitespace-nowrap">
              <InlineButton
                onClick={() => {
                  setFocusFilter('all');
                  setSearchQuery('');
                }}
                tone="secondary"
              >
                Clear filters
              </InlineButton>
            </div>
          </ToolbarInline>

          <DataTable
            columns={[
              { key: 'subject', label: config.subjectLabel, render: (row) => row.subject },
              { key: 'scope', label: config.scopeLabel, render: (row) => row.scope },
              { key: 'meter', label: config.meterLabel, render: (row) => row.meter },
              { key: 'status', label: 'Status', render: (row) => row.status },
              { key: 'detail', label: config.detailLabel, render: (row) => row.detail },
            ]}
            empty={(
              <div className="mx-auto flex max-w-[34rem] flex-col items-center gap-2 text-center">
                <strong className="text-base font-semibold text-zinc-950 dark:text-zinc-50">
                  {config.emptyTitle}
                </strong>
                <p className="text-sm text-zinc-500 dark:text-zinc-400">
                  {config.emptyDetail}
                </p>
              </div>
            )}
            getKey={(row) => row.id}
            rows={visibleRows}
          />
        </div>
      </Surface>

      <Surface
        detail="Desktop runtime cards keep the local bind story visible while Restart desktop runtime remains intentionally narrow."
        title="Desktop runtime"
      >
        <div className="grid gap-6 xl:grid-cols-[1.15fr_0.85fr]">
          <div className="grid gap-4">
            <GatewayPostureGrid cards={snapshot.runtimeCards} />
          </div>
          <div className="grid gap-4">
            <GatewayRuntimeControlsGrid
              busyAction={restartingRuntime ? 'restart-desktop-runtime' : null}
              controls={snapshot.runtimeControls}
              onAction={(action) => {
                void handleRuntimeControl(action);
              }}
            />
          </div>
        </div>
      </Surface>

      <Surface
        detail="Mode switchboard and topology playbooks keep the path from desktop mode to hosted server mode explicit."
        title="Deployment playbooks"
      >
        <div className="grid gap-6 xl:grid-cols-[1.05fr_0.95fr]">
          <section className="grid gap-4">
            <div className="space-y-2">
              <strong className="text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                Mode switchboard
              </strong>
              <p className="text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                Keep the product launch path readable whether the router is running on one machine
                or transitioning into a hosted topology.
              </p>
            </div>
            <GatewayModeGrid cards={snapshot.modeCards} />
          </section>

          <section className="grid gap-4">
            <div className="space-y-2">
              <strong className="text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                Topology playbooks
              </strong>
              <p className="text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                Promote runtime documentation into executable rollout playbooks that operators can
                apply immediately.
              </p>
            </div>
            <GatewayTopologyGrid playbooks={snapshot.topologyPlaybooks} />
          </section>
        </div>
      </Surface>

      <Surface
        detail="Commerce catalog and launch actions keep access, routing, and billing runway on one commercial surface."
        title="Commercial runway"
      >
        <div className="grid gap-6 xl:grid-cols-[1.05fr_0.95fr]">
          <section className="grid gap-4">
            <div className="space-y-2">
              <strong className="text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                Commerce catalog
              </strong>
              <p className="text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                Active membership, recharge packs, and coupon campaigns remain visible as backend
                product inventory instead of drifting into frontend-only launch copy.
              </p>
            </div>
            <GatewayPostureGrid cards={snapshot.commerceCatalogCards} />
          </section>

          <section className="grid gap-4">
            <div className="space-y-2">
              <strong className="text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                Launch actions
              </strong>
              <p className="text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                Open API Keys, Open Routing, and Open Billing are the three fastest actions for
                turning this command center into a real launch workflow.
              </p>
            </div>
            <GatewayReadinessGrid actions={snapshot.readinessActions} onNavigate={onNavigate} />
          </section>
        </div>
      </Surface>
    </div>
  );
}
