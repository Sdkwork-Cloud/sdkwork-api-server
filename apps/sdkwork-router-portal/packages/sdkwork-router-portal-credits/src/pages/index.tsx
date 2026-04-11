import { startTransition, useEffect, useMemo, useState } from 'react';
import type { ChangeEvent, FormEvent } from 'react';

import {
  copyText,
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
import {
  Input,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from 'sdkwork-router-portal-commons/framework/entry';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from 'sdkwork-router-portal-commons/framework/layout';
import { portalErrorMessage } from 'sdkwork-router-portal-portal-api';
import type {
  BillingAccountingMode,
  BillingEventAccountingModeSummary,
  BillingEventCapabilitySummary,
  BillingEventSummary,
  CouponCodeStatus,
  CouponRedemptionStatus,
  CouponReservationStatus,
  PortalCommerceOrder,
  PortalCommerceOrderStatus,
  PortalMarketingCodeItem,
  PortalMarketingCodesResponse,
  PortalMarketingRedemptionsResponse,
  PortalMarketingRewardHistoryItem,
  ProjectBillingSummary,
} from 'sdkwork-router-portal-types';

import {
  loadCreditsPageData,
  redeemCreditsCouponCode,
  validateCreditsCouponCode,
} from '../repository';
import {
  buildPortalCouponSelfServiceDecision,
  buildPortalCreditsFinanceProjection,
  buildRedeemInviteProgram,
} from '../services';
import type {
  PortalCreditsFinanceProjection,
  PortalCreditsPageProps,
} from '../types';

const PAGE_SIZE = 8;

type TranslateFn = (text: string, values?: Record<string, string | number>) => string;

function clampPage(page: number, totalPages: number): number {
  return Math.min(Math.max(page, 1), Math.max(totalPages, 1));
}

function orderStatusLabel(status: PortalCommerceOrderStatus, t: TranslateFn): string {
  switch (status) {
    case 'pending_payment':
      return t('Payment pending');
    case 'fulfilled':
      return t('Fulfilled');
    case 'canceled':
      return t('Canceled');
    case 'failed':
      return t('Failed');
    default:
      return t('Status');
  }
}

function orderStatusVariant(status: PortalCommerceOrderStatus): 'secondary' | 'success' | 'warning' {
  switch (status) {
    case 'fulfilled':
      return 'success';
    case 'pending_payment':
      return 'warning';
    default:
      return 'secondary';
  }
}

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

function accountingModeLabel(
  mode: BillingAccountingMode | null | undefined,
  t: TranslateFn,
): string {
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
  t: TranslateFn,
): string {
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
  t: TranslateFn,
): string {
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
  t: TranslateFn,
): string {
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

function resolveRedeemCode(order: PortalCommerceOrder): string {
  return order.applied_coupon_code?.trim() || order.target_id;
}

function resolveRedeemOfferLabel(order: PortalCommerceOrder, t: TranslateFn): string {
  return order.target_name?.trim() || t('Coupon');
}

function formatBalanceValue(
  summary: ProjectBillingSummary | null,
  t: TranslateFn,
): string {
  if (!summary) {
    return t('Loading...');
  }

  return summary.remaining_units === null
    ? t('Unlimited')
    : formatUnits(summary.remaining_units ?? 0);
}

function buildFinanceProjection(
  summary: ProjectBillingSummary | null,
  orders: PortalCommerceOrder[],
  billingEventSummary: BillingEventSummary | null,
): PortalCreditsFinanceProjection | null {
  if (!summary) {
    return null;
  }

  return buildPortalCreditsFinanceProjection({
    summary,
    orders,
    billingEventSummary,
  });
}

function emptyMarketingCodesResponse(): PortalMarketingCodesResponse {
  return {
    summary: {
      total_count: 0,
      available_count: 0,
      reserved_count: 0,
      redeemed_count: 0,
      disabled_count: 0,
      expired_count: 0,
    },
    items: [],
  };
}

function emptyMarketingRedemptionsResponse(): PortalMarketingRedemptionsResponse {
  return {
    summary: {
      total_count: 0,
      redeemed_count: 0,
      partially_rolled_back_count: 0,
      rolled_back_count: 0,
      failed_count: 0,
    },
    items: [],
  };
}

function couponCodeStatusLabel(status: CouponCodeStatus, t: TranslateFn): string {
  switch (status) {
    case 'available':
      return t('Available');
    case 'reserved':
      return t('Reserved');
    case 'redeemed':
      return t('Redeemed');
    case 'expired':
      return t('Expired');
    case 'disabled':
      return t('Disabled');
    default:
      return t('Status');
  }
}

function couponCodeStatusVariant(
  status: CouponCodeStatus,
): 'secondary' | 'success' | 'warning' {
  switch (status) {
    case 'available':
      return 'success';
    case 'reserved':
      return 'warning';
    default:
      return 'secondary';
  }
}

function reservationStatusLabel(
  status: CouponReservationStatus | null | undefined,
  t: TranslateFn,
): string {
  switch (status) {
    case 'reserved':
      return t('Reserved');
    case 'released':
      return t('Released');
    case 'confirmed':
      return t('Confirmed');
    case 'expired':
      return t('Expired');
    default:
      return t('None');
  }
}

function reservationStatusVariant(
  status: CouponReservationStatus | null | undefined,
): 'secondary' | 'success' | 'warning' {
  switch (status) {
    case 'reserved':
      return 'warning';
    case 'confirmed':
      return 'success';
    default:
      return 'secondary';
  }
}

function redemptionStatusLabel(
  status: CouponRedemptionStatus | null | undefined,
  t: TranslateFn,
): string {
  switch (status) {
    case 'pending':
      return t('Pending');
    case 'redeemed':
      return t('Redeemed');
    case 'partially_rolled_back':
      return t('Partially rolled back');
    case 'rolled_back':
      return t('Rolled back');
    case 'failed':
      return t('Failed');
    default:
      return t('None');
  }
}

function redemptionStatusVariant(
  status: CouponRedemptionStatus | null | undefined,
): 'secondary' | 'success' | 'warning' {
  switch (status) {
    case 'redeemed':
      return 'success';
    case 'pending':
    case 'partially_rolled_back':
      return 'warning';
    default:
      return 'secondary';
  }
}

function rewardHistoryRollbackDetail(
  item: PortalMarketingRewardHistoryItem,
  t: TranslateFn,
): string {
  if (!item.rollbacks.length) {
    return t('None');
  }

  const latest = item.rollbacks[item.rollbacks.length - 1];
  return t('{count} rollback(s) / {type}', {
    count: item.rollbacks.length,
    type: titleCaseToken(latest.rollback_type),
  });
}

function rewardHistoryAccountArrivalDetail(
  item: PortalMarketingRewardHistoryItem,
  t: TranslateFn,
) {
  if (item.effect.effect_kind !== 'account_entitlement') {
    return (
      <span className="text-xs leading-5 text-zinc-500 dark:text-zinc-400">
        {t('No account arrival for checkout discount')}
      </span>
    );
  }

  if (item.account_arrival.benefit_lot_count === 0) {
    return (
      <span className="text-xs leading-5 text-zinc-500 dark:text-zinc-400">
        {t('No linked account lot evidence yet')}
      </span>
    );
  }

  const primaryLot = item.account_arrival.benefit_lots[0];
  const facts = [
    t('{count} units', { count: formatUnits(item.account_arrival.credited_quantity) }),
    t('{count} lot(s)', { count: item.account_arrival.benefit_lot_count }),
  ];
  const lineage = [];

  if (item.account_arrival.account_id !== null && item.account_arrival.account_id !== undefined) {
    lineage.push(t('Account #{id}', { id: item.account_arrival.account_id }));
  }
  lineage.push(t('Lot #{id}', { id: primaryLot.lot_id }));

  return (
    <div className="space-y-1">
      <strong>{t('Arrived to account')}</strong>
      <p className="text-xs leading-5 text-zinc-500 dark:text-zinc-400">{facts.join(' / ')}</p>
      <p className="text-xs leading-5 text-zinc-500 dark:text-zinc-400">{lineage.join(' / ')}</p>
    </div>
  );
}

function couponEffectLabel(
  item: Pick<PortalMarketingCodeItem, 'effect'> | Pick<PortalMarketingRewardHistoryItem, 'effect'>,
  t: TranslateFn,
): string {
  if (item.effect.effect_kind === 'account_entitlement') {
    return item.effect.grant_units
      ? t('Grant {count} units', { count: formatUnits(item.effect.grant_units) })
      : t('Account entitlement');
  }

  if (item.effect.discount_percent !== null && item.effect.discount_percent !== undefined) {
    return t('{percent}% off', { percent: item.effect.discount_percent });
  }

  if (
    item.effect.discount_amount_minor !== null
    && item.effect.discount_amount_minor !== undefined
  ) {
    return t('Fixed discount');
  }

  return t('Checkout discount');
}

function couponApplicabilityLabel(
  item:
    | Pick<PortalMarketingCodeItem, 'applicability'>
    | Pick<PortalMarketingRewardHistoryItem, 'applicability'>,
  t: TranslateFn,
): string {
  if (item.applicability.all_target_kinds_eligible) {
    return t('All market targets');
  }

  if (!item.applicability.target_kinds.length) {
    return t('Target scope pending');
  }

  return item.applicability.target_kinds.map((target) => titleCaseToken(target)).join(', ');
}

function redemptionCoverageHeadline(
  projection: PortalCreditsFinanceProjection | null,
  t: TranslateFn,
): string {
  if (!projection) {
    return t('Loading...');
  }

  return projection.redemption_coverage.next_funding_path === 'recharge'
    ? t('Recharge')
    : t('Redeem');
}

function redemptionCoverageDetail(
  projection: PortalCreditsFinanceProjection | null,
  t: TranslateFn,
): string {
  if (!projection) {
    return t('Billing event evidence will appear here after routed traffic starts recording chargeback activity.');
  }

  if (projection.redemption_coverage.fulfilled_redemptions === 0) {
    return t('No fulfilled coupon redemptions are recorded yet. Use the first coupon to establish a launch buffer before recharge is needed.');
  }

  const redeemedUnits = formatUnits(
    projection.redemption_coverage.granted_units + projection.redemption_coverage.bonus_units,
  );
  const redeemCount = formatUnits(projection.redemption_coverage.fulfilled_redemptions);

  return projection.redemption_coverage.next_funding_path === 'recharge'
    ? t(
      'Coupon orders have already granted {units} units across {count} fulfilled redemptions, and current demand is now better served by recharge or membership.',
      {
        units: redeemedUnits,
        count: redeemCount,
      },
    )
    : t(
      'Coupon orders have already granted {units} units across {count} fulfilled redemptions, and redeem remains a viable low-friction top-up path.',
      {
        units: redeemedUnits,
        count: redeemCount,
      },
    );
}

export function PortalCreditsPage({ workspace }: PortalCreditsPageProps) {
  const { t } = usePortalI18n();
  const loadingStatus = t('Loading redeem workspace...');
  const syncedStatus = t('Redeem history is synced with the latest workspace posture.');
  const defaultRedeemStatus = t(
    'Redeem one grant-style coupon code for the current workspace. Apply during checkout for discount-only promotions.',
  );
  const defaultInviteStatus = t('Copy the workspace invite code or direct link to share rewards.');
  const [summary, setSummary] = useState<ProjectBillingSummary | null>(null);
  const [billingEventSummary, setBillingEventSummary] = useState<BillingEventSummary | null>(null);
  const [orders, setOrders] = useState<PortalCommerceOrder[]>([]);
  const [marketingCodes, setMarketingCodes] = useState<PortalMarketingCodesResponse>(
    emptyMarketingCodesResponse,
  );
  const [marketingRedemptions, setMarketingRedemptions] =
    useState<PortalMarketingRedemptionsResponse>(emptyMarketingRedemptionsResponse);
  const [rewardHistory, setRewardHistory] = useState<PortalMarketingRewardHistoryItem[]>([]);
  const [status, setStatus] = useState(loadingStatus);
  const [redeemStatus, setRedeemStatus] = useState(defaultRedeemStatus);
  const [inviteStatus, setInviteStatus] = useState(defaultInviteStatus);
  const [couponCode, setCouponCode] = useState('');
  const [redeemLoading, setRedeemLoading] = useState(false);
  const [page, setPage] = useState(1);
  const inviteProgram = useMemo(() => buildRedeemInviteProgram(workspace), [workspace]);

  async function refreshRedeemPage(options?: { cancelled?: () => boolean }): Promise<void> {
    setStatus(loadingStatus);

    try {
      const data = await loadCreditsPageData();
      if (options?.cancelled?.()) {
        return;
      }

      setSummary(data.summary);
      setBillingEventSummary(data.billing_event_summary);
      setOrders(data.orders);
      setMarketingCodes(data.marketing_codes);
      setMarketingRedemptions(data.marketing_redemptions);
      setRewardHistory(data.marketing_reward_history);
      setStatus(syncedStatus);
    } catch (error) {
      if (!options?.cancelled?.()) {
        setStatus(portalErrorMessage(error));
      }
    }
  }

  useEffect(() => {
    let cancelled = false;

    void refreshRedeemPage({
      cancelled: () => cancelled,
    });

    return () => {
      cancelled = true;
    };
  }, [loadingStatus, syncedStatus]);

  const rewardHistoryRows = useMemo(
    () =>
      rewardHistory
        .slice()
        .sort((left, right) => right.redemption.redeemed_at_ms - left.redemption.redeemed_at_ms),
    [rewardHistory],
  );

  const walletItems = useMemo(
    () =>
      marketingCodes.items
        .slice()
        .sort(
          (left, right) =>
            Number(right.code.status === 'available') - Number(left.code.status === 'available')
            || left.code.code_value.localeCompare(right.code.code_value),
        ),
    [marketingCodes.items],
  );
  const availableWalletItems = useMemo(
    () => walletItems.filter((item) => item.code.status === 'available'),
    [walletItems],
  );

  const totalItems = rewardHistoryRows.length;
  const totalPages = Math.max(1, Math.ceil(totalItems / PAGE_SIZE));
  const currentPage = clampPage(page, totalPages);
  const paginatedRewardHistory = useMemo(() => {
    const start = (currentPage - 1) * PAGE_SIZE;
    return rewardHistoryRows.slice(start, start + PAGE_SIZE);
  }, [currentPage, rewardHistoryRows]);

  useEffect(() => {
    setPage((current) => clampPage(current, totalPages));
  }, [totalPages]);

  const pageStatus = status !== syncedStatus ? status : '';
  const showingStart = totalItems === 0 ? 0 : (currentPage - 1) * PAGE_SIZE + 1;
  const showingEnd = totalItems === 0 ? 0 : Math.min(currentPage * PAGE_SIZE, totalItems);
  const availableCouponCount = marketingCodes.summary.available_count;
  const redeemedRewardCount = marketingRedemptions.summary.redeemed_count;
  const selectedWalletCouponCode = availableWalletItems.some(
    (item) => item.code.code_value === couponCode,
  )
    ? couponCode
    : '';
  const financeProjection = useMemo(
    () => buildFinanceProjection(summary, orders, billingEventSummary),
    [summary, orders, billingEventSummary],
  );

  async function handleCopyInvite(kind: 'link' | 'code', value: string): Promise<void> {
    const copied = await copyText(value);

    if (!copied) {
      setInviteStatus(t('Clipboard copy is unavailable in this browser context.'));
      return;
    }

    setInviteStatus(
      kind === 'link'
        ? t('Invite link copied to clipboard.')
        : t('Invite code copied to clipboard.'),
    );
  }

  async function handleRedeemSubmit(event: FormEvent<HTMLFormElement>): Promise<void> {
    event.preventDefault();
    const normalizedCode = couponCode.trim().toUpperCase();
    if (!normalizedCode) {
      setRedeemStatus(defaultRedeemStatus);
      return;
    }

    setRedeemLoading(true);
    setRedeemStatus(t('Checking {code} against marketing policy...', { code: normalizedCode }));

    try {
      const validation = await validateCreditsCouponCode({
        coupon_code: normalizedCode,
      });
      const selfServiceDecision = buildPortalCouponSelfServiceDecision(validation);
      setRedeemStatus(selfServiceDecision.message);

      if (selfServiceDecision.flow !== 'grant_self_service') {
        return;
      }

      await redeemCreditsCouponCode({
        coupon_code: normalizedCode,
      });
      setCouponCode('');
      setRedeemStatus(
        t('{code} was redeemed and the reward history was refreshed.', {
          code: normalizedCode,
        }),
      );
      setPage(1);
      await refreshRedeemPage();
    } catch (error) {
      setRedeemStatus(portalErrorMessage(error));
    } finally {
      setRedeemLoading(false);
    }
  }

  return (
    <div className="space-y-4" data-slot="portal-redeem-page">
      <div className="grid gap-4 xl:grid-cols-[1.24fr_0.76fr]">
        <Card
          className="border-primary-500/15 bg-primary-500/8 shadow-none dark:border-primary-500/20 dark:bg-primary-500/10"
          data-slot="portal-redeem-entry-card"
        >
          <CardContent className="space-y-5 p-5 sm:p-6">
            <div
              className="flex flex-col gap-5 rounded-[28px] border border-white/80 bg-white/92 p-5 dark:border-zinc-900/80 dark:bg-zinc-950/80 xl:flex-row xl:items-start xl:justify-between"
              data-slot="portal-redeem-entry-hero"
            >
              <div className="max-w-[36rem] space-y-4">
                <span className="inline-flex items-center rounded-full border border-primary-500/15 bg-primary-500/10 px-3 py-1 text-[11px] font-semibold uppercase tracking-[0.18em] text-primary-700 dark:border-primary-500/20 dark:text-primary-200">
                  {t('Redeem now')}
                </span>
                <div className="space-y-2">
                  <h1 className="text-[1.75rem] font-semibold leading-tight text-zinc-950 dark:text-zinc-50">
                    {t('Redeem code')}
                  </h1>
                  <p className="max-w-[34rem] text-sm leading-6 text-zinc-600 dark:text-zinc-300">
                    {redeemStatus}
                  </p>
                </div>
              </div>

              <div className="grid gap-3 sm:grid-cols-3 xl:min-w-[18rem] xl:grid-cols-1">
                <div className="rounded-[24px] border border-primary-500/15 bg-primary-500/6 px-4 py-4 dark:border-primary-500/20 dark:bg-primary-500/8">
                  <span className="text-xs font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                    {t('Balance')}
                  </span>
                  <strong className="mt-2 block text-2xl font-semibold text-zinc-950 dark:text-zinc-50">
                    {formatBalanceValue(summary, t)}
                  </strong>
                </div>
                <div className="rounded-[24px] border border-primary-500/15 bg-primary-500/6 px-4 py-4 dark:border-primary-500/20 dark:bg-primary-500/8">
                  <span className="text-xs font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                    {t('Eligible offers')}
                  </span>
                  <strong className="mt-2 block text-2xl font-semibold text-zinc-950 dark:text-zinc-50">
                    {formatUnits(availableCouponCount)}
                  </strong>
                </div>
                <div className="rounded-[24px] border border-primary-500/15 bg-primary-500/6 px-4 py-4 dark:border-primary-500/20 dark:bg-primary-500/8">
                  <span className="text-xs font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                    {t('Redeemed rewards')}
                  </span>
                  <strong className="mt-2 block text-2xl font-semibold text-zinc-950 dark:text-zinc-50">
                    {formatUnits(redeemedRewardCount)}
                  </strong>
                </div>
              </div>
            </div>

            <form className="space-y-4" onSubmit={(event) => void handleRedeemSubmit(event)}>
              <div className="space-y-2">
                <span className="text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                  {t('Redeem code')}
                </span>
                <div className="grid gap-3 lg:grid-cols-[minmax(0,15rem)_minmax(0,1fr)_auto]">
                  <Select
                    disabled={!availableWalletItems.length}
                    onValueChange={(value: string) => {
                      setCouponCode(value);
                      setRedeemStatus(defaultRedeemStatus);
                    }}
                    value={selectedWalletCouponCode}
                  >
                    <SelectTrigger className="h-12 rounded-2xl border-zinc-200 bg-white shadow-none dark:border-zinc-800 dark:bg-zinc-950">
                      <SelectValue placeholder={t('Available coupons')} />
                    </SelectTrigger>
                    <SelectContent>
                      {availableWalletItems.map((item) => (
                        <SelectItem key={item.code.coupon_code_id} value={item.code.code_value}>
                          {item.code.code_value}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                  <Input
                    autoComplete="off"
                    className="h-12 rounded-2xl border-zinc-200 bg-white shadow-none dark:border-zinc-800 dark:bg-zinc-950"
                    onChange={(event: ChangeEvent<HTMLInputElement>) => {
                      setCouponCode(event.target.value.toUpperCase());
                      setRedeemStatus(defaultRedeemStatus);
                    }}
                    placeholder={t('Enter coupon code')}
                    value={couponCode}
                  />
                  <Button
                    className="h-12 min-w-[10.5rem] rounded-2xl px-5 text-sm font-semibold shadow-sm"
                    disabled={redeemLoading || !couponCode.trim()}
                    type="submit"
                    >
                    {redeemLoading ? t('Redeeming...') : t('Redeem')}
                  </Button>
                </div>
                <p className="text-xs leading-5 text-zinc-500 dark:text-zinc-400">
                  {t('Apply during checkout if the code discounts a plan or recharge purchase.')}
                </p>
              </div>
            </form>
          </CardContent>
        </Card>

        <Card
          className="border-zinc-200 bg-white shadow-none dark:border-zinc-800 dark:bg-zinc-950"
          data-slot="portal-redeem-invite-card"
        >
          <CardHeader className="pb-4">
            <CardTitle>{t('Invite rewards')}</CardTitle>
            <CardDescription>{inviteStatus}</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid gap-3">
              <div className="rounded-2xl border border-zinc-200 bg-zinc-50/80 px-4 py-4 dark:border-zinc-800 dark:bg-zinc-900/60">
                <span className="text-xs font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                  {t('Invite code')}
                </span>
                <strong className="mt-2 block text-xl font-semibold tracking-[0.08em] text-zinc-950 dark:text-zinc-50">
                  {inviteProgram.code}
                </strong>
              </div>
              <div className="rounded-2xl border border-zinc-200 bg-zinc-50/80 px-4 py-4 dark:border-zinc-800 dark:bg-zinc-900/60">
                <span className="text-xs font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                  {t('Invite link')}
                </span>
                <strong className="mt-2 block break-all text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                  {inviteProgram.link}
                </strong>
              </div>
            </div>

            <div className="flex flex-col gap-3 sm:flex-row">
              <Button
                onClick={() => void handleCopyInvite('code', inviteProgram.code)}
                type="button"
                variant="secondary"
              >
                {t('Copy invite code')}
              </Button>
              <Button
                onClick={() => void handleCopyInvite('link', inviteProgram.link)}
                type="button"
                variant="secondary"
              >
                {t('Copy invite link')}
              </Button>
            </div>
          </CardContent>
        </Card>
      </div>

      <Card
        className="border-zinc-200 bg-white shadow-none dark:border-zinc-800 dark:bg-zinc-950"
        data-slot="portal-redeem-decision-support"
      >
        <CardContent className="space-y-4 p-5">
          <div className="space-y-1">
            <h2 className="text-lg font-semibold text-zinc-950 dark:text-zinc-50">
              {t('Redeem decision support')}
            </h2>
            <p className="text-sm text-zinc-500 dark:text-zinc-400">
              {t('Redeem posture combines coupon runway, accounting mode mix, and workload shape before another code is applied.')}
            </p>
          </div>

          <div className="grid gap-4 xl:grid-cols-3">
            <div className="rounded-3xl border border-zinc-200 bg-zinc-50/80 p-4 dark:border-zinc-800 dark:bg-zinc-900/60">
              <span className="text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                {t('Redemption coverage')}
              </span>
              <strong className="mt-2 block text-xl font-semibold text-zinc-950 dark:text-zinc-50">
                {redemptionCoverageHeadline(financeProjection, t)}
              </strong>
              <p className="mt-2 text-sm leading-6 text-zinc-500 dark:text-zinc-400">
                {redemptionCoverageDetail(financeProjection, t)}
              </p>
              <div className="mt-4 grid gap-3 sm:grid-cols-2">
                <div className="rounded-2xl border border-zinc-200 bg-white px-3 py-3 dark:border-zinc-800 dark:bg-zinc-950">
                  <span className="text-[11px] font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                    {t('Granted units')}
                  </span>
                  <strong className="mt-2 block text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                    {formatUnits(financeProjection?.redemption_coverage.granted_units ?? 0)}
                  </strong>
                </div>
                <div className="rounded-2xl border border-zinc-200 bg-white px-3 py-3 dark:border-zinc-800 dark:bg-zinc-950">
                  <span className="text-[11px] font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                    {t('Bonus units')}
                  </span>
                  <strong className="mt-2 block text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                    {formatUnits(financeProjection?.redemption_coverage.bonus_units ?? 0)}
                  </strong>
                </div>
              </div>
            </div>

            <div className="rounded-3xl border border-zinc-200 bg-zinc-50/80 p-4 dark:border-zinc-800 dark:bg-zinc-900/60">
              <span className="text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500 dark:text-zinc-400">
                {t('Leading accounting mode')}
              </span>
              <strong className="mt-2 block text-xl font-semibold text-zinc-950 dark:text-zinc-50">
                {accountingModeLabel(financeProjection?.leading_accounting_mode?.accounting_mode, t)}
              </strong>
              <p className="mt-2 text-sm leading-6 text-zinc-500 dark:text-zinc-400">
                {accountingModeDetail(financeProjection?.leading_accounting_mode ?? null, t)}
              </p>
              <div className="mt-4 rounded-2xl border border-zinc-200 bg-white px-3 py-3 dark:border-zinc-800 dark:bg-zinc-950">
                <span className="text-[11px] font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                  {t('Customer charge')}
                </span>
                <strong className="mt-2 block text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                  {financeProjection?.leading_accounting_mode
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
                {capabilityLabel(financeProjection?.leading_capability?.capability, t)}
              </strong>
              <p className="mt-2 text-sm leading-6 text-zinc-500 dark:text-zinc-400">
                {capabilityDetail(financeProjection?.leading_capability ?? null, t)}
              </p>
              <div className="mt-4 rounded-2xl border border-zinc-200 bg-white px-3 py-3 dark:border-zinc-800 dark:bg-zinc-950">
                <span className="text-[11px] font-semibold uppercase tracking-[0.16em] text-zinc-500 dark:text-zinc-400">
                  {t('Customer charge')}
                </span>
                <strong className="mt-2 block text-sm font-semibold text-zinc-950 dark:text-zinc-50">
                  {financeProjection?.leading_capability
                    ? formatCurrency(financeProjection.leading_capability.total_customer_charge)
                    : t('n/a')}
                </strong>
              </div>
            </div>
          </div>

          <div
            className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4"
            data-slot="portal-redeem-multimodal-demand"
          >
            {[
              { label: t('Images'), value: formatUnits(financeProjection?.multimodal_totals.image_count ?? 0) },
              { label: t('Audio'), value: formatUnits(financeProjection?.multimodal_totals.audio_seconds ?? 0) },
              { label: t('Video'), value: formatUnits(financeProjection?.multimodal_totals.video_seconds ?? 0) },
              { label: t('Music'), value: formatUnits(financeProjection?.multimodal_totals.music_seconds ?? 0) },
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
            {t('Multimodal demand keeps image, audio, video, and music traffic visible before another coupon is applied.')}
          </p>
        </CardContent>
      </Card>

      {pageStatus ? (
        <div
          className="rounded-2xl border border-zinc-200 bg-zinc-50/85 px-4 py-3 text-sm text-zinc-600 dark:border-zinc-800 dark:bg-zinc-900/60 dark:text-zinc-300"
          data-slot="portal-redeem-feedback"
          role="status"
        >
          {pageStatus}
        </div>
      ) : null}

      <div className="grid gap-4 xl:grid-cols-[0.92fr_1.08fr]">
        <div className="space-y-3">
          <div className="space-y-1">
            <h2 className="text-lg font-semibold text-zinc-950 dark:text-zinc-50">
              {t('My coupons')}
            </h2>
            <p className="text-sm text-zinc-500 dark:text-zinc-400">
              {t('Wallet inventory shows which codes are still available for self-service redemption and which have already moved into reservation or redemption states.')}
            </p>
          </div>

          <DataTable
            className="rounded-[28px] border border-zinc-200 bg-white p-4 dark:border-zinc-800 dark:bg-zinc-950"
            columns={[
              {
                id: 'code',
                header: t('Coupon'),
                cell: (row: PortalMarketingCodeItem) => (
                  <div className="space-y-1">
                    <strong>{row.code.code_value}</strong>
                    <p className="text-xs leading-5 text-zinc-500 dark:text-zinc-400">
                      {`${row.template.display_name} · ${couponEffectLabel(row, t)} · ${couponApplicabilityLabel(row, t)}`}
                    </p>
                  </div>
                ),
              },
              {
                id: 'status',
                header: t('Status'),
                cell: (row: PortalMarketingCodeItem) => (
                  <Badge variant={couponCodeStatusVariant(row.code.status)}>
                    {couponCodeStatusLabel(row.code.status, t)}
                  </Badge>
                ),
              },
              {
                id: 'reservation',
                header: t('Reservation'),
                cell: (row: PortalMarketingCodeItem) => (
                  <Badge
                    variant={reservationStatusVariant(
                      row.latest_reservation?.reservation_status ?? null,
                    )}
                  >
                    {reservationStatusLabel(
                      row.latest_reservation?.reservation_status ?? null,
                      t,
                    )}
                  </Badge>
                ),
              },
              {
                id: 'redemption',
                header: t('Reward'),
                cell: (row: PortalMarketingCodeItem) => (
                  <Badge
                    variant={redemptionStatusVariant(
                      row.latest_redemption?.redemption_status ?? null,
                    )}
                  >
                    {redemptionStatusLabel(
                      row.latest_redemption?.redemption_status ?? null,
                      t,
                    )}
                  </Badge>
                ),
              },
              {
                id: 'expires',
                header: t('Expires'),
                cell: (row: PortalMarketingCodeItem) =>
                  row.code.expires_at_ms ? formatDateTime(row.code.expires_at_ms) : t('No expiry'),
              },
            ]}
            data-slot="portal-redeem-wallet-table"
            emptyState={(
              <div className="mx-auto flex max-w-[28rem] flex-col items-center gap-2 text-center">
                <strong className="text-base font-semibold text-zinc-950 dark:text-zinc-50">
                  {t('No wallet coupons yet')}
                </strong>
                <p className="text-sm text-zinc-500 dark:text-zinc-400">
                  {t('Claimed and reserved marketing codes will appear here as soon as they are issued to the current portal identity.')}
                </p>
              </div>
            )}
            footer={(
              <div className="grid gap-3 rounded-2xl border border-zinc-200 bg-zinc-50/80 px-4 py-3 text-sm text-zinc-600 dark:border-zinc-800 dark:bg-zinc-900/50 dark:text-zinc-300 sm:grid-cols-3">
                <span>{t('Available: {count}', { count: marketingCodes.summary.available_count })}</span>
                <span>{t('Reserved: {count}', { count: marketingCodes.summary.reserved_count })}</span>
                <span>{t('Redeemed: {count}', { count: marketingCodes.summary.redeemed_count })}</span>
              </div>
            )}
            getRowId={(row: PortalMarketingCodeItem) => row.code.coupon_code_id}
            rows={walletItems}
          />
        </div>

        <div className="space-y-3" data-slot="portal-redeem-history-table">
          <div className="space-y-1">
            <h2 className="text-lg font-semibold text-zinc-950 dark:text-zinc-50">
              {t('Reward history')}
            </h2>
            <p className="text-sm text-zinc-500 dark:text-zinc-400">
              <span>{syncedStatus}</span>{' '}
              <span>
                {t('{redeemed} redeemed / {rolledBack} rolled back.', {
                  redeemed: marketingRedemptions.summary.redeemed_count,
                  rolledBack: marketingRedemptions.summary.rolled_back_count,
                })}
              </span>
            </p>
          </div>

          <DataTable
            className="rounded-[28px] border border-zinc-200 bg-white p-4 dark:border-zinc-800 dark:bg-zinc-950"
            columns={[
              {
                id: 'recorded',
                header: t('Recorded'),
                cell: (row: PortalMarketingRewardHistoryItem) =>
                  formatDateTime(row.redemption.redeemed_at_ms),
              },
              {
                id: 'code',
                header: t('Coupon'),
                cell: (row: PortalMarketingRewardHistoryItem) => (
                  <div className="space-y-1">
                    <strong>{row.code.code_value}</strong>
                    <p className="text-xs leading-5 text-zinc-500 dark:text-zinc-400">
                      {`${row.template.display_name} · ${couponEffectLabel(row, t)} · ${couponApplicabilityLabel(row, t)}`}
                    </p>
                  </div>
                ),
              },
              {
                id: 'status',
                header: t('Status'),
                cell: (row: PortalMarketingRewardHistoryItem) => (
                  <Badge variant={redemptionStatusVariant(row.redemption.redemption_status)}>
                    {redemptionStatusLabel(row.redemption.redemption_status, t)}
                  </Badge>
                ),
              },
              {
                id: 'rollback',
                header: t('Rollback'),
                cell: (row: PortalMarketingRewardHistoryItem) => rewardHistoryRollbackDetail(row, t),
              },
              {
                id: 'arrival',
                header: t('Arrival'),
                cell: (row: PortalMarketingRewardHistoryItem) =>
                  rewardHistoryAccountArrivalDetail(row, t),
              },
              {
                id: 'evidence',
                header: t('Evidence'),
                cell: (row: PortalMarketingRewardHistoryItem) =>
                  row.redemption.order_id?.trim()
                  || row.redemption.payment_event_id?.trim()
                  || row.redemption.coupon_redemption_id,
              },
            ]}
            data-slot="portal-redeem-reward-history-table"
            emptyState={(
              <div className="mx-auto flex max-w-[32rem] flex-col items-center gap-2 text-center">
                <strong className="text-base font-semibold text-zinc-950 dark:text-zinc-50">
                  {t('No reward history yet')}
                </strong>
                <p className="text-sm text-zinc-500 dark:text-zinc-400">
                  {t('Confirmed coupon rewards and rollbacks will appear here after the first successful marketing redemption closes the loop.')}
                </p>
              </div>
            )}
            footer={(
              <div
                className="flex flex-col gap-3 rounded-2xl border border-zinc-200 bg-zinc-50/80 px-4 py-3 dark:border-zinc-800 dark:bg-zinc-900/50 lg:flex-row lg:items-center lg:justify-between"
                data-slot="portal-redeem-pagination"
              >
                <div className="text-sm text-zinc-600 dark:text-zinc-300">
                  {t('Showing {start}-{end} of {total} records', {
                    end: showingEnd,
                    start: showingStart,
                    total: totalItems,
                  })}
                </div>
                <div className="flex flex-wrap items-center gap-2">
                  <Button
                    disabled={currentPage <= 1}
                    onClick={() =>
                      startTransition(() => {
                        setPage((current) => clampPage(current - 1, totalPages));
                      })}
                    variant="secondary"
                  >
                    {t('Previous page')}
                  </Button>
                  <span className="min-w-[8rem] text-center text-sm font-medium text-zinc-600 dark:text-zinc-300">
                    {t('Page {page} of {total}', {
                      page: currentPage,
                      total: totalPages,
                    })}
                  </span>
                  <Button
                    disabled={currentPage >= totalPages}
                    onClick={() =>
                      startTransition(() => {
                        setPage((current) => clampPage(current + 1, totalPages));
                      })}
                    variant="secondary"
                  >
                    {t('Next page')}
                  </Button>
                </div>
              </div>
            )}
            getRowId={(row: PortalMarketingRewardHistoryItem) => row.redemption.coupon_redemption_id}
            rows={paginatedRewardHistory}
          />
        </div>
      </div>
    </div>
  );
}
