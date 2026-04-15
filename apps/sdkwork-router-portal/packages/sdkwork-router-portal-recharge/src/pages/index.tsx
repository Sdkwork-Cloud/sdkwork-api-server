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
  buildPortalRechargeOptionMerchandising,
  buildPortalRechargePendingPaymentSpotlight,
  buildPortalRechargeQuoteSnapshot,
  formatRechargeAmountInput,
  parseRechargeAmountInput,
  validatePortalRechargeAmount,
} from '../services';
import type { PortalRechargePendingPaymentSpotlight } from '../services';
import type {
  PortalRechargePageProps,
  PortalRechargeSelection,
} from '../types';
import {
  buildPortalRechargeFlowTrackerState,
  buildPortalRechargeMobileActionState,
  buildPortalRechargePrimaryActionState,
  resolvePortalRechargePostOrderHandoffActive,
} from './presentation';
import type {
  PortalRechargeFlowTrackerState,
  PortalRechargeMobileActionState,
  PortalRechargePrimaryActionState,
} from './presentation';

const PAGE_SIZE = 8;
type TranslateFn = ReturnType<typeof usePortalI18n>['t'];
type NavigateFn = PortalRechargePageProps['onNavigate'];
type PortalRechargeActionMode = PortalRechargePrimaryActionState['mode'];

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

function resolveOptionSupportText(input: {
  isActive: boolean;
  merchandising: {
    supportLabel: string;
  };
  t: ReturnType<typeof usePortalI18n>['t'];
}) {
  const { isActive, merchandising, t } = input;
  if (isActive) {
    return t('Live quote ready for this selection.');
  }

  return merchandising.supportLabel;
}

function resolveCustomSelectionStory(
  t: TranslateFn,
) {
  return {
    badge: t('Operator choice'),
    intentLabel: t('Precision funding for an exact target'),
    supportLabel: t('Use a custom amount when the preset packages are close but you already know the exact funding move.'),
  };
}

function triggerPortalRechargeAction(input: {
  mode: PortalRechargeActionMode;
  onNavigate: NavigateFn;
  onCreateOrder: () => void;
}) {
  const { mode, onNavigate, onCreateOrder } = input;
  if (mode === 'billing_handoff') {
    onNavigate('billing');
    return;
  }

  onCreateOrder();
}

function PortalRechargePrimaryActionButton(input: {
  actionState: PortalRechargePrimaryActionState;
  onNavigate: NavigateFn;
  onCreateOrder: () => void;
}) {
  const { actionState, onNavigate, onCreateOrder } = input;

  return (
    <Button
      className="h-12 w-full rounded-2xl bg-[linear-gradient(135deg,var(--theme-primary-950),var(--theme-primary-600))] text-sm font-semibold text-white shadow-[0_18px_42px_rgba(15,23,42,0.24)] hover:opacity-95 dark:bg-[linear-gradient(135deg,var(--theme-primary-900),var(--theme-primary-500))]"
      data-slot="portal-recharge-primary-cta"
      disabled={actionState.disabled}
      onClick={() =>
        triggerPortalRechargeAction({
          mode: actionState.mode,
          onNavigate,
          onCreateOrder,
        })}
    >
      {actionState.label}
    </Button>
  );
}

