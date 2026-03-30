import { useDeferredValue, useState } from 'react';
import type { ReactNode } from 'react';

import {
  DataTable,
  formatAdminCurrency,
  formatAdminDateTime,
  formatAdminNumber,
  InlineButton,
  PageToolbar,
  Pill,
  Select,
  ToolbarDisclosure,
  ToolbarField,
  ToolbarInline,
  ToolbarSearchField,
  useAdminI18n,
} from 'sdkwork-router-admin-commons';
import type {
  AdminPageProps,
  ManagedUser,
  RoutingDecisionLogRecord,
  UsageRecord,
} from 'sdkwork-router-admin-types';

type ViewMode = 'usage' | 'routing' | 'billing' | 'users' | 'projects';
type RecentWindow = 'all' | '24h' | '7d' | '30d';

type FilteredPortalTrafficRow = ManagedUser & {
  filtered_request_count: number;
  filtered_usage_units: number;
  filtered_total_tokens: number;
  filtered_amount: number;
};

type BillingRow = AdminPageProps['snapshot']['billingSummary']['projects'][number] & {
  kind: 'billing';
};

type UsageRow = UsageRecord & {
  kind: 'usage';
};

type RoutingRow = RoutingDecisionLogRecord & {
  kind: 'routing';
};

type UserTrafficRow = FilteredPortalTrafficRow & {
  kind: 'users';
};

type ProjectHotspotRow = {
  kind: 'projects';
  project_id: string;
  request_count: number;
  total_tokens: number;
  total_units: number;
  total_amount: number;
};

type TrafficTableRow = UsageRow | RoutingRow | BillingRow | UserTrafficRow | ProjectHotspotRow;

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

function rowKey(row: TrafficTableRow, index: number): string {
  switch (row.kind) {
    case 'usage':
      return `${row.project_id}:${row.model}:${row.provider}:${row.created_at_ms}:${index}`;
    case 'routing':
      return row.decision_id;
    case 'billing':
      return row.project_id;
    case 'users':
      return row.id;
    case 'projects':
      return row.project_id;
    default:
      return String(index);
  }
}

