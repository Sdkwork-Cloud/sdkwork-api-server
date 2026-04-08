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

export function PortalRechargePage({ onNavigate }: PortalRechargePageProps) {
  const { t } = usePortalI18n();
  const loadingStatus = t('Loading recharge workspace...');
  const syncedStatus = t('Recharge options and payment information are synced with the current workspace balance.');
  const defaultQuoteStatus = t('Choose a recharge package to load payment information.');
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
        setQuoteStatus(t('Payment information is ready. Review the package before creating the recharge order.'));
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

  useEffect(() => {
    setPage((current) => Math.min(Math.max(current, 1), totalPages));
  }, [totalPages]);

  async function handleCustomPreview(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const amountCents = parseRechargeAmountInput(customAmountInput);

    if (!amountCents) {
      setQuoteStatus(t('Enter a valid recharge amount.'));
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
      setQuoteStatus(t('Select a recharge amount before creating an order.'));
      return;
    }

    const validation = validatePortalRechargeAmount(selection.amountCents, policy);
    const validationMessage = resolveRechargeValidationMessage(validation, policy, t);
    if (validationMessage) {
      setQuoteStatus(validationMessage);
      return;
    }

    setCreateLoading(true);
    setQuoteStatus(t('Creating recharge order...'));

    try {
      await createPortalRechargeOrder({
        amount_cents: selection.amountCents,
      });
      await refreshRechargePage({ preserveSelection: true });
      setQuoteStatus(t('Recharge order created. Complete payment from billing when you are ready to settle it.'));
      setPage(1);
    } catch (error) {
      setQuoteStatus(portalErrorMessage(error));
    } finally {
      setCreateLoading(false);
    }
  }

  return (
    <div className="space-y-5" data-slot="portal-recharge-page">
      {pageStatus ? (
        <div
          className="rounded-[24px] border border-zinc-200 bg-zinc-50/90 px-4 py-3 text-sm text-zinc-600 shadow-sm dark:border-zinc-800 dark:bg-zinc-900/60 dark:text-zinc-300"
          role="status"
        >
          {pageStatus}
        </div>
      ) : null}

      <div className="grid gap-5 xl:grid-cols-[1.14fr_0.86fr] xl:items-start">
        <Card
          className="overflow-hidden border-zinc-200 bg-white shadow-[0_24px_80px_rgba(15,23,42,0.07)] dark:border-zinc-800 dark:bg-zinc-950"
          data-slot="portal-recharge-options"
        >
          <CardContent className="space-y-6 p-5 sm:p-6">
            <div className="space-y-3">
              <Badge variant="secondary">{t('Commercial recharge')}</Badge>
              <div className="space-y-2">
                <h2 className="text-[1.8rem] font-semibold tracking-tight text-zinc-950 dark:text-zinc-50">
                  {t('Recharge options')}
                </h2>
                <p className="max-w-[42rem] text-sm leading-6 text-zinc-500 dark:text-zinc-400">
                  {t('Choose the package that restores your workspace runway fastest, or switch to a custom amount when you need tighter control.')}
                </p>
              </div>
            </div>

            <div className="grid gap-4 md:grid-cols-2">
              {sortedOptions.map((option) => {
                const isActive =
                  selection?.amountCents === option.amount_cents && selection.mode === 'preset';

                return (
                  <button
                    className={`group rounded-[30px] border p-5 text-left transition-all ${
                      isActive
                        ? 'border-zinc-950 bg-zinc-950 text-white shadow-[0_26px_60px_rgba(15,23,42,0.22)] dark:border-zinc-50 dark:bg-zinc-50 dark:text-zinc-950'
                        : option.recommended
                          ? 'border-zinc-950/15 bg-[linear-gradient(180deg,rgba(255,255,255,0.98),rgba(244,244,245,0.9))] shadow-[0_18px_48px_rgba(15,23,42,0.08)] hover:border-zinc-950/30 dark:border-zinc-700 dark:bg-[linear-gradient(180deg,rgba(24,24,27,0.96),rgba(39,39,42,0.88))]'
                          : 'border-zinc-200 bg-white shadow-sm hover:border-zinc-300 hover:shadow-[0_16px_40px_rgba(15,23,42,0.06)] dark:border-zinc-800 dark:bg-zinc-950'
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
                    <div className="flex items-start justify-between gap-4">
                      <div className="space-y-3">
                        <div className="flex flex-wrap items-center gap-2">
                          {option.recommended ? (
                            <Badge variant={isActive ? 'secondary' : 'success'}>
                              {t('Recommended')}
                            </Badge>
                          ) : null}
                          <span className={`text-[11px] font-semibold uppercase tracking-[0.18em] ${
                            isActive ? 'text-zinc-300 dark:text-zinc-700' : 'text-zinc-500 dark:text-zinc-400'
                          }`}
                          >
                            {t('Recharge pack')}
                          </span>
                        </div>
                        <div className="space-y-1">
                          <strong className="block text-[1.85rem] font-semibold tracking-tight">
                            {option.amount_label}
                          </strong>
                          <p className={`text-sm leading-6 ${
                            isActive ? 'text-zinc-300 dark:text-zinc-700' : 'text-zinc-500 dark:text-zinc-400'
                          }`}
                          >
                            {option.note}
                          </p>
                        </div>
                      </div>
                      {isActive ? (
                        <Badge variant="secondary">{t('Selected')}</Badge>
                      ) : null}
                    </div>

                    <div className="mt-5 grid gap-3 sm:grid-cols-2">
                      <div className={`rounded-[22px] border px-4 py-4 ${
                        isActive
                          ? 'border-white/15 bg-white/10 dark:border-zinc-900/10 dark:bg-zinc-900/10'
                          : 'border-zinc-200 bg-zinc-50/80 dark:border-zinc-800 dark:bg-zinc-900/70'
                      }`}
                      >
                        <span className={`text-[11px] font-semibold uppercase tracking-[0.16em] ${
                          isActive ? 'text-zinc-300 dark:text-zinc-700' : 'text-zinc-500 dark:text-zinc-400'
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
                          ? 'border-white/15 bg-white/10 dark:border-zinc-900/10 dark:bg-zinc-900/10'
                          : 'border-zinc-200 bg-zinc-50/80 dark:border-zinc-800 dark:bg-zinc-900/70'
                      }`}
                      >
                        <span className={`text-[11px] font-semibold uppercase tracking-[0.16em] ${
                          isActive ? 'text-zinc-300 dark:text-zinc-700' : 'text-zinc-500 dark:text-zinc-400'
                        }`}
                        >
                          {t('Effective ratio')}
                        </span>
                        <strong className="mt-2 block text-base font-semibold">
                          {option.effective_ratio_label}
                        </strong>
                      </div>
                    </div>
                  </button>
                );
              })}
            </div>

            <form
              className="space-y-4 rounded-[30px] border border-dashed border-zinc-300 bg-[linear-gradient(180deg,rgba(250,250,250,0.96),rgba(244,244,245,0.92))] p-5 dark:border-zinc-700 dark:bg-[linear-gradient(180deg,rgba(24,24,27,0.96),rgba(39,39,42,0.82))]"
              data-slot="portal-recharge-custom-form"
              onSubmit={(event) => void handleCustomPreview(event)}
            >
              <div className="space-y-1">
                <h3 className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                  {t('Custom amount')}
                </h3>
                <p className="text-sm leading-6 text-zinc-500 dark:text-zinc-400">
                  {t('Need a tailored top-up? Enter a custom amount and the platform will refresh the payment information before you create the order.')}
                </p>
              </div>

              <div className="grid gap-3 lg:grid-cols-[minmax(0,1fr)_auto]">
                <Input
                  inputMode="decimal"
                  onChange={(event: ChangeEvent<HTMLInputElement>) => {
                    setCustomAmountInput(event.target.value);
                    setQuoteStatus(defaultQuoteStatus);
                  }}
                  placeholder={t('Enter amount')}
                  value={customAmountInput}
                />
                <Button className="h-11 rounded-2xl px-5" type="submit" variant="secondary">
                  {t('Preview custom amount')}
                </Button>
              </div>

              {policy ? (
                <div className="grid gap-2 text-sm text-zinc-500 dark:text-zinc-400 sm:grid-cols-3">
                  <span>
                    {t('Min {amount}', { amount: formatRechargeAmountInput(policy.min_amount_cents) })}
                  </span>
                  <span>
                    {t('Step {amount}', { amount: formatRechargeAmountInput(policy.step_amount_cents) })}
                  </span>
                  <span>
                    {t('Max {amount}', { amount: formatRechargeAmountInput(policy.max_amount_cents) })}
                  </span>
                </div>
              ) : null}
            </form>
          </CardContent>
        </Card>

        <Card
          className="overflow-hidden border-zinc-200 bg-[linear-gradient(180deg,rgba(255,255,255,0.98),rgba(244,244,245,0.94))] shadow-[0_26px_90px_rgba(15,23,42,0.09)] dark:border-zinc-800 dark:bg-[linear-gradient(180deg,rgba(24,24,27,0.98),rgba(39,39,42,0.92))]"
          data-slot="portal-recharge-quote-card"
        >
          <CardContent className="space-y-5 p-5 sm:p-6">
            <div className="space-y-2">
              <div className="flex flex-wrap items-center gap-2">
                <Badge variant="secondary">{t('Payment information')}</Badge>
                {selection?.mode === 'custom' ? (
                  <Badge variant="warning">{t('Custom amount')}</Badge>
                ) : selectedPresetOption?.recommended ? (
                  <Badge variant="success">{t('Recommended')}</Badge>
                ) : null}
              </div>
              <div className="space-y-1">
                <h2 className="text-[1.55rem] font-semibold tracking-tight text-zinc-950 dark:text-zinc-50">
                  {t('Payment information')}
                </h2>
                <p className="text-sm leading-6 text-zinc-500 dark:text-zinc-400">{quoteStatus}</p>
              </div>
            </div>

            {quoteSnapshot ? (
              <div className="space-y-4">
                <div className="rounded-[28px] border border-zinc-200 bg-white/85 p-5 shadow-sm dark:border-zinc-800 dark:bg-zinc-950/80">
                  <div className="flex items-start justify-between gap-3">
                    <div className="space-y-2">
                      <span className="text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                        {selection?.mode === 'custom' ? t('Custom amount') : t('Selected package')}
                      </span>
                      <strong className="block text-[2.2rem] font-semibold tracking-tight text-zinc-950 dark:text-zinc-50">
                        {quoteSnapshot.amountLabel}
                      </strong>
                    </div>
                    <Badge variant="secondary">{quoteSnapshot.pricingRuleLabel}</Badge>
                  </div>
                </div>

                <div className="grid gap-3">
                  <div className="flex items-center justify-between gap-3 rounded-[22px] border border-zinc-200 bg-white/90 px-4 py-3 dark:border-zinc-800 dark:bg-zinc-950/85">
                    <span className="text-sm text-zinc-500 dark:text-zinc-400">{t('Granted units')}</span>
                    <strong className="text-zinc-950 dark:text-zinc-50">{quoteSnapshot.grantedUnitsLabel}</strong>
                  </div>
                  <div className="flex items-center justify-between gap-3 rounded-[22px] border border-zinc-200 bg-white/90 px-4 py-3 dark:border-zinc-800 dark:bg-zinc-950/85">
                    <span className="text-sm text-zinc-500 dark:text-zinc-400">{t('Effective ratio')}</span>
                    <strong className="text-zinc-950 dark:text-zinc-50">{quoteSnapshot.effectiveRatioLabel}</strong>
                  </div>
                  <div className="flex items-center justify-between gap-3 rounded-[22px] border border-zinc-200 bg-white/90 px-4 py-3 dark:border-zinc-800 dark:bg-zinc-950/85">
                    <span className="text-sm text-zinc-500 dark:text-zinc-400">{t('Projected balance')}</span>
                    <strong className="text-zinc-950 dark:text-zinc-50">{quoteSnapshot.projectedBalanceLabel}</strong>
                  </div>
                  <div className="flex items-center justify-between gap-3 rounded-[22px] border border-zinc-200 bg-white/90 px-4 py-3 dark:border-zinc-800 dark:bg-zinc-950/85">
                    <span className="text-sm text-zinc-500 dark:text-zinc-400">{t('Current balance')}</span>
                    <strong className="text-zinc-950 dark:text-zinc-50">{formatBalance(summary, t)}</strong>
                  </div>
                </div>

                <Button
                  className="h-12 w-full rounded-2xl text-sm font-semibold shadow-sm"
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
        className="border-zinc-200 bg-white shadow-none dark:border-zinc-800 dark:bg-zinc-950"
        data-slot="portal-recharge-history-table"
      >
        <CardContent className="space-y-4 p-5">
          <div className="flex flex-wrap items-start justify-between gap-3">
            <div className="space-y-1">
              <h2 className="text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                {t('Recharge history')}
              </h2>
              <p className="text-sm text-zinc-500 dark:text-zinc-400">
                {t('Recent recharge orders stay visible here so finance operators can confirm payment progress without interrupting the purchase flow above.')}
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
                <strong className="text-base font-semibold text-zinc-950 dark:text-zinc-50">
                  {t('No recharge history yet')}
                </strong>
                <p className="text-sm text-zinc-500 dark:text-zinc-400">
                  {t('Create the first recharge order to start building balance history for this workspace.')}
                </p>
              </div>
            )}
            footer={(
              <div className="flex flex-col gap-3 rounded-2xl border border-zinc-200 bg-zinc-50/80 px-4 py-3 dark:border-zinc-800 dark:bg-zinc-900/50 sm:flex-row sm:items-center sm:justify-between">
                <span className="text-sm font-medium text-zinc-600 dark:text-zinc-300">
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
