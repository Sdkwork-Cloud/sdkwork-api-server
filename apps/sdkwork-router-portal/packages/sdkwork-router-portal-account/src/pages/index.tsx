import { useDeferredValue, useEffect, useState } from 'react';
import {
  Button,
  DataTable,
  EmptyState,
  formatCurrency,
  formatUnits,
  InlineButton,
  Surface,
  ToolbarSearchField,
  usePortalI18n,
} from 'sdkwork-router-portal-commons';
import { portalErrorMessage } from 'sdkwork-router-portal-portal-api';
import type { LedgerEntry, ProjectBillingSummary } from 'sdkwork-router-portal-types';

import { getPortalBillingSummary, listPortalBillingLedger } from '../repository';
import type { PortalAccountPageProps } from '../types';

export function PortalAccountPage({ onNavigate }: PortalAccountPageProps) {
  const { t } = usePortalI18n();
  const [summary, setSummary] = useState<ProjectBillingSummary | null>(null);
  const [ledger, setLedger] = useState<LedgerEntry[]>([]);
  const [status, setStatus] = useState('Loading the financial account posture...');
  const [searchQuery, setSearchQuery] = useState('');
  const deferredSearch = useDeferredValue(searchQuery.trim().toLowerCase());

  useEffect(() => {
    let cancelled = false;

    void Promise.all([getPortalBillingSummary(), listPortalBillingLedger()])
      .then(([nextSummary, nextLedger]) => {
        if (cancelled) {
          return;
        }

        setSummary(nextSummary);
        setLedger(nextLedger);
        setStatus('Financial account posture is synced with the latest billing summary and ledger evidence.');
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

  const visibleLedger = ledger.filter((entry) =>
    !deferredSearch
    || entry.project_id.toLowerCase().includes(deferredSearch));

  if (!summary) {
    return (
      <Surface detail={status} title={t('Financial account')}>
        <EmptyState
          detail={t('Financial account posture will appear after the portal loads billing summary and ledger evidence.')}
          title={t('Preparing account')}
        />
      </Surface>
    );
  }

  return (
    <div className="grid gap-4">
      <section
        data-slot="portal-account-toolbar"
        className="rounded-[28px] border border-zinc-200/80 bg-white/92 p-4 shadow-[0_18px_48px_rgba(15,23,42,0.08)] backdrop-blur dark:border-zinc-800/80 dark:bg-zinc-950/70 sm:p-5"
      >
        <div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
          <div className="flex flex-wrap items-center gap-3">
            <Button type="button" onClick={() => onNavigate('credits')}>
              {t('Open credits')}
            </Button>
            <InlineButton onClick={() => onNavigate('billing')} tone="secondary">
              {t('Review billing')}
            </InlineButton>
            <InlineButton onClick={() => onNavigate('usage')} tone="secondary">
              {t('Open usage')}
            </InlineButton>
          </div>

          <ToolbarSearchField
            label={t('Search ledger')}
            value={searchQuery}
            onChange={(event) => setSearchQuery(event.target.value)}
            placeholder={t('Search ledger')}
            className="w-full lg:max-w-[24rem]"
          />
        </div>
      </section>

      {visibleLedger.length ? (
        <DataTable
          columns={[
            { key: 'project', label: 'Project', render: (row) => row.project_id },
            { key: 'units', label: 'Units', render: (row) => formatUnits(row.units) },
            { key: 'amount', label: 'Amount', render: (row) => formatCurrency(row.amount) },
          ]}
          empty={t('No ledger entries recorded yet.')}
          getKey={(row, index) => `${row.project_id}-${row.units}-${index}`}
          rows={visibleLedger}
        />
      ) : (
        <EmptyState
          detail={
            ledger.length
              ? 'Adjust the search or wait for quota and billing activity to populate the ledger.'
              : status
          }
            title={t('No ledger entries for this slice')}
        />
      )}
    </div>
  );
}
