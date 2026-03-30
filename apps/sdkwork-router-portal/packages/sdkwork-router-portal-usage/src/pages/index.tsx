import { RefreshCw } from 'lucide-react';
import { startTransition, useEffect, useMemo, useState } from 'react';

import {
  DataTable,
  formatCurrency,
  formatDateTime,
  formatUnits,
  InlineButton,
  MetricCard,
  Select,
  Surface,
  ToolbarField,
  ToolbarInline,
  usePortalI18n,
} from 'sdkwork-router-portal-commons';
import { portalErrorMessage } from 'sdkwork-router-portal-portal-api';
import type { GatewayApiKeyRecord, UsageRecord } from 'sdkwork-router-portal-types';

import { loadUsageWorkbenchData } from '../repository';
import { buildPortalUsageViewModel } from '../services';
import type { PortalUsagePageProps, UsageFilters } from '../types';

const PAGE_SIZE = 12;

function formatLatency(latencyMs: number | null): string {
  if (latencyMs === null || latencyMs === undefined) {
    return 'Pending';
  }

  if (latencyMs >= 1000) {
    return `${(latencyMs / 1000).toFixed(latencyMs >= 10_000 ? 0 : 1)}s`;
  }

  return `${formatUnits(latencyMs)} ms`;
}

function buildUsageRecordKey(record: UsageRecord, index: number): string {
  return [
    record.project_id,
    record.api_key_hash ?? 'workspace',
    record.model,
    record.created_at_ms,
    index,
  ].join(':');
}