function PortalRechargePostOrderHandoffPanel(input: {
  active: boolean;
  amountLabel: string;
  t: TranslateFn;
  onNavigate: NavigateFn;
  onReset: () => void;
}) {
  const { active, amountLabel, t, onNavigate, onReset } = input;
  if (!active) {
    return null;
  }

  return (
    <div
      className="rounded-[26px] border border-emerald-200/75 bg-[linear-gradient(180deg,rgba(236,253,245,0.98),rgba(255,255,255,0.96))] p-4 shadow-[0_16px_38px_rgba(6,95,70,0.08)] dark:border-emerald-900/35 dark:bg-[linear-gradient(180deg,rgba(9,26,20,0.96),rgba(8,18,16,0.94))]"
      data-slot="portal-recharge-post-order-handoff"
    >
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="space-y-1">
          <span className="text-[11px] font-semibold uppercase tracking-[0.18em] text-emerald-700 dark:text-emerald-300">
            {t('Order ready for payment')}
          </span>
          <h3 className="text-lg font-semibold text-emerald-950 dark:text-emerald-50">
            {t('Continue in billing')}
          </h3>
          <p className="text-sm leading-6 text-emerald-800 dark:text-emerald-200">
            {t('The latest recharge order has been created and is now waiting for settlement in billing.')}
          </p>
        </div>
        <Badge variant="success">{amountLabel}</Badge>
      </div>

      <div className="mt-4 flex flex-wrap items-center justify-between gap-3">
        <p className="max-w-[22rem] text-sm leading-6 text-emerald-800 dark:text-emerald-200">
          {t('Finish payment capture before starting another recharge cycle to keep the settlement trail clean.')}
        </p>
        <div className="flex flex-wrap items-center gap-2">
          <Button onClick={onReset} variant="secondary">
            {t('Create another order')}
          </Button>
          <Button onClick={() => onNavigate('billing')} variant="secondary">
            {t('Continue in billing')}
          </Button>
        </div>
      </div>
    </div>
  );
}

function PortalRechargePendingSettlementCallout(input: {
  pendingPaymentSpotlight: PortalRechargePendingPaymentSpotlight | null;
  t: TranslateFn;
  onNavigate: NavigateFn;
}) {
  const { pendingPaymentSpotlight, t, onNavigate } = input;
  if (!pendingPaymentSpotlight) {
    return null;
  }

  return (
    <div
      className="rounded-[26px] border border-amber-200/75 bg-[linear-gradient(180deg,rgba(255,251,235,0.98),rgba(255,255,255,0.96))] p-4 shadow-[0_16px_38px_rgba(120,53,15,0.08)] dark:border-amber-900/35 dark:bg-[linear-gradient(180deg,rgba(28,20,10,0.96),rgba(18,14,10,0.94))]"
      data-slot="portal-recharge-next-step-callout"
    >
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="space-y-1">
          <span className="text-[11px] font-semibold uppercase tracking-[0.18em] text-amber-700 dark:text-amber-300">
            {pendingPaymentSpotlight.headline ?? t('Pending settlement queue')}
          </span>
          <h3 className="text-lg font-semibold text-amber-950 dark:text-amber-50">
            {pendingPaymentSpotlight.latestOrderLabel ?? t('Latest pending order')}
          </h3>
          <p className="text-sm leading-6 text-amber-800 dark:text-amber-200">
            {pendingPaymentSpotlight.detail}
          </p>
        </div>
        <Badge variant="warning">
          {t('{count} Pending payment queue', {
            count: pendingPaymentSpotlight.count,
          })}
        </Badge>
      </div>

      <div className="mt-4 grid gap-3 sm:grid-cols-2">
        <div className="rounded-[20px] border border-amber-200/70 bg-white/82 px-4 py-3 dark:border-amber-900/30 dark:bg-amber-950/12">
          <span className="text-[11px] font-semibold uppercase tracking-[0.16em] text-amber-700 dark:text-amber-300">
            {pendingPaymentSpotlight.latestOrderLabel ?? t('Latest pending order')}
          </span>
          <strong className="mt-2 block text-base font-semibold text-amber-950 dark:text-amber-50">
            {pendingPaymentSpotlight.latestOrder.payable_price_label}
          </strong>
        </div>
        <div className="rounded-[20px] border border-amber-200/70 bg-white/82 px-4 py-3 dark:border-amber-900/30 dark:bg-amber-950/12">
          <span className="text-[11px] font-semibold uppercase tracking-[0.16em] text-amber-700 dark:text-amber-300">
            {t('Recorded')}
          </span>
          <strong className="mt-2 block text-base font-semibold text-amber-950 dark:text-amber-50">
            {formatDateTime(pendingPaymentSpotlight.latestOrder.created_at_ms)}
          </strong>
        </div>
      </div>

      <div className="mt-4 flex flex-wrap items-center justify-between gap-3">
        <p className="max-w-[22rem] text-sm leading-6 text-amber-800 dark:text-amber-200">
          {t('The recharge order is already created. Billing is the next step to complete settlement and clear the queue.')}
        </p>
        <Button onClick={() => onNavigate('billing')} variant="secondary">
          {pendingPaymentSpotlight.ctaLabel ?? t('Open billing to complete payment')}
        </Button>
      </div>
    </div>
  );
}