export function TrafficPage({ snapshot }: AdminPageProps) {
  const { t } = useAdminI18n();
  const [search, setSearch] = useState('');
  const [viewMode, setViewMode] = useState<ViewMode>('usage');
  const [recentWindow, setRecentWindow] = useState<RecentWindow>('all');
  const deferredQuery = useDeferredValue(search.trim().toLowerCase());

  const recentCutoff = recentWindowCutoff(recentWindow);

  const filteredUsageRecords = snapshot.usageRecords.filter((record) => {
    if (recentCutoff && record.created_at_ms < recentCutoff) {
      return false;
    }

    const haystack = [record.project_id, record.model, record.provider].join(' ').toLowerCase();
    return haystack.includes(deferredQuery);
  });

  const filteredRoutingLogs = snapshot.routingLogs.filter((log) => {
    if (recentCutoff && log.created_at_ms < recentCutoff) {
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

  const userRows: UserTrafficRow[] = snapshot.portalUsers
    .map((user) => {
      const projectUsage = usageByProject.get(user.workspace_project_id ?? '');
      return {
        ...user,
        kind: 'users' as const,
        filtered_request_count: projectUsage?.request_count ?? 0,
        filtered_total_tokens: projectUsage?.total_tokens ?? 0,
        filtered_usage_units: projectUsage?.total_units ?? 0,
        filtered_amount: projectUsage?.total_amount ?? 0,
      };
    })
    .filter((user) => !deferredQuery || userMatchesQuery(user, deferredQuery) || user.filtered_request_count > 0)
    .sort((left, right) => (
      right.filtered_request_count - left.filtered_request_count
      || right.filtered_total_tokens - left.filtered_total_tokens
      || right.filtered_usage_units - left.filtered_usage_units
      || right.filtered_amount - left.filtered_amount
    ));

  const projectRows: ProjectHotspotRow[] = Array.from(usageByProject.entries())
    .map(([project_id, entry]) => ({
      kind: 'projects' as const,
      project_id,
      ...entry,
    }))
    .sort((left, right) => (
      right.request_count - left.request_count
      || right.total_tokens - left.total_tokens
      || right.total_amount - left.total_amount
    ));

  const billingRows: BillingRow[] = snapshot.billingSummary.projects
    .filter((project) => {
      if (!deferredQuery) {
        return true;
      }

      return [
        project.project_id,
        project.quota_policy_id ?? '',
        project.remaining_units ?? '',
        project.exhausted ? 'exhausted' : 'healthy',
      ]
        .join(' ')
        .toLowerCase()
        .includes(deferredQuery);
    })
    .map((project) => ({
      ...project,
      kind: 'billing' as const,
    }));

  const usageRows: UsageRow[] = filteredUsageRecords.map((record) => ({
    ...record,
    kind: 'usage' as const,
  }));

  const routingRows: RoutingRow[] = filteredRoutingLogs.map((log) => ({
    ...log,
    kind: 'routing' as const,
  }));

  function clearFilters() {
    setSearch('');
    setViewMode('usage');
    setRecentWindow('all');
  }

  function exportCurrentCsv() {
    switch (viewMode) {
      case 'routing':
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
          routingRows.map((log) => [
            log.decision_id,
            log.selected_provider_id,
            log.capability,
            log.route_key,
            log.strategy ?? '',
            log.selection_reason ?? '',
            log.requested_region ?? '',
            log.slo_applied,
            log.slo_degraded,
            new Date(log.created_at_ms).toISOString(),
          ]),
        );
        return;
      case 'billing':
        downloadCsv(
          'sdkwork-router-billing-summary.csv',
          ['project_id', 'entry_count', 'used_units', 'booked_amount', 'remaining_units', 'status'],
          billingRows.map((project) => [
            project.project_id,
            project.entry_count,
            project.used_units,
            project.booked_amount.toFixed(2),
            project.remaining_units ?? '',
            project.exhausted ? 'exhausted' : 'healthy',
          ]),
        );
        return;
      case 'users':
        downloadCsv(
          'sdkwork-router-user-traffic.csv',
          ['user_id', 'email', 'project_id', 'request_count', 'total_tokens', 'usage_units', 'amount', 'status'],
          userRows.map((user) => [
            user.id,
            user.email,
            user.workspace_project_id ?? '',
            user.filtered_request_count,
            user.filtered_total_tokens,
            user.filtered_usage_units,
            user.filtered_amount.toFixed(4),
            user.active ? 'active' : 'disabled',
          ]),
        );
        return;
      case 'projects':
        downloadCsv(
          'sdkwork-router-project-hotspots.csv',
          ['project_id', 'request_count', 'total_tokens', 'total_units', 'total_amount'],
          projectRows.map((project) => [
            project.project_id,
            project.request_count,
            project.total_tokens,
            project.total_units,
            project.total_amount.toFixed(4),
          ]),
        );
        return;
      case 'usage':
      default:
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
          usageRows.map((record) => [
            record.project_id,
            record.model,
            record.provider,
            record.input_tokens,
            record.output_tokens,
            record.total_tokens,
            record.units,
            record.amount.toFixed(4),
            new Date(record.created_at_ms).toISOString(),
          ]),
        );
    }
  }

  let tableRows: TrafficTableRow[] = usageRows;
  let tableColumns: Array<{ key: string; label: string; render: (row: TrafficTableRow) => ReactNode }> = [
    { key: 'project', label: t('Project'), render: (row) => row.kind === 'usage' ? row.project_id : '-' },
    { key: 'model', label: t('Model'), render: (row) => row.kind === 'usage' ? <strong>{row.model}</strong> : '-' },
    { key: 'provider', label: t('Provider'), render: (row) => row.kind === 'usage' ? row.provider : '-' },
    { key: 'input_tokens', label: t('Input tokens'), render: (row) => row.kind === 'usage' ? formatAdminNumber(row.input_tokens) : '-' },
    { key: 'output_tokens', label: t('Output tokens'), render: (row) => row.kind === 'usage' ? formatAdminNumber(row.output_tokens) : '-' },
    { key: 'total_tokens', label: t('Total tokens'), render: (row) => row.kind === 'usage' ? formatAdminNumber(row.total_tokens) : '-' },
    { key: 'units', label: t('Units'), render: (row) => row.kind === 'usage' ? formatAdminNumber(row.units) : '-' },
    { key: 'amount', label: t('Amount'), render: (row) => row.kind === 'usage' ? formatAdminCurrency(row.amount, 4) : '-' },
    { key: 'time', label: t('Created'), render: (row) => row.kind === 'usage' ? formatAdminDateTime(row.created_at_ms) : '-' },
  ];
  let emptyLabel = t('No usage records match the current filter.');

  switch (viewMode) {
    case 'routing':
      tableRows = routingRows;
      tableColumns = [
        { key: 'provider', label: t('Selected provider'), render: (row) => row.kind === 'routing' ? <strong>{row.selected_provider_id}</strong> : '-' },
        { key: 'capability', label: t('Capability'), render: (row) => row.kind === 'routing' ? row.capability : '-' },
        { key: 'route', label: t('Route key'), render: (row) => row.kind === 'routing' ? row.route_key : '-' },
        { key: 'strategy', label: t('Strategy'), render: (row) => row.kind === 'routing' ? row.strategy ?? '-' : '-' },
        { key: 'reason', label: t('Reason'), render: (row) => row.kind === 'routing' ? row.selection_reason ?? '-' : '-' },
        { key: 'time', label: t('Created'), render: (row) => row.kind === 'routing' ? formatAdminDateTime(row.created_at_ms) : '-' },
      ];
      emptyLabel = t('No routing decision logs match the current filter.');
      break;
    case 'billing':
      tableRows = billingRows;
      tableColumns = [
        { key: 'project', label: t('Project'), render: (row) => row.kind === 'billing' ? <strong>{row.project_id}</strong> : '-' },
        { key: 'entries', label: t('Entries'), render: (row) => row.kind === 'billing' ? formatAdminNumber(row.entry_count) : '-' },
        { key: 'units', label: t('Units'), render: (row) => row.kind === 'billing' ? formatAdminNumber(row.used_units) : '-' },
        { key: 'amount', label: t('Amount'), render: (row) => row.kind === 'billing' ? formatAdminCurrency(row.booked_amount, 2) : '-' },
        { key: 'remaining', label: t('Remaining'), render: (row) => row.kind === 'billing' ? row.remaining_units ?? '-' : '-' },
        {
          key: 'status',
          label: t('Quota'),
          render: (row) => row.kind === 'billing' ? (
            <Pill tone={row.exhausted ? 'danger' : 'live'}>
              {row.exhausted ? t('exhausted') : t('healthy')}
            </Pill>
          ) : '-',
        },
      ];
      emptyLabel = t('No billing records available.');
      break;
    case 'users':
      tableRows = userRows;
      tableColumns = [
        {
          key: 'user',
          label: t('Portal user'),
          render: (row) => row.kind === 'users' ? (
            <div className="adminx-table-cell-stack">
              <strong>{row.display_name}</strong>
              <span>{row.email}</span>
            </div>
          ) : '-',
        },
        { key: 'workspace', label: t('Project'), render: (row) => row.kind === 'users' ? row.workspace_project_id ?? '-' : '-' },
        { key: 'requests', label: t('Requests'), render: (row) => row.kind === 'users' ? formatAdminNumber(row.filtered_request_count) : '-' },
        { key: 'tokens', label: t('Tokens'), render: (row) => row.kind === 'users' ? formatAdminNumber(row.filtered_total_tokens) : '-' },
        { key: 'units', label: t('Units'), render: (row) => row.kind === 'users' ? formatAdminNumber(row.filtered_usage_units) : '-' },
        { key: 'amount', label: t('Amount'), render: (row) => row.kind === 'users' ? formatAdminCurrency(row.filtered_amount, 4) : '-' },
        {
          key: 'status',
          label: t('Status'),
          render: (row) => row.kind === 'users' ? (
            <Pill tone={row.active ? 'live' : 'danger'}>
              {row.active ? t('active') : t('disabled')}
            </Pill>
          ) : '-',
        },
      ];
      emptyLabel = t('No portal users match the current filter.');
      break;
    case 'projects':
      tableRows = projectRows;
      tableColumns = [
        { key: 'project', label: t('Project'), render: (row) => row.kind === 'projects' ? <strong>{row.project_id}</strong> : '-' },
        { key: 'requests', label: t('Requests'), render: (row) => row.kind === 'projects' ? formatAdminNumber(row.request_count) : '-' },
        { key: 'tokens', label: t('Tokens'), render: (row) => row.kind === 'projects' ? formatAdminNumber(row.total_tokens) : '-' },
        { key: 'units', label: t('Units'), render: (row) => row.kind === 'projects' ? formatAdminNumber(row.total_units) : '-' },
        { key: 'amount', label: t('Amount'), render: (row) => row.kind === 'projects' ? formatAdminCurrency(row.total_amount, 4) : '-' },
      ];
      emptyLabel = t('No hotspot projects match the current filter.');
      break;
    case 'usage':
    default:
      break;
  }

  return (
    <div className="adminx-page-grid">
      <PageToolbar
        compact
        actions={(
          <>
            <InlineButton tone="primary" onClick={exportCurrentCsv}>
              {t('Export CSV')}
            </InlineButton>
            <InlineButton onClick={clearFilters}>{t('Clear filters')}</InlineButton>
          </>
        )}
      >
        <ToolbarInline>
          <ToolbarSearchField
            label={t('Search logs and usage')}
            value={search}
            onChange={(event) => setSearch(event.target.value)}
            placeholder={t('project, model, provider, route, reason...')}
          />
          <ToolbarDisclosure>
            <ToolbarInline className="adminx-toolbar-inline-disclosure">
              <ToolbarField label={t('View mode')}>
                <Select
                  value={viewMode}
                  onChange={(event) => setViewMode(event.target.value as ViewMode)}
                >
                  <option value="usage">{t('Usage records')}</option>
                  <option value="routing">{t('Routing decision logs')}</option>
                  <option value="billing">{t('Billing summary by project')}</option>
                  <option value="users">{t('User traffic leaderboard')}</option>
                  <option value="projects">{t('Project hotspots')}</option>
                </Select>
              </ToolbarField>
              <ToolbarField label={t('Recent window')}>
                <Select
                  value={recentWindow}
                  onChange={(event) => setRecentWindow(event.target.value as RecentWindow)}
                >
                  <option value="all">{t('All time')}</option>
                  <option value="24h">{t('Last 24 hours')}</option>
                  <option value="7d">{t('Last 7 days')}</option>
                  <option value="30d">{t('Last 30 days')}</option>
                </Select>
              </ToolbarField>
            </ToolbarInline>
          </ToolbarDisclosure>
        </ToolbarInline>
      </PageToolbar>

      <DataTable
        columns={tableColumns}
        rows={tableRows}
        empty={emptyLabel}
        getKey={rowKey}
      />
    </div>
  );
}
