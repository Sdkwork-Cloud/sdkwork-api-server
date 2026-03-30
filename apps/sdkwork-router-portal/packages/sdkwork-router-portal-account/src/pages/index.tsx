import { useDeferredValue, useEffect, useState } from 'react';
import {
  Button,
  DataTable,
  EmptyState,
  formatCurrency,
  formatUnits,
  InlineButton,
  MetricCard,
  Pill,
  Surface,
  ToolbarInline,
  ToolbarSearchField,
  usePortalI18n,
} from 'sdkwork-router-portal-commons';
import { portalErrorMessage } from 'sdkwork-router-portal-portal-api';
import type {
  LedgerEntry,
  PortalCommerceMembership,
  ProjectBillingSummary,
} from 'sdkwork-router-portal-types';

import {
  getPortalBillingSummary,
  getPortalCommerceMembership,
  listPortalBillingLedger,
} from '../repository';
import type { PortalAccountPageProps } from '../types';

function formatPercentage(value: number): string {
  return `${Math.round(value * 100)}%`;
}

function clampPercentage(value: number): number {
  return Math.min(100, Math.max(0, value));
}

export function PortalAccountPage({ onNavigate }: PortalAccountPageProps) {
  const { t } = usePortalI18n();
  const [summary, setSummary] = useState<ProjectBillingSummary | null>(null);
  const [ledger, setLedger] = useState<LedgerEntry[]>([]);
  const [membership, setMembership] = useState<PortalCommerceMembership | null>(null);
  const [status, setStatus] = useState('Loading the financial account posture...');
  const [searchQuery, setSearchQuery] = useState('');
  const deferredSearch = useDeferredValue(searchQuery.trim().toLowerCase());

  useEffect(() => {
    let cancelled = false;

    void Promise.all([
      getPortalBillingSummary(),
      listPortalBillingLedger(),
      getPortalCommerceMembership(),
    ])
      .then(([nextSummary, nextLedger, nextMembership]) => {
        if (cancelled) {
          return;
        }

        setSummary(nextSummary);
        setLedger(nextLedger);
        setMembership(nextMembership);
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
  const remainingUnits = summary?.remaining_units ?? null;
  const quotaLimit = summary?.quota_limit_units ?? null;
  const usageRatio =
    quotaLimit && quotaLimit > 0
      ? clampPercentage((summary?.used_units ?? 0) / quotaLimit * 100)
      : null;
  const bookedAmount = summary?.booked_amount ?? 0;

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
        <ToolbarInline>
          <ToolbarSearchField
            label={t('Search ledger')}
            value={searchQuery}
            onChange={(event) => setSearchQuery(event.target.value)}
            placeholder={t('Search ledger')}
            className="min-w-[15rem] flex-[0_1_20rem]"
          />
          <div className="ml-auto flex shrink-0 items-center gap-2.5 whitespace-nowrap">
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
        </ToolbarInline>
      </section>

      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        <MetricCard
          label="Booked amount"
          value={formatCurrency(summary.booked_amount)}
          detail="Amount already booked into the workspace ledger for the current billing slice."
        />
        <MetricCard
          label="Used units"
          value={formatUnits(summary.used_units)}
          detail="Metered usage volume already consumed against the current quota posture."
        />
        <MetricCard
          label="Remaining units"
          value={remainingUnits === null ? 'Unlimited' : formatUnits(remainingUnits)}
          detail="Headroom still available before the current workspace hits its quota guardrail."
        />
        <MetricCard
          label="Ledger entries"
          value={formatUnits(summary.entry_count)}
          detail="Total billing ledger rows currently recorded for this workspace account."
        />
      </div>

      <Surface
        detail="Financial posture turns booked spend, consumed units, and quota pressure into one account-facing control surface."
        title="Financial posture"
      >
        <div className="grid gap-4 xl:grid-cols-[1.1fr_0.9fr]">
          <article className="rounded-[28px] border border-zinc-200 bg-zinc-50/90 p-5 dark:border-zinc-800 dark:bg-zinc-900/80">
            <div className="flex flex-wrap items-start justify-between gap-3">
              <div className="space-y-2">
                <p className="text-xs font-semibold uppercase tracking-[0.2em] text-zinc-500 dark:text-zinc-400">
                  Quota health
                </p>
                <div className="flex flex-wrap items-center gap-2">
                  <strong className="text-2xl text-zinc-950 dark:text-zinc-50">
                    {summary.exhausted ? 'Exhausted' : 'Healthy'}
                  </strong>
                  <Pill tone={summary.exhausted ? 'warning' : 'positive'}>
                    {summary.exhausted ? 'Action required' : 'Within guardrail'}
                  </Pill>
                </div>
              </div>
              <div className="rounded-2xl border border-zinc-200 bg-white/90 px-3 py-2 text-sm text-zinc-600 dark:border-zinc-800 dark:bg-zinc-950/80 dark:text-zinc-300">
                {quotaLimit === null
                  ? 'Unlimited quota'
                  : `${formatUnits(summary.used_units)} / ${formatUnits(quotaLimit)}`}
              </div>
            </div>

            <div className="mt-4 h-3 overflow-hidden rounded-full bg-zinc-200/80 dark:bg-zinc-800/80">
              <div
                className={`h-full rounded-full transition-all ${
                  summary.exhausted ? 'bg-rose-500' : 'bg-primary-500'
                }`}
                style={{ width: `${usageRatio ?? 32}%` }}
              />
            </div>

            <div className="mt-4 grid gap-3 md:grid-cols-3 text-sm text-zinc-600 dark:text-zinc-300">
              <div className="rounded-2xl border border-zinc-200 bg-white/90 p-4 dark:border-zinc-800 dark:bg-zinc-950/80">
                <span className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                  Utilization
                </span>
                <strong className="mt-2 block text-xl text-zinc-950 dark:text-zinc-50">
                  {usageRatio === null ? 'Tracked live' : formatPercentage(usageRatio / 100)}
                </strong>
              </div>
              <div className="rounded-2xl border border-zinc-200 bg-white/90 p-4 dark:border-zinc-800 dark:bg-zinc-950/80">
                <span className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                  Quota policy
                </span>
                <strong className="mt-2 block text-xl text-zinc-950 dark:text-zinc-50">
                  {summary.quota_policy_id ?? 'Workspace default'}
                </strong>
              </div>
              <div className="rounded-2xl border border-zinc-200 bg-white/90 p-4 dark:border-zinc-800 dark:bg-zinc-950/80">
                <span className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                  Current slice
                </span>
                <strong className="mt-2 block text-xl text-zinc-950 dark:text-zinc-50">
                  {visibleLedger.length} rows
                </strong>
              </div>
            </div>
          </article>

          <article className="rounded-[28px] border border-zinc-200 bg-zinc-50/90 p-5 dark:border-zinc-800 dark:bg-zinc-900/80">
            <p className="text-xs font-semibold uppercase tracking-[0.2em] text-zinc-500 dark:text-zinc-400">
              Account guidance
            </p>
            <div className="mt-4 grid gap-3 text-sm text-zinc-600 dark:text-zinc-300">
              <div className="flex items-center justify-between gap-3 rounded-2xl border border-zinc-200 bg-white/90 px-4 py-3 dark:border-zinc-800 dark:bg-zinc-950/80">
                <span>Booked amount</span>
                <strong className="text-zinc-950 dark:text-zinc-50">
                  {formatCurrency(summary.booked_amount)}
                </strong>
              </div>
              <div className="flex items-center justify-between gap-3 rounded-2xl border border-zinc-200 bg-white/90 px-4 py-3 dark:border-zinc-800 dark:bg-zinc-950/80">
                <span>Remaining units</span>
                <strong className="text-zinc-950 dark:text-zinc-50">
                  {remainingUnits === null ? 'Unlimited' : formatUnits(remainingUnits)}
                </strong>
              </div>
              <div className="flex items-center justify-between gap-3 rounded-2xl border border-zinc-200 bg-white/90 px-4 py-3 dark:border-zinc-800 dark:bg-zinc-950/80">
                <span>Billing status</span>
                <strong className="text-zinc-950 dark:text-zinc-50">
                  {membership?.status ?? 'inactive'}
                </strong>
              </div>
              <p>
                Use this surface to verify whether the current workspace still has quota headroom,
                whether booked spend matches expected usage, and whether the ledger is ready for
                finance review.
              </p>
            </div>
          </article>
        </div>
      </Surface>

      <Surface
        detail={
          membership
            ? `${membership.plan_name} is the current membership posture for this workspace.`
            : 'No active membership is recorded yet. Subscription purchases will appear here once activated.'
        }
        title="Membership posture"
      >
        <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
          <article className="rounded-[28px] border border-zinc-200 bg-zinc-50/90 p-5 dark:border-zinc-800 dark:bg-zinc-900/80">
            <span className="text-xs font-semibold uppercase tracking-[0.2em] text-zinc-500 dark:text-zinc-400">
              Plan
            </span>
            <strong className="mt-3 block text-2xl text-zinc-950 dark:text-zinc-50">
              {membership?.plan_name ?? 'No membership'}
            </strong>
          </article>
          <article className="rounded-[28px] border border-zinc-200 bg-zinc-50/90 p-5 dark:border-zinc-800 dark:bg-zinc-900/80">
            <span className="text-xs font-semibold uppercase tracking-[0.2em] text-zinc-500 dark:text-zinc-400">
              Status
            </span>
            <strong className="mt-3 block text-2xl text-zinc-950 dark:text-zinc-50">
              {membership?.status ?? 'inactive'}
            </strong>
          </article>
          <article className="rounded-[28px] border border-zinc-200 bg-zinc-50/90 p-5 dark:border-zinc-800 dark:bg-zinc-900/80">
            <span className="text-xs font-semibold uppercase tracking-[0.2em] text-zinc-500 dark:text-zinc-400">
              Included units
            </span>
            <strong className="mt-3 block text-2xl text-zinc-950 dark:text-zinc-50">
              {membership ? formatUnits(membership.included_units) : 'n/a'}
            </strong>
          </article>
          <article className="rounded-[28px] border border-zinc-200 bg-zinc-50/90 p-5 dark:border-zinc-800 dark:bg-zinc-900/80">
            <span className="text-xs font-semibold uppercase tracking-[0.2em] text-zinc-500 dark:text-zinc-400">
              Billing
            </span>
            <strong className="mt-3 block text-2xl text-zinc-950 dark:text-zinc-50">
              {membership?.price_label ?? 'n/a'}
            </strong>
          </article>
        </div>
      </Surface>

      <Surface
        detail="Ledger overview keeps the searchable account evidence in one commercial-grade table."
        title="Ledger overview"
      >
        <DataTable
          columns={[
            {
              key: 'project',
              label: 'Project',
              render: (row) => (
                <div className="flex flex-col gap-1">
                  <strong>{row.project_id}</strong>
                  <span className="text-xs text-zinc-500 dark:text-zinc-400">
                    {row.project_id === summary.project_id ? 'Current workspace' : 'Linked project'}
                  </span>
                </div>
              ),
            },
            { key: 'units', label: 'Units', render: (row) => formatUnits(row.units) },
            { key: 'amount', label: 'Amount', render: (row) => formatCurrency(row.amount) },
            {
              key: 'share',
              label: 'Booked share',
              render: (row) =>
                bookedAmount > 0
                  ? formatPercentage(Math.min(1, Math.abs(row.amount) / bookedAmount))
                  : 'n/a',
            },
            {
              key: 'health',
              label: 'Quota health',
              render: (row) => (
                <Pill
                  tone={
                    row.project_id === summary.project_id
                      ? summary.exhausted
                        ? 'warning'
                        : 'positive'
                      : 'default'
                  }
                >
                  {row.project_id === summary.project_id
                    ? summary.exhausted
                      ? 'Exhausted'
                      : 'Healthy'
                    : 'Observed'}
                </Pill>
              ),
            },
          ]}
          empty={(
            <div className="mx-auto flex max-w-[32rem] flex-col items-center gap-2 text-center">
              <strong className="text-base font-semibold text-zinc-950 dark:text-zinc-50">
                {t('No ledger entries for this slice')}
              </strong>
              <p className="text-sm text-zinc-500 dark:text-zinc-400">
                {ledger.length
                  ? 'Adjust the search or wait for quota and billing activity to populate the ledger.'
                  : status}
              </p>
            </div>
          )}
          getKey={(row, index) => `${row.project_id}-${row.units}-${index}`}
          rows={visibleLedger}
        />
      </Surface>
    </div>
  );
}