function PortalRechargeMobileActionBar(input: {
  visible: boolean;
  actionState: PortalRechargeMobileActionState;
  onNavigate: NavigateFn;
  onCreateOrder: () => void;
}) {
  const { visible, actionState, onNavigate, onCreateOrder } = input;
  if (!visible) {
    return null;
  }

  return (
    <div
      className="fixed inset-x-0 bottom-0 z-40 border-t border-primary-200/70 bg-white/92 px-4 py-3 shadow-[0_-18px_40px_rgba(15,23,42,0.12)] backdrop-blur xl:hidden dark:border-primary-900/35 dark:bg-[rgba(8,12,22,0.94)]"
      data-slot="portal-recharge-mobile-cta"
    >
      <div className="mx-auto flex max-w-[88rem] items-center gap-3">
        <div className="min-w-0 flex-1">
          <span className="text-[11px] font-semibold uppercase tracking-[0.18em] text-primary-700 dark:text-primary-300">
            {actionState.eyebrow}
          </span>
          <strong className="mt-1 block truncate text-base font-semibold text-primary-950 dark:text-primary-50">
            {actionState.amountLabel}
          </strong>
          <p className="truncate text-sm text-zinc-600 dark:text-zinc-300">
            {actionState.supportingText}
          </p>
        </div>
        <Button
          className="h-11 rounded-2xl bg-[linear-gradient(135deg,var(--theme-primary-950),var(--theme-primary-600))] px-4 text-sm font-semibold text-white shadow-[0_16px_36px_rgba(15,23,42,0.18)] hover:opacity-95 dark:bg-[linear-gradient(135deg,var(--theme-primary-900),var(--theme-primary-500))]"
          disabled={actionState.disabled}
          onClick={() =>
            triggerPortalRechargeAction({
              mode: actionState.mode,
              onNavigate,
              onCreateOrder,
            })}
        >
          {actionState.buttonLabel}
        </Button>
      </div>
    </div>
  );
}

