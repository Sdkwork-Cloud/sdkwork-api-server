import { useDeferredValue, useState } from 'react';

import {
  DataTable,
  InlineButton,
  PageToolbar,
  Pill,
  StatCard,
  Surface,
} from 'sdkwork-router-admin-commons';
import type { AdminPageProps, ManagedUser } from 'sdkwork-router-admin-types';

type ViewMode = 'all' | 'usage' | 'routing';
type RecentWindow = 'all' | '24h' | '7d' | '30d';

type FilteredPortalTrafficRow = ManagedUser & {
  filtered_request_count: number;
  filtered_usage_units: number;
  filtered_total_tokens: number;
  filtered_amount: number;
};

function formatTimestamp(timestamp: number): string {
  if (!timestamp) {
    return '-';
  }
  return new Date(timestamp).toLocaleString();
}

function formatIsoTimestamp(timestamp: number): string {
  if (!timestamp) {
    return '';
  }
  return new Date(timestamp).toISOString();
}

function recentWindowCutoff(window: RecentWindow): number | null {
  const now = Date.now();
  switch (window) {
    case '24h':
      return now - 24 * 60 * 60 * 1000;
    case '7d':
      return now - 7 * 24 * 60 * 60 * 1000;
    case '30d':
      return now - 30 * 24 * 60 * 60 * 1000;
    case 'all':
    default:
      return null;
  }
}

function csvValue(value: string | number | boolean | null | undefined): string {
  const normalized = value == null ? '' : String(value);
  return `"${normalized.replaceAll('"', '""')}"`;
}

