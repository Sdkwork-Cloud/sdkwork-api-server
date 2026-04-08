import { startTransition, useEffect, useMemo, useState } from 'react';
import type { ChangeEvent, FormEvent } from 'react';

import {
  formatCurrency,
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
  BillingAccountingMode,
  BillingEventAccountingModeSummary,
  BillingEventCapabilitySummary,
  BillingEventSummary,
  PortalCommerceMembership,
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
  buildPortalRechargeFinanceProjection,
  buildPortalRechargeQuoteSnapshot,
  buildPortalRechargeSummaryCards,
  formatRechargeAmountInput,
  parseRechargeAmountInput,
  validatePortalRechargeAmount,
} from '../services';
import type {
  PortalRechargeFinanceProjection,
  PortalRechargePageProps,
  PortalRechargeSelection,
} from '../types';

const PAGE_SIZE = 8;

function titleCaseToken(value: string): string {
  return value
    .split(/[-_\s]+/g)
    .filter(Boolean)
    .map((segment) =>
      segment.length <= 3
        ? segment.toUpperCase()
        : `${segment.slice(0, 1).toUpperCase()}${segment.slice(1)}`,
    )
    .join(' ');
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

function resolveMembershipStatusLabel(
  status: string | null | undefined,
  t: ReturnType<typeof usePortalI18n>['t'],
) {
  const normalized = status?.trim().toLowerCase() ?? '';

  switch (normalized) {
    case 'active':
      return t('Active');
    case 'inactive':
      return t('Inactive');
    case 'canceled':
    case 'cancelled':
      return t('Canceled');
    case 'past_due':
    case 'past-due':
      return t('Past due');
    case 'grace_period':
    case 'grace-period':
      return t('Grace period');
    case 'paused':
      return t('Paused');
    default:
      return status?.trim() ? titleCaseToken(status) : t('Inactive');
  }
}

function accountingModeLabel(
  mode: BillingAccountingMode | null | undefined,
  t: ReturnType<typeof usePortalI18n>['t'],
) {
  switch (mode) {
    case 'platform_credit':
      return t('Platform credit');
    case 'byok':
      return t('BYOK');
    case 'passthrough':
      return t('Passthrough');
    default:
      return t('Accounting mode');
  }
}

function capabilityLabel(
  capability: string | null | undefined,
  t: ReturnType<typeof usePortalI18n>['t'],
) {
  switch (capability?.trim().toLowerCase()) {
    case 'responses':
      return t('Responses');
    case 'images':
      return t('Images');
    case 'audio':
      return t('Audio');
    case 'video':
      return t('Video');
    case 'music':
      return t('Music');
    default:
      return capability?.trim() ? titleCaseToken(capability) : t('Capability');
  }
}

function accountingModeDetail(
  summary: BillingEventAccountingModeSummary | null,
  t: ReturnType<typeof usePortalI18n>['t'],
) {
  if (!summary) {
    return t('Billing event evidence will appear here after routed traffic starts recording chargeback activity.');
  }

  return t('{requests} requests / {events} events', {
    requests: formatUnits(summary.request_count),
    events: formatUnits(summary.event_count),
  });
}

function capabilityDetail(
  summary: BillingEventCapabilitySummary | null,
  t: ReturnType<typeof usePortalI18n>['t'],
) {
  if (!summary) {
    return t('Billing event evidence will appear here after routed traffic starts recording chargeback activity.');
  }

  const facts: string[] = [];

  if (summary.total_tokens > 0) {
    facts.push(t('{count} tokens', { count: formatUnits(summary.total_tokens) }));
  }
  if (summary.image_count > 0) {
    facts.push(t('{count} images', { count: formatUnits(summary.image_count) }));
  }
  if (summary.audio_seconds > 0) {
    facts.push(t('{count} audio sec', { count: formatUnits(summary.audio_seconds) }));
  }
  if (summary.video_seconds > 0) {
    facts.push(t('{count} video sec', { count: formatUnits(summary.video_seconds) }));
  }
  if (summary.music_seconds > 0) {
    facts.push(t('{count} music sec', { count: formatUnits(summary.music_seconds) }));
  }

  facts.push(t('{count} requests', { count: formatUnits(summary.request_count) }));

  return facts.join(' / ');
}

function membershipDetail(
  membership: PortalCommerceMembership | null,
  t: ReturnType<typeof usePortalI18n>['t'],
) {
  if (!membership) {
    return t('No active membership is recorded yet. Complete a subscription checkout to activate monthly entitlement posture.');
  }

  return t('{planName} is the active workspace membership and defines the current subscription entitlement baseline.', {
    planName: membership.plan_name,
  });
}

function buildFinanceProjection(
  membership: PortalCommerceMembership | null,
  billingEventSummary: BillingEventSummary | null,
): PortalRechargeFinanceProjection {
  return buildPortalRechargeFinanceProjection({
    membership,
    billingEventSummary,
  });
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
  const syncedStatus = t('Recharge options and history are synced with the current workspace balance.');
  const defaultQuoteStatus = t('Select a server-managed recharge amount to preview the top-up quote.');
  const [summary, setSummary] = useState<ProjectBillingSummary | null>(null);
  const [membership, setMembership] = useState<PortalCommerceMembership | null>(null);
  const [billingEventSummary, setBillingEventSummary] = useState<BillingEventSummary | null>(null);
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
      setMembership(data.membership);
      setBillingEventSummary(data.billing_event_summary);
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
      setQuoteStatus(t('Loading recharge quote...'));

      try {
        const nextQuote = await previewPortalRechargeQuote({
          amount_cents: amountCents,
          current_remaining_units: remainingUnits,
        });

        if (cancelled) {
          return;
        }

        setQuote(nextQuote);
        setQuoteStatus(t('Recharge quote is ready and priced by the server-managed recharge policy.'));
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
  const summaryCards = useMemo(
    () =>
      buildPortalRechargeSummaryCards({
        quote,
        rechargeOptions: options,
        summary,
        orders,
        t,
      }),
    [options, orders, quote, summary, t],
  );
  const financeProjection = useMemo(
    () => buildFinanceProjection(membership, billingEventSummary),
    [billingEventSummary, membership],
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
      setQuoteStatus(t('Recharge order created. Open billing when you are ready to settle it.'));
      setPage(1);
    } catch (error) {
      setQuoteStatus(portalErrorMessage(error));
    } finally {
      setCreateLoading(false);
    }
  }

  return (
    <div className="space-y-4" data-slot="portal-recharge-page">
      <div
        className="grid gap-4 md:grid-cols-2 xl:grid-cols-3"
        data-slot="portal-recharge-summary-grid"
      >
        {summaryCards.map((item) => (
          <Card
            className="border-zinc-200 bg-white shadow-none dark:border-zinc-800 dark:bg-zinc-950"
            key={item.label}
          >
            <CardContent className="space-y-3 p-5">
              <p className="text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                {item.label}
              </p>
              <strong className="block text-3xl font-semibold tracking-tight text-zinc-950 dark:text-zinc-50">
                {item.value}
              </strong>
              <p className="text-sm leading-6 text-zinc-500 dark:text-zinc-400">{item.detail}</p>
            </CardContent>
          </Card>
        ))}
      </div>

      <Card
        className="border-zinc-200 bg-white shadow-none dark:border-zinc-800 dark:bg-zinc-950"
        data-slot="portal-recharge-decision-support"
      >
        <CardContent className="space-y-4 p-5">
          <div className="space-y-1">
            <h2 className="text-lg font-semibold text-zinc-950 dark:text-zinc-50">
              {t('Recharge decision support')}
            </h2>
            <p className="text-sm text-zinc-500 dark:text-zinc-400">
              {t('Recharge posture combines membership baseline, accounting mode mix, and workload shape before you create a top-up order.')}
            </p>
          </div>

          <div className="grid gap-4 xl:grid-cols-3">
            <div className="rounded-3xl border border-zinc-200 bg-zinc-50/80 p-4 dark:border-zinc-800 dark:bg-zinc-900/60">
              <span className="text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                {t('Active membership')}
              </span>
              <strong className="mt-2 block text-xl font-semibold text-zinc-950 dark:text-zinc-50">
                {financeProjection.membership?.plan_name ?? t('No active membership')}
              </strong>
              <p className="mt-2 text-sm leading-6 text-zinc-500 dark:text-zinc-400">
                {membershipDetail(financeProjection.membership, t)}
              </p>
              <div className="mt-4 grid gap-3 sm:grid-cols-2">
                <div className="rounded-2xl border border-zinc-200 bg-white px-3 py-3 dark:border-zinc-800 dark:bg-zinc-950">
                  <span className="text-[11px] font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                    {t('Status')}
                  </span>
                  <strong className="mt-2 block text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                    {resolveMembershipStatusLabel(financeProjection.membership?.status, t)}
                  </strong>
                </div>
                <div className="rounded-2xl border border-zinc-200 bg-white px-3 py-3 dark:border-zinc-800 dark:bg-zinc-950">
                  <span className="text-[11px] font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                    {t('Included units')}
                  </span>
                  <strong className="mt-2 block text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                    {financeProjection.membership
                      ? formatUnits(financeProjection.membership.included_units)
                      : t('n/a')}
                  </strong>
                </div>
              </div>
            </div>

            <div className="rounded-3xl border border-zinc-200 bg-zinc-50/80 p-4 dark:border-zinc-800 dark:bg-zinc-900/60">
              <span className="text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                {t('Leading accounting mode')}
              </span>
              <strong className="mt-2 block text-xl font-semibold text-zinc-950 dark:text-zinc-50">
                {accountingModeLabel(financeProjection.leading_accounting_mode?.accounting_mode, t)}
              </strong>
              <p className="mt-2 text-sm leading-6 text-zinc-500 dark:text-zinc-400">
                {accountingModeDetail(financeProjection.leading_accounting_mode, t)}
              </p>
              <div className="mt-4 rounded-2xl border border-zinc-200 bg-white px-3 py-3 dark:border-zinc-800 dark:bg-zinc-950">
                <span className="text-[11px] font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                  {t('Customer charge')}
                </span>
                <strong className="mt-2 block text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                  {financeProjection.leading_accounting_mode
                    ? formatCurrency(financeProjection.leading_accounting_mode.total_customer_charge)
                    : t('n/a')}
                </strong>
              </div>
            </div>

            <div className="rounded-3xl border border-zinc-200 bg-zinc-50/80 p-4 dark:border-zinc-800 dark:bg-zinc-900/60">
              <span className="text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                {t('Leading capability')}
              </span>
              <strong className="mt-2 block text-xl font-semibold text-zinc-950 dark:text-zinc-50">
                {capabilityLabel(financeProjection.leading_capability?.capability, t)}
              </strong>
              <p className="mt-2 text-sm leading-6 text-zinc-500 dark:text-zinc-400">
                {capabilityDetail(financeProjection.leading_capability, t)}
              </p>
              <div className="mt-4 rounded-2xl border border-zinc-200 bg-white px-3 py-3 dark:border-zinc-800 dark:bg-zinc-950">
                <span className="text-[11px] font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                  {t('Customer charge')}
                </span>
                <strong className="mt-2 block text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                  {financeProjection.leading_capability
                    ? formatCurrency(financeProjection.leading_capability.total_customer_charge)
                    : t('n/a')}
                </strong>
              </div>
            </div>
          </div>

          <div
            className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4"
            data-slot="portal-recharge-multimodal-demand"
          >
            {[
              { label: t('Images'), value: formatUnits(financeProjection.multimodal_totals.image_count) },
              { label: t('Audio'), value: formatUnits(financeProjection.multimodal_totals.audio_seconds) },
              { label: t('Video'), value: formatUnits(financeProjection.multimodal_totals.video_seconds) },
              { label: t('Music'), value: formatUnits(financeProjection.multimodal_totals.music_seconds) },
            ].map((item) => (
              <div
                className="rounded-2xl border border-zinc-200 bg-zinc-50/80 px-4 py-4 dark:border-zinc-800 dark:bg-zinc-900/60"
                key={item.label}
              >
                <span className="text-[11px] font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                  {item.label}
                </span>
                <strong className="mt-2 block text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                  {item.value}
                </strong>
              </div>
            ))}
          </div>

          <p className="text-sm text-zinc-500 dark:text-zinc-400">
            {t('Multimodal demand keeps image, audio, video, and music traffic visible before a top-up is created.')}
          </p>
        </CardContent>
      </Card>

      {pageStatus ? (
        <div
          className="rounded-2xl border border-zinc-200 bg-zinc-50/80 px-4 py-3 text-sm text-zinc-600 dark:border-zinc-800 dark:bg-zinc-900/50 dark:text-zinc-300"
          role="status"
        >
          {pageStatus}
        </div>
      ) : null}

      <div className="grid gap-4 xl:grid-cols-[1.24fr_0.76fr]">
        <Card
          className="border-zinc-200 bg-white shadow-none dark:border-zinc-800 dark:bg-zinc-950"
          data-slot="portal-recharge-options"
        >
          <CardContent className="space-y-5 p-5">
            <div className="space-y-1">
              <h2 className="text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                {t('Recharge options')}
              </h2>
              <p className="text-sm text-zinc-500 dark:text-zinc-400">
                {t('Choose a server-managed top-up amount or enter a custom recharge amount below.')}
              </p>
            </div>

            <div className="grid gap-3 md:grid-cols-2">
              {options.map((option) => {
                const isActive =
                  selection?.amountCents === option.amount_cents && selection.mode === 'preset';

                return (
                  <button
                    className={`rounded-3xl border p-4 text-left transition-colors ${
                      isActive
                        ? 'border-zinc-950 bg-zinc-950 text-white dark:border-zinc-50 dark:bg-zinc-50 dark:text-zinc-950'
                        : 'border-zinc-200 bg-zinc-50/70 text-zinc-950 hover:border-zinc-300 dark:border-zinc-800 dark:bg-zinc-900/70 dark:text-zinc-50'
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
                    <div className="flex items-start justify-between gap-3">
                      <div className="space-y-2">
                        <div className="flex flex-wrap items-center gap-2">
                          <strong className="text-base font-semibold">{option.label}</strong>
                          {option.recommended ? (
                            <Badge variant={isActive ? 'secondary' : 'success'}>
                              {t('Recommended')}
                            </Badge>
                          ) : null}
                        </div>
                        <p className={`text-sm ${isActive ? 'text-zinc-300 dark:text-zinc-700' : 'text-zinc-500 dark:text-zinc-400'}`}>
                          {option.note}
                        </p>
                      </div>
                      <strong className="text-xl font-semibold">{option.amount_label}</strong>
                    </div>
                    <div className="mt-4 grid gap-2 sm:grid-cols-2">
                      <div className={`rounded-2xl border px-3 py-3 ${isActive ? 'border-white/15 bg-white/10 dark:border-zinc-900/10 dark:bg-zinc-900/10' : 'border-zinc-200 bg-white dark:border-zinc-800 dark:bg-zinc-950'}`}>
                        <span className={`text-[11px] font-semibold uppercase tracking-[0.16em] ${isActive ? 'text-zinc-300 dark:text-zinc-700' : 'text-zinc-500 dark:text-zinc-400'}`}>
                          {t('Granted units')}
                        </span>
                        <strong className="mt-2 block text-sm font-semibold">
                          {formatUnits(option.granted_units)}
                        </strong>
                      </div>
                      <div className={`rounded-2xl border px-3 py-3 ${isActive ? 'border-white/15 bg-white/10 dark:border-zinc-900/10 dark:bg-zinc-900/10' : 'border-zinc-200 bg-white dark:border-zinc-800 dark:bg-zinc-950'}`}>
                        <span className={`text-[11px] font-semibold uppercase tracking-[0.16em] ${isActive ? 'text-zinc-300 dark:text-zinc-700' : 'text-zinc-500 dark:text-zinc-400'}`}>
                          {t('Effective ratio')}
                        </span>
                        <strong className="mt-2 block text-sm font-semibold">
                          {option.effective_ratio_label}
                        </strong>
                      </div>
                    </div>
                  </button>
                );
              })}
            </div>

            <form
              className="space-y-4 rounded-3xl border border-dashed border-zinc-300 bg-zinc-50/60 p-4 dark:border-zinc-700 dark:bg-zinc-900/50"
              data-slot="portal-recharge-custom-form"
              onSubmit={(event) => void handleCustomPreview(event)}
            >
              <div className="space-y-1">
                <h3 className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                  {t('Custom amount')}
                </h3>
                <p className="text-sm text-zinc-500 dark:text-zinc-400">
                  {t('Custom recharge follows the server-managed ratio policy and validation step.')}
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
                <Button type="submit" variant="secondary">
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
          className="border-zinc-200 bg-white shadow-none dark:border-zinc-800 dark:bg-zinc-950"
          data-slot="portal-recharge-quote-card"
        >
          <CardContent className="space-y-5 p-5">
            <div className="space-y-1">
              <h2 className="text-lg font-semibold text-zinc-950 dark:text-zinc-50">
                {t('Create recharge order')}
              </h2>
              <p className="text-sm text-zinc-500 dark:text-zinc-400">{quoteStatus}</p>
            </div>

            {quoteSnapshot ? (
              <div className="space-y-4">
                <div className="rounded-3xl border border-zinc-200 bg-zinc-50/80 p-4 dark:border-zinc-800 dark:bg-zinc-900/60">
                  <div className="flex items-center justify-between gap-3">
                    <div>
                      <span className="text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                        {t('Recharge amount')}
                      </span>
                      <strong className="mt-2 block text-3xl font-semibold tracking-tight text-zinc-950 dark:text-zinc-50">
                        {quoteSnapshot.amountLabel}
                      </strong>
                    </div>
                    <Badge variant="secondary">{quoteSnapshot.pricingRuleLabel}</Badge>
                  </div>
                </div>

                <div className="grid gap-3">
                  <div className="flex items-center justify-between gap-3 rounded-2xl border border-zinc-200 bg-white px-4 py-3 dark:border-zinc-800 dark:bg-zinc-950">
                    <span className="text-sm text-zinc-500 dark:text-zinc-400">{t('Granted units')}</span>
                    <strong className="text-zinc-950 dark:text-zinc-50">{quoteSnapshot.grantedUnitsLabel}</strong>
                  </div>
                  <div className="flex items-center justify-between gap-3 rounded-2xl border border-zinc-200 bg-white px-4 py-3 dark:border-zinc-800 dark:bg-zinc-950">
                    <span className="text-sm text-zinc-500 dark:text-zinc-400">{t('Effective ratio')}</span>
                    <strong className="text-zinc-950 dark:text-zinc-50">{quoteSnapshot.effectiveRatioLabel}</strong>
                  </div>
                  <div className="flex items-center justify-between gap-3 rounded-2xl border border-zinc-200 bg-white px-4 py-3 dark:border-zinc-800 dark:bg-zinc-950">
                    <span className="text-sm text-zinc-500 dark:text-zinc-400">{t('Projected balance')}</span>
                    <strong className="text-zinc-950 dark:text-zinc-50">{quoteSnapshot.projectedBalanceLabel}</strong>
                  </div>
                  <div className="flex items-center justify-between gap-3 rounded-2xl border border-zinc-200 bg-white px-4 py-3 dark:border-zinc-800 dark:bg-zinc-950">
                    <span className="text-sm text-zinc-500 dark:text-zinc-400">{t('Current balance')}</span>
                    <strong className="text-zinc-950 dark:text-zinc-50">{formatBalance(summary, t)}</strong>
                  </div>
                </div>

                <Button
                  className="h-11 w-full"
                  disabled={quoteLoading || createLoading || !selection?.amountCents}
                  onClick={() => void handleCreateRechargeOrder()}
                >
                  {createLoading ? t('Creating...') : t('Create recharge order')}
                </Button>
              </div>
            ) : (
              <EmptyState
                description={t('Select a preset amount or preview a custom amount to inspect the server quote.')}
                title={quoteLoading ? t('Loading quote') : t('No quote selected')}
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
                {t('Recharge history keeps both preset top-ups and custom recharge orders on one table.')}
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