export function PortalUsagePage({ onNavigate }: PortalUsagePageProps) {
  const { t } = usePortalI18n();
  const [apiKeys, setApiKeys] = useState<GatewayApiKeyRecord[]>([]);
  const [records, setRecords] = useState<UsageRecord[]>([]);
  const [filters, setFilters] = useState<UsageFilters>({
    api_key_hash: 'all',
    channel_id: 'all',
    model: 'all',
    time_range: '30d',
  });
  const [page, setPage] = useState(1);
  const [status, setStatus] = useState('Loading request telemetry...');

  async function refreshUsageWorkbench(): Promise<void> {
    setStatus('Loading request telemetry...');

    try {
      const data = await loadUsageWorkbenchData();
      setApiKeys(data.apiKeys);
      setRecords(data.records);
      setStatus('Usage cards and ledger-grade request facts reflect the current filtered workspace slice.');
    } catch (error) {
      setStatus(portalErrorMessage(error));
    }
  }

  useEffect(() => {
    void refreshUsageWorkbench();
  }, []);

  const viewModel = useMemo(
    () =>
      buildPortalUsageViewModel({
        apiKeys,
        records,
        filters,
        page,
        page_size: PAGE_SIZE,
      }),
    [apiKeys, filters, page, records],
  );

  function updateFilters(nextFilters: Partial<UsageFilters>): void {
    startTransition(() => {
      setFilters((current) => ({
        ...current,
        ...nextFilters,
      }));
      setPage(1);
    });
  }

  function clearFilters(): void {
    startTransition(() => {
      setFilters({
        api_key_hash: 'all',
        channel_id: 'all',
        model: 'all',
        time_range: '30d',
      });
      setPage(1);
    });
  }

  return (
    <div className="grid gap-4">
      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        <MetricCard
          detail="Total request count after API key, time range, channel, and model filters are applied."
          label={t('Total requests')}
          value={formatUnits(viewModel.summary.total_requests)}
        />
        <MetricCard
          detail={`Input ${formatUnits(viewModel.summary.input_tokens)} / Output ${formatUnits(viewModel.summary.output_tokens)}`}
          label={t('Total tokens')}
          value={formatUnits(viewModel.summary.total_tokens)}
        />
        <MetricCard
          detail={`${formatCurrency(viewModel.summary.actual_amount)} / ${formatCurrency(viewModel.summary.reference_amount)} actual deduction / reference original price.`}
          label={t('Total spend')}
          value={`${formatCurrency(viewModel.summary.actual_amount)} / ${formatCurrency(viewModel.summary.reference_amount)}`}
        />
        <MetricCard
          detail="Average latency across usage rows that already contain request latency facts."
          label={t('Average latency')}
          value={formatLatency(viewModel.summary.average_latency_ms)}
        />
      </div>

      <Surface
        actions={(
          <>
            <InlineButton onClick={() => void refreshUsageWorkbench()} tone="secondary">
              <RefreshCw className="h-4 w-4" />
              {t('Refresh')}
            </InlineButton>
            <InlineButton onClick={() => onNavigate('billing')} tone="secondary">
              {t('Review billing')}
            </InlineButton>
            <InlineButton onClick={() => onNavigate('api-keys')} tone="secondary">
              {t('Manage keys')}
            </InlineButton>
          </>
        )}
        detail={status}
        title="Usage records"
      >
        <div className="grid gap-4">
          <ToolbarInline
            data-slot="portal-usage-filter-bar"
          >
            <ToolbarField label={t('API key')} className="min-w-[14rem] flex-[0_1_18rem]">
              <Select
                value={filters.api_key_hash}
                onChange={(event) => updateFilters({ api_key_hash: event.target.value })}
              >
                {viewModel.filter_options.api_keys.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </Select>
            </ToolbarField>

            <ToolbarField label={t('Time range')} className="min-w-[12rem] shrink-0">
              <Select
                value={filters.time_range}
                onChange={(event) =>
                  updateFilters({
                    time_range: event.target.value as UsageFilters['time_range'],
                  })
                }
              >
                <option value="24h">{t('Last 24 hours')}</option>
                <option value="7d">{t('Last 7 days')}</option>
                <option value="30d">{t('Last 30 days')}</option>
                <option value="all">{t('All time')}</option>
              </Select>
            </ToolbarField>

            <ToolbarField label={t('Channel')} className="min-w-[12rem] shrink-0">
              <Select
                value={filters.channel_id}
                onChange={(event) => updateFilters({ channel_id: event.target.value })}
              >
                <option value="all">All channels</option>
                {viewModel.filter_options.channels
                  .filter((option) => option !== 'all')
                  .map((option) => (
                    <option key={option} value={option}>
                      {option}
                    </option>
                ))}
              </Select>
            </ToolbarField>

            <ToolbarField label={t('Model')} className="min-w-[14rem] flex-[0_1_18rem]">
              <Select
                value={filters.model}
                onChange={(event) => updateFilters({ model: event.target.value })}
              >
                <option value="all">All models</option>
                {viewModel.filter_options.models
                  .filter((option) => option !== 'all')
                  .map((option) => (
                    <option key={option} value={option}>
                      {option}
                    </option>
                ))}
              </Select>
            </ToolbarField>

            <div className="ml-auto flex shrink-0 items-center gap-2.5 whitespace-nowrap">
              <InlineButton onClick={clearFilters} tone="secondary">
                Clear filters
              </InlineButton>
            </div>
          </ToolbarInline>

          <DataTable
            columns={[
              { key: 'api-key', label: 'API key', render: (row) => <strong>{row.api_key_label}</strong> },
              { key: 'channel', label: 'Channel', render: (row) => row.channel_label },
              { key: 'model', label: 'Model', render: (row) => <strong>{row.model}</strong> },
              { key: 'input', label: 'Input tokens', render: (row) => formatUnits(row.input_tokens) },
              { key: 'output', label: 'Output tokens', render: (row) => formatUnits(row.output_tokens) },
              { key: 'total', label: 'Total tokens', render: (row) => formatUnits(row.total_tokens) },
              { key: 'actual', label: 'Actual spend', render: (row) => formatCurrency(row.amount) },
              {
                key: 'reference',
                label: 'Reference price',
                render: (row) => formatCurrency(row.reference_amount),
              },
              { key: 'latency', label: 'Latency', render: (row) => formatLatency(row.latency_ms) },
              { key: 'time', label: 'Recorded', render: (row) => formatDateTime(row.created_at_ms) },
            ]}
            empty={(
              <div className="mx-auto flex max-w-[32rem] flex-col items-center gap-2 text-center">
                <strong className="text-base font-semibold text-zinc-950 dark:text-zinc-50">
                  No usage records for this slice
                </strong>
                <p className="text-sm text-zinc-500 dark:text-zinc-400">
                  {records.length
                    ? 'Adjust the API key, time range, channel, or model filter to reveal more request facts.'
                    : status}
                </p>
              </div>
            )}
            getKey={(row, index) => buildUsageRecordKey(row, index)}
            rows={viewModel.rows}
          />

          <div className="flex flex-col gap-3 border-t border-zinc-200/80 pt-4 text-sm text-zinc-500 dark:border-zinc-800/80 dark:text-zinc-400 sm:flex-row sm:items-center sm:justify-between">
            <span>
              {`Page ${viewModel.pagination.page} of ${viewModel.pagination.total_pages} / ${formatUnits(viewModel.pagination.total_items)} records`}
            </span>

            <div className="flex flex-wrap gap-2">
              <InlineButton
                disabled={viewModel.pagination.page <= 1}
                onClick={() =>
                  startTransition(() => {
                    setPage((current) => Math.max(1, current - 1));
                  })
                }
                tone="secondary"
              >
                {t('Previous page')}
              </InlineButton>
              <InlineButton
                disabled={viewModel.pagination.page >= viewModel.pagination.total_pages}
                onClick={() =>
                  startTransition(() => {
                    setPage((current) =>
                      Math.min(viewModel.pagination.total_pages, current + 1),
                    );
                  })
                }
                tone="primary"
              >
                {t('Next page')}
              </InlineButton>
            </div>
          </div>
        </div>
      </Surface>
    </div>
  );
}