function PortalRechargeFlowTracker(input: {
  flowState: PortalRechargeFlowTrackerState;
  t: TranslateFn;
}) {
  const { flowState, t } = input;

  return (
    <div
      className="space-y-3 rounded-[26px] border border-primary-200/60 bg-[linear-gradient(180deg,rgba(255,255,255,0.96),rgba(244,248,255,0.92))] p-4 shadow-[0_16px_36px_rgba(15,23,42,0.06)] dark:border-primary-900/30 dark:bg-[linear-gradient(180deg,rgba(10,16,28,0.96),rgba(8,12,22,0.92))]"
      data-slot="portal-recharge-flow-tracker"
    >
      <div className="space-y-1">
        <span className="text-[11px] font-semibold uppercase tracking-[0.18em] text-primary-700 dark:text-primary-300">
          {flowState.title}
        </span>
        <p className="text-sm leading-6 text-zinc-600 dark:text-zinc-300">
          {t('The highlighted stage shows the next operator action across selection, order creation, and billing settlement.')}
        </p>
      </div>

      <div className="grid gap-3 sm:grid-cols-3">
        {flowState.steps.map((step, index) => {
          const tone = step.status === 'complete'
            ? {
                card: 'border-emerald-200/75 bg-emerald-50/82 dark:border-emerald-900/35 dark:bg-emerald-950/16',
                badge: 'bg-emerald-600 text-white',
                labelClassName: 'text-emerald-950 dark:text-emerald-50',
                detailClassName: 'text-emerald-800 dark:text-emerald-200',
                eyebrow: t('Done'),
              }
            : step.status === 'current'
              ? {
                  card: 'border-primary-300/75 bg-primary-50/86 dark:border-primary-700/40 dark:bg-primary-950/22',
                  badge: 'bg-[linear-gradient(135deg,var(--theme-primary-900),var(--theme-primary-600))] text-white',
                  labelClassName: 'text-primary-950 dark:text-primary-50',
                  detailClassName: 'text-primary-800 dark:text-primary-200',
                  eyebrow: t('Current'),
                }
              : step.status === 'attention'
                ? {
                    card: 'border-amber-200/75 bg-amber-50/82 dark:border-amber-900/35 dark:bg-amber-950/14',
                    badge: 'bg-amber-400 text-amber-950',
                    labelClassName: 'text-amber-950 dark:text-amber-50',
                    detailClassName: 'text-amber-800 dark:text-amber-200',
                    eyebrow: t('Queue'),
                  }
                : {
                    card: 'border-primary-200/60 bg-white/88 dark:border-primary-900/30 dark:bg-primary-950/12',
                    badge: 'border border-primary-300/70 bg-white text-primary-700 dark:border-primary-700/45 dark:bg-primary-950/24 dark:text-primary-200',
                    labelClassName: 'text-primary-950 dark:text-primary-50',
                    detailClassName: 'text-zinc-600 dark:text-zinc-300',
                    eyebrow: t('Next'),
                  };

          return (
            <div
              className={`relative overflow-hidden rounded-[22px] border px-4 py-4 ${tone.card}`}
              key={step.id}
            >
              {index < flowState.steps.length - 1 ? (
                <div
                  aria-hidden="true"
                  className="pointer-events-none absolute inset-y-0 -right-3 hidden w-6 sm:block"
                >
                  <div className="absolute left-1/2 top-1/2 h-[1px] w-6 -translate-y-1/2 bg-primary-200/80 dark:bg-primary-800/70" />
                </div>
              ) : null}

              <div className="relative space-y-3">
                <div className="flex items-start justify-between gap-3">
                  <span
                    className={`inline-flex h-8 min-w-8 items-center justify-center rounded-full px-2 text-sm font-semibold ${tone.badge}`}
                  >
                    {index + 1}
                  </span>
                  <span className="text-[11px] font-semibold uppercase tracking-[0.18em] text-primary-700/80 dark:text-primary-300/80">
                    {tone.eyebrow}
                  </span>
                </div>

                <div className="space-y-1">
                  <strong className={`block text-sm font-semibold ${tone.labelClassName}`}>
                    {step.label}
                  </strong>
                  <p className={`text-sm leading-6 ${tone.detailClassName}`}>
                    {step.detail}
                  </p>
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
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
  const [lastCreatedOrderId, setLastCreatedOrderId] = useState<string | null>(null);

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
  const optionMerchandisingById = useMemo(
    () =>
      new Map(
        sortedOptions.map((option) => [
          option.id,
          buildPortalRechargeOptionMerchandising({
            option,
            options: sortedOptions,
            t,
          }),
        ]),
      ),
    [sortedOptions, t],
  );
  const rechargeOrders = useMemo(() => buildPortalRechargeHistoryRows(orders), [orders]);
  const pendingPaymentOrders = useMemo(
    () => rechargeOrders.filter((order) => order.status === 'pending_payment'),
    [rechargeOrders],
  );
  const pendingPaymentSpotlight = useMemo(
    () => buildPortalRechargePendingPaymentSpotlight({ orders, t }),
    [orders, t],
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
  const selectionStory = useMemo(() => {
    if (selection?.mode === 'custom') {
      return resolveCustomSelectionStory(t);
    }

    if (selectedPresetOption) {
      return optionMerchandisingById.get(selectedPresetOption.id) ?? null;
    }

    if (recommendedOption) {
      return optionMerchandisingById.get(recommendedOption.id) ?? null;
    }

    return null;
  }, [optionMerchandisingById, recommendedOption, selectedPresetOption, selection, t]);
  const selectedAmountLabel = quoteSnapshot?.amountLabel
    ?? selectedPresetOption?.amount_label
    ?? formatMoney(selection?.amountCents ?? recommendedOption?.amount_cents ?? null);
  const pendingFollowUpLabel = t('{count} orders', { count: pendingPaymentOrders.length });
  const mobileGrantedUnitsLabel = quoteSnapshot?.grantedUnitsLabel
    ?? (selectedPresetOption ? formatUnits(selectedPresetOption.granted_units) : t('Preview to confirm units'));
  const hasActiveSelection = Boolean(selection?.amountCents);
  const postOrderHandoffActive = resolvePortalRechargePostOrderHandoffActive({
    lastCreatedOrderId,
    pendingPaymentSpotlight,
  });
  const handoffAmountLabel = pendingPaymentSpotlight?.latestOrder.payable_price_label ?? selectedAmountLabel;
  const primaryActionState = buildPortalRechargePrimaryActionState({
    postOrderHandoffActive,
    quoteLoading,
    createLoading,
    hasSelection: hasActiveSelection,
    t,
  });
  const mobileActionState = buildPortalRechargeMobileActionState({
    postOrderHandoffActive,
    selectedAmountLabel,
    grantedUnitsLabel: mobileGrantedUnitsLabel,
    quoteLoading,
    createLoading,
    hasSelection: hasActiveSelection,
    t,
  });
  const flowTrackerState = buildPortalRechargeFlowTrackerState({
    hasSelection: hasActiveSelection,
    hasQuote: Boolean(quoteSnapshot),
    postOrderHandoffActive,
    pendingPaymentCount: pendingPaymentOrders.length,
    t,
  });

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
      setLastCreatedOrderId(null);
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
      const createdOrder = await createPortalRechargeOrder({
        amount_cents: selection.amountCents,
      });
      setLastCreatedOrderId(createdOrder.order_id);
      await refreshRechargePage({ preserveSelection: true });
      setQuoteStatus(t('Order created. Complete payment in billing.'));
      setPage(1);
    } catch (error) {
      setQuoteStatus(portalErrorMessage(error));
    } finally {
      setCreateLoading(false);
    }
  }

  function handleResumePurchaseFlow() {
    setLastCreatedOrderId(null);
    setQuoteStatus(t('Ready to create order.'));
  }

  function handleCreateRechargeOrderClick() {
    void handleCreateRechargeOrder();
  }

  return (
    <div className="space-y-6 pb-28 xl:pb-0" data-slot="portal-recharge-page">
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
              className="grid gap-3 lg:grid-cols-3"
              data-slot="portal-recharge-guidance-band"
            >
              <div className="rounded-[24px] border border-primary-200/60 bg-white/92 px-4 py-4 shadow-[0_16px_40px_rgba(15,23,42,0.05)] dark:border-primary-900/30 dark:bg-primary-950/18">
                <span className="text-[11px] font-semibold uppercase tracking-[0.18em] text-primary-700 dark:text-primary-300">
                  {t('Step 1')}
                </span>
                <strong className="mt-2 block text-base font-semibold text-primary-950 dark:text-primary-50">
                  {t('Select package')}
                </strong>
                <p className="mt-2 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                  {t('Start with the package that matches your current funding posture or switch to a precise custom amount.')}
                </p>
              </div>
              <div className="rounded-[24px] border border-primary-200/60 bg-primary-50/82 px-4 py-4 shadow-[0_16px_40px_rgba(15,23,42,0.05)] dark:border-primary-900/30 dark:bg-primary-950/20">
                <span className="text-[11px] font-semibold uppercase tracking-[0.18em] text-primary-700 dark:text-primary-300">
                  {t('Step 2')}
                </span>
                <strong className="mt-2 block text-base font-semibold text-primary-950 dark:text-primary-50">
                  {t('Checkout summary')}
                </strong>
                <p className="mt-2 text-sm leading-6 text-primary-700 dark:text-primary-300">
                  {t('Confirm granted units, projected balance, and pricing rule before you create the order.')}
                </p>
              </div>
              <div className="rounded-[24px] border border-primary-200/60 bg-white/92 px-4 py-4 shadow-[0_16px_40px_rgba(15,23,42,0.05)] dark:border-primary-900/30 dark:bg-primary-950/18">
                <span className="text-[11px] font-semibold uppercase tracking-[0.18em] text-primary-700 dark:text-primary-300">
                  {t('Step 3')}
                </span>
                <strong className="mt-2 block text-base font-semibold text-primary-950 dark:text-primary-50">
                  {t('Create order in billing')}
                </strong>
                <p className="mt-2 text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                  {t('The order is created here, then billing handles the settlement follow-up without losing context.')}
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
                const merchandising = optionMerchandisingById.get(option.id)
                  ?? buildPortalRechargeOptionMerchandising({
                    option,
                    options: sortedOptions,
                    t,
                  });

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
                        setLastCreatedOrderId(null);
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
                            <Badge variant={isActive ? 'secondary' : 'warning'}>
                              {merchandising.badge}
                            </Badge>
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
                          <p className={`text-sm font-medium leading-6 ${
                            isActive ? 'text-white' : 'text-primary-800 dark:text-primary-100'
                          }`}
                          >
                            {merchandising.intentLabel}
                          </p>
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
                        {resolveOptionSupportText({
                          isActive,
                          merchandising,
                          t,
                        })}
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
                        setLastCreatedOrderId(null);
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

            <div
              className="grid gap-3 rounded-[30px] border border-primary-200/60 bg-[linear-gradient(180deg,rgba(255,255,255,0.98),rgba(240,246,255,0.94))] p-5 shadow-[0_18px_48px_rgba(15,23,42,0.08)] dark:border-primary-900/30 dark:bg-[linear-gradient(180deg,rgba(10,16,28,0.96),rgba(8,12,22,0.94))] lg:grid-cols-[minmax(0,1fr)_auto]"
              data-slot="portal-recharge-selection-story"
            >
              <div className="space-y-2">
                <span className="text-[11px] font-semibold uppercase tracking-[0.18em] text-primary-700 dark:text-primary-300">
                  {t('Selection story')}
                </span>
                <h3 className="text-xl font-semibold tracking-tight text-primary-950 dark:text-primary-50">
                  {selectionStory?.intentLabel ?? t('Best fit for steady usage')}
                </h3>
                <p className="max-w-[42rem] text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                  {selectionStory?.supportLabel ?? t('Choose a recharge path to see the live quote and settlement story in one place.')}
                </p>
              </div>
              <div className="rounded-[24px] border border-primary-200/60 bg-primary-50/78 px-4 py-4 dark:border-primary-900/28 dark:bg-primary-950/22">
                <span className="text-[11px] font-semibold uppercase tracking-[0.18em] text-primary-700 dark:text-primary-300">
                  {selection?.mode === 'custom' ? t('Custom operator path') : selectionStory?.badge ?? t('Recommended default')}
                </span>
                <strong className="mt-2 block text-2xl font-semibold tracking-tight text-primary-950 dark:text-primary-50">
                  {selectedAmountLabel}
                </strong>
                <p className="mt-2 text-sm leading-6 text-primary-700 dark:text-primary-300">
                  {t('The selected amount is already wired into the live quote and ready for order creation.')}
                </p>
              </div>
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
                <Badge variant="secondary">{t('Checkout summary')}</Badge>
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

            <PortalRechargeFlowTracker flowState={flowTrackerState} t={t} />

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

                <div
                  className="space-y-4 rounded-[26px] border border-primary-200/60 bg-white/90 p-4 shadow-[0_18px_40px_rgba(15,23,42,0.06)] dark:border-primary-900/30 dark:bg-primary-950/18"
                  data-slot="portal-recharge-quote-breakdown"
                >
                  <div className="flex items-start justify-between gap-3">
                    <div className="space-y-1">
                      <span className="text-[11px] font-semibold uppercase tracking-[0.18em] text-primary-700 dark:text-primary-300">
                        {t('Checkout summary')}
                      </span>
                      <p className="text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                        {t('Review the commercial outcome before you create the order and hand off settlement to billing.')}
                      </p>
                    </div>
                    <Badge variant="secondary">
                      {selection?.mode === 'custom' ? t('Operator choice') : selectionStory?.badge ?? t('Recommended default')}
                    </Badge>
                  </div>

                  <div className="space-y-3">
                    <div className="flex items-center justify-between gap-3 rounded-[20px] border border-primary-200/55 bg-primary-50/78 px-4 py-3 dark:border-primary-900/28 dark:bg-primary-950/22">
                      <span className="text-sm font-medium text-primary-700 dark:text-primary-300">
                        {t('Current balance')}
                      </span>
                      <strong className="text-sm font-semibold text-primary-950 dark:text-primary-50">
                        {currentBalanceLabel}
                      </strong>
                    </div>
                    <div className="flex items-center justify-between gap-3 rounded-[20px] border border-primary-200/55 bg-white/86 px-4 py-3 dark:border-primary-900/28 dark:bg-primary-950/12">
                      <span className="text-sm font-medium text-zinc-600 dark:text-zinc-300">
                        {t('Granted units')}
                      </span>
                      <strong className="text-sm font-semibold text-primary-950 dark:text-primary-50">
                        {quoteSnapshot.grantedUnitsLabel}
                      </strong>
                    </div>
                    <div className="flex items-center justify-between gap-3 rounded-[20px] border border-primary-200/55 bg-white/86 px-4 py-3 dark:border-primary-900/28 dark:bg-primary-950/12">
                      <span className="text-sm font-medium text-zinc-600 dark:text-zinc-300">
                        {t('Effective ratio')}
                      </span>
                      <strong className="text-sm font-semibold text-primary-950 dark:text-primary-50">
                        {quoteSnapshot.effectiveRatioLabel}
                      </strong>
                    </div>
                    <div className="flex items-center justify-between gap-3 rounded-[20px] border border-primary-200/55 bg-white/86 px-4 py-3 dark:border-primary-900/28 dark:bg-primary-950/12">
                      <span className="text-sm font-medium text-zinc-600 dark:text-zinc-300">
                        {t('Projected balance')}
                      </span>
                      <strong className="text-sm font-semibold text-primary-950 dark:text-primary-50">
                        {quoteSnapshot.projectedBalanceLabel}
                      </strong>
                    </div>
                    <div className="flex items-center justify-between gap-3 rounded-[20px] border border-primary-200/55 bg-white/86 px-4 py-3 dark:border-primary-900/28 dark:bg-primary-950/12">
                      <span className="text-sm font-medium text-zinc-600 dark:text-zinc-300">
                        {t('Settlement path')}
                      </span>
                      <strong className="text-sm font-semibold text-primary-950 dark:text-primary-50">
                        {t('Create order in billing')}
                      </strong>
                    </div>
                  </div>
                </div>

                <PortalRechargePrimaryActionButton
                  actionState={primaryActionState}
                  onCreateOrder={handleCreateRechargeOrderClick}
                  onNavigate={onNavigate}
                />

                <PortalRechargePostOrderHandoffPanel
                  active={postOrderHandoffActive}
                  amountLabel={handoffAmountLabel}
                  onNavigate={onNavigate}
                  onReset={handleResumePurchaseFlow}
                  t={t}
                />

                <PortalRechargePendingSettlementCallout
                  onNavigate={onNavigate}
                  pendingPaymentSpotlight={pendingPaymentSpotlight}
                  t={t}
                />
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

      <PortalRechargeMobileActionBar
        actionState={mobileActionState}
        onCreateOrder={handleCreateRechargeOrderClick}
        onNavigate={onNavigate}
        visible={hasActiveSelection}
      />

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
