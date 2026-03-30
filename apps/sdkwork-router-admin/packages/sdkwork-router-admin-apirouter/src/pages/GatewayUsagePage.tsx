import { startTransition, useDeferredValue, useMemo, useState } from 'react';

import {
  DataTable,
  formatAdminCurrency,
  formatAdminDateTime,
  InlineButton,
  PageToolbar,
  Select,
  ToolbarDisclosure,
  ToolbarField,
  ToolbarInline,
  ToolbarSearchField,
  useAdminI18n,
} from 'sdkwork-router-admin-commons';
import type {
  AdminPageProps,
  GatewayApiKeyRecord,
  UsageRecord,
} from 'sdkwork-router-admin-types';

type GatewayUsagePageProps = AdminPageProps & {
  onRefreshWorkspace: () => Promise<void>;
};

type TimeRangePreset = 'all' | '24h' | '7d' | '30d';

const PAGE_SIZE = 20;

function csvValue(value: string | number | boolean | null | undefined): string {
  const normalized = value == null ? '' : String(value);
  return `"${normalized.replaceAll('"', '""')}"`;
}

function downloadCsv(
  filename: string,
  headers: string[],
  rows: Array<Array<string | number | boolean | null | undefined>>,
): void {
  const contents = [
    headers.map(csvValue).join(','),
    ...rows.map((row) => row.map(csvValue).join(',')),
  ].join('\n');
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

function recentWindowCutoff(window: TimeRangePreset): number | null {
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

function buildUsageRecordKey(record: UsageRecord, index: number): string {
  return `${record.project_id}:${record.model}:${record.provider}:${record.created_at_ms}:${index}`;
}

function compareUsageRecords(left: UsageRecord, right: UsageRecord): number {
  return (
    right.created_at_ms - left.created_at_ms
    || left.project_id.localeCompare(right.project_id)
    || left.provider.localeCompare(right.provider)
    || left.model.localeCompare(right.model)
  );
}

export function GatewayUsagePage({
  snapshot,
  onRefreshWorkspace,
}: GatewayUsagePageProps) {
  const { t, formatNumber } = useAdminI18n();
  const [search, setSearch] = useState('');
  const [selectedKey, setSelectedKey] = useState('all');
  const [timeRange, setTimeRange] = useState<TimeRangePreset>('30d');
  const [page, setPage] = useState(1);
  const deferredSearch = useDeferredValue(search.trim().toLowerCase());

  const keyByHashed = useMemo(
    () => new Map(snapshot.apiKeys.map((key) => [key.hashed_key, key])),
    [snapshot.apiKeys],
  );
  const selectedKeyRecord: GatewayApiKeyRecord | null =
    selectedKey === 'all' ? null : keyByHashed.get(selectedKey) ?? null;
  const presetCutoff = recentWindowCutoff(timeRange);

  const filteredRecords = snapshot.usageRecords.filter((record) => {
    if (selectedKeyRecord && record.project_id !== selectedKeyRecord.project_id) {
      return false;
    }
    if (presetCutoff && record.created_at_ms < presetCutoff) {
      return false;
    }

    if (!deferredSearch) {
      return true;
    }

    const haystack = [record.project_id, record.model, record.provider].join(' ').toLowerCase();
    return haystack.includes(deferredSearch);
  });

  const sortedRecords = [...filteredRecords].sort(compareUsageRecords);
  const totalPages = Math.max(1, Math.ceil(sortedRecords.length / PAGE_SIZE));
  const safePage = Math.min(page, totalPages);
  const pagedRecords = sortedRecords.slice((safePage - 1) * PAGE_SIZE, safePage * PAGE_SIZE);

  function exportCsv(): void {
    if (!sortedRecords.length) {
      return;
    }

    downloadCsv(
      'sdkwork-router-gateway-usage.csv',
      [
        'project_id',
        'selected_hashed_key',
        'provider',
        'model',
        'input_tokens',
        'output_tokens',
        'total_tokens',
        'units',
        'amount',
        'created_at',
      ],
      sortedRecords.map((record) => [
        record.project_id,
        selectedKeyRecord?.hashed_key ?? '',
        record.provider,
        record.model,
        record.input_tokens,
        record.output_tokens,
        record.total_tokens,
        record.units,
        record.amount.toFixed(4),
        new Date(record.created_at_ms).toISOString(),
      ]),
    );
  }

  function clearFilters(): void {
    startTransition(() => {
      setSearch('');
      setSelectedKey('all');
      setTimeRange('30d');
      setPage(1);
    });
  }

  return (
    <div className="adminx-page-grid">
      <PageToolbar
        compact
        actions={(
          <>
            <InlineButton onClick={() => void onRefreshWorkspace()}>
              {t('Refresh workspace')}
            </InlineButton>
            <InlineButton tone="primary" onClick={exportCsv} disabled={!sortedRecords.length}>
              {t('Export usage CSV')}
            </InlineButton>
            <InlineButton onClick={clearFilters}>{t('Clear filters')}</InlineButton>
          </>
        )}
      >
        <ToolbarInline>
          <ToolbarSearchField
            label={t('Search usage')}
            value={search}
            onChange={(event) => {
              setSearch(event.target.value);
              setPage(1);
            }}
            placeholder={t('project, model, provider...')}
          />
          <ToolbarDisclosure>
            <ToolbarInline className="adminx-toolbar-inline-disclosure">
              <ToolbarField label={t('Api key')}>
                <Select
                  value={selectedKey}
                  onChange={(event) => {
                    setSelectedKey(event.target.value);
                    setPage(1);
                  }}
                >
                  <option value="all">{t('All API keys')}</option>
                  {snapshot.apiKeys.map((key) => (
                    <option key={key.hashed_key} value={key.hashed_key}>
                      {key.label || key.project_id} ({key.environment})
                    </option>
                  ))}
                </Select>
              </ToolbarField>
              <ToolbarField label={t('Time range')}>
                <Select
                  value={timeRange}
                  onChange={(event) => {
                    setTimeRange(event.target.value as TimeRangePreset);
                    setPage(1);
                  }}
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
        columns={[
          { key: 'project', label: t('Project'), render: (record) => <strong>{record.project_id}</strong> },
          { key: 'provider', label: t('Provider'), render: (record) => record.provider },
          { key: 'model', label: t('Model'), render: (record) => <strong>{record.model}</strong> },
          { key: 'input', label: t('Input tokens'), render: (record) => formatNumber(record.input_tokens) },
          { key: 'output', label: t('Output tokens'), render: (record) => formatNumber(record.output_tokens) },
          { key: 'total', label: t('Total tokens'), render: (record) => formatNumber(record.total_tokens) },
          { key: 'units', label: t('Units'), render: (record) => formatNumber(record.units) },
          { key: 'amount', label: t('Amount'), render: (record) => formatAdminCurrency(record.amount, 4) },
          { key: 'time', label: t('Created'), render: (record) => formatAdminDateTime(record.created_at_ms) },
        ]}
        rows={pagedRecords}
        empty={t('No usage records match the current gateway filter.')}
        getKey={(record, index) => buildUsageRecordKey(record, index)}
      />

      <div className="adminx-row">
        <span>
          {t('Page {page} of {total} · {count} records', {
            page: safePage,
            total: totalPages,
            count: formatNumber(sortedRecords.length),
          })}
        </span>
        <InlineButton
          onClick={() =>
            startTransition(() => {
              setPage((current) => Math.max(1, current - 1));
            })
          }
          disabled={safePage <= 1}
        >
          {t('Previous page')}
        </InlineButton>
        <InlineButton
          tone="primary"
          onClick={() =>
            startTransition(() => {
              setPage((current) => Math.min(totalPages, current + 1));
            })
          }
          disabled={safePage >= totalPages}
        >
          {t('Next page')}
        </InlineButton>
      </div>
    </div>
  );
}
