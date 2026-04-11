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
  buildPortalRechargePickerOptions,
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

function resolveInitialAmountCents(
  options: PortalRechargeOption[],
  policy: PortalCustomRechargePolicy | null,
) {
  const pickerOptions = buildPortalRechargePickerOptions(options, policy);
  return (
    pickerOptions.find((option) => option.recommended)?.amount_cents
    ?? pickerOptions[0]?.amount_cents
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
      return t('Custom amount unavailable.');
    case 'below_minimum':
      return t('Min {amount}.', {
        amount: formatRechargeAmountInput(policy?.min_amount_cents ?? null),
      });
    case 'above_maximum':
      return t('Max {amount}.', {
        amount: formatRechargeAmountInput(policy?.max_amount_cents ?? null),
      });
    case 'step_mismatch':
      return t('Step {amount}.', {
        amount: formatRechargeAmountInput(policy?.step_amount_cents ?? null),
      });
    default:
      return null;
  }
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

  const pickerOptions = useMemo(
    () => buildPortalRechargePickerOptions((options ?? []).slice(), policy),
    [options, policy],
  );
  const isCustomSelected = selection?.mode === 'custom';
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
          className="rounded-[24px] border border-primary-200/60 bg-primary-50/85 px-4 py-3 text-sm text-primary-800 shadow-sm dark:border-primary-900/45 dark:bg-primary-950/35 dark:text-primary-200"
          role="status"
        >
          {pageStatus}
        </div>
      ) : null}

      <div className="grid gap-6 xl:grid-cols-[minmax(0,1.08fr)_minmax(22rem,0.92fr)] xl:items-start">
        <Card
          className="overflow-hidden border-primary-200/70 bg-gradient-to-b from-primary-100/95 via-primary-50/88 to-primary-100/65 dark:border-primary-900/40 dark:from-primary-950/45 dark:via-primary-950/32 dark:to-zinc-950"
          data-slot="portal-recharge-options"
          style={{
            boxShadow: '0 24px 80px color-mix(in srgb, var(--theme-primary-500) 10%, rgba(15,23,42,0.08))',
          }}
        >
          <CardContent className="space-y-6 p-5 sm:p-6">
            <div>
              <h2 className="text-[1.8rem] font-semibold tracking-tight text-primary-950 dark:text-primary-50">
                {t('Recharge options')}
              </h2>
            </div>

            <div
              className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3"
              data-slot="portal-recharge-options-grid"
            >
              {pickerOptions.map((option) => {
                const isActive =
                  selection?.amountCents === option.amount_cents && selection.mode === 'preset';

                return (
                  <button
                    className={`group relative overflow-hidden rounded-[30px] border p-5 text-left transition-all ${
                      isActive
                        ? 'border-primary-400/70 bg-gradient-to-br from-primary-950 to-primary-700 text-white shadow-[0_28px_70px_rgba(15,23,42,0.28)] dark:border-primary-300/50 dark:from-primary-50 dark:to-primary-100 dark:text-primary-950'
                        : option.recommended
                          ? 'border-primary-300/70 bg-gradient-to-b from-primary-200/95 via-primary-100/85 to-primary-50/85 text-primary-950 shadow-[0_18px_48px_rgba(15,23,42,0.08)] hover:border-primary-400/75 hover:shadow-[0_24px_56px_rgba(15,23,42,0.12)] dark:border-primary-700/55 dark:from-primary-950/70 dark:via-primary-950/55 dark:to-zinc-950 dark:text-primary-50'
                          : 'border-primary-200/65 bg-gradient-to-b from-primary-100/80 via-primary-50/70 to-primary-100/50 text-primary-950 shadow-[0_14px_36px_rgba(15,23,42,0.06)] hover:border-primary-300/70 hover:shadow-[0_20px_44px_rgba(15,23,42,0.1)] dark:border-primary-800/40 dark:from-primary-950/42 dark:via-primary-950/30 dark:to-zinc-950 dark:text-primary-50'
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
                    {isActive ? (
                      <div
                        aria-hidden="true"
                        className="pointer-events-none absolute -right-10 -top-10 h-28 w-28 rounded-full bg-primary-300/20 blur-2xl dark:bg-primary-200/25"
                      />
                    ) : option.recommended ? (
                      <div
                        aria-hidden="true"
                        className="pointer-events-none absolute -right-8 -top-8 h-24 w-24 rounded-full bg-primary-200/30 blur-2xl dark:bg-primary-700/20"
                      />
                    ) : null}
                    <div className="flex items-start justify-between gap-4">
                      <div className="relative space-y-4">
                        <div className="flex flex-wrap items-center gap-2">
                          {option.recommended ? (
                            <Badge variant={isActive ? 'secondary' : 'success'}>
                              {t('Recommended')}
                            </Badge>
                          ) : null}
                        </div>
                        <div className="space-y-1">
                          <strong className="block text-[1.85rem] font-semibold tracking-tight">
                            {option.amount_label}
                          </strong>
                        </div>
                      </div>
                      <span
                        aria-hidden="true"
                        className={`mt-1 h-3.5 w-3.5 rounded-full border transition-colors ${
                          isActive
                            ? 'border-primary-200 bg-primary-200 dark:border-primary-950 dark:bg-primary-950'
                            : 'border-primary-300/70 bg-transparent dark:border-primary-700/70'
                        }`}
                      />
                    </div>

                    <div className="relative mt-5 grid gap-3 sm:grid-cols-2">
                      <div className={`rounded-[22px] border px-4 py-4 ${
                        isActive
                          ? 'border-primary-100/15 bg-primary-50/12 dark:border-primary-800/30 dark:bg-primary-950/10'
                          : 'border-primary-200/50 bg-primary-50/75 dark:border-primary-900/35 dark:bg-primary-950/20'
                      }`}
                      >
                        <span className={`text-[11px] font-semibold uppercase tracking-[0.16em] ${
                          isActive ? 'text-zinc-300 dark:text-primary-700' : 'text-primary-700 dark:text-primary-300'
                        }`}
                        >
                          {t('Granted units')}
                        </span>
                        <strong className={`mt-2 block text-base font-semibold ${
                          isActive ? 'text-white dark:text-primary-950' : 'text-primary-950 dark:text-primary-50'
                        }`}
                        >
                          {formatUnits(option.granted_units)}
                        </strong>
                      </div>
                      <div className={`rounded-[22px] border px-4 py-4 ${
                        isActive
                          ? 'border-primary-100/18 bg-primary-50/12 dark:border-primary-800/30 dark:bg-primary-950/10'
                          : 'border-primary-200/50 bg-primary-50/75 dark:border-primary-900/35 dark:bg-primary-950/20'
                      }`}
                      >
                        <span className={`text-[11px] font-semibold uppercase tracking-[0.16em] ${
                          isActive ? 'text-zinc-300 dark:text-primary-700' : 'text-primary-700 dark:text-primary-300'
                        }`}
                        >
                          {t('Effective ratio')}
                        </span>
                        <strong className={`mt-2 block text-base font-semibold ${
                          isActive ? 'text-white dark:text-primary-950' : 'text-primary-950 dark:text-primary-50'
                        }`}
                        >
                          {option.effective_ratio_label}
                        </strong>
                      </div>
                    </div>
                  </button>
                );
              })}

              <div data-slot="portal-recharge-custom-tile">
                <form
                  className={`group relative h-full overflow-hidden rounded-[30px] border p-5 text-left shadow-sm transition-all ${
                    isCustomSelected
                      ? 'border-primary-400/70 bg-gradient-to-br from-primary-950 to-primary-700 text-white shadow-[0_28px_70px_rgba(15,23,42,0.28)] dark:border-primary-300/50 dark:from-primary-50 dark:to-primary-100 dark:text-primary-950'
                      : 'border-primary-300/55 bg-gradient-to-b from-primary-200/92 via-primary-100/75 to-primary-50/70 text-primary-950 hover:border-primary-400/65 hover:shadow-[0_20px_48px_rgba(15,23,42,0.08)] dark:border-primary-900/35 dark:from-primary-950/45 dark:via-primary-950/30 dark:to-zinc-950 dark:text-primary-50'
                  }`}
                  data-slot="portal-recharge-custom-form"
                  onSubmit={(event) => void handleCustomPreview(event)}
                >
                  {isCustomSelected ? (
                    <div
                      aria-hidden="true"
                      className="pointer-events-none absolute -right-10 -top-10 h-28 w-28 rounded-full bg-primary-300/20 blur-2xl dark:bg-primary-200/25"
                    />
                  ) : null}
                  <div className="flex items-start justify-between gap-4">
                    <div className="relative space-y-1">
                      <h3 className={`text-sm font-semibold ${
                        isCustomSelected
                          ? 'text-white dark:text-primary-950'
                          : 'text-primary-950 dark:text-primary-50'
                      }`}
                      >
                        {t('Custom amount')}
                      </h3>
                    </div>
                    <span
                      aria-hidden="true"
                      className={`mt-1 h-3.5 w-3.5 rounded-full border transition-colors ${
                        isCustomSelected
                          ? 'border-primary-200 bg-primary-200 dark:border-primary-950 dark:bg-primary-950'
                          : 'border-primary-300/70 bg-transparent dark:border-primary-700/70'
                      }`}
                    />
                  </div>

                  <div className="relative mt-5 grid gap-3">
                    <Input
                      className={isCustomSelected
                        ? 'border-primary-100/18 bg-primary-50/12 text-white placeholder:text-zinc-300 dark:border-primary-200/45 dark:bg-primary-50/85 dark:text-primary-950 dark:placeholder:text-primary-700'
                        : 'border-primary-200/60 bg-primary-50/85 text-primary-950 placeholder:text-primary-500 dark:border-primary-800/60 dark:bg-primary-950/30 dark:text-primary-50 dark:placeholder:text-primary-400'}
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
                    <div className="relative mt-5 grid grid-cols-3 gap-2 text-sm">
                      <span className={`rounded-2xl border px-3 py-2 text-center ${
                        isCustomSelected
                          ? 'border-primary-100/15 bg-primary-50/12 text-zinc-100 dark:border-primary-200/50 dark:bg-primary-50/80 dark:text-primary-900'
                          : 'border-primary-200/50 bg-primary-50/80 text-primary-700 dark:border-primary-900/35 dark:bg-primary-950/20 dark:text-primary-300'
                      }`}
                      >
                        {t('Min {amount}', { amount: formatRechargeAmountInput(policy.min_amount_cents) })}
                      </span>
                      <span className={`rounded-2xl border px-3 py-2 text-center ${
                        isCustomSelected
                          ? 'border-primary-100/15 bg-primary-50/12 text-zinc-100 dark:border-primary-200/50 dark:bg-primary-50/80 dark:text-primary-900'
                          : 'border-primary-200/50 bg-primary-50/80 text-primary-700 dark:border-primary-900/35 dark:bg-primary-950/20 dark:text-primary-300'
                      }`}
                      >
                        {t('Step {amount}', { amount: formatRechargeAmountInput(policy.step_amount_cents) })}
                      </span>
                      <span className={`rounded-2xl border px-3 py-2 text-center ${
                        isCustomSelected
                          ? 'border-primary-100/15 bg-primary-50/12 text-zinc-100 dark:border-primary-200/50 dark:bg-primary-50/80 dark:text-primary-900'
                          : 'border-primary-200/50 bg-primary-50/80 text-primary-700 dark:border-primary-900/35 dark:bg-primary-950/20 dark:text-primary-300'
                      }`}
                      >
                        {t('Max {amount}', { amount: formatRechargeAmountInput(policy.max_amount_cents) })}
                      </span>
                    </div>
                  ) : null}
                </form>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card
          className="overflow-hidden border-primary-200/70 bg-gradient-to-b from-primary-100/95 via-primary-50/80 to-primary-100/55 xl:sticky xl:top-6 dark:border-primary-900/40 dark:from-primary-950/45 dark:via-primary-950/30 dark:to-zinc-950"
          data-slot="portal-recharge-quote-card"
          style={{
            boxShadow: '0 26px 90px color-mix(in srgb, var(--theme-primary-500) 12%, rgba(15,23,42,0.1))',
          }}
        >
          <CardContent className="relative space-y-5 p-5 sm:p-6">
            <div
              aria-hidden="true"
              className="pointer-events-none absolute right-0 top-0 h-40 w-40 translate-x-1/3 -translate-y-1/3 rounded-full bg-primary-200/50 blur-3xl dark:bg-primary-600/15"
            />
            <div className="space-y-2">
              <h2 className="text-[1.55rem] font-semibold tracking-tight text-primary-950 dark:text-primary-50">
                {t('Payment information')}
              </h2>
              <p className="text-sm leading-6 text-primary-700 dark:text-primary-300">{quoteStatus}</p>
            </div>

            {quoteSnapshot ? (
              <div className="space-y-4">
                <div
                  className="relative overflow-hidden rounded-[28px] border border-primary-200/70 bg-gradient-to-br from-primary-100/95 via-primary-50/85 to-primary-100/60 p-5 shadow-sm dark:border-primary-900/35 dark:from-primary-950/50 dark:via-primary-950/35 dark:to-zinc-950"
                  data-slot="portal-recharge-quote-hero"
                >
                  <div
                    aria-hidden="true"
                    className="pointer-events-none absolute -right-8 -top-8 h-24 w-24 rounded-full bg-primary-200/45 blur-2xl dark:bg-primary-500/20"
                  />
                  <div className="flex items-start justify-between gap-3">
                    <strong className="relative block text-[2.35rem] font-semibold tracking-tight text-primary-950 dark:text-primary-50">
                      {quoteSnapshot.amountLabel}
                    </strong>
                    <Badge variant="secondary">{quoteSnapshot.pricingRuleLabel}</Badge>
                  </div>
                </div>

                <div className="grid gap-3 sm:grid-cols-3" data-slot="portal-recharge-quote-metrics">
                  <div className="rounded-[22px] border border-primary-200/55 bg-primary-50/75 px-4 py-4 dark:border-primary-900/35 dark:bg-primary-950/20">
                    <span className="text-[11px] font-semibold uppercase tracking-[0.16em] text-primary-700 dark:text-primary-300">
                      {t('Granted units')}
                    </span>
                    <strong className="mt-2 block text-lg font-semibold text-primary-950 dark:text-primary-50">
                      {quoteSnapshot.grantedUnitsLabel}
                    </strong>
                  </div>
                  <div className="rounded-[22px] border border-primary-200/55 bg-primary-50/75 px-4 py-4 dark:border-primary-900/35 dark:bg-primary-950/20">
                    <span className="text-[11px] font-semibold uppercase tracking-[0.16em] text-primary-700 dark:text-primary-300">
                      {t('Effective ratio')}
                    </span>
                    <strong className="mt-2 block text-lg font-semibold text-primary-950 dark:text-primary-50">
                      {quoteSnapshot.effectiveRatioLabel}
                    </strong>
                  </div>
                  <div className="rounded-[22px] border border-primary-200/55 bg-primary-50/75 px-4 py-4 dark:border-primary-900/35 dark:bg-primary-950/20">
                    <span className="text-[11px] font-semibold uppercase tracking-[0.16em] text-primary-700 dark:text-primary-300">
                      {t('Projected balance')}
                    </span>
                    <strong className="mt-2 block text-lg font-semibold text-primary-950 dark:text-primary-50">
                      {quoteSnapshot.projectedBalanceLabel}
                    </strong>
                  </div>
                </div>

                <Button
                  className="h-12 w-full rounded-2xl bg-[linear-gradient(135deg,var(--theme-primary-950),var(--theme-primary-600))] text-sm font-semibold text-white shadow-[0_18px_40px_rgba(15,23,42,0.25)] hover:opacity-95 dark:bg-[linear-gradient(135deg,var(--theme-primary-900),var(--theme-primary-500))] dark:text-white"
                  data-slot="portal-recharge-primary-cta"
                  disabled={quoteLoading || createLoading || !selection?.amountCents}
                  onClick={() => void handleCreateRechargeOrder()}
                >
                  {createLoading ? t('Creating...') : t('Create recharge order')}
                </Button>
              </div>
            ) : (
              <div className="space-y-4">
                <div
                  className="grid gap-3 sm:grid-cols-3"
                  aria-hidden="true"
                  data-slot="portal-recharge-quote-skeleton"
                >
                  {Array.from({ length: 3 }).map((_, index) => (
                    <div
                      className="rounded-[22px] border border-primary-200/55 bg-primary-50/70 px-4 py-4 dark:border-primary-900/35 dark:bg-primary-950/20"
                      key={index}
                    >
                      <div className="h-2.5 w-16 rounded-full bg-zinc-200 dark:bg-zinc-800" />
                      <div className="mt-3 h-4 w-20 rounded-full bg-zinc-300 dark:bg-zinc-700" />
                    </div>
                  ))}
                </div>
                <Button
                  className="h-12 w-full rounded-2xl bg-[linear-gradient(135deg,var(--theme-primary-950),var(--theme-primary-600))] text-sm font-semibold text-white opacity-60 shadow-[0_18px_40px_rgba(15,23,42,0.16)] dark:bg-[linear-gradient(135deg,var(--theme-primary-900),var(--theme-primary-500))] dark:text-white"
                  data-slot="portal-recharge-primary-cta"
                  disabled
                >
                  {t('Create recharge order')}
                </Button>
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      <Card
        className="border-primary-200/70 bg-gradient-to-b from-primary-100/88 via-primary-50/74 to-primary-100/45 shadow-sm dark:border-primary-900/35 dark:from-primary-950/30 dark:via-primary-950/20 dark:to-zinc-950"
        data-slot="portal-recharge-history-table"
      >
        <CardContent className="space-y-4 p-5">
          <div className="flex flex-wrap items-start justify-between gap-3">
            <div>
              <h2 className="text-lg font-semibold text-primary-950 dark:text-primary-50">
                {t('Recharge history')}
              </h2>
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
                cell: (row) => formatDateTime(row.created_at_ms),
              },
              {
                id: 'amount',
                header: t('Amount'),
                cell: (row) => row.payable_price_label,
              },
              {
                id: 'units',
                header: t('Granted units'),
                cell: (row) => formatUnits(row.granted_units + row.bonus_units),
              },
              {
                id: 'kind',
                header: t('Kind'),
                cell: (row) => (
                  <Badge variant="secondary">
                    {row.target_kind === 'custom_recharge' ? t('Custom recharge') : t('Recharge pack')}
                  </Badge>
                ),
              },
              {
                id: 'status',
                header: t('Status'),
                cell: (row) => (
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
              </div>
            )}
            footer={(
              <div className="flex flex-col gap-3 rounded-2xl border border-primary-200/55 bg-primary-50/80 px-4 py-3 dark:border-primary-900/35 dark:bg-primary-950/25 sm:flex-row sm:items-center sm:justify-between">
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
            getRowId={(row) => row.order_id}
            rows={visibleOrders}
          />
        </CardContent>
      </Card>
    </div>
  );
}
