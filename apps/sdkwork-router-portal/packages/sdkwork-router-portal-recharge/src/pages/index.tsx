import { startTransition, useEffect, useMemo, useState } from 'react';
import type { ChangeEvent, FormEvent } from 'react';

import {
  formatDateTime,
  formatUnits,
  usePortalI18n,
} from 'sdkwork-router-portal-commons';
import { Button } from 'sdkwork-router-portal-commons/framework/actions';
import {
  Badge,
  DataTable,
} from 'sdkwork-router-portal-commons/framework/display';
import { Input } from 'sdkwork-router-portal-commons/framework/entry';
import { EmptyState } from 'sdkwork-router-portal-commons/framework/feedback';
import {
  Card,
  CardContent,
} from 'sdkwork-router-portal-commons/framework/layout';
import { portalErrorMessage } from 'sdkwork-router-portal-portal-api';
import type {
  PortalCommerceOrder,
  PortalCommerceOrderStatus,
  PortalCustomRechargePolicy,
  PortalRechargeOption,
  ProjectBillingSummary,
} from 'sdkwork-router-portal-types';

import {
  createPortalRechargeOrder,
  loadPortalRechargePageData,
  previewPortalRechargeQuote,
} from '../repository';
import {
  buildPortalRechargeHistoryRows,
  buildPortalRechargeQuoteSnapshot,
  formatRechargeAmountInput,
  parseRechargeAmountInput,
  validatePortalRechargeAmount,
} from '../services';
import type {
  PortalRechargePageProps,
  PortalRechargeSelection,
} from '../types';

const PAGE_SIZE = 8;

const rechargeCurrencyFormatter = new Intl.NumberFormat('en-US', {
  style: 'currency',
  currency: 'USD',
  minimumFractionDigits: 2,
  maximumFractionDigits: 2,
});

function formatMoney(amountCents: number | null | undefined) {
  if (amountCents === null || amountCents === undefined || amountCents <= 0) {
    return '--';
  }

  return rechargeCurrencyFormatter.format(amountCents / 100);
}

function orderStatusLabel(
  status: PortalCommerceOrderStatus,
  t: ReturnType<typeof usePortalI18n>['t'],
) {
  switch (status) {
    case 'fulfilled':
      return t('Fulfilled');
    case 'pending_payment':
      return t('Payment pending');
    case 'failed':
      return t('Failed');
    case 'canceled':
      return t('Canceled');
    default:
      return t('Status');
  }
}

function orderStatusVariant(
  status: PortalCommerceOrderStatus,
): 'secondary' | 'success' | 'warning' {
  switch (status) {
    case 'fulfilled':
      return 'success';
    case 'pending_payment':
      return 'warning';
    default:
      return 'secondary';
  }
}

function formatBalance(
  summary: ProjectBillingSummary | null,
  t: ReturnType<typeof usePortalI18n>['t'],
) {
  if (!summary) {
    return t('Loading...');
  }

  return summary.remaining_units === null || summary.remaining_units === undefined
    ? t('Unlimited')
    : formatUnits(summary.remaining_units);
}

function resolveInitialAmountCents(
  options: PortalRechargeOption[],
  policy: PortalCustomRechargePolicy | null,
) {
  return (
    options.find((option) => option.recommended)?.amount_cents
    ?? options[0]?.amount_cents
    ?? policy?.suggested_amount_cents
    ?? null
  );
}

function resolveRechargeValidationMessage(
  validation: ReturnType<typeof validatePortalRechargeAmount>,
  policy: PortalCustomRechargePolicy | null,
  t: ReturnType<typeof usePortalI18n>['t'],
) {
  switch (validation) {
    case 'disabled':
      return t('Custom recharge is currently disabled for this workspace.');
    case 'below_minimum':
      return t('Custom recharge must be at least {amount}.', {
        amount: formatRechargeAmountInput(policy?.min_amount_cents ?? null),
      });
    case 'above_maximum':
      return t('Custom recharge must stay below {amount}.', {
        amount: formatRechargeAmountInput(policy?.max_amount_cents ?? null),
      });
    case 'step_mismatch':
      return t('Custom recharge must increase in steps of {amount}.', {
        amount: formatRechargeAmountInput(policy?.step_amount_cents ?? null),
      });
    default:
      return null;
  }
}

function resolveOptionSupportText(
  option: PortalRechargeOption,
  isActive: boolean,
  t: ReturnType<typeof usePortalI18n>['t'],
) {
  if (isActive) {
    return t('Live quote ready for this selection.');
  }

  if (option.recommended) {
    return t('Best balance between immediate runway and effective unit return.');
  }

  return option.note?.trim() || t('Server-managed pricing keeps this amount ready for fast checkout.');
}