function downloadCsv(
  filename: string,
  headers: string[],
  rows: Array<Array<string | number | boolean | null | undefined>>,
): void {
  const contents = [headers.map(csvValue).join(','), ...rows.map((row) => row.map(csvValue).join(','))].join('\n');
  const blob = new Blob([contents], { type: 'text/csv;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement('a');
  anchor.href = url;
  anchor.download = filename;
  document.body.appendChild(anchor);
  anchor.click();
  anchor.remove();
  URL.revokeObjectURL(url);
}

function userMatchesQuery(user: ManagedUser, query: string): boolean {
  const haystack = [
    user.display_name,
    user.email,
    user.workspace_tenant_id ?? '',
    user.workspace_project_id ?? '',
  ]
    .join(' ')
    .toLowerCase();
  return haystack.includes(query);
}

function sortUnique(values: string[]): string[] {
  return [...new Set(values.filter(Boolean))].sort((left, right) => left.localeCompare(right));
}

export function TrafficPage({ snapshot }: AdminPageProps) {
  const [search, setSearch] = useState('');
  const [mode, setMode] = useState<ViewMode>('all');
  const [recentWindow, setRecentWindow] = useState<RecentWindow>('all');
  const [projectFilter, setProjectFilter] = useState('all');
  const [providerFilter, setProviderFilter] = useState('all');
  const [modelFilter, setModelFilter] = useState('all');
  const [portalUserScope, setPortalUserScope] = useState('all');
  const deferredQuery = useDeferredValue(search.trim().toLowerCase());

  const projectOptions = sortUnique([
    ...snapshot.projects.map((project) => project.id),
    ...snapshot.usageRecords.map((record) => record.project_id),
  ]);
  const providerOptions = sortUnique([
    ...snapshot.providers.map((provider) => provider.id),
    ...snapshot.usageRecords.map((record) => record.provider),
    ...snapshot.routingLogs.map((log) => log.selected_provider_id),
  ]);
  const modelOptions = sortUnique(snapshot.usageRecords.map((record) => record.model));
  const recentCutoff = recentWindowCutoff(recentWindow);
  const scopedProjectId = portalUserScope === 'all'
    ? null
    : snapshot.portalUsers.find((user) => user.id === portalUserScope)?.workspace_project_id ?? '__missing__';

  const filteredUsageRecords = snapshot.usageRecords.filter((record) => {
    if (recentCutoff && record.created_at_ms < recentCutoff) {
      return false;
    }
    if (projectFilter !== 'all' && record.project_id !== projectFilter) {
      return false;
    }
    if (providerFilter !== 'all' && record.provider !== providerFilter) {
      return false;
    }
    if (modelFilter !== 'all' && record.model !== modelFilter) {
      return false;
    }
    if (scopedProjectId && record.project_id !== scopedProjectId) {
      return false;
    }

    const haystack = [record.project_id, record.model, record.provider].join(' ').toLowerCase();
    return haystack.includes(deferredQuery);
  });

  const filteredRoutingLogs = snapshot.routingLogs.filter((log) => {
    if (recentCutoff && log.created_at_ms < recentCutoff) {
      return false;
    }
    if (providerFilter !== 'all' && log.selected_provider_id !== providerFilter) {
      return false;
    }

    const haystack = [
      log.selected_provider_id,
      log.capability,
      log.route_key,
      log.strategy ?? '',
      log.selection_reason ?? '',
      log.requested_region ?? '',
    ]
      .join(' ')
      .toLowerCase();
    return haystack.includes(deferredQuery);
  });

  const usageByProject = filteredUsageRecords.reduce((projects, record) => {
    const entry = projects.get(record.project_id) ?? {
      request_count: 0,
      total_tokens: 0,
      total_units: 0,
      total_amount: 0,
    };

    entry.request_count += 1;
    entry.total_tokens += record.total_tokens;
    entry.total_units += record.units;
    entry.total_amount += record.amount;
    projects.set(record.project_id, entry);
    return projects;
  }, new Map<string, {
    request_count: number;
    total_tokens: number;
    total_units: number;
    total_amount: number;
  }>());

  const filteredPortalUsers: FilteredPortalTrafficRow[] = snapshot.portalUsers
    .map((user) => {
      const projectUsage = usageByProject.get(user.workspace_project_id ?? '');
      return {
        ...user,
        filtered_request_count: projectUsage?.request_count ?? 0,
        filtered_total_tokens: projectUsage?.total_tokens ?? 0,
        filtered_usage_units: projectUsage?.total_units ?? 0,
        filtered_amount: projectUsage?.total_amount ?? 0,
      };
    })
    .filter((user) => {
      if (portalUserScope !== 'all' && user.id !== portalUserScope) {
        return false;
      }
      if (!deferredQuery) {
        return true;
      }
      return userMatchesQuery(user, deferredQuery) || user.filtered_request_count > 0;
    })
    .sort((left, right) => (
      right.filtered_request_count - left.filtered_request_count
      || right.filtered_total_tokens - left.filtered_total_tokens
      || right.filtered_usage_units - left.filtered_usage_units
      || right.filtered_amount - left.filtered_amount
    ))
    .slice(0, 8);

  const projectHotspots = Array.from(usageByProject.entries())
    .map(([project_id, entry]) => ({
      project_id,
      ...entry,
    }))
    .sort((left, right) => (
      right.request_count - left.request_count
      || right.total_tokens - left.total_tokens
      || right.total_amount - left.total_amount
    ))
    .slice(0, 8);

  const filteredUnits = filteredUsageRecords.reduce((sum, record) => sum + record.units, 0);
  const filteredTokens = filteredUsageRecords.reduce((sum, record) => sum + record.total_tokens, 0);
  const filteredAmount = filteredUsageRecords.reduce((sum, record) => sum + record.amount, 0);

  function clearFilters() {
    setSearch('');
    setMode('all');
    setRecentWindow('all');
    setProjectFilter('all');
    setProviderFilter('all');
    setModelFilter('all');
    setPortalUserScope('all');
  }

  function exportUsageCsv() {
    downloadCsv(
      'sdkwork-router-usage-records.csv',
      [
        'project_id',
        'model',
        'provider',
        'input_tokens',
        'output_tokens',
        'total_tokens',
        'units',
        'amount',
        'created_at',
      ],
      filteredUsageRecords.map((record) => [
        record.project_id,
        record.model,
        record.provider,
        record.input_tokens,
        record.output_tokens,
        record.total_tokens,
        record.units,
        record.amount.toFixed(4),
        formatIsoTimestamp(record.created_at_ms),
      ]),
    );
  }

  function exportRoutingCsv() {
    downloadCsv(
      'sdkwork-router-routing-logs.csv',
      [
        'decision_id',
        'selected_provider_id',
        'capability',
        'route_key',
        'strategy',
        'selection_reason',
        'requested_region',
        'slo_applied',
        'slo_degraded',
        'created_at',
      ],
      filteredRoutingLogs.map((log) => [
        log.decision_id,
        log.selected_provider_id,
        log.capability,
        log.route_key,
        log.strategy ?? '',
        log.selection_reason ?? '',
        log.requested_region ?? '',
        log.slo_applied,
        log.slo_degraded,
        formatIsoTimestamp(log.created_at_ms),
      ]),
    );
  }

  return (
    <div className="adminx-page-grid">
      <section className="adminx-stat-grid">
        <StatCard
          label="Filtered requests"
          value={String(filteredUsageRecords.length)}
          detail="Usage records matching the current query scope."
        />
        <StatCard
          label="Filtered units"
          value={String(filteredUnits)}
          detail="Metered units across the filtered usage result set."
        />
        <StatCard
          label="Filtered tokens"
          value={String(filteredTokens)}
          detail="Prompt and completion tokens from the filtered request set."
        />
        <StatCard
          label="Filtered amount"
          value={filteredAmount.toFixed(2)}
          detail="Booked amount across matching usage records."
        />
        <StatCard
          label="Decision logs"
          value={String(filteredRoutingLogs.length)}
          detail="Routing logs matching the current provider/search/time scope."
        />
      </section>

      <PageToolbar
        title="Traffic query workbench"
        detail="Inspect usage and routing evidence, keep filters on the canvas, and export the exact result set you are reviewing."
        actions={(
          <>
            <InlineButton tone="primary" onClick={exportUsageCsv}>
              Export usage CSV
            </InlineButton>
            <InlineButton onClick={exportRoutingCsv}>Export routing CSV</InlineButton>
          </>
        )}
      />

      <Surface
        title="Request query console"
        detail="Search and narrow usage or routing data by traffic ownership, provider, model, portal-user scope, and recent time window."
      >
        <div className="adminx-form-grid">
          <label className="adminx-field">
            <span>Search logs and usage</span>
            <input
              value={search}
              onChange={(event) => setSearch(event.target.value)}
              placeholder="project, model, provider, route, reason..."
            />
          </label>
          <label className="adminx-field">
            <span>View mode</span>
            <select
              value={mode}
              onChange={(event) => setMode(event.target.value as ViewMode)}
            >
              <option value="all">Usage and routing</option>
              <option value="usage">Usage only</option>
              <option value="routing">Routing only</option>
            </select>
          </label>
          <label className="adminx-field">
            <span>Recent window</span>
            <select
              value={recentWindow}
              onChange={(event) => setRecentWindow(event.target.value as RecentWindow)}
            >
              <option value="all">All time</option>
              <option value="24h">Last 24 hours</option>
              <option value="7d">Last 7 days</option>
              <option value="30d">Last 30 days</option>
            </select>
          </label>
          <label className="adminx-field">
            <span>Project scope</span>
            <select
              value={projectFilter}
              onChange={(event) => setProjectFilter(event.target.value)}
            >
              <option value="all">All projects</option>
              {projectOptions.map((projectId) => (
                <option key={projectId} value={projectId}>
                  {projectId}
                </option>
              ))}
            </select>
          </label>
          <label className="adminx-field">
            <span>Provider scope</span>
            <select
              value={providerFilter}
              onChange={(event) => setProviderFilter(event.target.value)}
            >
              <option value="all">All providers</option>
              {providerOptions.map((providerId) => (
                <option key={providerId} value={providerId}>
                  {providerId}
                </option>
              ))}
            </select>
          </label>
          <label className="adminx-field">
            <span>Model scope</span>
            <select
              value={modelFilter}
              onChange={(event) => setModelFilter(event.target.value)}
            >
              <option value="all">All models</option>
              {modelOptions.map((model) => (
                <option key={model} value={model}>
                  {model}
                </option>
              ))}
            </select>
          </label>
          <label className="adminx-field">
            <span>Portal user scope</span>
            <select
              value={portalUserScope}
              onChange={(event) => setPortalUserScope(event.target.value)}
            >
              <option value="all">All portal users</option>
              {snapshot.portalUsers.map((user) => (
                <option key={user.id} value={user.id}>
                  {user.display_name} ({user.email})
                </option>
              ))}
            </select>
          </label>
          <div className="adminx-form-actions">
            <InlineButton tone="primary" onClick={clearFilters}>
              Clear filters
            </InlineButton>
          </div>
          <div className="adminx-note">
            <strong>Query semantics</strong>
            <p>Project, model, and portal-user scope apply to usage records and the derived leaderboard. Routing logs remain scoped by search text, provider, and recent window because the control plane does not yet attach per-project context to routing decisions.</p>
          </div>
        </div>
      </Surface>

      {mode !== 'routing' ? (
        <div className="adminx-users-grid">
          <Surface
            title="User traffic leaderboard"
            detail="Portal users ranked by the current filtered request set so operators can isolate heavy traffic owners quickly."
          >
            <DataTable
              columns={[
                {
                  key: 'user',
                  label: 'Portal user',
                  render: (user) => (
                    <div className="adminx-table-cell-stack">
                      <strong>{user.display_name}</strong>
                      <span>{user.email}</span>
                    </div>
                  ),
                },
                { key: 'workspace', label: 'Project', render: (user) => user.workspace_project_id ?? '-' },
                { key: 'requests', label: 'Requests', render: (user) => user.filtered_request_count },
                { key: 'tokens', label: 'Tokens', render: (user) => user.filtered_total_tokens },
                { key: 'units', label: 'Units', render: (user) => user.filtered_usage_units },
                { key: 'amount', label: 'Amount', render: (user) => user.filtered_amount.toFixed(4) },
                {
                  key: 'status',
                  label: 'Status',
                  render: (user) => (
                    <Pill tone={user.active ? 'live' : 'danger'}>
                      {user.active ? 'active' : 'disabled'}
                    </Pill>
                  ),
                },
              ]}
              rows={filteredPortalUsers}
              empty="No portal users match the current filter."
              getKey={(user) => user.id}
            />
          </Surface>

          <Surface
            title="Project hotspots"
            detail="Projects ranked from the filtered usage set by matching request volume, token usage, and booked amount."
          >
            <DataTable
              columns={[
                { key: 'project', label: 'Project', render: (project) => <strong>{project.project_id}</strong> },
                { key: 'requests', label: 'Requests', render: (project) => project.request_count },
                { key: 'tokens', label: 'Tokens', render: (project) => project.total_tokens },
                { key: 'units', label: 'Units', render: (project) => project.total_units },
                { key: 'amount', label: 'Amount', render: (project) => project.total_amount.toFixed(4) },
              ]}
              rows={projectHotspots}
              empty="No hotspot projects match the current filter."
              getKey={(project) => project.project_id}
            />
          </Surface>
        </div>
      ) : null}

      {mode !== 'routing' ? (
        <Surface title="Usage records" detail="Raw request records grouped by project, model, provider, tokens, metered units, and amount.">
          <DataTable
            columns={[
              { key: 'project', label: 'Project', render: (record) => record.project_id },
              { key: 'model', label: 'Model', render: (record) => <strong>{record.model}</strong> },
              { key: 'provider', label: 'Provider', render: (record) => record.provider },
              { key: 'input_tokens', label: 'Input tokens', render: (record) => record.input_tokens },
              { key: 'output_tokens', label: 'Output tokens', render: (record) => record.output_tokens },
              { key: 'total_tokens', label: 'Total tokens', render: (record) => record.total_tokens },
              { key: 'units', label: 'Units', render: (record) => record.units },
              { key: 'amount', label: 'Amount', render: (record) => record.amount.toFixed(4) },
              { key: 'time', label: 'Created', render: (record) => formatTimestamp(record.created_at_ms) },
            ]}
            rows={filteredUsageRecords}
            empty="No usage records match the current filter."
            getKey={(record, index) => `${record.project_id}:${record.model}:${record.provider}:${record.created_at_ms}:${index}`}
          />
        </Surface>
      ) : null}

      <Surface title="Billing summary by project" detail="Quota and cost posture rolled up by project.">
        <DataTable
          columns={[
            { key: 'project', label: 'Project', render: (project) => <strong>{project.project_id}</strong> },
            { key: 'entries', label: 'Entries', render: (project) => project.entry_count },
            { key: 'units', label: 'Units', render: (project) => project.used_units },
            { key: 'amount', label: 'Amount', render: (project) => project.booked_amount.toFixed(2) },
            { key: 'remaining', label: 'Remaining', render: (project) => project.remaining_units ?? '-' },
            {
              key: 'status',
              label: 'Quota',
              render: (project) => (
                <Pill tone={project.exhausted ? 'danger' : 'live'}>
                  {project.exhausted ? 'exhausted' : 'healthy'}
                </Pill>
              ),
            },
          ]}
          rows={snapshot.billingSummary.projects}
          empty="No billing records available."
          getKey={(project) => project.project_id}
        />
      </Surface>

      {mode !== 'usage' ? (
        <Surface title="Routing decision logs" detail="Recent routing selections, strategy, and selection reasons.">
          <DataTable
            columns={[
              { key: 'provider', label: 'Selected provider', render: (log) => <strong>{log.selected_provider_id}</strong> },
              { key: 'capability', label: 'Capability', render: (log) => log.capability },
              { key: 'route', label: 'Route key', render: (log) => log.route_key },
              { key: 'strategy', label: 'Strategy', render: (log) => log.strategy ?? '-' },
              { key: 'reason', label: 'Reason', render: (log) => log.selection_reason ?? '-' },
              { key: 'time', label: 'Created', render: (log) => formatTimestamp(log.created_at_ms) },
            ]}
            rows={filteredRoutingLogs}
            empty="No routing decision logs match the current filter."
            getKey={(log) => log.decision_id}
          />
        </Surface>
      ) : null}
    </div>
  );
}
