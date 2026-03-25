import { useDeferredValue, useEffect, useMemo, useState } from 'react';
import { RefreshCw } from 'lucide-react';

import {
  DataTable,
  EmptyState,
  formatCurrency,
  formatDateTime,
  formatUnits,
  InlineButton,
  Select,
  ToolbarField,
  ToolbarDisclosure,
  ToolbarSearchField,
  usePortalI18n,
} from 'sdkwork-router-portal-commons';
import { portalErrorMessage } from 'sdkwork-router-portal-portal-api';
import type { UsageRecord, UsageSummary } from 'sdkwork-router-portal-types';

import { loadUsageWorkbenchData } from '../repository';
import { buildUsageWorkbenchViewModel } from '../services';
import type { PortalUsagePageProps, UsageFilters } from '../types';

const emptySummary: UsageSummary = {
  total_requests: 0,
  project_count: 0,
  model_count: 0,
  provider_count: 0,
  projects: [],
  providers: [],
  models: [],
};

export function PortalUsagePage({ onNavigate }: PortalUsagePageProps) {
  const { t } = usePortalI18n();
  const [summary, setSummary] = useState<UsageSummary>(emptySummary);
  const [records, setRecords] = useState<UsageRecord[]>([]);
  const [filters, setFilters] = useState<UsageFilters>({
    model: '',
    provider: '',
    date_range: '30d',
  });
  const [searchQuery, setSearchQuery] = useState('');
  const [status, setStatus] = useState('Loading request telemetry...');
  const deferredSearch = useDeferredValue(searchQuery.trim().toLowerCase());

  async function refreshUsageWorkbench(): Promise<void> {
    setStatus('Loading request telemetry...');

    try {
      const data = await loadUsageWorkbenchData();
      setSummary(data.summary);
      setRecords(data.records);
      setStatus('Per-call request telemetry is filtered to your workspace project.');
    } catch (error) {
      setStatus(portalErrorMessage(error));
    }
  }

  useEffect(() => {
    void refreshUsageWorkbench();
  }, []);

  const viewModel = useMemo(
    () => buildUsageWorkbenchViewModel(summary, records, filters),
    [filters, records, summary],
  );
  const visibleRecords = viewModel.filtered_records.filter((record) => {
    if (!deferredSearch) {
      return true;
    }

    const haystack = [record.project_id, record.model, record.provider]
      .join(' ')
      .toLowerCase();
    return haystack.includes(deferredSearch);
  });

  return (
    <div className="grid gap-4">
      <section
        data-slot="portal-usage-toolbar"
        className="rounded-[28px] border border-zinc-200/80 bg-white/92 p-4 shadow-[0_18px_48px_rgba(15,23,42,0.08)] backdrop-blur dark:border-zinc-800/80 dark:bg-zinc-950/70 sm:p-5"
      >
        <div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
          <div className="flex flex-wrap items-center gap-3">
            <InlineButton onClick={() => void refreshUsageWorkbench()} tone="secondary">
              <RefreshCw className="h-4 w-4" />
              {t('Refresh')}
            </InlineButton>
            <InlineButton onClick={() => onNavigate('billing')} tone="primary">
              {t('Review billing')}
            </InlineButton>
            <InlineButton onClick={() => onNavigate('api-keys')} tone="secondary">
              {t('Manage keys')}
            </InlineButton>
          </div>

          <div className="flex min-w-0 flex-1 flex-col gap-3 lg:max-w-[28rem] lg:items-end">
            <ToolbarSearchField
              label={t('Search usage')}
              value={searchQuery}
              onChange={(event) => setSearchQuery(event.target.value)}
              placeholder={t('Search usage')}
              className="w-full lg:max-w-[24rem]"
            />
            <ToolbarDisclosure>
              <ToolbarField label={t('Time range')} className="w-full sm:max-w-[18rem]">
                <Select
                  value={filters.date_range}
                  onChange={(event) =>
                    setFilters((current) => ({
                      ...current,
                      date_range: event.target.value as UsageFilters['date_range'],
                    }))
                  }
                >
                  <option value="24h">{t('Last 24 hours')}</option>
                  <option value="7d">{t('Last 7 days')}</option>
                  <option value="30d">{t('Last 30 days')}</option>
                  <option value="all">{t('All time')}</option>
                </Select>
              </ToolbarField>
            </ToolbarDisclosure>
          </div>
        </div>
      </section>

      {visibleRecords.length ? (
        <DataTable
          columns={[
            { key: 'project', label: 'Project', render: (row) => row.project_id },
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
          rows={visibleRecords}
        />
      ) : (
        <EmptyState
          detail={
            records.length
              ? 'Adjust the filters or run a gateway call from your project and the request list will populate here.'
              : status
          }
          title="No request history for this slice"
        />
      )}
    </div>
  );
}