export function PortalRechargePage({ onNavigate }: PortalRechargePageProps) {
  const { t } = usePortalI18n();
  const loadingStatus = t('Loading recharge data...');
  const syncedStatus = t('Recharge data synced.');
  const defaultQuoteStatus = t('Choose amount to preview.');
  const [summary, setSummary] = useState<ProjectBillingSummary | null>(null);
  const [options, setOptions] = useState<PortalRechargeOption[]>([]);
  const [policy, setPolicy] = useState<PortalCustomRechargePolicy | null>(null);
  const [orders, setOrders] = useState<PortalCommerceOrder[]>([]);
  const [status, setStatus] = useState(loadingStatus);
  const [quoteStatus, setQuoteStatus] = useState(defaultQuoteStatus);
  const [quoteLoading, setQuoteLoading] = useState(false);
  const [createLoading, setCreateLoading] = useState(false);
  const [quote, setQuote] = useState<Awaited<ReturnType<typeof previewPortalRechargeQuote>> | null>(null);
  const [selection, setSelection] = useState<PortalRechargeSelection | null>(null);
  const [customAmountInput, setCustomAmountInput] = useState('');
  const [page, setPage] = useState(1);

  async function refreshRechargePage(input?: { preserveSelection?: boolean }) {
    setStatus(loadingStatus);

    try {
      const data = await loadPortalRechargePageData();
      setSummary(data.summary);
      setOptions(data.rechargeOptions);
      setPolicy(data.customRechargePolicy);
      setOrders(data.orders);
      setStatus(syncedStatus);

      const nextAmountCents = input?.preserveSelection
        ? selection?.amountCents ?? resolveInitialAmountCents(data.rechargeOptions, data.customRechargePolicy)
        : resolveInitialAmountCents(data.rechargeOptions, data.customRechargePolicy);
      const nextMode = input?.preserveSelection ? selection?.mode ?? 'preset' : 'preset';

      if (nextAmountCents) {
        setSelection({
          amountCents: nextAmountCents,
          mode: nextMode,
        });
        if (nextMode === 'custom') {
          setCustomAmountInput(formatRechargeAmountInput(nextAmountCents));
        }
      }
    } catch (error) {
      setStatus(portalErrorMessage(error));
    }
  }

  useEffect(() => {
    void refreshRechargePage();
  }, [loadingStatus, syncedStatus]);

  useEffect(() => {
    const selectedAmountCents = selection?.amountCents ?? null;
    const remainingUnits = summary?.remaining_units ?? null;
    if (selectedAmountCents === null || !summary) {
      return;
    }
    const amountCents = selectedAmountCents;

    let cancelled = false;

    async function loadQuote() {
      setQuoteLoading(true);
      setQuoteStatus(t('Loading payment information...'));

      try {
        const nextQuote = await previewPortalRechargeQuote({
          amount_cents: amountCents,
          current_remaining_units: remainingUnits,
        });

        if (cancelled) {
          return;
        }

        setQuote(nextQuote);
        setQuoteStatus(t('Ready to create order.'));
      } catch (error) {
        if (!cancelled) {
          setQuote(null);
          setQuoteStatus(portalErrorMessage(error));
        }
      } finally {
        if (!cancelled) {
          setQuoteLoading(false);
        }
      }
    }

    void loadQuote();

    return () => {
      cancelled = true;
    };
  }, [selection, summary, t]);

  const sortedOptions = useMemo(
    () =>
      (options ?? [])
        .slice()
        .sort(
          (left, right) =>
            Number(right.recommended) - Number(left.recommended)
            || left.amount_cents - right.amount_cents,
        ),
    [options],
  );
  const recommendedOption = useMemo(
    () => sortedOptions.find((option) => option.recommended) ?? sortedOptions[0] ?? null,
    [sortedOptions],
  );
  const selectedPresetOption = useMemo(
    () =>
      selection?.mode === 'preset'
        ? sortedOptions.find((option) => option.amount_cents === selection.amountCents) ?? null
        : null,
    [selection, sortedOptions],
  );
  const rechargeOrders = useMemo(() => buildPortalRechargeHistoryRows(orders), [orders]);
  const pendingPaymentOrders = useMemo(
    () => rechargeOrders.filter((order) => order.status === 'pending_payment'),
    [rechargeOrders],
  );
  const totalPages = Math.max(1, Math.ceil(rechargeOrders.length / PAGE_SIZE));
  const currentPage = Math.min(Math.max(page, 1), totalPages);
  const visibleOrders = useMemo(() => {
    const start = (currentPage - 1) * PAGE_SIZE;
    return rechargeOrders.slice(start, start + PAGE_SIZE);
  }, [currentPage, rechargeOrders]);
  const quoteSnapshot = useMemo(
    () =>
      buildPortalRechargeQuoteSnapshot({
        customRechargePolicy: policy,
        quote,
        summary,
        t,
      }),
    [policy, quote, summary, t],
  );
  const pageStatus = status !== syncedStatus ? status : '';
  const currentBalanceLabel = formatBalance(summary, t);
  const recommendedAmountLabel = recommendedOption?.amount_label ?? formatMoney(policy?.suggested_amount_cents);
  const selectedAmountLabel = quoteSnapshot?.amountLabel
    ?? selectedPresetOption?.amount_label
    ?? formatMoney(selection?.amountCents ?? recommendedOption?.amount_cents ?? null);
  const pendingFollowUpLabel = t('{count} orders', { count: pendingPaymentOrders.length });

  useEffect(() => {
    setPage((current) => Math.min(Math.max(current, 1), totalPages));
  }, [totalPages]);

  async function handleCustomPreview(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const amountCents = parseRechargeAmountInput(customAmountInput);

    if (!amountCents) {
      setQuoteStatus(t('Enter a valid amount.'));
      return;
    }

    const validation = validatePortalRechargeAmount(amountCents, policy);
    const validationMessage = resolveRechargeValidationMessage(validation, policy, t);
    if (validationMessage) {
      setQuoteStatus(validationMessage);
      return;
    }

    startTransition(() => {
      setSelection({
        amountCents,
        mode: 'custom',
      });
    });
  }

  async function handleCreateRechargeOrder() {
    if (!selection?.amountCents) {
      setQuoteStatus(t('Select an amount first.'));
      return;
    }

    const validation = validatePortalRechargeAmount(selection.amountCents, policy);
    const validationMessage = resolveRechargeValidationMessage(validation, policy, t);
    if (validationMessage) {
      setQuoteStatus(validationMessage);
      return;
    }

    setCreateLoading(true);
    setQuoteStatus(t('Creating order...'));

    try {
      await createPortalRechargeOrder({
        amount_cents: selection.amountCents,
      });
      await refreshRechargePage({ preserveSelection: true });
      setQuoteStatus(t('Order created. Complete payment in billing.'));
      setPage(1);
    } catch (error) {
      setQuoteStatus(portalErrorMessage(error));
    } finally {
      setCreateLoading(false);
    }
  }

  return (
    <div className="space-y-6" data-slot="portal-recharge-page">
      {pageStatus ? (
        <div
          className="rounded-[26px] border border-primary-200/65 bg-primary-50/90 px-4 py-3 text-sm text-primary-800 shadow-sm dark:border-primary-900/35 dark:bg-primary-950/35 dark:text-primary-200"
          role="status"
        >
          {pageStatus}
        </div>
      ) : null}

      <div className="grid gap-6 xl:grid-cols-[minmax(0,1.08fr)_minmax(22rem,0.92fr)] xl:items-start">
        <Card
          className="overflow-hidden border-primary-200/65 bg-[linear-gradient(180deg,rgba(255,255,255,0.98),rgba(244,248,255,0.95))] dark:border-primary-900/35 dark:bg-[linear-gradient(180deg,rgba(9,14,24,0.98),rgba(7,10,18,0.92))]"
          data-slot="portal-recharge-options"
          style={{
            boxShadow: '0 30px 84px color-mix(in srgb, var(--theme-primary-500) 12%, rgba(15,23,42,0.12))',
          }}
        >
          <CardContent className="space-y-6 p-5 sm:p-6">
            <div
              className="relative overflow-hidden rounded-[32px] border border-primary-200/70 bg-[linear-gradient(135deg,rgba(23,37,84,0.98),rgba(37,99,235,0.92)_52%,rgba(191,219,254,0.78)_100%)] px-5 py-5 text-white shadow-[0_24px_70px_rgba(15,23,42,0.24)] dark:border-primary-700/35 dark:bg-[linear-gradient(135deg,rgba(2,6,23,0.98),rgba(23,37,84,0.96)_55%,rgba(29,78,216,0.85)_100%)] sm:px-6 sm:py-6"
              data-slot="portal-recharge-selection-hero"
            >
              <div
                aria-hidden="true"
                className="pointer-events-none absolute -right-10 -top-10 h-32 w-32 rounded-full bg-white/18 blur-3xl"
              />
              <div className="relative grid gap-5 lg:grid-cols-[minmax(0,1fr)_auto] lg:items-end">
                <div className="space-y-3">
                  <Badge variant="secondary">{t('Decision Studio')}</Badge>
                  <div className="space-y-2">
                    <h2 className="text-[1.9rem] font-semibold tracking-tight text-white">
                      {t('Recharge options')}
                    </h2>
                    <p className="max-w-[38rem] text-sm leading-6 text-primary-100">
                      {t('Move from balance posture to live quote in one surface, then finish settlement in billing without losing the order trail.')}
                    </p>
                  </div>
                </div>

                <div className="min-w-[14rem] rounded-[26px] border border-white/15 bg-white/10 px-4 py-4 backdrop-blur-sm">
                  <span className="text-[11px] font-semibold uppercase tracking-[0.18em] text-primary-100">
                    {t('Current balance')}
                  </span>
                  <strong className="mt-3 block text-[2.3rem] font-semibold tracking-tight text-white">
                    {currentBalanceLabel}
                  </strong>
                  <p className="mt-2 text-sm text-primary-100">
                    {t('The recharge amount below updates the quote and projected runway immediately.')}
                  </p>
                </div>
              </div>
            </div>

            <div
              className="grid gap-3 sm:grid-cols-3"
              data-slot="portal-recharge-posture-strip"
            >
              <div className="rounded-[24px] border border-primary-200/60 bg-primary-50/80 px-4 py-4 dark:border-primary-900/30 dark:bg-primary-950/22">
                <span className="text-[11px] font-semibold uppercase tracking-[0.18em] text-primary-700 dark:text-primary-300">
                  {t('Current balance')}
                </span>
                <strong className="mt-2 block text-lg font-semibold text-primary-950 dark:text-primary-50">
                  {currentBalanceLabel}
                </strong>
                <p className="mt-2 text-sm text-primary-700 dark:text-primary-300">
                  {t('Live workspace balance before the next quota edge is reached.')}
                </p>
              </div>
              <div className="rounded-[24px] border border-primary-200/60 bg-white/88 px-4 py-4 dark:border-primary-900/30 dark:bg-primary-950/18">
                <span className="text-[11px] font-semibold uppercase tracking-[0.18em] text-primary-700 dark:text-primary-300">
                  {t('Recommended next top-up')}
                </span>
                <strong className="mt-2 block text-lg font-semibold text-primary-950 dark:text-primary-50">
                  {recommendedAmountLabel}
                </strong>
                <p className="mt-2 text-sm text-zinc-600 dark:text-zinc-300">
                  {t('Use the curated amount when you want the cleanest balance-to-headroom tradeoff.')}
                </p>
              </div>
              <div className="rounded-[24px] border border-primary-200/60 bg-white/88 px-4 py-4 dark:border-primary-900/30 dark:bg-primary-950/18">
                <span className="text-[11px] font-semibold uppercase tracking-[0.18em] text-primary-700 dark:text-primary-300">
                  {t('Pending follow-up')}
                </span>
                <strong className="mt-2 block text-lg font-semibold text-primary-950 dark:text-primary-50">
                  {pendingFollowUpLabel}
                </strong>
                <p className="mt-2 text-sm text-zinc-600 dark:text-zinc-300">
                  {t('Queued orders remain visible below so finance operators can settle them without losing context.')}
                </p>
              </div>
            </div>

            <div
              className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3"
              data-slot="portal-recharge-options-grid"
            >
              {sortedOptions.map((option) => {
                const isActive =
                  selection?.amountCents === option.amount_cents && selection.mode === 'preset';

                return (
                  <button
                    className={`group relative overflow-hidden rounded-[30px] border p-5 text-left transition-all ${
                      isActive
                        ? 'border-primary-400/70 bg-[linear-gradient(145deg,var(--theme-primary-950),var(--theme-primary-700))] text-white shadow-[0_30px_72px_rgba(15,23,42,0.28)]'
                        : option.recommended
                          ? 'border-primary-300/70 bg-[linear-gradient(180deg,rgba(239,246,255,0.98),rgba(255,255,255,0.98))] text-primary-950 shadow-[0_20px_54px_rgba(15,23,42,0.08)] hover:border-primary-400/75 hover:shadow-[0_24px_60px_rgba(15,23,42,0.12)] dark:border-primary-700/45 dark:bg-[linear-gradient(180deg,rgba(15,23,42,0.92),rgba(9,14,24,0.95))] dark:text-primary-50'
                          : 'border-primary-200/65 bg-white/92 text-primary-950 shadow-[0_16px_42px_rgba(15,23,42,0.06)] hover:border-primary-300/70 hover:shadow-[0_20px_48px_rgba(15,23,42,0.1)] dark:border-primary-900/35 dark:bg-primary-950/16 dark:text-primary-50'
                    }`}
                    key={option.id}
                    onClick={() =>
                      startTransition(() => {
                        setSelection({
                          amountCents: option.amount_cents,
                          mode: 'preset',
                        });
                      })}
                    type="button"
                  >
                    <div
                      aria-hidden="true"
                      className={`pointer-events-none absolute -right-8 -top-8 h-24 w-24 rounded-full blur-2xl ${
                        isActive
                          ? 'bg-white/15'
                          : option.recommended
                            ? 'bg-primary-200/50 dark:bg-primary-500/18'
                            : 'bg-primary-100/55 dark:bg-primary-900/20'
                      }`}
                    />
                    <div className="relative space-y-4">
                      <div className="flex items-start justify-between gap-3">
                        <div className="space-y-2">
                          <div className="flex flex-wrap items-center gap-2">
                            {option.recommended ? (
                              <Badge variant={isActive ? 'secondary' : 'success'}>
                                {t('Recommended')}
                              </Badge>
                            ) : null}
                            <span className={`text-[11px] font-semibold uppercase tracking-[0.18em] ${
                              isActive ? 'text-primary-100' : 'text-primary-700 dark:text-primary-300'
                            }`}
                            >
                              {t('Preset amount')}
                            </span>
                          </div>
                          <strong className="block text-[2rem] font-semibold tracking-tight">
                            {option.amount_label}
                          </strong>
                        </div>
                        <span
                          aria-hidden="true"
                          className={`mt-1 h-3.5 w-3.5 rounded-full border ${
                            isActive
                              ? 'border-white/80 bg-white'
                              : 'border-primary-300/75 bg-transparent dark:border-primary-600/70'
                          }`}
                        />
                      </div>

                      <p className={`text-sm leading-6 ${
                        isActive ? 'text-primary-100' : 'text-zinc-600 dark:text-zinc-300'
                      }`}
                      >
                        {resolveOptionSupportText(option, isActive, t)}
                      </p>

                      <div className="grid gap-3 sm:grid-cols-2">
                      <div className={`rounded-[22px] border px-4 py-4 ${
                        isActive
                          ? 'border-white/12 bg-white/10'
                          : 'border-primary-200/55 bg-primary-50/75 dark:border-primary-900/28 dark:bg-primary-950/16'
                      }`}
                      >
                        <span className={`text-[11px] font-semibold uppercase tracking-[0.16em] ${
                          isActive ? 'text-primary-100' : 'text-primary-700 dark:text-primary-300'
                        }`}
                        >
                          {t('Granted units')}
                        </span>
                        <strong className="mt-2 block text-base font-semibold">
                          {formatUnits(option.granted_units)}
                        </strong>
                      </div>
                      <div className={`rounded-[22px] border px-4 py-4 ${
                        isActive
                          ? 'border-white/12 bg-white/10'
                          : 'border-primary-200/55 bg-primary-50/75 dark:border-primary-900/28 dark:bg-primary-950/16'
                      }`}
                      >
                        <span className={`text-[11px] font-semibold uppercase tracking-[0.16em] ${
                          isActive ? 'text-primary-100' : 'text-primary-700 dark:text-primary-300'
                        }`}
                        >
                          {t('Effective ratio')}
                        </span>
                        <strong className="mt-2 block text-base font-semibold">
                          {option.effective_ratio_label}
                        </strong>
                      </div>
                    </div>
                    </div>
                  </button>
                );
              })}
            </div>

            <div data-slot="portal-recharge-custom-tile">
              <form
                className={`group relative overflow-hidden rounded-[30px] border p-5 text-left transition-all ${
                  selection?.mode === 'custom'
                    ? 'border-primary-400/70 bg-[linear-gradient(145deg,var(--theme-primary-950),var(--theme-primary-700))] text-white shadow-[0_30px_72px_rgba(15,23,42,0.28)]'
                    : 'border-primary-300/65 bg-[linear-gradient(180deg,rgba(239,246,255,0.95),rgba(255,255,255,0.96))] text-primary-950 shadow-[0_18px_46px_rgba(15,23,42,0.08)] hover:border-primary-400/70 hover:shadow-[0_22px_52px_rgba(15,23,42,0.12)] dark:border-primary-900/35 dark:bg-[linear-gradient(180deg,rgba(12,18,30,0.9),rgba(9,14,24,0.95))] dark:text-primary-50'
                }`}
                data-slot="portal-recharge-custom-form"
                onSubmit={(event) => void handleCustomPreview(event)}
              >
                <div
                  aria-hidden="true"
                  className={`pointer-events-none absolute -right-8 -top-8 h-24 w-24 rounded-full blur-2xl ${
                    selection?.mode === 'custom' ? 'bg-white/18' : 'bg-primary-200/45 dark:bg-primary-500/18'
                  }`}
                />
                <div className="relative space-y-4">
                  <div className="flex items-start justify-between gap-3">
                    <div className="space-y-2">
                      <span className={`text-[11px] font-semibold uppercase tracking-[0.18em] ${
                        selection?.mode === 'custom' ? 'text-primary-100' : 'text-primary-700 dark:text-primary-300'
                      }`}
                      >
                        {t('Flexible amount')}
                      </span>
                      <h3 className="text-lg font-semibold">
                        {t('Custom amount')}
                      </h3>
                    </div>
                    <span
                      aria-hidden="true"
                      className={`mt-1 h-3.5 w-3.5 rounded-full border ${
                        selection?.mode === 'custom'
                          ? 'border-white/80 bg-white'
                          : 'border-primary-300/75 bg-transparent dark:border-primary-600/70'
                      }`}
                    />
                  </div>

                  <p className={`text-sm leading-6 ${
                    selection?.mode === 'custom' ? 'text-primary-100' : 'text-zinc-600 dark:text-zinc-300'
                  }`}
                  >
                    {t('Use a custom amount when the preset matrix is close but not precise enough for the next funding move.')}
                  </p>

                  <div className="grid gap-3">
                    <Input
                      className={selection?.mode === 'custom'
                        ? 'border-white/18 bg-white/10 text-white placeholder:text-primary-100'
                        : 'border-primary-200/70 bg-white/88 text-primary-950 placeholder:text-primary-400 dark:border-primary-900/35 dark:bg-primary-950/16 dark:text-primary-50 dark:placeholder:text-primary-300'}
                      inputMode="decimal"
                      onChange={(event: ChangeEvent<HTMLInputElement>) => {
                        setCustomAmountInput(event.target.value);
                        setQuoteStatus(defaultQuoteStatus);
                      }}
                      placeholder={t('Enter amount')}
                      value={customAmountInput}
                    />
                    <Button className="h-11 rounded-2xl px-5 shadow-sm" type="submit" variant="secondary">
                      {t('Preview amount')}
                    </Button>
                  </div>

                  {policy ? (
                    <div className="grid gap-2 text-sm sm:grid-cols-3">
                      <span className={`rounded-2xl border px-3 py-2 text-center ${
                        selection?.mode === 'custom'
                          ? 'border-white/12 bg-white/10 text-primary-50'
                          : 'border-primary-200/55 bg-primary-50/75 text-primary-700 dark:border-primary-900/28 dark:bg-primary-950/16 dark:text-primary-300'
                      }`}
                      >
                        {t('Min {amount}', { amount: formatRechargeAmountInput(policy.min_amount_cents) })}
                      </span>
                      <span className={`rounded-2xl border px-3 py-2 text-center ${
                        selection?.mode === 'custom'
                          ? 'border-white/12 bg-white/10 text-primary-50'
                          : 'border-primary-200/55 bg-primary-50/75 text-primary-700 dark:border-primary-900/28 dark:bg-primary-950/16 dark:text-primary-300'
                      }`}
                      >
                        {t('Step {amount}', { amount: formatRechargeAmountInput(policy.step_amount_cents) })}
                      </span>
                      <span className={`rounded-2xl border px-3 py-2 text-center ${
                        selection?.mode === 'custom'
                          ? 'border-white/12 bg-white/10 text-primary-50'
                          : 'border-primary-200/55 bg-primary-50/75 text-primary-700 dark:border-primary-900/28 dark:bg-primary-950/16 dark:text-primary-300'
                      }`}
                      >
                        {t('Max {amount}', { amount: formatRechargeAmountInput(policy.max_amount_cents) })}
                      </span>
                    </div>
                  ) : null}
                </div>
              </form>
            </div>
          </CardContent>
        </Card>

        <Card
          className="overflow-hidden border-primary-200/65 bg-[linear-gradient(180deg,rgba(255,255,255,0.98),rgba(242,247,255,0.94))] xl:sticky xl:top-6 dark:border-primary-900/35 dark:bg-[linear-gradient(180deg,rgba(9,14,24,0.98),rgba(7,10,18,0.96))]"
          data-slot="portal-recharge-quote-card"
          style={{
            boxShadow: '0 32px 90px color-mix(in srgb, var(--theme-primary-500) 14%, rgba(15,23,42,0.14))',
          }}
        >
          <CardContent className="relative space-y-5 p-5 sm:p-6">
            <div
              aria-hidden="true"
              className="pointer-events-none absolute right-0 top-0 h-40 w-40 translate-x-1/3 -translate-y-1/3 rounded-full bg-primary-200/55 blur-3xl dark:bg-primary-600/16"
            />
            <div className="relative space-y-2">
              <div className="flex flex-wrap items-center gap-2">
                <Badge variant="secondary">{t('Payment information')}</Badge>
                {selection?.mode === 'custom' ? (
                  <Badge variant="warning">{t('Custom amount')}</Badge>
                ) : selectedPresetOption?.recommended ? (
                  <Badge variant="success">{t('Recommended')}</Badge>
                ) : null}
              </div>
              <div className="space-y-1">
                <h2 className="text-[1.55rem] font-semibold tracking-tight text-primary-950 dark:text-primary-50">
                  {t('Payment information')}
                </h2>
                <p className="text-sm leading-6 text-primary-700 dark:text-primary-300">{quoteStatus}</p>
              </div>
            </div>

            <div
              className="rounded-[22px] border border-primary-200/60 bg-primary-50/82 px-4 py-3 text-sm text-primary-800 dark:border-primary-900/30 dark:bg-primary-950/24 dark:text-primary-200"
              data-slot="portal-recharge-quote-note"
            >
              {t('Checkout stays in billing after order creation.')}
            </div>

            {quoteSnapshot ? (
              <div className="space-y-4">
                <div
                  className="relative overflow-hidden rounded-[30px] border border-primary-200/70 bg-[linear-gradient(145deg,rgba(23,37,84,0.98),rgba(37,99,235,0.92)_55%,rgba(191,219,254,0.78)_100%)] p-5 text-white shadow-[0_22px_58px_rgba(15,23,42,0.24)] dark:border-primary-700/35 dark:bg-[linear-gradient(145deg,rgba(2,6,23,0.98),rgba(23,37,84,0.95)_58%,rgba(29,78,216,0.82)_100%)]"
                  data-slot="portal-recharge-quote-hero"
                >
                  <div
                    aria-hidden="true"
                    className="pointer-events-none absolute -right-8 -top-8 h-24 w-24 rounded-full bg-white/18 blur-2xl"
                  />
                  <div className="relative flex items-start justify-between gap-3">
                    <div className="space-y-2">
                      <span className="text-[11px] font-semibold uppercase tracking-[0.18em] text-primary-100">
                        {selection?.mode === 'custom' ? t('Custom amount') : t('Selected amount')}
                      </span>
                      <strong className="block text-[2.4rem] font-semibold tracking-tight text-white">
                        {selectedAmountLabel}
                      </strong>
                    </div>
                    <Badge variant="secondary">{quoteSnapshot.pricingRuleLabel}</Badge>
                  </div>
                </div>

                <div className="grid gap-3 sm:grid-cols-3" data-slot="portal-recharge-quote-metrics">
                  <div className="rounded-[22px] border border-primary-200/60 bg-white/88 px-4 py-4 dark:border-primary-900/30 dark:bg-primary-950/18">
                    <span className="text-[11px] font-semibold uppercase tracking-[0.16em] text-primary-700 dark:text-primary-300">
                      {t('Granted units')}
                    </span>
                    <strong className="mt-2 block text-lg font-semibold text-primary-950 dark:text-primary-50">
                      {quoteSnapshot.grantedUnitsLabel}
                    </strong>
                  </div>
                  <div className="rounded-[22px] border border-primary-200/60 bg-white/88 px-4 py-4 dark:border-primary-900/30 dark:bg-primary-950/18">
                    <span className="text-[11px] font-semibold uppercase tracking-[0.16em] text-primary-700 dark:text-primary-300">
                      {t('Effective ratio')}
                    </span>
                    <strong className="mt-2 block text-lg font-semibold text-primary-950 dark:text-primary-50">
                      {quoteSnapshot.effectiveRatioLabel}
                    </strong>
                  </div>
                  <div className="rounded-[22px] border border-primary-200/60 bg-white/88 px-4 py-4 dark:border-primary-900/30 dark:bg-primary-950/18">
                    <span className="text-[11px] font-semibold uppercase tracking-[0.16em] text-primary-700 dark:text-primary-300">
                      {t('Projected balance')}
                    </span>
                    <strong className="mt-2 block text-lg font-semibold text-primary-950 dark:text-primary-50">
                      {quoteSnapshot.projectedBalanceLabel}
                    </strong>
                  </div>
                </div>

                <Button
                  className="h-12 w-full rounded-2xl bg-[linear-gradient(135deg,var(--theme-primary-950),var(--theme-primary-600))] text-sm font-semibold text-white shadow-[0_18px_42px_rgba(15,23,42,0.24)] hover:opacity-95 dark:bg-[linear-gradient(135deg,var(--theme-primary-900),var(--theme-primary-500))]"
                  data-slot="portal-recharge-primary-cta"
                  disabled={quoteLoading || createLoading || !selection?.amountCents}
                  onClick={() => void handleCreateRechargeOrder()}
                >
                  {createLoading ? t('Creating...') : t('Create recharge order')}
                </Button>
              </div>
            ) : (
              <EmptyState
                description={t('Select a package or preview a custom amount to review payment information before you create the order.')}
                title={quoteLoading ? t('Loading payment information') : t('No package selected')}
              />
            )}
          </CardContent>
        </Card>
      </div>

      <Card
        className="border-primary-200/65 bg-[linear-gradient(180deg,rgba(255,255,255,0.96),rgba(246,249,255,0.94))] shadow-sm dark:border-primary-900/35 dark:bg-[linear-gradient(180deg,rgba(10,14,24,0.96),rgba(8,12,22,0.94))]"
        data-slot="portal-recharge-history-table"
      >
        <CardContent className="space-y-4 p-5">
          <div className="flex flex-wrap items-start justify-between gap-3" data-slot="portal-recharge-history-header">
            <div className="space-y-2">
              <h2 className="text-lg font-semibold text-primary-950 dark:text-primary-50">
                {t('Recharge history')}
              </h2>
              <p className="max-w-[40rem] text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                {t('Track queued, settled, and failed recharge orders here so the buying decision above and the payment follow-up below stay in one operating loop.')}
              </p>
            </div>
            <div className="flex flex-wrap items-center gap-2">
              <Badge variant={pendingPaymentOrders.length ? 'warning' : 'secondary'}>
                {t('{count} Pending payment queue', {
                  count: pendingPaymentOrders.length,
                })}
              </Badge>
              <Button onClick={() => onNavigate('billing')} variant="secondary">
                {t('Open billing workbench')}
              </Button>
            </div>
          </div>

          <DataTable
            columns={[
              {
                id: 'recorded',
                header: t('Recorded'),
                cell: (row: PortalCommerceOrder) => formatDateTime(row.created_at_ms),
              },
              {
                id: 'amount',
                header: t('Amount'),
                cell: (row: PortalCommerceOrder) => row.payable_price_label,
              },
              {
                id: 'units',
                header: t('Granted units'),
                cell: (row: PortalCommerceOrder) => formatUnits(row.granted_units + row.bonus_units),
              },
              {
                id: 'kind',
                header: t('Kind'),
                cell: (row: PortalCommerceOrder) => (
                  <Badge variant="secondary">
                    {row.target_kind === 'custom_recharge' ? t('Custom recharge') : t('Recharge pack')}
                  </Badge>
                ),
              },
              {
                id: 'status',
                header: t('Status'),
                cell: (row: PortalCommerceOrder) => (
                  <Badge variant={orderStatusVariant(row.status)}>
                    {orderStatusLabel(row.status, t)}
                  </Badge>
                ),
              },
            ]}
            emptyState={(
                <div className="mx-auto flex max-w-[32rem] flex-col items-center gap-2 text-center">
                <strong className="text-base font-semibold text-primary-950 dark:text-primary-50">
                  {t('No recharge history yet')}
                </strong>
                <p className="text-sm text-zinc-600 dark:text-zinc-300">
                  {t('The first created recharge order will appear here for payment follow-up and settlement review.')}
                </p>
              </div>
            )}
            footer={(
              <div className="flex flex-col gap-3 rounded-2xl border border-primary-200/55 bg-primary-50/75 px-4 py-3 dark:border-primary-900/28 dark:bg-primary-950/18 sm:flex-row sm:items-center sm:justify-between">
                <span className="text-sm font-medium text-primary-700 dark:text-primary-300">
                  {t('Page {page} of {total}', {
                    page: currentPage,
                    total: totalPages,
                  })}
                </span>
                <div className="flex flex-wrap items-center gap-2">
                  <Button
                    disabled={currentPage <= 1}
                    onClick={() =>
                      startTransition(() => {
                        setPage((current) => Math.max(1, current - 1));
                      })}
                    variant="secondary"
                  >
                    {t('Previous page')}
                  </Button>
                  <Button
                    disabled={currentPage >= totalPages}
                    onClick={() =>
                      startTransition(() => {
                        setPage((current) => Math.min(totalPages, current + 1));
                      })}
                    variant="secondary"
                  >
                    {t('Next page')}
                  </Button>
                </div>
              </div>
            )}
            getRowId={(row: PortalCommerceOrder) => row.order_id}
            rows={visibleOrders}
          />
        </CardContent>
      </Card>
    </div>
  );
}
